#!/usr/bin/env bash
# Final attacks: the source file gone from disk, the load-bearing wall in the
# states that DO answer, and the read-only/one-implementation checks.
set -uo pipefail
SCRATCH=seatloom_sw
U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
ROOT="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
CTX="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts"
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "$1" 2>&1; }
call() {
  local cd="${3:-$CTX}"
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom \
      SEATLOOM_HISTORY_CONTEXTS_DIR="$cd" "$BIN" 2>/dev/null | tail -1
}
text() { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }

psql "DELETE FROM memories WHERE id LIKE 'mem-flux-%';" >/dev/null
GOOD="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='both_agree' ORDER BY 1 LIMIT 1;")"
GH="${GOOD%%|*}"; GI="${GOOD#*|}"
RAWPATH="$(psql "SELECT a1_corpus_path FROM v_memory_session_join WHERE a1_corpus_path IS NOT NULL LIMIT 1;")"
ins() {
  psql "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,updated_at,kind,confidence,l2_line_start,l2_line_end)
          VALUES ('$1','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'flux record','{}','s','file','$2',repeat('a',64),'archived_session','$3',0,0,'warm',NOW(),'active',NOW(),NOW(),'decision','quoted',1,10);" >/dev/null
}

echo "########## C1 — a session whose SOURCE FILE is no longer on disk ##########"
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
mkdir -p "$FAKE/$(dirname "$RAWPATH")"
echo "  exactly ONE transcript present on disk: $FAKE/$RAWPATH"
echo "  every other session's transcript is ABSENT"
echo "  input : session_work {\"harness\":\"$GH\",\"id\":\"$GI\"}  (its file IS present)"
call session_work "{\"harness\":\"$GH\",\"id\":\"$GI\"}" "$FAKECTX" | text > /tmp/sw_present.json
ABSENT="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='both_agree' AND session_harness||'|'||session_id <> '$GH|$GI' ORDER BY 1 LIMIT 1;")"
AH="${ABSENT%%|*}"; AI="${ABSENT#*|}"
echo "  input : session_work {\"harness\":\"$AH\",\"id\":\"$AI\"}  (its file is ABSENT)"
call session_work "{\"harness\":\"$AH\",\"id\":\"$AI\"}" "$FAKECTX" | text > /tmp/sw_absent.json
python3 - <<'PY'
import json, os
for label, f in (("PRESENT", "/tmp/sw_present.json"), ("ABSENT", "/tmp/sw_absent.json")):
    d = json.load(open(f))
    r = d["records"][0]
    ondisk = os.path.isfile(os.path.expanduser(r["sourcePath"]))
    print("  %s: numbers=%s records=%d  sourcePath on disk=%s" % (label, d["numbers"], len(d["records"]), ondisk))
    print("     first record sourcePath:", r["sourcePath"])
    print("     joinState=%s resolvedBy=%s missing=%s" % (r["joinState"], r["resolvedBy"], r["missing"]))
    notes = d["notes"]
    print("     notes:")
    for n in notes: print("       -", n[:160])
    says = any("disk" in n.lower() for n in notes)
    print("     ANY note mentioning the disk / a missing file?", says)
PY
rm -rf "$FAKE" "$FAKECTX"

echo
echo "########## C2 — THE LOAD-BEARING WALL, in the states that DO answer ##########"
NOREC="$(psql "SELECT harness||'|'||id FROM archived_sessions WHERE message_count IS NOT NULL AND (harness,id) NOT IN (SELECT session_harness, session_id FROM v_memory_session_join WHERE session_id IS NOT NULL) ORDER BY 1 LIMIT 1;")"
echo "  (a) a corpus session with NO records: ($NOREC)"
call session_work "{\"harness\":\"${NOREC%%|*}\",\"id\":\"${NOREC#*|}\"}" | text > /tmp/w_a.json
DRIFT="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='path_only' LIMIT 1;")"
echo "  (b) a session whose records do not resolve: ($DRIFT)"
call session_work "{\"harness\":\"${DRIFT%%|*}\",\"id\":\"${DRIFT#*|}\"}" | text > /tmp/w_b.json
ins mem-flux-mixed "$RAWPATH" "codex:00000000-0000-0000-0000-000000000000"
MIX="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE memory_id='mem-flux-mixed';")"
echo "  (c) a session with a MIXED record (one good, one broken reference): ($MIX)"
call session_work "{\"harness\":\"${MIX%%|*}\",\"id\":\"${MIX#*|}\"}" | text > /tmp/w_c.json
python3 - <<'PY'
import json
a = json.load(open('/tmp/w_a.json')); b = json.load(open('/tmp/w_b.json')); c = json.load(open('/tmp/w_c.json'))
for label, d in (("(a) no records", a), ("(b) do not resolve", b), ("(c) mixed", c)):
    print("  %-19s numbers=%-46s records=%d session=%s" % (
        label, d["numbers"], len(d["records"]), "null" if d["session"] is None else "row"))
assert a["numbers"]["total"] == 0 and a["records"] == []
assert b["numbers"]["total"] > 0 and b["numbers"]["unresolved"] > 0 and b["records"] != []
assert a["numbers"] != b["numbers"], "the two states must not carry equal numbers"
for name, d in (("a", a), ("b", b), ("c", c)):
    assert (len(d["records"]) == 0) == (d["numbers"]["total"] == 0), (name, len(d["records"]), d["numbers"])
assert c["numbers"]["resolved"] > 0 and c["numbers"]["unresolved"] > 0, c["numbers"]
print("  the wall holds: empty list <=> total == 0, and a mixed session shows BOTH halves at once")
print("  (a) note:", [n for n in a["notes"] if "no distilled" in n][0][:120])
print("  (b) note:", [n for n in b["notes"] if "do not fully" in n][0][:120])
for r in b["records"] + c["records"]:
    if r["missing"]:
        print("   ", r["memoryId"], r["joinState"], "resolvedBy=", r["resolvedBy"], "->", r["missing"][0][:110])
PY
echo "  --- and the session's own row for a session with NO records ---"
python3 -c '
import json
a = json.load(open("/tmp/w_a.json"))
print("     session:", a["session"], "| notes about it:", [n[:90] for n in a["notes"] if "no distilled" in n])
'

echo
echo "########## C3 — read-only, and one implementation ##########"
B="$(psql "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions)||'/'||(SELECT count(*) FROM topics);")"
call session_work "{\"harness\":\"$GH\",\"id\":\"$GI\"}" >/dev/null
call session_work "{\"harness\":\"${DRIFT%%|*}\",\"id\":\"${DRIFT#*|}\"}" >/dev/null
call session_work '{"harness":"codex","id":"nope"}' >/dev/null
call memory_session '{"memory_id":"nope"}' >/dev/null
A="$(psql "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM memories)||'/'||(SELECT count(*) FROM archived_sessions)||'/'||(SELECT count(*) FROM topics);")"
echo "  events/memories/archived_sessions/topics: before=$B after=$A"
[ "$B" = "$A" ] && echo "  THE READ WROTE NOTHING" || echo "  FAIL: the read wrote"
echo "  Tauri commands (the second surface):"
grep -n '"session_work"\|"memory_session"\|pub async fn cmd_' "$ROOT/src-tauri/src/commands/session_work_cmds.rs" | sed 's/^/    /'
echo "  MCP arms reaching the core reader:"
grep -n 'session_work::session_work(db\|session_work::memory_session(db' "$ROOT/crates/seatloom-mcp-emit/src/main.rs" | sed 's/^/    /'
echo "  a second assembler anywhere?"
grep -rn "FROM v_memory_session_join" "$ROOT/src-tauri/" "$ROOT/ui/" 2>/dev/null | sed 's/^/    /' || echo "    none — no second SQL against the view"
psql "DELETE FROM memories WHERE id LIKE 'mem-flux-%';" >/dev/null
echo
echo "########## DONE ##########"
