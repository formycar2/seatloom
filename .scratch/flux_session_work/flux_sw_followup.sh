#!/usr/bin/env bash
# Follow-up: the two states that DO NOT panic (ambiguity, disagree) and the exact
# boundary of the one that does.
set -uo pipefail
SCRATCH=seatloom_sw
U="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}"
ROOT="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "$1" 2>&1; }
call() {
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | DATABASE_URL="$U" SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom "$BIN" 2>/dev/null | tail -1
}
text() { python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }

psql "DELETE FROM memories WHERE id LIKE 'mem-flux-%';" >/dev/null
GOOD="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='both_agree' LIMIT 1;")"
GH="${GOOD%%|*}"; GI="${GOOD#*|}"
OTHER="$(psql "SELECT session_harness||'|'||session_id FROM v_memory_session_join WHERE join_state='both_agree' AND session_harness <> '$GH' LIMIT 1;")"
OH="${OTHER%%|*}"; OI="${OTHER#*|}"
RAWPATH="$(psql "SELECT a1_corpus_path FROM v_memory_session_join WHERE a1_corpus_path IS NOT NULL LIMIT 1;")"
ins() {
  psql "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,updated_at,kind,confidence,l2_line_start,l2_line_end)
          VALUES ('$1','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'flux record','{}','s','file','$2',repeat('a',64),'archived_session','$3',0,0,'warm',NOW(),'active',NOW(),NOW(),'decision','quoted',1,10);" >/dev/null
}
echo "good=($GH,$GI) other=($OH,$OI) rawpath=$RAWPATH"
echo
echo "########## B1 — the AMBIGUOUS path: two rows declare the same raw_path, A2 resolves ##########"
ins mem-flux-ambig "$RAWPATH" "$GH:$GI"
psql "INSERT INTO archived_sessions (id,harness,project,raw_path,archived_at) VALUES ('flux-ambig-twin','claude-code','seatloom','$RAWPATH',NOW()) ON CONFLICT (id,harness) DO NOTHING;" >/dev/null
echo "  rows declaring that raw_path: $(psql "SELECT count(*) FROM archived_sessions WHERE raw_path='$RAWPATH';")"
call memory_session '{"memory_id":"mem-flux-ambig"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"], "| session:", (d["session"] or {}).get("harness"), (d["session"] or {}).get("id"))
for m in d["missing"]: print("  missing:", m)
'
echo "  the memory is NOT double-counted: view rows = $(psql "SELECT count(*) FROM v_memory_session_join;") (was 90)"
psql "DELETE FROM archived_sessions WHERE id='flux-ambig-twin' AND harness='claude-code';" >/dev/null

echo
echo "########## B2 — the DISAGREE state: both keys resolve, to DIFFERENT rows ##########"
ins mem-flux-disagree "$RAWPATH" "$OH:$OI"
call memory_session '{"memory_id":"mem-flux-disagree"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("  joinState:", d["joinState"], "| resolvedBy:", d["resolvedBy"])
print("  session shown (A1 primary):", (d["session"] or {}).get("harness"), (d["session"] or {}).get("id"))
for m in d["missing"]: print("  missing:", m)
'
echo "  ...and it is attributed to BOTH sessions, each counting it unresolved:"
for pair in "$GH:$GI" "$OH:$OI"; do
  H="${pair%%:*}"; I="${pair#*:}"
  echo -n "    ($H, $I) -> "
  call session_work "{\"harness\":\"$H\",\"id\":\"$I\"}" | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
st = [r["joinState"] for r in d["records"]]
print("numbers:", d["numbers"], "| disagree rows:", st.count("disagree"))
'
done

echo
echo "########## B3 — the exact boundary of the panic ##########"
# (a) the path resolves to ZERO rows -> a1_path_rows IS NULL -> a1_resolves IS NULL -> PANIC
ins mem-flux-zerorows "raw/claude-code/no-such-session-at-all.jsonl" "$GH:$GI"
echo "  (a) a1_corpus_path = raw/claude-code/no-such-session-at-all.jsonl (the bare form, matching no row)"
podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "SELECT '      a1_corpus_path='||coalesce(a1_corpus_path,'<NULL>')||' a1_path_rows='||coalesce(a1_path_rows::text,'<NULL>')||' a1_resolves IS NULL='||(a1_resolves IS NULL)||' join_state='||join_state FROM v_memory_session_join WHERE memory_id='mem-flux-zerorows';"
echo -n "      the read: "
OUT="$(call memory_session '{"memory_id":"mem-flux-zerorows"}' 2>&1)"
if printf '%s' "$OUT" | grep -q '"result"'; then printf '%s' "$OUT" | text | python3 -c 'import sys,json; d=json.load(sys.stdin); print("answered, joinState=", d["joinState"])'; else echo "NO RESPONSE — the process panicked (stdin closed by a dead server)"; fi
# (b) the path is neither of the two accepted forms -> a1_corpus_path IS NULL -> same panic
ins mem-flux-badform "/some/other/prefix/file.jsonl" "$GH:$GI"
echo "  (b) l2_file_path = /some/other/prefix/file.jsonl (neither the ~-rooted nor the bare form)"
podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -tAc "SELECT '      a1_corpus_path='||coalesce(a1_corpus_path,'<NULL>')||' a1_resolves IS NULL='||(a1_resolves IS NULL)||' join_state='||join_state FROM v_memory_session_join WHERE memory_id='mem-flux-badform';"
echo -n "      the read: "
OUT="$(call memory_session '{"memory_id":"mem-flux-badform"}' 2>&1)"
if printf '%s' "$OUT" | grep -q '"result"'; then printf '%s' "$OUT" | text | python3 -c 'import sys,json; d=json.load(sys.stdin); print("answered, joinState=", d["joinState"])'; else echo "NO RESPONSE — the process panicked"; fi
# (c) the path resolves to exactly one row -> fine, even with a broken reference
ins mem-flux-onerow "$RAWPATH" "codex:00000000-0000-0000-0000-000000000000"
echo "  (c) the same bare form but pointing at a row that DOES exist, with a broken reference"
echo -n "      the read: "
call memory_session '{"memory_id":"mem-flux-onerow"}' | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
print("answered, joinState=", d["joinState"], "| resolvedBy=", d["resolvedBy"])
for m in d["missing"]: print("        missing:", m)
'
psql "DELETE FROM memories WHERE id LIKE 'mem-flux-%';" >/dev/null
echo
echo "########## the panic boundary is exactly: a1_path_rows IS NULL ##########"
