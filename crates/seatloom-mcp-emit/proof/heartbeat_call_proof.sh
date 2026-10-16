#!/usr/bin/env bash
# Proof for the heartbeat call — a seat that only exists when dispatched is not
# doing its job (dispatch disp-Krsp8j-EDOpWDIiWjuHo_).
#
# THE DEFECT. Mr Zhang, 2026-10-16: Lyra is the product owner, with a duty to
# design and to find what is broken and to own the delivery — but her shape is
# "work when dispatched, stop when not". That is not a product owner, that is a
# contractor. He is right, and the root cause is not in her: it is in the waker.
#
# `sweep` matched each seat on `(in_flight_id, ready_n)` and only ever called it
# in two cases — idle WITH ready work, and busy with a stale claim. "Idle with
# zero ready dispatches" was not in the match at all, so such a seat was never
# called; and the only instruction any seat ever received was "drain your
# dispatches and stop", whose entire field of view is the queue.
#
# WHAT IS PROVEN:
#   1  the OLD match never called an idle seat with nothing ready (reproduced)
#   2  the dispatch message is BYTE-IDENTICAL to the previous commit's
#   3  the heartbeat message is a different instruction, pointing at the seat's
#     own document — not the drain instruction reworded
#   4  which real seats get called: the seat map x the documents that exist
#   5  a seat WITHOUT a document is never called, however long it sits idle
#   6  the heartbeat cooldown is separate and longer, and prevents spam
#   7  the two existing paths are untouched
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
MAIN="${SEATLOOM_MAIN_CHECKOUT:-/Users/jyxc-dz-0100609/Documents/GitHub/seatloom}"
MAP="$MAIN/.seatloom/dispatch-seats.tsv"
TRACK_REF="${TRACK_REF:-track/infra-foundation}"

command -v tmux >/dev/null || { echo "tmux is not installed — nothing to measure"; exit 1; }
[ -f "$MAP" ] || { echo "no seat map at $MAP — the waker has nothing mapped"; exit 1; }

# The "before" state this dispatch changed. PINNED, not `HEAD`: this proof was
# written while the change was the tip, and the first time it ran after the commit
# it compared the new source against itself and passed vacuously — a proof whose
# baseline moves with the tip is a proof that can silently stop measuring anything.
# Override with SEATLOOM_WAKER_BEFORE_REF if the pin is ever rebased away.
BEFORE_REF="${SEATLOOM_WAKER_BEFORE_REF:-47079e1^}"
git -C "$ROOT" cat-file -e "${BEFORE_REF}^{commit}" 2>/dev/null \
  || { echo "FAIL: the before-ref $BEFORE_REF is not a commit in this repository"; exit 1; }
git -C "$ROOT" merge-base --is-ancestor "$BEFORE_REF" HEAD \
  || { echo "FAIL: $BEFORE_REF is not an ancestor of HEAD — the baseline is not in this history"; exit 1; }
if [ "$(git -C "$ROOT" rev-parse HEAD)" = "$(git -C "$ROOT" rev-parse "$BEFORE_REF")" ]; then
  echo "FAIL: HEAD IS the before-state — the change under test is not in this tree"
  exit 1
fi
# And the change must actually be present: the third arm exists in HEAD's source.
grep -q "NudgeKind::Heartbeat" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat nudge is not in the working tree"; exit 1; }

echo "seat map:     $MAP"
echo "waker source: $ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs"
echo "before ref:   $BEFORE_REF ($(git -C "$ROOT" log -1 --format=%h "$BEFORE_REF"))"
echo

echo "########## POINT 1 — the OLD match never called an idle seat with nothing ##########"
git -C "$ROOT" show "${BEFORE_REF}:crates/seatloom-mcp-emit/src/bin/waker.rs" > /tmp/waker_head.rs
# The old final arm, verbatim.
grep -A 1 "Idle with nothing ready: nothing to do." /tmp/waker_head.rs | head -2 | sed 's/^/  /'
grep -q "(None, _) => {}" /tmp/waker_head.rs \
  || { echo "FAIL: the old code's idle-with-nothing arm is not where I measured it"; exit 1; }
# And it matched on exactly two call-worthy shapes.
ARMS="$(grep -c "nudge(&session)\|nudge(session)" /tmp/waker_head.rs)"
echo "  nudge call sites in the OLD code: $ARMS"
[ "$ARMS" -ge 2 ] || { echo "FAIL: expected the two existing call sites"; exit 1; }
# A seat that is idle with zero ready dispatches falls into that empty arm.
echo "  (None, 0) fell into (None, _) => {} — never called"
echo "THE OLD CODE COULD NOT CALL AN IDLE SEAT"

echo
echo "########## POINT 2 — the dispatch message is BYTE-IDENTICAL ##########"
# Extract the literal from the OLD source and from the NEW one and compare them.
# A proof that merely re-uses the string would pass even if the string changed.
extract_dispatch_msg() {
  # The literal is a Rust string with `\` line continuations and escaped quotes,
  # so read it as Rust does: from the emoji to the closing unescaped quote, then
  # drop the continuation backslashes and the indentation that follows them.
  python3 - "$1" <<'PY'
import sys
src = open(sys.argv[1], encoding='utf-8').read()
i = src.find('"\U0001f4e5')
if i < 0:
    sys.exit("could not find the dispatch literal")
i += 1  # past the opening quote
out = []
while i < len(src):
    c = src[i]
    if c == '\\':
        # A backslash-newline continuation: the logical string continues on the
        # next line, and the leading whitespace of that line is not part of it.
        if i + 1 < len(src) and src[i + 1] == '\n':
            i += 2
            while i < len(src) and src[i] in ' \t':
                i += 1
            continue
        out.append(src[i:i + 2]); i += 2; continue
    if c == '"':
        break
    out.append(c); i += 1
else:
    sys.exit("unterminated literal")
print(''.join(out))
PY
}
OLD_MSG="$(extract_dispatch_msg /tmp/waker_head.rs)"
NEW_MSG="$(extract_dispatch_msg "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs")"
[ -n "$OLD_MSG" ] || { echo "FAIL: no dispatch literal in the old source"; exit 1; }
[ -n "$NEW_MSG" ] || { echo "FAIL: no dispatch literal in the new source"; exit 1; }
if [ "$OLD_MSG" != "$NEW_MSG" ]; then
  echo "FAIL: the dispatch message CHANGED — the acceptance criterion is that it does not"
  echo "--- old ---"; printf '%s\n' "$OLD_MSG"
  echo "--- new ---"; printf '%s\n' "$NEW_MSG"
  exit 1
fi
echo "  identical: $NEW_MSG"
echo "THE DISPATCH MESSAGE DID NOT MOVE"

echo
echo "########## POINT 3 — the heartbeat message is a different instruction ##########"
# It must name the seat's own document and must NOT be the drain instruction.
grep -q "NudgeKind::Heartbeat" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: there is no heartbeat nudge kind"; exit 1; }
grep -q "no dispatch waiting" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat message does not say there is no dispatch waiting"; exit 1; }
grep -q "HEARTBEAT.md" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat message does not point at a document"; exit 1; }
# The heartbeat arm must not reuse the drain wording.
HB_ARM="$(python3 - "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" <<'PY'
import re, sys
src = open(sys.argv[1], encoding='utf-8').read()
m = re.search(r'NudgeKind::Heartbeat => \{(.*?)\n            \}', src, re.S)
print(m.group(1) if m else "")
PY
)"
[ -n "$HB_ARM" ] || { echo "FAIL: could not read the heartbeat arm"; exit 1; }
printf '%s\n' "$HB_ARM" | grep -q "get_next_dispatch" \
  && { echo "FAIL: the heartbeat message tells the seat to drain dispatches"; exit 1; } || true
printf '%s\n' "$HB_ARM" | grep -q "nothing ready" \
  && { echo "FAIL: the heartbeat message reuses the drain instruction's wording"; exit 1; } || true
echo "  the heartbeat arm names the document and never mentions draining"
echo "TWO SEPARATE INSTRUCTIONS"

echo
echo "########## POINT 4 — which real seats get called ##########"
# The seat map x the documents that exist. Only a seat with a document is called.
CALLED=0
NOT_CALLED=0
while IFS=$'\t' read -r seat session; do
  case "$seat" in ''|\#*) continue ;; esac
  short="${seat#seat-}"; short="${short%%-*}"
  doc="docs/coordination/tasks/${short}/$(printf '%s' "$short" | tr 'a-z' 'A-Z')-HEARTBEAT.md"
  if git -C "$ROOT" cat-file -e "${TRACK_REF}:${doc}" 2>/dev/null; then
    echo "  $seat ($short): HAS $doc -> would be called"
    CALLED=$((CALLED + 1))
  else
    echo "  $seat ($short): no document -> left alone"
    NOT_CALLED=$((NOT_CALLED + 1))
  fi
done < "$MAP"
echo "  called: $CALLED, left alone: $NOT_CALLED"
[ "$CALLED" -ge 1 ] || { echo "FAIL: no seat would be called — the change does nothing"; exit 1; }
[ "$NOT_CALLED" -ge 1 ] || { echo "FAIL: every seat would be called — the doc guard does nothing"; exit 1; }
# The dispatch's own claim: only Lyra has one today.
[ "$CALLED" -eq 1 ] || { echo "NOTE: $CALLED seats have documents, not 1 as the dispatch said"; }
LYRA_CALLED="$(git -C "$ROOT" cat-file -e "${TRACK_REF}:docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md" 2>/dev/null && echo yes || echo no)"
[ "$LYRA_CALLED" = "yes" ] || { echo "FAIL: Lyra has no document on $TRACK_REF"; exit 1; }
echo "  Lyra is the one with a document, as the dispatch said"
echo "ONLY A SEAT WITH A DOCUMENT IS CALLED"

echo
echo "########## POINT 5 — a seat WITHOUT a document is never called ##########"
# The unit tests that prove it, driven against a real temp directory — the same
# predicate the daemon uses, so this is the filesystem and not a string compare.
cargo test -q -p seatloom-mcp-emit --bin seatloom-waker 2>&1 | tail -3
cargo test -q -p seatloom-mcp-emit --bin seatloom-waker heartbeat 2>&1 | tail -3
# And the SAME path convention, measured against the real checkout on disk rather
# than a fixture: the document is where the daemon will look for it.
while IFS=$'\t' read -r seat session; do
  case "$seat" in ''|\#*) continue ;; esac
  short="${seat#seat-}"; short="${short%%-*}"
  doc="$MAIN/docs/coordination/tasks/${short}/$(printf '%s' "$short" | tr 'a-z' 'A-Z')-HEARTBEAT.md"
  if [ -f "$doc" ]; then
    echo "  $seat -> $doc EXISTS -> would be called"
  else
    echo "  $seat -> $doc absent -> left alone"
  fi
done < "$MAP"
# The one seat that has it must be the one the dispatch named.
[ -f "$MAIN/docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md" ] \
  || { echo "FAIL: Lyra's document is not on disk at $MAIN"; exit 1; }
N_ON_DISK=0
while IFS=$'\t' read -r seat session; do
  case "$seat" in ''|\#*) continue ;; esac
  short="${seat#seat-}"; short="${short%%-*}"
  [ -f "$MAIN/docs/coordination/tasks/${short}/$(printf '%s' "$short" | tr 'a-z' 'A-Z')-HEARTBEAT.md" ] && N_ON_DISK=$((N_ON_DISK + 1))
done < "$MAP"
echo "  seats with a document on disk: $N_ON_DISK"
[ "$N_ON_DISK" -eq 1 ] || { echo "FAIL: expected exactly 1 (Lyra), found $N_ON_DISK"; exit 1; }
echo "A SEAT WITHOUT A DOCUMENT IS LEFT ALONE"

echo
echo "########## POINT 6 — the cooldown is separate, longer, and prevents spam ##########"
grep -q "heartbeat_poker" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: there is no separate heartbeat poker"; exit 1; }
grep -q "SEATLOOM_HEARTBEAT_COOLDOWN_SECS" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat cooldown is not its own knob"; exit 1; }
grep -q "DEFAULT_HEARTBEAT_COOLDOWN" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat cooldown default is missing"; exit 1; }
# And the two Pokers are constructed from the two different knobs.
grep -q "Poker::new(cfg.heartbeat_cooldown)" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the heartbeat poker is not built from its own knob"; exit 1; }
grep -q "Poker::new(cfg.poke_cooldown)" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the dispatch poker is not built from its own knob"; exit 1; }
cargo test -q -p seatloom-mcp-emit --bin seatloom-waker heartbeat 2>&1 | tail -3
echo "TWO COOLDOWNS, AND NO SPAM"

echo
echo "########## POINT 7 — the two existing paths are untouched ##########"
# The idle-with-ready arm still nudges, with the dispatch kind.
grep -A 8 "(None, n) if n > 0 =>" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" | grep -q "NudgeKind::Dispatch" \
  || { echo "FAIL: the idle-with-ready arm no longer uses the dispatch nudge"; exit 1; }
# The stale-claim backstop arms still nudge, with the dispatch kind.
N_DISPATCH_NUDGES="$(grep -c "NudgeKind::Dispatch" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs")"
echo "  NudgeKind::Dispatch call sites: $N_DISPATCH_NUDGES (the old code had 6 nudge sites)"
[ "$N_DISPATCH_NUDGES" -ge 6 ] || { echo "FAIL: a dispatch nudge site was lost"; exit 1; }
# And the NOTIFY path and catch_up still use it too.
grep -q "nudge(session, NudgeKind::Dispatch, seat_id, tasks_dir);" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the NOTIFY path lost the dispatch nudge"; exit 1; }
grep -q "nudge(&session, NudgeKind::Dispatch, &seat_id, tasks_dir);" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  || { echo "FAIL: the catch-up path lost the dispatch nudge"; exit 1; }
# The state machine and the stale logic are not touched: no UPDATE dispatches.
grep -q "UPDATE dispatches" "$ROOT/crates/seatloom-mcp-emit/src/bin/waker.rs" \
  && { echo "FAIL: the waker now writes to dispatches"; exit 1; } || true
echo "THE EXISTING PATHS ARE INTACT"

echo
echo "ALL POINTS PASS"
