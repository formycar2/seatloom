#!/usr/bin/env bash
# test-prepare-verification-build-dry-run.sh — 证明 --dry-run 只算不做。
#
# 为什么需要这个测试(2026-11-06):
#   操作者批准的是「先看会动到什么,再决定装不装」。而 prepare-verification-build.sh
#   的每一步都不可逆:pkill、rm -rf、cp -R、tauri build、重启。一个说自己在干跑
#   其实动了手的脚本,比没有干跑更糟 —— 它会让人以为已经看过了。
#
# 所以这个测试的承重命题不是「它打印了东西」,而是:
#   **干跑期间,那些命令一次都没有执行。**
#
# 做法:在一个临时 fixture 里造一个假仓库、假 tauri、假 launch 器、假 pkill/git,
# 干跑脚本,然后断言 fixture 里没有出现任何「执行过」的痕迹。探针全部放在
# fixture 里,**绝不碰主检出/主索引/在跑的应用**。
#
# 每一行都必须能失败。最后一条是变异:把干跑守卫删掉,这个测试必须红。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO/scripts/prepare-verification-build.sh"
[ -f "$SCRIPT" ] || { echo "❌ 找不到 $SCRIPT"; exit 1; }

FIX="$(mktemp -d)/dryrun-fixture"
FAIL=0
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }

cleanup(){ rm -rf "$(dirname "$FIX")"; }
trap cleanup EXIT

# ── 造 fixture ────────────────────────────────────────────────────────────
# 形状和真实仓库一致到脚本需要的程度:
#   $FIX/repo/scripts/prepare-verification-build.sh   脚本本体(拷贝)
#   $FIX/repo/ui/dist                                 (不存在 —— 干跑不该建)
#   $FIX/repo/node_modules/.bin/tauri                 假可执行文件,记录自己被调用
#   $FIX/repo/scripts/launch-seatloom.sh              假启动器,记录自己被调用
#   $FIX/worktrees/seat/                              ？ 不需要:分支就是当前检出的
mkdir -p "$FIX/repo/scripts" "$FIX/repo/ui" "$FIX/repo/node_modules/.bin" "$FIX/bin"
cp "$SCRIPT" "$FIX/repo/scripts/prepare-verification-build.sh"

cat > "$FIX/repo/node_modules/.bin/tauri" <<EOF
#!/usr/bin/env bash
echo "TAURI-RAN \$*" >> "$FIX/executed.log"
echo "fake tauri build"
EOF
chmod +x "$FIX/repo/node_modules/.bin/tauri"

cat > "$FIX/repo/scripts/launch-seatloom.sh" <<EOF
#!/usr/bin/env bash
echo "LAUNCH-RAN" >> "$FIX/executed.log"
echo "fake launch"
EOF
chmod +x "$FIX/repo/scripts/launch-seatloom.sh"

# pnpm 也要有 —— 干跑不该调它,但真实路径会。造一个会记录的假货,
# 这样「干跑没跑 pnpm」才是一个可断言的命题,而不是一句注释。
cat > "$FIX/bin/pnpm" <<EOF
#!/usr/bin/env bash
echo "PNPM-RAN \$*" >> "$FIX/executed.log"
echo "fake pnpm"
EOF
chmod +x "$FIX/bin/pnpm"

# git 的探测(rev-parse / status / branch)是真 git,但要在 fixture 里有个仓库。
# 不 source 主检出的 .git,所以这是独立的。
( cd "$FIX/repo" && git init -q -b packet/fixture/test >/dev/null 2>&1 \
  && git config user.email t@t.local && git config user.name t \
  && echo hi > README.md && git add . && git commit -qm base ) >/dev/null 2>&1

run_dry(){
  ( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
      bash scripts/prepare-verification-build.sh --dry-run packet/fixture/test MUST_HAVE_ME ) \
      2>&1
}

echo "── 1. 干跑要能跑完,并且退出码是 0 ──"
OUT="$(run_dry)"; RC=$?
if [ "$RC" = "0" ]; then pass "干跑退出 0"; else die "干跑退出 $RC"; printf '%s\n' "$OUT" | tail -20; fi

echo "── 2. 干跑期间,一个副作用命令都没有执行 ──"
# 这是整个测试的承重命题。
if [ -f "$FIX/executed.log" ]; then
  die "干跑执行了东西:"; cat "$FIX/executed.log" | sed 's/^/      /'
else
  pass "tauri / pnpm / launch 一次都没被调用"
fi
# 应用没有被删、没有被复制、没有被装。
if [ -e "$FIX/repo/ui/dist" ]; then
  die "干跑构建了前端(ui/dist 出现了)"
else
  pass "没有构建前端"
fi
if [ -e "$FIX/repo/target" ]; then
  die "干跑构建了 release(target/ 出现了)"
else
  pass "没有构建 release"
fi

echo "── 3. 干跑把该说的都说了 ──"
for want in \
  "packet/fixture/test" \
  "干跑" \
  "/Applications/SeatLoom.app" \
  "pnpm build" \
  "tauri build" \
  "需要人批准" \
  "MUST_HAVE_ME"
do
  if printf '%s\n' "$OUT" | grep -qF "$want"; then
    pass "打印了:$want"
  else
    die "没打印:$want"
  fi
done

echo ""
echo "── 3b. 工作区脏了必须说出来,并且说出是哪些路径 ──"
# Flux 的 FAIL:这一条原来是空的。他设 DIRTY="" 变异,套件全绿 —— 因为 fixture
# 从来造的是一个**干净**仓库,于是「工作区干净」那一条分支之外的三句话
# (列出改动、超过 20 条给截断标记、警告这个构建不只是那个提交)
# 从来没有被任何测试观察到。而脏工作区警告是整份报告里对验收最有用的一句:
# 它告诉操作者「你要验的东西不是卡上写的那个提交」。
#
# 所以这里造一个脏工作区,逐条断言。路径是**真实写出来的文件**,不是 grep 自己
# 的字符串 —— 那样断言的是字典,不是行为。
# 🔴 必须改**已跟踪**的文件,不是新建一个未跟踪文件。脚本用 grep -v '^??' 把
#    未跟踪文件排掉了 —— 那是故意的:一个没 git add 的文件本来就不会进构建,
#    不是这条警告要防的风险。第一版造的是未跟踪文件,于是四条断言全红,
#    看起来像脚本坏了,其实是 fixture 造错了状态(实测踩到)。
printf 'dirty change\n' >> "$FIX/repo/README.md"
OUT="$(run_dry)"; RC=$?
# 脏工作区是**阻碍**(BLOCKERS=1),所以干跑必须非零退出 —— 不是 0。
# 第一版这里断言退出 0,四条内容断言全过而退出码那条红,看起来像脚本坏了;
# 其实是脚本对(先提交再装),我的断言错。
[ "$RC" != "0" ] && pass "脏工作区时干跑非零退出(它是阻碍)" || die "脏工作区却退出 0 —— 那等于允许装一个不等于该提交的东西"
printf '%s\n' "$OUT" | grep -qF "README.md" \
  && pass "列出了未提交的路径" \
  || die "没列出脏路径 —— 操作者看不出构建里多了什么"
printf '%s\n' "$OUT" | grep -q "不只是提交" \
  && pass "警告了构建不只是那个提交" \
  || die "没说构建会包含未提交的改动 —— 这正是最有用的一句"
printf '%s\n' "$OUT" | grep -q "工作区干净" \
  && die "脏工作区却打印了「工作区干净」" \
  || pass "没有谎称干净"
git -C "$FIX/repo" checkout -q -- . 2>/dev/null || true

echo ""
echo "── 3c. 超过 20 条改动要给截断标记,不能只显示前 20 条就闭嘴 ──"
# 同样:已提交再改,才是 porcelain 里的 ' M' 而不是 '??'。
for i in $(seq 1 25); do
  printf 'original %s\n' "$i" > "$FIX/repo/dirty_$i.txt"
done
( cd "$FIX/repo" && git add . && git commit -qm "25 tracked files" ) >/dev/null 2>&1
for i in $(seq 1 25); do
  printf 'change %s\n' "$i" > "$FIX/repo/dirty_$i.txt"
done
OUT="$(run_dry)"; RC=$?
[ "$RC" != "0" ] && pass "脏工作区非零退出" || die "退出 0"
printf '%s\n' "$OUT" | grep -qF "dirty_1.txt" && pass "显示了改动" || die "一条改动都没显示"
printf '%s\n' "$OUT" | grep -q "还有更多" \
  && pass "超过 20 条时给了截断标记" \
  || die "超过 20 条却没有任何截断标记 —— 看起来像是全部改动只有这些"
for i in $(seq 1 25); do rm -f "$FIX/repo/dirty_$i.txt"; done
( cd "$FIX/repo" && git add -A && git commit -qm "fixture cleanup" ) >/dev/null 2>&1
# 恢复到干净状态,后面的用例假设工作区干净。
git -C "$FIX/repo" checkout -q -- . 2>/dev/null || true
OUT="$(run_dry)"
printf '%s\n' "$OUT" | grep -q "工作区干净" \
  && pass "清理后回到「工作区干净」" \
  || die "清理后仍说脏 —— fixture 没复原,后面的用例会被污染"

echo "── 4. 未知参数必须非零退出 ──"
for bad in "--nonsense" "--dry-run --nonsense"; do
  # shellcheck disable=SC2086
  ( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
      bash scripts/prepare-verification-build.sh $bad packet/fixture/test ) >/dev/null 2>&1
  if [ "$?" != "0" ]; then pass "拒绝:$bad"; else die "接受了未知参数:$bad"; fi
done

echo "── 5. 第二个 --dry-run 必须非零退出(不能静默忽略)──"
( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
    bash scripts/prepare-verification-build.sh --dry-run --dry-run packet/fixture/test ) >/dev/null 2>&1
if [ "$?" != "0" ]; then pass "拒绝重复的 --dry-run"; else die "静默接受了第二个 --dry-run"; fi

echo "── 6. 缺安全前提时,不得打印「可以装」──"
# 把 launch 器拿走:那是操作者身份的来源,缺了它装上去也点不了合并。
mv "$FIX/repo/scripts/launch-seatloom.sh" "$FIX/launch.bak"
OUT2="$(run_dry)"; RC2=$?
mv "$FIX/launch.bak" "$FIX/repo/scripts/launch-seatloom.sh"
if [ "$RC2" != "0" ]; then pass "缺 launch 器时退出非零"; else die "缺 launch 器却退出 0"; fi
if printf '%s\n' "$OUT2" | grep -q "前提都满足"; then
  die "缺 launch 器却打印「前提都满足」—— 那是在劝人装一个点不了合并的程序"
else
  pass "缺 launch 器时没有打印「前提都满足」"
fi

echo "── 7. 变异:删掉干跑守卫,这个测试必须红 ──"
# 把「if [ \"\$DRY\" = \"1\" 」改成永假,于是 --dry-run 会走真实路径。
# 真实路径会调 tauri/pnpm,于是第 2 条会失败 —— 证明第 2 条真的在守东西。
MUT="$FIX/repo/scripts/prepare-verification-build.sh"
cp "$MUT" "$FIX/orig.sh"
sed -i.bak 's/^if \[ "\$DRY" = "1" \]; then$/if [ "$DRY" = "never" ]; then/' "$MUT"
rm -f "$FIX/executed.log"
OUTM="$(run_dry)"; RCM=$?
if [ -s "$FIX/executed.log" ] || [ -e "$FIX/repo/ui/dist" ]; then
  pass "变异后干跑真的动手了(第 2 条会抓住它)"
else
  die "变异后仍然没有任何执行痕迹 —— 第 2 条守的是空的"
fi
# 还原
cp "$FIX/orig.sh" "$MUT"

echo ""
echo "── 8. git status 读失败时,必须既不说干净也不能说可以装 ──"
# 这一条来自 ev-dr3OFuoVhsMO3JYi7mHfK 实测到的 fail-open。
#
# 原来的形状:DIRTY="$(git status --porcelain 2>/dev/null | grep -v '^??' || true)"
# git 退出 128 时,2>/dev/null 把证据扔掉,|| true 让管道算成功,DIRTY 成空串,
# [ -n ] 为假 → 走 else 打印「工作区干净」,而那个分支从不设 BLOCKERS → 退出 0。
#
# 注入一个只在 status --porcelain 上失败的 git(别的子命令全部照常 exec 真 git),
# 那是权限坏了/索引坏了/磁盘出错的形状 —— 一个仓库还在、但读不出状态。
cat > "$FIX/bin/git" <<EOF
#!/usr/bin/env bash
for a in "\$@"; do
  if [ "\$a" = "status" ]; then
    echo "fatal: unable to read the index" >&2
    exit 128
  fi
done
exec /usr/bin/git "\$@"
EOF
chmod +x "$FIX/bin/git"
OUT="$( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
  bash scripts/prepare-verification-build.sh --dry-run packet/fixture/test MUST_HAVE_ME 2>&1 )"
RC=$?
[ "$RC" != "0" ] && pass "读不出工作区状态时干跑非零退出" \
  || die "读不出工作区状态却退出 0 —— 它认为可以装,而这份构建跟那个提交的关系根本没测过"
printf '%s\n' "$OUT" | grep -q "工作区干净" \
  && die "读失败被说成「工作区干净」—— 把没测过说成测过了" \
  || pass "没有把读失败说成干净"
printf '%s\n' "$OUT" | grep -q "前提都满足" \
  && die "读失败却打印「前提都满足」—— 那是在劝人装一个关系未定的构建" \
  || pass "读失败时没有打印「前提都满足」"
# 三态,不是两态:读不出来必须自己有个名字,不能落到 [ -n "$DIRTY" ] 这个二值里。
printf '%s\n' "$OUT" | grep -q "读不出工作区状态" \
  && pass "明确说了读不出工作区状态(第三种状态有自己的名字)" \
  || die "没说读不出工作区状态 —— 一个没有名字的状态最后一定会被算成干净"
# git 自己那句话必须出现在输出里。它才是「干净」和「读不出来」的区别所在。
printf '%s\n' "$OUT" | grep -q "unable to read the index" \
  && pass "把 git 的报错原话带出来了(证据没被 2>/dev/null 丢掉)" \
  || die "没带出 git 的报错 —— 那操作者还是分不清是干净还是读不出来"
# 退出码本身要说出来,不能只说「出错了」。
printf '%s\n' "$OUT" | grep -q "128" \
  && pass "说出了 git 的退出码" \
  || die "没说退出码 —— 无法区分权限、索引、磁盘是哪一种"
rm -f "$FIX/bin/git"

echo "── 9. 变异:把失败吞掉,这个测试必须红 ──"
# 承重命题:上面第 8 条不是靠运气绿的。把修复退回第一版的形状
#(2>/dev/null + || true 混在一条管道里),第 8 条必须红。
# 变异必须**只**动那一行,并且验完逐字节还原 —— 变异还原不干净比没有变异更糟。
MUT="$FIX/repo/scripts/prepare-verification-build.sh"
cp "$MUT" "$FIX/mut-orig.sh"
python3 - "$MUT" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
# 变异:把退出码和 stderr 一起丢掉,回到第一版的形状。锚点必须是**真实存在的那几行**,
# 上一版的锚点在我加固 mktemp 分支之后就找不到了 —— 于是变异没发生,
# 「变异后仍然绿」其实是「变异根本没跑」。锚点失配必须让第 9 条失败,
# 不能让它静悄悄地变成一条永远通过的测试。
old = '''  GIT_STATUS_ERR="$(mktemp 2>/dev/null)" || GIT_STATUS_ERR=""
  if [ -n "$GIT_STATUS_ERR" ] && [ -w "$GIT_STATUS_ERR" ]; then
    DIRTY_RAW="$(git -C "$WT" status --porcelain 2>"$GIT_STATUS_ERR")"
    GIT_STATUS_RC=$?'''
new = '''  GIT_STATUS_ERR="$(mktemp 2>/dev/null)" || GIT_STATUS_ERR=""
  if [ -n "$GIT_STATUS_ERR" ] && [ -w "$GIT_STATUS_ERR" ]; then
    DIRTY_RAW="$(git -C "$WT" status --porcelain 2>/dev/null || true)"
    GIT_STATUS_RC=0'''
assert old in s, "变异锚点没找到 —— 第 9 条没有测到任何东西"
assert new not in s.replace(old, ''), "变异后跟变异前一样"
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
PYEOF
cat > "$FIX/bin/git" <<EOF
#!/usr/bin/env bash
for a in "\$@"; do
  if [ "\$a" = "status" ]; then echo "fatal: unable to read the index" >&2; exit 128; fi
done
exec /usr/bin/git "\$@"
EOF
chmod +x "$FIX/bin/git"
OUTM="$( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
  bash scripts/prepare-verification-build.sh --dry-run packet/fixture/test MUST_HAVE_ME 2>&1 )"
RCM=$?
if [ "$RCM" = "0" ] || printf '%s\n' "$OUTM" | grep -q "工作区干净"; then
  pass "变异后失败被吞了(退出 $RCM)—— 第 8 条真的在守东西"
else
  die "变异后第 8 条仍然绿 —— 它守的是空的"
fi
# 还原,并证明还原是逐字节的。
cp "$FIX/mut-orig.sh" "$MUT"
cmp -s "$MUT" "$FIX/mut-orig.sh" || die "变异没还原干净 —— 后面的用例会被污染"
rm -f "$FIX/bin/git" "$FIX/mut-orig.sh"

echo ""
if [ "$FAIL" = "0" ]; then
  echo "✅ 全部通过"
  exit 0
fi
echo "❌ 有失败项"
exit 1
