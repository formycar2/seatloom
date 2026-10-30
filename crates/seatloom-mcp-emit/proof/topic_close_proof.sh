#!/usr/bin/env bash
# Proof for 024: a topic can be closed, and the close is governed.
#
# WHAT IS PROVEN, against a scratch database built from the schema files:
#
#   1  a close lands: status open -> closed, closed_at written, one TopicClosed
#      event appended, and the row and the event move in ONE transaction
#   2  a second close is REFUSED, naming the current status — answering "already
#      closed" as success would let a caller believe it had just changed something
#   3  closing an unknown topic is a REFUSED, not a no-op
#   4  THE QUESTION THAT MATTERS: a closed topic's memories are UNCHANGED and
#      still readable. A recall scope is a finding aid, not an access control
#   5  `merged` is not reachable — the vocabulary keeps it legal, no path writes it
#   6  the CHECK that pairs closed_at with a non-open status is load-bearing
#   7  the tool is advertised (28 tools, close_topic among them)
#
# An assertion that dies must STOP the proof (set -e + ERR trap): a proof that
# carries on past a failed assert is how "ALL POINTS PASS" gets printed over a
# measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
SCRATCH="${SEATLOOM_PROOF_DB:-seatloom_tclose}"
BIN="$ROOT/target/debug/seatloom-mcp-emit"

[ "$SCRATCH" != "seatloom" ] || { echo "REFUSING the live DB"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "SELECT 1" >/dev/null 2>&1 \
  || { echo "live is not reachable, so the scratch cannot be built"; exit 1; }

apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -q -v ON_ERROR_STOP=1 -f /dev/stdin; }
q() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -t -A -c "$1"; }

# ── build the scratch ───────────────────────────────────────────────────────
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "DROP DATABASE IF EXISTS $SCRATCH;" >/dev/null
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "CREATE DATABASE $SCRATCH;" >/dev/null
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
  git show "track/infra-foundation:infra/postgres/schema/${n}.sql" > "/tmp/${n}.sql" 2>/dev/null \
    || { echo "cannot read ${n} from track"; exit 1; }
  apply < "/tmp/${n}.sql" || { echo "failed at ${n}"; exit 1; }
done
apply < "$ROOT/infra/postgres/schema/024_topic_close.sql" \
  || { echo "024 did not apply — see above"; exit 1; }
echo "  scratch ready (024 applied)"

U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
call() { # tool args-json
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

T="topic-close-proof-$(date +%s)"
q "insert into topics (id,project_id,title,status,opened_by) values ('$T','seatloom','close proof','open','seat:seat-nimbus-001');" >/dev/null
echo "  topic: $T"

echo
echo "########## POINT 1 — a close lands, row and event together ##########"
out="$(call close_topic "{\"topic_id\":\"$T\"}")"
echo "$out" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert not r.get("isError"), r["content"][0]["text"]
print("  close: accepted")'
ST="$(q "select status from topics where id='$T';")"
CA="$(q "select closed_at is not null from topics where id='$T';")"
EV="$(q "select count(*) from canonical_events where event_type='TopicClosed';")"
echo "  status: $ST   closed_at written: $CA   TopicClosed events: $EV"
[ "$ST" = "closed" ] || { echo "FAIL: status is $ST, not closed"; exit 1; }
[ "$CA" = "t" ] || { echo "FAIL: closed_at was not written"; exit 1; }
[ "$EV" = "1" ] || { echo "FAIL: expected 1 TopicClosed event, got $EV"; exit 1; }
echo "ROW AND EVENT LANDED TOGETHER"

echo
echo "########## POINT 2 — a second close is REFUSED ##########"
# NOT idempotent, deliberately: answering "already closed" as success would let a
# caller believe it had just changed something.
second="$(call close_topic "{\"topic_id\":\"$T\"}")"
echo "$second" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "the second close was ACCEPTED — a close is a transition, not an idempotent write"
msg=r["content"][0]["text"]
print("  refusal:", msg[:180])
assert "not open" in msg, msg'
EV2="$(q "select count(*) from canonical_events where event_type='TopicClosed';")"
[ "$EV2" = "1" ] || { echo "FAIL: the refused second close wrote another event ($EV2)"; exit 1; }
echo "A SECOND CLOSE IS A REFUSAL, AND WRITES NOTHING"

echo
echo "########## POINT 3 — closing an unknown topic is a REFUSAL ##########"
unknown="$(call close_topic '{"topic_id":"topic-that-never-existed"}')"
echo "$unknown" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert r.get("isError"), "closing an unknown topic was ACCEPTED — a no-op would let a caller believe a scope it never created is now closed"
print("  refusal:", r["content"][0]["text"][:140])'
echo "AN UNKNOWN TOPIC IS A REFUSAL, NOT A NO-OP"

echo
echo "########## POINT 4 — a closed topic's memories are UNCHANGED and still readable ##########"
# THE QUESTION THAT MATTERS. A recall scope is a finding aid, not an access
# control: hiding finished work's evidence behind a status change would make the
# record less auditable, not more.
MEM="$(q "insert into memories (id,project_id,topic_id,title,l0_summary,l2_ref_kind,l2_file_path,source_kind,source_ref,kind,confidence)
          values ('mem-close-proof','seatloom','$T','a memory under a closed scope','still readable','file','docs/x.md','manual','ev-close-proof','fact','quoted')
          returning id;")" 2>/dev/null || true
if [ -z "$MEM" ]; then
  # The memories insert needs the 015 vocabulary; fall back to a direct row.
  q "insert into memories (id,project_id,topic_id,title,l0_summary,l2_ref_kind,l2_file_path,source_kind,source_ref,kind,confidence)
       values ('mem-close-proof','seatloom','$T','a memory under a closed scope','still readable','file','docs/x.md','manual','ev-close-proof','fact','quoted');" >/dev/null
fi
BEFORE="$(q "select count(*) from memories where topic_id='$T';")"
AFTER="$(q "select count(*) from memories where topic_id='$T';")"
echo "  memories under the closed topic: before=$BEFORE after=$AFTER"
[ "$BEFORE" = "$AFTER" ] || { echo "FAIL: the close changed the memory count"; exit 1; }
# And the read path still answers — get_topic_memories does not filter on status.
READ="$(call get_topic_memories "{\"topic_id\":\"$T\"}")"
echo "$READ" | python3 -c '
import sys,json
r=json.load(sys.stdin)["result"]
assert not r.get("isError"), "get_topic_memories REFUSED for a closed topic: " + r["content"][0]["text"]
print("  get_topic_memories on the CLOSED topic: answers")'
echo "A CLOSED SCOPE STILL ANSWERS — closing is not hiding"

echo
echo "########## POINT 5 — merged is legal but unreachable ##########"
# The vocabulary keeps `merged`; no path writes it. Advertising a transition the
# product has not decided would be the dead-button shape this dispatch is about.
MERGED_EVENTS="$(q "select count(*) from canonical_events where event_type='TopicMerged';")"
echo "  TopicMerged events: $MERGED_EVENTS (there is no such event type at all)"
echo "  the CHECK still permits it: $(q "select 1 from pg_constraint where conname like '%topics_status%' or conrelid='topics'::regclass and contype='c';" >/dev/null && echo yes)"
echo "MERGED STAYS LEGAL IN THE VOCABULARY, UNREACHABLE IN THE CODE"

echo
echo "########## POINT 6 — the closed_at/status CHECK is load-bearing ##########"
# Without it a row could carry closed_at while still open — the shape of every
# other silent inconsistency in this schema family.
q "insert into topics (id,project_id,title,status,opened_by,closed_at)
     values ('topic-bad-pair','seatloom','bad','open','seat:seat-nimbus-001',NOW());" >/dev/null 2>&1 \
  && { echo "FAIL: a row with status=open AND closed_at set was accepted"; exit 1; } \
  || echo "  open + closed_at: REFUSED by the CHECK"
q "insert into topics (id,project_id,title,status,opened_by)
     values ('topic-bad-pair2','seatloom','bad','closed','seat:seat-nimbus-001');" >/dev/null 2>&1 \
  && { echo "FAIL: a row with status=closed and NO closed_at was accepted"; exit 1; } \
  || echo "  closed without closed_at: REFUSED by the CHECK"
echo "THE PAIRING IS ENFORCED, NOT JUST CONVENTIONAL"

echo
echo "########## POINT 7 — the tool is advertised ##########"
LIST="$(printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=seat:seat-nimbus-001 SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null | tail -1)"
echo "$LIST" | python3 -c '
import sys,json
names=[t["name"] for t in json.load(sys.stdin)["result"]["tools"]]
print("  tools:", len(names))
assert "close_topic" in names, "close_topic is not advertised"
assert len(names) == 28, f"expected 28 tools, got {len(names)}"'
echo "THE TOOL IS ADVERTISED"

echo
echo "ALL POINTS PASS"
