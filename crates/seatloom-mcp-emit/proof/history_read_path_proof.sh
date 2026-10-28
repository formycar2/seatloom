#!/usr/bin/env bash
# Proof for the historical session corpus READ PATH (product contract
# LYRA-2026-10-14-history-sessions-in-ui-contract-v1 §8.2 slice 2, dispatch
# disp-history-sessions-read).
#
# Drives the REAL seatloom-mcp-emit binary over stdio against a SCRATCH database
# built in the canonical order (pre-seed 001-005+009, seed 001-004, post-seed
# 008+010-014) plus 015-019 from track/infra-foundation and 021 — the corpus
# migration — from packet/onyx/topic-opened-019, because 021 is not in this
# branch's infra/**. NEVER the live seatloom DB.
#
# =============================================================================
# THE SHAPE OF THE SCRATCH DATABASE — the whole point of this proof (Flux FAIL)
# =============================================================================
# Flux's verdict FLUX-2026-10-14-history-sessions-verdict-v1 §5 found this proof
# passing on a FRESH database while the product was wrong: a fresh database has
# no inventory, so the missing `message_count IS NOT NULL` filter could not be
# seen. The live table ALSO holds the 2026-10-08 out-of-band manifest load (271
# rows, 89 of them with no extracted conversation), and 78 of those 89 carry a
# project slug matching one of the 38 project ids — so every project-grouped
# count was inflated (sum 261 for a corpus of 183; sf-code/storage 58 for 2).
#
# HISTORY_PROOF_SHAPE=live (THE DEFAULT) reproduces that: the canonical order,
# then a READ-ONLY `pg_dump --data-only` of live `archived_sessions` loaded in,
# then 021 on top — exactly the state `scripts/apply-schema-to-live.sh` will
# face. The dump READS live; nothing here ever writes to it.
#
# HISTORY_PROOF_SHAPE=fresh keeps the minimal case (canonical order + 021). It is
# NOT sufficient on its own — that is the finding — so it is never the default.
#
# If live has already had 021 applied, its dump carries `message_count` and the
# pre-021 COPY cannot be loaded; the script says so and stops rather than
# silently falling back to a shape that cannot see the defect.
#
# The turn half reads the extracted contexts on disk, pointed at by
# SEATLOOM_HISTORY_CONTEXTS_DIR; the default is <repo root>/resume-contexts,
# which is gitignored and lives in the main checkout.
#
# POINTs:
#   1  the six numbers the operator's card checks: 38 / 183 / 36680 / 49 / 14 / 14
#      (plus the three counts Flux named as wrong, and the same numbers from SQL)
#   2  undated sessions come LAST and are flagged, never silently interleaved
#   3  the (root) group is labelled and unmapped, never folded into a neighbour
#   4  turns are pointers in source order with the ORIGINAL line numbers — and no bodies
#   5  every "nothing here" carries its reason
#   6  the read writes nothing: the ledger does not move
#   7  the inventory never enters a project's list, and `dated` never contradicts the sort
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

DB="${SEATLOOM_PROOF_DB:-seatloom_hist}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=hist-proof
TRACK_REF="${TRACK_REF:-track/infra-foundation}"
ONYX_REF="${ONYX_REF:-packet/onyx/topic-opened-019}"
# live (default) or fresh — see the header. live reproduces the 271-row manifest
# inventory that the live table holds on top of the corpus; fresh does not, and
# therefore cannot see the defect Flux measured.
SHAPE="${HISTORY_PROOF_SHAPE:-live}"
LIVE_DB="${SEATLOOM_PROOF_LIVE_DB:-seatloom}"
# Where the read-only live inventory dump is written. Supply
# SEATLOOM_PROOF_LIVE_INVENTORY to use a pre-made one instead of dumping live.
LIVE_DUMP="${SEATLOOM_PROOF_LIVE_INVENTORY:-/tmp/history_proof_live_archived_sessions.sql}"
# The extracted contexts. Defaults to the main checkout's copy, which is where
# the gitignored corpus lives; override with SEATLOOM_HISTORY_CONTEXTS_DIR.
CTX="${SEATLOOM_HISTORY_CONTEXTS_DIR:-/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts}"

[[ "$DB" == "seatloom" ]] && { echo "REFUSING the live DB"; exit 1; }
[[ "$SHAPE" == "live" || "$SHAPE" == "fresh" ]] || { echo "unknown shape '$SHAPE' (live|fresh)"; exit 1; }
[[ -d "$CTX/by-session" ]] || { echo "no contexts at $CTX/by-session"; exit 1; }
[[ "$LIVE_DB" != "$DB" ]] || { echo "REFUSING to dump the scratch DB as live"; exit 1; }

psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { # tool, args-json
  printf '%s\n%s\n%s\n' "$INIT" "$INITED" \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID="$PROJECT" \
      SEATLOOM_HISTORY_CONTEXTS_DIR="$CTX" "$BIN" 2>/dev/null | tail -1
}
text() { python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["content"][0]["text"])'; }
err()  { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; print(("ERR: " if r["isError"] else "")+r["content"][0]["text"])'; }

# ---------------------------------------------------------------------------
# Scratch database: canonical order + 015-019 (track) + 021 (Onyx's branch).
# ---------------------------------------------------------------------------
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${DB}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${DB}" >/dev/null
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
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
  git show "${TRACK_REF}:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from ${TRACK_REF}"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
git show "${ONYX_REF}:infra/postgres/schema/021_history_sessions_corpus.sql" > /tmp/021.sql 2>/dev/null \
  || { echo "cannot read 021 from ${ONYX_REF}"; exit 1; }

# ---------------------------------------------------------------------------
# The live shape: the 2026-10-08 manifest inventory, loaded BEFORE 021 so the
# corpus migration has to normalise it exactly as the operator's apply will.
# This is a READ of live (pg_dump --data-only); live is never written here.
# ---------------------------------------------------------------------------
if [ "$SHAPE" = "live" ]; then
  if [ ! -s "$LIVE_DUMP" ]; then
    podman exec seatloom-postgres pg_dump -U seatloom --data-only \
      --table=public.archived_sessions "$LIVE_DB" > "$LIVE_DUMP" 2>/dev/null \
      || { echo "cannot dump live ${LIVE_DB}.archived_sessions (is the container up?)"; exit 1; }
  fi
  if grep -q "message_count" "$LIVE_DUMP"; then
    echo "REFUSING: ${LIVE_DB}.archived_sessions already carries message_count, so 021 has"
    echo "  been applied to live. The pre-021 inventory this shape needs is no longer there;"
    echo "  verify against live directly instead (point SEATLOOM_PROOF_LIVE_INVENTORY at a"
    echo "  pre-021 dump, or run HISTORY_PROOF_SHAPE=fresh for the minimal case)."
    exit 1
  fi
  apply < "$LIVE_DUMP" || { echo "cannot load the live inventory into ${DB}"; exit 1; }
  INV=$(psql "SELECT count(*) FROM archived_sessions;")
  [ "$INV" -gt 200 ] || { echo "FAIL: the live inventory loaded $INV rows, expected the 271-row manifest load"; exit 1; }
  echo "live inventory: ${INV} rows dumped read-only from ${LIVE_DB} and loaded (pre-021)"
fi

apply < /tmp/021.sql || { echo "failed at 021"; exit 1; }
echo "scratch database ${DB}: canonical order + 015-019 (${TRACK_REF}) + 021 (${ONYX_REF}) [shape: ${SHAPE}]"

# ---------------------------------------------------------------------------
# The post-021 shape, asserted rather than assumed: on the live shape the table
# must hold the inventory AND the corpus, with the inventory-only rows still
# distinguishable — that separation is what the read path has to respect.
# ---------------------------------------------------------------------------
if [ "$SHAPE" = "live" ]; then
  TOTAL=$(psql "SELECT count(*) FROM archived_sessions;")
  CORPUS=$(psql "SELECT count(*) FROM archived_sessions WHERE message_count IS NOT NULL;")
  INVENTORY=$(psql "SELECT count(*) FROM archived_sessions WHERE message_count IS NULL;")
  PROJECTS=$(psql "SELECT count(*) FROM projects;")
  echo "after 021: ${TOTAL} rows = ${CORPUS} corpus + ${INVENTORY} inventory-only | ${PROJECTS} projects"
  [ "$CORPUS" = "183" ] || { echo "FAIL: ${CORPUS} corpus rows, expected 183"; exit 1; }
  [ "$INVENTORY" -gt 0 ] || { echo "FAIL: no inventory-only rows — this is the fresh shape wearing a live label"; exit 1; }
  [ "$PROJECTS" = "38" ] || { echo "FAIL: ${PROJECTS} projects, expected 38"; exit 1; }
fi
echo "contexts: ${CTX} ($(ls "$CTX/by-session"/*.md | wc -l) files)"

echo
cat <<EOF
########## POINT 1 — the six numbers the operator's card checks [shape: ${SHAPE}] ##########
EOF
OUT="$(call list_history_projects '{}' | text)"
echo "$OUT" | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("projects:", d["projectCount"], "| sessions:", d["sessionTotal"], "| messages:", d["messageTotal"])
by = {p["id"]: p for p in d["projects"]}
print("OfficeBench:", by["OfficeBench"]["sessionCount"], "sessions /", by["OfficeBench"]["messageCount"], "messages")
print("seatloom   :", by["seatloom"]["sessionCount"], "sessions /", by["seatloom"]["messageCount"], "messages")
assert d["projectCount"] == 38, d["projectCount"]
assert d["sessionTotal"] == 183, d["sessionTotal"]
assert d["messageTotal"] == 36680, d["messageTotal"]
assert by["OfficeBench"]["sessionCount"] == 49, by["OfficeBench"]
assert by["seatloom"]["sessionCount"] == 14, by["seatloom"]
'
CURSOR=$(psql "SELECT count(*) FROM archived_sessions WHERE harness='cursor';")
CURSOR_UNDATED=$(psql "SELECT count(*) FROM archived_sessions WHERE harness='cursor' AND first_ts IS NULL;")
echo "cursor sessions: ${CURSOR} (${CURSOR_UNDATED} undated)"
[ "$CURSOR" = "14" ] && [ "$CURSOR_UNDATED" = "14" ] || { echo "FAIL: cursor numbers"; exit 1; }

# The three counts Flux named as visibly wrong before the filter, plus the total
# the operator's card checks at step 1. On the fresh shape these are free; on the
# live shape they are the defect itself (sf-code/storage 58 -> 2, seatloom 16 ->
# 14, (root) 13 -> 8, sum 261 -> 183).
echo "$OUT" | python3 -c '
import sys, json
d = json.load(sys.stdin)
by = {p["id"]: p for p in d["projects"]}
for pid, truth in (("sf-code/storage", 2), ("seatloom", 14), ("(root)", 8)):
    got = by[pid]["sessionCount"]
    print(f"{pid:16s}: {got} sessions (corpus truth {truth})")
    assert got == truth, (pid, got, truth)
# Every project count must be the corpus count, never corpus + inventory.
assert sum(p["sessionCount"] for p in d["projects"]) == 183
print("  sum of per-project counts == 183, and the three Flux-named counts are right")
'
# The same six numbers straight out of SQL, so a wrong read cannot hide behind a
# wrong tool output.
SQL_TOTAL=$(psql "SELECT count(*) FROM archived_sessions WHERE message_count IS NOT NULL;")
[ "$SQL_TOTAL" = "183" ] || { echo "FAIL: SQL corpus total ${SQL_TOTAL}"; exit 1; }
SQL_SUM=$(psql "SELECT coalesce(sum(n),0) FROM (SELECT count(*) AS n FROM archived_sessions WHERE message_count IS NOT NULL GROUP BY project) s;")
[ "$SQL_SUM" = "183" ] || { echo "FAIL: per-project SQL sum ${SQL_SUM}"; exit 1; }
echo "  SQL agrees: 183 corpus rows, 183 across the 38 project groups"

if [ "$SHAPE" = "live" ]; then
  # The 89 inventory-only rows must contribute to NOTHING the tool reports.
  LEAK=$(psql "SELECT count(*) FROM archived_sessions WHERE message_count IS NULL AND project IN (SELECT id FROM projects);")
  [ "$LEAK" -gt 0 ] || { echo "FAIL: no inventory row shares a project slug — the live shape is not loaded"; exit 1; }
  echo "  ${LEAK} inventory-only rows share a project slug and are counted by no read"
fi
echo "  38 / 183 / 36680 / OfficeBench 49 / seatloom 14 / cursor 14 all undated"

echo
echo "########## POINT 2 — undated sessions LAST and flagged ##########"
call list_history_sessions '{"project_id":"ThoughtOnly"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("sessions:", d["sessionCount"], "| undated:", d["undatedCount"])
for s in d["sessions"]:
    print("   ", s["harness"], s["firstTs"], "dated" if s["dated"] else "时间未记录")
assert d["undatedCount"] == d["sessionCount"], "every cursor session here is undated"
assert all(not s["dated"] for s in d["sessions"])
'
call list_history_sessions '{"project_id":"(root)"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
ss = d["sessions"]
dated = [s for s in ss if s["dated"]]; undated = [s for s in ss if not s["dated"]]
print("(root):", d["sessionCount"], "sessions | dated", len(dated), "| undated", len(undated))
assert ss == dated + undated, "undated must come after dated, never interleaved"
'
# A project that has BOTH, so the ordering is actually exercised.
MIXED=$(psql "SELECT project FROM archived_sessions WHERE first_ts IS NOT NULL GROUP BY project INTERSECT SELECT project FROM archived_sessions WHERE first_ts IS NULL GROUP BY project LIMIT 1;")
if [ -n "$MIXED" ]; then
  call list_history_sessions "{\"project_id\":\"$MIXED\"}" | text | python3 -c '
import sys, json
d = json.load(sys.stdin); ss = d["sessions"]
dated = [s for s in ss if s["dated"]]; undated = [s for s in ss if not s["dated"]]
print("mixed project '"$MIXED"':", d["sessionCount"], "sessions |", len(dated), "dated then", len(undated), "undated")
assert ss == dated + undated, "dated first (newest first), undated last"
assert all(s["firstTs"] is None for s in undated)
'
else
  echo "  (no project in this corpus has both dated and undated sessions)"
fi

echo
echo "########## POINT 3 — the (root) group is labelled and unmapped ##########"
call list_history_projects '{}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
root = [p for p in d["projects"] if p["unclassified"]]
assert len(root) == 1, root
r = root[0]
print("(root):", r["sessionCount"], "sessions /", r["messageCount"], "messages | home:", r["homePath"])
assert r["id"] == "(root)" and r["sessionCount"] == 8
# It must not be folded into a neighbour: no other project may claim its sessions.
assert not any(p["id"] != "(root)" and p["homePath"] == r["homePath"] for p in d["projects"])
'
# 015's R5 view still reports those rows as unmapped — the escalation stands.
# The view projects EVERY row carrying the slug (it is a raw projection over
# archived_sessions.project, not a corpus filter), so on the live shape it sees
# the 8 corpus rows plus the inventory rows that share the slug — 13, which is
# the number 015's own comment records. What must hold on either shape: no
# (root) row is ever mapped, and the corpus subset is exactly 8.
ROOT_SLUG=$(psql "SELECT count(*) FROM archived_sessions WHERE project='(root)';")
ROOT_UNMAPPED=$(psql "SELECT count(*) FROM v_archived_sessions_normalized WHERE map_status='unmapped' AND raw_slug='(root)';")
ROOT_CORPUS=$(psql "SELECT count(*) FROM archived_sessions WHERE project='(root)' AND message_count IS NOT NULL;")
echo "  (root): ${ROOT_UNMAPPED}/${ROOT_SLUG} rows unmapped by R5, ${ROOT_CORPUS} of them corpus sessions"
[ "$ROOT_UNMAPPED" = "$ROOT_SLUG" ] || { echo "FAIL: some (root) row was mapped — the escalation is gone"; exit 1; }
[ "$ROOT_CORPUS" = "8" ] || { echo "FAIL: ${ROOT_CORPUS} corpus root sessions, expected 8"; exit 1; }

echo
echo "########## POINT 4 — turns are POINTERS in source order, no bodies ##########"
# A codex session whose native id the corpus re-keyed the row to.
SID=$(psql "SELECT id FROM archived_sessions WHERE harness='codex' AND message_count=7 LIMIT 1;")
call get_history_session_turns "{\"harness\":\"codex\",\"id\":\"$SID\"}" | text | python3 -c '
import sys, json
d = json.load(sys.stdin); s = d["session"]
print("source:", s["sourcePath"])
print("present:", s["sourcePresent"], "| lines:", s["sourceLineCount"],
      "| declared:", s["declaredMessageCount"], "| stored:", s["storedMessageCount"],
      "| agree:", s["countsAgree"])
print("turns:", len(s["turns"]), "| bodiesReturned:", d["bodiesReturned"])
lines = [t["line"] for t in s["turns"]]
print("original line numbers:", lines)
assert lines == sorted(lines), "source order"
assert len(set(lines)) == len(lines), "no duplicate anchors"
assert d["bodiesReturned"] is False
assert s["countsAgree"] is True
assert len(s["turns"]) == s["storedMessageCount"], "turn count == message count"
# No body text may cross the boundary.
blob = json.dumps(d)
for needle in ("AGENTS.md instructions", "first turn body", "<INSTRUCTIONS>"):
    assert needle not in blob, f"a body leaked: {needle}"
print("  no transcript body in the payload; each turn carries its original line number")
'
# The cursor case: 011's raw_path is a DIRECTORY, the conversation is the
# chat_dump.json inside it. The unique-directory resolution must find it.
CSID=$(psql "SELECT id FROM archived_sessions WHERE harness='cursor' LIMIT 1;")
call get_history_session_turns "{\"harness\":\"cursor\",\"id\":\"$CSID\"}" | text | python3 -c '
import sys, json
s = json.load(sys.stdin)["session"]
print("cursor source:", s["sourcePath"])
assert s["sourcePath"].endswith("/chat_dump.json"), s["sourcePath"]
assert s["contextPath"] is not None, "the context file was found"
print("  resolved through the workspace directory to the unique chat_dump.json")
'
echo
echo "########## POINT 5 — every 'nothing here' carries its reason ##########"
# A harness that produced no extracted conversation.
ZID=$(psql "SELECT id FROM archived_sessions WHERE harness='stepcode' AND message_count=0 LIMIT 1;")
call get_history_session_turns "{\"harness\":\"stepcode\",\"id\":\"$ZID\"}" | text | python3 -c '
import sys, json
s = json.load(sys.stdin)["session"]
assert s["turns"] == [] and s["emptyReason"], s
print("  0 turns, reason:", s["emptyReason"])
'
# An unknown project names what IS known.
call list_history_sessions '{"project_id":"atlas"}' | err
# An unknown session is a typed refusal, not an empty list.
call get_history_session_turns '{"harness":"codex","id":"nope"}' | err
# A contexts directory that is not there says 历史会话还没加载.
printf '%s\n%s\n%s\n' "$INIT" "$INITED" \
  '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"get_history_session_turns","arguments":{"harness":"codex","id":"'"$SID"'","contexts_dir":"/tmp/definitely-not-here"}}}' \
  | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID="$PROJECT" "$BIN" 2>/dev/null | tail -1 | err

echo
echo "########## POINT 6 — the read writes nothing ##########"
BEFORE=$(psql "SELECT count(*) FROM canonical_events;")
call list_history_projects '{}' >/dev/null
call list_history_sessions '{"project_id":"OfficeBench"}' >/dev/null
call get_history_session_turns "{\"harness\":\"codex\",\"id\":\"$SID\"}" >/dev/null
AFTER=$(psql "SELECT count(*) FROM canonical_events;")
echo "  ledger rows before=${BEFORE} after=${AFTER}"
[ "$BEFORE" = "$AFTER" ] || { echo "FAIL: the read wrote to the ledger"; exit 1; }
# And no transcript body was written to the database by anything here.
BODY=$(psql "SELECT count(*) FROM information_schema.columns WHERE table_name='archived_sessions' AND column_name IN ('body','content','transcript');")
[ "$BODY" = "0" ] || { echo "FAIL: a body column exists"; exit 1; }
ROWS=$(psql "SELECT count(*) FROM sessions;")
[ "$ROWS" = "5" ] && echo "  sessions table still holds its 5 seeded rows (nothing inserted)" \
  || { echo "FAIL: sessions changed ($ROWS)"; exit 1; }
SEATS=$(psql "SELECT count(*) FROM seats;")
[ "$SEATS" = "6" ] && echo "  seats table still holds its 6 seeded rows (no archive seat invented)" \
  || { echo "FAIL: seats changed ($SEATS)"; exit 1; }

echo
echo "########## POINT 7 — the inventory never enters a list, and dated never contradicts the order ##########"
# Every project's session list, walked one call at a time through the real tool.
# Two invariants, both of which the pre-fix code broke on the live shape:
#   * no session in any list may be an inventory-only row (messageCount null) —
#     78 of the 89 inventory rows carry a slug matching one of the 38 projects;
#   * dated must be (firstTs is not None) and the undated must come last, so the
#     flag and the sort can never disagree (Flux's gemini marker row: firstTs
#     null, dated true, sorted into the undated block).
PROJECT_IDS=$(psql "SELECT id FROM projects ORDER BY id;")
TOTAL_SESSIONS=0
while IFS= read -r PID; do
  [ -n "$PID" ] || continue
  SESS="$(call list_history_sessions "{\"project_id\":\"$PID\"}" | text)"
  printf '%s' "$SESS" | python3 -c '
import sys, json
d = json.load(sys.stdin)
ss = d["sessions"]
for s in ss:
    assert s["messageCount"] is not None, ("an inventory-only row entered a list", s["id"])
    assert s["dated"] == (s["firstTs"] is not None), ("dated contradicts firstTs", s["id"])
dated = [s for s in ss if s["dated"]]; undated = [s for s in ss if not s["dated"]]
assert ss == dated + undated, "undated must come after dated, never interleaved"
print(d["sessionCount"])
' > /tmp/hist_proof_count.txt || { echo "FAIL: project $PID"; exit 1; }
  TOTAL_SESSIONS=$((TOTAL_SESSIONS + $(cat /tmp/hist_proof_count.txt)))
done <<< "$PROJECT_IDS"
echo "  the 38 lists hold ${TOTAL_SESSIONS} sessions in total"
[ "$TOTAL_SESSIONS" = "183" ] || { echo "FAIL: the per-project lists sum to ${TOTAL_SESSIONS}, expected 183"; exit 1; }
echo "  no inventory row entered any list, and every dated flag agrees with its sort"

echo
echo "ALL POINTS PASS"
