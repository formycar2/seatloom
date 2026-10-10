#!/usr/bin/env bash
# =============================================================================
# waker_loop_proof.sh — reproducible ACCEPTANCE proof for the autonomous
# governed-dispatch pickup loop (G19), at the DB + MCP-binary layer.
#
# Packet / dispatch: disp-flux-loop-proof ("自主派单环的可复现验收脚本").
# Modelled on proof/step6_dispatch_proof.sh
# (NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1 §2), same house style:
# drive the REAL seatloom-mcp-emit binary over stdio (JSON-RPC 2.0) against a
# SCRATCH database, and check the `dispatches` projection AND `canonical_events`
# with psql in the podman postgres container. NEVER the live `seatloom` DB.
#
# SCOPE — what is and is NOT proven.
# The waker daemon (seatloom-waker: LISTEN dispatch_ready -> nudge an idle seat's
# tmux pane) and the seat runtime (pi) are deliberately OUT of scope: this proof
# covers the DB + binary layer they are built on. What it establishes is that
# the four facts the loop depends on are TRUE:
#
#   POINT 1  an enqueued dispatch is immediately `ready` (nothing else has to
#            happen for a seat to be able to pull it)
#   POINT 2  a seat identity (SEATLOOM_ACTOR_REF) drives get_next_dispatch and
#            gets back the WORK (title + payload), marked `in_flight`
#   POINT 3  complete_dispatch marks it `done` and leaves EXACTLY the three
#            lifecycle events in canonical_events (Issued / Delivered / Completed)
#   POINT 4  one-in-flight is enforced — by the DB index AND by the MCP pull
#            semantics AND by ownership (a seat cannot complete another's work)
#
# Usage:
#   crates/seatloom-mcp-emit/proof/waker_loop_proof.sh
#
#   SEATLOOM_PROOF_KEEP_DB=1   reuse the scratch database instead of dropping
#                              and rebuilding it (faster iteration; the proof
#                              still clears its own project's rows)
#
# Prereqs, all handled by this script:
#   - podman container `seatloom-postgres` running (psql is reached via
#     `podman exec`; the SeatLoom host has no local psql client)
#   - cargo on PATH (or ~/.cargo/bin) to build the binary if it is missing
#
# Exit code: 0 iff every assertion held.
# =============================================================================
set -uo pipefail

DB=seatloom_mcp_waker_proof
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
# The binary, the migrations and (inside the server) the escalation daily-log
# path are all resolved relative to the repo root, so run from there.
cd "$ROOT"
BIN="${ROOT}/target/debug/seatloom-mcp-emit"
PROJECT=p0-waker-proof
PG_CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
FLUX=seat-flux-001
MIRA=seat-mira-001

# --- SAFETY: this script must never be pointed at the live database ----------
if [[ "$DB" == "seatloom" ]]; then echo "REFUSING live DB"; exit 1; fi
# A scratch database is one whose NAME SAYS it is scratch. `seatloom` itself is
# caught above; anything else that does not advertise itself as throwaway is
# refused rather than dropped.
if [[ ! "$DB" =~ (proof|scratch|test|tmp) ]]; then
  echo "REFUSING: ${DB} does not look like a scratch database (name must contain proof|scratch|test|tmp)"
  exit 1
fi

PASS=0; FAIL=0
check() { # label got expected
  if [[ "$2" == "$3" ]]; then
    printf '  PASS  %-46s = %s\n' "$1" "$2"; PASS=$((PASS+1))
  else
    printf '  FAIL  %-46s got=%s want=%s\n' "$1" "$2" "$3"; FAIL=$((FAIL+1))
  fi
}
has() { # label haystack needle
  if [[ "$2" == *"$3"* ]]; then
    printf '  PASS  %-46s contains %s\n' "$1" "$3"; PASS=$((PASS+1))
  else
    printf '  FAIL  %-46s MISSING %s\n     in: %s\n' "$1" "$3" "$2"; FAIL=$((FAIL+1))
  fi
}

psql() { podman exec -i "$PG_CONTAINER" psql -U seatloom -d "$DB" -tAc "$1"; }

# --- build the binary if it is not there yet --------------------------------
if [[ ! -x "$BIN" ]]; then
  export PATH="$HOME/.cargo/bin:$PATH"
  echo "building seatloom-mcp-emit (binary missing)..."
  cargo build -p seatloom-mcp-emit || { echo "build failed"; exit 1; }
fi

# --- scratch database: 001..012 in the CANONICAL order ----------------------
# The ordering contract lives in scripts/apply-db.sh (pre-seed / seed /
# post-seed, classified BY RULE). This proof does not re-derive it: it calls
# that script through a psql shim that forwards to the podman container, so the
# proof and CI apply migrations in exactly the same order.
echo "target database = ${DB}"
if [[ "${SEATLOOM_PROOF_KEEP_DB:-0}" != "1" ]]; then
  podman exec -i "$PG_CONTAINER" psql -U seatloom -d postgres -tAc \
    "DROP DATABASE IF EXISTS ${DB} WITH (FORCE);" >/dev/null
  podman exec -i "$PG_CONTAINER" createdb -U seatloom "$DB"
  SHIMDIR="$(mktemp -d)/bin"; mkdir -p "$SHIMDIR"
  cat > "$SHIMDIR/psql" <<'SHIM'
#!/usr/bin/env bash
# psql shim: the host has no local psql client; forward `psql <url> -f <file>`
# to the podman postgres container.
set -euo pipefail
url=""; file=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    postgresql://*) url="$1" ;;
    -f) file="$2"; shift ;;
  esac
  shift
done
[[ -n "$url" && -n "$file" ]] || { echo "psql shim: need <url> and -f <file>" >&2; exit 64; }
db="${url##*/}"; db="${db%%\?*}"
exec podman exec -i "${SEATLOOM_PG_CONTAINER:-seatloom-postgres}" \
  psql -U seatloom -d "$db" -v ON_ERROR_STOP=1 -q -f - < "$file"
SHIM
  chmod +x "$SHIMDIR/psql"
  PATH="$SHIMDIR:$PATH" DATABASE_URL="$DBURL" "$ROOT/scripts/apply-db.sh" >/dev/null \
    || { echo "apply-db.sh failed"; exit 1; }
  echo "scratch database rebuilt: schema 001-012 + seed via scripts/apply-db.sh (canonical order)"
else
  echo "reusing existing scratch database (SEATLOOM_PROOF_KEEP_DB=1)"
fi
check "seats seeded (identity must pre-exist)" "$(psql "SELECT count(*) FROM seats;")" "6"
check "dispatches table exists" "$(psql "SELECT to_regclass('public.dispatches') IS NOT NULL;")" "t"

# --- fixture: a clean proof project -----------------------------------------
psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) VALUES ('${PROJECT}','waker loop proof',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM dispatches WHERE project_id='${PROJECT}';" >/dev/null
psql "DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='${PROJECT}');" >/dev/null
psql "DELETE FROM canonical_events WHERE project_id='${PROJECT}';" >/dev/null
echo "fixture: project ${PROJECT} clean"

# --- drive helpers: launch a server AS a given seat -------------------------
drive() { # <ACTOR_REF> <json-rpc lines...>
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" \
    SEATLOOM_PROJECT_ID="$PROJECT" "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
# "OK <text>" / "ERR <text>" of the last tool result. The prefix is exactly one
# space in both cases so an expected string reads the same way for a success and
# for a rejection.
lastres() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR " if r["isError"] else "OK ")+r["content"][0]["text"])'; }
ev() { psql "SELECT count(*) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' AND r.ref_id='$1' AND e.event_type='$2';"; }

echo
echo "########## POINT 0 — the tool surface: 4 tools, get_next_dispatch has NO seat parameter ##########"
TOOLS=$(drive "$FLUX" "$INIT" "$INITED" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | tail -1)
echo "$TOOLS" | python3 -c 'import sys,json; t=json.load(sys.stdin)["result"]["tools"]; print("tools:",[x["name"] for x in t]); gp=[x for x in t if x["name"]=="get_next_dispatch"][0]["inputSchema"]; print("get_next properties:",list(gp["properties"].keys())); print("get_next additionalProperties:",gp.get("additionalProperties"))'
check "get_next_dispatch seat params" \
  "$(echo "$TOOLS" | python3 -c 'import sys,json; t=json.load(sys.stdin)["result"]["tools"]; gp=[x for x in t if x["name"]=="get_next_dispatch"][0]["inputSchema"]["properties"]; print(",".join(sorted(k for k in gp if "seat" in k)) or "NONE")')" \
  "NONE"

echo
echo "########## POINT 1 — enqueue a dispatch: it is READY immediately ##########"
drive "$FLUX" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-wake-1","target_seat_id":"'$FLUX'","title":"wake the seat","payload":{"packet":"p/wake.md","note":"pull this and do it"},"priority":5}')" | lastres
check "disp-wake-1 status after enqueue" "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-1';")" "ready"
check "SeatDispatchIssued events"        "$(ev disp-wake-1 SeatDispatchIssued)" "1"
check "project events so far"            "$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")" "1"
check "payload recorded verbatim"        "$(psql "SELECT payload->>'packet' FROM dispatches WHERE id='disp-wake-1';")" "p/wake.md"

echo
echo "########## POINT 2 — the seat pulls: gets title + payload, marked in_flight ##########"
OUT2=$(drive "$FLUX" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres)
echo "$OUT2" | sed 's/^/  /'
has "pull result carries the dispatch id" "$OUT2" "disp-wake-1"
has "pull result carries the TITLE"       "$OUT2" "title: wake the seat"
has "pull result carries the PAYLOAD"     "$OUT2" "payload: {"
has "payload carries the packet path"   "$OUT2" '"packet":"p/wake.md"'
has "payload carries the instructions"  "$OUT2" '"note":"pull this and do it"'
has "pull result carries the next step"   "$OUT2" "complete_dispatch with dispatch_id=disp-wake-1"
check "disp-wake-1 status after pull"     "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-1';")" "in_flight"
check "SeatDispatchDelivered events"      "$(ev disp-wake-1 SeatDispatchDelivered)" "1"
check "started_at stamped"                "$(psql "SELECT (started_at IS NOT NULL) FROM dispatches WHERE id='disp-wake-1';")" "t"
check "delivery event names the seat"     "$(psql "SELECT r.ref_type||':'||r.ref_id FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='actor' WHERE e.project_id='${PROJECT}' AND e.event_type='SeatDispatchDelivered';")" "seat:${FLUX}"

echo
echo "########## POINT 2b — identity cannot be forged: a seat_id argument is ignored ##########"
# Another seat's server, passing flux's id as an argument. The argument is not
# part of get_next_dispatch's schema (additionalProperties:false, POINT 0), and
# the server reads only project_id from `arguments` — so the spoof buys nothing.
OUT2B=$(drive "$MIRA" "$INIT" "$INITED" "$(call get_next_dispatch '{"seat_id":"'$FLUX'","target_seat_id":"'$FLUX'"}')" | lastres)
echo "$OUT2B" | sed 's/^/  /'
has "spoofed pull does NOT hand over flux's work" "$OUT2B" "nothing ready for ${MIRA}"
if [[ "$OUT2B" == *"disp-wake-1"* ]]; then echo "  FAIL  spoofed seat_id returned another seat's dispatch"; FAIL=$((FAIL+1)); else printf '  PASS  %-46s\n' "no cross-seat leak"; PASS=$((PASS+1)); fi
check "flux's item still in_flight (untouched)" "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-1';")" "in_flight"

echo
echo "########## POINT 3 — complete: done + exactly 3 lifecycle events ##########"
OUT3=$(drive "$FLUX" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-wake-1"}')" | lastres)
echo "$OUT3" | sed 's/^/  /'
check "complete result"                  "$OUT3" "OK completed disp-wake-1 for ${FLUX}"
check "disp-wake-1 status after complete" "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-1';")" "done"
check "completed_at stamped"              "$(psql "SELECT (completed_at IS NOT NULL) FROM dispatches WHERE id='disp-wake-1';")" "t"
check "lifecycle events for disp-wake-1"  "$(psql "SELECT count(*) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' AND r.ref_id='disp-wake-1';")" "3"
check "  ...which are the three transitions" "$(psql "SELECT string_agg(e.event_type,',' ORDER BY e.occurred_at) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' AND r.ref_id='disp-wake-1';")" "SeatDispatchIssued,SeatDispatchDelivered,SeatDispatchCompleted"
check "every transition has exactly one subject" "$(psql "SELECT count(*) FROM (SELECT e.id FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' AND r.ref_id='disp-wake-1' GROUP BY e.id HAVING count(*)=1) s;")" "3"
check "every transition names the seat as actor" "$(psql "SELECT count(*) FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='actor' WHERE e.project_id='${PROJECT}' AND e.event_type IN ('SeatDispatchIssued','SeatDispatchDelivered','SeatDispatchCompleted') AND r.ref_type='seat' AND r.ref_id='${FLUX}';")" "3"
echo "--- the ledger, ordered (this is what makes the row replayable) ---"
psql "SELECT '  '||e.occurred_at::time||'  '||e.event_type||'  subj=dispatch:'||r.ref_id FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' ORDER BY e.occurred_at;" | sed 's/^/  /'
check "row status is REBUILDABLE from the ledger" "$(psql "SELECT CASE WHEN bool_or(event_type='SeatDispatchCompleted') THEN 'done' WHEN bool_or(event_type='SeatDispatchDelivered') THEN 'in_flight' ELSE 'ready' END FROM canonical_events e JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' WHERE e.project_id='${PROJECT}' AND r.ref_type='dispatch' AND r.ref_id='disp-wake-1';")" "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-1';")"

echo
echo "########## POINT 4 — one-in-flight is a fact, not a promise ##########"
# 4a. the DB index: a second in_flight row for the same seat is REJECTED.
drive "$FLUX" "$INIT" "$INITED" "$(call enqueue_dispatch '{"id":"disp-wake-2","target_seat_id":"'$FLUX'","title":"second task","payload":{"packet":"p/2.md"},"priority":9}')" | lastres
drive "$FLUX" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres >/dev/null
check "flux now holds disp-wake-2 in_flight" "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-2';")" "in_flight"
echo "  raw SQL, forcing a 2nd in_flight row for the same seat:"
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) VALUES ('disp-2nd-inflight','${FLUX}','seat:lyra','${PROJECT}','second','{}'::jsonb,'in_flight',1,NOW());" 2>&1 | sed 's/^/    /'
check "  -> rejected by the DB"            "$(psql "SELECT count(*) FROM dispatches WHERE id='disp-2nd-inflight';")" "0"
check "  -> flux still has exactly 1 in_flight" "$(psql "SELECT count(*) FROM dispatches WHERE target_seat_id='${FLUX}' AND status='in_flight';")" "1"
# The index is PARTIAL on status='in_flight': a done row alongside an in_flight
# row for the same seat is fine, which is why a seat may keep a full history.
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) VALUES ('disp-done-1','${FLUX}','seat:lyra','${PROJECT}','history','{}'::jsonb,'done',1,NOW());" >/dev/null 2>&1
check "  -> a done row is NOT unique-scoped" "$(psql "SELECT count(*) FROM dispatches WHERE id='disp-done-1' AND status='done';")" "1"
check "  -> flux in_flight count unchanged" "$(psql "SELECT count(*) FROM dispatches WHERE target_seat_id='${FLUX}' AND status='in_flight';")" "1"

# 4b. the MCP pull semantics: a second pull hands back the SAME in_flight item.
DELIV_BEFORE=$(ev disp-wake-2 SeatDispatchDelivered)
OUT4B=$(drive "$FLUX" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres)
echo "$OUT4B" | sed 's/^/  /'
has "2nd pull resumes the SAME dispatch" "$OUT4B" "disp-wake-2"
check "no second delivery event"        "$(ev disp-wake-2 SeatDispatchDelivered)" "$DELIV_BEFORE"

# 4c. ownership: another seat cannot complete this seat's in_flight work.
OUT4C=$(drive "$MIRA" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-wake-2"}')" | lastres)
echo "$OUT4C" | sed 's/^/  /'
EV_BEFORE_REJECTS=$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")
has "cross-seat completion is rejected"  "$OUT4C" "not an in_flight dispatch owned by \"${MIRA}\""
check "disp-wake-2 still in_flight"     "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-2';")" "in_flight"
check "no SeatDispatchCompleted leaked"  "$(ev disp-wake-2 SeatDispatchCompleted)" "0"
# ...and neither is a non-existent dispatch id.
OUT4D=$(drive "$FLUX" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-nonexistent"}')" | lastres)
check "unknown dispatch id rejected"     "$OUT4D" "ERR dispatch rejected: \"disp-nonexistent\" is not an in_flight dispatch owned by \"${FLUX}\""
check "rejections wrote no event"        "$(psql "SELECT count(*) FROM canonical_events WHERE project_id='${PROJECT}';")" "${EV_BEFORE_REJECTS}"

# 4d. clearing it: the seat completes its own in_flight -> done, and only then
# may it pull the next item. That close of the loop is what "idle -> pull -> do
# -> complete -> idle" means.
drive "$FLUX" "$INIT" "$INITED" "$(call complete_dispatch '{"dispatch_id":"disp-wake-2"}')" | lastres
check "disp-wake-2 done"                 "$(psql "SELECT status FROM dispatches WHERE id='disp-wake-2';")" "done"
OUT4E=$(drive "$FLUX" "$INIT" "$INITED" "$(call get_next_dispatch '{}')" | lastres)
echo "$OUT4E" | sed 's/^/  /'
check "idle seat with an empty queue"    "$OUT4E" "OK nothing ready for ${FLUX} (idle, not a stall)"

echo
echo "########## PROOF SUMMARY ##########"
printf 'passed=%d failed=%d\n' "$PASS" "$FAIL"
if [[ "$FAIL" -ne 0 ]]; then
  echo "WAKER LOOP PROOF FAILED"
  exit 1
fi
echo "WAKER LOOP PROOF HELD — enqueue(ready) -> pull(title+payload, in_flight) -> complete(done, 3 events), one-in-flight enforced, identity unforgeable."
echo "Scope: DB + binary only. Not covered here: the seatloom-waker daemon (LISTEN dispatch_ready) and the seat runtime."
