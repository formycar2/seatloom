#!/usr/bin/env bash
# saih-verify-seat.sh — 对一个席位跑完 saih 迁移的四层验真。
#
# 为什么单独有这个脚本:saih/docs/SWITCHING-SEATS.md §3 列了四层验真,但没给一条命令。
# 而第四层(账本上派单 ready → in_flight)是唯一硬证据,恰恰是最容易跳过的那层 ——
# 前三层看着都绿,第四层不做,迁移就是"未证明"。
#
# 四层(照文档):"
#   1 面板里那个席位跑的是 saih,不是 pi
#   2 从席位自己的工作区跑 saih mcp list,显示连上、22 个工具
#   3 唤醒服务的进程名门槛含 saih
#   4 🔴 账本上一条派单从 ready 翻成 in_flight —— 唯一硬证据
#
# 用法:
#   bash scripts/saih-verify-seat.sh <tmux 会话名> [派单 id]
#
# 例:
#   bash scripts/saih-verify-seat.sh Onyx-data-seatloom
#
# 第四层要一条派单。两种给法:
#   · 传第二个参数(派单 id)
#   · 不传:脚本从库里找一条派给这个席位、状态还是 ready 的单
#     (所以迁移前先给这个席位派一件小活,派单 id 让脚本自己找)
#
# 🔴 一层不过就停,后面不跑。set -e 加 ERR 陷阱,报错时说清死在哪一行 ——
#    一个会接着往下跑的验真脚本,会把「ALL PASS」印在一个测出来的缺陷上。
set -euo pipefail
trap 'echo "❌ 验真中断(第 $LINENO 行)" >&2' ERR

SESSION="${1:?用法: bash scripts/saih-verify-seat.sh <tmux 会话名> [派单 id]}"
DISPATCH="${2:-}"
SEATLOOM="${SEATLOOM_ROOT:-$HOME/Documents/GitHub/seatloom}"
SAIH_BIN="${SEATLOOM_PI_BIN:-$HOME/Documents/GitHub/seatloom/saih/bin/saih}"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
MAP="$SEATLOOM/.seatloom/dispatch-seats.tsv"

# 从映射表反查这个会话属于哪个席位(不靠猜)
seat_id="$(awk -F'\t' -v s="$SESSION" '$2==s || $3==s {print $1; exit}' "$MAP" 2>/dev/null || true)"
if [ -z "seat_id" ]; then
  echo "❌ 在 $MAP 里找不到会话 $SESSION 对应的席位 —— 先确认会话名和映射表"
  exit 1
fi
# 工作区从席位 id 推:seat-onyx-001 → ~/Documents/GitHub/seatloom-worktrees/onyx
# 映射表只有两列(席位、会话),没有工作区列,所以不从那读。
short="${seat_id#seat-}"; short="${short%%-*}"
worktree="$SEATLOOM/../seatloom-worktrees/$short"
[ -d "$worktree" ] || worktree="$SEATLOOM"
echo "  推定工作区: $worktree"

echo "=== 会话 $SESSION / 席位 $seat_id / 工作区 $worktree ==="
echo

echo "########## 第 1 层:面板里跑的是 saih ##########"
pane="$(tmux list-panes -t "$SESSION" -F '#{pane_pid}' 2>/dev/null | head -1)"
if [ -z "$pane" ]; then echo "❌ 会话 $SESSION 不存在"; exit 1; fi
child="$(pgrep -P "$pane" 2>/dev/null | head -1)"
comm="$(ps -o comm= -p "$child" 2>/dev/null | tr -d ' ')"
echo "  面板子进程: ${comm:-（没读到）}"
if [ "$comm" != "saih" ]; then
  echo "❌ 跑的是 '$comm' 不是 'saih' —— 迁移没生效,或者迁错了席位"
  exit 1
fi
echo "  ✅ 是 saih"
echo

echo "########## 第 2 层:从它自己的工作区连 MCP ##########"
out="$(cd "$worktree" && "$SAIH_BIN" mcp list 2>&1 || true)"
echo "$out" | grep -iE "seatloom.*(connected|tools)" | head -2 | sed 's/^/  /'
tools="$(echo "$out" | grep -oE "[0-9]+ tools?" | grep -oE "^[0-9]+" | head -1)"
if [ -z "$tools" ] || [ "$tools" -lt 1 ]; then
  echo "❌ 没读到工具数 —— MCP 没连上。原文:"
  echo "$out" | head -5 | sed 's/^/    /'
  exit 1
fi
echo "  ✅ 读到 $tools 个工具"
echo

echo "########## 第 3 层:唤醒服务认 saih ##########"
if grep -q 'node|bun|pi|saih' "$SEATLOOM/scripts/aegis-self-wake.sh"; then
  echo "  ✅ 门槛含 saih"
else
  echo "❌ aegis-self-wake.sh 的进程名门槛不含 saih —— 我看起来活着但永远收不到心跳"
  exit 1
fi
echo

echo "########## 第 4 层:账本上派单 ready → in_flight 🔴 唯一硬证据 ##########"
if [ -z "$DISPATCH" ]; then
  DISPATCH="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc \
    "select id from dispatches where target_seat_id='$seat_id' and status='ready' order by priority desc limit 1" \
    2>/dev/null | tr -d ' ')"
fi
if [ -z "$DISPATCH" ]; then
  echo "❌ 没给派单 id,库里也没有派给 $seat_id 的 ready 单。"
  echo "   先给它派一件小活,再把派单 id 传给这个脚本。"
  exit 1
fi
echo "  盯着派单: $DISPATCH"

before="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc \
  "select status from dispatches where id='$DISPATCH'" 2>/dev/null | tr -d ' ')"
echo "  当前状态: ${before:-（读不到）}"

flipped=0
for i in $(seq 1 20); do
  sleep 15
  now="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc \
    "select status from dispatches where id='$DISPATCH'" 2>/dev/null | tr -d ' ')"
  if [ "$now" = "in_flight" ]; then flipped=1; echo "  ✅ 翻了:ready → in_flight(第 $((i*15)) 秒)"; break; fi
  if [ "$now" != "$before" ]; then echo "  状态变成 '$now'(不是 in_flight),继续等"; before="$now"; fi
done

if [ "$flipped" != "1" ]; then
  echo "❌ 等了 5 分钟,派单没从 ready 翻成 in_flight。"
  echo "   前三层绿而第四层不绿 = MCP 没真通,或者席位没被唤醒。"
  echo "   🔴 这不算迁移成功。别接着迁别的席位。"
  exit 1
fi

echo
echo "=== 四层全过:$SESSION 在 saih 上,而且真领到了单 ==="
echo "派单 $DISPATCH 现在是 in_flight。这是唯一硬证据。"
echo "确认无误后再迁其余席位:"
echo "  source $SEATLOOM/saih/seatloom-saih.env"
echo "  cd $SEATLOOM && scripts/seat-launch.sh --all"
