#!/usr/bin/env bash
# fleet-process-census.sh — 逐席位的进程普查(只读)。
#
# 为什么需要它:Lyra 2026-10-12 要一次「逐席位 agent pid/起始时间、MCP 子进程
# pid/起始时间、工具数」的普查。它说自己只能看到自己的界面,这正是它误判
# 「我的进程是最新的」的原因 —— 而部分换动比不换动更糟:它让那句话失效。
#
# 它回答三个问题,每个都只读:
#   1. 这个席位的 agent 进程是哪个、什么时候起的
#   2. 它的 MCP 子进程是哪个、什么时候起的、**跑的是不是盘上这份二进制**
#   3. 盘上这份二进制对该席位可见多少个工具
#
# 第 2 问是关键,也是唯一能判定新旧镜像的判据:席位的 MCP 子进程是它启动那一刻的
# 二进制镜像。所以「进程在不在」不算数,要「进程起始时间 vs 盘上二进制 mtime」——
# 进程起得比二进制旧,它就是旧镜像。重建二进制不等于全队换上了新的。
#
# 关于第 3 问的诚实边界(重要):这里报的是**盘上二进制**对该席位可见的工具数,
# 不是那个席位活着的子进程里的工具数 —— 后者问不出来(不能跨进程查别人的 MCP)。
# 所以若判定为旧镜像,它的真实工具数只会更少,这里明确说"未知(≤ 盘上报数)",
# 绝不把 18 写成"它现在有 18 个"。
#
# 它不写任何东西:不改账本、不改进程、不重启席位。重启是
# scripts/restart-seats-for-new-tools.sh 的事,不是普查的事。
#
# 用法: bash scripts/fleet-process-census.sh
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

MAP="$REPO/.seatloom/dispatch-seats.tsv"
BIN="$REPO/target/debug/seatloom-mcp-emit"
DBURL="postgresql://seatloom:seatloom@localhost:5432/seatloom"

TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux; do
  [ -n "$c" ] && [ -x "$c" ] && { TMUX_BIN="$c"; break; }
done
[ -n "$TMUX_BIN" ] || { echo "❌ 找不到 tmux —— 普查做不了,不要假装做完了"; exit 1; }
[ -x "$BIN" ] || { echo "❌ MCP 二进制不在:$BIN —— 后面的工具数全都查不出,不要报 0"; exit 1; }

# 进程起始时间(epoch)。macOS 的 ps 没有 etimes,用 lstart 解析。
# 每一步都校验:解析不出来就返回空,由调用方报"未知",绝不让垃圾值当时间用。
start_epoch(){
  local raw e
  raw="$(ps -o lstart= -p "$1" 2>/dev/null | sed 's/^ *//;s/ *$//')"
  [ -n "$raw" ] || return 0
  e="$(date -j -f "%a %b %d %H:%M:%S %Y" "$raw" +%s 2>/dev/null)"
  case "$e" in ''|*[!0-9]*) return 0;; esac   # 空或非数字 ⇒ 当作未知
  printf '%s' "$e"
}
when_of(){ local e; e="$(start_epoch "$1")"; [ -n "$e" ] && date -r "$e" '+%m-%d %H:%M' || echo "未知"; }
age_min(){ local e; e="$(start_epoch "$1")"; [ -n "$e" ] && echo $(( ($(date +%s)-e)/60 )) || echo "未知"; }

BIN_EPOCH="$(stat -f %m "$BIN")"
BIN_WHEN="$(date -r "$BIN_EPOCH" '+%m-%d %H:%M')"

# 以一个席位的身份问**盘上**这份二进制要工具清单。只发 initialize + tools/list。
tool_count(){
  local seat="$1"
  printf '%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"census","version":"1"}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | SEATLOOM_ACTOR_REF="seat:$seat" DATABASE_URL="$DBURL" SEATLOOM_PROJECT_ID=seatloom \
      "$BIN" 2>/dev/null \
  | python3 -c '
import sys, json
for l in sys.stdin:
    l=l.strip()
    if not l: continue
    try: m=json.loads(l)
    except Exception: continue
    if m.get("id")!=2: continue
    if "error" in m: print("ERR"); break
    print(len(m["result"]["tools"])); break
' 2>/dev/null
}

echo "普查时间 $(date '+%Y-%m-%d %H:%M:%S')   盘上二进制 mtime=$BIN_WHEN"
echo "判据:MCP 子进程起得比二进制旧 ⇒ 它跑的是旧镜像(重建二进制≠它换上了新的)"
echo

stale=0; total=0
while IFS=$'\t' read -r seat sess; do
  case "$seat" in ''|\#*) continue;; esac
  [ -n "${sess:-}" ] || continue
  total=$((total+1))

  pane="$("$TMUX_BIN" list-panes -t "$sess" -F '#{pane_pid}' 2>/dev/null | head -1)"
  if [ -z "$pane" ]; then
    printf "%-18s %-28s ❌ pane 不存在(会话没起)\n" "$seat" "$sess"
    stale=$((stale+1)); continue
  fi
  pane_cmd="$("$TMUX_BIN" list-panes -t "$sess" -F '#{pane_current_command}' 2>/dev/null | head -1)"

  # agent = pane 的直接子进程。AGENTS.md 第 2 条:查子进程,不裸 grep pi。
  agent="$(pgrep -P "$pane" 2>/dev/null | head -1)"
  if [ -z "$agent" ]; then
    printf "%-18s %-28s ❌ pane 里没有 agent 进程(pane 跑着 '%s')\n" "$seat" "$sess" "$pane_cmd"
    stale=$((stale+1)); continue
  fi
  agent_cmd="$(ps -o command= -p "$agent" 2>/dev/null | cut -c1-40)"

  # MCP 子进程 = agent 的子进程里那个二进制。
  mcp=""
  for g in $(pgrep -P "$agent" 2>/dev/null); do
    case "$(ps -o command= -p "$g" 2>/dev/null)" in *seatloom-mcp-emit*) mcp="$g"; break;; esac
  done

  # 盘上二进制对该席位可见的工具数 —— 只回答"盘上这份能给多少",不冒充"它现在有多少"。
  n="$(tool_count "$seat")"

  printf "%-18s %s\n" "$seat" "$sess"
  printf "    agent  pid=%-7s 起于 %s(%s 分钟前) %s\n" \
    "$agent" "$(when_of "$agent")" "$(age_min "$agent")" "$agent_cmd"

  if [ -z "$mcp" ]; then
    printf "    MCP    ❌ 没有 MCP 子进程 —— 这个席位的工具一个都没加载(静默失效)\n"
    stale=$((stale+1))
  else
    me="$(start_epoch "$mcp")"
    if [ -z "$me" ]; then
      printf "    MCP    pid=%-7s 起于 未知 ⇒ 判不了新旧(时间取不到,按需处理)\n" "$mcp"
      stale=$((stale+1))
    elif [ "$me" -lt "$BIN_EPOCH" ]; then
      printf "    MCP    pid=%-7s 起于 %s ⇒ 旧镜像(二进制 %s 才构建)\n" "$mcp" "$(when_of "$mcp")" "$BIN_WHEN"
      printf "           它的真实工具数未知,只可能 ≤ 盘上报数 %s —— 别把 %s 当成它现在有的\n" "$n" "$n"
      stale=$((stale+1))
    else
      printf "    MCP    pid=%-7s 起于 %s ⇒ 当前镜像  工具数=%s\n" "$mcp" "$(when_of "$mcp")" "$n"
    fi
  fi

  [ "$n" = "ERR" ] && { printf "   ⠠ 以 %s 身份问 tools/list 报错 —— 这本身就是个发现\n" "$seat"; stale=$((stale+1)); }
done < "$MAP"

echo
echo "────────────────────────────────────────────────────────────"
if [ "$stale" -eq 0 ]; then
  echo "✅ ${total}/${total} 个席位都跑着当前镜像"
else
  echo "⚠️  ${stale} 个席位需要处理(旧镜像 / 无 MCP 子进程 / 工具数报错 / 时间取不到)"
  echo "   处理走 scripts/restart-seats-for-new-tools.sh —— 普查只报,不动手"
fi
