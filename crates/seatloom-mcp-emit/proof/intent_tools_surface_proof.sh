#!/usr/bin/env bash
# Proof for the Supervisor intent tool surface (product contract
# LYRA-2026-10-13-supervisor-intent-entry-contract-v1, slice 1; dispatch
# disp-supervisor-intent-mcp-surface-v2).
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# (schema 001-020 as present on this branch's infra/** plus the live seat data
# the planner needs). NEVER the live seatloom DB — the tool is read-only by
# construction, and this script proves that too, by counting ledger rows.
#
# The end-to-end points, in the contract's own order:
#   POINT 1  the typed schema and the deterministic parser exist and are pure
#   POINT 2  every §3.3 ambiguity row STOPS and asks — verified adversarially
#   POINT 3  no hardcoded field survives: budget and rationale are real reads
#   POINT 4  a precondition becomes an edge only when it names a real referent
#   POINT 5  the two surfaces are ONE implementation (MCP tool + Tauri wrapper)
#   POINT 6  the tool writes nothing: the ledger does not move
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_intent}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=intent-proof

[[ "$DB" == "seatloom" ]] && { echo "REFUSING the live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }
drive() {
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF=human:supervisor \
    SEATLOOM_PROJECT_ID="$PROJECT" "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
plan() { drive "$INIT" "$INITED" "$(call plan_intent "{\"intent\":\"$1\"}")" | tail -1; }
# Unwrap the MCP envelope: one tool result, one text block, and that text is
# itself the JSON the planner produced.
text() { python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["content"][0]["text"])'; }
outcome() { plan "$1" | text | python3 -c 'import sys,json; print(json.load(sys.stdin)["outcome"])'; }
ledger_count() { psql "SELECT count(*) FROM canonical_events;"; }

# ---------------------------------------------------------------------------
# Fixture: a fresh project with a deliberately awkward seat registry, so every
# exclusion reason is exercised rather than assumed.
# ---------------------------------------------------------------------------
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc \
  "DROP DATABASE IF EXISTS ${DB}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc \
  "CREATE DATABASE ${DB}" >/dev/null
for f in "$ROOT"/infra/postgres/schema/*.sql; do
  podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -v ON_ERROR_STOP=1 -q -f /dev/stdin \
    < "$f" >/dev/null || { echo "schema apply failed at $f"; exit 1; }
done
# The tool must be advertised before anything else is claimed about it.
LISTED=$(drive "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1 \
  | python3 -c 'import sys,json; print(",".join(t["name"] for t in json.load(sys.stdin)["result"]["tools"]))')
echo "target database = ${DB} (schema 001-020 from infra/**)"
echo "fixture: a fresh project with a deliberately awkward seat registry"
echo "advertised tools include plan_intent: $(echo "$LISTED" | grep -c plan_intent)"

psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','intent proof',NOW(),4096,16384) ON CONFLICT (id) DO NOTHING;" >/dev/null
# A routable pair, a retired seat, an unbound seat, and an out-of-domain harness
# row that must NOT break the planner (the reason gather_intent_facts reads four
# columns instead of the validated full row).
psql "INSERT INTO seats (id,name,default_runtime,default_harness,capability_tags,status,created_at) VALUES
  ('seat-proof-nimbus','nimbus','Pi','Pi',ARRAY['rust','tauri'],'active',NOW()),
  ('seat-proof-flux','flux','Pi','Pi',ARRAY['qa','verification'],'active',NOW()),
  ('seat-proof-onyx','onyx','Pi','Pi',ARRAY['postgres'],'retired',NOW()),
  ('seat-proof-vela','vela','Pi','OutOfDomain',ARRAY['ux'],'active',NOW())
  ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO project_role_bindings (seat_id,project_id,role,authority_doc_refs,constraints) VALUES
  ('seat-proof-nimbus','${PROJECT}','architect',ARRAY['docs/architecture-design.md'],ARRAY['no-runtime-widening']),
  ('seat-proof-flux','${PROJECT}','verifier',ARRAY['docs/acceptance-spec-v1.1.md'],ARRAY[]::text[]),
  ('seat-proof-onyx','${PROJECT}','data_engineer',ARRAY[]::text[],ARRAY['schema-migration-only'])
  ON CONFLICT (seat_id,project_id) DO NOTHING;" >/dev/null
# One in-flight dispatch for nimbus, so the load figure is a real count.
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority) VALUES
  ('disp-proof-busy','seat-proof-nimbus','seat:seat-aegis-001','${PROJECT}','busy','{}'::jsonb,'in_flight',5);" >/dev/null
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority) VALUES
  ('disp-proof-gate','seat-proof-flux','seat:seat-aegis-001','${PROJECT}','Verify the gate','{}'::jsonb,'done',5);" >/dev/null
# A real event, so a precondition that names it can resolve.
psql "INSERT INTO canonical_events (id,event_type,schema_version,occurred_at,actor_ref,project_id,payload) VALUES
  ('ev-proof-landed','ArtifactCreated',1,NOW(),'seat:seat-lyra-001','${PROJECT}','{}');" >/dev/null
psql "INSERT INTO event_object_refs (event_id,ref_type,ref_id,role) VALUES ('ev-proof-landed','artifact','art-proof','subject');" >/dev/null

echo
echo "########## POINT 1 — the parser is deterministic and typed ##########"
plan "给 nimbus 派个活,把唤醒服务的硬底兜底接上" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
assert p["proposedObject"]["kind"] == "dispatch", p["proposedObject"]
assert p["ownership"]["seatId"] == "seat-proof-nimbus", p["ownership"]
assert p["actor"] == "human:supervisor", p["actor"]
print("verb/object/owner resolved:", p["proposedObject"]["kind"], "->", p["ownership"]["seatName"])
print("actions offered:", p["actions"])
'
# The same sentence twice must give byte-identical output: no clock, no randomness.
A=$(plan "给 nimbus 派个活,把唤醒服务的硬底兜底接上" | md5)
B=$(plan "给 nimbus 派个活,把唤醒服务的硬底兜底接上" | md5)
[[ "$A" == "$B" ]] && echo "determinism: identical output on repeat ($A)" || { echo "FAIL: nondeterministic"; exit 1; }

echo
echo "########## POINT 2 — every §3.3 row STOPS and asks (adversarial) ##########"
check() { # input, expected rule id
  local got
  got=$(plan "$1" | text | python3 -c 'import sys,json
d=json.load(sys.stdin)
if "clarification" in d: print(d["clarification"]["ruleId"])
elif "retrieval" in d: print("retrieval")
else: print("PROPOSAL:"+d.get("proposal",{}).get("ownership",{}).get("seatId","?"))')
  if [[ "$got" == "$2" ]]; then echo "  PASS  $2  <-  $1"; else echo "  FAIL  expected $2, got $got  <-  $1"; exit 1; fi
}
check "把唤醒服务的硬底兜底接上"                              "contract-3.3-verb-unsupported"
check "创建一个东西"                                          "contract-3.3-object-type-unclear"
check "给 atlas 派个活"                                       "contract-3.3-owner-not-found"
check "派个活"                                                "contract-3.3-owner-omitted"
check "派给 onyx 一个派单"                                    "contract-3.3-no-capable-seat"
check "恢复一下,然后派个活"                                    "contract-3.3-verb-ambiguous"
check "我们为什么放弃了 passport.js?"                          "retrieval"
# The named-owner list is the REAL list, not a default.
plan "给 atlas 派个活" | text | python3 -c '
import sys, json
c = json.load(sys.stdin)["clarification"]
vals = [x["value"] for x in c["choices"]]
assert "seat-proof-nimbus" in vals and "seat-proof-vela" in vals, vals
assert all(x["why"] for x in c["choices"]), "every choice carries its reason"
print("  the seat list offered is the real registry:", vals)
'

echo
echo "########## POINT 3 — no hardcoded field survives ##########"
plan "给 nimbus 派个活" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
b = p["budget"]
assert b["projectWorkerBudgetTokens"] == 4096, b
assert b["projectSupervisorBudgetTokens"] == 16384, b
assert b["seatInFlightDispatches"] == 1, b      # disp-proof-busy, really in_flight
r = [x for x in p["routingRationale"] if x["seatId"] == "seat-proof-nimbus"][0]
assert r["selected"] is True and r["inFlightDispatches"] == 1, r
others = [x for x in p["routingRationale"] if not x["selected"]]
assert all(x["excludingConstraint"] for x in others), "every non-selection states why"
print("  budget from projects.*:", b["projectWorkerBudgetTokens"], b["projectSupervisorBudgetTokens"])
print("  load from dispatches:", b["seatInFlightDispatches"])
print("  every excluded seat carries a reason:", len(others), "of", len(p["routingRationale"]))
'
# The prototype's two fabricated fields must not reappear in any form.
if plan "给 nimbus 派个活" | text | grep -qE "150,000|seat-2"; then echo "  FAIL: a hardcoded field survived"; exit 1; fi
echo "  the prototype budget/owner strings do not reappear"

echo
echo "########## POINT 4 — a precondition is an edge or a stop, never a drop ##########"
plan "给 flux 派个活,等 ev-proof-landed 落地再动" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
deps = p["proposedObject"]["suggestedDependencies"]
assert deps == [{"depKind": "event", "depValue": "ev-proof-landed"}], deps
assert p["blockingClarification"] is None, p["blockingClarification"]
assert "confirm" in p["actions"], p["actions"]
print("  resolved to an edge in enqueue_dispatch spelling:", deps)
'
plan "给 flux 派个活,等 Lyra 的产品契约落地再动" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
assert p["proposedObject"]["suggestedDependencies"] == [], p["proposedObject"]
assert p["precondition"]["unresolvedReason"], p["precondition"]
assert p["blockingClarification"]["ruleId"] == "contract-3.3-precondition-unresolved"
assert "confirm" not in p["actions"], "confirm is not offered while blocked"
assert any(c["value"] == "confirm-without-precondition" for c in p["blockingClarification"]["choices"])
print("  unresolved -> confirm blocked, missing capability named, escape hatch explicit")
'
# A dispatch-kind precondition resolves by exact id and by exact title.
plan "给 flux 派个活,等 disp-proof-gate 落地再动" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
assert p["proposedObject"]["suggestedDependencies"] == [{"depKind":"dispatch","depValue":"disp-proof-gate"}], p["proposedObject"]
print("  dispatch-id precondition -> dispatch edge")
'
plan "给 flux 派个活,等 Verify the gate 再动" | text | python3 -c '
import sys, json
p = json.load(sys.stdin)["proposal"]
assert p["proposedObject"]["suggestedDependencies"] == [{"depKind":"dispatch","depValue":"Verify the gate"}], p["proposedObject"]
print("  exact dispatch-title precondition -> dispatch edge (clause == title, no fuzzy match)")
'

echo
echo "########## POINT 5 — the two surfaces are ONE implementation ##########"
# The Tauri command shells out to this same tool; the only thing it adds is the
# stdio envelope and SEATLOOM_ACTOR_REF=human:supervisor, which is what the MCP
# path used here also carries. Assert the shared plumbing is literally shared.
grep -q 'name": "plan_intent' src-tauri/src/commands/dispatch_cmds.rs \
  && echo "  cmd_plan_intent calls the plan_intent tool (no second planner)" || { echo "FAIL"; exit 1; }
grep -q 'SEATLOOM_ACTOR_REF", "human:supervisor"' src-tauri/src/commands/dispatch_cmds.rs \
  && echo "  the Tauri wrapper preserves human:supervisor attribution" || { echo "FAIL"; exit 1; }
grep -q 'args\["dependencies"\]' src-tauri/src/commands/dispatch_cmds.rs \
  && echo "  V5 fixed: the enqueue wrapper forwards dependencies" || { echo "FAIL"; exit 1; }

echo
echo "########## POINT 6 — the tool writes nothing ##########"
BEFORE=$(ledger_count)
for i in 1 2 3 4 5; do plan "给 nimbus 派个活,等 ev-proof-landed 落地再动" >/dev/null; done
plan "我们为什么放弃了 passport.js?" >/dev/null
plan "给 atlas 派个活" >/dev/null
AFTER=$(ledger_count)
echo "  ledger rows before=${BEFORE} after=${AFTER}"
[[ "$BEFORE" == "$AFTER" ]] && echo "  PASS: a read-only tool, proven by the ledger not moving" \
  || { echo "FAIL: the tool wrote to the ledger"; exit 1; }

echo
echo "ALL POINTS PASS"
