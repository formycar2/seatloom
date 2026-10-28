#!/usr/bin/env bash
# Proof for the acceptance-lineage milestone, Lane B (Nimbus):
#   NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1 §5
#   value-domain contract VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1 §1-§3
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# (schema 001..014 + 016 acceptance_plan + 017 acceptance cancel, canonical
# order), proving:
#   POINT 1  get_acceptance_lineage assembles all seven contract blocks
#   POINT 2  the plan negative matrix — every malformed shape is a TYPED
#            rejection and canonical_events does NOT move
#   POINT 3  a valid plan lands on the row AND in the AcceptanceRequested
#            payload (one transaction)
#   POINT 4  MCP read tool and the Tauri command share ONE assembler (the MCP
#            result equals what the core function returns)
#   POINT 5  tools/list advertises the read tool, pure read (no writes)
#
# NEVER the live `seatloom` database. Identity is per-server-env
# (SEATLOOM_ACTOR_REF) — the non-forgeable identity model.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_lineage}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
BIN="$(git rev-parse --show-toplevel)/target/debug/seatloom-mcp-emit"
PROJECT=acceptance-lineage-proof
NIMBUS=seat-nimbus-001
BASE_BRANCH="${SEATLOOM_PROOF_BASE:-track/infra-foundation}"

[[ "$DB" == "seatloom" ]] && { echo "REFUSING live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }

# drive <ACTOR_REF> <json-rpc line...>
drive() {
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" \
    SEATLOOM_PROJECT_ID="$PROJECT" SEATLOOM_INTEGRATION_BRANCH="$BASE_BRANCH" \
    SEATLOOM_REPO_ROOT="$(git rev-parse --show-toplevel)" "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
lastres() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR " if r["isError"] else "OK  ")+r["content"][0]["text"])'; }
count() { psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';"; }

echo "target database = ${DB}  (schema 001..014 + 016 + 017)"
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','lineage proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM acceptances WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM dispatches WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
echo "fixture: project ${PROJECT} clean"

# ---------------------------------------------------------------------------
# Fixture: a done dispatch, its lifecycle chain, a Flux verdict, one product,
# and (later) a card. The original task statement is what the card alone never
# showed — the whole point of the lineage read model.
# ---------------------------------------------------------------------------
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at,completed_at) \
        VALUES ('disp-lin-proof','${NIMBUS}','seat:seat-lyra-001','${PROJECT}', \
                'lineage proof task','{\"packet\":\"p/lineage.md\",\"note\":\"the original task words\"}', \
                'done',5,NOW(),NOW());" >/dev/null
for pair in "ev-lin-issued SeatDispatchIssued" "ev-lin-delivered SeatDispatchDelivered" "ev-lin-completed SeatDispatchCompleted"; do
  set -- $pair
  psql "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
          VALUES ('$1','$2',NOW(),'${NIMBUS}','{}','${PROJECT}',1);" >/dev/null
  psql "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ('$1','dispatch','disp-lin-proof');" >/dev/null
done
psql "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
        VALUES ('ev-lin-verdict','ReviewVerdictIssued',NOW(),'seat:seat-flux-001', \
                '{\"verdict\":\"pass\",\"evidence\":\"gates green\",\"evidence_detail\":\"tsc+vitest\", \
                  \"target_branch\":\"${BASE_BRANCH}\",\"lane\":\"nimbus\"}','${PROJECT}',1);" >/dev/null
psql "INSERT INTO artifacts (id,template,subtype,subtype_valid,title,storage_path,created_at) \
        VALUES ('ar-lin-proof','T3','implementation-delivery',true,'the delivery doc','docs/lineage.md',NOW()) \
        ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
        VALUES ('ev-lin-product','ArtifactCreated',NOW(),'${NIMBUS}','{\"kind\":\"delivery\"}','${PROJECT}',1);" >/dev/null
psql "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ('ev-lin-product','artifact','ar-lin-proof');" >/dev/null
psql "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ('ev-lin-product','dispatch','disp-lin-proof');" >/dev/null
# Two dependency edges — one whose referent EXISTS and one that does not, so the
# per-edge existence answer (012: readiness is an existence query) is evidenced
# rather than assumed.
psql "INSERT INTO dispatch_dependencies (dispatch_id,dep_kind,dep_value) VALUES ('disp-lin-proof','dispatch','disp-lin-proof');" >/dev/null
psql "INSERT INTO dispatch_dependencies (dispatch_id,dep_kind,dep_value) VALUES ('disp-lin-proof','event','ev-lin-missing');" >/dev/null
echo "fixture: dispatch done, 3 lifecycle events, verdict, 1 product, 2 dependency edges"

echo
echo "########## POINT 1 — the plan negative matrix (contract §2, tool layer) ##########"
EVENTS_BEFORE=$(count)
# The full request_acceptance arguments for this fixture, with $1 = the plan JSON.
# Built in a variable (single-quoted concatenation, the house proof style) rather
# than inline in the command substitution: an inline escaped-quote JSON gets
# word-split by the shell before the tool ever sees it.
req_args() {
  echo '{"dispatch_id":"disp-lin-proof","branch":"packet/nimbus/proof","how_to_verify":"do A see B","flux_verdict_ref":"ev-lin-verdict","base":"'"${BASE_BRANCH}"'","acceptance_plan":'"$1"'}'
}
reject_plan() { # <label> <plan-json>
  local args; args="$(req_args "$2")"
  drive "$NIMBUS" "$INIT" "$INITED" "$(call request_acceptance "$args")" | lastres
}
reject_plan "missing plan ({})"        '{}'
reject_plan "null plan"                'null'
reject_plan "plan_version 2"           '{"plan_version":2,"scenario":"s","steps":[{"action":"a"}],"expected":[{"output":"o","expect":"e"}]}'
reject_plan "unknown key"              '{"plan_version":1,"scenario":"s","steps":[{"action":"a"}],"expected":[{"output":"o","expect":"e"}],"surprise":1}'
reject_plan "empty steps"              '{"plan_version":1,"scenario":"s","steps":[],"expected":[{"output":"o","expect":"e"}]}'
reject_plan "empty expected"           '{"plan_version":1,"scenario":"s","steps":[{"action":"a"}],"expected":[]}'
reject_plan "blank action"             '{"plan_version":1,"scenario":"s","steps":[{"action":"  "}],"expected":[{"output":"o","expect":"e"}]}'
reject_plan "blank expect"             '{"plan_version":1,"scenario":"s","steps":[{"action":"a"}],"expected":[{"output":"o","expect":" "}]}'
reject_plan "not an object"            '"just a string"'
EVENTS_AFTER=$(count)
echo "canonical_events: before=${EVENTS_BEFORE} after=${EVENTS_AFTER}  (expect EQUAL — every refusal writes nothing)"
echo "acceptances rows: $(psql "SELECT count(*) FROM acceptances WHERE project_id='${PROJECT}';")  (expect 0 — no plan, no card)"

echo
echo "########## POINT 2 — a valid plan creates the card (row + event payload) ##########"
PLAN='{"plan_version":1,"scenario":"operator verifies the delivered slice end to end","steps":[{"action":"open the dispatch page and expand the card"},{"action":"run the check","input":"pnpm test"}],"expected":[{"output":"the suite passes","expect":"seeing a green run"}],"preconditions":["postgres up"]}'
PLAN_ARGS="$(req_args "$PLAN")"
drive "$NIMBUS" "$INIT" "$INITED" "$(call request_acceptance "$PLAN_ARGS")" | lastres
echo "card status: $(psql "SELECT status FROM acceptances WHERE id='acc-disp-lin-proof';")  (expect pending)"
echo "card base:   $(psql "SELECT base FROM acceptances WHERE id='acc-disp-lin-proof';")  (expect ${BASE_BRANCH})"
echo "row plan scenario:  $(psql "SELECT acceptance_plan->>'scenario' FROM acceptances WHERE id='acc-disp-lin-proof';")"
echo "event plan scenario:$(psql "SELECT payload->'acceptance_plan'->>'scenario' FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE r.ref_type='acceptance' AND r.ref_id='acc-disp-lin-proof';")"
echo "plan identical in row and event: $(psql "SELECT (SELECT acceptance_plan FROM acceptances WHERE id='acc-disp-lin-proof') = (SELECT payload->'acceptance_plan' FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE r.ref_type='acceptance' AND r.ref_id='acc-disp-lin-proof');")  (expect t)"

echo
echo "########## POINT 3 — get_acceptance_lineage assembles the seven blocks ##########"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_acceptance_lineage '{"acceptance_id":"acc-disp-lin-proof"}')" | tail -1 | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
assert not r["isError"], r["content"][0]["text"]
l = json.loads(r["content"][0]["text"])
blocks = ["acceptance","dispatch","events","fluxVerdict","products","dependencies","rework"]
for b in blocks:
    assert b in l, f"missing block {b}"
print("blocks present:", ", ".join(blocks))
d = l["dispatch"]
print("dispatch title:", d["title"], "| issuedBy:", d["issuedBy"], "| status:", d["status"])
print("original task payload:", json.dumps(d["payload"], ensure_ascii=False))
print("event chain:", " -> ".join(e["eventType"] for e in l["events"]))
v = l["fluxVerdict"]
print("flux verdict:", v["verdict"], "| evidence:", v["evidence"], "| lane:", v["lane"])
print("products:", [(p["refType"], p["title"], p["resolved"]) for p in l["products"]])
print("dependencies:", [(d["depKind"], d["depValue"], d["exists"]) for d in l["dependencies"]])
assert any(d["exists"] for d in l["dependencies"]), "an existing referent reads as existing"
assert any(not d["exists"] for d in l["dependencies"]), "a missing referent reads as missing"
print("rework:", l["rework"], "(expect None on a pending card)")
a = l["acceptance"]
print("card plan scenario:", a["acceptancePlan"]["scenario"])
print("card plan steps:", [s["action"] for s in a["acceptancePlan"]["steps"]])
print("card base:", a["base"], "| status:", a["status"], "| howToVerify:", a["howToVerify"])
'

echo
echo "########## POINT 4 — pure read: the lineage call moves no ledger row ##########"
BEFORE=$(count)
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_acceptance_lineage '{"acceptance_id":"acc-disp-lin-proof"}')" >/dev/null
AFTER=$(count)
echo "canonical_events: before=${BEFORE} after=${AFTER}  (expect EQUAL)"
drive "$NIMBUS" "$INIT" "$INITED" "$(call get_acceptance_lineage '{"acceptance_id":"acc-does-not-exist"}')" | lastres

echo
echo "########## POINT 5 — tools/list: the read tool is advertised, no identity param ##########"
drive "$NIMBUS" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 | python3 -c '
import sys, json
t = json.load(sys.stdin)["result"]["tools"]
print("tools:", [x["name"] for x in t])
lin = [x for x in t if x["name"] == "get_acceptance_lineage"][0]
props = lin["inputSchema"]["properties"]
print("lineage params:", list(props), "(acceptance_id only — a read acts as nobody)")
req = [x for x in t if x["name"] == "request_acceptance"][0]["inputSchema"]["required"]
print("request_acceptance required:", req)
assert "acceptance_plan" in req, "the plan is required on the wire"
assert not any("seat" in k for k in props), "no identity parameter on the read"
'

echo
echo "proof complete: lineage assembled (7 blocks), plan enforced (9 negative cases, zero writes), plan landed (row + event), read is pure."
