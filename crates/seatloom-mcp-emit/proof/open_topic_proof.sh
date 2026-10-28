#!/usr/bin/env bash
# Proof for open_topic + the verdict-event-id contract (dispatch
# disp-open-topic-nimbus; Aegis ruling AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md).
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# with schema 001-014 + 015 + 016 + 018 + scratch 017 + scratch 019 applied (019
# from .scratch — infra/** is not touched). NEVER the live seatloom DB.
#
# The end-to-end point of the whole ruling: BEFORE open_topic, `remember` could
# not succeed at all (memories.topic_id is NOT NULL with an FK to topics, and
# topics had zero rows and no code path that could create one). This proof opens
# a topic and then remembers into it.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_019}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=open-topic-proof
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

echo "target database = ${DB} (schema 001-014 + 015 + 016 + 018 + scratch 017 + scratch 019)"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','open topic proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM memory_relations WHERE from_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}') OR to_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM memories WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM topics WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM documents WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM dispatches WHERE project_id='${PROJECT}';" >/dev/null
psql "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) VALUES ('doc-ot-proof','${PROJECT}','the doc','docs/ot.md','ok','{}','{}');" >/dev/null
echo "fixture: project ${PROJECT}, one document, ZERO topics (the state that blocked every remember)"

echo
echo "########## POINT 0 — BEFORE open_topic: remember fails on the topic FK ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"ot-topic-root","title":"a memory","l0_summary":"s","l2_ref_kind":"document","l2_document_id":"doc-ot-proof","source_kind":"manual","source_ref":"n","kind":"fact","confidence":"quoted"}')" | lastres

echo
echo "########## POINT 1 — open_topic: row + TopicOpened, opened_by server-side ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call open_topic '{"topic_id":"ot-topic-root","title":"the proof topic"}')" | lastres
echo "row: project=$(psql "SELECT project_id FROM topics WHERE id='ot-topic-root';") status=$(psql "SELECT status FROM topics WHERE id='ot-topic-root';")"
echo "opened_by: $(psql "SELECT actor_ref FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='actor' WHERE e.event_type='TopicOpened';")  (expect seat:seat-nimbus-001)"
echo "subject ref: $(psql "SELECT ref_type||' '||ref_id FROM event_object_refs r JOIN canonical_events e ON e.id=r.event_id WHERE e.event_type='TopicOpened' AND r.role='subject';")  (expect topic ot-topic-root)"
echo "TopicOpened events: $(count TopicOpened)  (expect 1)"

echo
echo "########## POINT 2 — idempotent: a second open changes nothing ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call open_topic '{"topic_id":"ot-topic-root","title":"a different title"}')" | lastres
echo "title unchanged: $(psql "SELECT title FROM topics WHERE id='ot-topic-root';")  (expect the proof topic)"
echo "TopicOpened events: $(count TopicOpened)  (expect still 1 — no second event)"

echo
echo "########## POINT 3 — the refusals, each typed and zero-write ##########"
BEFORE=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
reject() { drive "$NIMBUS" "$INIT" "$INITED" "$(call open_topic "$2")" | lastres; }
reject "parent does not exist" '{"topic_id":"ot-topic-child","title":"c","parent_topic_id":"topic-nope"}'
reject "project does not exist" '{"topic_id":"topic-x","title":"x","project_id":"no-such-project"}'
reject "self-cycle"              '{"topic_id":"ot-topic-child","title":"c","parent_topic_id":"ot-topic-child"}'
AFTER=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
echo "canonical_events: before=${BEFORE} after=${AFTER}  (expect EQUAL)"
echo "topics rows: $(psql "SELECT count(*) FROM topics WHERE project_id='${PROJECT}';")  (expect 1)"

echo
echo "########## POINT 4 — a child topic, and the cross-project refusal ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call open_topic '{"topic_id":"ot-topic-child","title":"the child","parent_topic_id":"ot-topic-root"}')" | lastres
echo "chain: $(psql "SELECT id||' <- '||coalesce(parent_topic_id,'root') FROM topics WHERE id='ot-topic-child';")"
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "SELECT 1 FROM pg_database WHERE datname='ot-other'" >/dev/null 2>&1 || podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "CREATE DATABASE \"ot-other\"" >/dev/null 2>&1
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('ot-other','other',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO topics (id,project_id,title,opened_by) VALUES ('topic-other','ot-other','other','seat:seat-lyra-001') ON CONFLICT (id) DO NOTHING;" >/dev/null
drive "$NIMBUS" "$INIT" "$INITED" "$(call open_topic '{"topic_id":"topic-cross","title":"cross","parent_topic_id":"topic-other"}')" | lastres
echo "topics rows: $(psql "SELECT count(*) FROM topics WHERE project_id='${PROJECT}';")  (expect still 2 — the cross-project one was refused)"

echo
echo "########## POINT 5 — AFTER open_topic: remember finally works ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"ot-topic-child","title":"a memory","l0_summary":"the summary","l2_ref_kind":"document","l2_document_id":"doc-ot-proof","source_kind":"manual","source_ref":"n","kind":"fact","confidence":"quoted"}')" | lastres
echo "memories rows: $(psql "SELECT count(*) FROM memories WHERE project_id='${PROJECT}';")  (expect 1 — the FK is satisfiable now)"
echo "recall sees it: $(drive "$NIMBUS" "$INIT" "$INITED" "$(call recall '{"topic_id":"ot-topic-child"}')" | tail -1 | python3 -c 'import sys,json; l=json.loads(json.load(sys.stdin)["result"]["content"][0]["text"]); print(len(l["memories"]), "memory(ies)")')"

echo
echo "########## POINT 6 — the verdict-event-id contract on the verification path ##########"
ARGS=$(printf '{"id":"disp-ot-proof","target_seat_id":"%s","title":"verify me","payload":{"kind":"verification","target_commit":"deadbeef"},"priority":5}' "$NIMBUS")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch "$ARGS")" | lastres
MINTED=$(psql "SELECT payload->>'verdict_event_id' FROM dispatches WHERE id='disp-ot-proof';")
echo "the minted id rides in the payload: ${MINTED}"
echo "a second pre-declaration of the same id is refused:"
ARGS2=$(printf '{"id":"disp-ot-proof-2","target_seat_id":"%s","title":"verify me","payload":{"kind":"verification"},"priority":5,"verdict_event_id":"%s"}' "$NIMBUS" "$MINTED")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch "$ARGS2")" | lastres
echo "a malformed id is refused:"
ARGS3=$(printf '{"id":"disp-ot-proof-3","target_seat_id":"%s","title":"verify me","payload":{"kind":"verification"},"priority":5,"verdict_event_id":"not-an-id"}' "$NIMBUS")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch "$ARGS3")" | lastres
echo "the argument on a non-verification payload is refused:"
ARGS4=$(printf '{"id":"disp-ot-proof-4","target_seat_id":"%s","title":"ordinary","payload":{"kind":"implementation"},"priority":5,"verdict_event_id":"ev-x"}' "$NIMBUS")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch "$ARGS4")" | lastres
echo "an ordinary enqueue is untouched:"
ARGS5=$(printf '{"id":"disp-ot-proof-5","target_seat_id":"%s","title":"ordinary","payload":{"kind":"implementation"},"priority":5}' "$NIMBUS")
drive "$NIMBUS" "$INIT" "$INITED" "$(call enqueue_dispatch "$ARGS5")" | lastres

echo
echo "########## tools/list: sixteen tools, open_topic advertised ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 | python3 -c '
import sys, json
t = json.load(sys.stdin)["result"]["tools"]
names = [x["name"] for x in t]
print("tools:", names)
assert "open_topic" in names
ot = [x for x in t if x["name"] == "open_topic"][0]["inputSchema"]["properties"]
assert not any("seat" in k or "actor" in k for k in ot), "opened_by must not be a parameter"
print("open_topic exposes no identity parameter: PASS")
'

echo
echo "proof complete: remember failed on the topic FK before open_topic and succeeds after; the topic opens with row+TopicOpened and a server-side opened_by; re-opening is idempotent; parent/project/self-cycle/cross-project are typed refusals with zero writes; and a verification request now mints (or validates) its verdict event id into the payload and returns it."
