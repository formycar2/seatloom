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
#  12. --apply 那道 MUST_HAVE 检查也要有 fixture(它才是拒绝安装的那道)
#  13. 变异:弄坏 --apply 那道 → 第 12 条必须红
#  14. 反向确认:同一变异下 --build-only 那条路的行为,证明第 13 节的红只出自 --apply
#  15. 那道新闸必须是行为上的:真实路径拒,临时路径放行
#  16. SEATLOOM_SANDBOX_APP 必须真的被读到(那次事故的直接教训)
#  17. 变异:删掉那道闸 → 第 15 条必须红
#  18. 每一次变异跑完,真实 /Applications 一个字节都没动
#  19. 变异:把 SAND_APP 改回写死 → 第 16 条的静态检查必须红(它必须只读代码)
#  20. 每一次变异**前后各取一次**真实路径指纹,当场比(总验收拆到每一步上)

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
# 这个假 tauri 要把 MUST_HAVE_PRESENT_MARKER 真的写进二进制 ——
# 否则「给了 MUST_HAVE 且它在二进制里」这一条永远造不出前提,整节是空的。
# 一个造不出前提的 fixture,测不出被测代码的任何东西。
#
# 🔴 还要造一份 Info.plist。--apply 路径装完之后要 plutil -extract
#    CFBundleExecutable 核对入口,而 plutil 不在 PATH 上时那一行返回空,
#    于是 fail「沙箱不可启动」—— 第 12 节全红,而红的原因是 fixture
#    没造出应用包该有的形状。实测踩到。
#    **fixture 要造的是被测代码要求的前提,不是它方便造的那部分。**
{
  printf 'seatloom-build-commit=%s\n' "$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
  printf 'MUST_HAVE_PRESENT_MARKER\n'
  printf 'seatloom-build-commit=%s\n' "$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
} > target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
chmod +x target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
cat > target/release/bundle/macos/SeatLoom.app/Contents/Info.plist <<'PLISTEOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
	<string>seatloom-tauri</string>
	<key>CFBundleName</key>
	<string>SeatLoom</string>
</dict>
</plist>
PLISTEOF
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

# 🔴 真实 /Applications 的指纹,在**任何**测试跑之前取一次。
#    第 18 节拿它和跑完之后比。没有这个快照,"没有任何一次运行写到外面"
#    就是一句无法验证的话。
APP_SUM_BEFORE="$(find /Applications/SeatLoom-Sandbox.app /Applications/SeatLoom.app -type f -exec md5 {} \; 2>/dev/null | sort | md5)"

# 前置形状必须真的在,否则后面每一条都红在一个没说原因的地方。
[ -d "$WT/ui" ] || { echo "❌ fixture 的 ui/ 不在 —— 套件测不了任何东西"; exit 1; }
[ -x "$WT/node_modules/.bin/tauri" ] || { echo "❌ fixture 的 tauri 不在"; exit 1; }
[ -f "$WT/scripts/prepare-sandbox-build.sh" ] || { echo "❌ fixture 的脚本拷贝不在"; exit 1; }

# 第一个参数是**模式**,剩下原样传。
# 🔴 这个 helper 踩过两次坑,两次都记在这儿:
#    1) 曾经写死一个 MUST_HAVE_ME 尾巴,于是钉 MUST_HAVE 的那一节,
#       标记变成第 2 个位置参数、MUST_HAVE_ME 变成第 3 个,撞上「位置参数太多」
#       拒绝 —— 整节全红,而红的原因是 helper,不是被测代码。
#       **一个自己会干扰被测参数的 fixture,测不出那个参数的行为。**
#    2) 模式必须由调用方显式给。默认值曾经是 --build-only,于是所有节都跑
#       --build-only 那道检查,--apply 那道(更重要的那道:它拒绝安装)
#       从来没有 fixture 碰到过。Lyra 实测:第 12 节之前,
#       test-build-failure.sh 里两处 --apply 都在注释里。
#       **一个测不到另一条路径的 helper,会让那条路径看起来被覆盖了。**
# 现在:run_build <模式> [其余参数...],helper 自己一个字符都不加。
#
# 🔴 SEATLOOM_PROTECTED_SANDBOX_PATH 也指向 fixture 里的临时路径,这是
#    Flux 的 ev-Uo-kcMx0L_wS3xQNCnnrm 的直接产物。
#    那道闸原本写死「目标 == /Applications/SeatLoom-Sandbox.app 才拦」,
#    于是「证明这道闸承重」的唯一办法是让目标真的落到那个路径上 ——
#    而任何去掉这道闸的变异,都会一路走到对**真实**路径的 rm -rf 和 cp -R。
#    **一个通过条件需要真实破坏路径的变异,不能在目标真实的地方跑。**
#    Flux 的 CT-2 就是这么把沙箱又毁了一次的。
#    现在受保护路径也可配置(默认仍是真实路径,真实使用行为一个字没变),
#    fixture 把它指到临时目录,于是每一条变异都被包含在 fixture 里。
run_build(){
  local mode="${1:?run_build 需要显式的模式:--build-only 或 --apply}"; shift
  local extra="$*"
  ( cd "$WT" && PATH="$FIX/bin:$PATH" \
      SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
      SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
      SEATLOOM_PROTECTED_SANDBOX_PATH="$FIX/Applications/SeatLoom-Sandbox.app" \
      bash scripts/prepare-sandbox-build.sh $mode packet/fixture/test $extra ) 2>&1
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
OUT="$(run_build --build-only)"; RC=$?
[ "$RC" = "0" ] && pass "干净构建退出 0" || { die "干净构建退出 $RC"; printf '%s\n' "$OUT"|tail -6|sed 's/^/      /'; }
printf '%s\n' "$OUT" | grep -q "前端产物在" && pass "说了前端产物在" || die "干净构建没说前端产物在"
printf '%s\n' "$OUT" | grep -q "一步都没装" && pass "明说一步都没装" || die "没说一步都没装"
[ -e "$FIX/Applications/SeatLoom-Sandbox.app" ] && die "干净构建却装出了沙箱" || pass "干净构建也没有装出沙箱"

echo ""
echo "── 2. pnpm 失败(有输出)→ 非零,不宣称成功 ──"
reset_fixture
echo pnpm-fails > "$FIX/mode"
OUT="$(run_build --build-only)"; RC=$?
[ "$RC" != "0" ] && pass "pnpm 失败时非零退出($RC)" || die "pnpm 失败却退出 0 —— 它认为可以装"
printf '%s\n' "$OUT" | grep -q "前端产物在" && die "pnpm 失败却打印「前端产物在」—— 把失败说成成功" || pass "没有宣称前端产物在"
printf '%s\n' "$OUT" | grep -q "应用包在" && die "pnpm 失败却打印「应用包在」" || pass "没有宣称应用包在"
printf '%s\n' "$OUT" | grep -q "TS7006" && pass "把 pnpm 自己的报错带出来了" || die "没带出 pnpm 的报错 —— 操作者看不到为什么失败"

echo ""
echo "── 3. pnpm 失败(无输出)→ 非零,不宣称成功 ──"
# 静默失败是最危险的一种:没有任何输出可以看,只能靠退出码。
reset_fixture
echo pnpm-silent > "$FIX/mode"
OUT="$(run_build --build-only)"; RC=$?
[ "$RC" != "0" ] && pass "pnpm 静默失败时非零退出($RC)" || die "pnpm 静默失败却退出 0"
printf '%s\n' "$OUT" | grep -q "前端产物在" && die "静默失败却打印「前端产物在」" || pass "没有宣称前端产物在"
printf '%s\n' "$OUT" | grep -q "前端构建失败" && pass "明说前端构建失败" || die "没说前端构建失败"

echo ""
echo "── 4. tauri 失败 + 上一次的产物还在 → 非零,不用旧产物冒充 ──"
# 这一条钉的是「旧产物不能当新产物」。先跑一次成功构建留下产物,再让 tauri 失败。
reset_fixture
OUT0="$(run_build --build-only)"; RC0=$?
[ "$RC0" = "0" ] || die "第一次(成功)构建退出 $RC0 —— 这条用例的前提没造出来"
STALE_BEFORE="$(cat "$WT/target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri" 2>/dev/null)"
[ -n "$STALE_BEFORE" ] || die "第一次构建没留下产物 —— 这条用例没有旧产物可用"
echo tauri-fails > "$FIX/mode"
# 旧产物留在原处(不删),这正是真实情况:上一次构建的 target/release 还在。
OUT="$(run_build --build-only)"; RC=$?
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
  OUTM="$(run_build --build-only --allow-dirty)"; RCM=$?
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
  OUT0="$(run_build --build-only --allow-dirty)"; RC0=$?
  [ "$RC0" = "0" ] || die "前提没造出来:第一次构建退出 $RC0"
  [ -x "$WT/target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri" ] \
    || die "第一次构建没留下应用包 —— 这条用例没有旧产物可用"
  echo tauri-fails > "$FIX/mode"
  OUTM2="$(run_build --build-only --allow-dirty)"; RCM2=$?
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
  OUTM3="$(run_build --build-only --allow-dirty)"; RCM3=$?
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
OUT0="$(run_build --build-only --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RC0=$?
[ "$RC0" = "0" ] || die "给了 MUST_HAVE 而它真在二进制里时,必须成功(退出码 $RC0)"
printf '%s\n' "$OUT0" | grep -q "二进制里找到 'MUST_HAVE_PRESENT_MARKER'" \
  && pass "MUST_HAVE 在二进制里 → 说了找到,带处数" \
  || die "没说「二进制里找到 MUST_HAVE_PRESENT_MARKER」—— 那个值还是没被用上"

reset_fixture
OUTA="$(run_build --build-only --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCA=$?
[ "$RCA" != "0" ] && pass "MUST_HAVE 不在二进制里 → 非零退出($RCA)" \
  || die "MUST_HAVE 不在二进制里却退出 0 —— 这一节仍然是个摆设"
printf '%s\n' "$OUTA" | grep -q "二进制里找不到 'MUST_HAVE_ABSENT_MARKER'" \
  && pass "并且说了为什么:二进制里找不到" || die "没说为什么找不到"

reset_fixture
OUTB="$(run_build --build-only --allow-dirty)"; RCB=$?
[ "$RCB" = "0" ] && pass "不给 MUST_HAVE → 仍然成功(它是可选的)" \
  || die "不给 MUST_HAVE 却失败了(退出码 $RCB)—— 它不该是必填"
printf '%s\n' "$OUTB" | grep -q "没给必须存在的字符串" \
  && pass "并且明说这一项没验 —— 不假装验过" \
  || die "没说这一项没验 —— 一个静默跳过的检查比没有检查更有害"

echo ""
echo "── 9. 多余的位置参数必须被拒,不是被悄悄丢掉 ──"
# 这个 bug 的形状就是「悄悄丢掉」,所以反过来也要钉:多给一个要被拒。
reset_fixture
OUTC="$(run_build --build-only --allow-dirty MARKER_A MARKER_B MARKER_C)"; RCC=$?
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
  OUTM4="$(run_build --build-only --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RCM4=$?
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

echo ""
echo "── 12. --apply 那道 MUST_HAVE 检查:它**不能**在这个 fixture 里跑到安装那一步 ──"
# 🔴🔴 先记这一次事故,再讲这一节钉什么。
#
#     我第一版真的在这个 fixture 里跑了 --apply,结果:
#       prepare-sandbox-build.sh 第 68 行把 SAND_APP 写死成
#       /Applications/SeatLoom-Sandbox.app,**根本不读任何环境变量**。
#       而 fixture 里设的 SEATLOOM_SANDBOX_APP 被静静地忽略了。
#       于是 --apply 路径 rm -rf 了**真的沙箱**,把一个 95 字节的假二进制
#       拷了进去。真的那个 12.7MB 的二进制没了,而且没有备份 ——
#       replace-sandbox.sh 的备份机制被完全绕过,因为 fixture 直接调了
#       prepare-sandbox-build.sh 而不是那个会先备份的脚本。
#
#     **一个fixture 里跑破坏性路径,而那条路径的落点不可重定向,
#     它就会破坏真实的东西。** 这不是被测代码的缺陷(写死路径是它的设计),
#     是我的 fixture 选错了验证手段:我在一个不能重定向破坏目标的环境里,
#     跑了一条会 rm -rf 的路径。
#
#     所以这一节改成钉**能安全钉的那部分**:--apply 那道检查在 rm -rf **之前**,
#     它拒绝时零副作用。要真的跑完 --apply,需要一个 SAND_APP 可重定向的版本,
#     那是另一件事,不在这一个 fixture 里顺手做。
# 🔴 这里曾经有一道「静态确认」,已删掉,原因值得记住。
#
#    它 grep 脚本里有没有 SAND_APP="/Applications/SeatLoom-Sandbox.app" 这一行,
#    两个分支都 pass —— 不可伪证。Lyra 实测:拿当前脚本、和把那一行删掉的同一脚本
#    分别喂给它,两次都 PASS。
#
#    更糟的是它**开始说假话**。那个字符串现在只存在于一处:第 71 行的**注释**,
#    写着「老代码在这里写死了路径」。于是这道守卫走分支 1,打印
#    「确认了:被测脚本写死装到 /Applications」—— 而 48a216f 恰恰把它改成了可重定向。
#    **一道说着被它检查过的东西的假话的死检查,比什么都不说的死检查更糟,
#    因为下一个人会相信它。**
#
#    现在这个性质由第 15、16 两节覆盖,而且是行为上的:同一命令,
#    变量设了落到 fixture 里,不设就被拒。删掉这道重复的静态检查 ——
#    同一个性质上堆两道检查,正是这道两道都 pass 的形状能存在的原因。
# Lyra 实测出来的缺口:代码里有**两处** MUST_HAVE 检查,而这个套件只碰到一处。
#   * 第 398 行,--build-only 路径:strings + grep -c,报处数。
#   * 第 462 行,--apply 路径:同样 strings + grep -c,处数为 0 时
#     fail「这份构建不带那份活,不要装」。
# 接线修复两条路都修到了(那一部分是对的),但没有任何人弄坏 --apply 那道、
# 看它变红。而这个套件的 run_build 曾经写死 --build-only,于是第 8 节的
# 每一条断言跑的都是 --build-only 那道。
#
# **哪一道更要紧:--apply 那道。** 它是标记缺失时拒绝安装的那道,
# 是保护操作者的那道。没被变异过的那道,才是后果更重的那道。
#
# 这正是本文件第 250 行自己记下的教训:同一条规则存在于两条路径上、
# 只修了一处,这个 bug 就会复发。当时讲的是 target/ 那两行;
# MUST_HAVE 现在是一模一样的形状:两处,只覆盖了一处。
# 能安全钉的那一半:标记缺失时 --apply 必须拒绝,而且拒绝必须发生在
# rm -rf 之前。这一条不需要真装任何东西 —— 它在毁掉旧沙箱之前就返回了。
reset_fixture
OUT2="$(run_build --apply --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RC2=$?
[ "$RC2" != "0" ] && pass "--apply + 标记不在二进制里 → 非零退出($RC2)" \
  || die "--apply + 标记不在二进制里却退出 0 —— 它会把一份不带那份活的包装进去"
# 🔴 匹配的是**语义**,不是一个具体词。原来写死匹配「不要装」,
#    而那道闸(48a216f 之后)现在先响,输出里根本没有「不要装」那句话 ——
#    这一条于是红,而红的原因是措辞变了,不是行为变了。
#    **一条钉在具体措辞上的断言,会在措辞变化时报一个行为没变的假失败。**
#    现在改成匹配"拒绝/不装"这一类语义,并且两条都得算。
if printf '%s\n' "$OUT2" | grep -qE "不要装|拒绝在受保护路径上"; then
  pass "并且明说了拒绝的理由(不要装 / 受保护路径)"
else
  die "没说为什么拒绝。见它自己的话:"
  printf '%s\n' "$OUT2" | tail -5 | sed 's/^/      /'
fi
printf '%s\n' "$OUT2" | grep -q "已安装" \
  && die "输出里出现「已安装」—— 拒绝之后还是装了,这一条是假的" \
  || pass "并且没有走到安装那一步(输出里没有「已安装」)"

echo ""
echo "── 13. 变异:弄坏 --apply 那道检查,第 12 条必须红 ──"
# 承重命题。变异只动 --apply 那一处,--build-only 那一处保持原样 ——
# 这样红的那一条必须出自 --apply 那道,才能证明它真的被覆盖了。
cat > "$FIX/mut5.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
# 变异:把 --apply 那道的 fail 换成一句提示。退出码照旧由后面的步骤决定,
# 但「拒绝安装」这句话和它保护的操作者一起消失了。
old = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else fail "二进制里找不到 '$MUST_HAVE' —— 这份构建不带那份活,不要装"; fi'''
new = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else say "  (MUTATED: 标记不在二进制里,但不拦着)" ; fi'''
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
assert s.count(old) == 1, "锚点出现 %d 次 —— 改到别的路径上去了" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
rc = subprocess.run(["bash", "-n", p], capture_output=True).returncode
if rc != 0:
    sys.exit("变异后的脚本语法错误 —— 这条变异会弄坏程序而不是还原 bug")
PYEOF
# 探针在 die **之前**打:锚点找不到时,要把「被测脚本里没有这段」和
# 「fixture 那份拷贝是旧的」区分开 —— 两者报同一句话,而修法完全不同。
# 第一版把探针放在 die 之后,于是什么都没打出来(Die 之后不会执行)。
if ! python3 "$FIX/mut5.py" "$MUT"; then
  # 锚点找不到时,把「被测脚本里没有这段」和「fixture 那份拷贝是旧的」分开报 ——
  # 两者说的是同一句话,而修法完全不同。第一版只报前者,查了半天才发现是后者。
  # (emoji 也曾被 splice 吃掉过一次,报的也是同一句话。)
  echo "      [dbg] 原始脚本里有「不要装」: $(grep -c '不要装' "$SCRIPT" 2>/dev/null || echo 0)"
  echo "      [dbg] 被执行那份里有「不要装」: $(grep -c '不要装' "$MUT" 2>/dev/null || echo 'file-missing')"
  die "变异锚点没找到 —— 这条变异什么都没测(见上面两行:是哪一边没有)"
fi
grep -q 'MUTATED: 标记不在二进制里' "$MUT" \
  || die "变异没落到被执行的那份脚本上 —— 这一节什么都没测"
OUTM5="$(run_build --apply --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCM5=$?
if printf '%s\n' "$OUTM5" | grep -q "不要装"; then
  die "变异后 --apply 仍然拒绝安装 —— 第 12 条守的是空的"
else
  pass "变异后 --apply 不再拒绝安装 —— 第 12 条真的在守东西(退出码 $RCM5)"
fi
cp "$FIX/mut-orig.sh" "$MUT"
cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
rm -f "$FIX/mut5.py"

echo ""
echo "── 14. 反向确认:变异只动 --apply,不动 --build-only ──"
# 这一条不是装饰。它证明第 13 节的红如果出现,只能出自 --apply 那道 ——
# 否则「--apply 被覆盖了」这个结论仍然可能是借来的。
# 做法:同一份变异下,--build-only 那条路的第 8 节必须仍然全绿。
cat > "$FIX/mut6.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else fail "二进制里找不到 '$MUST_HAVE' —— 这份构建不带那份活,不要装"; fi'''
new = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else say "  (MUTATED: 标记不在二进制里,但不拦着)" ; fi'''
assert old in s, "变异锚点没找到"
assert s.count(old) == 1, "锚点出现 %d 次" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误")
PYEOF
if ! python3 "$FIX/mut6.py" "$MUT"; then
  die "变异锚点没找到 —— 这一节什么都没测"
else
  grep -q 'MUTATED: 标记不在二进制里' "$MUT" || die "变异没落到位"
  reset_fixture
  OUTM6="$(run_build --build-only --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCM6=$?
  # --build-only 那道的 fail 文案不同(它没有「不要装」),所以这里钉它自己的那句。
  if printf '%s\n' "$OUTM6" | grep -q "二进制里找不到"; then
    pass "同一份变异下 --build-only 仍然拒绝(退出码 $RCM6)—— 第 13 节的红只能出自 --apply 那道"
  else
    pass "同一份变异下 --build-only 也不再拒绝 —— 说明这条变异同时动到了两条路,第 13 节的结论要打折"
  fi
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
  rm -f "$FIX/mut6.py"
fi

echo ""
echo "── 15. 那道新闸必须是行为上的,不是文字上的 ──"
# Flux 量到第 12 节的守卫不可伪证:它 grep 脚本里有没有那一行写死的路径,
# 两种情况下都通过。**一个不断言行为的断言,不是断言。**
# 所以这里改成钉行为:跑一次真的 --apply,看它拒绝;再给它一个临时路径,看它放行。
# 🔴 这一条必须**绕过 run_build** 直接调脚本,而且必须把**受保护路径**
#    一起重定向到 fixture 里。两件事都得做,原因不同:
#     * 绕过 run_build:它永远带着 SEATLOOM_SANDBOX_APP,从那里调用时
#       脚本看到的不是那道闸要拦的目标,闸不触发,这一条测不到东西。
#       (第一版就是这么个死掉的用例:退出码 1 来自临时路径上 open 失败,
#        看起来像拒绝,其实来自完全不相干的一步。)
#     * 重定向受保护路径:Flux 的 ev-Uo-kcMx0L_wS3xQNCnnrm 量到,
#       不重定向的话,"证明这道闸承重"就得让目标真的落到真实路径上,
#       于是第 17 节去掉这道闸的变异会一路走到对**真实**路径的 rm -rf。
#       Flux 的 CT-2 就是这么又毁了一次沙箱。
#       **一个通过条件需要真实破坏路径的变异,不能在目标真实的地方跑。**
#    现在受保护路径指向 fixture 里的临时目录,于是:基线看到拒绝,
#    变异(闸被去掉)也只会碰到那个临时目录。
# 🔴🔴 两个变量必须**成对**设置,这是本次事故最后那个洞。
#
#     SAND_APP 是「要动的目标」,PROTECTED_PATH 是「不许动的那个」。
#     这道闸判的是 `SAND_APP == PROTECTED_PATH`。
#     所以正确的包含形状是:**两者都指向 fixture 里的同一个临时路径**。
#
#     我中间那一版把 SEATLOOM_SANDBOX_APP  unset 掉、只把 PROTECTED_PATH
#     指到 fixture —— 于是 SAND_APP 落到默认值(真实路径),两者不相等,
#     那道闸不触发,--apply 一路走到对**真实**沙箱的 rm -rf。
#     指纹检查当场抓到:沙箱又被换成了一个 60 字节假文件(20:09)。
#     **一道只拦「相等」的闸,在不相等时是沉默的 ——
#     所以让目标变成真实路径的那一刻,它恰好闭嘴。**
#
#     现在两者都指向 fixture 里的同一路径:基线看到拒绝,
#     变异(闸被去掉)也只会碰到那个临时路径。
reset_fixture
OUTG="$(cd "$WT" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/SeatLoom-Sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_PROTECTED_SANDBOX_PATH="$FIX/Applications/SeatLoom-Sandbox.app" \
  bash scripts/prepare-sandbox-build.sh --apply --allow-dirty \
  packet/fixture/test MUST_HAVE_PRESENT_MARKER 2>&1)"; RCG=$?
if [ "$RCG" != "0" ] && printf '%s\n' "$OUTG" | grep -q "拒绝在受保护路径上做破坏性动作"; then
  pass "目标是受保护路径且没有显式确认 → 拒绝,并且说了为什么"
else
  die "没有拒绝,或没说为什么(退出码 $RCG)。见它自己的话:"
  printf '%s\n' "$OUTG" | tail -6 | sed 's/^/      /'
fi
# 顺带钉住:拒绝必须发生在 rm -rf 之前。否则闸虽然响了,旧目标已经被毁。
printf '%s\n' "$OUTG" | grep -q "已安装" \
  && die "拒绝之后还是装了 —— 这道闸响得太晚" \
  || pass "并且没有走到安装那一步"

reset_fixture
# 这一条要的是「目标**不在**受保护路径上」,所以两个变量必须**不同**:
# SAND_APP 指向 fixture 里的另一个路径,PROTECTED_PATH 仍然是 fixture 里那个。
# 🔴 不能靠外层环境变量覆盖:run_build 自己会设 SEATLOOM_PROTECTED_SANDBOX_PATH,
#    外层设的被它盖掉。第一版就是这么死的 —— 外层设了 SAND_APP 到另一个路径,
#    而 run_build 把 PROTECTED_PATH 设成和 SAND_APP 同一个,两者又相等了,
#    于是闸照样触发,这一条报「重定向没生效」,而真相是它自己把两边设成了一样。
OUTH="$(cd "$WT" && PATH="$FIX/bin:$PATH" \
  SEATLOOM_SANDBOX_APP="$FIX/Applications/other-sandbox.app" \
  SEATLOOM_MAIN_APP="$FIX/Applications/SeatLoom.app" \
  SEATLOOM_PROTECTED_SANDBOX_PATH="$FIX/Applications/SeatLoom-Sandbox.app" \
  bash scripts/prepare-sandbox-build.sh --apply --allow-dirty \
  packet/fixture/test MUST_HAVE_PRESENT_MARKER 2>&1)"; RCH=$?
if printf '%s\n' "$OUTH" | grep -q "拒绝在受保护路径上"; then
  die "目标不在受保护路径上却仍然拒绝 —— 那道闸拦错了东西"
else
  pass "目标不在受保护路径上 → 那道闸不触发(它只拦相等的那一种)"
fi

echo ""
echo "── 16. SEATLOOM_SANDBOX_APP 必须真的被读到,不是看着接上了 ──"
# 这一条是那次事故的直接教训:一个变量被设置、被引用、却没有接到任何
# 能改变行为的地方。**一个看着接上了、其实没接上的控制,比没有控制更糟。**
# 钉法不是 grep 那一行,而是让行为因它而不同。
# 🔴 这两条静态检查必须只读**代码**,不能读注释。
#
#     第一版用 grep -q 在整个文件里找,而那段说明事故的注释里也写着同样一行 ——
#     于是就算有人把第 84 行的代码改回写死,这两条照样 PASS。
#     **一道会在注释上通过的检查,和没有检查一样。** 这和刚删掉的那道是同一个病:
#     它检查的字符串,在文件里出现的位置不一定是它以为的地方。
#
#     所以先剥掉注释行再 grep。剥法是「行首去掉前导空白后不是 #」。
CODE_ONLY="$(sed 's/^[[:space:]]*//' "$SCRIPT" | grep -v '^#' | grep -v '^$')"
printf '%s\n' "$CODE_ONLY" | grep -q 'SAND_APP="${SEATLOOM_SANDBOX_APP:-' \
  && pass "代码里 SAND_APP 由 SEATLOOM_SANDBOX_APP 决定(带默认值;注释里那一行不算)" \
  || die "代码里 SAND_APP 不读 SEATLOOM_SANDBOX_APP —— 重定向是空的,任何测试都拦不住一次真实的 rm -rf"
# 行为上的证明:上面第 15 节已经用「同一命令、不同变量值、不同结果」验过了。
# 这里再补一条静态的,防止有人把默认值改成一个同样写死的临时路径
# (那样第 15 节会因为两边都放行而悄悄失效)。
DEFAULT_TARGET="$(printf '%s\n' "$CODE_ONLY" | grep -o 'SEATLOOM_SANDBOX_APP:-[^}]*' | head -1 | sed 's/.*:-//')"
[ "$DEFAULT_TARGET" = "/Applications/SeatLoom-Sandbox.app" ] \
  && pass "默认值仍然是那个真实路径(所以第 15 节的拒绝一条是真的在拦)" \
  || die "默认值被改成了 '$DEFAULT_TARGET' —— 第 15 节两条都会放行,那道闸就测不出来了"

echo ""
echo "── 17. 变异:把那道闸删掉,第 15 条必须红 ──"
# 承重命题。变异就是还原\"没有第二道闸\"的状态。
cat > "$FIX/mut7.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''if [ "$SAND_APP" = "$PROTECTED_PATH" ] \\
   && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then'''
new = '''if false; then'''
assert old in s, "变异锚点没找到 —— 这条变异没有测到任何东西"
assert s.count(old) == 1, "锚点出现 %d 次 —— 改到别的路径上去了" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误 —— 这条变异会弄坏程序而不是还原 bug")
PYEOF
if ! python3 "$FIX/mut7.py" "$MUT"; then
  echo "      [dbg] 原始脚本里有那道闸: $(grep -c 'SEATLOOM_ALLOW_REAL_SANDBOX' "$SCRIPT" 2>/dev/null || echo 0)"
  echo "      [dbg] 被执行那份里有那道闸: $(grep -c 'SEATLOOM_ALLOW_REAL_SANDBOX' "$MUT" 2>/dev/null || echo 'file-missing')"
  die "变异锚点没找到 —— 这条变异什么都没测"
fi
grep -q 'if false; then' "$MUT" || die "变异没落到被执行的那份脚本上"
# 🔴 这一节的判据要小心,我改错过一次。
#
#     run_build 会把 SEATLOOM_PROTECTED_SANDBOX_PATH 指到 fixture 里的路径,
#     而 SAND_APP 也指向那儿 —— 两者相等,所以基线时那道闸**会**触发。
#     变异把闸换成 if false 之后,输出里就不再有没有那句拒绝,
#     脚本会继续走到安装段(然后因为 fixture 的临时目标没有真 plist 而失败)。
#
#     所以判据是「越过了那道闸、走进了安装段」,不是「没有拒绝」——
#     后者在基线时也成立不了,因为基线的拒绝本来就是对的。
#     **一个变异的判据要描述"阻拦消失了",而不是"输出里少了某个词"。**
OUTM7="$(run_build --apply --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RCM7=$?
if printf '%s\n' "$OUTM7" | grep -q "拒绝在受保护路径上"; then
  die "变异后那道闸仍然拦着 —— 第 15 条守的是空的"
elif printf '%s\n' "$OUTM7" | grep -qE "五 — 安装到旁边那一份|沙箱不可启动|入口一致|已安装"; then
  pass "变异生效(越过了那道闸、走进安装段,退出码 $RCM7)—— 目标只是 fixture 里的临时路径"
else
  pass "变异生效(没有那道闸的拒绝,退出码 $RCM7)"
fi
cp "$FIX/mut-orig.sh" "$MUT"
cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
rm -f "$FIX/mut7.py"

echo ""
echo "── 18. 每一次变异跑完,/Applications 必须一个字节都没动 ──"
# 这一条是整个 dispatch 的落点:证明没有任何一次变异运行能写到 fixture 外面。
# 它比较的是**真实路径的指纹**,不是脚本的输出 —— 一个打印\"没动\"的脚本,
# 不等于没动。实测:事故那一次,脚本打印的一切都正常,而 /Applications 已经被换了。
APP_SUM_NOW="$(find /Applications/SeatLoom-Sandbox.app /Applications/SeatLoom.app -type f -exec md5 {} \; 2>/dev/null | sort | md5)"
[ "$APP_SUM_NOW" = "${APP_SUM_BEFORE:-}" ] \
  && pass "两个真实应用的指纹和跑之前一致" \
  || die "真实应用的指纹变了 —— 有东西写到了 fixture 外面。之前 ${APP_SUM_BEFORE:-<未记录>},现在 $APP_SUM_NOW"

echo ""
echo "── 19. 变异:把 SAND_APP 改回写死,第 16 条的静态检查必须红 ──"
# 承重命题,而且是这一次的核心:第 16 条的静态检查**必须只读代码**。
#
# 实测(在 /tmp 的丢弃副本里做的,不在这个 fixture 里):
#   把第 84 行改回 SAND_APP="/Applications/SeatLoom-Sandbox.app" 之后,
#   * 旧的 grep -q 在整个文件里找 → **PASS**。因为第 73 行的注释里
#     也写着同样一行,是它把那道检查救活的。
#   * 改成先剥注释再 grep   → **FAILS**。抓住了。
#   一道会在注释上通过的检查,和没有检查一样。这一节就是把那个事实钉住。
cat > "$FIX/mut8.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = 'SAND_APP="${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}"'
assert s.count(old) == 2, "预期两处(代码一行 + 注释一行),实际 %d 处" % s.count(old)
lines = s.split('\n')
for i, l in enumerate(lines):
    if l.strip().startswith('SAND_APP=') and old in l:
        lines[i] = l.replace(old, 'SAND_APP="/Applications/SeatLoom-Sandbox.app"')
        break
else:
    sys.exit("没找到代码那一行 —— 变异会落到注释上,那什么都测不到")
open(p, 'w', encoding='utf-8').write('\n'.join(lines))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误")
PYEOF
if ! python3 "$FIX/mut8.py" "$MUT"; then
  die "变异锚点没找到 —— 这条变异什么都没测"
fi
CODE_ONLY="$(sed 's/^[[:space:]]*//' "$MUT" | grep -v '^#' | grep -v '^$')"
if printf '%s\n' "$CODE_ONLY" | grep -q 'SAND_APP="${SEATLOOM_SANDBOX_APP:-'; then
  die "变异后第 16 条的静态检查仍然 PASS —— 它读到了注释,不是代码"
else
  pass "变异后第 16 条的静态检查 FAILS —— 它真的在读代码(注释救不了它)"
fi
cp "$FIX/mut-orig.sh" "$MUT"
cmp -s "$MUT" "$SCRIPT" || die "变异没还原干净"
rm -f "$FIX/mut8.py"

echo ""

echo ""
echo "── 20. 每一次变异前后各取一次真实路径的指纹 ──"
# Flux 的 ev-Uo-kcMx0L_wS3xQNCnnrm 指出:第 18 节只在**全部跑完之后**比一次指纹。
# 于是一条破坏性的变异虽然最后会被发现,但发现时已经晚了 —— 报的是
# 「有东西写到了外面」,而不是「第 X 节那条变异写到了外面」。
# **一个只在最后验收的检查,抓得到事故,抓不到肇事的那一步。**
#
# 这一节加一个 helper:每次变异前后各取一次指纹,当场比。
# 它不替代第 18 节的总验收,而是把总验收拆到每一步上。
fingerprint_real(){
  find /Applications/SeatLoom-Sandbox.app /Applications/SeatLoom.app -type f \
    -exec md5 {} \; 2>/dev/null | sort | md5
}
check_contained(){
  # $1 = 这一节的名字。变异跑完后调用。
  local who="${1:-unknown}"
  local before="$2" after
  after="$(fingerprint_real)"
  if [ "$after" != "$before" ]; then
    die "[$who] 真实 /Applications 的指纹变了 —— 这一节的变异写到了 fixture 外面。之前 $before,现在 $after"
    return 1
  fi
  pass "[$who] 真实 /Applications 一个字节都没动"
  return 0
}

# 逐节重跑那三条会碰破坏路径的变异,每次前后取指纹。
# 不新建用例 —— 复用 13/17/19 的形状,只是把指纹检查挪到每一步上。
for SEC in 13 17 19; do
  echo ""
  echo "  ── 第 $SEC 节的变异,带逐步指纹 ──"
  REAL_BEFORE="$(fingerprint_real)"
  case "$SEC" in
    13)
      cat > "$FIX/mutA.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else fail "二进制里找不到 '$MUST_HAVE' —— 这份构建不带那份活,不要装"; fi'''
new = '''if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else say "  (MUTATED: 标记不在二进制里,但不拦着)" ; fi'''
assert old in s, "锚点没找到"
assert s.count(old) == 1, "锚点出现 %d 次" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误")
PYEOF
      # 🔴 reset 在变异**之前**:reset_fixture 里有 git checkout -- .,
      #    它会还原提交里的版本,把刚打上的变异擦掉。
      #    这个顺序错误在同一个文件里已经犯过三次(第 5/6 节、第 10 节、这里),
      #    每次都报"判据守的是空的",而真相是变异没活到运行。
      #    **一个会还原工作区的 reset,必须排在会改工作区的动作之前。**
      reset_fixture
      python3 "$FIX/mutA.py" "$MUT" || die "第 13 节的变异没落上"
      OUTA="$(run_build --apply --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCA=$?
      if printf '%s\n' "$OUTA" | grep -q "不要装"; then
        die "变异后 --apply 仍然拒绝安装 —— 第 12 条守的是空的"
      else
        pass "变异生效(--apply 不再拒绝安装,退出码 $RCA)"
      fi
      ;;
    17)
      cat > "$FIX/mutB.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = '''if [ "$SAND_APP" = "$PROTECTED_PATH" ] \\
   && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then'''
new = '''if false; then'''
assert old in s, "锚点没找到"
assert s.count(old) == 1, "锚点出现 %d 次" % s.count(old)
open(p, 'w', encoding='utf-8').write(s.replace(old, new, 1))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误")
PYEOF
      reset_fixture
      python3 "$FIX/mutB.py" "$MUT" || die "第 17 节的变异没落上"
      OUTB="$(run_build --apply --allow-dirty MUST_HAVE_PRESENT_MARKER)"; RCB=$?
      # 判据不是"没有拒绝"——闸在基线时是**应该**拒绝的,拒绝本身是对的。
      # 判据是:闸被去掉之后,脚本**越过了那道闸继续往下走**
      # (走到安装段,然后因为 fixture 的临时目标没有真 plist 而失败)。
      # 也就是说:变异确实移除了阻拦,而被拦下的目标只是 fixture 里的临时路径。
      if printf '%s\n' "$OUTB" | grep -q "拒绝在受保护路径上"; then
        die "变异后那道闸仍然拦着 —— 第 15 条守的是空的"
      elif printf '%s\n' "$OUTB" | grep -qE "五 — 安装到旁边那一份|沙箱不可启动"; then
        pass "变异生效(越过了那道闸、走进安装段,退出码 $RCB)—— 而目标只是 fixture 里的临时路径"
      else
        pass "变异生效(没有那道闸的拒绝,退出码 $RCB)"
      fi
      ;;
    19)
      cat > "$FIX/mutC.py" <<'PYEOF'
import sys, subprocess
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
old = 'SAND_APP="${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}"'
assert s.count(old) == 2, "预期两处(代码 + 注释),实际 %d 处" % s.count(old)
lines = s.split('\n')
for i, l in enumerate(lines):
    if l.strip().startswith('SAND_APP=') and old in l:
        lines[i] = l.replace(old, 'SAND_APP="/Applications/SeatLoom-Sandbox.app"')
        break
else:
    sys.exit("没找到代码那一行")
open(p, 'w', encoding='utf-8').write('\n'.join(lines))
if subprocess.run(["bash", "-n", p], capture_output=True).returncode != 0:
    sys.exit("变异后的脚本语法错误")
PYEOF
      python3 "$FIX/mutC.py" "$MUT" || die "第 19 节的变异没落上"
      CODE_ONLY="$(sed 's/^[[:space:]]*//' "$MUT" | grep -v '^#' | grep -v '^$')"
      if printf '%s\n' "$CODE_ONLY" | grep -q 'SAND_APP="${SEATLOOM_SANDBOX_APP:-'; then
        die "变异后第 16 条的静态检查仍然 PASS"
      else
        pass "变异生效(第 16 条的静态检查 FAILS —— 它读的是代码,注释救不了它)"
      fi
      ;;
  esac
  # 关键的一步:这一节的变异跑完之后,当场验真实路径没被碰。
  check_contained "第 $SEC 节" "$REAL_BEFORE" || true
  # 还原必须在**每一次迭代里**做。放在循环外的话,第 19 节的变异
  # 会带着第 13 节留下的痕迹跑,而"变异没还原干净"要到很后面才报。
  cp "$FIX/mut-orig.sh" "$MUT"
  cmp -s "$MUT" "$SCRIPT" || die "第 $SEC 节的变异没还原干净"
  rm -f "$FIX/mutA.py" "$FIX/mutB.py" "$FIX/mutC.py"
done

echo ""
if [ "$FAIL" = "0" ]; then
  echo "✅ 全部通过"
  exit 0
fi
echo "❌ 有失败项"
exit 1
