#!/usr/bin/env bash
# test-replace-sandbox-failclosed.sh — 证明 replace-sandbox.sh 的失败路径真的失败。
#
# 为什么需要这个测试(2026-11-07 Aegis 的 HOLD):
#   这个脚本做的事不可逆:停进程、rm -rf、cp -R、改 plist、启动。Aegis 读到
#   21cf1e9 那一版,发现三处 fail-open:
#     1. 二进制里读不到提交号 → 只打一句 ⚠️ 然后继续
#     2. git status 读失败 → 和干净混在一起
#     3. 脏工作区 → 只打一句 ⚠️ 然后继续换
#   还有:构建产物缺失时指人去跑 prepare-sandbox-build.sh --apply,而那个 --apply
#   顺手 rm -rf + cp -R 把沙箱换掉,一步备份都没有。
#
#   这三处的共同形状是**一个会说话的警告后面跟着继续执行**。警告不是阻碍,
#   而在这里唯一该做阻碍的就是它们。
#
# 所以这个测试的承重命题:
#   **每一次失败都必须是零效果,而且必须非零退出。**
#   不是「打印了一句警告」,是「什么都没动 + 退出码非零」。
#
# 做法:fixture 里造假主版、假沙箱、假备份根、假 git、假 pkill、假 open,
#      注入失败,然后断言 fixture 里没有出现任何「动过」的痕迹。
#      **绝不碰真实的 /Applications、live 账本、在跑的应用。**
#
# 每一行都必须能失败。最后一条是变异:把 fail-closed 退回 warn-then-continue,
# 对应的断言必须红。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO/scripts/replace-sandbox.sh"
[ -f "$SCRIPT" ] || { echo "❌ 找不到 $SCRIPT"; exit 1; }

ROOT="$(mktemp -d)/rsf"
FIX="$ROOT/fixture"
FAIL=0
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }
cleanup(){ rm -rf "$ROOT"; }
trap cleanup EXIT

# ── 造 fixture ────────────────────────────────────────────────────────────
# 形状和真实环境一致到脚本需要的程度:
#   $FIX/repo/scripts/replace-sandbox.sh        脚本本体(拷贝)
#   $FIX/repo/scripts/prepare-sandbox-build.sh  被指向的脚本(拷贝)
#   $FIX/seatloom-worktrees/wt/                 源工作区(一个真 git 仓库)
#   $FIX/Applications/                          假 /Applications
#   $FIX/bin/                                   假 git / pkill / open / plutil
mkdir -p "$FIX/repo/scripts" "$FIX/seatloom-worktrees/wt" "$FIX/Applications" "$FIX/bin" "$FIX/backups"
cp "$SCRIPT" "$FIX/repo/scripts/replace-sandbox.sh"
cp "$REPO/scripts/prepare-sandbox-build.sh" "$FIX/repo/scripts/prepare-sandbox-build.sh"

# 假主版和假沙箱。主版必须真的存在,因为「没碰主版」是一个可断言的命题。
mkdir -p "$FIX/Applications/SeatLoom.app/Contents/MacOS"
printf '#!/usr/bin/env bash\necho main\n' > "$FIX/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri"
chmod +x "$FIX/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri"

# 假 git:默认就是真 git,但可以用 $FIX/git-mode 让它按剧本坏。
# 剧本写在文件里,是因为每个用例要的是不同的坏法。
# 🔴 不能 shift 掉 `-C <目录>` 再 exec 真 git —— 那样真 git 就在当前目录跑,
#    而不是在工作区里跑,于是 rev-parse 全部失败,「找不到工作区」。
#    第一版就是这么写的,五个用例一起红,看起来像脚本坏了。
#    正确做法:扫全部参数找触发词,一个都不动,原样传给真 git。
REAL_GIT="$(command -v git)"
cat > "$FIX/bin/git" <<EOF
#!/usr/bin/env bash
MODE="\$(cat "$FIX/git-mode" 2>/dev/null || echo ok)"
case "\$MODE" in
  status-fails)
    for a in "\$@"; do
      if [ "\$a" = "status" ]; then echo "fatal: unable to read the index" >&2; exit 128; fi
    done;;
  revparse-fails)
    for a in "\$@"; do
      if [ "\$a" = "rev-parse" ]; then echo "fatal: not a git repository" >&2; exit 128; fi
    done;;
esac
exec "$REAL_GIT" "\$@"
EOF
chmod +x "$FIX/bin/git"
echo ok > "$FIX/git-mode"

# 假 pkill / open / plutil:全部记录自己被调用。
# 这个测试最重要的断言就是「这些一次都没被调用」。
cat > "$FIX/bin/pgrep" <<EOF
#!/usr/bin/env bash
echo "PGREP \$*" >> "$FIX/executed.log"
exit 1
EOF
cat > "$FIX/bin/open" <<EOF
#!/usr/bin/env bash
echo "OPEN \$*" >> "$FIX/executed.log"
echo fake-open
EOF
chmod +x "$FIX/bin/pgrep" "$FIX/bin/open"

# 源工作区:一个真 git 仓库,检出在 packet/fixture/test 上。
( cd "$FIX/seatloom-worktrees/wt" \
  && "$REAL_GIT" init -q -b packet/fixture/test \
  && "$REAL_GIT" config user.email t@t.local && "$REAL_GIT" config user.name t \
  && echo hi > README.md && "$REAL_GIT" add . && "$REAL_GIT" commit -qm base ) >/dev/null 2>&1
WT_SHORT="$(cd "$FIX/seatloom-worktrees/wt" && "$REAL_GIT" rev-parse --short=12 HEAD)"

# 一个「构建产物」。二进制里带提交标签,这样身份核对才有东西可核。
# 同时带 MUST_HAVE_ME —— 「必须存在的字符串」这项检查也是承重的,
# fixture 的二进制里没有它,基线就会在这里失败,而失败原因跟被测试的东西无关。
BUILT="$FIX/seatloom-worktrees/wt/target/release/bundle/macos/SeatLoom.app"
mkdir -p "$BUILT/Contents/MacOS"
# 用 printf 拼出一个真的二进制:标签 + 当前提交 + 必须存在的串 + 一些噪声十六进制。
# 噪声十六进制是故意放的:标签匹配必须只认标签,不能被噪声里的十六进制骗到
#(2026-11-07 实测:裸十六进制匹配在二进制里抓到 dbdbdbdbdbdb)。
make_bin(){
  { printf 'seatloom-build-commit=%s\n' "$1"
    printf 'MUST_HAVE_ME\n'
    printf 'dbdbdbdbdbdb deadbeefdead 0123456789ab\n'; } > "$BUILT/Contents/MacOS/seatloom-tauri"
  chmod +x "$BUILT/Contents/MacOS/seatloom-tauri"
}
make_bin "$WT_SHORT"

# 跑一次干跑。所有用例都用这个入口,只有 git-mode / 沙箱状态不同。
run_rs(){
  ( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
      SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
      SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
      SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
      bash scripts/replace-sandbox.sh --from-branch packet/fixture/test MUST_HAVE_ME ) 2>&1
}

echo "── 0. 基线:干净且身份一致时,干跑通过且什么都不动 ──"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
if [ "$RC" = "0" ]; then pass "基线干跑退出 0"; else die "基线干跑退出 $RC"; printf '%s\n' "$OUT"|tail -15; fi
printf '%s\n' "$OUT" | grep -qF "$WT_SHORT" && pass "核对了二进制里的提交号" || die "没核对提交号"
if [ -f "$FIX/executed.log" ]; then
  die "基线干跑就执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "基线一个副作用命令都没执行"
fi

echo ""
echo "── 1. 二进制里读不到提交号 → 必须停,不能只警告 ──"
# Aegis 的 HOLD 第一条。第一版只打 ⚠️「不是失败,是没核上」然后继续 ——
# 于是「装的是哪个提交」这个唯一能证明装对了的东西,可以在完全不知道的情况下通过。
{ printf 'no commit label here at all\n'; printf 'MUST_HAVE_ME\n'; } > "$BUILT/Contents/MacOS/seatloom-tauri"
chmod +x "$BUILT/Contents/MacOS/seatloom-tauri"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
[ "$RC" != "0" ] && pass "读不到提交号时非零退出" || die "读不到提交号却退出 0 —— 它认为可以装"
printf '%s\n' "$OUT" | grep -q "没能从二进制里读出提交号" \
  && pass "明说读不到提交号" || die "没说读不到提交号 —— 只非零退出但没说原因"
if [ -f "$FIX/executed.log" ]; then
  die "读不到提交号时仍然执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "读不到提交号时零副作用"
fi

echo ""
echo "── 2. 二进制里的提交和工作区不一致 → 必须停 ──"
{ printf 'seatloom-build-commit=aaaaaaaaaaaa\n'; printf 'MUST_HAVE_ME\n'; } > "$BUILT/Contents/MacOS/seatloom-tauri"
chmod +x "$BUILT/Contents/MacOS/seatloom-tauri"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
[ "$RC" != "0" ] && pass "身份不一致时非零退出" || die "身份不一致却退出 0 —— 装上一个自称 A 实际 B 的沙箱"
printf '%s\n' "$OUT" | grep -q "不一致" && pass "明说不一致" || die "没说不一致"

echo ""
echo "── 3. git status 读失败 → 必须停,不能读成干净 ──"
# 同一形状,和 prepare-verification-build.sh 2026-11-07 修的那个一模一样:
# 一条管道 + || true 让 git 的退出码被 grep 的「没匹配」吃掉。
make_bin "$WT_SHORT"
echo status-fails > "$FIX/git-mode"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
echo ok > "$FIX/git-mode"
[ "$RC" != "0" ] && pass "读不出工作区状态时非零退出" || die "读不出状态却退出 0 —— 这份构建跟提交的关系根本没测过"
printf '%s\n' "$OUT" | grep -q "工作区干净" \
  && die "git 读失败被说成「工作区干净」" || pass "没有把读失败说成干净"
printf '%s\n' "$OUT" | grep -q "读不出源工作区的状态" \
  && pass "明说读不出工作区状态" || die "没说读不出工作区状态"
printf '%s\n' "$OUT" | grep -q "unable to read the index" \
  && pass "带出了 git 的原话" || die "没带出 git 的报错 —— 分不清是干净还是读不出来"
if [ -f "$FIX/executed.log" ]; then
  die "读不出状态时仍然执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "读不出状态时零副作用"
fi

echo ""
echo "── 4. 脏工作区 → 必须是阻碍,不是一句提醒 ──"
printf 'dirty change\n' >> "$FIX/seatloom-worktrees/wt/README.md"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
( cd "$FIX/seatloom-worktrees/wt" && "$REAL_GIT" checkout -q -- . ) >/dev/null 2>&1
[ "$RC" != "0" ] && pass "脏工作区时非零退出(它是阻碍)" || die "脏工作区却退出 0 —— 那等于允许装一个不等于该提交的东西"
printf '%s\n' "$OUT" | grep -q "前提都满足" \
  && die "脏工作区却打印「前提都满足」" || pass "脏工作区时没有打印「前提都满足」"
if [ -f "$FIX/executed.log" ]; then
  die "脏工作区时仍然执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "脏工作区时零副作用"
fi

echo ""
echo "── 5. 构建产物缺失 → 指路只指构建,绝不指 --apply ──"
# 第一版叫人跑 prepare-sandbox-build.sh <分支> --apply,而那个 --apply 会
# rm -rf + cp -R 把沙箱换掉,一步备份都没有。在一个「先备份再换」的脚本里
# 指人去跑一个「直接换不备份」的脚本,是把刚立好的规矩拆掉。
mv "$BUILT" "$FIX/built.bak"
rm -f "$FIX/executed.log"
OUT="$(run_rs)"; RC=$?
mv "$FIX/built.bak" "$BUILT"
[ "$RC" != "0" ] && pass "没有构建产物时非零退出" || die "没有构建产物却退出 0"
printf '%s\n' "$OUT" | grep -q -- "--apply" \
  && die "指路里出现了 --apply —— 那会绕过备份直接换沙箱" \
  || pass "指路里没有 --apply"
printf '%s\n' "$OUT" | grep -q -- "--build-only" \
  && pass "指路指到 --build-only(只构建,不安装)" || die "没指到 --build-only"
if [ -f "$FIX/executed.log" ]; then
  die "没有构建产物时仍然执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "没有构建产物时零副作用"
fi

echo ""
echo "── 6. 沙箱路径 == 主版路径 → 必须停(否则就是把主版删掉)──"
# 这个脚本所有破坏性动作都只针对沙箱路径。一旦它被指到主版,后果就是删掉主版。
rm -f "$FIX/executed.log"
OUT="$( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
    SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom.app" \
    SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
    SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
    bash scripts/replace-sandbox.sh --from-branch packet/fixture/test 2>&1 )"
RC=$?
[ "$RC" != "0" ] && pass "沙箱路径等于主版路径时非零退出" || die "两条路径相同却继续 —— 它会删掉主版"
printf '%s\n' "$OUT" | grep -q "相同" && pass "明说两条路径相同" || die "没说路径相同"
[ -x "$FIX/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri" ] \
  && pass "主版还原样在(可执行文件还在)" || die "主版被动了"

echo ""
echo "── 7. 变异:把 fail-closed 退回 warn-then-continue,这个测试必须红 ──"
# 承重命题:上面 1/3/4 不是靠运气绿的。逐个把阻碍改回「警告然后继续」,
# 对应的断言必须红。三个都测,因为它们是三处独立的 fail-open。
for which in commit-missing status-fails dirty; do
  MUT="$FIX/repo/scripts/replace-sandbox.sh"
  cp "$MUT" "$FIX/mut-orig.sh"
  python3 - "$MUT" "$which" <<'PYEOF'
import sys
p, which = sys.argv[1], sys.argv[2]
s = open(p, encoding='utf-8').read()
# 变异 commit-missing 必须是**原来那个形状**:整个身份核对被「读到才核」包着,
# 读不到就只打一句 ⚠️ 然后往下走。只把那个 fail 改成 say 是不够的 ——
# 后面还有一道「不一致就 fail」的闸,空串和 $WT_SHORT 比必然不一致,于是变异
# 被下一道闸挡住,看起来像「变异没复现」,其实是变异造得不像原件。
if which == 'commit-missing':
    old = '''if [ -z "$BIN_COMMIT" ]; then
  fail "没能从二进制里读出提交号。二进制必须自带提交(build.rs 的 BUILD_COMMIT),否则「装的是哪个提交」不可核。重新构建再换。"
fi
say "      $BIN_COMMIT"
if [ "$BIN_COMMIT" != "$WT_SHORT" ]; then
  fail "二进制里的提交 ($BIN_COMMIT) 和工作区 HEAD ($WT_SHORT) 不一致 —— 这份构建不是这棵树。重新构建再换。"
fi
say "      ✅ 与工作区 HEAD 一致"'''
    new = '''say "      ⚠️  没能从二进制里读出提交号(不是失败,是没核上)"'''
elif which == 'status-fails':
    old = '  fail "读不出源工作区的状态(git status 退出码 $WT_STATUS_RC)${WT_STATUS_MSG:+ —— $WT_STATUS_MSG}。这份构建跟提交 $WT_SHORT 的关系没有测过,不能换。"'
    new = '  say "      ⚠️  读不出源工作区的状态(继续)"'
else:
    old = '  fail "源工作区脏 —— 先提交或 stash,否则沙箱里的东西不等于 $WT_SHORT"'
    new = '  say "      ⚠️  源工作区脏(继续)"'
assert old in s, f"变异锚点没找到:{which} —— 这个变异没有测到任何东西"
assert new not in s.replace(old, ''), f"变异后与变异前相同:{which}"
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
PYEOF
  if [ $? != 0 ]; then die "变异 $which 的锚点没找到 —— 它什么都没测"; cp "$FIX/mut-orig.sh" "$MUT"; continue; fi
  rm -f "$FIX/executed.log"
  # 🔴 先造出失败状态,再跑。第一版把「造状态」放在第一次 run 之后,
  #    于是第一次跑的是干净仓库(照样非零/照样通过),拿到的 RCM 不是变异的结果,
  #    「变异后仍然非零退出」其实是「变异根本没被测到」。顺序错了,
  #    一条变异测试就变成一条永远说「守的是空的」的测试。
  case "$which" in
    commit-missing) { printf 'no commit label\n'; printf 'MUST_HAVE_ME\n'; } > "$BUILT/Contents/MacOS/seatloom-tauri";;
    status-fails) echo status-fails > "$FIX/git-mode";;
    dirty) printf 'dirty change\n' >> "$FIX/seatloom-worktrees/wt/README.md";;
  esac
  chmod +x "$BUILT/Contents/MacOS/seatloom-tauri"
  OUTM="$(run_rs)"; RCM=$?
  if [ "$RCM" = "0" ]; then
    pass "变异 $which 之后 fail-open 复现(退出 0)—— 对应断言真的在守东西"
  else
    die "变异 $which 之后仍然非零退出 —— 对应断言守的是空的"
  fi
  # 还原,并证明逐字节。
  # 🔴 比的对象必须是**仓库里那份原始脚本**,不是循环开头拷出来的副本 ——
  #    比副本证明的是「我刚才把它拷回了它自己」,那是恒真的,一个永远不会报错的
  #    检查比没有检查更有害。
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异 $which 没还原干净 —— 后面的用例会被污染"
  echo ok > "$FIX/git-mode"
  ( cd "$FIX/seatloom-worktrees/wt" && "$REAL_GIT" checkout -q -- . ) >/dev/null 2>&1
  # 用 make_bin,不是手写 printf:手写的那一版只写提交标签,漏了 MUST_HAVE_ME,
  # 于是下一个变异用例在「必须存在的字符串」上就失败了 —— 看起来像变异没复现,
  # 其实是上一个用例还原时把 fixture 弄坏了(实测踩到)。
  make_bin "$WT_SHORT"
done

echo ""
echo "── 8. 回滚默认也必须零效果,而且失败时保住回滚前那一份 ──"
# 回滚同样会 rm -rf 当前沙箱,所以它同样是破坏性的。默认(不加 --apply)
# 必须一个字节都不动。这个形状和 --from-branch 的干跑一模一样:默认不动手。
# 造一个带元信息的备份,让回滚这条路有东西可回。
RBAKSRC="$FIX/backups/20260101-000000"
mkdir -p "$RBAKSRC/SeatLoom-Sandbox.app/Contents/MacOS"
printf 'seatloom-build-commit=%s\n' "bbbbbbbbbbbb" > "$RBAKSRC/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox"
chmod +x "$RBAKSRC/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox"
{ printf '这份是什么  : 被替换掉的旧沙箱\n'
  printf '旧沙箱提交   : bbbbbbbbbbbb\n'; } > "$RBAKSRC/BACKUP-META.txt"
rm -f "$FIX/executed.log"
OUTR="$( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
  bash scripts/replace-sandbox.sh --rollback 20260101-000000 2>&1 )"
RCR=$?
[ "$RCR" = "0" ] && pass "回滚预演退出 0" || die "回滚预演退出 $RCR"
if [ -f "$FIX/executed.log" ]; then
  die "回滚预演执行了东西:"; cat "$FIX/executed.log"|sed 's/^/      /'
else
  pass "回滚预演一个副作用命令都没执行"
fi
# 当前沙箱(这里不存在)没有被创建出来 —— 预演不能凭空装一个。
[ -e "$FIX/Applications/SeatLoom-Sandbox.app" ] \
  && die "回滚预演装出了沙箱 —— 默认不该动手" \
  || pass "回滚预演没有装出沙箱"
# 元信息必须被读出来并展示:回滚要能回答「我回到了哪」。
printf '%s\n' "$OUTR" | grep -q "bbbbbbbbbbbb" \
  && pass "回滚预演展示了备份里的提交号(回的是哪一份)" \
  || die "回滚预演没展示备份的提交号 —— 说不清回到哪"

# 没有元信息的备份不用于回滚。这条必须能报错。
NOMETA="$FIX/backups/20260102-000000"
mkdir -p "$NOMETA/SeatLoom-Sandbox.app/Contents/MacOS"
printf 'x\n' > "$NOMETA/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox"
chmod +x "$NOMETA/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox"
OUTN="$( cd "$FIX/repo" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
  bash scripts/replace-sandbox.sh --rollback 20260102-000000 2>&1 )"
RCN=$?
[ "$RCN" != "0" ] && pass "没有元信息的备份拒绝回滚" || die "没有元信息却允许回滚 —— 说不清回的是哪一份"


echo ""
echo "── 9. --apply 和 --build-only 同时给 → 必须在任何副作用之前就拒绝 ──"
# B-1 (Flux 的 ev-B5dq_hybh4VZbA-dkGvbl):这两个旗标的意思是**相反的** ——
# --apply 替换沙箱,--build-only 构建完就停。同时给会让结果取决于后面哪个分支被求值,
# 而不是取决于操作者要了什么。守卫本身在,而且注释写明了它为什么存在,
# 但套件里没有任何一条测它 —— 一条只有源码撑着的规则。
#
# 承重断言不是「打印了一句话」,是**零副作用 + 非零退出**:
# --apply 的路径会 pnpm build、tauri build、rm -rf、cp -R、起进程,所以一旦守卫失守,
# executed.log 一定会有东西。这也顺便证明守卫是在副作用**之前**,而不是之后。
# 🔴 prepare-sandbox-build.sh 在 $REPO 里 `git rev-parse --verify "$BRANCH"` 核对
#    「要构建的树就是参数指定的那棵树」,所以 $REPO 必须是一个**带这条分支、而且
#    分支指向的提交就是源工作区 HEAD** 的 git 仓库。
#
#    第一版把 $FIX/repo 留成普通目录,于是 rev-parse 失败,脚本在「解析不出 ref」上
#    退出 —— 退出码确实非零,但那和守卫没有关系,第 9 条会假绿。
#    第二版给它建了个仓库,但分支指向自己那个 base 提交,于是撞上
#    「要构建的树和参数指定的不是同一棵树」—— 同样非零,同样和守卫无关。
#    两次都假绿。正确的形状是:让**源工作区自己**当那个仓库。
# 🔴 这个脚本假定的是**主检出**的形状:REPO 是仓库本身,WROOT 是它下面的
#    seatloom-worktrees/,里面每个子目录是一个检出了某条分支的席位工作区。
#    所以 fixture 要照这个形状搭,不能图省事:
#      $PSB_REPO                      主检出(带 packet/fixture/test 分支)
#      $PSB_REPO/seatloom-worktrees/wt  检出该分支的工作区(构建目录)
#    放错位置会以各种方式非零退出,而每一种都和守卫无关 —— 实测连续踩了三次:
#      1. $FIX/repo 不是仓库          → 「解析不出 ref」
#      2. 分支指向别的提交            → 「不是同一棵树」
#      3. 脚本放进工作区里面          → WROOT 找不到任何工作区
#    三次的退出码都非零,第 9 条都会假绿。所以这一条的形状是承重的。
# WROOT 是 REPO 的**兄弟目录**(`$REPO/../seatloom-worktrees`),不是它的子目录 ——
# 这一点我自己看错了两次,fixture 按子目录搭,于是循环一次都没进去,报
# 「不是任何工作区当前检出的分支」。第五次假绿,而每一次的退出码都非零。
PSB_ROOT="$FIX/psb"
PSB_REPO="$PSB_ROOT/repo"
PSB_WT="$PSB_ROOT/seatloom-worktrees/wt"
mkdir -p "$PSB_REPO/scripts" "$PSB_WT"
cp "$REPO/scripts/prepare-sandbox-build.sh" "$PSB_REPO/scripts/"
# 主检出的 HEAD 必须在**另一条**分支上。git worktree add 拒绝把一个已经有别的
# 工作区检出了的分支再检出一份 —— 而主检出自己就占了它那条分支。第一版把主检出
# 直接建在 packet/fixture/test 上,于是 worktree add 失败,工作区里没有 .git,
# 脚本的循环一个都进不去,报「不是任何工作区当前检出的分支」。第四次假绿。
( cd "$PSB_REPO" && "$REAL_GIT" init -q -b packet/fixture/root >/dev/null 2>&1 \
  && "$REAL_GIT" config user.email t@t.local && "$REAL_GIT" config user.name t \
  && echo hi > README.md && "$REAL_GIT" add . \
  && "$REAL_GIT" -c core.hooksPath=/dev/null commit -q --no-verify -m base ) >/dev/null 2>&1
( cd "$PSB_REPO" && "$REAL_GIT" branch packet/fixture/test ) >/dev/null 2>&1
# 工作区是主检出的一个 LINKED worktree —— 这正是真实形状,而且 HEAD 就在那条分支上。
( cd "$PSB_REPO" && "$REAL_GIT" worktree add -q "$PSB_WT" packet/fixture/test ) >/dev/null 2>&1
# 构建产物放在工作区里,和真实形状一致。
BUILT2="$PSB_WT/target/release/bundle/macos/SeatLoom.app"
mkdir -p "$BUILT2/Contents/MacOS"
make_bin "$WT_SHORT"
# 🔴 工作区必须**干净**,而且包括我刚刚放进去的那些文件。
#    第一版把 target/ 和 scripts/ 放进工作区之后没有提交,于是 porcelain 里有
#    未跟踪文件之外还有真的未提交改动,脚本在「不从带未提交改动的工作区构建」上
#    就停了 —— 退出码非零,但那和旗标守卫无关。实测输出明明白白写着那句,
#    而断言在找「不能同时给」。** fixture 造错状态,正确的行为看起来就是坏的。**
#    scripts/ 也要提交:它是脚本自己的拷贝,放在工作区里就是未跟踪文件。
( cd "$PSB_WT" && "$REAL_GIT" add -A >/dev/null 2>&1 \
  && "$REAL_GIT" -c core.hooksPath=/dev/null commit -q --no-verify -m "fixture artifacts" ) >/dev/null 2>&1
WT_SHORT="$(cd "$PSB_WT" && "$REAL_GIT" rev-parse --short=12 HEAD)"
make_bin "$WT_SHORT"
DIRTY_NOW="$(cd "$PSB_WT" && "$REAL_GIT" status --porcelain | grep -v '^??' | grep -c .)"
[ "$DIRTY_NOW" = "0" ] || die "fixture 的工作区不干净($DIRTY_NOW 处)—— 这一条会假绿"
run_psb(){
  ( cd "$PSB_REPO" && PATH="$FIX/bin:$PATH" \
      SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
      SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
      SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
      bash "$PSB_REPO/scripts/prepare-sandbox-build.sh" packet/fixture/test MUST_HAVE_ME --apply --build-only ) 2>&1
}
rm -f "$FIX/executed.log"
OUTB="$(run_psb)"; RCB=$?
[ "$RCB" != "0" ] && pass "两个旗标同时给时非零退出" || die "两个旗标同时给却退出 0 —— 结果取决于哪个分支后被求值,不取决于操作者要了什么"
printf '%s\n' "$OUTB" | grep -q "不能同时给" && pass "明说这两个不能同时给" || die "没说为什么拒绝"
# 🔴 数的是**破坏性**副作用,不是所有调用。pgrep 是只读的:脚本在动任何东西之前
#    本来就要报告「主版在不在跑」。第一版把任何 executed.log 条目都算「动手了」,
#    于是 pgrep 的那一行让这一条恒红 —— 而它证明的正好相反:守卫确实在副作用之前。
#    🔴 破坏性的那几个是 pnpm / tauri / open(还有真路径里的 pkill、rm、cp)。
if [ -f "$FIX/executed.log" ]; then
  HARMFUL="$(grep -E '^(PNPM|TAURI|OPEN|PKILL|RM|CP)' "$FIX/executed.log" || true)"
  if [ -n "$HARMFUL" ]; then
    die "守卫失守,破坏性动作已经执行:"; printf '%s\n' "$HARMFUL"|sed 's/^/      /'
  else
    pass "守卫在副作用之前就拦住了(只有只读的 pgrep 被调用,零破坏性副作用)"
  fi
else
  pass "守卫在副作用之前就拦住了(零副作用)"
fi
# 🔴 冲突必须在**参数解析完之后立刻**判,不能等工作区、身份、数据库都查过。
#    Flux 的 ev-wZhxxQuaY0xUrg7IHNF2X 实测到守卫原来在第 255 行,排在工作区解析之后 ——
#    于是一起给两个旗标时先撞上的是「找不到工作区」,操作者看到的和他改错的东西无关,
#    而针对守卫的判据也**不可伪证**:守卫删掉后那次运行照样非零、照样零副作用
#    (更早的失败先响),判据分不清「守卫拒了」和「别的拒了」。
#    所以这里用一个**什么前提都不满足**的形状再测一次:分支不存在、没有工作区、
#    没有构建产物、没有快照库。只有旗标冲突能在这种形状下给出那个理由。
OUTB3="$( cd "$PSB_REPO" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
  bash "$PSB_REPO/scripts/prepare-sandbox-build.sh" no-such-branch-at-all MUST_HAVE_ME --apply --build-only 2>&1 )"
RCB3=$?
[ "$RCB3" != "0" ] && pass "分支不存在时,旗标冲突仍然非零退出" || die "分支不存在却退出 0"
printf '%s\n' "$OUTB3" | grep -q "不能同时给" \
  && pass "分支不存在时,给出的仍然是旗标冲突这个理由(不是工作区的理由)" \
  || die "分支不存在时给的不是旗标冲突 —— 守卫排在别的检查后面,判据不可伪证"

# 反向:只给 --build-only 必须仍然可用 —— 证明上面不是「给了 --build-only 就一律拒绝」。
rm -f "$FIX/executed.log"
OUTB2="$( cd "$PSB_REPO" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_SANDBOX_BACKUP_ROOT="$FIX/backups" \
  bash "$PSB_REPO/scripts/prepare-sandbox-build.sh" --build-only packet/fixture/test MUST_HAVE_ME 2>&1 )"
printf '%s\n' "$OUTB2" | grep -q "不能同时给" \
  && die "只给 --build-only 也被拒了 —— 守卫把合法的调用形状一起挡了" \
  || pass "只给 --build-only 不受影响(守卫没有误伤)"

echo ""
echo "── 10. 变异:删掉那个守卫,第 9 条必须红 ──"
# 承重命题:第 9 条不是靠运气绿的。
MUT2="$PSB_REPO/scripts/prepare-sandbox-build.sh"   # 测试跑的就是这个拷贝
cp "$MUT2" "$FIX/mut2-orig.sh"
# 锚点单独放在一个文件里,因为这段测试本身已经是 bash,再嵌一层 heredoc 会让
# 引号层级失控(实测:嵌进去之后整个测试文件语法错误,而且报错行号离现场 60 行)。
cat > "$FIX/mut2.py" <<'PYEOF'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = 'if [ "$APPLY" = "1" ] && [ "$BUILD_ONLY" = "1" ]; then\n  fail "--apply 和 --build-only 不能同时给。--apply 会替换沙箱,--build-only 只构建。分开跑。"\nfi'
assert old in s, "变异锚点没找到 —— 第 10 条没有测到任何东西"
open(p, 'w', encoding='utf-8').write(s.replace(old, '# MUTATION: guard removed\n', 1))
PYEOF
if ! python3 "$FIX/mut2.py" "$MUT2"; then
  die "变异锚点没找到 —— 第 10 条什么都没测"
  cp "$FIX/mut2-orig.sh" "$MUT2"
else
  rm -f "$FIX/executed.log"
  OUTM2="$(run_psb)"; RCM2=$?
  if [ "$RCM2" = "0" ] || [ -s "$FIX/executed.log" ]; then
    pass "变异后守卫失守(退出 $RCM2)—— 第 9 条真的在守东西"
  else
    die "变异后第 9 条仍然绿 —— 它守的是空的"
  fi
  # 还原,并证明逐字节。比的对象是**仓库里那份**,不是循环内的副本 ——
  # 比副本证明的是「我刚才把它拷回了它自己」,那是恒真的。
  cp "$FIX/mut2-orig.sh" "$MUT2"
  cmp -s "$MUT2" "$REPO/scripts/prepare-sandbox-build.sh" || die "变异没还原干净 —— 后面的用例会被污染"
  rm -f "$FIX/mut2.py"
fi

echo ""
if [ "$FAIL" = "0" ]; then
  echo "✅ 全部通过"
  exit 0
fi
echo "❌ 有失败项"
exit 1
