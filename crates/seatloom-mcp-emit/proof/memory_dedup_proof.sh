#!/usr/bin/env bash
# Proof for 023: `remember` refuses a repeat write instead of duplicating it.
#
# WHAT IS PROVEN, against a scratch database built from the schema files, with
# the rows the defect was measured on:
#
#   1  the key is (l2_file_path, l2_line_start, l2_line_end, l0_summary) — an
#      IDENTICAL repeat is refused, naming the row that is already there
#   2  the NULL-line case is caught: three of the four live duplicate groups
#      carry no line range at all, and `= NULL` matches nothing
#   3  a title change does NOT sneak it through (Vela's probe shape)
#   4  the legitimate shared-pointer rows are NOT refused: four distinct
#      memories cite the same lines and the same source_ref, and differ only in
#      what they say
#   5  a genuinely different record at the same pointer IS accepted
#   6  nothing is written when a write is refused (the transaction rolls back)
#   7  the refusal reaches the tool caller as a typed rejection, not a panic
#
# An assertion that dies must STOP the proof (set -e + ERR trap): a proof that
# carries on past a failed assert is how "ALL POINTS PASS" gets printed over a
# measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
SCRATCH="${SEATLOOM_PROOF_DB:-seatloom_dedup}"
BIN="$ROOT/target/debug/seatloom-mcp-emit"

[ "$SCRATCH" != "seatloom" ] || { echo "REFUSING the live DB"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "SELECT 1" >/dev/null 2>&1 \
  || { echo "live is not reachable, so the scratch cannot be built"; exit 1; }

# ── build the scratch ───────────────────────────────────────────────────────
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "DROP DATABASE IF EXISTS $SCRATCH;" >/dev/null
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "CREATE DATABASE $SCRATCH;" >/dev/null
# `apply` pipes the file into psql's stdin, so the whole file is ONE session and
# its own transaction. `-f /dev/stdin` does NOT do that: each psql invocation gets
# a fresh connection, so a DO block that RAISEs aborts every later statement and
# the failure surfaces as "relation X does not exist" far from its cause.
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -q -v ON_ERROR_STOP=1 -f /dev/stdin; }

S="$ROOT/infra/postgres/schema"; D="$ROOT/infra/postgres/seed"
# The full canonical order — the same one the other proofs use. `008` alone
# fails with "relation workitems does not exist" because it depends on the
# 001-005 tables; a partial order is how a proof ends up blaming the wrong file.
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
  git show "track/infra-foundation:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from track"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
apply < "$ROOT/infra/postgres/schema/025_memory_dedup.sql" \
  || { echo "023 did not apply — see above (it refuses while duplicates remain)"; exit 1; }

U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
call() { # tool args-json
  # Take the response to id=9 BY ITS ID, not by line position. `tail -1` grabbed
  # the initialize response whenever the server's "ready" line on stdout shifted
  # the count, and the proof then read a capabilities object as a tool result.
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=seat:seat-nimbus-001 SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null \
  | python3 -c '
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        d = json.loads(line)
    except ValueError:
        continue
    if d.get("id") == 9:
        print(json.dumps(d))
        break
else:
    sys.exit("no response to id=9")'
}
q() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -t -A -c "$1"; }

# A topic to hang the memories off. The topic id is read back, not invented.
# `id` and `opened_by` are both NOT NULL with no default (010), so the scratch
# insert supplies both. The id is generated here and read back, never invented
# twice — a mismatch between the id inserted and the id the tool is given would
# make every later point fail for a reason that has nothing to do with dedup.
TOPIC="topic-dedup-proof-$(date +%s)"
q "insert into topics (id,project_id,title,status,opened_by) values ('$TOPIC','seatloom','dedup proof','open','seat:seat-nimbus-001');" >/dev/null
echo "  topic: $TOPIC"
echo "  topic: $TOPIC"

L0="The evidence pointer plus what the record says is the dedup key; the title is not."
args() { # title l0 [start] [end]
  local t="$1" l="$2" s="${3:-null}" e="${4:-null}"
  # The L2 pointer kind is a CLOSED domain (document | file, objects/memory.rs:133)
  # and an unknown value is refused rather than defaulted -- so a scratch that
  # invents "file_lines" fails on shape before it ever reaches the dedup guard.
  #
  # 'file' is used for EVERY case, including the one with no line range. That is
  # legal (memory.rs:560 requires a range only when the source is an
  # archived_session transcript) and it avoids the 'document' kind's foreign key
  # to `documents`, which this scratch does not populate -- the first attempt at
  # this proof used 'document' and failed on that FK, not on the guard.
  # The first printf does NOT close the object -- it ends after "confidence", so
  # the pointer fields below are appended INSIDE it. Closing it early produced
  # `...},"l2_file_path":...}` and the server answered "parse error: trailing
  # characters", which reads as a proof failure while the guard was never called.
  printf '{"topic_id":"%s","title":"%s","l0_summary":"%s","l2_ref_kind":"file","source_kind":"manual","source_ref":"%s","kind":"%s","confidence":"%s"' \
    "$TOPIC" "$t" "$l" "ev-dedup-proof" "failure" "inferred"
  printf ',"l2_file_path":"docs/x.md"'
  [ "$s" = "null" ] || printf ',"l2_line_start":%s,"l2_line_end":%s' "$s" "$e"
  printf '}'
}

echo
echo "########## POINT 1 — an identical repeat is REFUSED ##########"
OUT="$(call remember "$(args 'first' "$L0" 10 20)")"
first="$OUT"
echo "$first" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert not r.get("isError"), r["content"][0]["text"]
print("  first write: accepted")'
# Count by the POINTER, not by the l0_summary text: L0 contains a semicolon, and
# interpolating it into a SQL literal made the statement end early and the count
# come back as something that was never measured.
N1="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start=10 and l2_line_end=20;")"
echo "  rows after first write: $N1"
[ "$N1" = "1" ] || { echo "FAIL: the first write did not land"; exit 1; }

second="$(call remember "$(args 'first' "$L0" 10 20)")"
echo "$second" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "the identical repeat was ACCEPTED — the defect is still there"
msg=r["content"][0]["text"]
print("  repeat write: REFUSED")
print("  refusal:", msg[:300])
assert ("already exists" in msg) or ("duplicate" in msg.lower()), msg'
N2="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start=10 and l2_line_end=20;")"
echo "  rows after the refused repeat: $N2"
[ "$N2" = "1" ] || { echo "FAIL: the refused write still created a row ($N2)"; exit 1; }
echo "REFUSED, AND NOTHING WAS WRITTEN"

echo
echo "########## POINT 2 — the NULL-line case is caught ##########"
# Three of the four live duplicate groups carry NO line range at all, and
# `= NULL` matches nothing. A guard written with plain equality misses them.
NL="A memory with no line range at all, which is what three of the four live groups look like."
call remember "$(args 'no-lines first' "$NL")" >/dev/null
nl2="$(call remember "$(args 'no-lines second' "$NL")")"
echo "$nl2" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "the NULL-line repeat was ACCEPTED"
print("  NULL-line repeat: REFUSED")'
N3="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start is null and l2_line_end is null;")"
[ "$N3" = "1" ] || { echo "FAIL: NULL-line duplicate created $N3 rows"; exit 1; }
echo "NULL LINES ARE CAUGHT — a guard using = NULL would have missed this"

echo
echo "########## POINT 3 — a title change does NOT sneak it through ##########"
# Vela's 2026-10-11 probe: she altered the title and the write went through.
T3="A record whose title is changed on the second write, which is Vela's probe shape."
call remember "$(args 'original title' "$T3" 30 40)" >/dev/null
t3="$(call remember "$(args 'original title (idempotency probe — expect refusal)' "$T3" 30 40)")"
echo "$t3" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "a title change SNUCK THROUGH — the key must not include the title"
print("  renamed repeat: REFUSED")'
N4="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start=30 and l2_line_end=40;")"
[ "$N4" = "1" ] || { echo "FAIL: the renamed repeat created $N4 rows"; exit 1; }
echo "THE TITLE IS NOT PART OF THE KEY"

echo
echo "########## POINT 4 — legitimate shared-pointer rows are NOT refused ##########"
# The live false-positive trap: four distinct memories cite the SAME lines AND
# the SAME source_ref, differing only in what they say. They are not duplicates.
for i in 1 2 3 4; do
  out="$(call remember "$(args "distinct $i" "Substance $i — a different record at the same pointer." 1 326)")"
  echo "$out" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert not r.get("isError"), "a LEGITIMATE distinct record was refused: " + r["content"][0]["text"]'
done
N5="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start=1 and l2_line_end=326;")"
echo "  four distinct records at one pointer: $N5 rows"
[ "$N5" = "4" ] || { echo "FAIL: expected 4 rows, got $N5"; exit 1; }
echo "SHARED POINTER, DIFFERENT SUBSTANCE — ALL FOUR ACCEPTED"

echo
echo "########## POINT 5 — the write path is unchanged for genuinely new rows ##########"
N6="$(q "select count(*) from memories;")"
echo "  total rows in the scratch: $N6"
# Exactly 7: point 1 (1) + point 2 (1) + point 3 (1) + point 4 (4). The first
# draft of this assertion said ">= 8" with a comment inventing a "topic-era" row
# that this scratch never creates — a check that fails on correct behaviour is
# how a proof gets "fixed" by loosening the thing it measures.
[ "$N6" = "7" ] || { echo "FAIL: expected exactly 7 rows (1+1+1+4), got $N6"; exit 1; }
echo "ONLY THE REPEATS WERE REFUSED"

echo
echo "########## POINT 6 — the SQL layer is proven, not masked by the index ##########"
# WHY THIS POINT EXISTS. Flux mutated the guard's NULL-safe comparison to plain
# `=` and BOTH the proof and the Rust test stayed green. The cause is the
# two-layer defence: the unique index is written with COALESCE and is NULL-safe,
# so it keeps refusing while the SQL layer is broken. That is good design, and it
# means a single-layer mutation cannot prove the SQL layer works — to test one
# layer you have to account for the other, which is Flux's own note on method.
#
# So: drop the index, re-run the NULL-line case, and require it to STILL be
# refused. With the index gone, only the guard's SQL stands between a retry and a
# second row.
podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -q -c "DROP INDEX IF EXISTS memories_dedup_key;" \
  || { echo "FAIL: could not drop the index to isolate the SQL layer"; exit 1; }
echo "  index dropped — only the guard's SQL is left"
call remember "$(args 'sql-layer only first' "$NL")" >/dev/null
sql_only="$(call remember "$(args 'sql-layer only second' "$NL")")"
echo "$sql_only" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "with the index dropped, the SQL layer let the NULL-line repeat through"
print("  NULL-line repeat with no index: REFUSED by the SQL alone")'
N7="$(q "select count(*) from memories where l2_file_path='docs/x.md' and l2_line_start is null and l2_line_end is null;")"
[ "$N7" = "1" ] || { echo "FAIL: SQL-only layer created $N7 rows"; exit 1; }
echo "THE SQL LAYER IS LOAD-BEARING ON ITS OWN"

echo
echo "ALL POINTS PASS"
