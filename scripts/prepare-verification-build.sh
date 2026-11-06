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
#   bash scripts/prepare-verification-build.sh --dry-run <分支名>     ← 只看,不动手
#   bash scripts/prepare-verification-build.sh --dry-run --from-dispatch <派单id>
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
#
# --dry-run(2026-11-06 张先生授权「继续优化 SeatLoom」后的第一个有界切片):
#   上面每一条都是**不可逆**的 —— pkill、rm -rf、cp -R、tauri build、重启。
#   而在点接受之前,操作者唯一需要回答的问题是「这一下会动到什么」。
#   于是干跑模式:把真实路径会做的每一件事**算出来并打印**,一件都不执行。
#
#   它不构建、不杀进程、不删不改不装应用、不动 refs/index、不写库、不重启。
#   它打印:源分支与提交、工作区是否干净、目标应用路径、会碰到的进程与路径、
#   以及哪些动作需要人批准。未知参数/ref/路径/读取失败一律非零退出;
#   缺安全前提时**不得**打印「可以装」。
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# 席位的工作区根。原来写死 "$REPO/../seatloom-worktrees",在**主检出**里跑时对,
# 在**席位自己的工作区**里跑时错:那时的 $REPO/.. 已经是 seatloom-worktrees,
# 再拼一层就成了 seatloom-worktrees/seatloom-worktrees。
# 实测(2026-11-06):在 nimbus-psr 里跑,两种调用形状都报
#   「找不到检出 <分支> 的工作区(在 .../seatloom-worktrees/seatloom-worktrees 下没有)」
# 这是个**可用性**缺陷,不是安全缺陷 —— 它fail closed,报错很清楚,不会误装。
# 所以按「哪个目录里真的有席位工作区」来定,而不是猜层级:
#   两边都试,先试原本那个(主检出的形状),再试上一层(工作区里的形状)。
WROOT=""
for cand in "$(cd "$REPO/.." && pwd)/seatloom-worktrees" "$(cd "$REPO/.." && pwd)"; do
  # 「真的有席位工作区」= 下面有带 .git 的子目录,且至少一个是 git 工作树
  if compgen -G "$cand/*/.git" >/dev/null 2>&1 || compgen -G "$cand/*/.git" >/dev/null 2>&1; then
    if ls -d "$cand"/*/ >/dev/null 2>&1; then WROOT="$cand"; break; fi
  fi
done
[ -n "$WROOT" ] || WROOT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
APP="/Applications/SeatLoom.app"

# ---- 参数 ----
# 两种调用形状:
#   prepare-verification-build.sh <分支名> [必须存在的字符串]
#   prepare-verification-build.sh --from-dispatch <派单id> [必须存在的字符串]
# 🔴 第一版把必须存在的字符串放在 $3,而自然的调用是 "<分支> <字符串>" ——
#    于是它永远收到空,还打一句「没给,强烈建议给一个」继续装。
#    一个该报错的检查静默跳过,正是这个项目反复栽的那一招。改成按形状解析。
BRANCH=""; DISPID=""; MUST_HAVE=""; DRY=0
case "${1:-}" in
  # 干跑必须是第一个参数,且只能给一次。放在最前是因为它改变的是整个脚本的
  # 性质(从「做」变成「算」),不是一个可选的附加步骤。
  --dry-run)
    DRY=1; shift
    case "${1:-}" in
      --from-dispatch) DISPID="${2:-}"; MUST_HAVE="${3:-}"; [ -n "$DISPID" ] || fail "给了 --from-dispatch 但没给派单 id";;
      "") fail "给了 --dry-run 但没给分支名或 --from-dispatch";;
      *) BRANCH="$1"; MUST_HAVE="${2:-}";;
    esac
    ;;
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

# 任何残留的 --dry-run 都是拼错或者重复给。静默忽略一个它不认识的参数,
# 正是这个脚本第一版栽过的坑(必须存在的字符串被静默跳过)。
for a in "$@"; do
  case "$a" in
    --dry-run) fail "--dry-run 只能给一次,且必须是第一个参数";;
    --*) fail "不认识的参数:$a";;
  esac
done

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

# ---- 前置(干跑也要查:前提不满足时不能打印「可以装」)----
line; say "一 — 前置检查"; line
[ -d "$WT/node_modules" ] || fail "$WT 没有 node_modules,先在那儿跑一次 pnpm install"
[ -d "$WT/ui" ] || fail "$WT 没有 ui 目录"
WT_HEAD="$(git -C "$WT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')"
WT_COMMIT="$(git -C "$WT" rev-parse --short "$BRANCH" 2>/dev/null || echo '?')"
WT_LONG="$(git -C "$WT" rev-parse "$BRANCH" 2>/dev/null || echo '?')"
say "  当前分支:$WT_HEAD"
say "  待构建的提交:$WT_COMMIT"
say "  ✅ 目录在,依赖在"

# ---- 干跑在这里结束:把真实路径会做的事算出来,一件都不执行 ----
if [ "$DRY" = "1" ]; then
  line; say "干跑 — 以下是将要发生的事,一件都没执行"; line

  # 身份:这个构建到底是谁、从哪来。工作区脏了要说,因为脏意味着
  # 「构建出来的东西不等于这个提交」,那是验收里最贵的意外。
  #
  # 🔴 git status 读失败 ≠ 工作区干净。第一版写成
  #      DIRTY="$(git -C "$WT" status --porcelain 2>/dev/null | grep -v '^??' || true)"
  #    于是 status 退出 128(权限坏了、索引坏了、磁盘出错)时:2>/dev/null 把唯一
  #    能区分「干净」和「读不出来」的证据扔掉,|| true 让整条管道算成功,DIRTY
  #    落成空串,[ -n ] 为假,走 else 打印「工作区干净」—— 而且那个分支从不设
  #    BLOCKERS,干跑退出 0,等于告诉操作者「可以装」。这份构建跟那个提交的关系
  #    根本没测过。2026-11-07 实测复现(ev-dr3OFuoVhsMO3JYi7mHfK):注入一个只在
  #    status --porcelain 上失败的 git,干跑退出 0 并打印「工作区干净」。
  #
  #    修法只有一条承重规则:**git 的退出码和 grep 的「没匹配」必须分开取。**
  #    grep 没匹配是「没有未跟踪文件」,是正常结果;git 退出 128 是「我没读到」。
  #    把两者用一条管道和一个 || true 混在一起,后者就被前者吃掉了。
  #
  #    stderr 走一个临时文件而不是 2>/dev/null —— 那句话就是「干净」和「读不出来」
  #    的全部区别,mktemp 本身失败时按「读不出来」处理,不能当成干净放过去。
  GIT_STATUS_ERR="$(mktemp 2>/dev/null)" || GIT_STATUS_ERR=""
  if [ -n "$GIT_STATUS_ERR" ] && [ -w "$GIT_STATUS_ERR" ]; then
    DIRTY_RAW="$(git -C "$WT" status --porcelain 2>"$GIT_STATUS_ERR")"
    GIT_STATUS_RC=$?
  else
    # 连临时文件都建不出来,一样是「读不出来」,不是干净。
    DIRTY_RAW=""
    GIT_STATUS_RC=99
  fi
  # stderr 是区分「干净」和「读不出来」的那句话,所以不丢。取前几行,压成一行。
  if [ -n "$GIT_STATUS_ERR" ] && [ -f "$GIT_STATUS_ERR" ]; then
    GIT_STATUS_MSG="$(tr -d '\r' <"$GIT_STATUS_ERR" | grep -v '^[[:space:]]*$' \
      | head -3 | tr '\n' ' ' | sed 's/[[:space:]]*$//')"
    rm -f "$GIT_STATUS_ERR"
  else
    GIT_STATUS_MSG=""
  fi
  # 未跟踪文件不算 —— 没 git add 过的文件本来就不进构建,这是故意的。这个 grep
  # 只在 git 真的读成功时才跑,所以它自己的退出码没有意义,|| true 不会吃掉任何东西。
  if [ "$GIT_STATUS_RC" = "0" ]; then
    DIRTY="$(printf '%s' "$DIRTY_RAW" | grep -v '^??' || true)"
  else
    DIRTY=""
  fi
  if [ "$GIT_STATUS_RC" != "0" ]; then
    # 三种状态,不是两种。读不出来是第三种,它既不是干净也不是脏,
    # 绝不能落到 [ -n "$DIRTY" ] 这个二值判断里。
    say "  ❌ 读不出工作区状态(git status 退出码 $GIT_STATUS_RC)"
    [ -n "$GIT_STATUS_MSG" ] && say "     git 说:$GIT_STATUS_MSG"
    say "     这不是「干净」—— 这份构建跟提交 ${WT_COMMIT} 的关系没有测过。"
    say "     权限、索引、磁盘任何一个出问题都会走到这里。修好再干跑,不要在没测过的情况下装。"
  elif [ -n "$DIRTY" ]; then
    say "  ⚠️  工作区有未提交的改动(不算未跟踪文件):"
    printf '%s\n' "$DIRTY" | sed 's/^/      /' | head -20
    [ "$(printf '%s\n' "$DIRTY" | grep -c .)" -gt 20 ] && say "      …(还有更多)"
    say "  ⚠️  装上以后,这个构建包含上面这些改动,不只是提交 ${WT_COMMIT}。"
  else
    say "  ✅ 工作区干净 —— 构建结果就是提交 $WT_COMMIT"
  fi
  # 分支指向的提交,和工作区 HEAD 是不是同一个。不是同一个说明分支被移动过,
  # 或者工作区停在别的提交上 —— 两者都会让「装的是这个分支」变成假的。
  if [ "$WT_HEAD" = "$BRANCH" ]; then
    say "  ✅ 构建目录检出的正是 $BRANCH"
  else
    say "  ⚠️  构建目录检出的是 '$WT_HEAD',不是 '$BRANCH' —— 装上去的会是 $WT_HEAD"
  fi

  line; say "会执行的动作(按真实顺序)"; line
  say "  1. 在 $WT/ui 跑 pnpm build            → 写 $WT/ui/dist"
  say "  2. 在 $WT 跑 tauri build              → 写 $WT/target/release/bundle/macos/SeatLoom.app"
  if [ -n "$MUST_HAVE" ]; then
    say "  3. 在产出的二进制里找 '$MUST_HAVE'    → 找不到就中止,不装"
  else
    say "  3. (跳过)没给必须存在的字符串 —— 不会验证这个构建真的带着要验的东西"
  fi
  say "  4. 停掉正在运行的 SeatLoom(如果有)   → pkill -f SeatLoom.app"
  say "  5. 删除并替换 $APP  → rm -rf + cp -R"
  say "  6. 用带操作者身份的启动器重启        → scripts/launch-seatloom.sh"
  say "  7. 等进程起来,起不来就报错"

  line; say "会碰到的进程"; line
  if pgrep -fl "SeatLoom.app" >/dev/null 2>&1; then
    pgrep -fl "SeatLoom.app" | sed 's/^/  pid /'
    say "  ⚠️  上面这些进程会被停掉。它们里面的未保存状态会丢。"
  else
    say "  (没有 SeatLoom 在跑)"
  fi

  line; say "会写到的路径"; line
  say "  $WT/ui/dist"
  say "  $WT/target/release/bundle/macos/SeatLoom.app"
  say "  $APP   ← 已存在的应用会被删掉替换"

  line; say "需要人批准的"; line
  say "  - 停掉并替换 $APP(不可逆,覆盖在跑的应用)"
  say "  - 实际重新构建与安装(干跑不做这些)"

  # 干跑的收尾判据:能不能装,取决于前提,不取决于运气。
  line; say "结论"; line
  BLOCKERS=0
  [ -x "$WT/node_modules/.bin/tauri" ] || [ -f "$WT/node_modules/.bin/tauri" ] || {
    say "  ❌ $WT 没有 tauri 可执行文件(node_modules/.bin/tauri)—— 装不了"; BLOCKERS=1; }
  [ -x "$REPO/scripts/launch-seatloom.sh" ] || {
    say "  ❌ 没有 scripts/launch-seatloom.sh —— 起来会缺操作者身份,点合并必然失败"; BLOCKERS=1; }
  if [ "$GIT_STATUS_RC" != "0" ]; then
    say "  ❌ 读不出工作区状态 —— 不知道这份构建是不是提交 $WT_COMMIT,现在不能装"; BLOCKERS=1
  elif [ -n "$DIRTY" ]; then
    say "  ❌ 工作区脏 —— 先提交或 stash,否则验收的东西不等于这个提交"; BLOCKERS=1
  fi
  if [ "$BLOCKERS" != "0" ]; then
    say ""
    say "❌ 有上面这些阻碍,现在不能装。修好再跑(可以再干跑一次)。"
    exit 1
  fi
  say "  ✅ 前提都满足。真要装,跑:"
  say "       bash scripts/prepare-verification-build.sh $BRANCH${MUST_HAVE:+ $MUST_HAVE}"
  say "  ⚠️  那一步会停掉在跑的应用并替换 $APP,不可逆。"
  exit 0
fi

# ---- 构建前端 ----

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
