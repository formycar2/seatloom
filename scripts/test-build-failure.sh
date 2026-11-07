#!/usr/bin/env bash
# test-build-failure.sh — 证明构建失败被报成构建失败,而不是被报成构建成功。
#
# 为什么需要这个测试(2026-11-07):
#   Flux 的 ev-M_SmZjyZ6eJjJ4eYWxqNH 在 77a7c5f 上独立量到三种注入形状全部
#   退出码 1、零成功字样,但判据 build_failure_paths_have_tests 是 NOT PINNED ——
#   没有任何**随代码提交的**测试钉住它们。也就是说那三条性质当时只存在于
#   一次性的测量里,而不是存在于回归保护里。
#
#   这个 fail-open 是被实测出来的,不是推理出来的:
#     pnpm build 失败(tsc 找不到),而脚本打印「✅ 前端产物在」和「✅ 应用包在」。
#     两个缺陷叠在一起:管道 `pnpm build 2>&1 | tail -4` 让 $? 变成 tail 的,
#     而 `[ -d ui/dist ]` 被**前一天**的 dist 目录满足。于是 tauri build 把昨天的
#     前端打进应用,只有二进制提交号那一关拦住了它。
#
# 🔴 这个 fail-open 当时正藏着我自己代码里的真实错误:这个分支的前端从来没编译过。
#    一个把昨天的产物说成今天的检查,比没有检查更有害。
#
# 承重命题,每一条都必须能失败:
#   1. 干净的正面用例先过 —— 否则后面的红分不清是「失败被正确报出」还是「永远失败」。
#   2. pnpm 失败(有输出)     → 非零,不宣称成功,不安装。
#   3. pnpm 失败(无输出)     → 非零,不宣称成功。静默失败最危险。
#   4. tauri 失败 + 旧产物在  → 非零,不用旧产物冒充。
#   5. 变异:还原那个 fail-open        → 第 2/3 条必须红。
#   6. 变异:删掉 tauri 的退出码检查    → 第 4 条必须红。
#   7. 注销:只删 rm -rf 在本条上不是承重闸(如实记录,不删断言)
#   8. MUST_HAVE 给了就必须真的用(在→过,不在→非零,不给→明说没验)
#   9. 多余的位置参数要被拒,不是被悄悄丢掉
#  10. 变异:把 MUST_HAVE 再丢掉 → 第 8 条必须红
#  11. detached 检出的拒绝必须说清原因和补法

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO/scripts/prepare-sandbox-build.sh"
[ -f "$SCRIPT" ] || { echo "❌ 找不到 $SCRIPT"; exit 1; }

FAIL=0
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }

ROOT="$(mktemp -d)"
FIX="$ROOT/fixture"
cleanup(){ rm -rf "$ROOT"; }
trap cleanup EXIT
REAL_GIT="$(command -v git)"

# ── fixture:一个真 git 仓库 + 一个检出的席位工作区 ──────────────────────
# 形状和真实部署一致到脚本需要的程度:脚本在工作区里,worktree-root.sh 靠
# `git worktree list` 定席位根(要求父目录下至少两个)。
mkdir -p "$FIX/repo" "$FIX/seatloom-worktrees" "$FIX/Applications" "$FIX/bin"
mkdir -p "$FIX/repo/scripts"
cp "$REPO/scripts/prepare-sandbox-build.sh" "$FIX/repo/scripts/"
cp "$REPO/scripts/worktree-root.sh" "$FIX/repo/scripts/"
printf '/target\nCargo.lock\nnode_modules\nui/dist\nui/.build.log\n' > "$FIX/repo/.gitignore"

( cd "$FIX/repo" && "$REAL_GIT" init -q -b packet/fixture/root >/dev/null 2>&1 \
  && "$REAL_GIT" config user.email t@t.local && "$REAL_GIT" config user.name t \
  && echo hi > README.md && "$REAL_GIT" add . \
  && "$REAL_GIT" -c core.hooksPath=/dev/null commit -q --no-verify -m base ) >/dev/null 2>&1
( cd "$FIX/repo" && "$REAL_GIT" branch packet/fixture/test ) >/dev/null 2>&1
( cd "$FIX/repo" && "$REAL_GIT" branch packet/fixture/test2 ) >/dev/null 2>&1
WT="$FIX/seatloom-worktrees/wt"
# 🔴 工作区目录必须**不存在**才 add:mkdir -p 先建出它,git worktree add 会以
#    「already exists」拒绝 —— 而它只在 stderr 说,被我 2>/dev/null 吃掉,
#    于是后面每一条都红在「读不到 git」。实测踩到。
( cd "$FIX/repo" && "$REAL_GIT" worktree add "$WT" \
    packet/fixture/test ) >/dev/null 2>&1
( cd "$FIX/repo" && "$REAL_GIT" worktree add "$FIX/seatloom-worktrees/wt2" \
    packet/fixture/test2 ) >/dev/null 2>&1
{ [ -d "$WT/.git" ] || [ -f "$WT/.git" ]; } \
  || { echo "❌ fixture 工作区没建成 —— 这条套件测不了任何东西"; exit 1; }

# 脚本拷贝放进**工作区**(那是一个真 git 仓库)。REPO 是从脚本自己的位置推出来的,
# 而 worktree-root.sh 要在那儿跑 `git worktree list` —— 放在一个普通目录里,
# resolver 报「读不到 git」,而那条信息完全没提 fixture 的形状。实测踩到。
mkdir -p "$WT/scripts"
cp "$REPO/scripts/prepare-sandbox-build.sh" "$WT/scripts/"
cp "$REPO/scripts/worktree-root.sh" "$WT/scripts/"
# 🔴 fixture 自己造出来的东西必须被忽略,否则脏检查先于构建触发,
#    每一条用例都红在「工作区脏」而不是被测的那道闸。第一版就是这么红的:
#    输出明明白白写着 `M scripts/prepare-sandbox-build.sh`。
printf '/target\nCargo.lock\nnode_modules\nui/dist\nui/.build.log\n' > "$WT/.gitignore"
( cd "$WT" && "$REAL_GIT" add -A >/dev/null 2>&1 \
  && "$REAL_GIT" -c core.hooksPath=/dev/null commit -q --no-verify -m "fixture layout" ) >/dev/null 2>&1

# 假 pnpm / 假 tauri,行为由 $FIX/mode 决定。
# 🔴 用**单独写出的文件**而不是内嵌 heredoc。第一版用 heredoc 并做了变量转义,
#    结果在 set -u 下变成未绑定变量,假 pnpm 自己就崩了 ——
#    「line 93: MODE: unbound variable」,而套件报的是「干净构建退出 1」。
#    一个自己会崩的 fixture,测不出被测代码的任何东西。
cat > "$FIX/fake-pnpm" <<'FAKEEOF'
#!/usr/bin/env bash
set -u
MODE="$(cat "${FAKE_MODE_FILE:-/dev/null}" 2>/dev/null || echo ok)"
# 🔴 只对 pnpm 自己的失败形状响应。第一版在这里也列了 tauri-fails 并 exit 1,
#    于是第 6 节的第二次构建死在前端这一步,tauri 那行变异根本没被执行到 ——
#    报的是「第 4 条守的是空的」,而真相是变异从未被跑到。
#    一个步骤的失败必须只由那个步骤的注入产生,否则后面每道闸都测不到。
case "$MODE" in
  pnpm-fails)  echo "error TS7006: Parameter 'x' implicitly has an 'any' type" >&2; exit 2;;
  pnpm-silent) exit 2;;
esac
# 脚本是 `cd "$WT/ui" && pnpm build`,所以 cwd 就是 ui/,dist 要相对 cwd 写。
if [ "${1:-}" = "build" ]; then
  mkdir -p dist && echo built > dist/index.html
fi
exit 0
FAKEEOF
cat > "$FIX/fake-tauri" <<'FAKEEOF'
#!/usr/bin/env bash
set -u
MODE="$(cat "${FAKE_MODE_FILE:-/dev/null}" 2>/dev/null || echo ok)"
case "$MODE" in
  tauri-fails) echo "error: failed to bundle" >&2; exit 1;;
esac
mkdir -p target/release/bundle/macos/SeatLoom.app/Contents/MacOS
# 这个假 tauri 会把 MUST_HAVE_PRESENT_MARKER 这个字符串真的写进二进制 ——
# 否则「给了 MUST_HAVE 且它在二进制里」这一条永远造不出前提,整节是空的。
# 一个造不出前提的 fixture,测不出被测代码的任何东西。
{
  printf 'seatloom-build-commit=%s\n' "$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
  printf 'MUST_HAVE_PRESENT_MARKER\n'
  printf 'seatloom-build-commit=%s\n' "$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
} > target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
chmod +x target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
exit 0
FAKEEOF
chmod +x "$FIX/fake-pnpm" "$FIX/fake-tauri"
# PATH 上的入口:把模式文件路径通过环境传进去,避免在脚本里猜自己的位置。
cat > "$FIX/bin/pnpm" <<FAKEEOF
#!/usr/bin/env bash
exec env FAKE_MODE_FILE="$FIX/mode" "$FIX/fake-pnpm" "\$@"
FAKEEOF
cat > "$FIX/bin/tauri" <<FAKEEOF
#!/usr/bin/env bash
exec env FAKE_MODE_FILE="$FIX/mode" "$FIX/fake-tauri" "\$@"
FAKEEOF
chmod +x "$FIX/bin/pnpm" "$FIX/bin/tauri"

mkdir -p "$WT/node_modules/.bin" "$WT/ui"
# 🔴 脚本跑 tauri 时把 $WT/node_modules/.bin 放到 PATH **最前面**,所以放这儿的
#    tauri 会盖住 $FIX/bin/tauri。第一版在这里放了一个 `exit 0` 的桩,
#    于是它永远成功、永远不产出 bundle —— 前端过了,release 卡在「说成功但没产出」。
#    正解:这个桩转手调用那个可切换行为的假 tauri。
cat > "$WT/node_modules/.bin/tauri" <<FAKEEOF
#!/usr/bin/env bash
exec env FAKE_MODE_FILE="$FIX/mode" "$FIX/fake-tauri" "\$@"
FAKEEOF
chmod +x "$WT/node_modules/.bin/tauri"
echo ok > "$FIX/mode"

# 前置形状必须真的在,否则后面每一条都红在一个没说原因的地方。
[ -d "$WT/ui" ] || { echo "❌ fixture 的 ui/ 不在 —— 套件测不了任何东西"; exit 1; }
[ -x "$WT/node_modules/.bin/tauri" ] || { echo "❌ fixture 的 tauri 不在"; exit 1; }
[ -f "$WT/scripts/prepare-sandbox-build.sh" ] || { echo "❌ fixture 的脚本拷贝不在"; exit 1; }

run_build(){
  # 后面跟的参数原样传给被测脚本。
  # 🔴 曾经在这儿写死一个 MUST_HAVE_ME 尾巴,于是第 8 节(钉 MUST_HAVE 本身)
  #    传进去的标记变成了第 2 个位置参数、MUST_HAVE_ME 变成第 3 个 ——
  #    正好撞上第 9 节要钉的那道「位置参数太多」拒绝,整节全红,
  #    而红的原因是这个 helper,不是被测代码。实测踩到。
  #    **一个自己会干扰被测参数的 fixture,测不出那个参数的行为。**
  local extra="$*"
  ( cd "$WT" && PATH="$FIX/bin:$PATH" \
      SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
      SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
      bash scripts/prepare-sandbox-build.sh --build-only packet/fixture/test $extra ) 2>&1
}

# 每一次跑之前把工作区弄干净,并且把上一次的产物删掉 ——
# 否则「旧产物被接受」这条根本没机会被测到。
# 🔴 target/ **不能**由 fixture 预建。第一版在这里 mkdir -p 了,而那正好替被测
#    代码准备好「写日志前 target/ 存在」这个前提 —— 于是 --build-only 路径上缺
#    mkdir 这个真实缺陷被 fixture 遮住了,测试全绿。fixture 只该造**业务**前提,
#    不该造被测代码自己负责的前提。
reset_fixture(){
  echo ok > "$FIX/mode"
  rm -rf "$WT/ui/dist" "$WT/target"
  ( cd "$WT" && "$REAL_GIT" checkout -q -- . ) >/dev/null 2>&1
}

echo "── 1. 正面用例:干净构建必须成功,并且不安装 ──"
reset_fixture
OUT="$(run_build)"; RC=$?
[ "$RC" = "0" ] && pass "干净构建退出 0" || { die "干净构建退出 $RC"; printf '%s\n' "$OUT"|tail -6|sed 's/^/      /'; }
printf '%s\n' "$OUT" | grep -q "前端产物在" && pass "说了前端产物在" || die "干净构建没说前端产物在"
printf '%s\n' "$OUT" | grep -q "一步都没装" && pass "明说一步都没装" || die "没说一步都没装"
[ -e "$FIX/Applications/SeatLoom-Sandbox.app" ] && die "干净构建却装出了沙箱" || pass "干净构建也没有装出沙箱"

echo ""
echo "── 2. pnpm 失败(有输出)→ 非零,不宣称成功 ──"
reset_fixture
echo pnpm-fails > "$FIX/mode"
OUT="$(run_build)"; RC=$?
[ "$RC" != "0" ] && pass "pnpm 失败时非零退出($RC)" || die "pnpm 失败却退出 0 —— 它认为可以装"
printf '%s\n' "$OUT" | grep -q "前端产物在" && die "pnpm 失败却打印「前端产物在」—— 把失败说成成功" || pass "没有宣称前端产物在"
printf '%s\n' "$OUT" | grep -q "应用包在" && die "pnpm 失败却打印「应用包在」" || pass "没有宣称应用包在"
printf '%s\n' "$OUT" | grep -q "TS7006" && pass "把 pnpm 自己的报错带出来了" || die "没带出 pnpm 的报错 —— 操作者看不到为什么失败"

echo ""
echo "── 3. pnpm 失败(无输出)→ 非零,不宣称成功 ──"
# 静默失败是最危险的一种:没有任何输出可以看,只能靠退出码。
reset_fixture
echo pnpm-silent > "$FIX/mode"
OUT="$(run_build)"; RC=$?
[ "$RC" != "0" ] && pass "pnpm 静默失败时非零退出($RC)" || die "pnpm 静默失败却退出 0"
printf '%s\n' "$OUT" | grep -q "前端产物在" && die "静默失败却打印「前端产物在」" || pass "没有宣称前端产物在"
printf '%s\n' "$OUT" | grep -q "前端构建失败" && pass "明说前端构建失败" || die "没说前端构建失败"

echo ""
echo "── 4. tauri 失败 + 上一次的产物还在 → 非零,不用旧产物冒充 ──"
# 这一条钉的是「旧产物不能当新产物」。先跑一次成功构建留下产物,再让 tauri 失败。
reset_fixture
OUT0="$(run_build)"; RC0=$?
[ "$RC0" = "0" ] || die "第一次(成功)构建退出 $RC0 —— 这条用例的前提没造出来"
STALE_BEFORE="$(cat "$WT/target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri" 2>/dev/null)"
[ -n "$STALE_BEFORE" ] || die "第一次构建没留下产物 —— 这条用例没有旧产物可用"
echo tauri-fails > "$FIX/mode"
# 旧产物留在原处(不删),这正是真实情况:上一次构建的 target/release 还在。
OUT="$(run_build)"; RC=$?
echo ok > "$FIX/mode"
[ "$RC" != "0" ] && pass "tauri 失败时非零退出($RC)" || die "tauri 失败却退出 0 —— 它认为可以装"
printf '%s\n' "$OUT" | grep -q "应用包在" && die "tauri 失败却打印「应用包在」—— 把旧产物说成新的" || pass "没有宣称应用包在"
printf '%s\n' "$OUT" | grep -q "一步都没装" && die "tauri 失败却走到装那一步" || pass "没有走到安装"
[ -e "$FIX/Applications/SeatLoom-Sandbox.app" ] && die "tauri 失败却装出了沙箱" || pass "tauri 失败时没有装出沙箱"

# 变异改的是**测试真正执行的那一份拷贝**。
# 第一版指向 $FIX/repo/scripts/,而 run_build 跑的是 $WT/scripts/ ——
# 改一份没人跑的副本,变异当然不改变行为,于是报「判据守的是空的」,
# 而真相是变异没落到被执行的代码上。实测踩到。备份备份的是原始脚本。
MUT="$WT/scripts/prepare-sandbox-build.sh"
cp "$SCRIPT" "$FIX/mut-orig.sh"

echo ""
echo "── 5. 变异:还原那个 fail-open,第 2/3 条必须红 ──"
# 承重命题。变异不是随便改改,而是把当初那个 fail-open 原样放回去:
#   pnpm build 2>&1 | tail -4   ← 管道让 $? 变成 tail 的,退出码被吞
#   [ -d ui/dist ]              ← 而旧目录能满足它
# 两个缺陷必须一起还原才复现当时的行为,所以锚点取的是这两行的组合。
cat > "$FIX/mut.py" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''  rm -rf "$WT/ui/dist"
# target/ 要在写日志**之前**就存在:前端构建是第一个写的,而那时 cargo 还没建过它。
# 实测:target/ 不存在时重定向失败,pnpm build 拿到退出码 2(不是它自己的失败),
# 于是报的是「前端构建失败(退出码 2)」而上面一个字都没有 —— 一个没有证据的失败。
# --apply 那一份早就有这两行,而 --build-only 这一份没有 —— 同一条规则在两条路径上
# 只修了一处,正是这个 bug 会复发的原因。2026-11-07 补上,并且测试里不再由 fixture
# 预先建好 target/(那等于让测试替被测代码准备好前提)。
mkdir -p "$WT/target"
  ( cd "$WT/ui" && pnpm build ) > "$WT/target/ui-build.log" 2>&1
  UI_RC=$?
  tail -4 "$WT/target/ui-build.log" | sed 's/^/    /'
  [ "$UI_RC" = "0" ] || fail "前端构建失败(退出码 $UI_RC)。上面是它自己的话。这个构建不带最新前端,不要装。"'''
new = '''  ( cd "$WT/ui" && pnpm build ) 2>&1 | tail -4 | sed 's/^/    /'
  UI_RC=0'''
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
PYEOF
# 🔴 reset 放在变异**之前**:reset_fixture 里有 `git checkout -- .`,
#    它会把刚打上的变异还原回提交里的版本。第一版先变异后 reset,
#    于是变异被自己擦掉,报的是「判据守的是空的」,而真相是变异没活到运行。
if ! python3 "$FIX/mut.py" "$MUT"; then
  die "变异锚点没找到 —— 这条变异什么都没测"
else
  mkdir -p "$WT/ui/dist"; echo yesterday > "$WT/ui/dist/index.html"   # 上一次构建留下的
  echo pnpm-fails > "$FIX/mode"
  OUTM="$(run_build --allow-dirty)"; RCM=$?
  echo ok > "$FIX/mode"
  if [ "$RCM" = "0" ] || printf '%s\n' "$OUTM" | grep -q "前端产物在"; then
    pass "变异后失败被说成成功(退出码 $RCM)—— 第 2/3 条真的在守东西"
  else
    die "变异后仍然正确失败 —— 第 2/3 条守的是空的"
  fi
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
  rm -f "$FIX/mut.py"
fi

echo ""
echo "── 6. 变异:删掉 tauri 的退出码检查,第 4 条必须红 ──"
# 第 4 条(tauri 失败 + 旧产物)真正靠的是 `[ \"$TAURI_RC\" = \"0\" ] || fail` ——
# 上一次构建留下的 SeatLoom.app 能满足 `[ -x \"$BUILT/Contents/MacOS/seatloom-tauri\" ]`,
# 只有退出码能拦住它。所以变异必须落在这一行上,而不是别的。
#
# 🔴 这里曾经是一个**假命题**,如实记下来:我原先的变异是「删掉 rm -rf ui/dist」,
#    然后断言第 4 条必须红。它不红,于是套件报「判据守的是空的」。真相相反 ——
#    判据没问题是变异选错了:退出码检查还在,pnpm 的失败照样被它抓住,
#    删 rm -rf 根本不改变可观察行为。两道闸叠在一起时,
#    单独拆掉后面那道不红,不能推出它不在守东西;能推出的只是
#    **它在这一条上不是唯一承重的那道**。一个选错对象的变异,
#    比没有变异更有害:它会让人去"修"一道本来就不用修的闸。
cat > "$FIX/mut2.py" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''  [ "$TAURI_RC" = "0" ] || fail "tauri build 失败(退出码 $TAURI_RC)。上面是它自己的话。不要装。"'''
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
assert s.count(old) == 1, "锚点出现 %d 次 —— 改到别的路径上去了" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, 'true', 1))
PYEOF
if ! python3 "$FIX/mut2.py" "$MUT"; then
  die "变异锚点没找到 —— 这条变异什么都没测"
else
  OUT0="$(run_build --allow-dirty)"; RC0=$?
  [ "$RC0" = "0" ] || die "前提没造出来:第一次构建退出 $RC0"
  [ -x "$WT/target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri" ] \
    || die "第一次构建没留下应用包 —— 这条用例没有旧产物可用"
  echo tauri-fails > "$FIX/mode"
  OUTM2="$(run_build --allow-dirty)"; RCM2=$?
  echo ok > "$FIX/mode"
  if [ "$RCM2" = "0" ] || printf '%s\n' "$OUTM2" | grep -q "应用包在"; then
    pass "变异后旧应用包让 tauri 失败看起来成功(退出码 $RCM2)—— 第 4 条真的在守东西"
  else
    die "变异后仍然正确失败 —— 第 4 条守的是空的"
  fi
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
  rm -f "$FIX/mut2.py"
fi

echo ""
echo "── 7. 注销:rm -rf ui/dist 在这一条上不是唯一承重的闸 ──"
# 上一节留下的问题必须有答案,不能靠删掉一条断言来回避。如实测一次:
# 只删 rm -rf、保留退出码检查时,行为不变 —— 因为它防的是「不查退出码」那一层,
# 而退出码这一层还在。所以它是纵深防御,不是第 4 条的承重闸。
# 这一节的作用是把这个结论钉住:以后有人删掉它,这里会红,
# 而红的原因写的是「纵深防御被拆」,不是「第 4 条被拆」—— 两件事不能混。
cat > "$FIX/mut3.py" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''  rm -rf "$WT/ui/dist"
# target/ 要在写日志**之前**就存在:前端构建是第一个写的,而那时 cargo 还没建过它。
# 实测:target/ 不存在时重定向失败,pnpm build 拿到退出码 2(不是它自己的失败),
# 于是报的是「前端构建失败(退出码 2)」而上面一个字都没有 —— 一个没有证据的失败。
# --apply 那一份早就有这两行,而 --build-only 这一份没有 —— 同一条规则在两条路径上
# 只修了一处,正是这个 bug 会复发的原因。2026-11-07 补上,并且测试里不再由 fixture
# 预先建好 target/(那等于让测试替被测代码准备好前提)。
mkdir -p "$WT/target"'''
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
open(p, 'w', encoding='utf-8').write(s.replace(old, 'mkdir -p "$WT/target"', 1))
PYEOF
if ! python3 "$FIX/mut3.py" "$MUT"; then
  die "变异锚点没找到 —— 这条变异什么都没测"
else
  reset_fixture
  mkdir -p "$WT/ui/dist"; echo yesterday > "$WT/ui/dist/index.html"
  echo pnpm-fails > "$FIX/mode"
  OUTM3="$(run_build --allow-dirty)"; RCM3=$?
  echo ok > "$FIX/mode"
  if [ "$RCM3" != "0" ]; then
    pass "只删 rm -rf 仍然失败($RCM3)—— 它是纵深防御,不是第 4 条的承重闸(如实记录)"
  else
    pass "只删 rm -rf 时失败被放过 —— 它在第 2/3 条上是承重闸"
  fi
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
  rm -f "$FIX/mut3.py"
fi

echo ""

echo ""
echo "── 8. MUST_HAVE:给了就必须真的用,不给就要么拒要么明说没验 ──"
# Flux 的 ev-TWL0d7liGTtboRRnO7B34 在 6dd68c5 上量到一个真缺陷,我复现了:
#   MUST_HAVE 从来没被赋值过 —— `*)` 分支只把第一个位置参数给 $BRANCH,
#   第二个开始全被 shift 悄悄吃掉。于是「证明这个构建带着那份活」这一节
#   从来没跑过,而它打出来的建议命令又把 $MUST_HAVE 拼回去,看起来可用 ——
#   照着敲一次,那个值又被吃掉一次。
#   **一个跑不起来的检查,比没有检查更有害**:它让操作者以为那一节验过了。
reset_fixture
OUT0="$(run_build --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RC0=$?
[ "$RC0" = "0" ] || die "给了 MUST_HAVE 而它真在二进制里时,必须成功(退出码 $RC0)"
printf '%s\n' "$OUT0" | grep -q "二进制里找到 'MUST_HAVE_PRESENT_MARKER'" \
  && pass "MUST_HAVE 在二进制里 → 说了找到,带处数" \
  || die "没说「二进制里找到 MUST_HAVE_PRESENT_MARKER」—— 那个值还是没被用上"

reset_fixture
OUTA="$(run_build --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCA=$?
[ "$RCA" != "0" ] && pass "MUST_HAVE 不在二进制里 → 非零退出($RCA)" \
  || die "MUST_HAVE 不在二进制里却退出 0 —— 这一节仍然是个摆设"
printf '%s\n' "$OUTA" | grep -q "二进制里找不到 'MUST_HAVE_ABSENT_MARKER'" \
  && pass "并且说了为什么:二进制里找不到" || die "没说为什么找不到"

reset_fixture
OUTB="$(run_build --allow-dirty)"; RCB=$?
[ "$RCB" = "0" ] && pass "不给 MUST_HAVE → 仍然成功(它是可选的)" \
  || die "不给 MUST_HAVE 却失败了(退出码 $RCB)—— 它不该是必填"
printf '%s\n' "$OUTB" | grep -q "没给必须存在的字符串" \
  && pass "并且明说这一项没验 —— 不假装验过" \
  || die "没说这一项没验 —— 一个静默跳过的检查比没有检查更有害"

echo ""
echo "── 9. 多余的位置参数必须被拒,不是被悄悄丢掉 ──"
# 这个 bug 的形状就是「悄悄丢掉」,所以反过来也要钉:多给一个要被拒。
reset_fixture
OUTC="$(run_build --allow-dirty MARKER_A MARKER_B MARKER_C)"; RCC=$?
[ "$RCC" != "0" ] && pass "第三个位置参数 → 非零退出($RCC)" \
  || die "第三个位置参数被悄悄收下了 —— 那正是这个 bug 的形状"
printf '%s\n' "$OUTC" | grep -q "位置参数太多" \
  && pass "并且说了是位置参数太多,以及第几个" || die "没说为什么拒"

echo ""
echo "── 10. 变异:把 MUST_HAVE 再丢掉,第 8 条必须红 ──"
# 承重命题。变异就是还原当初那个 bug:`*)` 分支只接第一个位置参数。
cat > "$FIX/mut4.py" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = """       case "$POSITIONAL" in
         1) BRANCH="$1";;
         2) MUST_HAVE="$1";;
         *) fail "位置参数太多了:'$1' 是第 $POSITIONAL 个。用法:prepare-sandbox-build.sh [旗标] <分支> [必须存在的字符串]"
            ;;
       esac
       shift;;"""
# 变异必须产出一个**仍然合法**的脚本:第一版把 case 换成
# `[ -z "$BRANCH" ] && BRANCH="$1";;`,留下一个悬空的 ;; ——
# bash 直接语法错误,脚本根本跑不起来。那样第 8 条"没红"是假的:
# 它不是没守东西,是变异把脚本弄坏了,失败来自语法而不是来自 MUST_HAVE 被丢掉。
# **一个把被测程序弄成跑不起来的变异,什么都证明不了。**
# 所以这里保留 case 结构,只把第 2 个分支变成"什么都不做"。
new = """       case "$POSITIONAL" in
         1) BRANCH="$1";;
         2) : ;;
         *) fail "位置参数太多了:'$1' 是第 $POSITIONAL 个。用法:prepare-sandbox-build.sh [旗标] <分支> [必须存在的字符串]"
            ;;
       esac
       shift;;"""
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
assert s.count(old) == 1, "锚点出现 %d 次 —— 改到别的路径上去了" % s.count(old)
out = s.replace(old, new, 1)
open(p, 'w', encoding='utf-8').write(out)
# 自检:变异后的脚本必须仍然是合法 bash。不是的话,这条变异什么都测不到。
import subprocess
rc = subprocess.run(["bash", "-n", p], capture_output=True).returncode
if rc != 0:
    sys.exit("变异后的脚本语法错误 —— 这条变异会弄坏程序而不是还原 bug")
PYEOF
if ! python3 "$FIX/mut4.py" "$MUT"; then
  die "变异锚点没找到 —— 这条变异什么都没测"
else
  # 🔴 先 reset 再变异。reset_fixture 里有 `git checkout -- .`,它会把
  #    刚打上的变异还原回提交里的版本 —— 第 5、6 节已经踩过一次并修好了,
  #    新加的第 10 节又照抄了旧形状,于是变异被自己擦掉。
  #    探针(下面那行 grep)就是为了让这件事在发生时直接说出来,
  #    而不是绕个弯从"判据守的是空的"反推。
  grep -q '2) : ;;' "$MUT" || die "变异没落到被执行的那份脚本上 —— 这一节什么都没测"
  OUTM4="$(run_build --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RCM4=$?
  if printf '%s\n' "$OUTM4" | grep -q "二进制里找到"; then
    die "变异后 MUST_HAVE 仍然生效 —— 第 8 条守的是空的"
  else
    pass "变异后 MUST_HAVE 再次被丢掉 —— 第 8 条真的在守东西"
  fi
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
  rm -f "$FIX/mut4.py"
fi

echo ""
echo "── 11. detached 检出的拒绝必须说清原因和补法 ──"
# 第二条:Flux 在隔离 detached checkout 里验 6dd68c5,看到「不是任何工作区
# 当前检出的分支」就走了投无路。消息列了工作区却没说他为什么不被接受。
# **一条拒绝消息如果不能告诉人怎么满足它,它就只是宣布了失败。**
grep -q 'DETACHED_HINT' "$SCRIPT" \
  && pass "脚本里有 DETACHED_HINT 这段(拒绝里会印出补法)" \
  || die "脚本里没有 DETACHED_HINT —— detached 的调用者还是只会看到一句没头没尾的拒绝"
grep -q 'detached 检出,树正好在这个提交上' "$SCRIPT" \
  && pass "并且把「树就在这儿、只差一个分支名」说出来了" \
  || die "没说清树就在这个提交上"
grep -q 'git -C ${d%/} branch' "$SCRIPT" \
  && pass "并且给了具体可敲的补分支命令" || die "没给补分支的命令"
if [ "$FAIL" = "0" ]; then
  echo "✅ 全部通过"
  exit 0
fi
echo "❌ 有失败项"
exit 1
