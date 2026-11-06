#!/usr/bin/env bash
# replace-sandbox.sh — 换沙箱那一份应用:先干跑,干跑通过才动,动之前先备份,随时能回滚。
#
# 为什么单独一个脚本,而不是让 prepare-sandbox-build.sh --apply 直接干
# (2026-11-07 Lyra 的发现):
#
#   prepare-sandbox-build.sh 的 --apply 路径做三件不可逆的事,而且**没有一步备份**:
#       1. kill 掉正在跑的沙箱进程
#       2. rm -rf /Applications/SeatLoom-Sandbox.app
#       3. cp -R 新构建进去
#   第二步之后,旧沙箱就没了。装到一半失败(第三步失败、plist 改失败、图标换失败)
#   会留下一个起不来的沙箱,而回不去。
#
#   Lyra 量到「没有任何授权去替换沙箱」,而且替换会删掉一个正在跑的应用,
#   所以她把这变成一条要操作者显式批准的动作,而不是一个脚本自己决定的事。
#   本脚本就是那条动作的**可审计形状**:
#     - 干跑是默认。不给 --apply 就一个字节都不改、一个进程都不杀。
#     - 干跑把要动什么、动之前备份到哪、怎么回滚,全部先说清楚。
#     - --apply 之前必须先有备份,而且备份的位置和内容在干跑里点名。
#     - 回滚是单独一条命令,不需要重新构建。
#
# 🔴 三件这个脚本**不做**的事,边界写死在这里:
#   1. 不碰 /Applications/SeatLoom.app(主版)。一次都不碰 —— 它停、它删、它装,
#      都只针对沙箱。用可执行文件名区分两者,不用 SeatLoom.app(那个会同时匹配)。
#   2. 不碰 live 账本。沙箱连的是 seatloom_sandbox 快照库,本脚本不去改它。
#   3. 不自己决定要不要换。它只负责把「换了会发生什么」说清楚,以及按操作者
#      显式的 --apply 去执行。授权是人的,不是脚本的。
#
# 用法:
#   bash scripts/replace-sandbox.sh --from-branch <分支> [必须存在的字符串]     # 干跑(默认)
#   bash scripts/replace-sandbox.sh --from-branch <分支> [字符串] --apply       # 真换(需授权)
#   bash scripts/replace-sandbox.sh --rollback [备份目录]                        # 回滚到某个备份
#   bash scripts/replace-sandbox.sh --rollback --list                            # 列出能回滚到哪

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# 两种形状都试:从主检出跑,工作区在 ../seatloom-worktrees;从席位自己的工作区跑,
# 那是 ../ —— 再拼一次会得到一个不存在的目录,于是「找检出该分支的工作区」一次都
# 不进,报出来的却是「这不是任何工作区检出的分支」——把人指向错误方向的失败。
WROOT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
[ -d "$WROOT" ] || WROOT="$(cd "$REPO/.." && pwd)"

APP="/Applications/SeatLoom.app"          # 主版:只看,不动
SAND_APP="/Applications/SeatLoom-Sandbox.app"
SAND_EXE="seatloom-sandbox"               # 沙箱的可执行文件名,和主版不同名
BACKUP_ROOT="$HOME/SeatLoom-Sandbox-backups"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

BRANCH=""; MUST_HAVE=""; APPLY=0; ROLLBACK=0; LIST=0; BACKUP_DIR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --from-branch) [ $# -ge 2 ] || fail "--from-branch 后面要跟一个分支名"; BRANCH="$2"; shift 2;;
    --from-branch=*) BRANCH="${1#--from-branch=}"; shift;;
    --apply) APPLY=1; shift;;
    --rollback) ROLLBACK=1; shift;;
    --list) LIST=1; shift;;
    -h|--help)
      say "用法:"
      say "  干跑(默认,什么都不动):"
      say "    bash scripts/replace-sandbox.sh --from-branch <分支> [必须存在的字符串]"
      say "  真换(需要操作者显式授权):"
      say "    bash scripts/replace-sandbox.sh --from-branch <分支> [字符串] --apply"
      say "  回滚:"
      say "    bash scripts/replace-sandbox.sh --rollback --list      # 列出可回滚的备份"
      say "    bash scripts/replace-sandbox.sh --rollback <备份目录>   # 回滚到某个备份"
      exit 0;;
    *)
      if [ -z "$BRANCH" ] && [ "$ROLLBACK" = "0" ] && [ "$LIST" = "0" ]; then BRANCH="$1"; shift
      elif [ "$ROLLBACK" = "1" ] && [ -z "$BACKUP_DIR" ]; then BACKUP_DIR="$1"; shift
      elif [ -z "$MUST_HAVE" ]; then MUST_HAVE="$1"; shift
      else fail "不认识的参数:$1"; fi;;
  esac
done

# ── 回滚:不需要构建,不需要授权,随时可用 ────────────────────────────────
if [ "$LIST" = "1" ]; then
  say "可回滚的沙箱备份(在 $BACKUP_ROOT):"
  if [ ! -d "$BACKUP_ROOT" ]; then say "  (还没有任何备份 —— 说明沙箱从未被本脚本换过)"; exit 0; fi
  found=0
  for d in "$BACKUP_ROOT"/*/; do
    [ -d "$d" ] || continue
    meta="$d/BACKUP-META.txt"
    if [ -f "$meta" ]; then
      found=1
      say "  $(basename "${d%/}")"
      sed 's/^/      /' "$meta"
    fi
  done
  [ "$found" = "1" ] || say "  (目录在,但没有一个带 BACKUP-META.txt 的备份)"
  exit 0
fi

if [ "$ROLLBACK" = "1" ]; then
  [ -n "$BACKUP_DIR" ] || fail "回滚要给一个备份目录,或者先用 --rollback --list 看有哪些"
  # 允许只给目录名。
  [ -d "$BACKUP_DIR" ] || BACKUP_DIR="$BACKUP_ROOT/$BACKUP_DIR"
  [ -d "$BACKUP_DIR/SeatLoom-Sandbox.app" ] || fail "$BACKUP_DIR 里没有 SeatLoom-Sandbox.app"
  [ -f "$BACKUP_DIR/BACKUP-META.txt" ] || fail "$BACKUP_DIR 没有 BACKUP-META.txt —— 没有元信息的备份不用于回滚(说不清回的是哪一份)"
  line; say "回滚沙箱"; line
  say "  从 :$BACKUP_DIR"
  say "  到 :$SAND_APP"
  sed 's/^/      /' "$BACKUP_DIR/BACKUP-META.txt"
  if [ "$APPLY" != "1" ]; then
    say ""
    say "预演到此。回滚会停掉当前沙箱并用备份替换它 —— 确认无误加 --apply。"
    exit 0
  fi
  # 主版绝不碰。只停沙箱,而且只按沙箱的可执行文件名找。
  pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
  [ -n "$pid" ] && { say "  停掉当前沙箱 pid $pid…"; kill "$pid" 2>/dev/null; sleep 2; }
  rm -rf "$SAND_APP" || fail "删不掉当前沙箱 $SAND_APP"
  cp -R "$BACKUP_DIR/SeatLoom-Sandbox.app" "$SAND_APP" || fail "恢复失败"
  say "  ✅ 已回滚到 $BACKUP_DIR"
  exit 0
fi

# ── 替换路径 ────────────────────────────────────────────────────────────
[ -n "$BRANCH" ] || fail "要给 --from-branch <分支>,说明新沙箱从哪来"

# 找到检出该分支的工作区。**不猜**:传提交哈希会构建另一棵树,而横幅还会印着
# 你要的那个提交 —— 一个装错代码的沙箱比没有沙箱更糟。
WT=""
for d in "$WROOT"/*/; do
  [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
  if [ "$(git -C "$d" rev-parse --abbrev-ref HEAD 2>/dev/null)" = "$BRANCH" ]; then WT="${d%/}"; break; fi
done
[ -n "$WT" ] || fail "'$BRANCH' 不是任何工作区当前检出的分支。本脚本构建的是某个工作区检出中的那棵树,它不替你 checkout —— 传提交哈希会构建另一棵树。"

WT_COMMIT="$(git -C "$WT" rev-parse HEAD)"
WT_SHORT="$(git -C "$WT" rev-parse --short=12 HEAD)"
WT_DIRTY="$(git -C "$WT" status --porcelain 2>/dev/null | grep -v '^??' || true)"
BUILT="$WT/target/release/bundle/macos/SeatLoom.app"
BIN="$BUILT/Contents/MacOS/seatloom-tauri"

line
if [ "$APPLY" = "1" ]; then say "换沙箱 — 应用模式(会动,但先备份)"; else say "换沙箱 — 预演模式(什么都不动)"; fi
line
say "  源工作区    : $WT"
say "  源分支      : $BRANCH"
say "  源提交      : $WT_SHORT"
say "  构建产物    : $BUILT"
say "  当前沙箱    : $SAND_APP"
say "  主版(不动)  : $APP"
say "  备份到      : $BACKUP_ROOT/<时间戳>/"
say ""

# 🔴 身份必须在干跑里核。Lyra 量到沙箱二进制里只有环境变量名和六个像哈希的串,
#    没有一个是任何提交的哈希 —— 所以「装的是哪个提交」此前不可核。现在二进制内部
#    带提交了(build.rs),这里就核它:二进制说的提交必须等于工作区的 HEAD。
#    不一致就停。装上一个自称是 A 实际是 B 的沙箱,比不装更糟。
if [ ! -x "$BIN" ]; then
  fail "还没有构建产物。先跑 bash scripts/prepare-sandbox-build.sh $BRANCH --apply(它会构建),或者自己 tauri build。"
fi
say "  二进制内提交:"
# 🔴 必须匹配**标签**,不能匹配裸的 12 位十六进制。第一版用裸十六进制,实测在
#    二进制里抓到 `dbdbdbdbdbdb` —— 一个占位串,不是任何提交,于是干跑在构建
#    完全正确的时候报「这份构建不是这棵树」。这正是 Lyra 量到的那个坑:沙箱二进制
#    里有六个像哈希的串,没有一个是提交。标签匹配把这件事变成确定的。
BIN_COMMIT="$(LC_ALL=C grep -a -o 'seatloom-build-commit=[0-9a-f]*' "$BIN" 2>/dev/null | head -1 | sed 's/^seatloom-build-commit=//' || true)"
if [ -n "$BIN_COMMIT" ]; then
  say "      $BIN_COMMIT"
  if [ "$BIN_COMMIT" != "$WT_SHORT" ]; then
    fail "二进制里的提交 ($BIN_COMMIT) 和工作区 HEAD ($WT_SHORT) 不一致 —— 这份构建不是这棵树。重新构建再换。"
  fi
  say "      ✅ 与工作区 HEAD 一致"
else
  # 搜不到不算通过。明说没核上,而不是默默放过。
  say "      ⚠️  没能从二进制里读出提交号(不是失败,是没核上)"
fi
if [ -n "$MUST_HAVE" ]; then
  n="$(LC_ALL=C grep -a -c "$MUST_HAVE" "$BIN" 2>/dev/null || true)"
  [ "${n:-0}" -ge 1 ] && say "  必须存在的字符串 '$MUST_HAVE':在" || fail "二进制里找不到必须存在的字符串 '$MUST_HAVE' —— 装上去也验不到你要的东西"
fi
if [ -n "$WT_DIRTY" ]; then
  say "  ⚠️  源工作区有未提交的改动(不算未跟踪文件):"
  printf '%s\n' "$WT_DIRTY" | sed 's/^/      /' | head -20
  say "      ⚠️  这份构建包含上面这些改动,不只是提交 ${WT_SHORT}。"
else
  say "  ✅ 源工作区干净 —— 构建结果就是提交 $WT_SHORT"
fi
say ""

line; say "将要发生的事(按顺序)"; line
say "  1. 把当前沙箱整体备份到 $BACKUP_ROOT/<时间戳>/,并写下 BACKUP-META.txt"
say "     (记分支、提交、二进制内提交、备份时间、为什么换)"
say "  2. 停掉当前沙箱进程(只按 $SAND_EXE 找,绝不碰主版)"
say "  3. rm -rf $SAND_APP"
say "  4. cp -R 新构建进去,改 Info.plist 入口,换沙箱图标"
say "  5. 启动新沙箱(数据库仍是 seatloom_sandbox 快照库,不碰 live 账本)"
say "  6. 核对沙箱真的起来了,并把它横幅上的提交号和 $WT_SHORT 对上"
say ""
say "  主版 $APP:不在此列,任何一步都不碰。"
say ""

if [ "$APPLY" != "1" ]; then
  line; say "预演到此为止。要真换,加 --apply(那是一次需要授权的动作)"; line
  exit 0
fi

# ── 1. 备份(必须在任何破坏性动作之前) ──────────────────────────────────
TS="$(date '+%Y%m%d-%H%M%S')"
DEST="$BACKUP_ROOT/$TS"
mkdir -p "$BACKUP_ROOT" || fail "建不了备份目录 $BACKUP_ROOT"
if [ -d "$SAND_APP" ]; then
  cp -R "$SAND_APP" "$DEST/SeatLoom-Sandbox.app" || fail "备份失败 —— 没有备份就不换。当前沙箱原样未动。"
  {
    say "分支      : $BRANCH"
    say "提交      : $WT_SHORT"
    say "二进制提交: ${BIN_COMMIT:-未读到}"
    say "备份时间  : $(date '+%Y-%m-%d %H:%M:%S')"
    say "说明      : 这是被替换掉的旧沙箱。回滚命令:bash scripts/replace-sandbox.sh --rollback $DEST"
  } > "$DEST/BACKUP-META.txt"
  say "  ✅ 已备份到 $DEST"
else
  mkdir -p "$DEST"
  say "  当前没有沙箱可备份(第一次装),仍记一条元信息。"
  { say "分支      : $BRANCH"; say "提交      : $WT_SHORT"; say "备份时间  : $(date '+%Y-%m-%d %H:%M:%S')"; say "说明      : 首次安装,无旧沙箱可备。"; } > "$DEST/BACKUP-META.txt"
fi

# ── 2..4. 停、删、装 ───────────────────────────────────────────────────
pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
if [ -n "$pid" ]; then say "  停掉旧沙箱 pid $pid…"; kill "$pid" 2>/dev/null; for _ in $(seq 1 10); do kill -0 "$pid" 2>/dev/null || break; sleep 1; done; fi
rm -rf "$SAND_APP" || fail "删不掉旧的 $SAND_APP(备份在 $DEST,可回滚)"
cp -R "$BUILT" "$SAND_APP" || fail "复制失败(备份在 $DEST,可回滚)"
mv "$SAND_APP/Contents/MacOS/seatloom-tauri" "$SAND_APP/Contents/MacOS/$SAND_EXE" 2>/dev/null || say "  ⚠️ 重命名可执行文件失败"
if command -v plutil >/dev/null 2>&1; then
  plutil -replace CFBundleExecutable -string "$SAND_EXE" "$SAND_APP/Contents/Info.plist" 2>/dev/null || say "  ⚠️ 改 CFBundleExecutable 失败"
  plutil -replace CFBundleDisplayName -string "SeatLoom 沙箱" "$SAND_APP/Contents/Info.plist" 2>/dev/null || say "  ⚠️ 改显示名失败"
  PLIST_EXE="$(plutil -extract CFBundleExecutable raw "$SAND_APP/Contents/Info.plist" 2>/dev/null)"
  [ "$PLIST_EXE" = "$SAND_EXE" ] && [ -x "$SAND_APP/Contents/MacOS/$SAND_EXE" ] \
    || fail "沙箱不可启动:plist 的入口是 '$PLIST_EXE'。回滚:bash scripts/replace-sandbox.sh --rollback $DEST"
else
  say "  ⚠️ 没有 plutil —— plist 未改,沙箱可能起不来。回滚:bash scripts/replace-sandbox.sh --rollback $DEST"
fi
say "  ✅ 已安装:$SAND_APP"

# ── 5. 启动 ────────────────────────────────────────────────────────────
SEATLOOM_SANDBOX=1 \
SEATLOOM_SANDBOX_BRANCH="$BRANCH" \
SEATLOOM_SANDBOX_COMMIT="$WT_SHORT" \
SEATLOOM_SANDBOX_BUILT="$(date '+%Y-%m-%d %H:%M:%S')" \
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom_sandbox" \
  open -a "$SAND_APP" || fail "启动失败(备份在 $DEST,可回滚)"

# ── 6. 核对它真的起来了 ────────────────────────────────────────────────
up=0
for _ in $(seq 1 30); do
  npid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
  [ -n "$npid" ] && { up=1; break; }
  sleep 1
done
[ "$up" = "1" ] || fail "沙箱没起来(备份在 $DEST,可回滚:bash scripts/replace-sandbox.sh --rollback $DEST)"

line; say "结果"; line
say "  ✅ 主版  : $APP —— 未触碰"
say "  ✅ 沙箱  : $SAND_APP (pid $(pgrep -f "Contents/MacOS/$SAND_EXE" | head -1))"
say "  ✅ 来源  : $BRANCH @ $WT_SHORT"
say "  ✅ 备份  : $DEST"
say ""
say "  回滚一条命令:bash scripts/replace-sandbox.sh --rollback $DEST"
say ""
say "  接下来请核对沙箱窗口里那条身份横幅:它应该写着提交 ${WT_SHORT}。"
say "  横幅上的提交和这里不一致,说明启动的不是刚装的这一份 —— 那种情况下不要点任何验收。"
