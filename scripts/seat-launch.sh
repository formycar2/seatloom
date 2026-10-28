#!/usr/bin/env bash
# seat-launch.sh — (re)launch a SeatLoom seat on pi, in its own worktree, with a pinned session ID.
#
# Two reasons this exists:
#  1. MCP: a seat only reaches the seatloom MCP when pi runs with the seat's worktree
#     as cwd, because that is where its .pi/mcp.json lives. A seat launched elsewhere —
#     or as Claude Code, which reads .mcp.json rather than .pi/mcp.json — comes up with
#     NO MCP, cannot call get_next_dispatch, and the dispatch queue stalls silently.
#  2. Traceability: --session-id makes the seat→transcript mapping exact instead of
#     guessed, which scripts/seat-status.sh and the ingestion layer both need.
#
# Usage:
#   scripts/seat-launch.sh <seat-session>     relaunch one seat
#   scripts/seat-launch.sh --all              relaunch every *-seatloom seat
#   scripts/seat-launch.sh --map              print the current seat→session map
#
#   scripts/seat-launch.sh --resume-uuid <uuid> <seat-session>
#     Pin the session id to an EXISTING one instead of minting a fresh uuid.
#     Use this right after saih/transfer-session.sh: the transferred transcript
#     carries the OLD pi session's uuid, and a fresh --session-id would start an
#     empty session — so "migrate the memory" would silently do nothing.
#     Pinning keeps both: exact seat→transcript mapping AND history continuity.
#
#     Extract the uuid from the transferred file name, which is
#     <timestamp>_<uuid>.jsonl — the part after the underscore.
#
# Refuses to relaunch a seat that is mid-turn unless --force is given: killing a
# working agent loses the turn. Never touches non-seatloom tmux sessions.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# 回退用的默认模型。真实模型**按席位从 seats.default_model 取**(见 model_for) ——
# 写死一个值给所有席位,会在重启时静默把席位降级。2026-10-11 实测踩过:
# 洗数据席位登记的是 stepplan-wash-anthropic(无限额,存在的意义就是不必为省 token 降低质量),
# 重启后被换成普通额度的 stepplan-anthropic,而且面板上不会有任何告警。
MODEL="${SEATLOOM_MODEL:-modelproxy-anthropic/step-5-preview-sys}"
PI="${SEATLOOM_PI_BIN:-$HOME/.local/bin/pi}"
WORKTREES="${SEATLOOM_WORKTREES:-$REPO/../seatloom-worktrees}"
MAP="$REPO/.seatloom/seat-sessions.tsv"
SEATMAP="$REPO/.seatloom/dispatch-seats.tsv"
PG="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
FORCE=0

# model_for <tmux 会话名> —— 从席位表查这个席位登记的模型。
# 查不到(容器没起/席位没登记/字段为空)就回退到 $MODEL,并在启动行里说明用的是哪个来源,
# 这样「降级了」永远是看得见的,不是静默的。
#
# ⚠️ 必须校验形状:seats.default_model 里有历史遗留值。实测 2026-10-11,
#    五个席位登记的是 `claude-opus-5[1m]` —— 那是 Claude Code 时代的模型名,
#    不是 pi 的 `provider/model` 格式,原样传给 pi 会直接起不来。
#    所以只接受含 `/` 的值(pi 的格式),其余一律回退并说明原因。
#    第一版漏了这个校验,会把五个席位修坏 —— 比它要修的那个回退更糟。
model_for(){
  local sess="$1" sid m
  sid="$(awk -F'\t' -v s="$sess" '$2==s {print $1}' "$SEATMAP" 2>/dev/null | head -1)"
  [ -z "${sid:-}" ] && { printf '%s\t%s' "$MODEL" "默认(席位表里没有这个会话)"; return; }
  m="$(podman exec -e PGPASSWORD=seatloom "$PG" psql -U seatloom -d seatloom -tAc \
        "select coalesce(default_model,'') from seats where id='$sid'" 2>/dev/null | tr -d ' ')"
  if [ -z "${m:-}" ]; then
    printf '%s\t%s' "$MODEL" "默认(${sid} 未登记模型)"
  elif [ "${m#*/}" = "$m" ]; then
    # 没有 `/` —— 不是 pi 的格式,是遗留值。不能用。
    printf '%s\t%s' "$MODEL" "默认(席位表登记的 '${m}' 不是 pi 的 provider/model 格式,已忽略)"
  else
    printf '%s\t%s' "$m" "seats.default_model"
  fi
}

mkdir -p "$(dirname "$MAP")"
[ -f "$MAP" ] || printf 'seat\tsession_id\tlaunched_at\tmodel\n' > "$MAP"

is_busy() {
  tmux capture-pane -p -t "$1" 2>/dev/null \
    | grep -qE '^[^ ]? ?[A-Z][a-z]+(ing|ed)… \([0-9]+[ms]'
}

launch_seat() {
  local seat="$1" uuid
  # 2026-10-17: --resume-uuid lets the caller pin the session id instead of
  # minting a fresh one. Without it a transferred session is orphaned: the
  # transcript lands under the OLD pi session's uuid, and a fresh --session-id
  # starts an empty one, so "migrate the memory" silently does nothing.
  # Pinning keeps BOTH properties the comment above claims — exact seat→transcript
  # mapping (the map records the same id) AND continuity with the transferred history.
  if [ -n "${RESUME_UUID:-}" ]; then uuid="$RESUME_UUID"; else
  uuid=$(python3 -c 'import uuid;print(uuid.uuid4())')
  fi
  if ! tmux has-session -t "$seat" 2>/dev/null; then
    echo "  $seat: no such tmux session — skipped"; return 1
  fi
  case "$seat" in
    *-seatloom) ;;
    *) echo "  $seat: not a -seatloom session — refused"; return 1 ;;
  esac
  if [ "$FORCE" = "0" ] && is_busy "$seat"; then
    echo "  $seat: mid-turn, refusing (use --force to override)"; return 1
  fi

  # (the uuid was decided above: pinned by --resume-uuid, or minted fresh)

  # Each seat runs from ITS OWN worktree, because that is where its .pi/mcp.json
  # lives — pi picks up the project-level MCP config from the cwd. Launching from
  # the shared checkout (or as Claude Code, which reads .mcp.json, not .pi/mcp.json)
  # leaves the seat with NO seatloom MCP: it cannot call get_next_dispatch and the
  # queue silently stalls. Observed 2026-10-10 after a reboot recreated the seat
  # sessions as Claude Code — every seat reported "No MCP servers configured".
  local wt seatname
  seatname=$(printf '%s' "${seat%%-*}" | tr '[:upper:]' '[:lower:]')
  wt="$WORKTREES/$seatname"
  if [ ! -d "$wt" ]; then
    echo "  $seat: no worktree at $wt — falling back to $REPO (MCP may be absent)"
    wt="$REPO"
  elif [ ! -f "$wt/.pi/mcp.json" ]; then
    echo "  $seat: WARNING $wt has no .pi/mcp.json — seat will have no seatloom MCP"
  fi

  # Stop whatever is in the pane, leaving the shell and tmux session intact.
  # Kill ALL children, not just the first: a pane can be occupied by a different
  # agent entirely. Observed 2026-10-10 — a reboot left Onyx running
  # `gemini --resume` stuck in an OAuth timeout, `head -1` picked the wrong pid, and
  # every relaunch typed the command into Gemini's TUI instead of a shell. Onyx was
  # the only seat with queued work, so the entire pipeline sat blocked on it.
  local pane child
  pane=$(tmux display-message -p -t "$seat" '#{pane_pid}')
  for child in $(pgrep -P "$pane" 2>/dev/null); do
    local what
    what=$(ps -o command= -p "$child" 2>/dev/null | cut -c1-40)
    echo "  $seat: stopping pid $child ($what)"
    kill "$child" 2>/dev/null
  done
  sleep 3
  # Anything that ignored SIGTERM gets SIGKILL — a wedged OAuth prompt will.
  for child in $(pgrep -P "$pane" 2>/dev/null); do kill -9 "$child" 2>/dev/null; done
  sleep 1

  tmux resize-window -t "$seat" -x 200 -y 50 2>/dev/null
  # Bail out of any leftover TUI prompt (pi trust dialog, a half-finished credential
  # entry) before typing a shell command, or the command lands in that prompt's text
  # field instead of the shell. Observed 2026-10-10: a reboot left one seat sitting in
  # pi's "Enter API Key" dialog.
  tmux send-keys -t "$seat" Escape 2>/dev/null; sleep 0.3
  tmux send-keys -t "$seat" C-c 2>/dev/null; sleep 0.5
  tmux send-keys -t "$seat" C-u 2>/dev/null; sleep 0.3
  # --approve trusts project-local files for the run, which is what lets pi load this
  # worktree's .pi/mcp.json. Without it pi stops on an interactive "Trust project
  # folder?" dialog and the seat never reaches the MCP — i.e. the queue stalls.
  # 模型按席位取,不用全队写死的那一个(见顶部 model_for 的注释)
  mline="$(model_for "$seat")"
  seat_model="${mline%%$'\t'*}"
  model_src="${mline#*$'\t'}"

  tmux send-keys -t "$seat" \
    "cd $wt && $PI --approve --model \"$seat_model\" --session-id $uuid" Enter

  printf '%s\t%s\t%s\t%s\n' "$seat" "$uuid" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$seat_model" >> "$MAP"
  echo "  $seat: launched on pi in $wt, session $uuid"
  echo "         模型 $seat_model  (来源: $model_src)"
}

case "${1:---help}" in
  --map)
    column -t -s $'\t' "$MAP" 2>/dev/null || cat "$MAP"
    ;;
  --all)
    shift; [ "${1:-}" = "--force" ] && FORCE=1
    echo "relaunching all seats on $MODEL"
    tmux list-sessions -F '#{session_name}' 2>/dev/null | grep -- '-seatloom$' | sort \
      | while read -r s; do launch_seat "$s"; done
    ;;
  -h|--help)
    sed -n '2,18p' "$0"
    ;;
  --resume-uuid)
    shift; RESUME_UUID="${1:?--resume-uuid needs a uuid}"
    shift
    if [ "${1:-}" = "--force" ]; then FORCE=1; shift; fi
    launch_seat "${1:?usage: seat-launch.sh --resume-uuid <uuid> <seat-session>}"
    ;;
  *)
    [ "${2:-}" = "--force" ] && FORCE=1
    launch_seat "$1"
    ;;
esac
