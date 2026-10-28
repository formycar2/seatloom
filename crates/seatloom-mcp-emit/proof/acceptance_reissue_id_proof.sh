#!/usr/bin/env bash
# Proof for the `acceptance_id` parameter on `request_acceptance`
# (dispatch disp-acceptance-reissue-id-param).
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# built in the canonical order (pre-seed schema 001-005+009, seed 001-004,
# post-seed 008+010-014, then 015-019 from track/infra-foundation because 017 is
# what puts `cancelled` in the acceptances status vocabulary). NEVER the live
# seatloom DB — this proof only ever writes to its own scratch database.
#
# The defect it closes, verbatim from the washing seat's dispatch:
#   duplicate key value violates unique constraint acceptances_pkey,
#   Key (id)=(acc-disp-wash-vela-p1/p2) already exists (sqlstate 23505)
# `cancel_acceptance` is terminal and never deletes the row, so a cancelled card
# keeps occupying its primary key, and there was no way to re-request the same
# delivery. The washing seat refused to fake a success and failed its own
# dispatch instead.
#
# POINTs:
#   1  omitting the id keeps the historical deterministic id, byte for byte
#   2  a cancelled card re-opens under the SAME id, contents refreshed
#   3  a pending id is never taken over, and nothing is written
#   4  another seat's id is refused; a decided id is not reusable
#   5  an id encoding a different dispatch is refused (lineage intact)
#   6  the live preconditions for the two real cards are verified read-only
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_accid}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=accid-proof
TRACK_REF="${TRACK_REF:-track/infra-foundation}"

[[ "$DB" == "seatloom" ]] && { echo "REFUSING the live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }

drive() { # actor, then jsonrpc lines
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" \
    SEATLOOM_PROJECT_ID="$PROJECT" "$BIN" 2>/dev/null
}
result() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR: " if r["isError"] else "")+r["content"][0]["text"])'; }

# ---------------------------------------------------------------------------
# Scratch database in the canonical order (scripts/apply-db.sh's grouping).
# ---------------------------------------------------------------------------
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${DB}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${DB}" >/dev/null
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
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
# 015-019 live on the integration branch (Onyx's); 017 adds `cancelled`.
for n in 015_knowledge_layer 016_acceptance_plan 017_acceptance_cancel 018_memory_hardening 019_topic_opened; do
  git show "${TRACK_REF}:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from ${TRACK_REF}"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
echo "scratch database ${DB} built in canonical order (001-014 + seed + 015-019 from ${TRACK_REF})"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','accid proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null

# The request-time base guard needs a real repo whose `track` branch exists.
BASEREPO="$(mktemp -d)"
git -C "$BASEREPO" init -q -b track
git -C "$BASEREPO" config user.email t@t.local; git -C "$BASEREPO" config user.name t
echo base > "$BASEREPO/f.txt"; git -C "$BASEREPO" add .; git -C "$BASEREPO" commit -qm base
git -C "$BASEREPO" branch packet/nimbus/x track
git -C "$BASEREPO" branch packet/flux/x track
export SEATLOOM_REPO_ROOT="$BASEREPO"

# A Flux verdict the request can reference.
psql "INSERT INTO canonical_events (id,event_type,schema_version,occurred_at,actor_ref,payload,project_id) VALUES ('ev-accid-verdict','ReviewVerdictIssued',1,NOW(),'seat:seat-flux-001','{\"verdict\":\"pass\"}','${PROJECT}');" >/dev/null

PLAN='{"plan_version":1,"scenario":"the operator opens the card and follows the plan","steps":[{"action":"open the acceptance card"}],"expected":[{"output":"the plan is readable","expect":"the card shows it"}]}'
# Seat ids in this project are `seat-<name>-001`; the actor ref is `seat:` + that.
seat_short() { echo "${1#seat-}" | sed 's/-001$//'; }
done_dispatch() { # seat_id, dispatch id
  # NOTE: the arguments JSON is built into a variable and passed as ONE word.
  # Nesting `"...\"..\"..."` directly inside `"$(call ...)"` is a bash quoting
  # trap that silently word-splits inside a function; this form is stable.
  local args
  args="{\"target_seat_id\":\"$1\",\"id\":\"$2\",\"title\":\"t\",\"payload\":{},\"priority\":5,\"project_id\":\"${PROJECT}\"}"
  drive "seat:$1" "$INIT" "$INITED" "$(call enqueue_dispatch "$args")" >/dev/null
  args="{\"project_id\":\"${PROJECT}\"}"
  drive "seat:$1" "$INIT" "$INITED" "$(call get_next_dispatch "$args")" >/dev/null
  args="{\"dispatch_id\":\"$2\"}"
  drive "seat:$1" "$INIT" "$INITED" "$(call complete_dispatch "$args")" >/dev/null
}
req() { # seat_id, dispatch, acceptance_id-or-empty, extra how_to_verify
  local short; short="$(seat_short "$1")"
  local extra="" idfield="" args
  [ -n "${4:-}" ] && extra=",\"how_to_verify\":\"$4\""
  [ -n "${3:-}" ] && idfield=",\"acceptance_id\":\"$3\""
  args="{\"dispatch_id\":\"$2\",\"branch\":\"packet/${short}/x\",\"how_to_verify\":\"do A see B\"${extra},\"flux_verdict_ref\":\"ev-accid-verdict\",\"base\":\"track\",\"acceptance_plan\":${PLAN}${idfield}}"
  drive "seat:$1" "$INIT" "$INITED" "$(call request_acceptance "$args")" | result
}
row() { psql "SELECT status||'|'||coalesce(how_to_verify,'-')||'|'||coalesce(feedback,'-') FROM acceptances WHERE id='$1';"; }

echo
echo "########## POINT 1 — omitting the id is the historical behaviour ##########"
done_dispatch seat-nimbus-001 disp-proof-derive
OUT="$(req seat-nimbus-001 disp-proof-derive "")"
echo "  $OUT"
echo "$OUT" | grep -q "requested acceptance acc-disp-proof-derive" || { echo "FAIL: not the deterministic id"; exit 1; }
echo "$OUT" | grep -q "re-opened" && { echo "FAIL: a fresh request must not say re-opened"; exit 1; }
echo "  row: $(row acc-disp-proof-derive)"

echo
echo "########## POINT 2 — a cancelled card re-opens under the SAME id ##########"
done_dispatch seat-nimbus-001 disp-proof-reopen
req seat-nimbus-001 disp-proof-reopen "" >/dev/null
drive seat-nimbus-001 "$INIT" "$INITED" \
  "$(call cancel_acceptance '{"acceptance_id":"acc-disp-proof-reopen","reason":"the plan was rewritten in plain language"}')" | result
echo "  after cancel: $(row acc-disp-proof-reopen)"
OUT="$(req seat-nimbus-001 disp-proof-reopen acc-disp-proof-reopen "open the card and read the plain-language plan")"
echo "  $OUT"
echo "$OUT" | grep -q "re-opened acceptance acc-disp-proof-reopen" || { echo "FAIL: not a re-open"; exit 1; }
EXP="pending|open the card and read the plain-language plan|-"
GOT="$(row acc-disp-proof-reopen)"
echo "  after re-open: ${GOT}"
[ "$GOT" = "$EXP" ] || { echo "FAIL: expected ${EXP}"; exit 1; }
N=$(psql "SELECT count(*) FROM acceptances WHERE id='acc-disp-proof-reopen';")
[ "$N" = "1" ] || { echo "FAIL: ${N} rows for one id"; exit 1; }
echo "  one row, contents refreshed, cancel reason cleared"
echo "  ledger: $(psql "SELECT string_agg(event_type,',' ORDER BY occurred_at) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE r.ref_type='acceptance' AND r.ref_id='acc-disp-proof-reopen';")"

echo
echo "########## POINT 3 — a pending id is never taken over ##########"
done_dispatch seat-nimbus-001 disp-proof-held
req seat-nimbus-001 disp-proof-held "" >/dev/null
BEFORE="$(row acc-disp-proof-held)"
done_dispatch seat-nimbus-001 disp-proof-squatter
OUT="$(req seat-nimbus-001 disp-proof-squatter acc-disp-proof-held)"
echo "  $OUT"
echo "$OUT" | grep -q "never taken over" || { echo "FAIL: not refused as occupied"; exit 1; }
[ "$(row acc-disp-proof-held)" = "$BEFORE" ] || { echo "FAIL: the occupant changed"; exit 1; }
N=$(psql "SELECT count(*) FROM acceptances WHERE dispatch_id='disp-proof-squatter';")
[ "$N" = "0" ] || { echo "FAIL: the refused request wrote a row"; exit 1; }
echo "  occupant unchanged, and no row was written for the refused request"

echo
echo "########## POINT 4 — another seat's id, and a decided id ##########"
done_dispatch seat-flux-001 disp-proof-flux
req seat-flux-001 disp-proof-flux "" >/dev/null
done_dispatch seat-nimbus-001 disp-proof-borrow
OUT="$(req seat-nimbus-001 disp-proof-borrow acc-disp-proof-flux)"
echo "  (flux's card is pending) $OUT"
echo "$OUT" | grep -q "never taken over" || { echo "FAIL: pending occupant not protected"; exit 1; }
echo "$OUT" | grep -q "seat-flux-001" || { echo "FAIL: the owner is not named"; exit 1; }
# Flux cancels its own card; nimbus still cannot take the id.
drive seat-flux-001 "$INIT" "$INITED" \
  "$(call cancel_acceptance '{"acceptance_id":"acc-disp-proof-flux","reason":"superseded"}')" >/dev/null
OUT="$(req seat-nimbus-001 disp-proof-borrow acc-disp-proof-flux)"
echo "  (flux cancelled it)    $OUT"
echo "$OUT" | grep -q "gates a dispatch owned by" || { echo "FAIL: seat-identity guard bypassed"; exit 1; }
# Now the operator decides it; a decided id is not reusable either.
psql "UPDATE acceptances SET status='accepted', decided_by='human:supervisor', decided_at=NOW(), merge_commit='deadbeef', feedback=NULL WHERE id='acc-disp-proof-flux';" >/dev/null
OUT="$(req seat-nimbus-001 disp-proof-borrow acc-disp-proof-flux)"
echo "  (operator accepted it) $OUT"
echo "$OUT" | grep -q "terminal state" || { echo "FAIL: a decided id was reused"; exit 1; }
[ "$(row acc-disp-proof-flux | cut -d'|' -f1)" = "accepted" ] || { echo "FAIL: the decided row changed"; exit 1; }
echo "  the decided row is untouched"

echo
echo "########## POINT 5 — an id encoding another dispatch is refused ##########"
done_dispatch seat-nimbus-001 disp-proof-first
done_dispatch seat-nimbus-001 disp-proof-second
req seat-nimbus-001 disp-proof-first "" >/dev/null
drive seat-nimbus-001 "$INIT" "$INITED" \
  "$(call cancel_acceptance '{"acceptance_id":"acc-disp-proof-first","reason":"superseded"}')" >/dev/null
OUT="$(req seat-nimbus-001 disp-proof-second acc-disp-proof-first)"
echo "  $OUT"
echo "$OUT" | grep -q "already gates dispatch" || { echo "FAIL: lineage not protected"; exit 1; }
echo "$OUT" | grep -q "disp-proof-first" || { echo "FAIL: the card's dispatch is not named"; exit 1; }
# The correct pairing still works.
OUT="$(req seat-nimbus-001 disp-proof-first acc-disp-proof-first)"
echo "  correct pairing: $OUT"
echo "$OUT" | grep -q "re-opened" || { echo "FAIL: the correct pairing did not re-open"; exit 1; }

echo
echo "########## POINT 6 — the live preconditions for the two real cards ##########"
# READ-ONLY against the live database: every fact the re-open path will need for
# the washing seat's two cards. Nothing here writes.
LIVE="${LIVE_DB_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"
live() { podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "$1"; }
for id in acc-disp-wash-vela-p1 acc-disp-wash-vela-p2; do
  LINE="$(live "SELECT a.status||'|'||a.dispatch_id||'|'||d.target_seat_id||'|'||d.status FROM acceptances a JOIN dispatches d ON d.id=a.dispatch_id WHERE a.id='${id}';")"
  echo "  ${id}: ${LINE}"
  IFS='|' read -r st dp ow ds <<< "$LINE"
  [ "$st" = "cancelled" ] || { echo "FAIL: ${id} is ${st}, not cancelled"; exit 1; }
  [ "$ow" = "seat-vela-001" ] || { echo "FAIL: ${id} is owned by ${ow}"; exit 1; }
  [ "$ds" = "done" ] || { echo "FAIL: ${dp} is ${ds}, not done"; exit 1; }
done
P=$(live "SELECT count(*) FROM acceptances WHERE id IN ('acc-disp-wash-vela-p1','acc-disp-wash-vela-p2') AND status='pending';")
[ "$P" = "0" ] || { echo "FAIL: ${P} of the two ids is already pending"; exit 1; }
echo "  both cards are cancelled, owned by seat-vela-001, gating done dispatches, and neither id is pending"
echo "  (the re-open itself is the washing seat's act — this point proves the path is open, it does not take it)"
echo "  the integration binary does NOT advertise acceptance_id yet: $(printf '%s\n%s\n%s\n' "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit 2>/dev/null | tail -1 | python3 -c 'import sys,json; t=[x for x in json.load(sys.stdin)["result"]["tools"] if x["name"]=="request_acceptance"][0]; print("acceptance_id" in t["inputSchema"]["properties"])')"

echo
echo "ALL POINTS PASS"
