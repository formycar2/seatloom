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
# Refuses to relaunch a seat that is mid-turn unless --force is given: killing a
# working agent loses the turn. Never touches non-seatloom tmux sessions.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL="${SEATLOOM_MODEL:-stepplan-anthropic/step-5-preview}"
PI="${SEATLOOM_PI_BIN:-$HOME/.local/bin/pi}"
WORKTREES="${SEATLOOM_WORKTREES:-$REPO/../seatloom-worktrees}"
MAP="$REPO/.seatloom/seat-sessions.tsv"
FORCE=0

mkdir -p "$(dirname "$MAP")"
[ -f "$MAP" ] || printf 'seat\tsession_id\tlaunched_at\tmodel\n' > "$MAP"

is_busy() {
  tmux capture-pane -p -t "$1" 2>/dev/null \
    | grep -qE '^[^ ]? ?[A-Z][a-z]+(ing|ed)… \([0-9]+[ms]'
}

launch_seat() {
  local seat="$1" uuid
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

  uuid=$(python3 -c 'import uuid;print(uuid.uuid4())')

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

  # Stop the current agent, leave the shell and the tmux session intact.
  local pane child
  pane=$(tmux display-message -p -t "$seat" '#{pane_pid}')
  child=$(pgrep -P "$pane" 2>/dev/null | head -1)
  [ -n "$child" ] && { kill "$child" 2>/dev/null; sleep 3; }

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
  tmux send-keys -t "$seat" \
    "cd $wt && $PI --approve --model \"$MODEL\" --session-id $uuid" Enter

  printf '%s\t%s\t%s\t%s\n' "$seat" "$uuid" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$MODEL" >> "$MAP"
  echo "  $seat: launched on pi in $wt, session $uuid"
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
  *)
    [ "${2:-}" = "--force" ] && FORCE=1
    launch_seat "$1"
    ;;
esac
