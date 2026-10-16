#!/usr/bin/env bash
# Flux's ADVERSARIAL attack on the session↔work bridge (contract
# LYRA-2026-10-16-session-work-bridge-contract-v1 §11.2 slice 5, dispatch
# disp-session-key-verify). Slices under test: Onyx's view 022 (0e32054) and
# Nimbus's read model (703c863 on packet/nimbus/gate-vs-announcement).
#
# Nimbus's own proof (proof/session_work_readmodel_proof.sh) passes; this attacks
# the cases it does NOT construct. Live-shaped scratch `seatloom_sw` only — never
# the live `seatloom` database.
set -uo pipefail

SCRATCH=seatloom_sw
U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
ROOT="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
CTX="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts"

psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "$1" 2>&1; }
call() { # tool args-json [contexts-dir]
  local cd="${3:-$CTX}"
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom \
      SEATLOOM_HISTORY_CONTEXTS_DIR="$cd" "$BIN" 2>/dev/null | tail -1
}
text() { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }
err()  { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert r["isError"], "expected a refusal, got: "+r["content"][0]["text"]; print(r["content"][0]["text"])'; }
banner() { echo; echo "########## $* ##########"; }

[ "$SCRATCH" != "seatloom" ] || { echo "REFUSING the live DB"; exit 1; }
psql "SELECT 1" >/dev/null || { echo "the scratch is not built; run Nimbus's proof first"; exit 1; }

echo "scratch: $(psql "SELECT count(*) FROM v_memory_session_join;") view rows, states: $(psql "SELECT string_agg(join_state||'='||n, ', ') FROM (SELECT join_state, count(*) n FROM v_memory_session_join GROUP BY join_state) s;")"
echo "the live-shaped scratch has NO ref_only / none / disagree / ambiguous-path row, so those"
echo "states are CONSTRUCTED below (the scratch is disposable and was never live)."

# =============================================================================
banner "A1 — a memory whose recorded session reference is misspelled AND whose path also fails (the 'none' state)"
# This is the silent break in its purest form: a memory that IS an
# archived_session record but reaches no session by either key.
psql "DELETE FROM memories WHERE id IN ('mem-flux-none','mem-flux-refonly','mem-flux-disagree','mem-flux-ambig','mem-flux-nocolon','mem-flux-emptyid','mem-flux-multicolon','mem-flux-gonefile');" >/dev/null
# A real corpus session to point the good key at.
GOOD="$(psql "SELECT harness||'|'||id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='claude-code' ORDER BY id LIMIT 1;")"
GH="${GOOD%%|*}"; GI="${GOOD#*|}"
OTHER="$(psql "SELECT harness||'|'||id FROM archived_sessions WHERE message_count IS NOT NULL AND harness='codex' ORDER BY id LIMIT 1;")"
OH="${OTHER%%|*}"; OI="${OTHER#*|}"
echo "  good key: ($GH, $GI)   other: ($OH, $OI)"

ins() { # id l2_file_path source_ref
  psql "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,updated_at,kind,confidence,l2_line_start,l2_line_end)
          VALUES ('$1','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'flux attack record','{}','summary','file','$2',repeat('a',64),'archived_session','$3',0,0,'warm',NOW(),'active',NOW(),NOW(),'decision','quoted',1,10)
         ON CONFLICT (id) DO UPDATE SET l2_file_path=EXCLUDED.l2_file_path, source_ref=EXCLUDED.source_ref;" >/dev/null
}
ins mem-flux-none      "/nonexistent/path/does-not-exist.jsonl"            "claude-code:00000000-0000-0000-0000-000000000000"
ins mem-flux-refonly   "/nonexistent/path/does-not-exist.jsonl"            "$GH:$GI"
RAWPATH="$(psql "SELECT a1_corpus_path FROM v_memory_session_join WHERE a1_corpus_path IS NOT NULL LIMIT 1;")"
ins mem-flux-nocolon   "$RAWPATH"  "no-colon-at-all"
ins mem-flux-emptyid   "$RAWPATH"  "claude-code:"
ins mem-flux-multicolon "$RAWPATH"  "stepcode:claude:$(psql "SELECT id FROM archived_sessions WHERE harness='stepcode' LIMIT 1;")"
echo "  constructed: none, ref_only, no-colon, empty-id, multi-colon"
echo "  states now: $(psql "SELECT string_agg(join_state||'='||n, ', ') FROM (SELECT join_state, count(*) n FROM v_memory_session_join GROUP BY join_state) s;")"
echo
echo "--- A1a the 'none' memory: does the read say what is missing for BOTH keys? ---"
call memory_session '{"memory_id":"mem-flux-none"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"], "| session:", d["session"])
for m in d["missing"]: print("  missing:", m)
for n in d["notes"]: print("  note:", n[:150])
'
echo
echo "--- A1b the reverse read on the same memory (a memory whose session does not resolve) ---"
call memory_session '{"memory_id":"mem-flux-none"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
assert d["session"] is None, "the session did not resolve, so it must be null"
assert any("NEITHER recorded key resolves" in n for n in d["notes"]), d["notes"]
assert any("different state from" in n for n in d["notes"]), d["notes"]
print("  session is null AND the note says this is NOT \"this memory has no session\"")
'
echo
echo "--- A1c is that memory visible in the drift number? ---"
echo "  drift (a2 unresolved) = $(psql "SELECT count(*) FROM v_memory_session_join WHERE NOT a2_resolves;")"
echo "  join_state 'none' rows = $(psql "SELECT count(*) FROM v_memory_session_join WHERE join_state='none';")"

# =============================================================================
banner "A2 — the other constructed states, each naming what is missing"
for m in mem-flux-refonly mem-flux-nocolon mem-flux-emptyid mem-flux-multicolon; do
  echo "--- $m ---"
  call memory_session "{\"memory_id\":\"$m\"}" | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"])
print("  session:", (d["session"] or {}).get("harness"), (d["session"] or {}).get("id"))
for x in d["missing"]: print("  missing:", x)
'
done

# =============================================================================
banner "A3 — a session whose source file is no longer on disk"
# The dispatch: "show the record, say the file is missing; do not hide the record".
# Point the contexts dir at a copy whose one transcript is removed, and read a
# session whose records cite that path.
FAKE=/tmp/flux-sw-fake; FAKECTX=/tmp/flux-sw-fake-ctx
rm -rf "$FAKE" "$FAKECTX"; mkdir -p "$FAKE" "$FAKECTX/by-session"
python3 - "$CTX" "$FAKECTX" "$FAKE" <<'PY'
import sys, pathlib, re
src, dst, fake = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
prefix = "/Users/jyxc-dz-0100609/Documents/GitHub/AgentHistory"
for p in sorted(src.glob("by-session/*.md")):
    t = p.read_text(encoding="utf-8", errors="replace")
    t = re.sub(r"^(来源文件\s+)(/.*)$", lambda m: m.group(1) + fake + m.group(2)[len(prefix):], t, count=1, flags=re.M)
    (dst / "by-session" / p.name).write_text(t, encoding="utf-8")
PY
SRCP="$(psql "SELECT a1_corpus_path FROM v_memory_session_join WHERE a1_corpus_path IS NOT NULL AND join_state='both_agree' LIMIT 1;")"
mkdir -p "$FAKE/$(dirname "$SRCP")"
echo "  exactly one transcript present on disk: $FAKE/$SRCP"
TARGET="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='both_agree' AND a1_corpus_path='$SRCP' LIMIT 1;")"
TH="${TARGET%%|*}"; TI="${TARGET#*|}"
echo "  the session whose records cite it: ($TH, $TI)"
echo "  input : session_work {\"harness\":\"$TH\",\"id\":\"$TI\"}  with the file ABSENT"
call session_work "{\"harness\":\"$TH\",\"id\":\"$TI\"}" "$FAKECTX" | text | python3 -c '
import sys, json, os
d = json.load(sys.stdin)
print("  numbers:", d["numbers"])
print("  records:", len(d["records"]))
r = d["records"][0]
print("  first record sourcePath:", r["sourcePath"])
print("  ...is that path on disk?", os.path.isfile(os.path.expanduser(r["sourcePath"])))
print("  joinState:", r["joinState"], "| resolvedBy:", r["resolvedBy"], "| missing:", r["missing"])
print("  notes:")
for n in d["notes"]: print("    -", n[:170])
says = any("disk" in n.lower() or "not on disk" in n or "missing" in n.lower() for n in d["notes"])
print("  ANY note or field saying the source file is missing?", says)
'
rm -rf "$FAKE" "$FAKECTX"

# =============================================================================
banner "A4 — looking a session up by TOPIC ID"
# `wash-seg-cc-621ae382` is a session id truncated to 8 chars and formatted into a
# string. The dispatch: must be refused, or explicitly labelled derived.
TID="$(psql "SELECT topic_id FROM v_memory_session_join LIMIT 1;")"
echo "  a real topic id from the view: $TID"
echo "  input : session_work {\"harness\":\"claude-code\",\"id\":\"$TID\"}"
call session_work "{\"harness\":\"claude-code\",\"id\":\"$TID\"}" | err | sed 's/^/    /'
echo "  input : memory_session {\"memory_id\":\"$TID\"}"
call memory_session "{\"memory_id\":\"$TID\"}" | text | sed 's/^/    /'
echo "  the truncated form the contract names: wash-sess-cc-621ae382"
call session_work '{"harness":"claude-code","id":"wash-sess-cc-621ae382"}' | err | sed 's/^/    /'

# =============================================================================
banner "A5 — an empty topic, and a session id that does not exist"
echo "  topics with no memories on this scratch: $(psql "SELECT count(*) FROM topics t WHERE NOT EXISTS (SELECT 1 FROM memories m WHERE m.topic_id=t.id);")"
echo "  (a topic is never a key, so an empty topic is unreachable rather than answerable)"
echo "  input : session_work {\"harness\":\"codex\",\"id\":\"does-not-exist-at-all\"}"
call session_work '{"harness":"codex","id":"does-not-exist-at-all"}' | err | sed 's/^/    /'
echo "  input : session_work {\"harness\":\"\",\"id\":\"x\"}  (empty harness)"
call session_work '{"harness":"","id":"x"}' | err | sed 's/^/    /'
echo "  input : session_work {\"harness\":\"codex\",\"id\":\"   \"}  (whitespace id)"
call session_work '{"harness":"codex","id":"   "}' | err | sed 's/^/    /'

# =============================================================================
banner "A6 — THE LOAD-BEARING WALL, attacked rather than asserted"
# Can "no records" and "records that do not resolve" ever be made to look alike?
NOREC="$(psql "SELECT harness||'|'||id FROM archived_sessions WHERE message_count IS NOT NULL AND (harness,id) NOT IN (SELECT session_harness, session_id FROM v_memory_session_join WHERE session_id IS NOT NULL) ORDER BY harness, id LIMIT 1;")"
echo "  (a) a corpus session with NO records: ($NOREC)"
call session_work "{\"harness\":\"${NOREC%%|*}\",\"id\":\"${NOREC#*|}\"}" | text > /tmp/sw_a.json
echo "  (b) a session whose records do not resolve: (codex, rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d)"
call session_work '{"harness":"codex","id":"rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d"}' | text > /tmp/sw_b.json
# (c) a session where SOME records resolve and some do not — the mixed case.
psql "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,updated_at,kind,confidence,l2_line_start,l2_line_end)
        VALUES ('mem-flux-mixed','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'flux mixed record','{}','summary','file','/nonexistent/mixed.jsonl',repeat('a',64),'archived_session','$GH:$GI',0,0,'warm',NOW(),'active',NOW(),NOW(),'decision','quoted',1,10)
       ON CONFLICT (id) DO UPDATE SET l2_file_path=EXCLUDED.l2_file_path, source_ref=EXCLUDED.source_ref;" >/dev/null
MIX="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE memory_id='mem-flux-mixed';")"
echo "  (c) a session with a MIXED record: ($MIX)"
call session_work "{\"harness\":\"${MIX%%|*}\",\"id\":\"${MIX#*|}\"}" | text > /tmp/sw_c.json
python3 - <<'PY'
import json
a = json.load(open('/tmp/sw_a.json'))
b = json.load(open('/tmp/sw_b.json'))
c = json.load(open('/tmp/sw_c.json'))
print("  (a) no records      : numbers=%s  records=%d  session=%s" % (a["numbers"], len(a["records"]), a["session"]))
print("  (b) do not resolve  : numbers=%s  records=%d" % (b["numbers"], len(b["records"])))
print("  (c) mixed           : numbers=%s  records=%d" % (c["numbers"], len(c["records"])))
# The wall, tested from every side.
assert a["numbers"]["total"] == 0 and a["records"] == []
assert b["numbers"]["total"] > 0 and b["numbers"]["unresolved"] > 0 and b["records"] != []
assert a["numbers"] != b["numbers"], "the two states must not carry equal numbers"
# Can an empty list ever accompany a non-zero total?
for name, d in (("a", a), ("b", b), ("c", c)):
    assert (len(d["records"]) == 0) == (d["numbers"]["total"] == 0), \
        (name, len(d["records"]), d["numbers"])
# And can a non-empty list accompany total == 0?
assert not (len(c["records"]) > 0 and c["numbers"]["total"] == 0)
# The mixed case keeps the two halves of the wall visible at once.
assert c["numbers"]["resolved"] > 0 and c["numbers"]["unresolved"] > 0, c["numbers"]
print("  the wall holds from all three sides: empty list <=> total == 0, and a mixed session")
print("  shows BOTH resolved and unresolved at once rather than collapsing to one")
PY
echo "  --- and each unresolved record names which key and the exact value ---"
python3 -c '
import json
for f, label in (("/tmp/sw_b.json","(b)"), ("/tmp/sw_c.json","(c)")):
    d = json.load(open(f))
    for r in d["records"]:
        if r["missing"]:
            print("   ", label, r["memoryId"], r["joinState"], "resolvedBy=", r["resolvedBy"])
            for m in r["missing"]: print("        missing:", m)
'

# =============================================================================
banner "A7 — the disagree state and the ambiguous path, both constructed"
psql "DELETE FROM memories WHERE id='mem-flux-disagree';" >/dev/null
psql "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,updated_at,kind,confidence,l2_line_start,l2_line_end)
        VALUES ('mem-flux-disagree','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'flux disagree record','{}','summary','file','$RAWPATH',repeat('a',64),'archived_session','$OH:$OI',0,0,'warm',NOW(),'active',NOW(),NOW(),'decision','quoted',1,10)
       ON CONFLICT (id) DO UPDATE SET l2_file_path=EXCLUDED.l2_file_path, source_ref=EXCLUDED.source_ref;" >/dev/null
GOODPATH="$RAWPATH"
echo "  a memory whose PATH resolves to ($GH, $GI) but whose REFERENCE resolves to ($OH, $OI)"
call memory_session '{"memory_id":"mem-flux-disagree"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"], "| session:", (d["session"] or {}).get("harness"), (d["session"] or {}).get("id"))
for m in d["missing"]: print("  missing:", m)
'
echo "  ...and it is attributed to BOTH sessions (each counting it unresolved):"
for pair in "$GH:$GI" "$OH:$OI"; do
  H="${pair%%:*}"; I="${pair#*:}"
  echo -n "    ($H, $I) -> "
  call session_work "{\"harness\":\"$H\",\"id\":\"$I\"}" | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
states = [r["joinState"] for r in d["records"]]
print("numbers:", d["numbers"], "| disagree rows:", states.count("disagree"))
'
done
echo
echo "  --- the ambiguous path: a second archived_sessions row declaring the same raw_path ---"
psql "INSERT INTO archived_sessions (id,harness,project,raw_path,archived_at) VALUES ('flux-ambig-twin','claude-code','seatloom','$GOODPATH',NOW()) ON CONFLICT (id,harness) DO NOTHING;" >/dev/null
echo "  now $(psql "SELECT count(*) FROM archived_sessions WHERE raw_path='$GOODPATH';") rows declare raw_path=$GOODPATH"
call memory_session '{"memory_id":"mem-flux-disagree"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"], "| session:", (d["session"] or {}).get("harness"), (d["session"] or {}).get("id"))
for m in d["missing"]: print("  missing:", m)
'
echo "  and the memory is NOT double-counted: view rows = $(psql "SELECT count(*) FROM v_memory_session_join;")"
psql "DELETE FROM archived_sessions WHERE id='flux-ambig-twin' AND harness='claude-code';" >/dev/null
echo "  (the twin row removed; the scratch is disposable)"

# =============================================================================
banner "A8 — read-only, and one implementation"
B="$(psql "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions)||'/'||(SELECT count(*) FROM topics);")"
call session_work '{"harness":"codex","id":"nope"}' >/dev/null
call session_work "{\"harness\":\"$GH\",\"id\":\"$GI\"}" >/dev/null
call memory_session '{"memory_id":"mem-flux-none"}' >/dev/null
call memory_session '{"memory_id":"nope"}' >/dev/null
A="$(psql "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions)||'/'||(SELECT count(*) FROM topics);")"
echo "  events/memories/archived_sessions/topics: before=$B after=$A"
[ "$B" = "$A" ] && echo "  THE READ WROTE NOTHING" || echo "  FAIL: the read wrote"
echo "  Tauri surface:"
grep -n "session_work\|memory_session" "$ROOT/src-tauri/src/commands/session_work_cmds.rs" | grep -E '"session_work"|"memory_session"|fn cmd_' | sed 's/^/    /'
echo "  MCP arms:"
grep -n 'session_work::session_work(db\|session_work::memory_session(db' "$ROOT/crates/seatloom-mcp-emit/src/main.rs" | sed 's/^/    /'
echo "  tools advertised: $(call memory_session '{"memory_id":"x"}' >/dev/null; printf '%s\n%s\n%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":9,"method":"tools/list"}' | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null | tail -1 | python3 -c 'import sys,json; t=[x["name"] for x in json.load(sys.stdin)["result"]["tools"]]; print(len(t), "tools;", "session_work" in t and "memory_session" in t and "both advertised")')"

echo
echo "########## ATTACK COMPLETE ##########"
