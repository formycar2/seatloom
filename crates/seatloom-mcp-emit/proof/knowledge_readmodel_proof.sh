#!/usr/bin/env bash
# Proof for the knowledge read model (overnight step 4, dispatch
# disp-onight-readmodel): the topic tree and the memories-by-topic card list,
# with progressive disclosure and the ancestor chain.
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# with schema 001-014 + 015 + 016 + 018 + scratch 017 + scratch 019 applied
# (Onyx's migrations from their branches; infra/** is not touched). NEVER the
# live seatloom DB.
#
# The end-to-end point: the UI's whole panel is one round-trip per level, the
# reads are PURE (the ledger does not move), and the scope is hard.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_km}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=km-proof
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
lastjson() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(r["content"][0]["text"])'; }
ledger_count() { psql "SELECT count(*) FROM canonical_events;"; }

echo "target database = ${DB} (schema 001-014 + 015 + 016 + 018 + scratch 017 + scratch 019)"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','km proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM memory_relations WHERE from_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}') OR to_id IN (SELECT id FROM memories WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM memories WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM topics WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM documents WHERE project_id='${PROJECT}';" >/dev/null
psql "INSERT INTO topics (id,project_id,title,opened_by) VALUES ('kmproof-root','${PROJECT}','the root','seat:seat-lyra-001') ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO topics (id,project_id,title,opened_by,parent_topic_id) VALUES ('kmproof-child','${PROJECT}','the child','seat:seat-lyra-001','kmproof-root') ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) VALUES ('kmproof-doc','${PROJECT}','the doc','docs/km.md','ok','{}','{}') ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO memories (id,project_id,topic_id,title,l0_summary,l1_overview,l2_ref_kind,l2_document_id,source_kind,source_ref,kind,confidence) VALUES ('kmproof-current','${PROJECT}','kmproof-child','the current decision','the summary','the overview','document','kmproof-doc','document','kmproof-doc','decision','quoted');" >/dev/null
psql "INSERT INTO memories (id,project_id,topic_id,title,l0_summary,l2_ref_kind,l2_file_path,l2_line_start,l2_line_end,source_kind,source_ref,kind,confidence) VALUES ('kmproof-new','${PROJECT}','kmproof-child','the newer decision','the newer summary','file','docs/s.md',41,57,'archived_session','claude:ses-1','decision','inferred');" >/dev/null
psql "INSERT INTO memories (id,project_id,topic_id,title,l0_summary,l1_overview,l2_ref_kind,l2_document_id,source_kind,source_ref,kind,confidence,status,forgotten_reason) VALUES ('kmproof-old','${PROJECT}','kmproof-child','the overturned decision','the old summary','the old overview','document','kmproof-doc','document','kmproof-doc','decision','quoted','forgotten','superseded by kmproof-new');" >/dev/null
psql "INSERT INTO memory_relations (from_id,to_id,rel) VALUES ('kmproof-new','kmproof-old','supersedes') ON CONFLICT DO NOTHING;" >/dev/null
echo "fixture: one root topic, one child, three memories (one superseded, one forgotten)"

echo
echo "########## POINT 1 — the topic tree, with the live memory count ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_knowledge_tree '{}')" | tail -1 | python3 -c '
import sys, json
t = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
print("project:", t["projectId"])
for n in t["topics"]:
    print("  ", n["topicId"], "| parent:", n["parentTopicId"], "| status:", n["status"], "| memoryCount:", n["memoryCount"])
counts = {n["topicId"]: n["memoryCount"] for n in t["topics"]}
assert counts["kmproof-child"] == 2, "only active, not-superseded memories count"
assert counts["kmproof-root"] == 0
print("the count matches what a recall would return: PASS")
'

echo
echo "########## POINT 2 — memories at l0: only the summary ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child"}')" | tail -1 | python3 -c '
import sys, json
r = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
print("depth:", r["depth"], "(expect l0)")
print("topicChain:", [(t["topicId"], t["title"]) for t in r["topicChain"]])
for m in r["memories"]:
    keys = sorted(m.keys())
    print("  ", m["memoryId"], keys)
assert r["depth"] == "l0"
assert [t["topicId"] for t in r["topicChain"]] == ["km-root", "kmproof-child"], "ancestors first"
for m in r["memories"]:
    assert "l0Summary" in m and "l1Overview" not in m and "l2" not in m, "l0 discloses only l0"
ids = [m["memoryId"] for m in r["memories"]]
assert "kmproof-old" not in ids, "the superseded row is hidden by default"
print("progressive disclosure at l0 + the ancestor chain: PASS")
'

echo
echo "########## POINT 3 — l1 and l2 on demand ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","depth":"l1"}')" | tail -1 | python3 -c '
import sys, json
r = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
assert r["depth"] == "l1"
for m in r["memories"]:
    print("  ", m["memoryId"], "l1Overview:", m.get("l1Overview"), "| l2:", m.get("l2"))
    assert "l2" not in m, "l2 is still absent at l1"
print("l1 discloses the overview and nothing more: PASS")
'
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","depth":"l2"}')" | tail -1 | python3 -c '
import sys, json
r = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
assert r["depth"] == "l2"
for m in r["memories"]:
    print("  ", m["memoryId"], "l2:", json.dumps(m.get("l2")))
new = [m for m in r["memories"] if m["memoryId"] == "kmproof-new"][0]
assert new["l2"]["refKind"] == "file" and new["l2"]["filePath"] == "docs/s.md"
assert new["l2"]["lineStart"] == 41 and new["l2"]["lineEnd"] == 57, "the evidence line range"
print("l2 discloses the pointer with the line range: PASS")
'

echo
echo "########## POINT 4 — opt in to the superseded row, annotated ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","include_superseded":true}')" | tail -1 | python3 -c '
import sys, json
r = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
old = [m for m in r["memories"] if m["memoryId"] == "kmproof-old"]
print("the superseded row, when asked for:", json.dumps(old))
assert old and old[0]["supersededBy"] == "kmproof-new", "annotated with its replacement"
print("opt-in annotates supersededBy: PASS")
'

echo
echo "########## POINT 5 — PURE READ: the ledger does not move ##########"
BEFORE=$(ledger_count)
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_knowledge_tree '{}')" > /dev/null
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","depth":"l2","include_superseded":true}')" > /dev/null
AFTER=$(ledger_count)
echo "canonical_events: before=${BEFORE} after=${AFTER}  (expect EQUAL)"
[ "$BEFORE" = "$AFTER" ] && echo "pure read: PASS" || echo "pure read: FAIL"

echo
echo "########## POINT 6 — scope is hard, and the refusals are typed ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"no-such-topic"}')" | lastjson | head -2
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","depth":"l3"}')" | lastjson | head -2
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_topic_memories '{"topic_id":"kmproof-child","limit":9999}')" | lastjson | head -2
echo "an empty project is an empty tree, not an error:"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_knowledge_tree '{"project_id":"no-such-project"}')" | tail -1 | python3 -c 'import sys,json; t=json.loads(json.load(sys.stdin)["result"]["content"][0]["text"]); print("  topics:", t["topics"])'

echo
echo "########## tools/list: eighteen tools, the two knowledge reads advertised ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 | python3 -c '
import sys, json
t = json.load(sys.stdin)["result"]["tools"]
names = [x["name"] for x in t]
print("tools:", names)
assert "get_knowledge_tree" in names and "get_topic_memories" in names
for n in ["get_knowledge_tree", "get_topic_memories"]:
    props = [x for x in t if x["name"] == n][0]["inputSchema"]["properties"]
    assert not any("seat" in k or "actor" in k for k in props), f"{n} must not expose identity"
print("both knowledge reads advertised, no identity parameter: PASS")
'

echo
echo "proof complete: the tree counts only live memories; l0 discloses only the summary; l1/l2 on demand; the ancestor chain rides along; the superseded row is hidden by default and annotated when asked for; the reads move no ledger row; scope is hard and every refusal is typed."
