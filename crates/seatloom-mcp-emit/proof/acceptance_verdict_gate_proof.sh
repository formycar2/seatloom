#!/usr/bin/env bash
# Proof for the acceptance gate's verdict criterion — the two defects of
# 2026-10-16 (dispatch disp-ftEie9vQ9MJEh0lyv7YkA).
#
# THE DEFECTS, measured on the LIVE database before anything was changed:
#
#   · three PENDING cards (the supervisor-panel event-type translation, the
#     two-cancelled-cards count, the live-seat reconnect fix) all named
#     `ev-verdict-history-sessions` as their flux_verdict_ref.
#   · that event is a **fail** verdict about `disp-history-sessions-verify`.
#   · no ReviewVerdictIssued event names any of those three dispatches — Flux
#     never verified them.
#   · and a click on Accept would have merged, because the criterion was
#     `event_or_artifact_exists`: existence, with no regard for the value or the
#     subject. The comment above it said "green"; the code never read green.
#
# WHAT IS PROVEN:
#   1  the three live cards are exactly as described (measured, not asserted)
#   2  the OLD criterion accepted them — existence only, red and foreign
#   3  the NEW criterion refuses them, naming the verdict and its value
#   4  a green verdict about ANOTHER dispatch is refused, naming whose it is
#   5  a green verdict about THIS dispatch is still accepted — no regression
#   6  the three DB tests the dispatch required both exist and pass
#   7  the read is read-only: the live ledger does not move
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
LIVE="${SEATLOOM_PROOF_LIVE_DB:-seatloom}"
SCRATCH="${SEATLOOM_PROOF_DB:-seatloom_verdict_gate}"

[ "$LIVE" != "$SCRATCH" ] || { echo "REFUSING: live and scratch are the same database"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT 1" >/dev/null 2>&1 \
  || { echo "the live database is not reachable"; exit 1; }

q_live()   { podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "$1"; }
q_scratch() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "$1"; }

echo "live database:    $LIVE"
echo "scratch database: $SCRATCH"
echo

echo "########## POINT 1 — the defect's shape on live, and the harm it already did ##########"
# Measured when this proof was written (2026-10-16 ~08:10Z), NOT when the dispatch
# was issued: the three cards the dispatch named are no longer PENDING — their
# owner withdrew them — and one further card was ACCEPTED on the same bad verdict.
# So the shape is measured as it stands now, in both halves.
WRONG="$(q_live "SELECT count(*) FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions';")"
echo "  cards naming ev-verdict-history-sessions: $WRONG"
[ "$WRONG" -ge 4 ] || { echo "FAIL: expected at least 4 such cards, found $WRONG"; exit 1; }
q_live "SELECT '    ' || id || ' ' || status FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions' ORDER BY created_at;"

# That verdict is a FAIL, about a DIFFERENT dispatch.
VAL="$(q_live "SELECT payload->>'verdict' FROM canonical_events WHERE id='ev-verdict-history-sessions';")"
VDISP="$(q_live "SELECT payload->>'dispatch_id' FROM canonical_events WHERE id='ev-verdict-history-sessions';")"
VTYPE="$(q_live "SELECT event_type FROM canonical_events WHERE id='ev-verdict-history-sessions';")"
echo "  ev-verdict-history-sessions: type=$VTYPE verdict=$VAL about=$VDISP"
[ "$VTYPE" = "ReviewVerdictIssued" ] || { echo "FAIL: not a verdict event"; exit 1; }
[ "$VAL" = "fail" ] || { echo "FAIL: the verdict is not fail ($VAL) — the defect's shape changed"; exit 1; }
[ "$VDISP" = "disp-history-sessions-verify" ] || { echo "FAIL: the verdict is about $VDISP"; exit 1; }

# Flux never verified any of the dispatches those cards gate.
for d in $(q_live "SELECT dispatch_id FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions';"); do
  N="$(q_live "SELECT count(*) FROM canonical_events e WHERE e.event_type='ReviewVerdictIssued' AND (e.payload->>'dispatch_id'='$d' OR EXISTS (SELECT 1 FROM event_object_refs r WHERE r.event_id=e.id AND r.ref_type='dispatch' AND r.ref_id='$d'));")"
  echo "  ReviewVerdictIssued about $d: $N"
  [ "$N" = "0" ] || { echo "FAIL: Flux DID verify $d"; exit 1; }
done

# THE HARM, MEASURED. Cards were ACCEPTED and merged on that verdict. Each of
# them gates a dispatch Flux never verified.
for row in $(q_live "SELECT id || '|' || coalesce(merge_commit,'') FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions' AND status='accepted';"); do
  HARM="${row%%|*}"; MC="${row#*|}"
  echo "  ACCEPTED on the fail verdict: $HARM -> merge_commit=${MC:-<none>}"
  [ -n "$MC" ] || { echo "FAIL: an accepted card with no merge commit"; exit 1; }
  git -C "$ROOT" cat-file -e "$MC^{commit}" 2>/dev/null \
    || { echo "FAIL: merge commit $MC is not in this repository"; exit 1; }
  echo "    $MC exists in this repository"
done
echo "  -> unverified merges already landed on that verdict. Reported to Aegis;"
echo "     NOT reverted here — whether they stand is Aegis's ruling, not mine."
echo "THE DEFECT'S SHAPE AND ITS HARM ARE BOTH MEASURED ON LIVE"

echo
echo "########## POINT 2 — the OLD criterion accepted all of it ##########"
# `event_or_artifact_exists` is the question the old code asked. It is true.
OLD_OK="$(q_live "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id='ev-verdict-history-sessions') OR EXISTS (SELECT 1 FROM artifacts WHERE id='ev-verdict-history-sessions');")"
echo "  event_or_artifact_exists('ev-verdict-history-sessions') = $OLD_OK"
[ "$OLD_OK" = "t" ] || { echo "FAIL: the old criterion would not even have passed"; exit 1; }
echo "  existence is true, the value is fail, the subject is another dispatch"
echo "  -> the old criterion would have merged on the operator's click"
echo "THE OLD CRITERION WAS EXISTENCE, NOT GREEN"

echo
echo "########## POINT 3 — the NEW criterion refuses them, naming verdict and value ##########"
# The new read, measured against live through the REAL function. The probe is a
# committed example (`cargo run -p seatloom-core --example verdict_facts_probe`),
# not a file the proof writes into the tree — a proof that creates its own source
# can silently replace a fixed version with a broken one.
PROBE="$ROOT/target/debug/examples/verdict_facts_probe"
cargo build -q -p seatloom-core --example verdict_facts_probe 2>&1 | tail -3
[ -x "$PROBE" ] || { echo "FAIL: the probe did not build"; exit 1; }

for d in $(q_live "SELECT dispatch_id FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions';"); do
  OUT="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/$LIVE" \
         VERDICT_REF=ev-verdict-history-sessions DISPATCH_ID="$d" "$PROBE")"
  echo "  $d -> $OUT"
  echo "$OUT" | grep -q 'exists=true'          || { echo "FAIL: the verdict should exist"; exit 1; }
  echo "$OUT" | grep -q 'verdict=Some("fail")' || { echo "FAIL: the verdict should read fail"; exit 1; }
  echo "$OUT" | grep -q 'names_dispatch=false' || { echo "FAIL: it should not name this dispatch"; exit 1; }
  echo "$OUT" | grep -q 'green=false'          || { echo "FAIL: a fail verdict about another dispatch must not be the merge criterion"; exit 1; }
done
echo "EVERY ONE OF THEM READS green=false"

echo
echo "########## POINT 4 — a green verdict about ANOTHER dispatch is still refused ##########"
# Find a real PASS verdict about some dispatch, and ask about a different one.
GREEN="$(q_live "SELECT id FROM canonical_events WHERE event_type='ReviewVerdictIssued' AND payload->>'verdict'='pass' AND payload->>'dispatch_id' IS NOT NULL ORDER BY occurred_at DESC LIMIT 1;")"
GD="$(q_live "SELECT payload->>'dispatch_id' FROM canonical_events WHERE id='$GREEN';")"
echo "  a real green verdict: $GREEN (about $GD)"
OUT="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/$LIVE" \
       VERDICT_REF="$GREEN" DISPATCH_ID="disp-some-other-dispatch" "$PROBE")"
echo "  asked about 'disp-some-other-dispatch' -> $OUT"
echo "$OUT" | grep -q 'verdict=Some("pass")'     || { echo "FAIL: it should read pass"; exit 1; }
echo "$OUT" | grep -q 'names_dispatch=false'     || { echo "FAIL: it must not name a dispatch it does not name"; exit 1; }
echo "$OUT" | grep -q 'green=false'              || { echo "FAIL: green must require being about THIS dispatch"; exit 1; }
# And asked about the dispatch it IS about, it is green — no regression.
OUT="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/$LIVE" \
       VERDICT_REF="$GREEN" DISPATCH_ID="$GD" "$PROBE")"
echo "  asked about $GD -> $OUT"
echo "$OUT" | grep -q 'green=true' || { echo "FAIL: a green verdict about this dispatch must be green"; exit 1; }
echo "A GREEN VERDICT IS GREEN ONLY FOR ITS OWN DISPATCH"

echo
echo "########## POINT 5 — the two required DB tests exist and pass ##########"
grep -q 'async fn a_fail_verdict_cannot_be_the_merge_criterion' "$ROOT/crates/seatloom-mcp-emit/src/dispatch.rs" \
  || { echo "FAIL: the fail-verdict test is missing"; exit 1; }
grep -q 'async fn another_dispatches_verdict_cannot_be_the_merge_criterion' "$ROOT/crates/seatloom-mcp-emit/src/dispatch.rs" \
  || { echo "FAIL: the foreign-verdict test is missing"; exit 1; }
grep -q 'async fn a_green_verdict_about_this_dispatch_still_opens_a_card' "$ROOT/crates/seatloom-mcp-emit/src/dispatch.rs" \
  || { echo "FAIL: the no-regression test is missing"; exit 1; }
echo "  all three tests are present"

# Build the scratch database in the canonical order (001-014 + 015-019 from track).
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${SCRATCH}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${SCRATCH}" >/dev/null
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
S="$ROOT/infra/postgres/schema"; D="$ROOT/infra/postgres/seed"
for f in 001_seatloom_core 002_document_authority 003_write_ingest_reconcile \
         004_operational_review_and_continuity 005_prompt_and_channel_action_authority 009_runtime_triple; do
  apply < "$S/${f}.sql" || { echo "pre-seed schema failed at $f"; exit 1; }
done
for f in 001_real_collaboration_baseline 002_document_seed \
         003_operational_review_and_continuity_seed 004_prompt_and_channel_action_seed; do
  apply < "$D/${f}.sql" || { echo "seed failed at $f"; exit 1; }
done
for f in 008_project_isolation 010_p0_schema_foundation 011_agent_history_archive \
         012_governed_dispatch 013_dispatch_states 014_acceptance_gate; do
  apply < "$S/${f}.sql" || { echo "post-seed schema failed at $f"; exit 1; }
done
for n in 015_knowledge_layer 016_acceptance_plan 017_acceptance_cancel 018_memory_hardening 019_topic_opened; do
  git show "${TRACK_REF:-track/infra-foundation}:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from track"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
echo "  scratch database ${SCRATCH} built (canonical order + 015-019)"

DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}" \
  cargo test -q -p seatloom-mcp-emit --bin seatloom-mcp-emit -- --ignored --test-threads=1 2>&1 | tail -3
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}" \
  cargo test -q -p seatloom-mcp-emit --bin seatloom-mcp-emit -- --ignored --test-threads=1 2>&1 \
  | grep -q "test result: ok" || { echo "FAIL: the DB tests did not all pass"; exit 1; }
echo "ALL DB TESTS PASS ON THE SCRATCH DATABASE"

echo
echo "########## POINT 6 — the read is read-only on live ##########"
B="$(q_live "SELECT (SELECT count(*) FROM canonical_events) || '/' || (SELECT count(*) FROM event_object_refs) || '/' || (SELECT count(*) FROM acceptances);")"
for d in $(q_live "SELECT dispatch_id FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions';"); do
  DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/$LIVE" \
    VERDICT_REF=ev-verdict-history-sessions DISPATCH_ID="$d" "$PROBE" >/dev/null
done
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/$LIVE" \
  VERDICT_REF="$GREEN" DISPATCH_ID="disp-some-other-dispatch" "$PROBE" >/dev/null
A="$(q_live "SELECT (SELECT count(*) FROM canonical_events) || '/' || (SELECT count(*) FROM event_object_refs) || '/' || (SELECT count(*) FROM acceptances);")"
echo "  live ledger events/refs/acceptances: before=$B after=$A"
[ "$B" = "$A" ] || { echo "FAIL: the read wrote to live"; exit 1; }
echo "THE READ IS READ ONLY"

echo
echo "ALL POINTS PASS"
