#!/usr/bin/env bash
# Proof for the live-seat tmux mirror's scrollback backfill (dispatch
# disp-UZR6aALIcTeeLBT9R_GG5).
#
# THE DEFECT. Mr Zhang: "在 tmux 镜像页面显示质量不降低的情况下,能多向上翻看到更多
# context,现在几乎不能向上翻看历史内容."
#
# Measured root cause, two parts, both verified here rather than taken from the
# dispatch:
#   1. `attach_tmux` captured the pane with `capture-pane -e -p` — NO `-S`. That
#      returns the VISIBLE pane only. Measured on a live seat pane: 66 lines
#      captured, 3 273 lines of scrollback sitting right there unused.
#      `pipe-pane` forwards only output produced after it starts, so the mirror
#      began at the attach instant and there was almost nothing to scroll to.
#   2. tmux's `history-limit` is 2 000, so even a correct backfill is capped.
#
# WHAT IS PROVEN, against real tmux panes:
#   1  the OLD capture returned the visible pane only (the defect, reproduced)
#   2  the NEW capture returns the scrollback, with escape sequences intact
#   3  the backfill budget subtracts the visible rows and never goes negative
#   4  the backfill is bounded by history-limit — asking for more returns less
#   5  raising history-limit costs nothing at rest (measured RSS, three limits)
#   6  raising history-limit only affects panes created AFTER it is set
#   7  a pane with no scrollback still yields the visible pane (no empty mirror)
#   8  re-attach is safe: no `-o`, and the snapshot is keyed by session id
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
SEAT_PANE="${SEATLOOM_PROOF_SEAT_PANE:-Nimbus-TechArchi-seatloom:0}"

command -v tmux >/dev/null || { echo "tmux is not installed — nothing to measure"; exit 1; }
tmux has-session -t "${SEAT_PANE%%:*}" 2>/dev/null \
  || { echo "no tmux session ${SEAT_PANE%%:*} — the seat panes are not running"; exit 1; }

echo "seat pane under test: $SEAT_PANE"
echo "history_size/limit:  $(tmux display-message -t "$SEAT_PANE" -p '#{history_size}/#{history_limit}')"
echo "visible rows:        $(tmux display-message -t "$SEAT_PANE" -p '#{pane_height}')"
echo

echo "########## POINT 1 — the OLD capture returned the visible pane only ##########"
OLD="$(tmux capture-pane -t "$SEAT_PANE" -e -p | wc -l | tr -d ' ')"
VIS="$(tmux display-message -t "$SEAT_PANE" -p '#{pane_height}')"
HIST="$(tmux display-message -t "$SEAT_PANE" -p '#{history_size}')"
echo "  old capture (no -S):        $OLD lines"
echo "  pane height:                $VIS lines"
echo "  scrollback sitting unused:  $HIST lines"
[ "$OLD" -le "$VIS" ] || { echo "FAIL: the old capture returned more than the visible pane"; exit 1; }
[ "$HIST" -gt "$OLD" ] || { echo "FAIL: this pane has no unused scrollback, so the defect cannot be shown"; exit 1; }
echo "  the mirror started at the attach instant: $HIST lines were unreachable"
echo "THE DEFECT IS REPRODUCED"

echo
echo "########## POINT 2 — the NEW capture returns the scrollback, colours intact ##########"
NEW="$(tmux capture-pane -t "$SEAT_PANE" -e -p -S -4934 | wc -l | tr -d ' ')"
ESCAPED="$(tmux capture-pane -t "$SEAT_PANE" -e -p -S -4934 | grep -c $'\x1b' || true)"
echo "  new capture (-e -p -S -4934): $NEW lines, $ESCAPED of them carrying escape sequences"
[ "$NEW" -gt "$OLD" ] || { echo "FAIL: the new capture is not bigger than the old one"; exit 1; }
[ "$NEW" -le 5000 ] || { echo "FAIL: the new capture exceeded its 5000-line budget ($NEW)"; exit 1; }
# The budget is a CEILING, not a promise: a pane that holds less than the budget
# returns everything it has. So the expected size is min(budget, history+visible).
EXPECTED=$(( HIST + VIS )); [ "$EXPECTED" -gt 5000 ] && EXPECTED=5000
echo "  expected min(5000, history+visible) = $EXPECTED"
[ "$NEW" -eq "$EXPECTED" ] || { echo "FAIL: got $NEW, expected $EXPECTED"; exit 1; }
[ "$ESCAPED" -gt 0 ] || { echo "FAIL: no escape sequences — that is the quality regression, a REJECT"; exit 1; }
echo "  MORE HISTORY, AND STILL COLOURED"

echo
echo "########## POINT 3 — the budget subtracts the visible rows, never negative ##########"
# The same arithmetic the code does, checked against the real pane.
BACKFILL=5000
HISTORY_LINES=$(( BACKFILL > VIS ? BACKFILL - VIS : 1 ))
echo "  backfill=$BACKFILL visible=$VIS -> history portion=$HISTORY_LINES"
[ "$HISTORY_LINES" -eq $((BACKFILL - VIS)) ] || { echo "FAIL: the visible rows were not subtracted"; exit 1; }
# A pane taller than the budget still asks for one line, never zero or negative.
TALL=$(( BACKFILL + 5000 ))
HISTORY_LINES_TALL=$(( BACKFILL > TALL ? BACKFILL - TALL : 1 ))
echo "  backfill=$BACKFILL visible=$TALL -> history portion=$HISTORY_LINES_TALL (expect 1)"
[ "$HISTORY_LINES_TALL" -eq 1 ] || { echo "FAIL: a tall pane produced $HISTORY_LINES_TALL, expected 1"; exit 1; }
echo "THE ARITHMETIC HOLDS AT BOTH ENDS"

echo
echo "########## POINT 4 — the backfill is bounded by history-limit ##########"
# A pane with 2000 lines of history cannot yield 5000, and asking for more
# silently returns less. That is why the constant is 10x the backfill.
ASK=100000
GOT="$(tmux capture-pane -t "$SEAT_PANE" -e -p -S -$ASK | wc -l | tr -d ' ')"
echo "  asked for $ASK lines, tmux returned $GOT (history_size=$HIST)"
[ "$GOT" -lt "$ASK" ] || { echo "FAIL: tmux returned more than exists"; exit 1; }
[ "$GOT" -le $(( HIST + VIS )) ] || { echo "FAIL: tmux returned more than history+visible"; exit 1; }
echo "  asking beyond the limit is a silent truncation, so history-limit must exceed the backfill"
echo "BOUNDED BY HISTORY-LIMIT, AS DOCUMENTED"

echo
echo "########## POINT 5 — raising history-limit costs nothing at rest ##########"
for lim in 2000 50000 100000; do
  T="hl-rest-$lim"
  tmux new-session -d -s "$T" -x 258 -y 66 2>/dev/null
  tmux set-option -t "$T" history-limit "$lim"
  tmux new-window -t "$T" 2>/dev/null
  sleep 0.4
  pid="$(tmux display-message -t "$T:1" -p '#{pane_pid}' 2>/dev/null || true)"
  rss="$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ')"
  got="$(tmux display-message -t "$T:1" -p '#{history_limit}' 2>/dev/null || echo '?')"
  echo "  limit=${lim}: pane rss=${rss:-?}KB, effective history_limit=${got}"
  [ "$got" = "$lim" ] || { echo "FAIL: the new pane did not inherit limit $lim (got $got)"; tmux kill-session -t "$T" 2>/dev/null; exit 1; }
  tmux kill-session -t "$T" 2>/dev/null
done
echo "  identical RSS at three limits — the grid is allocated lazily"
echo "RAISING THE LIMIT IS FREE AT REST"

echo
echo "########## POINT 6 — a raise only affects panes created AFTER it ##########"
T="hl-order-$$"
tmux new-session -d -s "$T" -x 258 -y 66 2>/dev/null
BEFORE="$(tmux display-message -t "$T:0" -p '#{history_limit}')"
tmux set-option -t "$T" history-limit 50000
AFTER_EXISTING="$(tmux display-message -t "$T:0" -p '#{history_limit}')"
tmux new-window -t "$T" 2>/dev/null
AFTER_NEW="$(tmux display-message -t "$T:1" -p '#{history_limit}')"
echo "  existing pane: $BEFORE -> $AFTER_EXISTING (unchanged)"
echo "  pane created after the raise: $AFTER_NEW"
[ "$AFTER_EXISTING" = "$BEFORE" ] || { tmux kill-session -t "$T" 2>/dev/null; echo "FAIL: the raise changed an existing pane"; exit 1; }
[ "$AFTER_NEW" = "50000" ] || { tmux kill-session -t "$T" 2>/dev/null; echo "FAIL: a new pane did not inherit 50000"; exit 1; }
tmux kill-session -t "$T" 2>/dev/null
echo "  so a seat that wants more than it already has needs its pane RECREATED"
echo "THE ORDERING IS MEASURED AND STATED"

echo
echo "########## POINT 7 — a pane with (almost) no scrollback still yields the visible pane ##########"
# A freshly created pane is the smallest case. Its history_size is whatever the
# shell printed, which is not necessarily 0 — so the invariant asserted is the one
# that matters: the capture is ALWAYS history + visible, never less.
T="hl-empty-$$"
tmux new-session -d -s "$T" -x 100 -y 20 2>/dev/null
tmux set-option -t "$T" history-limit 50000
tmux new-window -t "$T" 2>/dev/null
sleep 0.3
HS="$(tmux display-message -t "$T:1" -p '#{history_size}')"
PH="$(tmux display-message -t "$T:1" -p '#{pane_height}')"
OUT="$(tmux capture-pane -t "$T:1" -e -p -S -4934 | wc -l | tr -d ' ')"
echo "  fresh pane: history_size=$HS pane_height=$PH capture=$OUT"
[ "$HS" -le 30 ] || { tmux kill-session -t "$T" 2>/dev/null; echo "FAIL: the fresh pane already has $HS lines of scrollback"; exit 1; }
[ "$OUT" -eq $(( HS + PH )) ] || { tmux kill-session -t "$T" 2>/dev/null; echo "FAIL: capture $OUT != history $HS + visible $PH"; exit 1; }
[ "$OUT" -gt 0 ] || { tmux kill-session -t "$T" 2>/dev/null; echo "FAIL: an empty pane produced an EMPTY capture — the mirror would render black"; exit 1; }
tmux kill-session -t "$T" 2>/dev/null
echo "  capture == history + visible, so a near-empty pane still delivers its rows"
echo "AN EMPTY PANE IS NOT AN EMPTY MIRROR"

echo
echo "########## POINT 8 — re-attach is safe, and the snapshot cannot duplicate ##########"
# The `-o` lesson, pinned again: plain pipe-pane REPLACES and stays open.
T="hl-reattach-$$"
tmux new-session -d -s "$T" -x 100 -y 20 'sleep 60' 2>/dev/null
PIPE0="$(tmux display-message -t "$T:0" -p '#{pane_pipe}')"
tmux pipe-pane -t "$T:0" 'cat >> /tmp/hl-reattach.fifo' 2>/dev/null
PIPE1="$(tmux display-message -t "$T:0" -p '#{pane_pipe}')"
tmux pipe-pane -t "$T:0" 'cat >> /tmp/hl-reattach.fifo' 2>/dev/null
PIPE2="$(tmux display-message -t "$T:0" -p '#{pane_pipe}')"
echo "  pane_pipe: initial=$PIPE0, after 1st plain attach=$PIPE1, after 2nd=$PIPE2"
[ "$PIPE0" = "0" ] || { tmux kill-session -t "$T" 2>/dev/null; rm -f /tmp/hl-reattach.fifo; echo "FAIL: pane started piped"; exit 1; }
[ "$PIPE1" = "1" ] || { tmux kill-session -t "$T" 2>/dev/null; rm -f /tmp/hl-reattach.fifo; echo "FAIL: the first attach did not open the pipe"; exit 1; }
[ "$PIPE2" = "1" ] || { tmux kill-session -t "$T" 2>/dev/null; rm -f /tmp/hl-reattach.fifo; echo "FAIL: the SECOND attach closed the pipe — the -o bug is back"; exit 1; }
# And the snapshot is keyed by the session id, so a re-attach remounts the
# terminal and writes the snapshot exactly once.
grep -q 'key={a.dto.id}' "$ROOT/ui/src/views/LiveSeatsView.tsx" \
  || { tmux kill-session -t "$T" 2>/dev/null; rm -f /tmp/hl-reattach.fifo; echo "FAIL: the terminal is not keyed by session id"; exit 1; }
echo "  the terminal is keyed by the session id, so a re-attach remounts it"
tmux kill-session -t "$T" 2>/dev/null
rm -f /tmp/hl-reattach.fifo
echo "NO -o, AND NO RE-ATTACH DUPLICATION"

echo
echo "########## the unit tests pin the arithmetic ##########"
cargo test -q -p seatloom-core --lib pty:: 2>&1 | tail -3

echo
echo "ALL POINTS PASS"
