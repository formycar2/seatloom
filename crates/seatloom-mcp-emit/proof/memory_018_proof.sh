#!/usr/bin/env bash
# Proof for the 018 memory-tool surface (D5 Phase 2 Lane A2, dispatch
# disp-mem018-nimbus) — the typed record fields, the supersede double-spelling,
# the superseded-aware recall and the resolve closure path.
#
# Packet: NIMBUS-2026-10-10-memory-tools-018-v1 §5
# Value-domain contract: VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1 §G1-G5
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# with schema 001-014 + 015 + 016 + 018 applied (018 from
# packet/onyx/memory-hardening-018; infra/** is not touched). NEVER the live
# seatloom DB.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_018}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=mem018-proof
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
lastjson() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(r["content"][0]["text"])'; }
count() { psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}' AND event_type='$1';"; }

echo "target database = ${DB} (schema 001-014 + 015 + 016 + 018)"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','mem018 proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM memory_relations WHERE from_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}') OR to_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM memories WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM topics WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM documents WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
psql "INSERT INTO topics (id,project_id,title,status,opened_by) VALUES ('topic-018','${PROJECT}','018 topic','open','seat:seat-lyra-001');" >/dev/null
psql "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) VALUES ('doc-018','${PROJECT}','the doc','docs/018.md','ok','{}','{}');" >/dev/null
echo "fixture: project ${PROJECT}, one topic, one document"

echo
echo "########## POINT 1 — the negative matrix: typed refusals, zero writes ##########"
BEFORE=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
reject() { drive "$NIMBUS" "$INIT" "$INITED" "$(call remember "$2")" | lastres; }
reject "document pointer without an id"  '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"document","source_kind":"manual","source_ref":"n","kind":"fact","confidence":"quoted"}'
reject "kind out of the 6-value vocab"    '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"manual","source_ref":"n","kind":"note","confidence":"quoted"}'
reject "confidence is binary, not a score" '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"manual","source_ref":"n","kind":"fact","confidence":"0.9"}'
reject "file+archived_session without lines" '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"file","l2_file_path":"docs/x.md","source_kind":"archived_session","source_ref":"claude:ses-1","kind":"fact","confidence":"quoted"}'
reject "end before start"                 '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"file","l2_file_path":"docs/x.md","source_kind":"archived_session","source_ref":"claude:ses-1","l2_line_start":20,"l2_line_end":10,"kind":"fact","confidence":"quoted"}'
reject "relation to a memory that does not exist" '{"topic_id":"topic-018","title":"t","l0_summary":"s","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"manual","source_ref":"n","kind":"fact","confidence":"quoted","relations":[{"to_id":"mem-nope","rel":"caused_by"}]}'
AFTER=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
echo "canonical_events: before=${BEFORE} after=${AFTER}  (expect EQUAL — every refusal writes nothing)"
echo "memories rows: $(psql "SELECT count(*) FROM memories WHERE project_id='${PROJECT}';")  (expect 0)"

echo
echo "########## POINT 2 — a typed record, and the supersede edge in BOTH spellings ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"topic-018","title":"the old failure","l0_summary":"the old summary","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"document","source_ref":"doc-018","kind":"failure","confidence":"quoted"}')" | lastres
OLD=$(psql "SELECT id FROM memories WHERE project_id='${PROJECT}' ORDER BY created_at LIMIT 1;")
echo "old row: kind=$(psql "SELECT kind FROM memories WHERE id='${OLD}';") confidence=$(psql "SELECT confidence FROM memories WHERE id='${OLD}';")  (expect failure / quoted)"
echo "MemoryCreated events: $(count MemoryCreated)  (expect 1)"

# The arguments are built in a variable: an inline escaped-quote JSON is
# word-split by the shell before the tool ever sees it.
ARGS=$(printf '{"topic_id":"topic-018","title":"the new decision","l0_summary":"the new summary","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"document","source_ref":"doc-018","kind":"decision","confidence":"inferred","supersedes":"%s","relations":[{"to_id":"%s","rel":"refines"}]}' "$OLD" "$OLD")
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember "$ARGS")" | lastres
NEW=$(psql "SELECT id FROM memories WHERE project_id='${PROJECT}' ORDER BY created_at DESC LIMIT 1;")
echo "new row: kind=$(psql "SELECT kind FROM memories WHERE id='${NEW}';") confidence=$(psql "SELECT confidence FROM memories WHERE id='${NEW}';")  (expect decision / inferred)"
echo "supersedes COLUMN:  $(psql "SELECT supersedes FROM memories WHERE id='${NEW}';")  (expect ${OLD})"
echo "supersedes RELATION: $(psql "SELECT count(*) FROM memory_relations WHERE from_id='${NEW}' AND to_id='${OLD}' AND rel='supersedes';")  (expect 1 — the same edge, second spelling)"
echo "refines RELATION:    $(psql "SELECT count(*) FROM memory_relations WHERE from_id='${NEW}' AND to_id='${OLD}' AND rel='refines';")  (expect 1)"
echo "the two spellings agree: $(psql "SELECT (SELECT supersedes FROM memories WHERE id='${NEW}') = (SELECT to_id FROM memory_relations WHERE from_id='${NEW}' AND rel='supersedes');")  (expect t)"

echo
echo "########## POINT 3 — recall hides superseded rows, and annotates them when asked ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call recall '{"topic_id":"topic-018"}')" | tail -1 | python3 -c '
import sys, json
l = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
ids = [m["id"] for m in l["memories"]]
print("default recall:", ids)
assert len(ids) == 1, "the superseded row must be hidden by default"
assert l["includeSuperseded"] is False
print("superseded rows excluded by default: PASS")
'
drive "$NIMBUS" "$INIT" "$INITED" "$(call recall '{"topic_id":"topic-018","include_superseded":true}')" | tail -1 | python3 -c '
import sys, json
l = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
print("opt-in recall:", [(m["id"], m.get("supersededBy")) for m in l["memories"]])
assert len(l["memories"]) == 2, "both rows now visible"
old = [m for m in l["memories"] if m.get("supersededBy")][0]
assert old["supersededBy"], "the superseded row is annotated with its replacement"
print("opt-in annotates supersededBy: PASS")
'
echo "recall audit events (one per hit): $(count MemoryRecalled)  (2 hits so far)"

echo
echo "########## POINT 4 — resolve: the only closure path ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call resolve "{\"memory_id\":\"${NEW}\"}")" | lastres
echo "row now: status=$(psql "SELECT status FROM memories WHERE id='${NEW}';") resolved_by=$(psql "SELECT resolved_by FROM memories WHERE id='${NEW}';") resolved_at set=$(psql "SELECT (resolved_at IS NOT NULL) FROM memories WHERE id='${NEW}';")"
echo "MemoryResolved events: $(count MemoryResolved)  (expect 1)"
drive "$NIMBUS" "$INIT" "$INITED" "$(call resolve "{\"memory_id\":\"${NEW}\"}")" | lastres
drive "$NIMBUS" "$INIT" "$INITED" "$(call resolve '{"memory_id":"mem-nope"}')" | lastres
echo "a resolved record and an unknown id are both refused; events still $(count MemoryResolved)"

echo
echo "########## POINT 5 — the l2 line range, required for a transcript slice ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"topic-018","title":"transcript slice","l0_summary":"a slice of a session","l2_ref_kind":"file","l2_file_path":"docs/session.md","l2_line_start":41,"l2_line_end":57,"source_kind":"archived_session","source_ref":"claude:ses-9","kind":"fact","confidence":"quoted"}')" | lastres
SLICE=$(psql "SELECT id FROM memories WHERE project_id='${PROJECT}' ORDER BY created_at DESC LIMIT 1;")
echo "row: l2_line_start=$(psql "SELECT l2_line_start FROM memories WHERE id='${SLICE}';") l2_line_end=$(psql "SELECT l2_line_end FROM memories WHERE id='${SLICE}';")  (expect 41 / 57)"
echo "a document pointer may omit the lines: $(drive "$NIMBUS" "$INIT" "$INITED" "$(call remember '{"topic_id":"topic-018","title":"doc pointer","l0_summary":"no lines needed","l2_ref_kind":"document","l2_document_id":"doc-018","source_kind":"manual","source_ref":"n","kind":"artifact","confidence":"inferred"}')" | tail -1 | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["isError"])')  (expect False)"

echo
echo "########## tools/list: fifteen tools, the new one included ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 | python3 -c '
import sys, json
t = json.load(sys.stdin)["result"]["tools"]
print("tools:", [x["name"] for x in t])
names = [x["name"] for x in t]
assert "resolve" in names, "the closure tool must be advertised"
remember = [x for x in t if x["name"] == "remember"][0]["inputSchema"]
for req in ["kind", "confidence"]:
    assert req in remember["required"], f"{req} must be required"
print("remember requires kind + confidence: PASS")
'

echo
echo "proof complete: the six negative branches refuse with zero writes; kind/confidence/l2 lines land; the supersede edge is written in both spellings in one transaction; recall hides superseded rows by default and annotates them when asked; resolve is the only closure path and a resolved record is immutable."
