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
#
# 账本不在时,这两个数是空字符串而不是 0。「(没有)」和「读不到」必须分开 ——
# 2026-10-31 整个 live 库被清空,这一节因此打印「(没有)」,而他名下其实压着
# 5 张待验收卡。一个沉默的 0 比一个明说的空更危险。
#
# 这里只数,不打印。打印在下面的「等你拍板」一节,因为它属于那一节的话。
# 在结论行里提前打印,会把一条属于「等你拍板」的消息扔到快照最上面,
# 读起来像另一件事。
cards_n=$(PSQL "select count(*) from acceptances where status='pending'" 2>/dev/null)
cards_readable=1
if ! printf '%s' "$cards_n" | grep -qE '^[0-9]+$'; then
  cards_n=""
  cards_readable=0
fi
dec_n=$(PSQL "select count(*) from operator_decisions where status='pending' and decider='human:supervisor'" 2>/dev/null)
dec_readable=1
if ! printf '%s' "$dec_n" | grep -qE '^[0-9]+$'; then
  dec_n=""
  dec_readable=0
fi
if [ "${busy_n:-0}" -gt 0 ]; then
  echo "▶ 项目在推进中:${busy_n} 个席位在干活,${ready_n} 件排队。"
elif [ "${ready_n:-0}" -gt 0 ]; then
  echo "⏸ 有活排队但没人在做 —— 可能需要唤醒(见下方「唤醒服务」)。"
elif [ "${blocked_n:-0}" -gt 0 ]; then
  echo "⏸ 全部停在依赖上:${blocked_n} 件在等前置条件完成。"
else
  # 读不到队列表时,"队列是空的"是一句谎。说读不到。
  if ! printf '%s' "${ready_n:-}" | grep -qE '^[0-9]+$'; then
    echo "⏸ 读不到队列(dispatches 表不在 live 库里)—— 这一项没被检查,不是'空的'"
  else
    echo "⏸ 全队空闲,队列是空的。"
  fi
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
# 表可能还没建到库里(迁移由张先生执行),也可能建过然后被清空。
# 这两种事实对听者的含义相反,所以话说得不一样:读不到就说读不到,
# 不说"还没建"。而无论是哪一种,都不能当成 0 —— 一个沉默的 0 和一个
# 真实的 0 不是一回事。
if [ "$cards_readable" != "1" ]; then
  echo "  🔴 读不到验收卡清单(acceptances 表不在 live 库里)—— 这一项没被检查,不是'没有'"
  echo "     他名下可能压着待验收的卡,而这里一条也列不出来。"
  echo "     先跑:bash scripts/check-ledger-exists.sh"
  need=1
fi
if [ "$dec_readable" != "1" ]; then
  echo "  ⚠️ 读不到裁定清单(operator_decisions 表不在 live 库里)—— 这一项没被检查,不是'没有'"
  echo "     修:bash scripts/apply-schema-to-live.sh"
  need=1
fi
if [ -z "$cards_n" ]; then
  :  # 已经明说读不到了,不要再补一个「(没有)」
elif [ "${cards_n:-0}" = "0" ] && [ "${dec_n:-0}" = "0" ]; then
  # 0 张可能是真的没有,也可能是全被清空了。2026-10-31 是后者:
  # 5 张待验收卡连记录一起没了。说清是哪种,不要说「(没有)」。
  # 判据不是「事件数」—— 事故后 Lyra 自己发了一条 ArtifactCreated,
  # 所以 canonical_events 有 1 行,不是 0。真正的判据是**有没有接受记录**:
  # 一个健康的账本不可能 0 张已接受的卡。
  _acc=$(PSQL "select count(*) from acceptances where status='accepted'" 2>/dev/null)
  if [ "${_acc:-0}" = "0" ]; then
    echo "  ⚠️ 0 张 —— 但账本也是空的(库 10-31 00:34 被清空后重建)。"
    echo "     这 5 张卡连记录一起没了:首屏默认页 ×2、主题能关闭、待验收卡计数、SAMPLE 徽章。"
    echo "     代码都在 git 上(46 个提交),要重开卡,不是从零开始。"
  else
    echo "  (没有)"
  fi
else
  for id in $(PSQL "select id from acceptances where status='pending'"); do
    br=$(PSQL "select coalesce(branch,'?') from acceptances where id='$id'")
    base=$(PSQL "select coalesce(base,'?') from acceptances where id='$id'")
    how=$(PSQL "select coalesce(how_to_verify,'(没写验法)') from acceptances where id='$id'")
    # base 必须是真实存在的分支,否则这张卡点了只会报错 —— 必须明说
    #
    # 🔴 2026-11-05 03:43:40 UTC 张先生裁定(dec-clickable-criterion-level-with-base):
    #    clickability 只问「合并会不会丢掉主线提交或删掉文件」;
    #    「和主线齐平」不再是拒绝的理由,containment 留在 request_acceptance。
    #
    # 🔴🔴 2026-11-08 01:50 修:上面那段 10-28 的理由**本身就是被推翻的那条**,
    #    而它还一直在跑,并且打印一句假话:
    #      「已派该席位的 owner 合并 track 进它的 packet 分支」
    #    实测这一轮,两句话都不成立 —— 没有派过单,卡也点得动。
    #    我用 check-cards-clickable.sh 量过同一对分支:合并干净、删 0 文件、丢 0 提交。
    #    那个脚本 11-05 就按裁定改了,这一个没有 —— 于是同一个仓库里两个检查
    #    给出相反答案,而张先生看的是这一个。
    #
    #    为什么 accept 路径真的不看齐平:它跑 `git merge --no-ff`(acceptance_cmds.rs:311),
    #    --no-ff 天生不会丢提交。`branch_contains` 全代码只有一个调用点,
    #    在 request_acceptance(dispatch.rs:1808)—— 开卡那一刻。
    #
    #    现在问裁定问的那个问题:合并会不会冲突、会不会删主线文件。
    #    落后只作备注,不作拒绝 —— 原因写在下面。
    if ! git -C "$REPO" rev-parse --verify "refs/heads/$base" >/dev/null 2>&1; then
      echo "  ⚠️ 这张点不了(坏卡,不用管):$br"
      echo "     原因:要合进的分支「${base}」不存在,点了只会报错,不会损坏任何东西"
    else
      # 真正会让点击失败的东西:合并冲突,或删掉主线跟踪的文件。
      # merge-tree 不动任何 ref、不改工作区,干净时退 0。
      if MERGED=$(git -C "$REPO" merge-tree --write-tree "$base" "$br" 2>/dev/null); then
        DELETED=$(git -C "$REPO" diff --diff-filter=D --name-only "$base" "$MERGED" 2>/dev/null | wc -l | tr -d ' ')
        echo "  ✅ 可以点:$br"
        echo "     怎么验:$how"
        # 落后是事实,要说,但不是拒绝的理由(裁定原文)。
        behind=$(git -C "$REPO" rev-list --count "$br".."$base" 2>/dev/null)
        [ "$behind" = "0" ] || echo "     备注:分支落后「${base}」${behind} 个提交 —— 不影响点击,合并不会丢提交"
        [ "$DELETED" = "0" ] || echo "     ⚠️ 合并会删掉 ${DELETED} 个主线文件,点之前先看清是什么"
      else
        echo "  ⛔ 这张点了会被拒:$br"
        echo "     原因:把它合进「${base}」会冲突 —— 点了只会失败。"
        echo "     修:让该席位 owner 把 ${base} 合进它的 packet 分支(用 merge,不要 rebase)再 push。"
        echo "     这不是它的错,落后通常是我提交造成的。"
      fi
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
# 席位没注册 = 任何派给它的单都插不进去(target_seat_id 有 FK 到 seats)。
# 2026-10-31 事故后 schema 重建只留下 4 个外部项目席位,五个核心席位不在表里,
# 于是整个舰队无法再被派活,而面板从 tmux 读,照样显示「在线 6/6」。
# 一个查进程、一个查账本,两者都绿才真的是绿的。
_seat_n=$(PSQL "select count(*) from seats" 2>/dev/null)
_core_n=0
for _s in seat-mira-001 seat-nimbus-001 seat-flux-001 seat-onyx-001 seat-lyra-001 seat-vela-001; do
  [ "$(PSQL "select count(*) from seats where id='$_s'" 2>/dev/null)" = "1" ] && _core_n=$((_core_n+1))
done
if [ "${_core_n:-0}" -lt 6 ]; then
  echo "  • 🔴 六个核心席位只有 ${_core_n:-0} 个注册在 seats 表里(共 ${_seat_n:-?} 行)。"
  echo "      target_seat_id 有 FK 到 seats(id),所以派给未注册席位的单插不进去。"
  echo "      面板显示『在线 6/6』是从 tmux 读的,和账本是两回事。"
  echo "      修:bash scripts/register-fleet-seats.sh  (写 live 库,要你执行)"
  need=1
fi
# 账本不在,是这里最要紧的一件事,必须排在第一行。2026-10-31 它没在这一节里,
# 于是快照打印"没有需要你动手的事",而整个账本刚被清空。
if [ "$cards_readable" != "1" ]; then
  echo "  • 🔴 live 库里 public schema 的表少于 30 张 —— 账本不在。"
  echo "      下面的每个数字都在读不存在的表,每个 0 都不是『没有』。"
  echo "      先跑:bash scripts/check-ledger-exists.sh"
  need=1
fi
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

# 🔴 第一件事:账本还在吗。2026-10-31 加的 —— 那天整个 live 库被清空,
#    而我七个每小时跑一次的检查没有一个发现,因为它们问的都是「有多少」,
#    而一个不存在的表和一个空的表给出同一个数字:0。0 读起来像「没事可做」,
#    不像「全没了」。这一条必须在任何计数之前,并且必须能 FAIL。
if bash "$REPO/scripts/check-ledger-exists.sh" --quiet; then
  :
else
  echo "  🔴 账本不在:live 库里 public schema 的表少于 30 张。"
  echo "     下面所有的数字(待验收几张、记忆几条、派单几件)都是在读不存在的表,"
  echo "     每一个 0 都不是『没有』,是『读不到』。**先看这个,别先看别的。**"
  echo "     定位:bash scripts/check-ledger-exists.sh"
fi

mig=$(PSQL "select max(substring(f from '[0-9]+')) from (select unnest(array['x']) f) t" >/dev/null 2>&1; ls "$REPO"/infra/postgres/schema/*.sql 2>/dev/null | tail -1 | sed 's#.*/##;s/_.*//')
printf "  代码里的最新表结构   %s\n" "${mig:-读不到}"
if [ "$(PSQL "select count(*) from pg_tables where tablename='memories'")" = "1" ]; then
  _mem=$(PSQL "select count(*) from memories")
  # 2026-10-31:库被清空后重建。schema 回来了,数据没有。
  # 「已上线(0 条记忆」读起来像「还没开始记」,而事实是「记过 452 条,然后没了」。
  # 这两种含义对张先生完全不同,所以 0 要说清是哪种 0。
  if [ "${_mem:-0}" = "0" ]; then
    printf "  记忆功能             ⚠️ 表在,0 条记忆 —— 库在 10-31 00:34 被清空后重建,452 条没了\n"
  else
    printf "  记忆功能             已上线(%s 条记忆)\n" "$_mem"
  fi
else
  # 不写「还没建」—— 那次的事实是「建过,然后被清空了」。这两个说法对听者的
  # 含义完全相反,而我差点让产品说错这一个。
  printf "  记忆功能             ❌ 读不到(memories 表不在 live 库里)\n"
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
# 🔴 2026-11-07 22:15 修:上一版把「没有日志」直接打成 ❌ 自醒死了,
#    那是把一个**无法证明**的结论说成了**已证明为假**。
#
#    实测形状:日志和 seatloom-waker 的日志**都**在 22:11 前从 /tmp 消失,
#    而同一批里的 .txt 和子目录都在。两件事同时指向外部清理,而不是自醒死了。
#    而我按协议去"修"它,只会把一个健康的系统改坏 —— 或者更糟:
#    为了让它变绿而重装 cron,把原本正常的配置覆盖掉。
#
#    **判据必须是「能不能证明」,不是「文件在不在」。** 文件不在只说明
#    这个信使不在了,不说明信使要说的事是假的。
#
#    现在分三层,每层只报告它能证明的东西:
#      1. 日志在且新  → 在跑(可证)
#      2. 日志在但旧  → ❌ 真的超过间隔没有记录(可证)
#      3. 日志不在    → ⚠️ 无法证明,而不是宣布它死了。
#         改去查两个独立事实:cron 条目在不在、被 cron 跑的那个脚本
#         在不在且可执行。这两个在,自醒就是装着的;不在,才是真的坏了。
#
#    为什么不用「pane 里有没有心跳指令」当替代证据:试过,不可靠。
#    pane 的 scrollback 里有**我自己**的命令输出 —— 我拿一个「绝不该出现」
#    的控制字符串去撞它,返回 1,因为我那条含该字符串的命令本身就在里面。
#    **一个会被测量动作自身污染的通道,不能当证据。**
WLOG="/tmp/seatloom_aegis_waker.log"
CRON_SCRIPT="$HOME/.local/bin/aegis-self-wake.sh"
if [ ! -s "$WLOG" ]; then
  # 第三层:信使不在。查两个可证的独立事实,不宣布死亡。
  cron_ok=0; script_ok=0
  crontab -l 2>/dev/null | grep -q 'aegis-self-wake.sh' && cron_ok=1
  [ -x "$CRON_SCRIPT" ] && script_ok=1
  if [ "$cron_ok" = "1" ] && [ "$script_ok" = "1" ]; then
    printf "  Aegis 自醒           ⚠️  无法证明(日志 %s 被删)\n" "$WLOG"
    need=1
    printf "                     cron 条目在,cron 跑的脚本也在且可执行 —— 所以它是装着的,\n"
    printf "                     只是这一小时没有留下记录。别当成死了去重装。\n"
    printf "                     真想确认:直接跑一次它,看 pane 收不收到:\n"
    printf "                       bash %s\n" "$CRON_SCRIPT"
  else
    printf "  Aegis 自醒           ❌ 没有日志,而且自醒没装好\n"
    need=1
    [ "$cron_ok" = "0" ] && printf "                     crontab 里没有 aegis-self-wake 这一条\n"
    [ "$script_ok" = "0" ] && printf "                     %s 不在或不可执行\n" "$CRON_SCRIPT"
    printf "                     修:bash scripts/install-aegis-self-wake.sh\n"
  fi
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
# 一个空字符串不是 0。「累计完成派单        件」中间那个洞,是这张表不在的信号,
# 而 0 会把「表没了」显示成「一件都没做完过」。宁可留洞,不要给一个假的 0。
_done="$(PSQL "select count(*) from dispatches where status='done'" 2>/dev/null)"
if printf '%s' "$_done" | grep -qE '^[0-9]+$'; then
  if [ "$_done" = "0" ]; then
    printf "  累计完成派单         0 件 —— 库被清空后重建,约 400 件派单记录没了\n"
  else
    printf "  累计完成派单         %s 件\n" "$_done"
  fi
else
  printf "  累计完成派单         读不到(dispatches 表不在 live 库里)\n"
fi
