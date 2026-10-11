#!/usr/bin/env bash
# fleet-status.sh — 一屏看懂舰队在干什么。给人看的。
#
# 设计原则(v2,因 v1 被张先生指出"看不懂而且怀疑准确性"而重写):
#   1. **每一项检查都必须能报错。** v1 有一行写成 `... && echo OK || echo OK`,
#      无论真假都打印 OK —— 那不是检查,是装饰,比没有检查更有害。
#   2. **不打印派单标题。** 标题是 Aegis 的黑话("Lane C Part 1 — S0 归一化"),
#      v1 忠实转发给张先生,所以必然看不懂。这里只讲席位的中文职责 + 忙了多久。
#   3. **席位名单从唤醒表读,不硬编码。** v1 硬编码 5 个、漏了新加的 vela,
#      于是上面列着 vela 在干活、下面写"5/5",自相矛盾。
#   4. **"失败"不等于"要你管"。** 分开列:哪些真的需要人,哪些席位已自行诊断、阻塞在别处。
#   5. 带时间戳 —— 这是一张快照,几秒后可能就变了。
#
# 用法: bash scripts/fleet-status.sh
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAP="$REPO/.seatloom/dispatch-seats.tsv"
PSQL(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }

# 席位中文职责 —— 让「谁在忙」有意义
role_of(){ case "$1" in
  *lyra*)   echo "产品负责人(拆活、定验收标准)";;
  *onyx*)   echo "数据库表结构";;
  *nimbus*) echo "后端/架构";;
  *mira*)   echo "界面";;
  *flux*)   echo "质量验证";;
  *vela*)   echo "洗历史记录";;
  *aegis*)  echo "总控(我)";;
  *)        echo "未知职责";;
esac; }

echo "快照时间 $(date '+%H:%M:%S')"
echo

if ! PSQL "select 1" | grep -q 1; then
  echo "❌ 数据库连不上 —— 整个舰队停了。"
  echo "   修:podman machine start && podman start seatloom-postgres"
  exit 1
fi

# ── 一句话结论 ────────────────────────────────────────────────
busy_n=$(PSQL "select count(*) from dispatches where status='in_flight'")
ready_n=$(PSQL "select count(*) from dispatches where status='ready'")
blocked_n=$(PSQL "select count(*) from dispatches where status='blocked'")
cards_n=$(PSQL "select count(*) from acceptances where status='pending'")
if [ "${busy_n:-0}" -gt 0 ]; then
  echo "▶ 项目在推进中:${busy_n} 个席位在干活,${ready_n} 件排队。"
elif [ "${ready_n:-0}" -gt 0 ]; then
  echo "⏸ 有活排队但没人在做 —— 可能需要唤醒(见下方「唤醒服务」)。"
elif [ "${blocked_n:-0}" -gt 0 ]; then
  echo "⏸ 全部停在依赖上:${blocked_n} 件在等前置条件完成。"
else
  echo "⏸ 全队空闲,队列是空的。"
fi
echo

# ── 席位 ──────────────────────────────────────────────────────
echo "── 席位 ──"
total=0; alive=0
while IFS=$'\t' read -r sid sess; do
  case "$sid" in ''|'#'*) continue;; esac
  total=$((total+1))
  pane=$(tmux list-panes -t "$sess" -F '#{pane_pid}' 2>/dev/null | head -1)
  proc=""; [ -n "$pane" ] && proc=$(pgrep -P "$pane" 2>/dev/null | head -1)
  if [ -n "$proc" ]; then alive=$((alive+1)); up="在线"; else up="❌掉线"; fi
  # 它在忙吗,忙了多久
  work=$(PSQL "select coalesce(to_char(now()-started_at,'HH24:MI')||' 前开始','') from dispatches where target_seat_id='$sid' and status='in_flight' limit 1")
  q=$(PSQL "select count(*) from dispatches where target_seat_id='$sid' and status='ready'")
  if [ -n "$work" ]; then state="干活中($work)"; else
    if [ "${q:-0}" -gt 0 ]; then state="空闲,有 ${q} 件排队"; else state="空闲"; fi
  fi
  # 不用 printf 列对齐 —— 中文宽度按字节算会错位,看起来像坏了
  echo "  $(echo "$sid" | sed 's/seat-//;s/-001//') [$up] $(role_of "$sid") — $state"
done < "$MAP"
echo "  aegis [在线] $(role_of aegis) — 就是我,你正在跟我说话"
echo "  在线 $alive/$total"
echo

# ── 等你拍板 ──────────────────────────────────────────────────
echo "── 等你拍板 ──"
if [ "${cards_n:-0}" = "0" ]; then echo "  (没有)"; else
  for id in $(PSQL "select id from acceptances where status='pending'"); do
    br=$(PSQL "select coalesce(branch,'?') from acceptances where id='$id'")
    base=$(PSQL "select coalesce(base,'?') from acceptances where id='$id'")
    how=$(PSQL "select coalesce(how_to_verify,'(没写验法)') from acceptances where id='$id'")
    # base 必须是真实存在的分支,否则这张卡点了只会报错 —— 必须明说
    if git -C "$REPO" rev-parse --verify "refs/heads/$base" >/dev/null 2>&1; then
      echo "  ✅ 可以点:$br"
      echo "     怎么验:$how"
    else
      echo "  ⚠️ 这张点不了(坏卡,不用管):$br"
      echo "     原因:要合进的分支「${base}」不存在,点了只会报错,不会损坏任何东西"
    fi
    echo "     卡号 $id"
  done
fi
echo

# ── 真的需要人 ────────────────────────────────────────────────
echo "── 真的需要人介入 ──"
need=0
if [ "$alive" != "$total" ]; then echo "  • 有席位掉线(见上)—— 修:bash scripts/seat-launch.sh --all --force"; need=1; fi
tmux has-session -t seatloom-waker 2>/dev/null || { echo "  • 唤醒服务停了,新活不会被派下去"; need=1; }
[ "$need" = "0" ] && echo "  (没有需要你动手的事)"
# 失败的单单独说明,它不算「需要你动手」——所以放在结论之后,不影响上面那句
f=$(PSQL "select count(*) from dispatches where status='failed'")
if [ "${f:-0}" -gt 0 ]; then
  echo "  (另:有 ${f} 件派单失败过,席位已记录原因并上报;等前置条件齐了会重试,Aegis 在盯)"
fi
echo

# ── 底座 ──────────────────────────────────────────────────────
echo "── 底座 ──"
mig=$(PSQL "select max(substring(f from '[0-9]+')) from (select unnest(array['x']) f) t" >/dev/null 2>&1; ls "$REPO"/infra/postgres/schema/*.sql 2>/dev/null | tail -1 | sed 's#.*/##;s/_.*//')
printf "  代码里的最新表结构   %s\n" "${mig:-读不到}"
if [ "$(PSQL "select count(*) from pg_tables where tablename='memories'")" = "1" ]; then
  printf "  记忆功能             已上线(%s 条记忆)\n" "$(PSQL "select count(*) from memories")"
else
  printf "  记忆功能             ❌ 未上线(memories 表还没建到数据库里)\n"
fi
printf "  唤醒服务             %s\n" "$(tmux has-session -t seatloom-waker 2>/dev/null && echo 在跑 || echo '❌ 停了')"
# Aegis 的自醒必须由自醒之外的东西来查。心跳协议里那句"看日志验证自醒活着"
# 只有心跳跑得起来才执行 —— 自醒死了就不会有心跳,那是循环依赖。
# 所以这项检查放在任何人都会跑的这个脚本里(张先生随时会跑)。
WLOG="/tmp/seatloom_aegis_waker.log"
if [ ! -s "$WLOG" ]; then
  printf "  Aegis 自醒           ❌ 没有日志(%s)—— 它可能从来没跑过,或日志被清\n" "$WLOG"
  need=1
else
  # 日志里最后一行的分钟数(宿主机时钟,只和宿主机时钟比,不碰数据库时钟)
  last_epoch="$(date -j -f "%Y-%m-%d %H:%M:%S" "$(grep -o '^[0-9-]* [0-9:]*' "$WLOG" | tail -1)" +%s 2>/dev/null)"
  case "$last_epoch" in
    ''|*[!0-9]*)
      printf "  Aegis 自醒           ⚠️  日志有时间戳但解析不出来 —— 别当成正常\n"; need=1;;
    *)
      ago=$(( ( $(date +%s) - last_epoch ) / 60 ))
      if [ "$ago" -le 35 ]; then
        printf "  Aegis 自醒           在跑(%s 分钟前最后一次)\n" "$ago"
      else
        printf "  Aegis 自醒           ❌ %s 分钟没有记录了(每 :11/:41 一次,超过 35 分钟就是死了)\n" "$ago"
        need=1
        printf "                     修:bash scripts/install-aegis-self-wake.sh\n"
      fi;;
  esac
  # 位置错了也会静默失效:cron 进不了 ~/Documents,脚本一行都不跑。
  # 只看**最后一行** —— 日志是追加的,历史错误会永远留在里面,
  # 拿整个日志去 grep 会让一个早就修好的问题一直报警(那同样是噪音)。
  if tail -1 "$WLOG" | grep -q "Operation not permitted"; then
    printf "                     ⚠️  最后一次就是 Operation not permitted —— cron 进不去那个目录\n"
    need=1
    printf "                     修:bash scripts/install-aegis-self-wake.sh\n"
  fi
fi
# 按**二进制名**认,不要按路径认。2026-10-12 连着错过两次:
#   1. 只匹配 target/debug —— 换成 release 的集成构建后报「没开」,而应用正在跑;
#   2. 改成匹配三条绝对路径 —— 应用是用 ./target/release/... 相对路径起的,
#      命令行里没有 seatloom/ 前缀,三条全不中,还是报「没开」;
#      而且 pgrep -f 的模式是正则,'SeatLoom.app' 里的 . 会匹配任意字符,还误报过一次。
# 所以:先用名字抓到 pid,再从这条进程真实的命令行判断它是哪个构建。
sl_pid="$(pgrep -f 'seatloom-tauri' 2>/dev/null | head -1)"
if [ -z "$sl_pid" ]; then
  printf "  SeatLoom 窗口        %s\n" "没开"
else
  case "$(ps -o command= -p "$sl_pid" 2>/dev/null)" in
    *release*) sl_which="release = 集成构建";;
    *debug*)   sl_which="debug = 可能没有面板";;
    *)         sl_which="来路不明";;
  esac
  printf "  SeatLoom 窗口        开着(%s, pid %s)\n" "$sl_which" "$sl_pid"
fi
echo
printf "  累计完成派单         %s 件\n" "$(PSQL "select count(*) from dispatches where status='done'")"
