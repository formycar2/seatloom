#!/usr/bin/env bash
# prepare-sandbox-for-card.sh — 按验收卡号,把对应的那一版装成沙箱,让人点开就能验。
#
# 为什么需要它(2026-10-20 张先生的要求):
#   「能否在让我点卡的时候,把对应版本构建好,让我点验证的时候就可以打开对应版本的
#     应用,让我验真后再回到主线 SeatLoom 点卡。」
#
#   这正是 2026-10-16 他就指出的那个死锁的完整解法:
#     验收卡规矩是「只在他点接受时才合并」—— 对的;但验一张卡需要一份带着那些命令
#     的程序,而这份东西从来没人自动准备。于是卡开着、活做完了、他坐在电脑前,
#     除了盲点没有第二个选项。**那不是谨慎,那是死锁。**
#
#   所以规矩是「卡和构建一起到」。本脚本把那一步接到卡号上:
#     给一个卡号 → 从账本反查分支 → 建沙箱 → 报告验收步骤。
#
# 用法:
#   bash scripts/prepare-sandbox-for-card.sh <验收卡号> [--apply]
#
#   默认 dry-run:只报「这张卡是哪一版、会装什么、验收要看什么」,不动任何东西。
#   --apply:真的构建并安装沙箱(不碰主版,见 prepare-sandbox-build.sh)。
#
# 🔴 一张卡可能指向一条「不干净」的分支 —— 它带着不止一次交付。
#   2026-10-20 实测:packet/mira/knowledge-panel 名下就有多张卡(acc-disp-DgFKxw7...、
#   acc-disp-KnJYxcf7... 等),其中至少一张已被组合候选取代。
#   如果照着分支装,人会验到**别次的活**,而他自己不知道。
#   所以本脚本在装之前先报:这条分支有几个提交、几张卡分别验的是什么、
#   有没有已被取代的卡。让他手里有这张单子再决定,而不是装完才发现。
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# 🔴 2026-10-20 第一版的错:把 DB 定义成字符串变量,展开时没加引号,
#   于是带空格的多字段 SQL 被 shell 拆成多个参数,psql 只收到第一个 select 片段,
#   查询静默返回空,脚本报「账本里没有这张卡」—— 而卡明明在。
#   一个查不到真数据的检查比没有检查更有害。改成函数,参数不会被拆。
q(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -X -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }

CARD=""; APPLY=0
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1;;
    acc-*) CARD="$a";;
    "") ;;
    *) [ -z "$CARD" ] && CARD="$a";;
  esac
done
[ -n "$CARD" ] || fail "要给一个验收卡号,例如 acc-disp-9H3OCmprBFAkiYBa5MDwE"

# ---- 一、从账本读这张卡 ----
line; say "一 — 这张卡是什么"; line
# stderr 不吞:2>/dev/null 会把 SQL 报错咽掉,让「查不到」伪装成「账本里没这张卡」。
# 2026-10-20 第一版两连错就在这里:先是用 char(10)(PostgreSQL 里 char 是定长类型,
# 不是换行函数),报错被吞成空,脚本错报「没有这张卡」,而卡真实存在。
ROW="$(podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -X -U seatloom -d seatloom -tAc "select branch||chr(10)||status||chr(10)||dispatch_id||chr(10)||coalesce(flux_verdict_ref,'')||chr(10)||how_to_verify from acceptances where id='$CARD'")"
[ -n "$ROW" ] || fail "账本里没有验收卡 $CARD"
BRANCH="$(printf '%s\n' "$ROW" | sed -n 1p | tr -d ' ')"
STATUS="$(printf '%s\n' "$ROW" | sed -n 2p | tr -d ' ')"
DISPID="$(printf '%s\n' "$ROW" | sed -n 3p | tr -d ' ')"
VERDICT="$(printf '%s\n' "$ROW" | sed -n 4p | tr -d ' ')"
HOWTO="$(printf '%s\n' "$ROW" | sed -n 5p)"

say "  卡号      : $CARD"
say "  状态      : $STATUS"
say "  交付分支  : $BRANCH"
say "  派单      : $DISPID"
say "  验绿结论  : ${VERDICT:-（无）}"
[ -n "$HOWTO" ] && say "  怎么验    : $HOWTO"

case "$STATUS" in
  accepted) say ""; say "  ⚠️  这张卡已经接受过了。装它只能看历史版本,不会改任何事。";;
  cancelled) fail "这张卡已取消,没有可验的东西";;
esac

# ---- 二、这条分支干净吗:有没有带着别次的活 ----
line; say "二 — 这条分支干净吗"; line
[ -n "$BRANCH" ] || fail "卡里没有分支号,没法定位要装哪一版"
NCARD="$(q "select count(*) from acceptances where branch='$BRANCH'" | tr -d ' ')"
say "  同一分支上的验收卡:$NCARD 张"
if [ "$NCARD" -gt 1 ]; then
  say ""
  say "  ⚠️  这条分支上有不止一次交付。装这条分支会把**全部**活带进去,"
  say "     包括别次验的、可能已经被取代的。逐张列出来:"
  q "select '     '||id||'  '||status from acceptances where branch='$BRANCH' order by created_at"
  say ""
  say "  验收时盯 $DISPID 那一件。看到别次的东西改了,不等于这一件要打回 ——"
  say "  但如果别次的东西本来就不该在这个包里,那要打回,并且告诉 Aegis。"
else
  say "  ✅ 这条分支上只有这一张卡,装出来就是这一件事"
fi

# ---- 三、这个包里到底有几个提交 ----
line; say "三 — 包里有什么"; line
BASE="track/infra-foundation"
if git -C "$REPO" rev-parse --verify "$BASE" >/dev/null 2>&1; then
  NCOMMIT="$(git -C "$REPO" rev-list --count "$BASE".."$BRANCH" 2>/dev/null || echo '?')"
  say "  以 $BASE 为基准,$BRANCH 多了 $NCOMMIT 个提交"
  say "  这些提交:"
  git -C "$REPO" log --oneline "$BASE".."$BRANCH" 2>/dev/null | sed 's/^/     /'
  [ "$NCOMMIT" = "0" ] && fail "这条分支和主干没有差异 —— 没东西可验"
else
  say "  ⚠️  找不到主干 $BASE,提交清点这一项没验"
fi

# ---- 四、装 ----
line; say "四 — 装沙箱"; line
if [ "$APPLY" != "1" ]; then
  say "  预演到此为止。要真的装,加 --apply:"
  say "    bash scripts/prepare-sandbox-for-card.sh $CARD --apply"
  say ""
  say "  装的是旁边那份「SeatLoom 沙箱」,你现在这份不动。"
  exit 0
fi

# 交给沙箱构建器,由它负责「不碰主版」「装完验证真的起来了」
MUST="$(git -C "$REPO" log --format='%s' "$BASE".."$BRANCH" 2>/dev/null | head -1 | grep -o '[a-zA-Z_]\{6,\}' | head -1)"
say "  调用沙箱构建器…"
say "  证明用的字符串:${MUST:-(没有,这一项不验)}"
say ""
bash "$REPO/scripts/prepare-sandbox-build.sh" "$BRANCH" ${MUST:+"$MUST"} --apply
rc=$?
[ "$rc" = "0" ] || fail "沙箱没装成(退出码 $rc)。别去点卡 —— 你手上没有能验的那一版。"

line; say "接下来"; line
say "  1. 打开「SeatLoom 沙箱」(窗口标题写的是 SeatLoom 沙箱)"
say "  2. 按上面「怎么验」那一行走,对着看"
say "  3. 对上了 → 回到主线那份 SeatLoom 点接受"
say "  4. 对不上 → 在卡上写打回理由,那比点接受有价值"
say ""
say "  ⚠️ 沙箱连的是同一个库,在沙箱里点接受会真的改账本。"
say "     只看不点,或者先用 --sandbox-db 指定另一个库重建。"
exit 0
