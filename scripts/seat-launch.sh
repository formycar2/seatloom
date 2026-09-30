#!/usr/bin/env bash
# seat-launch.sh — (re)launch a SeatLoom seat with a pinned session ID.
#
# Why this exists: every seat writes its structured transcript to
# ~/.claude/projects/<repo-slug>/<session-uuid>.jsonl, but the filename carries no
# seat identity and all seats share one project directory. Passing --session-id at
# launch makes the seat→transcript mapping exact instead of guessed, which is what
# scripts/seat-status.sh and the future ingestion layer both need.
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
MODEL="${SEATLOOM_MODEL:-claude-opus-5[1m]}"
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

  # Stop the current agent, leave the shell and the tmux session intact.
  local pane child
  pane=$(tmux display-message -p -t "$seat" '#{pane_pid}')
  child=$(pgrep -P "$pane" 2>/dev/null | head -1)
  [ -n "$child" ] && { kill "$child" 2>/dev/null; sleep 3; }

  tmux resize-window -t "$seat" -x 200 -y 50 2>/dev/null
  tmux send-keys -t "$seat" \
    "cd $REPO && stepcode claude --model \"$MODEL\" --session-id $uuid" Enter

  printf '%s\t%s\t%s\t%s\n' "$seat" "$uuid" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$MODEL" >> "$MAP"
  echo "  $seat: launched, session $uuid"
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
