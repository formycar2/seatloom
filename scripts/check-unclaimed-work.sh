#!/usr/bin/env bash
# Which finished work has nobody to hand it to the operator?
#
# THE THREE QUESTIONS, and why one query is not enough (heartbeat protocol 2b/2c):
#   - "dispatch done"        -> work finished, nobody opened a card        (2b)
#   - "verdict pass, no card" -> verified green, operator never asked       (2c)
#   - "card pending"          -> is the card still clickable                (check-cards-clickable)
#
# 🔴 THE DEFECT THIS FIXES. The 2b query in the protocol is:
#      where d.status='done' and not exists (acceptances where dispatch_id=d.id)
# It does not ask WHAT KIND of work the dispatch was, so it reports every
# coordination dispatch too — the ones whose entire payload is "please open a
# card for X". Measured on live 2026-11-05 16:00: 9 rows, of which 5 were
# coordination dispatches that were never supposed to have a card. A query that
# reports the same five false rows every round trains the reader to skip it,
# and then the real row is skipped with them.
#
# 🔴🔴 AND MY FIRST FIX WAS STILL WRONG. I classified by payload SHAPE
# (packet/kind present = implementation) and it reported 4 rows — all four of
# them mine, all four verification dispatches, none of which should have a card.
# The shape does not distinguish them: my verification dispatches carry
# kind=verification AND note AND packet, the same keys as real work.
#
# Measured on live, the value is what separates them:
#     kind='verification'  -> 4 dispatches, 0 cards. Correct: a verification
#                             request is not a deliverable, it is a question.
#     kind absent          -> 7 dispatches, 2 of which have cards — and those 2
#                             are exactly the product work the operator accepted.
# So: verification is excluded, and everything else with no card is reported.
#
# Usage:
#   scripts/check-unclaimed-work.sh          # report, exit 1 if implementation work lacks a card
#   scripts/check-unclaimed-work.sh --quiet   # only print on failure
#
# Exit codes: 0 = nothing implementation-grade is stranded.
#             1 = at least one implementation dispatch has no card.
#             2 = the ledger could not be read (never a silent 0).

set -u

QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

DB="podman exec -i seatloom-postgres psql -X -v ON_ERROR_STOP=1 -U seatloom -d seatloom -t -A -F|"

# Prove the query can return rows before believing any empty answer.
# A missing table raises, it does not return 0 — but a swallowed stderr turns
# that raise into an empty result, which reads as "nothing is stranded".
if ! $DB -c "select count(*) from dispatches;" >/tmp/cuw-probe.txt 2>&1; then
  echo "FAIL: could not read the dispatches table. This is NOT 'no work'." >&2
  cat /tmp/cuw-probe.txt >&2
  exit 2
fi
TOTAL=$($DB -c "select count(*) from dispatches;")
[ -n "$TOTAL" ] || { echo "FAIL: dispatches count came back empty." >&2; exit 2; }

# Work that owes the operator a card: done, NOT a verification request, no card.
# A verification request is a question asked of a seat, not a deliverable —
# requiring a card for it would be a category error, and would put a fake
# obligation on the operator's plate.
IMPL=$($DB -c "
  select d.id || '|' || d.target_seat_id || '|' || coalesce(d.payload->>'kind','(none)') || '|' || coalesce(d.completed_at::text,'?')
  from dispatches d
  where d.status='done'
    -- verification: a question asked of a seat, not a deliverable
    and coalesce(d.payload->>'kind','') <> 'verification'
    -- note+acceptance: the task WAS the card step; a card of its own would be
    -- a category error. Measured: 5 such dispatches, 0 cards, correctly.
    and not (d.payload::jsonb ? 'acceptance')
    and not exists (select 1 from acceptances a where a.dispatch_id = d.id)
    and d.completed_at > now() - interval '48 hours'
  order by d.completed_at desc;")

# Verification requests and coordination payloads: reported for transparency,
# never as a defect.
OTHER=$($DB -c "
  select d.id || '|' || d.target_seat_id || '|' || coalesce(d.payload->>'kind','coordination')
  from dispatches d
  where d.status='done'
    and (coalesce(d.payload->>'kind','') = 'verification' or d.payload::jsonb ? 'acceptance')
    and not exists (select 1 from acceptances a where a.dispatch_id = d.id)
    and d.completed_at > now() - interval '48 hours'
  order by d.completed_at desc;")

IMPL_N=0
[ -n "$IMPL" ] && IMPL_N=$(printf '%s\n' "$IMPL" | grep -c .)
OTHER_N=0
[ -n "$OTHER" ] && OTHER_N=$(printf '%s\n' "$OTHER" | grep -c .)

if [ "$IMPL_N" -eq 0 ]; then
  [ "$QUIET" = "1" ] || echo "OK: no implementation-grade dispatch is stranded without a card"
  [ "$QUIET" = "1" ] || [ "$OTHER_N" -eq 0 ] || \
    echo "    (${OTHER_N} verification/coordination dispatch(es) have no card — correct, they owe no card)"
  exit 0
fi

echo "FAIL: ${IMPL_N} implementation dispatch(es) are done with NO acceptance card."
echo
echo "The chain is: dispatch done -> Flux verdict -> CARD CITING THAT VERDICT ->"
echo "operator accepts. Work stopped before the third link is invisible to every"
echo "mechanical check, because 'done' and 'verdict pass' both look like the end."
echo
echo "dispatch | seat | completed_at"
printf '%s\n' "$IMPL"
echo
echo "For each: write the delivery note, dispatch Flux to verify (naming the"
echo "implementation dispatch id, from the ledger, not typed), then open a card"
echo "citing the PASS verdict."
exit 1
