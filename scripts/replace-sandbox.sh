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

APP="${SEATLOOM_MAIN_APP:-/Applications/SeatLoom.app}"          # 主版:只看,不动
SAND_APP="${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}"
SAND_EXE="seatloom-sandbox"               # 沙箱的可执行文件名,和主版不同名
BACKUP_ROOT="${SEATLOOM_SANDBOX_BACKUP_ROOT:-$HOME/SeatLoom-Sandbox-backups}"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

# 🔴 两个路径必须不是同一个。这个脚本的所有破坏性动作都只针对 $SAND_APP,
#    所以一旦它被(手打、环境变量、符号链接)指到主版,后果就是把主版删掉。
#    在动任何东西之前先核,而不是指望调用方永远打对。
#    这条必须能报错 —— 一个永远通过的路径相等检查比没有检查更有害。
if [ "$SAND_APP" = "$APP" ]; then
  fail "沙箱路径和主版路径相同 ($SAND_APP)。本脚本只动沙箱,绝不会动主版 —— 停。"
fi
# 再核一次解析后的真实路径:符号链接可以让两个不同的字符串指向同一份东西。
# 只在本机 macOS 上做(realpath 不是每个环境都有),做不了就跳过。
RP_MAIN=""; RP_SAND=""
if command -v realpath >/dev/null 2>&1; then
  RP_MAIN="$(realpath "$APP" 2>/dev/null || true)"
  RP_SAND="$(realpath "$SAND_APP" 2>/dev/null || true)"
fi
if [ -n "$RP_MAIN" ] && [ -n "$RP_SAND" ] && [ "$RP_MAIN" = "$RP_SAND" ]; then
  fail "沙箱路径解析后就是主版路径 ($RP_SAND)。停。"
fi

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
  # 🔴 回滚也会 rm -rf 当前沙箱 —— 那同样是破坏性的,所以先备份当前这一份,
  #    再删。第一版没有这一步:回滚到一半失败(cp 失败)时,旧备份还在,
  #    但**回滚前正在跑的那一份没了**,而它可能比备份里那一份更新。
  #    「随时能回滚」这句话在那种情况下是假的。
  RTS="$(date '+%Y%m%d-%H%M%S')"
  RBAK="$BACKUP_ROOT/before-rollback-$RTS"
  if [ -d "$SAND_APP" ]; then
    mkdir -p "$BACKUP_ROOT" || fail "建不了备份目录 $BACKUP_ROOT —— 回滚会删掉当前沙箱,没有备份就不回滚"
    mkdir -p "$RBAK" || fail "建不了 $RBAK —— 没有备份就不回滚"
    cp -R "$SAND_APP" "$RBAK/SeatLoom-Sandbox.app" || fail "备份当前沙箱失败 —— 不回滚。当前沙箱原样未动。"
    CUR_COMMIT="$(LC_ALL=C grep -a -o 'seatloom-build-commit=[0-9a-f]*' "$SAND_APP/Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1 | sed 's/^seatloom-build-commit=//' || true)"
    [ -n "$CUR_COMMIT" ] || CUR_COMMIT="未知(二进制里读不到)"
    {
      say "这份是什么  : 回滚前正在用的那一份沙箱(回滚失败时用它恢复)"
      say "它的提交     : $CUR_COMMIT"
      say "备份时间     : $(date '+%Y-%m-%d %H:%M:%S')"
      say "恢复命令     : bash scripts/replace-sandbox.sh --rollback $RBAK"
    } > "$RBAK/BACKUP-META.txt"
    say "  已把当前这一份备份到 $RBAK(回滚失败时能回到它)"
  fi
  pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
  [ -n "$pid" ] && { say "  停掉当前沙箱 pid $pid…"; kill "$pid" 2>/dev/null; sleep 2; }
  if ! rm -rf "$SAND_APP"; then
    fail "删不掉当前沙箱 $SAND_APP —— 回滚没做成,当前沙箱原样未动"
  fi
  # cp 失败时不能只说一句「恢复失败」就走:当前沙箱已经删了,必须把它装回去,
  # 否则这一下是把一个能用的沙箱换成了什么都没有。
  if ! cp -R "$BACKUP_DIR/SeatLoom-Sandbox.app" "$SAND_APP"; then
    if [ -d "$RBAK/SeatLoom-Sandbox.app" ]; then
      cp -R "$RBAK/SeatLoom-Sandbox.app" "$SAND_APP" 2>/dev/null
      fail "回滚失败,已把回滚前那一份装回去($RBAK)。系统回到回滚前的状态。"
    fi
    fail "回滚失败,而且没有回滚前的备份可恢复 —— $SAND_APP 现在是空的。备份:$BACKUP_DIR"
  fi
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

WT_COMMIT="$(git -C "$WT" rev-parse HEAD 2>/dev/null)"
WT_SHORT="$(git -C "$WT" rev-parse --short=12 HEAD 2>/dev/null)"
# git 的退出码和 grep 的「没匹配」分开取。一条管道 + || true 会让后者吃掉前者,
# 于是「我没读到」变成「没有未提交改动」—— 这正是 2026-11-07 修
# prepare-verification-build.sh 时量到的同一个病,在这里一模一样地复发。
WT_STATUS_ERR="$(mktemp 2>/dev/null)" || WT_STATUS_ERR=""
if [ -n "$WT_STATUS_ERR" ] && [ -w "$WT_STATUS_ERR" ]; then
  WT_STATUS_RAW="$(git -C "$WT" status --porcelain 2>"$WT_STATUS_ERR")"
  WT_STATUS_RC=$?
else
  WT_STATUS_RAW=""; WT_STATUS_RC=99
fi
if [ -n "$WT_STATUS_ERR" ] && [ -f "$WT_STATUS_ERR" ]; then
  WT_STATUS_MSG="$(tr -d '\r' <"$WT_STATUS_ERR" | grep -v '^[[:space:]]*$' \
    | head -3 | tr '\n' ' ' | sed 's/[[:space:]]*$//')"
  rm -f "$WT_STATUS_ERR"
else
  WT_STATUS_MSG=""
fi
if [ "$WT_STATUS_RC" = "0" ]; then
  WT_DIRTY="$(printf '%s' "$WT_STATUS_RAW" | grep -v '^??' || true)"
else
  WT_DIRTY=""
fi
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
#
# 🔴 读不到提交号**不是**「没核上就放过」。第一版在这里只打一句 ⚠️ 然后继续,
#    于是「这份构建是哪个提交」这个唯一能证明装对了的东西,可以在完全不知道的
#    情况下通过 —— 一个说不清的替换比不换更糟。现在:读不到就停。
if [ ! -x "$BIN" ]; then
  # 这里**不能**推荐 prepare-sandbox-build.sh --apply:那个脚本的 --apply 会
  # 顺手 rm -rf + cp -R 把沙箱换掉,一步备份都没有。让人在一个「先备份再换」
 # 的脚本里被告知去跑一个「直接换不备份」的脚本,是把刚立好的规矩拆掉。
 # 所以指路只指「构建」,不指「安装」。
  fail "还没有构建产物。构建和安装是两步,先只构建(不安装、不替换、不停进程):
    bash scripts/prepare-sandbox-build.sh --build-only $BRANCH
  构建好了再跑本脚本的干跑确认。替换沙箱必须走本脚本 —— 它先备份再换。"
fi
say "  二进制内提交:"
# 🔴 必须匹配**标签**,不能匹配裸的 12 位十六进制。第一版用裸十六进制,实测在
#    二进制里抓到 `dbdbdbdbdbdb` —— 一个占位串,不是任何提交,于是干跑在构建
#    完全正确的时候报「这份构建不是这棵树」。这正是 Lyra 量到的那个坑:沙箱二进制
#    里有六个像哈希的串,没有一个是提交。标签匹配把这件事变成确定的。
BIN_COMMIT="$(LC_ALL=C grep -a -o 'seatloom-build-commit=[0-9a-f]*' "$BIN" 2>/dev/null | head -1 | sed 's/^seatloom-build-commit=//' || true)"
# 工作区的提交号也读不到,同样停。两个都读不到时 BIN_COMMIT != WT_SHORT 会比不上,
# 但错误信息会指向「二进制不对」,而真正的原因可能是 git 坏了。所以分别核。
[ -n "$WT_SHORT" ] || fail "读不出工作区 HEAD 的提交号(git rev-parse 失败)—— 不知道这份构建该对应哪个提交。停。"
if [ -z "$BIN_COMMIT" ]; then
  fail "没能从二进制里读出提交号。二进制必须自带提交(build.rs 的 BUILD_COMMIT),否则「装的是哪个提交」不可核。重新构建再换。"
fi
say "      $BIN_COMMIT"
if [ "$BIN_COMMIT" != "$WT_SHORT" ]; then
  fail "二进制里的提交 ($BIN_COMMIT) 和工作区 HEAD ($WT_SHORT) 不一致 —— 这份构建不是这棵树。重新构建再换。"
fi
say "      ✅ 与工作区 HEAD 一致"
if [ -n "$MUST_HAVE" ]; then
  # grep -c 在没匹配时退出 1,|| true 之后 $n 是""而不是 0 —— 那 [ "" -ge 1 ] 会
  # 自己报错。先把没匹配归一成 0,再比。
  n="$(LC_ALL=C grep -a -c "$MUST_HAVE" "$BIN" 2>/dev/null)" || n=0
  [ "${n:-0}" -ge 1 ] && say "  必须存在的字符串 '$MUST_HAVE':在" || fail "二进制里找不到必须存在的字符串 '$MUST_HAVE' —— 装上去也验不到你要的东西"
fi
# 三态,不是两态:读不出工作区状态既不是干净也不是脏。
if [ "$WT_STATUS_RC" != "0" ]; then
  fail "读不出源工作区的状态(git status 退出码 $WT_STATUS_RC)${WT_STATUS_MSG:+ —— $WT_STATUS_MSG}。这份构建跟提交 $WT_SHORT 的关系没有测过,不能换。"
elif [ -n "$WT_DIRTY" ]; then
  # 🔴 脏工作区是**阻碍**,不是一句提醒。第一版只打 ⚠️ 然后继续换,
 #    于是「装的是这个提交」在最需要它是真的时候变成假的。
  say "  ❌ 源工作区有未提交的改动(不算未跟踪文件):"
  printf '%s\n' "$WT_DIRTY" | sed 's/^/      /' | head -20
  say "      ❌ 这份构建包含上面这些改动,不只是提交 ${WT_SHORT}。先提交或 stash,再换。"
  fail "源工作区脏 —— 先提交或 stash,否则沙箱里的东西不等于 $WT_SHORT"
else
  say "  ✅ 源工作区干净 —— 构建结果就是提交 $WT_SHORT"
fi
say ""
line; say "将要发生的事(按顺序)"; line
say "  1. 把当前沙箱整体备份到 $BACKUP_ROOT/<时间戳>/,并写下 BACKUP-META.txt"
say "     (记的是**被换掉的旧沙箱**是什么:它的提交号、它是什么时候被装上的、"
say "      为什么换 —— 不是新来源的信息。回滚要能回答「我回到了哪」。)"
say "  2. 核备份真的可用:里面的 SeatLoom-Sandbox.app 和元信息都在,才继续"
say "  3. 停掉当前沙箱进程(只按 $SAND_EXE 找,绝不碰主版)"
say "  4. rm -rf $SAND_APP"
say "  5. cp -R 新构建进去,改 Info.plist 入口,换沙箱图标"
say "  6. 启动新沙箱(数据库仍是 seatloom_sandbox 快照库,不碰 live 账本)"
say "  7. 核对沙箱真的起来了,并把它横幅上的提交号和 $WT_SHORT 对上"
say ""
say "  主版 $APP:不在此列,任何一步都不碰。"
say ""
say "  任一步失败:停在那里,不宣称成功,并告诉你怎么回到刚才那一份。"

if [ "$APPLY" != "1" ]; then
  line; say "预演到此为止。要真换,加 --apply(那是一次需要授权的动作)"; line
  exit 0
fi

# ── 1. 备份(必须在任何破坏性动作之前) ──────────────────────────────────
TS="$(date '+%Y%m%d-%H%M%S')"
DEST="$BACKUP_ROOT/$TS"
mkdir -p "$BACKUP_ROOT" || fail "建不了备份目录 $BACKUP_ROOT —— 没有备份就不换"
# cp -R 到 $DEST/... 之前,$DEST 必须已经存在。mkdir -p "$DEST" 不能省:
# 省了它 cp 会因为父目录不存在而失败,而那时 $SAND_APP 还没删,
# 于是「备份失败」和「没备份成功」两件事看起来一样。
mkdir -p "$DEST" || fail "建不了备份目标 $DEST —— 没有备份就不换"
if [ -d "$SAND_APP" ]; then
  cp -R "$SAND_APP" "$DEST/SeatLoom-Sandbox.app" || fail "备份失败 —— 没有备份就不换。当前沙箱原样未动。"
  # 元信息描述的是**被换掉的旧沙箱**。旧沙箱自己的提交号从哪来?从它上一次被装时
  # 写下的元信息链,或者(没有链的时候)从它二进制里那个标签读。读不到就写「未知」,
  # 不猜 —— 一条写错的元信息会在回滚时把人送回错误的那一份。
  OLD_COMMIT="$(LC_ALL=C grep -a -o 'seatloom-build-commit=[0-9a-f]*' "$SAND_APP/Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1 | sed 's/^seatloom-build-commit=//' || true)"
  [ -n "$OLD_COMMIT" ] || OLD_COMMIT="未知(二进制里读不到)"
  {
    say "这份是什么  : 被本次替换换掉的旧沙箱(回滚它会回到这一份)"
    say "旧沙箱提交   : $OLD_COMMIT"
    say "换入的提交   : $WT_SHORT"
    say "换入的分支   : $BRANCH"
    say "备份时间     : $(date '+%Y-%m-%d %H:%M:%S')"
    say "回滚命令     : bash scripts/replace-sandbox.sh --rollback $DEST"
  } > "$DEST/BACKUP-META.txt"
  # 核备份,再动破坏性的东西。备份不可用就不能往下走 ——
  # 「备份成功」这句话必须是核过的,不能是 cp 没报错就当真。
  [ -f "$DEST/BACKUP-META.txt" ] || fail "备份元信息没写成功 —— 不换"
  [ -x "$DEST/SeatLoom-Sandbox.app/Contents/MacOS/$SAND_EXE" ] \
    || fail "备份里的可执行文件不在 —— 这份备份回不去,不换"
  say "  ✅ 已备份并核对到 $DEST"
else
  # 当前没有沙箱可备:元信息说明这是首次安装,并且它描述的是「换入的这一份」,
  # 因为被换掉的东西不存在。两处措辞不能一样,否则回滚时说不清回到的是什么。
  {
    say "这份是什么  : 首次安装前的状态(当时没有沙箱,所以没有旧沙箱可备)"
    say "换入的提交   : $WT_SHORT"
    say "换入的分支   : $BRANCH"
    say "备份时间     : $(date '+%Y-%m-%d %H:%M:%S')"
    say "回滚命令     : bash scripts/replace-sandbox.sh --rollback $DEST"
  } > "$DEST/BACKUP-META.txt"
  [ -f "$DEST/BACKUP-META.txt" ] || fail "备份元信息没写成功 —— 不换"
  say "  当前没有沙箱可备份(第一次装),已记一条元信息。"
fi

# ── 2..5. 停、删、装 ───────────────────────────────────────────────────
# 到这里每一行都已经备份好了。失败不再是「回不去」,但**仍然必须失败得诚实**:
# 每一步失败就停,不往下走,并且把「现在系统是什么状态」说清楚。
pid="$(pgrep -f "Contents/MacOS/$SAND_EXE" 2>/dev/null | head -1)"
if [ -n "$pid" ]; then
  say "  停掉旧沙箱 pid $pid…"
  kill "$pid" 2>/dev/null
  for _ in $(seq 1 10); do kill -0 "$pid" 2>/dev/null || break; sleep 1; done
  # 停不掉就不继续。在进程还握着旧沙箱文件的时候 rm -rf,删掉的是它正在用的东西。
  if kill -0 "$pid" 2>/dev/null; then
    fail "旧沙箱 pid $pid 停不掉 —— 不动它。备份在 $DEST,可回滚。"
  fi
  say "  ✅ 已停"
fi
rm -rf "$SAND_APP" || fail "删不掉旧的 $SAND_APP(备份在 $DEST,可回滚)"
cp -R "$BUILT" "$SAND_APP" || fail "复制失败(备份在 $DEST,可回滚)"
# 重命名可执行文件:这一步失败,沙箱就和主版同名,pgrep/活动监视器里分不清哪个是
# 哪个,也就无法只停其中一个。第一版只 say ⚠️ 就往下走 —— 那等于把「装好了」
# 这句话押在一个失败的步骤上。现在失败就停。
mv "$SAND_APP/Contents/MacOS/seatloom-tauri" "$SAND_APP/Contents/MacOS/$SAND_EXE" \
  || fail "重命名可执行文件失败(备份在 $DEST,可回滚)"
if ! command -v plutil >/dev/null 2>&1; then
  fail "没有 plutil —— plist 改不了,入口仍指向 seatloom-tauri,沙箱起不来(备份在 $DEST,可回滚)"
fi
plutil -replace CFBundleExecutable -string "$SAND_EXE" "$SAND_APP/Contents/Info.plist" \
  || fail "改 CFBundleExecutable 失败(备份在 $DEST,可回滚)"
plutil -replace CFBundleDisplayName -string "SeatLoom 沙箱" "$SAND_APP/Contents/Info.plist" \
  || fail "改显示名失败(备份在 $DEST,可回滚)"
PLIST_EXE="$(plutil -extract CFBundleExecutable raw "$SAND_APP/Contents/Info.plist" 2>/dev/null)"
[ "$PLIST_EXE" = "$SAND_EXE" ] && [ -x "$SAND_APP/Contents/MacOS/$SAND_EXE" ] \
  || fail "沙箱不可启动:plist 的入口是 '$PLIST_EXE'。回滚:bash scripts/replace-sandbox.sh --rollback $DEST"
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
if [ "$up" != "1" ]; then
  # 起不来不是「装好了」:把现在这一份的状态说清楚,再指回滚。
  say "  ❌ 沙箱没起来。当前 $SAND_APP 是新装的这一份,备份在 $DEST。"
  fail "沙箱没起来 —— 不要点任何验收。回滚:bash scripts/replace-sandbox.sh --rollback $DEST"
fi
# 起来了,还要确认跑起来的**就是刚装的那一个文件**。只 pgrep 到一个 pid 不够:
# 那个 pid 可能是上一个还没死透的进程。核它的可执行路径。
RUNNING_EXE="$(ps -p "$npid" -o command= 2>/dev/null || true)"
case "$RUNNING_EXE" in
  *"$SAND_APP/Contents/MacOS/$SAND_EXE"*) say "  ✅ 跑起来的就是刚装的那一份";;
  *) say "  ⚠️  pid $npid 的可执行路径不是刚装的那一份: $RUNNING_EXE";;
esac
line; say "结果"; line
say "  ✅ 主版  : $APP —— 未触碰"
say "  ✅ 沙箱  : $SAND_APP (pid $npid)"
say "  ✅ 来源  : $BRANCH @ $WT_SHORT"
say "  ✅ 备份  : $DEST"
say ""
say "  回滚一条命令:bash scripts/replace-sandbox.sh --rollback $DEST"
say ""
say "  接下来请核对沙箱窗口里那条身份横幅:它应该写着提交 ${WT_SHORT}。"
say "  横幅上的提交和这里不一致,说明启动的不是刚装的这一份 —— 那种情况下不要点任何验收。"
