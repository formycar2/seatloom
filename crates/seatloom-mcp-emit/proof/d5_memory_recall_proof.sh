#!/usr/bin/env bash
# Proof for D5 knowledge-layer Lane B (Nimbus) — the six MCP tools:
#   recall / remember / promote / forget / get_dispatch / query_ledger
# Packet: NIMBUS-2026-10-10-d5-mcp-recall-tools-v1 §5
# Value-domain contract: VALUE-DOMAIN-2026-10-10-knowledge-layer-v1 §4/§5/§9/§10
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# (schema 001..014 + 015 knowledge layer + 016 acceptance_plan + 017 acceptance
# cancel), checking the memories projection AND canonical_events via psql in the
# podman postgres container. NEVER the live seatloom DB.
#
# Identity is per-server-env (SEATLOOM_ACTOR_REF) — the non-forgeable model.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_d5}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
BIN="$(git rev-parse --show-toplevel)/target/debug/seatloom-mcp-emit"
PROJECT=d5-lane-b-proof
NIMBUS=seat-nimbus-001

[[ "$DB" == "seatloom" ]] && { echo "REFUSING live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }

drive() {
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" \
    SEATLOOM_PROJECT_ID="$PROJECT" SEATLOOM_INTEGRATION_BRANCH="track/infra-foundation" \
    "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
lastres() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR " if r["isError"] else "OK  ")+r["content"][0]["text"])'; }
count() { psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='$1';"; }

echo "target database = ${DB}  (schema 001..014 + 015 + 016 + 017)"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','d5 lane b proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM memories WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM topics WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM dispatches WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null

# Fixture: a nested topic pair (parent -> child) so the ancestor chain is real.
psql "INSERT INTO topics (id,project_id,title,status,opened_by) VALUES ('d5-proof-topic-parent','${PROJECT}','parent topic','open','seat:seat-lyra-001') ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO topics (id,project_id,title,status,opened_by,parent_topic_id) VALUES ('d5-proof-topic-child','${PROJECT}','child topic','open','seat:seat-lyra-001','d5-proof-topic-parent') ON CONFLICT (id) DO NOTHING;" >/dev/null
echo "fixture: project ${PROJECT}, nested topics parent -> child"

echo
echo "########## POINT 1 — remember: the row AND MemoryCreated, one transaction ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"d5-proof-topic-child","title":"how the gate works","l0_summary":"accept needs flux green plus the operator","l1_overview":"the long version","l2_ref_kind":"file","l2_file_path":"docs/gate.md","l2_body_digest":"deadbeef","source_kind":"document","source_ref":"doc-gate","tags":["d2"]}')" | lastres
MEM=$(psql "SELECT id FROM memories WHERE project_id='${PROJECT}' ORDER BY created_at LIMIT 1;")
echo "memory id: ${MEM}  (expect mem-*)"
echo "row: tier=$(psql "SELECT tier FROM memories WHERE id='${MEM}';") status=$(psql "SELECT status FROM memories WHERE id='${MEM}';") heat=$(psql "SELECT heat FROM memories WHERE id='${MEM}';")  (expect cold / active / 0)"
echo "MemoryCreated events: $(count MemoryCreated)  (expect 1)"
echo "subject ref: $(psql "SELECT ref_type||' '||ref_id||' role='||role FROM event_object_refs r JOIN canonical_events e ON e.id=r.event_id WHERE e.event_type='MemoryCreated' AND r.role='subject' AND e.project_id='${PROJECT}';")  (expect memory ${MEM} role=subject)"
echo "actor ref:   $(psql "SELECT ref_id FROM event_object_refs r JOIN canonical_events e ON e.id=r.event_id WHERE e.event_type='MemoryCreated' AND r.role='actor' AND e.project_id='${PROJECT}';")  (expect ${NIMBUS})"

echo
echo "########## POINT 2 — recall at l0: only the small layer, and the audit moves the row ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call recall '{"topic_id":"d5-proof-topic-child"}')" | tail -1 | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
l = json.loads(r["content"][0]["text"])
print("depth:", l["depth"], "(expect l0)")
print("topic chain:", [t["id"] for t in l["topic_chain"]], "(expect child then parent)")
m = l["memories"][0]
print("memory fields:", sorted(m.keys()))
assert "l0Summary" in m, "l0 is always returned"
assert "l1Overview" not in m, "l1 omitted at l0"
assert "l2RefKind" not in m, "l2 omitted at l0"
print("recalled:", l["recalled"], "(expect 1)")
'
echo "row after recall: heat=$(psql "SELECT heat FROM memories WHERE id='${MEM}';") recall_count=$(psql "SELECT recall_count FROM memories WHERE id='${MEM}';") last_recalled=$(psql "SELECT (last_recalled_at IS NOT NULL) FROM memories WHERE id='${MEM}';")  (expect 1 / 1 / t)"
echo "MemoryRecalled events: $(count MemoryRecalled)  (expect 1 — every recall is audited)"

echo
echo "########## POINT 3 — l1 / l2 disclose only on explicit depth ##########"
for d in l1 l2; do
  ARGS=$(printf '{"topic_id":"d5-proof-topic-child","depth":"%s"}' "$d")
  drive "$NIMBUS" "$INIT" "$INITED" "$(call recall "$ARGS")" | tail -1 | python3 -c "
import sys, json
r = json.load(sys.stdin)['result']
l = json.loads(r['content'][0]['text'])
m = l['memories'][0]
print('$d fields:', sorted(m.keys()))
"
done

echo
echo "########## POINT 4 — the negative matrix: each a typed refusal, zero writes ##########"
BEFORE=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
reject() { drive "$NIMBUS" "$INIT" "$INITED" "$(call "$1" "$2")" | lastres; }
reject recall       '{"topic_id":"d5-proof-topic-child","depth":"l3"}'
reject recall       '{"topic_id":"d5-proof-topic-child","limit":101}'
reject remember     '{"topic_id":"d5-proof-topic-child","title":"t","l0_summary":"s","l2_ref_kind":"file","source_kind":"manual","source_ref":"n"}'
reject remember     '{"topic_id":"d5-proof-topic-child","title":"t","l0_summary":"s","l2_ref_kind":"document","l2_document_id":"d","l2_file_path":"docs/x.md","source_kind":"manual","source_ref":"n"}'
reject remember     '{"topic_id":"d5-proof-topic-child","title":"t","l0_summary":"s","l2_ref_kind":"file","l2_file_path":"docs/x.md","source_kind":"vibes","source_ref":"n"}'
reject promote      "{\"memory_id\":\"${MEM}\",\"to_tier\":\"tepid\"}"
reject promote      "{\"memory_id\":\"${MEM}\",\"to_tier\":\"cold\"}"
reject forget       '{"memory_id":"mem-does-not-exist","reason":"r"}'
reject forget       "{\"memory_id\":\"${MEM}\",\"reason\":\"   \"}"
reject get_dispatch '{"dispatch_id":"disp-nope"}'
reject query_ledger '{"ref_type":"dispatch"}'
reject query_ledger '{"event_types":["SeatDispatchReopened"]}'
AFTER=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
echo "canonical_events: before=${BEFORE} after=${AFTER}  (expect EQUAL — every refusal writes nothing)"
echo "memories rows: $(psql "SELECT count(*) FROM memories WHERE project_id='${PROJECT}';")  (expect 1 — no refusal created one)"

echo
echo "########## POINT 5 — promote up, then down: the only tier channel ##########"
# The arguments are built in a variable: an inline escaped-quote JSON is
# word-split by the shell before the tool ever sees it.
for tier in warm hot cold; do
  ARGS=$(printf '{"memory_id":"%s","to_tier":"%s"}' "$MEM" "$tier")
  drive "$NIMBUS" "$INIT" "$INITED" "$(call promote "$ARGS")" | lastres
  echo "tier now: $(psql "SELECT tier FROM memories WHERE id='${MEM}';")  (expect ${tier})"
done
echo "MemoryPromoted payloads: $(psql "SELECT concat(payload->>'from_tier','->',payload->>'to_tier') FROM canonical_events WHERE event_type='MemoryPromoted' AND project_id='${PROJECT}' ORDER BY occurred_at;" | tr '\n' ' ')  (expect cold->warm warm->hot hot->cold)"

echo
echo "########## POINT 6 — forget: soft tombstone, row stays, second forget refused ##########"
ARGS=$(printf '{"memory_id":"%s","reason":"superseded by a newer note"}' "$MEM")
drive "$NIMBUS" "$INIT" "$INITED" "$(call forget "$ARGS")" | lastres
echo "row still present: $(psql "SELECT status||' / '||coalesce(forgotten_reason,'-') FROM memories WHERE id='${MEM}';")  (expect forgotten / superseded by a newer note)"
echo "MemoryForgotten events: $(count MemoryForgotten)  (expect 1)"
ARGS=$(printf '{"memory_id":"%s","reason":"again"}' "$MEM")
drive "$NIMBUS" "$INIT" "$INITED" "$(call forget "$ARGS")" | lastres
ARGS='{"topic_id":"d5-proof-topic-child"}'
drive "$NIMBUS" "$INIT" "$INITED" "$(call recall "$ARGS")" | tail -1 | python3 -c '
import sys, json
l = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
print("recall after forget:", len(l["memories"]), "memories,", l["recalled"], "recalled  (expect 0, 0 — zero hits, zero events)")
assert len(l["memories"]) == 0 and l["recalled"] == 0
'

echo
echo "########## POINT 7 — get_dispatch / query_ledger: no seat needs psql again ##########" ##########"
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
        VALUES ('disp-d5-proof','${NIMBUS}','seat:seat-lyra-001','${PROJECT}','d5 proof task','{\"packet\":\"p/d5.md\"}','ready',5,NOW());" >/dev/null
psql "INSERT INTO dispatch_dependencies (dispatch_id,dep_kind,dep_value) VALUES ('disp-d5-proof','event','ev-not-there');" >/dev/null
ARGS='{"dispatch_id":"disp-d5-proof"}'
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_dispatch "$ARGS")" | tail -1 | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
d = json.loads(r["content"][0]["text"])
print("dispatch:", d["title"], "| stored status:", d["status"], "| live status:", d["liveStatus"])
print("dependencies:", [(x["depKind"], x["exists"]) for x in d["dependencies"]])
assert d["liveStatus"] == "blocked", "a missing dependency reads as blocked"
assert d["dependencies"][0]["depKind"] == "event"
assert d["dependencies"][0]["exists"] is False
'
ARGS=$(printf '{"project_id":"%s","event_types":["MemoryCreated","MemoryRecalled","MemoryPromoted","MemoryForgotten"],"limit":100}' "$PROJECT")
drive "$NIMBUS" "$INIT" "$INITED" "$(call query_ledger "$ARGS")" | tail -1 | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
evs = json.loads(r["content"][0]["text"])
print("ledger events:", [(e["eventType"], len(e["refs"])) for e in evs])
assert all(e["refs"] for e in evs), "each event carries its refs"
'
ARGS=$(printf '{"ref_type":"memory","ref_id":"%s"}' "$MEM")
drive "$NIMBUS" "$INIT" "$INITED" "$(call query_ledger "$ARGS")" | tail -1 | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
evs = json.loads(r["content"][0]["text"])
print("events naming the memory:", [e["eventType"] for e in evs])
assert len(evs) >= 4, "created + recalls + promotes + forgotten all name the memory"
'

echo
echo "########## POINT 8 — tools/list: six new tools, none exposing identity ##########" ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 | python3 -c '
import sys, json
t = json.load(sys.stdin)["result"]["tools"]
print("tools:", [x["name"] for x in t])
for name in ["recall","remember","promote","forget","get_dispatch","query_ledger"]:
    tool = [x for x in t if x["name"] == name][0]
    props = tool["inputSchema"]["properties"]
    assert not any("seat" in k for k in props), f"{name} must not expose identity"
print("no seat/actor parameter on any of the six — identity is server-side")
'

echo
echo "proof complete: memory lifecycle audited end to end, progressive disclosure per depth, every negative branch a typed refusal with zero writes, tier moves explicit only, forget is a soft tombstone, and the two reads replace psql."
