#!/usr/bin/env bash
# seat-dispatch.sh — reliable dispatch to a seat's Claude Code session.
#
# send-keys of a long literal string truncates (observed: seats received partial
# messages). Instead, write the message to a file and tell the seat to read it —
# the payload never rides the tmux input line, so nothing is lost.
#
# Usage: seat-dispatch.sh <tmux-session> <message-file>
#        (message-file is any path the seat can Read)
set -euo pipefail
seat="$1"; msgfile="$2"
[ -f "$msgfile" ] || { echo "no such message file: $msgfile" >&2; exit 1; }
tmux has-session -t "$seat" 2>/dev/null || { echo "no such seat: $seat" >&2; exit 1; }

abs="$(cd "$(dirname "$msgfile")" && pwd)/$(basename "$msgfile")"
# Clear any pending text in the input line, then send a short read instruction.
tmux send-keys -t "$seat" C-u 2>/dev/null || true
sleep 0.3
tmux send-keys -t "$seat" -l "AEGIS dispatch — Read the file $abs in full and act on it."
sleep 0.3
tmux send-keys -t "$seat" Enter
echo "dispatched to $seat -> $abs"
