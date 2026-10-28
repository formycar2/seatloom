#!/usr/bin/env bash
# Poke every live seat with a short message. The session names are NOT the
# obvious ones and are not derivable from the seat id.
#
# Why this exists: on 2026-10-28 I poked 'Flux-QA/seatloom', 'Lyra-PO/seatloom',
# 'Nimbus-Backend/seatloom', 'Onyx-Data/seatloom' and 'Vela-Wash/seatloom'.
# Every one of them is the wrong name. tmux send-keys returns exit 0 for a
# session that does not exist in some invocations and non-zero in others, so the
# failure mode is silent: it looks like nine seats were poked and in fact zero
# were. The real names carry the seat's role, not its discipline:
#
#   Lyra-po-seatloom            (not Lyra-PO)
#   Mira-UX/UED-seatloom        (the only one that matches the seat id)
#   Nimbus-TechArchi-seatloom   (not Nimbus-Backend)
#   Flux-Quality&Ops-seatloom   (not Flux-QA)   <- contains a bare &
#   Onyx-data-seatloom          (not Onyx-Data)
#   Vela-DataWash-seatloom      (not Vela-Wash)
#
# Two consequences this script enforces:
#   1. session names are read from the live tmux server, never hardcoded here.
#      The authoritative list is scripts/restart-seats-for-new-binary.sh.
#   2. the message is verified to have LANDED by reading the pane back. send-keys
#      succeeding is not evidence the seat saw it.
#
# Usage: scripts/poke-all-seats.sh "<message>"
#        scripts/poke-all-seats.sh --list     # print the live seat sessions

set -u

# NOTE: deliberately no `pipefail`. `grep -q` exits on first match and closes
# the pipe, which sends SIGPIPE to `tmux capture-pane`; with pipefail that turns a
# successful read-back into a failure and this script reports every seat as
# UNVERIFIED while the message is plainly sitting in the pane. This was the actual
# bug on the first run of this script.

cd "$(dirname "$0")/.." || exit 2

TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux /usr/bin/tmux; do
  [ -n "$c" ] && [ -x "$c" ] && { TMUX_BIN="$c"; break; }
done
if [ -z "$TMUX_BIN" ]; then
  echo "FAIL: tmux not found (PATH=$PATH)" >&2
  exit 2
fi

sessions=$("$TMUX_BIN" list-sessions -F '#{session_name}' 2>/dev/null | grep -- '-seatloom$')
if [ -z "$sessions" ]; then
  echo "FAIL: no live seat sessions (nothing ending in -seatloom)" >&2
  exit 2
fi

if [ "${1:-}" = "--list" ]; then
  echo "$sessions"
  exit 0
fi

MSG="${1:-}"
if [ -z "$MSG" ]; then
  echo "usage: $0 \"<message>\" | --list" >&2
  exit 2
fi

# A marker that is short and unlikely to appear in a seat's own output. The pane
# is read back for this string; without it a send-keys success is indistinguishable
# from a message that never arrived.
MARKER="POKE-$(date +%s)"
BODY="$MSG  [$MARKER]"

poked=0
failed=0

while IFS= read -r s; do
  [ -n "$s" ] || continue
  # A bare & in the session name is fine for tmux (it is one argument) but would
  # be a shell operator if this string were ever interpolated unquoted.
  if "$TMUX_BIN" send-keys -t "$s:0" -l "$BODY" 2>/dev/null && \
     "$TMUX_BIN" send-keys -t "$s:0" Enter 2>/dev/null; then
    :
  else
    echo "  FAIL send-keys: $s"
    failed=$((failed + 1))
    continue
  fi

  # Read the pane back. The seat may have already consumed the line into its
  # prompt, in which case the raw text is still in the last few scrollback rows.
  #
  # The read is RETRIED, not single-shot: on the first run of this script every
  # busy seat read as UNVERIFIED. Measured, not guessed — a marker sent to a
  # working pane read back as absent at t=0 and present at t=3s, because the
  # agent's TUI owns the pane and redraws on its own schedule. A single-shot
  # read-back is a check that fails when the seat is working, which is exactly
  # when it matters.
  seen=0
  for _ in 1 2 3 4 5 6; do
    if "$TMUX_BIN" capture-pane -p -t "$s:0" -S -15 2>/dev/null | grep -q "$MARKER"; then
      seen=1
      break
    fi
    sleep 1
  done

  if [ "$seen" = "1" ]; then
    printf '  OK   %s\n' "$s"
    poked=$((poked + 1))
  else
    printf '  UNVERIFIED %s — send-keys returned 0 but the text is not in the pane\n' "$s"
    failed=$((failed + 1))
  fi
done <<< "$sessions"

echo "poked=$poked failed=$failed"
[ "$failed" -eq 0 ] || exit 1
exit 0
