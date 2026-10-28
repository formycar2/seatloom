#!/usr/bin/env bash
# Poke seats with a short message, verified by reading the pane back.
#
# Two things this script exists to stop, both of which happened on 2026-10-28:
#
# 1. WRONG SESSION NAMES. I poked 'Flux-QA', 'Lyra-PO', 'Nimbus-Backend',
#    'Onyx-Data' and 'Vela-Wash'. All five are wrong; tmux send-keys to a
#    non-existent session looks like success. The real names are
#    `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`, `Nimbus-TechArchi-seatloom`,
#    `Flux-Quality&Ops-seatloom` (contains a bare &), `Onyx-data-seatloom`,
#    `Vela-DataWash-seatloom` - the pattern is <seat>-<ROLE>-seatloom, and the
#    role is a job title, not a discipline. So the names are READ FROM THE LIVE
#    SERVER here, never hardcoded.
#
# 2. BROADCASTING TO ADOPTED EXTERNAL PROJECTS. I sent an urgent SeatLoom
#    branch-sync message to all nine -seatloom sessions. Three of them are
#    adopted external projects (QL, networkOrchestrator, stepaicloud) that have
#    nothing to do with track/infra-foundation and had zero pending acceptance
#    cards. QL spent a session re-deriving the whole situation to conclude it
#    should do nothing - the right answer, reached at the cost of its context.
#
# So the default scope is SeatLoom seats ONLY. External projects are listed
# explicitly and require --all-seats, which prints what it is about to touch.
#
# Usage:
#   scripts/poke-all-seats.sh --scope          # who counts as a SeatLoom seat
#   scripts/poke-all-seats.sh "<message>"      # SeatLoom seats only (default)
#   scripts/poke-all-seats.sh --all-seats "<message>"
#
# Verification: each message carries a per-run marker and the pane is read back
# for it. send-keys returning 0 is not evidence of delivery.

set -u
# NOTE: no `pipefail`. `grep -q` exits on match and closes the pipe, which
# SIGPIPEs `tmux capture-pane`; with pipefail every successful read-back turns
# into a failure and this script reports all seats UNVERIFIED while the message
# is plainly in the pane. That was the first bug found testing this script.

cd "$(dirname "$0")/.." || exit 2

TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux /usr/bin/tmux; do
  if [ -n "$c" ] && [ -x "$c" ]; then TMUX_BIN="$c"; break; fi
done 2>/dev/null
if [ -z "$TMUX_BIN" ]; then
  echo "FAIL: tmux not found (PATH=$PATH)" >&2
  exit 2
fi

db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "$1" 2>/dev/null
}

sessions=$("$TMUX_BIN" list-sessions -F '#{session_name}' 2>/dev/null | grep -- '-seatloom$')
if [ -z "$sessions" ]; then
  echo "FAIL: no live seat sessions (nothing ending in -seatloom)" >&2
  exit 2
fi

# The adopted external projects are the projects bound in project_role_bindings
# whose id is not seatloom. Ask the ledger, because "which projects are ours" is
# exactly the thing that changes - QL, networkOrchestrator and stepaicloud were
# all adopted during this project's life.
#
# The session name is NOT the project name and not even the same case:
#   project_delivery/networkOrchestrator  ->  session NetworkOrchestrator-seatloom
#   project_delivery/stepaicloud           ->  session Stepaicloud-seatloom
#   project_delivery/QL                    ->  session QL-seatloom
# So compare the lowercased leading segment of the session name against the
# lowercased last path segment of the project id. (bash 3.2 has no ${v,,}; use
# tr.)
external_projects=$(db "select project_id from project_role_bindings where project_id <> 'seatloom' order by 1")

lower() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]'; }

# A session belongs to an external project when its leading segment matches an
# external project's last id segment. SeatLoom's own seats carry ROLE names
# (po, UX/UED, TechArchi, Quality&Ops, data, DataWash) that match no project
# segment, so this does not misfire on Flux-Quality&Ops-seatloom.
is_external() {
  s_name="$1"
  seg=$(lower "${s_name%%-*}")
  [ -n "$seg" ] || return 1
  [ -n "$external_projects" ] || return 1
  while IFS= read -r proj; do
    [ -n "$proj" ] || continue
    base=$(lower "${proj##*/}")
    if [ "$seg" = "$base" ]; then return 0; fi
  done <<< "$external_projects"
  return 1
}

seatloom_sessions=""
external_sessions=""
while IFS= read -r s; do
  [ -n "$s" ] || continue
  if is_external "$s"; then
    external_sessions="$external_sessions$s\n"
  else
    seatloom_sessions="$seatloom_sessions$s\n"
  fi
done <<< "$sessions"

if [ "${1:-}" = "--scope" ]; then
  echo "SeatLoom seats (default scope):"
  printf "$seatloom_sessions" | sed '/^$/d'
  echo
  echo "Adopted external projects (NOT SeatLoom seats; --all-seats required):"
  printf "$external_sessions" | sed '/^$/d'
  exit 0
fi

ALL=0
TARGET=""
case "${1:-}" in
  --scope|--list) echo "$sessions"; exit 0 ;;
  --all-seats) ALL=1; shift ;;
  --to)
    # Address ONE seat. A seat-specific instruction broadcast to everyone is
    # noise for five seats and, when the content is false for them, a lie.
    TARGET="${2:-}"
    shift 2 || true
    ;;
esac
MSG="${1:-}"
if [ -z "$MSG" ]; then
  echo "usage: $0 --scope | \"<message>\" | --all-seats \"<message>\"" >&2
  exit 2
fi

if [ -n "$TARGET" ]; then
  # Exact session name, no guessing. If it is not live, say so rather than
  # silently poking nobody - that is the failure this script was written for.
  if ! printf '%s\n' "$sessions" | grep -qxF "$TARGET"; then
    echo "FAIL: session '$TARGET' is not live. Live sessions:" >&2
    echo "$sessions" >&2
    exit 2
  fi
  targets="$TARGET"
elif [ "$ALL" = "1" ]; then
  targets="$sessions"
  echo "WARNING: broadcasting to SeatLoom seats AND adopted external projects." >&2
else
  targets="$seatloom_sessions"
  if [ -z "$(printf '%s' "$targets" | tr -d '\n')" ]; then
    echo "FAIL: no SeatLoom seat sessions found; refusing to fall back to everyone" >&2
    exit 2
  fi
fi

# The verification token is a PREFIX fragment, not a suffix marker.
#
# On 2026-10-29 02:1x a poke to a working seat reported UNVERIFIED while the pane
# showed the message plainly as "Steering: # 紧急:优先级 99 ... 我提..." - the TUI
# had TRUNCATED the line, and the truncation cut off exactly the "[POKE-...]" at
# the end. A suffix marker is invisible to any message long enough to wrap.
#
# So: send the token at the START, where it survives, and search for the token
# alone. The message itself stays after it, so the seat still reads plain text.
MARKER="POKE-$(date +%s)"
BODY="[$MARKER] $MSG"

poked=0
failed=0

while IFS= read -r s; do
  [ -n "$s" ] || continue
  if "$TMUX_BIN" send-keys -t "$s:0" -l "$BODY" 2>/dev/null && \
     "$TMUX_BIN" send-keys -t "$s:0" Enter 2>/dev/null; then
    :
  else
    echo "  FAIL send-keys: $s"
    failed=$((failed + 1))
    continue
  fi

  # Read the pane back, RETRIED. Single-shot read-back is a check that fails when
  # the seat is working: measured, a marker sent to a busy pane reads back absent
  # at t=0 and present at t=3s, because the agent's TUI owns the pane and redraws
  # on its own schedule. A check that fails exactly when it matters is not a
  # check.
  #
  # Window is 15s x a DEEP scrollback, not 6s x a shallow one.
  #
  # Two separate bugs found by testing this, both of which produced a false
  # UNVERIFIED while the marker was provably in the pane:
  #
  # 1. TOO SHORT a wait. A marker sent to a busy pane reads back absent at t=0
  #    and present at t=3s, because the agent TUI owns the pane and redraws on
  #    its own schedule. A check that fails exactly when the seat is working
  #    fails when it matters.
  #
  # 2. TOO SHALLOW a capture. `capture-pane -S -15` on a 50-row pane returns 65
  #    lines, and a long message wraps to 3-4 of them. If the TUI redraws during
  #    the wait - which is exactly what a working seat does - the message scrolls
  #    out of those 65 lines and is then permanently invisible to this read, no
  #    matter how long we wait. On 2026-10-29 02:10 the marker was visible in the
  #    pane AFTER the script had already given up.
  #
  # A false UNVERIFIED is worse than a slow OK: the caller re-sends, and the seat
  # gets the message twice.
  seen=0
  for _ in $(seq 1 15); do
    if "$TMUX_BIN" capture-pane -p -t "$s:0" -S -400 2>/dev/null | grep -q "$MARKER"; then
      seen=1
      break
    fi
    sleep 1
  done

  if [ "$seen" = "1" ]; then
    printf '  OK   %s\n' "$s"
    poked=$((poked + 1))
  else
    printf '  UNVERIFIED %s — send-keys returned 0 but the text is not in the pane after 15s\n' "$s"
    failed=$((failed + 1))
  fi
done <<< "$(printf '%b' "$targets")"

echo "poked=$poked failed=$failed"
[ "$failed" -eq 0 ] || exit 1
exit 0
