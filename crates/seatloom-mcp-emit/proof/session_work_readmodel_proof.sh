#!/usr/bin/env bash
# Proof for the session↔work read model (contract
# LYRA-2026-10-16-session-work-bridge-contract-v1 §11.2 slice 2, dispatch
# disp-session-key-readmodel).
#
# WHAT IS PROVEN, by driving the REAL seatloom-mcp-emit binary over stdio against
# a LIVE-SHAPED scratch database (canonical order + 015-019 + 021 + 022, then a
# read-only pg_dump of live's topics/archived_sessions/memories loaded in):
#
#   1  the scratch reproduces live's numbers exactly (272 rows, 183 corpus, 90
#      view rows, drift 3, both_agree=87 / path_only=3)
#   2  the forward read: a session WITH records returns them with pointers and
#      line ranges, and NO transcript body
#   3  THE LOAD-BEARING CASE: "no records at all" and "records that exist but do
#      not resolve" are STRUCTURALLY DIFFERENT answers, and each unresolved record
#      names which key failed and the exact value
#   4  the reverse read resolves BY KEY and carries the topic id as context only
#   5  a memory that is not an archived_session record answers `null`, not an
#      empty object
#   6  an unknown session is a refusal, distinct from "no records"
#   7  a database without the bridge view refuses rather than answering "none"
#   8  the read writes nothing, and the two thin surfaces share one implementation
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
SCRATCH="${SEATLOOM_PROOF_DB:-seatloom_sw}"
ONYX_REF="${ONYX_REF:-packet/onyx/topic-opened-019}"
BIN="$ROOT/target/debug/seatloom-mcp-emit"

[ "$SCRATCH" != "seatloom" ] || { echo "REFUSING the live DB"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "SELECT 1" >/dev/null 2>&1 \
  || { echo "live is not reachable, so a live-shaped scratch cannot be built"; exit 1; }

q() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "$1"; }
qlive() { podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "$1"; }

# ---------------------------------------------------------------------------
# The scratch database: canonical order + 015-019 + 021 + 022, then live's rows.
# 022 is read from Onyx's commit, because it is not in this branch's infra/**.
# ---------------------------------------------------------------------------
echo "building the live-shaped scratch ${SCRATCH} ..."
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${SCRATCH}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${SCRATCH}" >/dev/null
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
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
for n in 015_knowledge_layer 016_acceptance_plan 017_acceptance_cancel 018_memory_hardening 019_topic_opened; do
  git show "${TRACK_REF:-track/infra-foundation}:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from track"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
git show "${ONYX_REF}:infra/postgres/schema/021_history_sessions_corpus.sql" > /tmp/021.sql 2>/dev/null \
  || { echo "cannot read 021 from ${ONYX_REF}"; exit 1; }
apply < /tmp/021.sql || { echo "failed at 021"; exit 1; }
# The view comes from TRACK, not from Onyx's 0e32054. Those are two different
# views: 0e32054 exposes 27 columns (join_state, a1_harness, a1_corpus_path,
# a1_path_rows, ref_harness, ...) and was never on track; track's 022 exposes 20
# columns (resolved_by, missing, l0_summary, ...) and is the one that is LIVE.
# A scratch built from the wrong view makes the read model look broken for a
# reason that does not exist on the real database — the first run of this proof
# did exactly that: point 1 passed, point 2 failed with
# `session_work rejected: postgres error`, and the cause was the scratch, not
# the read.
VIEW_COMMIT="${SESSION_KEY_VIEW_COMMIT:-${TRACK_REF:-track/infra-foundation}}"
git show "${VIEW_COMMIT}:infra/postgres/schema/022_session_work_bridge_view.sql" > /tmp/022.sql 2>/dev/null \
  || { echo "cannot read 022 from ${VIEW_COMMIT}"; exit 1; }
apply < /tmp/022.sql || { echo "failed at 022"; exit 1; }
# Live's rows, read-only. Truncate first so a re-run cannot hit a duplicate key.
q "TRUNCATE memories, archived_sessions, topics RESTART IDENTITY CASCADE;" >/dev/null
for t in topics archived_sessions memories; do
  podman exec seatloom-postgres pg_dump -U seatloom --data-only --table=public.$t seatloom > /tmp/dump_$t.sql 2>/dev/null \
    || { echo "cannot dump live $t"; exit 1; }
  apply < /tmp/dump_$t.sql || { echo "cannot load $t into the scratch"; exit 1; }
done
cargo build -q -p seatloom-mcp-emit
[ -x "$BIN" ] || { echo "FAIL: the binary did not build"; exit 1; }
echo "scratch ready"

U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
call() { # tool args-json
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null \
  | tail -1
}
text() { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }
err()  { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert r["isError"], "expected a refusal, got: "+r["content"][0]["text"]; print(r["content"][0]["text"])'; }

echo
echo "########## POINT 1 — the scratch is live-shaped, and the view's numbers match live's ##########"
# (a) The rows that exist on BOTH databases: proving the scratch really carries
#     live's data, so a measurement on it is a measurement on live's data.
for pair in "archived_sessions total|SELECT count(*) FROM archived_sessions" \
            "archived_sessions corpus|SELECT count(*) FROM archived_sessions WHERE message_count IS NOT NULL" \
            "memories total|SELECT count(*) FROM memories" \
            "archived_session memories|SELECT count(*) FROM memories WHERE source_kind='archived_session'"; do
  label="${pair%%|*}"; sql="${pair#*|}"
  a="$(qlive "$sql")"; b="$(q "$sql")"
  echo "  $label: live=$a scratch=$b"
  [ "$a" = "$b" ] || { echo "FAIL: $label differs between live and the scratch"; exit 1; }
done
# (b) The view's own numbers. These CANNOT be compared against live, because the
#     view (022) is not applied to live — that is exactly why this round's -live
#     gate is deferred. They are asserted against Onyx's measurement, recorded in
#     ev-session-key-view-landed's payload and in 022's own header, taken on live
#     read-only on 2026-10-16. Cited, not re-measured here.
LIVE_VIEW_ROWS=90
LIVE_DRIFT=3
LIVE_BOTH_AGREE=87
LIVE_PATH_ONLY=3
for pair in "view rows|SELECT count(*) FROM v_memory_session_join|$LIVE_VIEW_ROWS" \
            "drift (a2 unresolved)|SELECT count(*) FROM v_memory_session_join WHERE NOT a2_resolves|$LIVE_DRIFT" \
            "both_agree|SELECT count(*) FROM v_memory_session_join WHERE resolved_by='both'|$LIVE_BOTH_AGREE" \
            "path_only|SELECT count(*) FROM v_memory_session_join WHERE resolved_by='a1_path'|$LIVE_PATH_ONLY"; do
  label="${pair%%|*}"; rest="${pair#*|}"; sql="${rest%%|*}"; want="${rest#*|}"
  got="$(q "$sql")"
  echo "  $label: scratch=$got  live-measured-by-Onyx=$want"
  [ "$got" = "$want" ] || { echo "FAIL: $label is $got, expected $want"; exit 1; }
done
echo "  the scratch carries live's rows AND reproduces the numbers Onyx measured on live"
echo "THE SCRATCH IS LIVE-SHAPED"

echo
echo "########## POINT 2 — the forward read returns records, with no bodies ##########"
BIG="$(q "SELECT session_harness, session_id FROM v_memory_session_join WHERE resolved_by='both' GROUP BY 1,2 ORDER BY count(*) DESC LIMIT 1;")"
BH="${BIG%%|*}"; BI="${BIG#*|}"
echo "  the session with the most records: ($BH, $BI)"
OUT="$(call session_work "{\"harness\":\"$BH\",\"id\":\"$BI\"}" | text)"
echo "$OUT" | python3 -c '
import sys, json
d = json.load(sys.stdin)
n = d["numbers"]
print("  numbers:", n)
assert n["total"] > 0, "this session should have records"
assert n["total"] == n["resolved"] + n["unresolved"], "the three numbers must be consistent"
r = d["records"][0]
print("  first record:", r["title"][:50], "|", r["kind"], "|", r["confidence"], "| lines", r["lineStart"], "-", r["lineEnd"])
print("  source path:", r["sourcePath"])
assert r["sourcePath"], "every record carries its source path"
assert r["lineStart"] is not None and r["lineEnd"] is not None, "every record carries a line range"
assert r["lineEnd"] >= r["lineStart"], "the range is ordered"
assert d["driftUnresolvedRefs"] == 3, "the drift number rides along"
'
# And NO transcript body anywhere in the payload.
printf '%s' "$OUT" | grep -qi "AGENTS.md instructions\|<INSTRUCTIONS>\|system prompt" \
  && { echo "FAIL: a transcript body leaked into the answer"; exit 1; } || true
echo "$OUT" | python3 -c '
import sys, json
d = json.load(sys.stdin)
# The shape carries pointers and line ranges only. No field holds text.
for r in d["records"]:
    assert set(r.keys()) == {"memoryId","title","kind","confidence","status","sourcePath",
                             "lineStart","lineEnd","joinState","resolvedBy","missing","topicId"}, sorted(r.keys())
print("  every record is pointers + line range + which key; no body field exists")
'
echo "RECORDS ARE POINTERS, NOT BODIES"

echo
echo "########## POINT 3 — THE LOAD-BEARING CASE ##########"
# (a) a corpus session with NO records.
NOREC="$(q "SELECT harness, id FROM archived_sessions WHERE message_count IS NOT NULL AND (harness,id) NOT IN (SELECT session_harness, session_id FROM v_memory_session_join WHERE session_id IS NOT NULL) ORDER BY harness, id LIMIT 1;")"
NH="${NOREC%%|*}"; NI="${NOREC#*|}"
# (b) the session whose records do NOT resolve: the 3 path_only memories.
DRIFT="$(q "SELECT session_harness, session_id FROM v_memory_session_join WHERE resolved_by='a1_path' LIMIT 1;")"
DH="${DRIFT%%|*}"; DI="${DRIFT#*|}"
echo "  (a) no records:    ($NH, $NI)"
echo "  (b) do not resolve: ($DH, $DI)"

call session_work "{\"harness\":\"$NH\",\"id\":\"$NI\"}" | text > /tmp/sw_norec.json
call session_work "{\"harness\":\"$DH\",\"id\":\"$DI\"}" | text > /tmp/sw_drift.json

python3 - "$NH" "$NI" "$DH" "$DI" <<'PY'
import json, sys
nh, ni, dh, di = sys.argv[1:5]
a = json.load(open('/tmp/sw_norec.json'))
b = json.load(open('/tmp/sw_drift.json'))
na, nb = a["numbers"], b["numbers"]
print("  (a) numbers:", na)
print("  (b) numbers:", nb)
# The wall: the two states are structurally different.
assert na["total"] == 0 and na["unresolved"] == 0, a
assert nb["total"] > 0 and nb["unresolved"] > 0, b
assert na["total"] != nb["total"], "the two states must not look alike"
assert a["records"] == [] and b["records"] != []
# Each unresolved record names WHICH key failed and the exact value.
for r in b["records"]:
    assert r["joinState"] == "path_only", r
    assert "a1_declared_path" in r["resolvedBy"], r
    assert "a2_session_ref" not in r["resolvedBy"], r
    assert len(r["missing"]) == 1, r
    m = r["missing"][0]
    assert "resolves against no archived_sessions row" in m, m
    assert "019dda18-07f6-79e0-8008-e4eb16a5448d" in m, m
    print("  (b) record", r["memoryId"], "->", m)
# And the reason is stated, not left to a reader.
assert any("no distilled records" in n for n in a["notes"]), a["notes"]
assert any("do not fully resolve" in n for n in b["notes"]), b["notes"]
print("  (a) note:", [n for n in a["notes"] if "no distilled" in n][0][:110])
print("  (b) note:", [n for n in b["notes"] if "do not fully" in n][0][:110])
print("  THE TWO STATES ARE STRUCTURALLY DIFFERENT")
PY
echo "THE LOAD-BEARING WALL HOLDS"

echo
echo "########## POINT 4 — the reverse read resolves BY KEY ##########"
MEM="$(q "SELECT memory_id FROM v_memory_session_join WHERE resolved_by='both' LIMIT 1;")"
OUT="$(call memory_session "{\"memory_id\":\"$MEM\"}" | text)"
echo "$OUT" | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  memory:", d["memoryId"])
print("  session:", d["session"]["harness"], d["session"]["id"])
print("  joinState:", d["joinState"], "resolvedBy:", d["resolvedBy"])
assert d["session"] is not None
assert set(d["resolvedBy"]) == {"a1_declared_path", "a2_session_ref"}, d["resolvedBy"]
assert d["missing"] == []
# The topic id rides along as CONTEXT, and nothing joins on it.
print("  topicId (context only):", d["topicId"])
assert d["topicId"], "carried"
assert any("never by topic id" in n for n in d["notes"]), d["notes"]
'
# A topic id is never accepted as a lookup key: the tool takes a memory_id.
TOOL="$(call memory_session '{"memory_id":"x"}')"
echo "$TOOL" | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
assert not r["isError"], r["content"][0]["text"]
print("  a memory_id that names no archived_session record answers:", r["content"][0]["text"][:40])
'
echo "RESOLVED BY KEY, TOPIC ID IS CONTEXT ONLY"

echo
echo "########## POINT 5 — an unknown memory answers null, not an empty object ##########"
OUT="$(call memory_session '{"memory_id":"mem-does-not-exist"}')"
echo "$OUT" | python3 -c '
import sys, json
r = json.load(sys.stdin)["result"]
assert not r["isError"], "no answer is not an error"
t = r["content"][0]["text"]
print("  answer:", t[:40])
assert t.strip() == "null", "must be null, not {} and not an error: " + t
'
echo "NO ANSWER IS null, NOT AN EMPTY OBJECT"

echo
echo "########## POINT 6 — an unknown session is a refusal, distinct from no records ##########"
call session_work '{"harness":"codex","id":"nope"}' | err | sed 's/^/  /'
call session_work '{"harness":"codex","id":"nope"}' | err | python3 -c '
import sys
msg = sys.stdin.read()
assert "no archived session" in msg, msg
assert "different fact" in msg, msg
print("  and it says so in a way that cannot be read as \"no records\"")
'
# Half a key is refused too.
call session_work '{"harness":"codex","id":""}' | err | sed 's/^/  /'
echo "AN UNKNOWN SESSION IS A REFUSAL"

echo
echo "########## POINT 7 — a database without the view refuses ##########"
NOVIEW="${SEATLOOM_PROOF_DB:-seatloom_sw}_noview"
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${NOVIEW}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${NOVIEW}" >/dev/null
for f in 001_seatloom_core 002_document_authority 003_write_ingest_reconcile \
         004_operational_review_and_continuity 005_prompt_and_channel_action_authority 009_runtime_triple; do
  podman exec -i seatloom-postgres psql -U seatloom -d "$NOVIEW" -v ON_ERROR_STOP=1 -q -f /dev/stdin < "$S/${f}.sql" >/dev/null
done
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"session_work","arguments":{"harness":"codex","id":"x"}}}' \
  | DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${NOVIEW}" \
    SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null | tail -1 | err | sed 's/^/  /'
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"session_work","arguments":{"harness":"codex","id":"x"}}}' \
  | DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${NOVIEW}" \
    SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null | tail -1 | err | python3 -c '
import sys
msg = sys.stdin.read()
assert "v_memory_session_join" in msg and "022" in msg, msg
assert "must not look like a read that measured nothing" in msg, msg
print("  it names the view and the migration rather than answering \"no records\"")
'
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${NOVIEW}" >/dev/null 2>&1
echo "A MISSING VIEW IS A REFUSAL"

echo
echo "########## POINT 8 — read-only, and ONE implementation ##########"
B="$(q "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions);")"
call session_work "{\"harness\":\"$BH\",\"id\":\"$BI\"}" >/dev/null
call session_work "{\"harness\":\"$DH\",\"id\":\"$DI\"}" >/dev/null
call memory_session "{\"memory_id\":\"$MEM\"}" >/dev/null
call session_work '{"harness":"codex","id":"nope"}' >/dev/null
A="$(q "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions);")"
echo "  scratch events/memories/sessions: before=$B after=$A"
[ "$B" = "$A" ] || { echo "FAIL: the read wrote to the scratch"; exit 1; }
# One implementation: the Tauri commands shell out to the same tools, and there is
# no second assembler in either surface.
grep -q "seatloom_core::db::session_work::lineage\|fn session_work_assembler" "$ROOT/src-tauri/src/commands/session_work_cmds.rs" \
  && { echo "FAIL: a second assembler appeared in the Tauri command"; exit 1; } || true
grep -q '"session_work"' "$ROOT/src-tauri/src/commands/session_work_cmds.rs" \
  || { echo "FAIL: the Tauri command no longer shells out to the session_work tool"; exit 1; }
grep -q '"memory_session"' "$ROOT/src-tauri/src/commands/session_work_cmds.rs" \
  || { echo "FAIL: the reverse Tauri command no longer shells out"; exit 1; }
grep -q "session_work::session_work(db, a).await" "$ROOT/crates/seatloom-mcp-emit/src/main.rs" \
  || { echo "FAIL: the MCP tool no longer calls the core reader"; exit 1; }
grep -q "session_work::memory_session(db, a).await" "$ROOT/crates/seatloom-mcp-emit/src/main.rs" \
  || { echo "FAIL: the reverse MCP tool no longer calls the core reader"; exit 1; }
echo "  both surfaces reach seatloom_core::db::session_work and nothing else"
echo "READ ONLY, ONE IMPLEMENTATION"

echo
echo "ALL POINTS PASS"
