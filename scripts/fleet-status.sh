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
# 020: 等你拍板的件数现在有两类,分开数。验收卡 + 只有操作者能裁的事。
# 合成一个数字会把两件不同的事说成一件 —— 而张先生要回答的是"有几件在等我"。
cards_n=$(PSQL "select count(*) from acceptances where status='pending'")
dec_n=$(PSQL "select count(*) from operator_decisions where status='pending' and decider='human:supervisor'")
if [ "${busy_n:-0}" -gt 0 ]; then
  echo "▶ 项目在推进中:${busy_n} 个席位在干活,${ready_n} 件排队。"
elif [ "${ready_n:-0}" -gt 0 ]; then
  echo "⏸ 有活排队但没人在做 —— 可能需要唤醒(见下方「唤醒服务」)。"
elif [ "${blocked_n:-0}" -gt 0 ]; then
  echo "⏸ 全部停在依赖上:${blocked_n} 件在等前置条件完成。"
else
  echo "⏸ 全队空闲,队列是空的。"
fi
# 等他的事单独一句:活可以全空,但他名下仍可能压着没裁的事 ——
# 那正是 020 之前看不见的东西。
if [ "${cards_n:-0}" -gt 0 ] || [ "${dec_n:-0}" -gt 0 ]; then
  echo "👉 有你的事:${cards_n} 件待验收,${dec_n} 件要你裁定(见下方「等你拍板」)。"
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
# 020: 这里以前只读 acceptances —— 于是产品方向的裁定、只有你能解的前置条件
# ("要不要给防护加豁免"、"沙箱连哪个库"、"删不删那个文件")只以消息存在,
# 没有任何一处能回答"现在有几件事在等他"。失败模式是静默遗漏。
# 现在两类分开数、分开列,并且标明哪类是验收、哪类是裁定 ——
# 合并成一张表会让这个数字变错,而那正是 020 要修的问题。
cards_n=$(PSQL "select count(*) from acceptances where status='pending'")
dec_n=$(PSQL "select count(*) from operator_decisions where status='pending' and decider='human:supervisor'")
# 表可能还没建到库里(迁移由张先生执行)。表不存在时这个数是空的,
# 必须说出来,不能当成 0 —— 一个沉默的 0 和一个真实的 0 不是一回事。
dec_table=$(PSQL "select count(*) from pg_tables where tablename='operator_decisions'")
if [ "${dec_table:-0}" != "1" ]; then
  echo "  ⚠️ 读不到裁定清单(operator_decisions 表还没建)—— 这一项没被检查,不是'没有'"
  echo "     修:bash scripts/apply-schema-to-live.sh"
  need=1
fi
if [ "${cards_n:-0}" = "0" ] && [ "${dec_n:-0}" = "0" ]; then echo "  (没有)"; else
  for id in $(PSQL "select id from acceptances where status='pending'"); do
    br=$(PSQL "select coalesce(branch,'?') from acceptances where id='$id'")
    base=$(PSQL "select coalesce(base,'?') from acceptances where id='$id'")
    how=$(PSQL "select coalesce(how_to_verify,'(没写验法)') from acceptances where id='$id'")
    # base 必须是真实存在的分支,否则这张卡点了只会报错 —— 必须明说
    #
    # 🔴 2026-10-28 13:00 补第二个条件,它才是真正会卡住 Accept 的那个:
    #    base 存在**不代表**卡的分支包含 base。track 往前走一个提交,
    #    分支就落后一个提交,验收门会拒绝未同步的分支 —— 张先生点任何一张
    #    都收到拒绝,而他不知道为什么。这个检查原来只查 base 存在,
    #    于是 23 张全点不动的卡被打成 23 个 "✅ 可以点"。
    #    那天早上我正好在犯这个错(开卡后往 track 提了 8 次),
    #    而唯一该提醒他的脚本在说反话。
    if ! git -C "$REPO" rev-parse --verify "refs/heads/$base" >/dev/null 2>&1; then
      echo "  ⚠️ 这张点不了(坏卡,不用管):$br"
      echo "     原因:要合进的分支「${base}」不存在,点了只会报错,不会损坏任何东西"
    elif ! git -C "$REPO" merge-base --is-ancestor "$base" "$br" 2>/dev/null; then
      behind=$(git -C "$REPO" rev-list --count "$br".."$base" 2>/dev/null)
      echo "  ⛔ 这张点了会被拒:$br"
      echo "     原因:分支落后「${base}」${behind} 个提交,验收门拒绝未同步的分支。"
      echo "     已派该席位的 owner 合并 track 进它的 packet 分支;同步完这张就能点。"
    else
      echo "  ✅ 可以点:$br"
      echo "     怎么验:$how"
    fi
    echo "     卡号 $id"
    echo "     类型:验收(交付做完了,要不要收)"
  done
  # 裁定类:目标、选项、定了之后会怎样。三样都列 ——
  # 说不清后果的选择是让他在黑箱里点,这正是 020 拒绝创建的那种。
  for id in $(PSQL "select id from operator_decisions where status='pending' and decider='human:supervisor' order by created_at"); do
    tgt=$(PSQL "select replace(target, E'\\n', ' ') from operator_decisions where id='$id'")
    ch=$(PSQL "select array_to_string(ARRAY(select jsonb_array_elements_text(choices)), ' / ') from operator_decisions where id='$id'")
    cons=$(PSQL "select replace(consequence, E'\\n', ' ') from operator_decisions where id='$id'")
    echo "  ⚖️ 要你裁定:$tgt"
    echo "     选项:$ch"
    echo "     定了之后:$cons"
    echo "     编号 $id"
    echo "     类型:裁定(只有你能定的事)"
  done
fi
echo

# ── 真的需要人 ────────────────────────────────────────────────
echo "── 真的需要人介入 ──"
need=0
if [ "$alive" != "$total" ]; then echo "  • 有席位掉线(见上)—— 修:bash scripts/seat-launch.sh --all --force"; need=1; fi
# waker 是一个 Rust 常驻守护进程(src/bin/waker.rs),不是一个 tmux 会话。
# 原来这里查 `tmux has-session -t seatloom-waker`,而那个会话在 2026-10-28 16:00
# 之前从未存在过 —— 二进制也没编译、没跑、没有日志。会话检查会为一个
# 恰好同名的空 tmux 会话报"在跑",而那正是发生过的事:我建了会话,
# 这一行立刻变绿,而守护进程依然没跑。查进程,不查会话名。
if ! pgrep -f 'seatloom-waker' >/dev/null 2>&1; then
  echo "  • 唤醒服务停了,新活不会被派下去 —— 起法:bash scripts/start-waker.sh"
  need=1
fi
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
# 同上:查进程。pgrep -f 的模式是 'seatloom-waker',会同时匹配到
# target/*/seatloom-waker 和包名 crate seatloom-mcp-emit 的路径,所以还要再确认
# 它真的是那个二进制的进程,而不是一条命令里提到了这个名字。
# 只认 waker 二进制本身的进程。pgrep -f 'seatloom-waker' 会抓到三类:
#   外层 zsh -c ...target/debug/seatloom-waker(是壳,不是守护进程)
#   target/debug/seatloom-waker          (这才是)
#   任何命令行里提到这个名字的进程(包括查它的那条命令自己)
# 所以用 ps 抓全表,再筛 command 以 target/ 下的可执行路径开头的那条。
waker_pid=""
for p in $(pgrep -f 'seatloom-waker' 2>/dev/null); do
  c=$(ps -p "$p" -o command= 2>/dev/null)
  case "$c" in
    target/*seatloom-waker|*/target/*seatloom-waker) waker_pid="$p"; break ;;
  esac
done
if [ -n "$waker_pid" ]; then
  printf "  唤醒服务             在跑(pid %s)\n" "$waker_pid"
else
  printf "  唤醒服务             ❌ 停了\n"
fi
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
      # 每小时 :00 一次(张先生 2026-10-20 定的,原为每 30 分钟)。阈值必须新于间隔,
      # 否则每轮都会把「还没到下一班」误报成「自醒死了」。间隔改,这里必须同步改。
      if [ "$ago" -le 75 ]; then
        printf "  Aegis 自醒           在跑(%s 分钟前最后一次)\n" "$ago"
      else
        printf "  Aegis 自醒           ❌ %s 分钟没有记录了(每小时 :00 一次,超过 75 分钟就是死了)\n" "$ago"
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
# 所以:先用名字抓到候选,再按**可执行名**过滤掉包装进程,最后从真实命令行判断是哪个构建。
# 🔴 2026-10-16 修:原来这里是 `pgrep -f 'seatloom-tauri' | head -1`,那是错的。
#    `pgrep -f` 匹配**完整命令行**,所以任何命令行里含 "seatloom-tauri" 的进程都会中 ——
#    Aegis 自己的 bash 命令、cargo 构建、tmux 的 sh -c 全都中过。`head -1` 再随便挑一个,
#    于是把一个已经退出的构建进程报成「SeatLoom 窗口 开着」,而应用其实没开。
#    症状:一个永远不会报错的检查 —— 比没有检查更有害。
#    判据改成 `ps -o comm=`:真正的应用进程可执行名就是 seatloom-tauri,
#    包装进程(bash/sh/cargo/node)的可执行名不是,天然分开。
app_pid=""; app_which=""
for _p in $(pgrep -f 'seatloom-tauri' 2>/dev/null); do
  _comm="$(ps -o comm= -p "$_p" 2>/dev/null | tr -d ' ')"
  # macOS 的 comm 给的是**完整路径**(/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri),
  # 不是 basename —— 第一次写成等于 seatloom-tauri,结果真应用进程反而不中,
  # 而包装进程(/bin/bash)照样被排除,症状是应用明明在跑却报「没开」。
  case "$_comm" in
    seatloom-tauri|*/seatloom-tauri) ;;
    *) continue;;
  esac
  _cmd="$(ps -o command= -p "$_p" 2>/dev/null)"
  case "$_cmd" in
    */Applications/SeatLoom.app/*) app_pid="$_p"; app_which="装好的那份(/Applications)"; break;;
    *target/release/*)           [ -z "$app_pid" ] && { app_pid="$_p"; app_which="release = 集成构建"; };;
    *target/debug/*)             [ -z "$app_pid" ] && { app_pid="$_p"; app_which="debug = 可能没有面板"; };;
    *)                           [ -z "$app_pid" ] && { app_pid="$_p"; app_which="来路不明(相对路径起的)"; };;
  esac
done
if [ -z "$app_pid" ]; then
  printf "  SeatLoom 窗口        %s\n" "没开"
else
  printf "  SeatLoom 窗口        开着(%s, pid %s)\n" "$app_which" "$app_pid"
  # 装好的那份会比刚构建的二进制旧 —— 那意味着张先生从启动台点开的可能是没有面板的版本。
  # 这是「部分换动」在应用层的同一形状:进程在跑,不等于跑的是最新的。
  APP_BIN="/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri"
  BUILT_BIN="$REPO/target/release/seatloom-tauri"
  if [ -x "$APP_BIN" ] && [ -x "$BUILT_BIN" ]; then
    app_e="$(stat -f %m "$APP_BIN")"; built_e="$(stat -f %m "$BUILT_BIN")"
    if [ "$app_e" -lt "$built_e" ]; then
      printf "                     ⚠️  装好的那份比刚构建的旧 —— 从启动台点开的就是没有面板的版本\n"
      printf "                     修:cp -R target/release/bundle/macos/SeatLoom.app /Applications/\n"
      need=1
    fi
  fi
fi
echo
printf "  累计完成派单         %s 件\n" "$(PSQL "select count(*) from dispatches where status='done'")"
