#!/usr/bin/env bash
# seat-status.sh — observe SeatLoom seat progress without attaching.
#
# Read-only. Never sends keys, never kills, never touches a transcript.
# Safe to run at any time, including while a seat is mid-turn.
#
# Usage:
#   scripts/seat-status.sh              one-shot table
#   scripts/seat-status.sh -w [SECS]    refresh every SECS (default 10)
#   scripts/seat-status.sh -v           also show each seat's last tool line
#
# Observation channels, in order of fidelity:
#   1. ~/.claude/projects/<slug>/*.jsonl  structured, complete, written by the harness
#   2. tmux capture-pane                  live state, no history
#   3. docs/coordination/                 durable decisions (file-first protocol)
#
# Channel 1 has no built-in seat mapping: transcripts are named by session UUID and
# every seat shares one project directory. Until seats are launched with an explicit
# --session-id (see scripts/seat-launch.sh), this script attributes a transcript to a
# seat by matching the seat's first prompt text, and falls back to mtime ordering.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SLUG="$(printf '%s' "$REPO" | sed 's#/#-#g')"
TDIR="$HOME/.claude/projects/$SLUG"
MAP="$REPO/.seatloom/seat-sessions.tsv"

WATCH=0; INTERVAL=10; VERBOSE=0
while [ $# -gt 0 ]; do
  case "$1" in
    -w|--watch) WATCH=1; [[ "${2:-}" =~ ^[0-9]+$ ]] && { INTERVAL="$2"; shift; } ;;
    -v|--verbose) VERBOSE=1 ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
  esac
  shift
done

c_dim=$'\033[2m'; c_red=$'\033[31m'; c_grn=$'\033[32m'; c_yel=$'\033[33m'
c_cyn=$'\033[36m'; c_bld=$'\033[1m'; c_off=$'\033[0m'
[ -t 1 ] || { c_dim=; c_red=; c_grn=; c_yel=; c_cyn=; c_bld=; c_off=; }

seat_sessions() { tmux list-sessions -F '#{session_name}' 2>/dev/null | grep -- '-seatloom$' | sort; }

# Resolve a seat's transcript via the launch map, else by first-prompt match.
resolve_transcript() {
  local seat="$1" f
  if [ -f "$MAP" ]; then
    local uuid; uuid=$(awk -F'\t' -v s="$seat" '$1==s{print $2}' "$MAP" | tail -1)
    [ -n "$uuid" ] && [ -f "$TDIR/$uuid.jsonl" ] && { printf '%s' "$TDIR/$uuid.jsonl"; return; }
  fi
  # Fallback: a seat's opening dispatch names the seat. Search recent transcripts.
  # Plain -i match, no \b: BSD grep -E does not support word boundaries.
  local short="${seat%%-*}"
  for f in $(ls -t "$TDIR"/*.jsonl 2>/dev/null | head -20); do
    if grep -qi "you are $short" "$f" 2>/dev/null; then
      printf '%s' "$f"; return
    fi
  done
}

transcript_summary() {
  local f="$1"
  [ -z "$f" ] || [ ! -f "$f" ] && { printf '%s' "-"; return; }
  python3 - "$f" <<'PY' 2>/dev/null || printf '%s' "-"
import sys,json,os,datetime
p=sys.argv[1]; u=a=t=0; last=None
with open(p,encoding='utf-8',errors='ignore') as fh:
    for line in fh:
        try: d=json.loads(line)
        except Exception: continue
        ty=d.get('type')
        if ty=='user': u+=1
        elif ty=='assistant': a+=1
        if d.get('timestamp'): last=d['timestamp']
        if ty=='assistant':
            msg=d.get('message') or {}
            for b in (msg.get('content') or []):
                if isinstance(b,dict) and b.get('type')=='tool_use': t+=1
age=''
if last:
    try:
        dt=datetime.datetime.fromisoformat(last.replace('Z','+00:00'))
        s=int((datetime.datetime.now(datetime.timezone.utc)-dt).total_seconds())
        age=f"{s}s" if s<90 else (f"{s//60}m" if s<5400 else f"{s//3600}h")
    except Exception: pass
print(f"{u}u/{a}a/{t}tool {os.path.getsize(p)//1024}KB last:{age or '?'}")
PY
}

render() {
  printf '%s%-28s %-22s %-9s %s%s\n' "$c_bld" "SEAT" "STATE" "CTX" "TRANSCRIPT (u/a/tool)" "$c_off"
  printf '%s%s%s\n' "$c_dim" "$(printf '%.0s─' {1..104})" "$c_off"

  local any=0
  while read -r s; do
    [ -z "$s" ] && continue
    any=1
    local pane pid child cmd pane_txt state color ctx spin tfile tsum
    pane=$(tmux display-message -p -t "$s" '#{pane_pid}' 2>/dev/null)
    child=$(pgrep -P "${pane:-0}" 2>/dev/null | head -1)
    cmd=$(tmux display-message -p -t "$s" '#{pane_current_command}' 2>/dev/null)
    pane_txt=$(tmux capture-pane -p -t "$s" 2>/dev/null)

    if [ -z "$child" ] || [ "$cmd" = "zsh" ] || [ "$cmd" = "bash" ]; then
      state="NO AGENT"; color="$c_red"
    else
      spin=$(printf '%s' "$pane_txt" | grep -oE '^[^ ]? ?[A-Z][a-z]+(ing|ed)… \([0-9]+[ms][^)]*\)' | tail -1)
      if [ -n "$spin" ]; then
        state="BUSY $(printf '%s' "$spin" | grep -oE '\([0-9]+[ms][^·)]*' | tr -d '(' | sed 's/ *$//')"
        color="$c_yel"
      else
        state="idle"; color="$c_grn"
      fi
    fi

    ctx=$(printf '%s' "$pane_txt" | grep -oE '[0-9]+k?/[0-9]+[Mk]' | tail -1)
    tfile=$(resolve_transcript "$s")
    tsum=$(transcript_summary "$tfile")

    printf '%-28s %s%-22s%s %-9s %s\n' "$s" "$color" "$state" "$c_off" "${ctx:--}" "$tsum"

    if [ "$VERBOSE" = "1" ] && [ -n "$child" ]; then
      printf '%s' "$pane_txt" | grep -E '^\s*(⎿|●)' | tail -2 | cut -c1-100 \
        | sed "s/^/    ${c_dim}/;s/\$/${c_off}/"
    fi
  done < <(seat_sessions)

  [ "$any" = "0" ] && printf '%sno *-seatloom tmux sessions found%s\n' "$c_red" "$c_off"

  local recent
  recent=$(find "$REPO/docs/coordination" -type f -newermt '-30 minutes' 2>/dev/null \
           | sed "s#$REPO/##" | head -8)
  if [ -n "$recent" ]; then
    printf '\n%scoordination artifacts touched in the last 30 min%s\n' "$c_cyn" "$c_off"
    printf '%s\n' "$recent" | sed 's/^/  /'
  fi
}

if [ "$WATCH" = "1" ]; then
  while true; do clear; printf '%s  (refresh %ss, ctrl-c to stop)\n\n' "$(date '+%H:%M:%S')" "$INTERVAL"; render; sleep "$INTERVAL"; done
else
  render
fi
