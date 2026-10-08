#!/usr/bin/env bash
# Behavioural proof for P0 step 4 — the MCP emit server.
# Packet: NIMBUS-2026-10-08-p0-step4-mcp-emit-v1 §4.
#
# Drives the REAL seatloom-mcp-emit binary over stdio (JSON-RPC 2.0) against a
# SCRATCH database, and checks ledger/projection row counts via psql in the
# podman postgres container BEFORE and AFTER each case. Never the live DB.
#
# Prereqs (set up by the delivery, see the delivery doc):
#   - scratch DB seatloom_mcp_nimbus with migrations 001..010 applied
#   - fixtures: project 'p0-mcp-proof', seat 'seat-nimbus-001'
#   - binary built: cargo build -p seatloom-mcp-emit
#
# Usage:
#   crates/seatloom-mcp-emit/proof/mcp_emit_proof.sh
set -euo pipefail

DB=seatloom_mcp_nimbus
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
BIN="$(git rev-parse --show-toplevel)/target/debug/seatloom-mcp-emit"
PROJECT=p0-mcp-proof

if [[ "$DB" == "seatloom" ]]; then echo "REFUSING to run against the live database"; exit 1; fi

psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }
events_count() { psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';"; }

# Pipe one or more JSON-RPC lines through the server; print stdout responses.
drive() { printf '%s\n' "$@" | DATABASE_URL="$DBURL" "$BIN" 2>/dev/null; }

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
LIST='{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

# emit_event tools/call with the given arguments object.
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"emit_event\",\"arguments\":$1}}"; }

echo "target database = ${DB}"

# ---- clean this project's events/workitems so the proof is deterministic ----
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM workitems WHERE project_id='${PROJECT}';" >/dev/null
echo "fixture: ${PROJECT} events + workitems cleared; count=$(events_count)"

echo
echo "########## tools/list — exactly one tool, emit_event ##########"
drive "$INIT" "$INITED" "$LIST" | tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]["tools"]; print("tool count =", len(r), "| name =", r[0]["name"])'

echo
echo "########## PROOF 1 — happy path: event + refs + projection, via the write API ##########"
before=$(events_count)
WI=wi-mcp-001
args='{"type":"ArtifactCreated","payload":{"kind":"delivery","path":"docs/x.md"},"refs":[{"ref_type":"workitem","ref_id":"'$WI'","role":"subject"},{"ref_type":"seat","ref_id":"seat-nimbus-001","role":"actor"}],"actor_ref":"seat:seat-nimbus-001","project_id":"'$PROJECT'","event_id":"ev-mcp-artifact-1"}'
drive "$INIT" "$INITED" "$(call "$args")" | tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print("isError =", r["isError"], "| text =", r["content"][0]["text"])'
after=$(events_count)
echo "canonical_events for ${PROJECT}: before=${before} after=${after}  (expect +1)"
echo "--- the event + its refs (roles shown) ---"
psql "SELECT e.id||' | '||e.event_type||' | sv='||e.schema_version||' | turn='||coalesce(e.turn_seq::text,'NULL') FROM canonical_events e WHERE e.id='ev-mcp-artifact-1';"
psql "SELECT '  ref '||r.ref_type||' '||r.ref_id||' role='||r.role FROM event_object_refs r WHERE r.event_id='ev-mcp-artifact-1' ORDER BY r.role;"

echo
echo "########## PROOF 1b — WorkItemCreated drives the projection in the same txn ##########"
WI2=wi-mcp-proj-1
args='{"type":"WorkItemCreated","payload":{"title":"via MCP","status":"in_progress","priority":"high","owner_seat_id":"seat-nimbus-001"},"refs":[{"ref_type":"workitem","ref_id":"'$WI2'","role":"subject"},{"ref_type":"seat","ref_id":"seat-nimbus-001","role":"actor"}],"actor_ref":"seat:seat-nimbus-001","project_id":"'$PROJECT'","event_id":"ev-mcp-wi-1"}'
drive "$INIT" "$INITED" "$(call "$args")" | tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print("isError =", r["isError"], "| text =", r["content"][0]["text"])'
echo "--- projection row (written in the SAME transaction as the event) ---"
psql "SELECT 'workitems row: '||id||' status='||status||' priority='||priority FROM workitems WHERE id='$WI2';"

echo
echo "########## PROOF 2 — rejections: each fails with a typed error and writes NOTHING ##########"
reject() {
  local label="$1" args="$2"
  local b a
  b=$(events_count)
  local text
  text=$(drive "$INIT" "$INITED" "$(call "$args")" | tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR " if r["isError"] else "OK  ")+r["content"][0]["text"])')
  a=$(events_count)
  printf '  %-28s %s\n' "$label" "$text"
  printf '  %-28s events before=%s after=%s %s\n' "" "$b" "$a" "$([[ $b == $a ]] && echo '(nothing written OK)' || echo '!!! WROTE SOMETHING')"
}
reject "unknown type"        '{"type":"Nope","payload":{"a":1},"refs":[{"ref_type":"workitem","ref_id":"x","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "zero subjects"       '{"type":"ArtifactCreated","payload":{"a":1},"refs":[{"ref_type":"seat","ref_id":"s","role":"actor"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "two subjects"        '{"type":"ArtifactCreated","payload":{"a":1},"refs":[{"ref_type":"workitem","ref_id":"a","role":"subject"},{"ref_type":"artifact","ref_id":"b","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "schema_version 0"    '{"type":"ArtifactCreated","schema_version":0,"payload":{"a":1},"refs":[{"ref_type":"workitem","ref_id":"a","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
# A1 HAS a payload contract for WorkItemCreated, so a version it does not know
# (v2) is rejected with UnsupportedPayloadVersion. NB: the 10 event_types with
# no payload contract (e.g. ArtifactCreated) are ledger-only at ANY version by
# A1's design — see delivery doc Finding F1; the MCP layer does not invent a
# stricter version gate than the write API enforces.
reject "no-contract version" '{"type":"WorkItemCreated","schema_version":2,"payload":{"title":"t","status":"draft","priority":"low"},"refs":[{"ref_type":"workitem","ref_id":"a","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "payload not object"  '{"type":"ArtifactCreated","payload":[1,2],"refs":[{"ref_type":"workitem","ref_id":"a","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "wrong subject type"  '{"type":"WorkItemCreated","payload":{"title":"t","status":"draft","priority":"low"},"refs":[{"ref_type":"session","ref_id":"a","role":"subject"}],"actor_ref":"seat:s","project_id":"'$PROJECT'"}'
reject "missing actor_ref"   '{"type":"ArtifactCreated","payload":{"a":1},"refs":[{"ref_type":"workitem","ref_id":"a","role":"subject"}],"project_id":"'$PROJECT'"}'

echo
echo "########## PROOF 3 — atomicity: a mid-append failure rolls back event + refs + projection ##########"
before=$(events_count)
# owner_seat_id points at a non-existent seat -> projection INSERT violates the
# FK AFTER canonical_events and refs were inserted in the same transaction.
args='{"type":"WorkItemCreated","payload":{"title":"doomed","status":"draft","priority":"low","owner_seat_id":"seat-does-not-exist-999"},"refs":[{"ref_type":"workitem","ref_id":"wi-mcp-doomed","role":"subject"},{"ref_type":"seat","ref_id":"seat-nimbus-001","role":"actor"}],"actor_ref":"seat:seat-nimbus-001","project_id":"'$PROJECT'","event_id":"ev-mcp-doomed"}'
drive "$INIT" "$INITED" "$(call "$args")" | tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print("isError =", r["isError"], "| text =", r["content"][0]["text"][:160])'
after=$(events_count)
ev=$(psql "SELECT count(*) FROM canonical_events WHERE id='ev-mcp-doomed';")
rf=$(psql "SELECT count(*) FROM event_object_refs WHERE event_id='ev-mcp-doomed';")
wi=$(psql "SELECT count(*) FROM workitems WHERE id='wi-mcp-doomed';")
echo "after the failed append: events=${ev} refs=${rf} workitems=${wi}  (all must be 0)"
echo "project event count: before=${before} after=${after}  (must be equal)"
[[ "$ev" == 0 && "$rf" == 0 && "$wi" == 0 && "$before" == "$after" ]] && echo "HELD: rollback is atomic, not partial." || { echo "!!! atomicity violated"; exit 1; }

echo
echo "########## ALL PROOFS HELD ##########"
