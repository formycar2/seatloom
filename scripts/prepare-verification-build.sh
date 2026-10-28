#!/usr/bin/env bash
# prepare-verification-build.sh — 开一张验收卡的同时,给张先生一份能点开就验的程序。
#
# 为什么必须有这个脚本(2026-10-16 张先生指出的死锁):
#   验收卡的规矩是「只在他点接受时才合并」—— 那是为了不蒙他,是对的。
#   但验一张卡需要一份**带着这些命令的程序**,而这份东西从来没人准备。
#   于是卡开着、活做完了、他人坐在电脑前,却没有任何办法在点之前试一下。
#   除了盲点,他没有第二个选项。**那不是谨慎,那是死锁。**
#
#   所以规矩改成:**卡和构建一起到。** 任何一张验收卡开到 pending,
#   就必须先跑这个脚本,把带着那份活的程序装好、起来,再把卡告诉他。
#   卡到了而构建没到,等于没告诉。
#
# 用法:
#   bash scripts/prepare-verification-build.sh <分支名>
#   bash scripts/prepare-verification-build.sh --from-dispatch <派单id>
#
# 可选:第三个参数是「这个构建里必须存在的字符串」,用来证明构建真的带着那份活。
#   给不出来就报错退出 —— 一个装好了但其实没带那份活的构建,比不装更糟。
#
#   bash scripts/prepare-verification-build.sh packet/mira/knowledge-panel list_history_projects
#
# 安全性:
#   - 装之前先停掉正在跑的应用(覆盖一个在跑的应用会出事)
#   - 构建失败就原地停,不装半成品
#   - 装完必须验证进程真的起来了,否则报错
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WROOT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
APP="/Applications/SeatLoom.app"

# ---- 参数 ----
# 两种调用形状:
#   prepare-verification-build.sh <分支名> [必须存在的字符串]
#   prepare-verification-build.sh --from-dispatch <派单id> [必须存在的字符串]
# 🔴 第一版把必须存在的字符串放在 $3,而自然的调用是 "<分支> <字符串>" ——
#    于是它永远收到空,还打一句「没给,强烈建议给一个」继续装。
#    一个该报错的检查静默跳过,正是这个项目反复栽的那一招。改成按形状解析。
BRANCH=""; DISPID=""; MUST_HAVE=""
case "${1:-}" in
  --from-dispatch)
    DISPID="${2:-}"; MUST_HAVE="${3:-}"
    [ -n "$DISPID" ] || fail "给了 --from-dispatch 但没给派单 id"
    ;;
  "") fail "要给一个分支名,或者 --from-dispatch <派单id>";;
  *) BRANCH="$1"; MUST_HAVE="${2:-}";;
esac

if [ -n "$DISPID" ]; then
  # 从账本反查:这张单是哪家席位做的,它的分支就是那个席位的工作区分支
  SEAT="$(podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \
    "select target_seat_id from dispatches where id='$DISPID'" 2>/dev/null | tr -d ' ')"
  [ -n "$SEAT" ] || fail "账本里找不到派单 $DISPID"
  SHORT="$(printf '%s' "$SEAT" | sed 's/seat-//;s/-001//')"
  BRANCH="$(git -C "$REPO" branch -a --list "*$SHORT*" 2>/dev/null | head -1 | sed 's/^[* ]*//' | sed 's#^remotes/origin/##')"
  [ -n "$BRANCH" ] || fail "找不到 $SEAT 的分支(试过匹配 '$SHORT')"
  say "派单 $DISPID 属于 $SEAT,分支:$BRANCH"
fi
[ -n "$BRANCH" ] || fail "解析完分支还是空的"

# ---- 在哪构建:主检出(track)还是某个席位的工作区 ----
if [ "$BRANCH" = "track/infra-foundation" ]; then
  WT="$REPO"
else
  # 找哪个工作区检出的正是这个分支
  WT=""
  for d in "$WROOT"/*/; do
    [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
    if [ "$(git -C "$d" rev-parse --abbrev-ref HEAD 2>/dev/null)" = "$BRANCH" ]; then WT="${d%/}"; break; fi
  done
  # 找不到检出的工作区,就找包含这个分支的工作区(git worktree 可以检出同一个分支的提交)
  if [ -z "$WT" ]; then
    for d in "$WROOT"/*/; do
      [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
      if git -C "$d" rev-parse --verify "$BRANCH" >/dev/null 2>&1; then WT="${d%/}"; break; fi
    done
  fi
  [ -n "$WT" ] || fail "找不到检出 $BRANCH 的工作区(在 $WROOT 下没有)"
fi
say "构建目录:$WT"

# ---- 前置 ----
line; say "一 — 前置检查"; line
[ -d "$WT/node_modules" ] || fail "$WT 没有 node_modules,先在那儿跑一次 pnpm install"
[ -d "$WT/ui" ] || fail "$WT 没有 ui 目录"
git -C "$WT" rev-parse --abbrev-ref HEAD 2>/dev/null | sed 's/^/  当前分支: /'
say "  待构建的提交:$(git -C "$WT" rev-parse --short "$BRANCH" 2>/dev/null || echo '?')"
say "  ✅ 目录在,依赖在"

# ---- 构建前端 ----
line; say "二 — 构建前端"; line
( cd "$WT/ui" && pnpm build ) 2>&1 | tail -4
[ -d "$WT/ui/dist" ] || fail "前端构建没产出 ui/dist"
say "  ✅ 前端产物在"

# ---- 构建 release ----
line; say "三 — 构建 release 应用(要几分钟)"; line
. "$HOME/.cargo/env" 2>/dev/null || true
( cd "$WT" && PATH="$WT/node_modules/.bin:$PATH" tauri build ) 2>&1 | tail -5
BUILT="$WT/target/release/bundle/macos/SeatLoom.app"
[ -x "$BUILT/Contents/MacOS/seatloom-tauri" ] || fail "tauri build 没产出 $BUILT"
say "  ✅ 应用包在:$BUILT"

# ---- 证明构建真的带着那份活 ----
line; say "四 — 证明这个构建真的带着要验的东西"; line
if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then
    say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)—— 这份活确实在里面"
  else
    fail "二进制里找不到 '$MUST_HAVE' —— 装上去也验不了,说明分支或构建不对。不要装。"
  fi
else
  say "  ⚠️  没给必须存在的字符串,这一项没验。强烈建议给一个。"
fi

# ---- 停掉旧的,装新的 ----
line; say "五 — 安装并启动"; line
if pgrep -f "SeatLoom.app" >/dev/null 2>&1; then
  say "  停掉正在跑的应用…"
  pkill -f "SeatLoom.app" 2>/dev/null
  for _ in $(seq 1 15); do pgrep -f "SeatLoom.app" >/dev/null 2>&1 || break; sleep 1; done
  pgrep -f "SeatLoom.app" >/dev/null 2>&1 && fail "旧应用停不掉,不覆盖安装"
  say "  ✅ 已停"
fi
rm -rf "$APP" || fail "删不掉旧的 $APP"
cp -R "$BUILT" "$APP" || fail "复制到 $APP 失败"
say "  ✅ 已安装:$APP"
# 用带操作者身份的启动器起,不要直接 open ——
# 从启动台/open 起的应用没有 SEATLOOM_OPERATOR_REF,点合并必然失败
# (2026-10-16 实测)。身份那一套逻辑在 launch-seatloom.sh 里,不在这里重复。
if [ -x "$REPO/scripts/launch-seatloom.sh" ]; then
  bash "$REPO/scripts/launch-seatloom.sh" 2>&1 | sed 's/^/    /'
else
  open -a "$APP" 2>/dev/null
fi

# ---- 验证真的起来了 ----
say "  等它起来…"
up=0
for _ in $(seq 1 30); do
  if pgrep -f "SeatLoom.app" >/dev/null 2>&1; then up=1; break; fi
  sleep 1
done
if [ "$up" != "1" ]; then
  fail "装好了但应用没起来。看 /Applications/SeatLoom.app 能不能手动点开,把现象报给 Aegis。"
fi
line; say "结果"; line
say "  ✅ 应用在跑,pid $(pgrep -f 'SeatLoom.app' | head -1)"
say "  ✅ 这份构建来自:$BRANCH @ $(git -C "$WT" rev-parse --short "$BRANCH")"
say ""
say "  现在可以验了。验的时候对照验收卡上的「看什么 → 什么算对 → 什么算打回」。"
say "  任何一条对不上,在卡上写打回理由 —— 那比点接受有价值。"
exit 0
