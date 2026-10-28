#!/usr/bin/env bash
# refresh-history-ui-fixture.sh — re-measure the historical corpus through the
# SAME binary the Tauri app shells out to, and write the measured payloads into
# the UI test fixture.
#
# Why a fixture and not a live call in the test: the numbers the operator checks
# (38 projects / 183 sessions / 36680 messages / OfficeBench 49 / seatloom 14 /
# cursor 14 all undated) are a *claim about the source corpus*, and a test that
# recomputes them from a live database cannot tell "the read is right" from "the
# database moved". Pinning them makes drift visible instead of silently green.
#
# Run this after the corpus changes, then read the diff: a changed number here is
# a changed claim, and the acceptance card's arithmetic has to change with it.
#
# Usage: bash scripts/refresh-history-ui-fixture.sh
# Needs: target/debug/seatloom-mcp-emit built, DATABASE_URL reachable, and the
#        extracted contexts on disk (SEATLOOM_HISTORY_CONTEXTS_DIR).

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

BIN="target/debug/seatloom-mcp-emit"
OUT="ui/src/app-v2/panel/__fixtures__/history-measured.json"
CONTEXTS="${SEATLOOM_HISTORY_CONTEXTS_DIR:-$PWD/resume-contexts}"

[ -x "$BIN" ] || { echo "missing $BIN — build it first: cargo build -p seatloom-mcp-emit" >&2; exit 1; }
[ -d "$CONTEXTS/by-session" ] || { echo "missing $CONTEXTS/by-session — point SEATLOOM_HISTORY_CONTEXTS_DIR at the corpus" >&2; exit 1; }

export SEATLOOM_PROJECT_ID="${SEATLOOM_PROJECT_ID:-seatloom}"
export SEATLOOM_ACTOR_REF="${SEATLOOM_ACTOR_REF:-human:supervisor}"
export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"

# call_tool <tool> <args-json> -> the tool's text result, or its rejection verbatim
call_tool() {
  printf '%s\n%s\n%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
    | "$BIN" 2>/dev/null | tail -1 \
    | python3 -c 'import sys,json; d=json.load(sys.stdin); r=d["result"]; t=r["content"][0]["text"];
import sys as s
if r.get("isError"): s.stderr.write("tool refused: "+t+"\n"); s.exit(1)
s.stdout.write(t)'
}

mkdir -p "$(dirname "$OUT")"

python3 - "$OUT" "$CONTEXTS" <<'PY'
import json, os, subprocess, sys

out_path, contexts = sys.argv[1], sys.argv[2]
env = dict(os.environ)

def call(tool, args):
    lines = [
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}',
        '{"jsonrpc":"2.0","method":"notifications/initialized"}',
        json.dumps({"jsonrpc": "2.0", "id": 9, "method": "tools/call",
                    "params": {"name": tool, "arguments": args}}),
    ]
    p = subprocess.run(["target/debug/seatloom-mcp-emit"], input="\n".join(lines) + "\n",
                       capture_output=True, text=True, env=env)
    last = [l for l in p.stdout.splitlines() if l.strip()][-1]
    r = json.loads(last)["result"]
    if r.get("isError"):
        raise SystemExit(f"{tool} refused: {r['content'][0]['text']}")
    return json.loads(r["content"][0]["text"])

fixture = {
    "measuredWith": {
        "binary": "target/debug/seatloom-mcp-emit (the same one the Tauri app shells out to)",
        "contextsDir": contexts,
        "database": env.get("DATABASE_URL"),
    },
    "projects": call("list_history_projects", {}),
    "sessions": {
        pid: call("list_history_sessions", {"project_id": pid})
        for pid in [p["id"] for p in call("list_history_projects", {})["projects"]]
    },
    "turns": {
        "seatloom/claude-code/621ae382": call(
            "get_history_session_turns",
            {"harness": "claude-code", "id": "621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8"}),
        "ThoughtOnly/cursor": call(
            "get_history_session_turns",
            {"harness": "cursor", "id": "ThoughtOnly__50dd37a85ce0f751ec33e6eba750d417"}),
    },
}

p = fixture["projects"]
# Sweep every project, not a sample: the operator's card checks 183 sessions and
# 14 undated cursor sessions across the WHOLE corpus, and a four-project sample
# cannot prove either.
all_sessions = [s for payload in fixture["sessions"].values() for s in payload["sessions"]]
cursor = [s for s in all_sessions if s["harness"] == "cursor"]
fixture["theSixNumbers"] = {
    "projects": p["projectCount"],
    "sessions": p["sessionTotal"],
    "messages": p["messageTotal"],
    "officebenchSessions": fixture["sessions"]["OfficeBench"]["sessionCount"],
    "seatloomSessions": fixture["sessions"]["seatloom"]["sessionCount"],
    "cursorSessions": len(cursor),
    "cursorUndated": sum(1 for s in cursor if not s["dated"]),
    "sessionsSummedPerProject": sum(pp["sessionCount"] for pp in p["projects"]),
    "messagesSummedPerProject": sum(pp["messageCount"] or 0 for pp in p["projects"]),
}

with open(out_path, "w") as f:
    json.dump(fixture, f, ensure_ascii=False, indent=1)
    f.write("\n")
print(json.dumps(fixture["theSixNumbers"], ensure_ascii=False))
PY

echo "wrote $OUT"
