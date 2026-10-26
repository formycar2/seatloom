#!/usr/bin/env bash
# notify-operator.sh — 给张先生弹一条 macOS 通知(像 IM 软件那样主动找他)。
#
# 为什么需要它(2026-10-26,张先生的原话):
#   「你有需要我做的,得给我个弹出消息框,就像 IM 软件主动通知我,
#     不要让我被动去看,我就变成了卡点。」
#
# 这个批评是对的。Aegis 一直在"报告"(写进回复、等对方来看),
# 从没"通知"(主动弹到对方面前)。结果每一件需要他的事都卡在他来问为止。
#
# 🔴 关键设计:Aegis 不能自己弹。为什么 ——
#   Aegis 跑在 tmux 里的 saih 会话中,他的 bash 是沙箱,`osascript` 到不了
#   张先生的 GUI 会话(那条权限也被封顶)。所以这里给的是一条**可审计的命令**,
#   由张先生(或任何有 GUI 权限的进程)执行。
#
#   但"要张先生自己跑才知道有没有事"本身就是卡点。所以配套做法是:
#   Aegis 的 crontab 自醒(crontab 跑在张先生的登录会话下,**有** GUI 权限)
#   在扫到"有需要张先生做的事"时,自动执行本脚本。
#   这样不需要他来看,通知自己会到。
#
# 用法:
#   notify-operator.sh "标题" "正文"            # 弹一条
#   notify-operator.sh --list                   # 看当前有哪些事需要他做(不弹)
#   notify-operator.sh --push                   # 扫一遍待办,有待办就弹,没有就不弹
#   notify-operator.sh --push --apply           # 同上(兼容旧写法)
#
# 去重:同一件事不重复弹。已弹过的事记在 /tmp/seatloom_notified.txt,
#      用 dispatch/card id 做键。想重弹:删掉那一行,或 --force。
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB="${SEATLOOM_DB:-seatloom}"
SEEN="/tmp/seatloom_notified.txt"
FORCE=0; MODE="once"; TITLE=""; BODY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --list) MODE="list"; shift;;
    --push) MODE="push"; shift;;
    --apply) shift;;
    --force) FORCE=1; shift;;
    -h|--help) sed -n '2,30p' "$0"; exit 0;;
    *) if [ -z "$TITLE" ]; then TITLE="$1"; else BODY="$BODY $1"; fi; shift;;
  esac
done

# ── 找 podman:cron 的 PATH 极简(实测只有 /usr/bin:/bin),而 podman 在 ──
#    /opt/homebrew/bin。找不到就明说失败,**不静默跳过** ——
#    一个查不到账本就认为「没待办」的通知,比没有通知更有害:它会一直安静。
PODMAN_BIN=""
for c in "$(command -v podman || true)" /opt/homebrew/bin/podman /usr/local/bin/podman /usr/bin/podman; do
  [ -n "$c" ] && [ -x "$c" ] && { PODMAN_BIN="$c"; break; }
done
if [ -z "$PODMAN_BIN" ]; then
  say "❌ 找不到 podman(PATH=$PATH)—— 查不到账本,这一轮不能当成『没有待办』"
  exit 1
fi

q(){ "$PODMAN_BIN" exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$DB" -tAc "$1" 2>/dev/null; }

# 已经弹过的事(键 = dispatch/card id)
touch "$SEEN" 2>/dev/null || true
seen(){ grep -qxF "$1" "$SEEN" 2>/dev/null; }
mark(){ printf '%s\n' "$1" >> "$SEEN" 2>/dev/null || true; }

# 弹一条。副标题固定标来源,免得他以为是什么垃圾广告。
pop(){
  local t="$1" b="$2" key="$3"
  seen "$key" && { say "  (已弹过,跳过:$key)"; return 1; }
  osascript -e "display notification \"$b\" with title \"SeatLoom\" subtitle \"$t\"" 2>/dev/null
  mark "$key"
  say "  ✅ 已弹:$t"
  return 0
}

# ── 模式一:只列,不弹 ──
if [ "$MODE" = "list" ]; then
  line; say "现在需要张先生做的事"; line
  n=0
  while read -r cid; do
    [ -z "$cid" ] && continue
    seen "$cid" && echo "  [已通知过] $cid" || echo "  [未通知]   $cid"
    n=$((n+1))
  done < <(q "select id from acceptances where status='pending' order by created_at;")
  [ "$n" = "0" ] && say "  (没有待拍板的验收卡)"
  say ""
  say "  待你执行的命令(不弹,只提醒):"
  # 022 视图没应用就要提醒一次 —— 它是三张单卡 10 天的唯一真因
  if [ "$(q "select count(*) from information_schema.views where table_name='v_memory_session_join';" | tr -d ' ')" != "1" ]; then
    seen "022-view" && say "    [已通知过] 022 桥接视图未应用" || say "    [未通知]   022 桥接视图未应用"
  fi
  exit 0
fi

# ── 模式二:弹一条指定的 ──
if [ "$MODE" = "once" ]; then
  [ -z "$TITLE" ] && { say "用法: $0 \"标题\" \"正文\""; exit 1; }
  line; say "弹通知"; line
  pop "$TITLE" "$BODY" "manual:${TITLE}"
  exit 0
fi

# ── 模式三:扫待办,该弹的弹 ──
line; say "扫待办并通知张先生"; line
did=0

# 1. 待拍板的验收卡(最重要:只有他能点)
while read -r cid; do
  [ -z "$cid" ] && continue
  short="${cid#acc-}"
  [ "$FORCE" = "1" ] && grep -vxF "$cid" "$SEEN" > "$SEEN.tmp" 2>/dev/null; mv "$SEEN.tmp" "$SEEN" 2>/dev/null
  if pop "有张验收卡等你拍板" "卡 $short 已就绪。对上了点接受,对不上写打回理由。" "$cid"; then did=$((did+1)); fi
done < <(q "select id from acceptances where status='pending' order by created_at;")

# 2. 门C 的唯一真因:022 视图没应用(它卡着三张单)
if [ "$(q "select count(*) from information_schema.views where table_name='v_memory_session_join';" | tr -d ' ')" != "1" ]; then
  if pop "有条命令要你跑" "022 桥接视图未应用,三张单卡在它上面。跑:bash scripts/apply-022-bridge-view.sh --apply" "022-view"; then did=$((did+1)); fi
fi

# 3. 队列躺了 ready 单超过 30 分钟(说明没人推动,不是他的活但要他知道)
stale="$(q "select count(*) from dispatches where status='ready' and issued_at < now() - interval '30 minutes';" | tr -d ' ')"
if [ "${stale:-0}" -gt 0 ]; then
  if pop "有 $stale 张单躺了 30 分钟没人领" "waker 停了,单子推不下去。这是系统问题,不是你的事,告知你一声。" "stale-ready" ; then did=$((did+1)); fi
fi

say ""
if [ "$did" = "0" ]; then say "  没有新事需要通知他(该弹的都弹过了)"; else say "  弹了 $did 条"; fi
exit 0
