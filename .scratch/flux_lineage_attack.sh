#!/usr/bin/env bash
# Flux's ADVERSARIAL attack on the lineage read model — the cases Nimbus's own
# proof (crates/seatloom-mcp-emit/proof/lineage_ref_proof.sh) does NOT cover.
# That proof's seven points pass (measured separately); this attacks the seams.
#
# Scratch DB only: seatloom_lineage_ref, never the live `seatloom` database.
set -uo pipefail

DB=seatloom_lineage_ref
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=lineage-ref-proof
NIMBUS=seat-nimbus-001
CTX="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts"
ARCH=/Users/jyxc-dz-0100609/Documents/GitHub/AgentHistory

psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1" 2>&1; }
drive() {
  local actor="$1"; shift
  printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$actor" \
    SEATLOOM_PROJECT_ID="$PROJECT" SEATLOOM_INTEGRATION_BRANCH="track/infra-foundation" \
    SEATLOOM_HISTORY_CONTEXTS_DIR="${CTX_OVERRIDE:-$CTX}" "$BIN" 2>/dev/null
}
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
lref_args() { echo "{\"ref\":{\"kind\":\"$1\",\"id\":\"$2\"$( [ -n "${3:-}" ] && echo ",\"harness\":\"$3\"" )}}"; }
lref() { local args; args="$(lref_args "$1" "$2" "${3:-}")"; drive "$NIMBUS" "$INIT" "$INITED" "$(call lineage "$args")"; }
text() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }
err()  { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert r["isError"], "expected a refusal, got: "+r["content"][0]["text"]; print(r["content"][0]["text"])'; }
banner() { echo; echo "########## $* ##########"; }

# =============================================================================
banner "A1 — an archived_sessions row that EXISTS, with 021 APPLIED, but no extracted conversation"
# The LIVE table holds 272 rows, 89 of them the 2026-10-08 manifest load with
# message_count IS NULL (021 §3 normalises 182 of the 271; the other 89 are left
# exactly as the out-of-band load wrote them). Those rows EXIST on live today.
# Reproduce one on the scratch DB and ask lineage about it.
psql "DELETE FROM event_object_refs WHERE ref_id='inv-row-flux';" >/dev/null
psql "DELETE FROM archived_sessions WHERE id='inv-row-flux';" >/dev/null
psql "INSERT INTO archived_sessions (id,harness,project,raw_path,archived_at) \
        VALUES ('inv-row-flux','stepcode','fde/field_evaluation', \
                'raw/stepcode/06ef2d08-ba8f-4439-bdd0-cacd018b324f.jsonl',NOW());" >/dev/null
echo "  input : {\"ref\":{\"kind\":\"history_session\",\"id\":\"inv-row-flux\",\"harness\":\"stepcode\"}}"
echo "  row   : $(psql "SELECT id||' harness='||harness||' project='||project FROM archived_sessions WHERE id='inv-row-flux';")"
echo "  021 applied on this DB? message_count column exists = $(psql "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='archived_sessions' AND column_name='message_count');")"
echo "  output: $(lref history_session inv-row-flux stepcode | err)"
echo "  on LIVE (read-only): 272 archived_sessions rows, $(podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "SELECT count(*) FROM archived_sessions WHERE message_count IS NULL;") with message_count IS NULL"
echo "  FINDING: the row EXISTS and the migration IS applied, yet the answer instructs the operator to apply 021."

# =============================================================================
banner "A2 — the universal claim inside no_record, when ANOTHER session IS named"
# no_record.reason hardcodes: "nothing writes an event_object_ref for an archived
# session and no archived session id is ever an event referent". That is an
# assertion about the WHOLE corpus inside an answer labelled "measured:". Write
# one join key for a DIFFERENT archived session, then ask about an unnamed one.
psql "DELETE FROM event_object_refs WHERE event_id='ev-flux-join-write';" >/dev/null
psql "DELETE FROM canonical_events WHERE id='ev-flux-join-write';" >/dev/null
psql "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
        VALUES ('ev-flux-join-write','ArtifactCreated',NOW(),'${NIMBUS}','{}','${PROJECT}',1);" >/dev/null
psql "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ('ev-flux-join-write','session','ev-flux-other-archived-id');" >/dev/null
ANON=$(psql "SELECT id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='codex' ORDER BY id LIMIT 1;")
echo "  wrote : event_object_refs ('session','ev-flux-other-archived-id') — a join key, for a DIFFERENT session"
echo "  input : {\"ref\":{\"kind\":\"history_session\",\"id\":\"${ANON}\",\"harness\":\"codex\"}}"
echo "  output reason: $(lref history_session "$ANON" codex | text | python3 -c 'import sys,json; print(json.load(sys.stdin)["noRecord"]["reason"])')"
echo "  SQL   : event_object_refs naming an archived session, whole corpus = $(psql "SELECT count(*) FROM event_object_refs r JOIN archived_sessions a ON a.id=r.ref_id WHERE r.ref_type='session';")"
echo "  FINDING: the reason asserts a universal fact it did not measure, and it is now false."

# =============================================================================
banner "A3 — an acceptance card whose flux_verdict_ref names an event that does not exist"
psql "DELETE FROM acceptances WHERE id='acc-flux-dangling-verdict';" >/dev/null
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
        VALUES ('disp-flux-dangling','${NIMBUS}','seat:seat-lyra-001','${PROJECT}', \
                'the dangling-verdict card dispatch','{}','ready',5,NOW()) ON CONFLICT (id) DO NOTHING;" >/dev/null
psql "INSERT INTO acceptances (id,dispatch_id,project_id,branch,base,how_to_verify,flux_verdict_ref,status,acceptance_plan) \
        VALUES ('acc-flux-dangling-verdict','disp-flux-dangling','${PROJECT}','packet/flux/x', \
                'track/infra-foundation','look','ev-this-verdict-was-never-emitted','pending', \
                '{\"plan_version\":1,\"scenario\":\"s\",\"steps\":[],\"expected\":[]}');" >/dev/null
echo "  input : {\"ref\":{\"kind\":\"acceptance\",\"id\":\"acc-flux-dangling-verdict\"}}"
echo "  card  : flux_verdict_ref = ev-this-verdict-was-never-emitted ; that event exists in the ledger? $(psql "SELECT count(*) FROM canonical_events WHERE id='ev-this-verdict-was-never-emitted';")"
lref acceptance acc-flux-dangling-verdict | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  facts :", [f["eventType"] for f in d["facts"]])
print("  notes :", d["notes"])
print("  evidence:", [(e["kind"], e["id"]) for e in d["evidence"]])
print("  subject.fluxVerdictRef:", d["subject"]["fluxVerdictRef"])
'
echo "  FINDING: the verdict event the card names is absent from facts, evidence AND notes."

# =============================================================================
banner "A4 — the 证据在哪 slot: which kinds can leave it empty without saying so"
for k in "dispatch disp-lref-proof" "event ev-lref-verdict" "event ev-lref-delivered" \
         "acceptance acc-disp-lref-proof" "session sess-lref-proof"; do
  set -- $k
  echo "  --- $1 $2 ---"
  lref "$1" "$2" | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("    evidence:", [(e["kind"], e["id"]) for e in d["evidence"]])
print("    notes   :", d["notes"])
mentions = any("证据" in n for n in d["notes"])
print("    a note explains an empty 证据在哪?", mentions)
'
done
echo "  FINDING: dispatch and session can return evidence == [] with no note naming that slot."

# =============================================================================
banner "A5 — a session whose source file is no longer on disk"
# context_evidence() asserts "the original transcript on disk" WITHOUT checking.
# Its sibling read_history_session_turns() DOES check (source_present).
FAKE=/tmp/flux-fake-archive; FAKECTX=/tmp/flux-fake-ctx
rm -rf "$FAKE" "$FAKECTX"; mkdir -p "$FAKE" "$FAKECTX/by-session"
python3 - "$CTX" "$FAKECTX" "$FAKE" <<'PY'
import sys, pathlib, re
src, dst, fake = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
prefix = "/Users/jyxc-dz-0100609/Documents/GitHub/AgentHistory"
n = 0
for p in sorted(src.glob("by-session/*.md")):
    t = p.read_text(encoding="utf-8", errors="replace")
    t2 = re.sub(r"^(来源文件\s+)(/.*)$", lambda m: m.group(1) + fake + m.group(2)[len(prefix):], t, count=1, flags=re.M)
    n += (t2 != t)
    (dst / "by-session" / p.name).write_text(t2, encoding="utf-8")
print("    rewrote 来源文件 on %d context files to %s/..." % (n, fake))
PY
SRC=$(psql "SELECT raw_path FROM archived_sessions WHERE message_count IS NOT NULL AND harness='codex' ORDER BY id LIMIT 1;")
mkdir -p "$FAKE/$(dirname "$SRC")"
cp "$ARCH/$SRC" "$FAKE/$SRC" && echo "    present on disk: $FAKE/$SRC" || echo "    copy failed"
PRESENT=$(psql "SELECT id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='codex' AND raw_path='${SRC}';")
ABSENT=$(psql "SELECT id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='codex' AND raw_path<>'${SRC}' ORDER BY id LIMIT 1;")
export CTX_OVERRIDE="$FAKECTX"
echo "  --- PRESENT: source file IS on disk ---"
echo "  input : {\"ref\":{\"kind\":\"history_session\",\"id\":\"${PRESENT}\",\"harness\":\"codex\"}}"
lref history_session "$PRESENT" codex | python3 /tmp/flux_show_evidence.py
echo "  --- ABSENT: source file is NOT on disk ---"
echo "  input : {\"ref\":{\"kind\":\"history_session\",\"id\":\"${ABSENT}\",\"harness\":\"codex\"}}"
lref history_session "$ABSENT" codex | python3 /tmp/flux_show_evidence.py
unset CTX_OVERRIDE
rm -rf "$FAKE" "$FAKECTX"
echo "  FINDING: for the absent one the detail still says 'the original transcript on disk'."
echo "           For contrast, read_history_session_turns() computes source_present and reports it."

# =============================================================================
banner "A6 — a dependency whose dep_kind is outside the three referent_exists knows"
# dispatch_dependencies has CHECK (dep_kind IN ('event','artifact','dispatch'))
# and referent_exists() has a `_ => \"SELECT FALSE\"` arm that still binds one
# parameter. Drop the CHECK (this project has a documented out-of-band load
# history), insert a 'session' edge to a session that DOES exist, and read it.
psql "ALTER TABLE dispatch_dependencies DROP CONSTRAINT IF EXISTS dispatch_dependencies_dep_kind_vocab;" >/dev/null
psql "INSERT INTO dispatch_dependencies (dispatch_id,dep_kind,dep_value) VALUES ('disp-lref-proof','session','sess-lref-proof');" >/dev/null
echo "  edge  : dep_kind='session' dep_value='sess-lref-proof'"
echo "  that session row exists? $(psql "SELECT count(*) FROM sessions WHERE id='sess-lref-proof';")"
echo "  input : {\"ref\":{\"kind\":\"dispatch\",\"id\":\"disp-lref-proof\"}}"
echo "  output: $(lref dispatch disp-lref-proof | err)"
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id='disp-lref-proof' AND dep_kind='session';" >/dev/null
psql "ALTER TABLE dispatch_dependencies ADD CONSTRAINT dispatch_dependencies_dep_kind_vocab CHECK (dep_kind IN ('event','artifact','dispatch'));" >/dev/null
echo "  FINDING: ONE unjoinable edge destroys the WHOLE answer with a raw postgres error."
echo "           repositories.rs:710 `_ => \"SELECT FALSE\"` still receives &[&dep_value]."

# =============================================================================
banner "A7 — the other dep_kinds, existing and missing, for contrast"
for pair in "artifact ar-lref-proof" "artifact ar-never-created" "dispatch disp-lref-proof" "dispatch disp-never-issued"; do
  set -- $pair
  psql "INSERT INTO dispatch_dependencies (dispatch_id,dep_kind,dep_value) VALUES ('disp-lref-proof','$1','$2') ON CONFLICT DO NOTHING;" >/dev/null
done
lref dispatch disp-lref-proof | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
for b in d["blockedOn"]:
    print("    %-9s %-24s exists=%-5s statement=%s missing=%s" % (
        b["depKind"], b["depValue"], b["exists"], b["statement"], (b["missing"] or "")[:40]))
'
for pair in "artifact ar-lref-proof" "artifact ar-never-created" "dispatch disp-lref-proof" "dispatch disp-never-issued"; do
  set -- $pair
  psql "DELETE FROM dispatch_dependencies WHERE dispatch_id='disp-lref-proof' AND dep_kind='$1' AND dep_value='$2';" >/dev/null
done
echo "  NOTE: an artifact/dispatch referent gets a sentence that only restates kind+id."

# =============================================================================
banner "A8 — a history_session refusal that does not name the harness it looked up"
echo "  the row exists under cursor, not codex:"
psql "SELECT id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='cursor' ORDER BY id LIMIT 1;" | while read -r CID; do
  echo "  input : {\"ref\":{\"kind\":\"history_session\",\"id\":\"${CID}\",\"harness\":\"codex\"}}"
  echo "  output: $(lref history_session "$CID" codex | err)"
  echo "  (the archive is keyed by (harness, id); the refusal names only the id)"
done

# =============================================================================
banner "A9 — the acceptance whose gated dispatch is gone: which answer wins?"
psql "ALTER TABLE acceptances DROP CONSTRAINT IF EXISTS acceptances_dispatch_id_fkey;" >/dev/null
psql "DELETE FROM dispatches WHERE id='disp-lref-proof';" >/dev/null
echo "  deleted the dispatch that acc-disp-lref-proof gates (FK dropped first)"
echo "  input : {\"ref\":{\"kind\":\"acceptance\",\"id\":\"acc-disp-lref-proof\"}}"
echo "  output: $(lref acceptance acc-disp-lref-proof | err)"
echo "  FINDING: the card EXISTS. get_acceptance() INNER JOINs dispatches, so the"
echo "           'this card gates a dispatch that is gone' note is unreachable and the"
echo "           answer instead claims the acceptance itself does not exist."
echo "  (the scratch DB is disposable and was never the live one)"

echo
echo "########## ATTACK COMPLETE ##########"
