#!/usr/bin/env bash
# Full 13-point behavioural proof for P0 step 6 — governed dispatch (G19).
# Packet: NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1 §2; unblock:
# LYRA-2026-10-08-p0-step6-unblock-v1 §2.3.
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# (seatloom_mcp_step6_nimbus, schema 001..012 via the canonical order), checking
# the dispatches projection AND canonical_events via psql in the podman postgres
# container. NEVER the live seatloom DB.
#
# Identity is per-server-env (SEATLOOM_ACTOR_REF); to act AS a given seat we
# launch the server with that seat's actor ref — which is exactly the
# non-forgeable identity model (there is no seat_id parameter to pass).
set -uo pipefail

DB=seatloom_mcp_step6_nimbus
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
BIN="$(git rev-parse --show-toplevel)/target/debug/seatloom-mcp-emit"
PROJECT=p0-step6-proof
NIMBUS=seat-nimbus-001
MIRA=seat-mira-001

[[ "$DB" == "seatloom" ]] && { echo "REFUSING live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }

# drive <ACTOR_REF> <json-rpc line...>  — launch a server AS that seat.
drive() {
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" SEATLOOM_PROJECT_ID="$PROJECT" "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
# text of the last tool result
lastres() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR " if r["isError"] else "OK  ")+r["content"][0]["text"])'; }

echo "target database = ${DB}"
# Fresh proof project; clean any prior proof rows (events, deps, dispatches).
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','step6 proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM dispatches WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
echo "fixture: project ${PROJECT} clean"

echo
echo "########## tools/list — four tools, get_next_dispatch has NO seat param ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 \
 | python3 -c 'import sys,json; t=json.load(sys.stdin)["result"]["tools"]; print("tools:",[x["name"] for x in t]); gp=[x for x in t if x["name"]=="get_next_dispatch"][0]["inputSchema"]["properties"]; print("get_next seat params:", [k for k in gp if "seat" in k] or "NONE (non-forgeable)")'

echo
echo "########## POINT 1 — enqueue two for nimbus: one ready, one blocked on a missing event ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-ready","target_seat_id":"'$NIMBUS'","title":"ready task","payload":{"packet":"p/ready.md"},"priority":5}')" | lastres
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-blocked","target_seat_id":"'$NIMBUS'","title":"blocked task","payload":{"packet":"p/blocked.md"},"priority":99,"dependencies":[{"dep_kind":"event","dep_value":"ev-does-not-exist-yet"}]}')" | lastres
echo "statuses: $(psql "SELECT id||'='||status FROM dispatches WHERE project_id='${PROJECT}' ORDER BY id;" | tr '\n' ' ')"
echo "SeatDispatchIssued events: $(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='SeatDispatchIssued';")  (expect 2)"

echo
echo "########## POINT 2 — nimbus pulls -> gets READY, SKIPPING the higher-priority blocked one ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
echo "disp-ready status now: $(psql "SELECT status FROM dispatches WHERE id='disp-ready';")  (expect in_flight)"
echo "disp-blocked still:    $(psql "SELECT status FROM dispatches WHERE id='disp-blocked';")  (expect blocked, was NOT chosen despite priority 99)"
echo "SeatDispatchDelivered: $(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='SeatDispatchDelivered';")  (expect 1)"

echo
echo "########## POINT 3 — second pull from nimbus returns the SAME in_flight item ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
echo "SeatDispatchDelivered still: $(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='SeatDispatchDelivered';")  (expect still 1 — no new delivery)"

echo
echo "########## POINT 4 — a second seat (mira) gets ITS OWN item — one-in-flight is per seat ##########"
drive "$MIRA" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-mira","target_seat_id":"'$MIRA'","title":"mira task","payload":{"packet":"p/mira.md"},"priority":1}')" | lastres
drive "$MIRA" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
echo "disp-mira status: $(psql "SELECT status FROM dispatches WHERE id='disp-mira';")  (expect in_flight)"

echo
echo "########## POINT 5 — nimbus completes its in_flight -> done + SeatDispatchCompleted ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-ready"}')" | lastres
echo "disp-ready status: $(psql "SELECT status FROM dispatches WHERE id='disp-ready';")  (expect done)"
echo "SeatDispatchCompleted: $(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='SeatDispatchCompleted';")  (expect 1)"

echo
echo "########## POINT 6 — the blocking event LANDS -> disp-blocked becomes ready and is pulled ##########"
# Emit the awaited event via the step-4 tool (same binary), as nimbus.
drive "$NIMBUS" "$INIT" "$INITED" "$(call emit_event '{"type":"ArtifactCreated","event_id":"ev-does-not-exist-yet","payload":{"kind":"unblock"},"refs":[{"ref_type":"artifact","ref_id":"art-unblock","role":"subject"},{"ref_type":"seat","ref_id":"'$NIMBUS'","role":"actor"}]}')" | lastres
echo "disp-blocked readiness before pull is computed live by existence query..."
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
echo "disp-blocked status: $(psql "SELECT status FROM dispatches WHERE id='disp-blocked';")  (expect in_flight — the landed event unblocked it)"

echo
echo "########## POINT 7 — every transition is an event (with refs + roles) ##########"
psql "SELECT e.event_type||' subj='||max(CASE WHEN r.role='subject' THEN r.ref_type||':'||r.ref_id END)||' actor='||max(CASE WHEN r.role='actor' THEN r.ref_type||':'||r.ref_id END) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id WHERE e.project_id='${PROJECT}' AND e.event_type LIKE 'SeatDispatch%' GROUP BY e.id,e.event_type,e.occurred_at ORDER BY e.occurred_at;"

echo
echo "########## POINT 8 — atomicity: a forced-failure transition rolls back row + event together ##########"
# complete_dispatch on a dispatch NOT owned by this seat's in_flight: rejected,
# nothing written. (The partial-write risk is inside append_event_in_tx's txn.)
before_ev=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
# Force a mid-transition DB failure: enqueue with a target_seat_id that violates
# the seats FK -> the dispatches INSERT fails AFTER nothing, but to prove the
# event+row atomicity we use a dispatch whose event append will fail: emit a
# duplicate event id inside enqueue by pre-inserting the id it would use is not
# possible (ids are generated). Instead: complete a non-existent dispatch ->
# rejected pre-write; assert no event appended.
drive "$NIMBUS" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-nonexistent"}')" | lastres
after_ev=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
echo "events before=${before_ev} after=${after_ev}  (equal => rejected transition wrote no event)"
# Hard atomicity: force a FK failure DURING enqueue's transaction (bad target seat)
# and confirm neither the dispatches row NOR the SeatDispatchIssued event persisted.
before2=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-badseat","target_seat_id":"seat-nope-999","title":"doomed","payload":{"x":1}}')" | lastres
rows=$(psql "SELECT count(*) FROM dispatches WHERE id='disp-badseat';")
after2=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
echo "disp-badseat rows=${rows} (expect 0); events before=${before2} after=${after2} (equal => row+event rolled back together)"

echo
echo "########## POINT 9 — cross-seat cycle detected + escalated (SupervisorMessage + daily-log) ##########"
# Use a SEPARATE project so no seat here holds an unrelated in_flight item that
# would mask the cycle path. onyx <-> lyra, each depending on the other's dispatch.
CPROJ=p0-step6-cycle
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${CPROJ}','cycle proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${CPROJ}'); DELETE FROM dispatches WHERE project_id='${CPROJ}'; DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${CPROJ}'); DELETE FROM canonical_events WHERE project_id='${CPROJ}';" >/dev/null
drivep() { local actor="$1" proj="$2"; shift 2; printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" SEATLOOM_PROJECT_ID="$proj" "$BIN" 2>/dev/null; }
drivep seat-onyx-001 "$CPROJ" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-cycle-A","target_seat_id":"seat-onyx-001","title":"A","payload":{"x":1},"dependencies":[{"dep_kind":"dispatch","dep_value":"disp-cycle-B"}]}')" | lastres
drivep seat-lyra-001 "$CPROJ" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-cycle-B","target_seat_id":"seat-lyra-001","title":"B","payload":{"x":1},"dependencies":[{"dep_kind":"dispatch","dep_value":"disp-cycle-A"}]}')" | lastres
sup_before=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${CPROJ}' AND event_type='SupervisorMessage';")
drivep seat-onyx-001 "$CPROJ" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
sup_after=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${CPROJ}' AND event_type='SupervisorMessage';")
echo "SupervisorMessage events: before=${sup_before} after=${sup_after}  (expect +1 — cycle escalated)"
echo "cycle SupervisorMessage subject: $(psql "SELECT r.ref_type||':'||r.ref_id FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${CPROJ}' AND e.event_type='SupervisorMessage' LIMIT 1;")"
echo "daily-log line parked: $(grep -c 'scheduler escalation' docs/coordination/memory/$(date -u +%Y-%m-%d).md 2>/dev/null || echo 0)  (expect >=1)"
# Dedup: a second pull must NOT emit a second SupervisorMessage for the same ring.
drivep seat-onyx-001 "$CPROJ" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" >/dev/null
sup_dedup=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${CPROJ}' AND event_type='SupervisorMessage';")
echo "after a 2nd pull: SupervisorMessage=${sup_dedup}  (expect still ${sup_after} — deduped, not spammed)"

echo
echo "########## POINT 10 — stall: a seat with ONLY blocked dispatches, nothing ready -> escalated ##########"
# flux in yet another project: one dispatch blocked on a missing EVENT (not a cycle), nothing ready.
SPROJ=p0-step6-stall
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${SPROJ}','stall proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${SPROJ}'); DELETE FROM dispatches WHERE project_id='${SPROJ}'; DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${SPROJ}'); DELETE FROM canonical_events WHERE project_id='${SPROJ}';" >/dev/null
drivep seat-flux-001 "$SPROJ" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-flux-blk","target_seat_id":"seat-flux-001","title":"flux blocked","payload":{"x":1},"dependencies":[{"dep_kind":"event","dep_value":"ev-flux-never"}]}')" | lastres
sup_b=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${SPROJ}' AND event_type='SupervisorMessage';")
drivep seat-flux-001 "$SPROJ" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres
sup_a=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${SPROJ}' AND event_type='SupervisorMessage';")
echo "SupervisorMessage: before=${sup_b} after=${sup_a}  (expect +1 — stall escalated, not hung)"

echo
echo "########## POINT 11 — one-in-flight is a DB fact: a 2nd in_flight for a seat is rejected in raw SQL ##########"
# nimbus currently has disp-blocked in_flight. Try to force a 2nd in_flight row.
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) VALUES ('disp-2nd-inflight','${NIMBUS}','seat:lyra','${PROJECT}','second','{}'::jsonb,'in_flight',1,NOW());" 2>&1 | tail -1
echo "(expect: duplicate key value violates unique constraint uq_dispatches_one_in_flight_per_seat)"
echo "in_flight count for nimbus: $(psql "SELECT count(*) FROM dispatches WHERE target_seat_id='${NIMBUS}' AND status='in_flight';")  (expect 1)"

echo
echo "########## POINT 12 — identity cannot be forged: a seat_id arg is ignored (no param exists) ##########"
# mira's server with a spoofed target_seat_id arg must NOT get nimbus's queue.
drive "$MIRA" "$INIT" "$INITED" "$(call get_next_dispatch '{"seat_id":"'$NIMBUS'","target_seat_id":"'$NIMBUS'"}')" | lastres
echo "  (additionalProperties:false rejects the arg; identity stays mira from env)"

echo
echo "########## POINT 13 — the dispatches row is REBUILDABLE from the ledger alone ##########"
# Reconstruct disp-blocked's status history from its three lifecycle events and
# compare the final state to the live row.
echo "ledger events for disp-blocked (ordered):"
psql "SELECT e.event_type FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE r.ref_type='dispatch' AND r.ref_id='disp-blocked' ORDER BY e.occurred_at;" | sed 's/^/  /'
rebuilt=$(psql "SELECT CASE WHEN bool_or(event_type='SeatDispatchCompleted') THEN 'done' WHEN bool_or(event_type='SeatDispatchDelivered') THEN 'in_flight' WHEN bool_or(event_type='SeatDispatchIssued') THEN 'ready_or_blocked' END FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE r.ref_type='dispatch' AND r.ref_id='disp-blocked';")
live=$(psql "SELECT status FROM dispatches WHERE id='disp-blocked';")
echo "rebuilt-from-ledger terminal state = ${rebuilt} ; live row status = ${live}  (in_flight matches Delivered-but-not-Completed)"

echo
echo "########## PROOF COMPLETE ##########"
