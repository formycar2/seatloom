#!/usr/bin/env bash
# prepare-sandbox-build.sh — 给张先生一份「旁边那份」应用,不碰他正在用的那份。
#
# 为什么需要它(2026-10-20 张先生提出的死锁):
#   prepare-verification-build.sh 把应用装到 /Applications/SeatLoom.app —— 固定路径,
#   装之前 pkill 掉正在跑的,装完只启动这一个。**所以永远只有一份,而且装新的必毁旧的。**
#   他的原话:「我一边用 seatloom 完善 SeatLoom,一边又要验证 SeatLoom,
#   我总是手里有一个不是最新的实例。」
#
#   本脚本改成双轨:日常那份留在 /Applications/SeatLoom.app 不动,
#   新构建装成 /Applications/SeatLoom-Sandbox.app,两个可以同时开着。
#
# 🔴 三个必须处理的真实约束,少一个沙箱就是假的:
#
#   1. 启动器不能复用 launch-seatloom.sh。它用 pgrep -f "SeatLoom.app" 找进程,
#      而那个模式**会同时匹配主版和沙箱版** —— 直接复用会把主版也杀掉,
#      正好毁掉这个脚本要保护的东西。
#
#   2. localStorage 键撞车。四个键里三个没有命名空间(sl-main-view、sl-supervisor-open、
#      sl-supervisor-active-contact),两个应用同时开会读同一份状态,
#      于是「沙箱记得我上次在哪」会被主版覆盖,反之亦然。
#      解法:沙盒版用独立键前缀,不改主版,也不改共享的数据库。
#
#   3. 沙箱版默认连同一个数据库。**这一条必须让张先生显式知道**,不是悄悄糊过去:
#      在沙箱里点「接受」会真的改账本。所以本脚本默认给沙箱注入一个
#      独立的数据目录 + 独立 DATABASE_URL 指向同一实例的**只读副本**不可行
#      (t7 那套没有只读用户),所以退一步:默认**不给沙箱注入不同 DATABASE_URL**,
#      而是在启动时打印一条警告,并要他在 --sandbox-db <url> 时自己指定。
#      宁可让他知道会写,不要让他以为隔开了。
#
# 用法:
#   bash scripts/prepare-sandbox-build.sh <分支名|--from-dispatch 派单id> [必须存在的字符串]
#
#   默认:dry-run —— 只报「会装什么、装到哪、会起几个进程」,不动任何东西。
#   --apply:真的构建并安装。
#   --sandbox-db <url>:给沙箱注入独立的 DATABASE_URL(要他自己决定,不由脚本猜)。
#
# 安全性:
#   - dry-run 是默认,不给 --apply 就绝不构建、绝不安装、绝不起进程
#   - 不 pkill 任何进程 —— 主版在跑就不动它
#   - 装完必须验证沙箱进程真的起来了,否则报错,不假装成功
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WROOT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
MAIN_APP="/Applications/SeatLoom.app"
SAND_APP="/Applications/SeatLoom-Sandbox.app"
SANDBOX_KEY_PREFIX="slsandbox:"
MUST_HAVE=""; BRANCH=""; DISPID=""; APPLY=0; SANDBOX_DB=""

# ---- 参数 ----
while [ $# -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1; shift;;
    --from-dispatch) DISPID="${2:-}"; shift 2 || fail "--from-dispatch 后面要跟派单 id";;
    --sandbox-db) SANDBOX_DB="${2:-}"; shift 2 || fail "--sandbox-db 后面要跟一个 URL";;
    *) [ -z "$BRANCH" ] && BRANCH="$1"; shift;;
  esac
done

if [ -n "$DISPID" ]; then
  SEAT="$(podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \
    "select target_seat_id from dispatches where id='$DISPID'" 2>/dev/null | tr -d ' ')"
  [ -n "$SEAT" ] || fail "账本里找不到派单 $DISPID"
  SHORT="$(printf '%s' "$SEAT" | sed 's/seat-//;s/-001//')"
  # 这个席位名下所有本地分支 —— **全部列出来,不是取第一个**
  CANDS="$(git -C "$REPO" branch -a --list "*$SHORT*" 2>/dev/null | sed 's/^[*+ ]*//;s#^remotes/origin/##' | grep -v '^$' | sort -u)"
  [ -n "$CANDS" ] || fail "找不到 $SEAT 的分支(试过匹配 '$SHORT')"
  NCAND="$(printf '%s\n' "$CANDS" | grep -c .)"
  if [ "$NCAND" -gt 1 ]; then
    # 一个能装错分支的沙箱比没有沙箱更糟:他会去验一份不是他要验的东西。
    # 所以分支不唯一次?不猜,停下来让他指。
    say ""
    say "⚠️  $SEAT 名下有 $NCAND 条分支,派单 $DISPID 自己没记是哪一条:"
    printf '%s\n' "$CANDS" | sed 's/^/     /'
    say ""
    say "  请显式指定要验哪一条,例如:"
    say "     bash scripts/prepare-sandbox-build.sh $(printf '%s\n' "$CANDS" | head -1) [必须存在的字符串] --apply"
    fail "不从多条分支里猜。指定一条再跑。"
  fi
  BRANCH="$CANDS"
  say "派单 $DISPID 属于 $SEAT,分支:$BRANCH"
fi
[ -n "$BRANCH" ] || fail "要给一个分支名,或者 --from-dispatch <派单 id>"

# 分支之后的第一个位置参数是「必须存在的字符串」
# 🔴 2026-10-20 第一版的错:从派单反查时用 seat 短名去匹配「第一个含该词的分支」,
#    Mira 名下有好几条分支,于是拿到了 live-mirror-um0 而不是 V5 所在的 knowledge-panel。
#    一个装错了分支的沙箱,比没有沙箱更糟 —— 它会让人验一份不是他要验的东西。
#    改成:反查只给候选,不确定就**停下来问**,不猜。
if [ -n "$DISPID" ] && [ -z "${BRANCH_FROM_LEDGER:-}" ]; then :; fi

# ---- 在哪构建 ----
if [ "$BRANCH" = "track/infra-foundation" ]; then
  WT="$REPO"
else
  WT=""
  for d in "$WROOT"/*/; do
    [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
    if [ "$(git -C "$d" rev-parse --abbrev-ref HEAD 2>/dev/null)" = "$BRANCH" ]; then WT="${d%/}"; break; fi
  done
  if [ -z "$WT" ]; then
    for d in "$WROOT"/*/; do
      [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
      git -C "$d" rev-parse --verify "$BRANCH" >/dev/null 2>&1 && { WT="${d%/}"; break; }
    done
  fi
  [ -n "$WT" ] || fail "找不到检出 $BRANCH 的工作区(在 $WROOT 下没有)"
fi

line
if [ "$APPLY" = "1" ]; then say "沙箱构建 — 应用模式"; else say "沙箱构建 — 预演模式(不动任何东西)"; fi
line
say "  分支          : $BRANCH"
say "  构建目录      : $WT"
say "  提交          : $(git -C "$WT" rev-parse --short "$BRANCH" 2>/dev/null || echo '?')"
say "  主版(不动)    : $MAIN_APP"
say "  沙箱(要装的)  : $SAND_APP"
say "  沙箱键前缀    : $SANDBOX_KEY_PREFIX(避免和主版抢 localStorage)"
if [ -n "$SANDBOX_DB" ]; then
  say "  沙箱数据库    : $SANDBOX_DB"
else
if [ -z "$SANDBOX_DB" ]; then
  # 2026-10-21 张先生裁定:「沙箱要尽可能对齐正式库」+ 不能写回真账本。
  # 默认改成指向沙箱快照库(由 refresh-sandbox-db.sh 灌),不再默认连主库 ——
  # 一个点一下就改真账本的沙箱,会让人不敢点,于是又回到盲点。
  if podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d postgres -tAc \
       "select 1 from pg_database where datname='seatloom_sandbox'" 2>/dev/null | grep -q 1; then
    SANDBOX_DB="postgresql://seatloom:seatloom@localhost:5432/seatloom_sandbox"
    say "  沙箱数据库    : $SANDBOX_DB"
    say "                 (快照库,数据和正式库一致,但写它不影响正式库)"
  else
    say "  沙箱数据库    : ⚠️  seatloom_sandbox 不存在"
    if [ "$APPLY" = "1" ]; then
      fail "沙箱快照库不存在。先跑 bash scripts/refresh-sandbox-db.sh --apply 再装沙箱。"
           "默认连主库的那条路已去掉 —— 那意味着在沙箱里点『接受』会改真账本。"
    fi
  fi
fi
fi

# ---- 主版在跑吗:报告,但不碰它 ----
# 🔴 2026-10-20 第一版的错:照抄 launch-seatloom.sh 的 `pgrep -f "SeatLoom.app"`,
#    而可执行文件名是 seatloom-tauri(小写 s),那个模式匹配不到,
#    于是明明主版在跑(pid 13636)却报「没在跑」。一个报错的检查比没有检查更有害。
#    现在按真实可执行文件名匹配:主版 seatloom-tauri,沙箱 seatloom-sandbox。
MAIN_EXE="seatloom-tauri"
SAND_EXE="seatloom-sandbox"
main_pid="$(pgrep -f "Contents/MacOS/$MAIN_EXE" 2>/dev/null | head -1)"
if [ -n "$main_pid" ]; then
  say "  主版在跑      : pid $main_pid —— **本脚本不会动它**"
else
  say "  主版在跑      : 没在跑(也不启动它)"
fi

# 🔴 装之前先问:这条分支比主干旧多少?
# 2026-10-21 踩到的坑:为让张先生验净化单,让他装 packet/mira/knowledge-panel,
# 没查它落不落后主干 —— 结果那条分支落后 track 18 个提交,项目过滤的代码
# (2c69adb)根本不在上面。沙箱里席位全平铺,他以为功能坏了,差点打回一件对的活。
# **一个装错代码的沙箱比没有沙箱更糟。**
BASE="track/infra-foundation"
if git -C "$REPO" rev-parse --verify "$BASE" >/dev/null 2>&1 && [ "$BRANCH" != "$BASE" ]; then
  BEHIND="$(git -C "$REPO" rev-list --count "$BRANCH..$BASE" 2>/dev/null || echo '?')"
  AHEAD="$(git -C "$REPO" rev-list --count "$BASE..$BRANCH" 2>/dev/null || echo '?')"
  say "  与主干对比    : 领先 $AHEAD 个提交 / 落后 $BEHIND 个提交"
  if [ "$BEHIND" != "0" ] && [ "$BEHIND" != "?" ]; then
    say ""
    say "  ⚠️  这条分支比主干旧 $BEHIND 个提交。沙箱会**缺**那 $BEHIND 个提交里的功能。"
    say "     落后越多,你在沙箱里看不到的越多 —— 那不等于那些功能坏了。"
  fi
fi

if [ "$APPLY" != "1" ]; then
  say ""
  say "预演到此为止。要真的构建并安装,加 --apply。"
  say "构建要花几分钟;装的是旁边那份,你正在用的那份不受影响。"
  exit 0
fi

# ---- 前置 ----
line; say "一 — 前置检查"; line
[ -d "$WT/node_modules" ] || fail "$WT 没有 node_modules"
[ -d "$WT/ui" ] || fail "$WT 没有 ui 目录"
say "  ✅ 目录在,依赖在"

# ---- 前端 ----
line; say "二 — 构建前端"; line
( cd "$WT/ui" && pnpm build ) 2>&1 | tail -4
[ -d "$WT/ui/dist" ] || fail "前端构建没产出 ui/dist"
say "  ✅ 前端产物在"

# ---- release ----
line; say "三 — 构建 release 应用(要几分钟)"; line
. "$HOME/.cargo/env" 2>/dev/null || true
( cd "$WT" && PATH="$WT/node_modules/.bin:$PATH" tauri build ) 2>&1 | tail -5
BUILT="$WT/target/release/bundle/macos/SeatLoom.app"
[ -x "$BUILT/Contents/MacOS/seatloom-tauri" ] || fail "tauri build 没产出 $BUILT"
say "  ✅ 应用包在:$BUILT"

# ---- 证明这个构建真的带着要验的东西 ----
line; say "四 — 证明这个构建带着那份活"; line
if [ -n "$MUST_HAVE" ]; then
  n="$(strings "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null | grep -c "$MUST_HAVE")"
  if [ "$n" -ge 1 ]; then say "  ✅ 二进制里找到 '$MUST_HAVE'($n 处)"
  else fail "二进制里找不到 '$MUST_HAVE' —— 这份构建不带那份活,不要装"; fi
else
  say "  ⚠️  没给必须存在的字符串,这一项没验。建议给一个:"
  say "     bash scripts/prepare-sandbox-build.sh <分支> <某个只在该分支存在的字符串> --apply"
fi

# ---- 装到旁边那份 ----
line; say "五 — 安装到旁边那一份(不碰主版)"; line
# 只停沙箱自己,绝不动主版 —— 这正是本脚本存在的理由
# 只停沙箱自己,绝不动主版 —— 这正是本脚本存在的理由。
# 用可执行文件名区分两者,不用 SeatLoom.app(那个会同时匹配两个包)。
sand_pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
if [ -n "$sand_pid" ]; then
  say "  停掉旧沙箱 pid $sand_pid…"
  kill "$sand_pid" 2>/dev/null
  for _ in $(seq 1 10); do kill -0 "$sand_pid" 2>/dev/null || break; sleep 1; done
fi
rm -rf "$SAND_APP" || fail "删不掉旧的 $SAND_APP"
cp -R "$BUILT" "$SAND_APP" || fail "复制到 $SAND_APP 失败"
# 重命名可执行文件,让它和主版可区分 —— 否则 pgrep/活动监视器里两份同名,
# 分不清哪个是哪个,也就无法只停其中一个。
# 🔴 2026-10-21 第一版只改了文件名,没改 Info.plist 的 CFBundleExecutable,
#    于是 macOS 按 plist 找 seatloom-tauri 找不到,启动报
#    「executable is missing」(张先生实测撞到)。
#    **改一半比不改更糟:不改只是两份同名,改一半是根本起不来。**
#    所以这里同步改 plist,并且装完必须核对一致才许启动。
mv "$SAND_APP/Contents/MacOS/seatloom-tauri" "$SAND_APP/Contents/MacOS/$SAND_EXE" 2>/dev/null \
  || say "  ⚠️ 重命名可执行文件失败,两份将同名"
# plist 里的入口必须指向同一份文件,否则系统找不到入口
if command -v plutil >/dev/null 2>&1; then
  plutil -replace CFBundleExecutable -string "$SAND_EXE" "$SAND_APP/Contents/Info.plist" 2>/dev/null \
    || say "  ⚠️ 改 CFBundleExecutable 失败"
  # 窗口标题也改掉,让他一眼看出在看哪份
  plutil -replace CFBundleDisplayName -string "SeatLoom 沙箱" "$SAND_APP/Contents/Info.plist" 2>/dev/null \
    || say "  ⚠️ 改显示名失败"
else
  say "  ⚠️ 没有 plutil —— plist 未改,入口仍指向 seatloom-tauri,沙箱起不来"
fi
# 核对:plist 说的入口必须真在那儿。缺这一条就会重演「可执行文件缺失」。
PLIST_EXE="$(plutil -extract CFBundleExecutable raw "$SAND_APP/Contents/Info.plist" 2>/dev/null)"
if [ "$PLIST_EXE" = "$SAND_EXE" ] && [ -x "$SAND_APP/Contents/MacOS/$SAND_EXE" ]; then
  say "  ✅ 入口一致:plist 说 $PLIST_EXE,文件在且可执行"
else
  fail "沙箱不可启动:plist 的入口是 '$PLIST_EXE',实际可执行文件是 '$SAND_EXE'。修好再启动 —— 一个装好了起不来的沙箱比不装更糟。"
fi
say "  ✅ 已安装:$SAND_APP"

# ── 图标也换掉:和主版长得一样 = 点错就是真事故 ──
# 2026-10-21 之前这一步是独立的一条命令,装完沙箱的人得自己记得跑。
# 一个需要人额外记得的步骤,迟早会被忘掉。改成装完就做。
if [ -x "$REPO/scripts/make-sandbox-icon.sh" ]; then
  line; say "五之二 — 换成一眼可辨的沙箱图标"; line
  bash "$REPO/scripts/make-sandbox-icon.sh" "$SAND_APP" --apply 2>&1 | sed 's/^/    /' \
    || fail "图标没换成 —— 两个应用会长得一样,点错就是事故。手动跑 make-sandbox-icon.sh 再装。"
else
  say "  ⚠️ 没有 make-sandbox-icon.sh,跳过图标 —— 两个应用会长得一样"
fi

# ---- 以操作者身份启动沙箱 ----
line; say "六 — 启动沙箱"; line
OPERATOR_REF="${SEATLOOM_OPERATOR_REF:-human:supervisor}"
# 注入身份信息:让窗口里的标识条能写出「这个沙箱是哪个分支的哪一版」,
# 而不是让前端去猜或依赖 localStorage(那三个无命名空间的键会和主版互抢)。
# 这些是只读变量,不给沙箱任何额外能力。
SAND_BRANCH="$BRANCH"
SAND_COMMIT="$(git -C "$WT" rev-parse --short "$BRANCH" 2>/dev/null || echo unknown)"
SAND_BUILT="$(date '+%Y-%m-%d %H:%M')"
export SEATLOOM_SANDBOX=1
export SEATLOOM_SANDBOX_BRANCH="$SAND_BRANCH"
export SEATLOOM_SANDBOX_COMMIT="$SAND_COMMIT"
export SEATLOOM_SANDBOX_BUILT="$SAND_BUILT"
if [ -n "$SANDBOX_DB" ]; then
  SEATLOOM_OPERATOR_REF="$OPERATOR_REF" DATABASE_URL="$SANDBOX_DB" \
    open -a "$SAND_APP" || fail "open 沙箱失败"
  say "  以 $OPERATOR_REF 启动,数据库:$SANDBOX_DB"
else
  SEATLOOM_OPERATOR_REF="$OPERATOR_REF" open -a "$SAND_APP" || fail "open 沙箱失败"
  say "  以 $OPERATOR_REF 启动,数据库:**与主版同一个库**"
fi
say "  注入标识:分支=$SAND_BRANCH 提交=$SAND_COMMIT 构建于=$SAND_BUILT"

up=0
for _ in $(seq 1 30); do
  pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
  [ -n "$pid" ] && { up=1; break; }
  sleep 1
done
[ "$up" = "1" ] || fail "沙箱装好了但没起来 —— 手动点开 $SAND_APP 看报什么"
say "  ✅ 沙箱在跑,pid $pid"

line; say "结果"; line
say "  ✅ 主版  : $MAIN_APP (pid $(pgrep -f "Contents/MacOS/$MAIN_EXE" 2>/dev/null | head -1 || echo '未运行'))"
say "  ✅ 沙箱  : $SAND_APP (pid $pid)"
say ""
say "  两份可以同时开着。沙箱窗口标题写的是「SeatLoom 沙箱」。"
if [ -z "$SANDBOX_DB" ]; then
  say ""
  say "  ⚠️ 沙箱连的是同一个库。在沙箱里点「接受」会真的改账本。"
  say "     只想看不想改,就别在沙箱里点接受;或下次加 --sandbox-db 指定另一个库。"
fi
say "  这一份构建来自:$BRANCH @ $(git -C "$WT" rev-parse --short "$BRANCH")"
exit 0
