import sys, json, collections

BIN = "/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit"
ENV = {
    "PATH": "/usr/bin:/bin:/usr/local/bin",
    "DATABASE_URL": "postgresql://seatloom:seatloom@localhost:5432/seatloom",
    "SEATLOOM_ACTOR_REF": "seat:seat-flux-001",
    "SEATLOOM_PROJECT_ID": "seatloom",
    "SEATLOOM_HISTORY_CONTEXTS_DIR": "/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts",
}
INIT = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED = '{"jsonrpc":"2.0","method":"notifications/initialized"}'


def call(tool, args):
    import subprocess
    payload = json.dumps({"jsonrpc": "2.0", "id": 9, "method": "tools/call",
                          "params": {"name": tool, "arguments": args}})
    p = subprocess.run([BIN], input="\n".join([INIT, INITED, payload]) + "\n",
                       env=ENV, capture_output=True, text=True)
    last = [l for l in p.stdout.strip().splitlines() if l.strip()][-1]
    r = json.loads(last)["result"]
    if r["isError"]:
        return {"__error__": r["content"][0]["text"]}
    return json.loads(r["content"][0]["text"])


projs = call("list_history_projects", {})
bad_order = []
interleaved = []
sums_ok = True
for p in projs["projects"]:
    s = call("list_history_sessions", {"project_id": p["id"]})
    if "__error__" in s:
        print("ERROR", p["id"], s["__error__"][:100]); continue
    rows = s["sessions"]
    # dated first, then undated; never interleaved
    seen_undated = False
    for r in rows:
        if not r["dated"]:
            seen_undated = True
        elif seen_undated:
            interleaved.append(p["id"])
            break
    # dated block newest-first
    dated = [r["firstTs"] for r in rows if r["dated"]]
    if dated != sorted(dated, reverse=True):
        bad_order.append(p["id"])
    if s["sessionCount"] != len(rows):
        sums_ok = False
    if p["sessionCount"] != len(rows):
        sums_ok = False

print("projects whose dated block is NOT newest-first:", bad_order or "none")
print("projects where a dated row appears AFTER an undated one:", interleaved or "none")
print("per-project counts agree with the project list and with sessionCount:", sums_ok)
print("sum of per-project sessionCount:", sum(p["sessionCount"] for p in projs["projects"]))
print("sum of per-project messageCount:", sum(p["messageCount"] for p in projs["projects"]))

print()
print("=== a real session's turns (line numbers from the ORIGINAL file) ===")
t = call("get_history_session_turns", {"harness": "claude-code",
                                       "id": "621ae382-d6c1-4f0c-b"})
print("keys:", list(t.keys()))
print("turns:", len(t["session"]["turns"]))
for turn in t["session"]["turns"][:5]:
    print("   line", turn["line"], "| context lines", turn["contextLineStart"], "-", turn["contextLineEnd"],
          "| chars", turn["chars"], "| label", turn.get("label"))
print("   ...")
for turn in t["session"]["turns"][-2:]:
    print("   line", turn["line"], "| context lines", turn["contextLineStart"], "-", turn["contextLineEnd"],
          "| chars", turn["chars"])
print("sourcePresent:", t["session"]["sourcePresent"], "| countsAgree:", t["session"]["countsAgree"])
print("sourcePath:", t["session"]["sourcePath"])
print("contextPath:", t["session"]["contextPath"])
print()
print("=== verify those line numbers against the ORIGINAL file on disk ===")
import os
sp = t["session"]["sourcePath"]
print("original file exists:", os.path.isfile(sp))
if os.path.isfile(sp):
    lines = open(sp, encoding="utf-8", errors="replace").read().splitlines()
    print("original file has", len(lines), "lines")
    for turn in t["session"]["turns"][:3]:
        ln = turn["line"]
        print("   turn says line", ln, "-> original file line", ln, ":", repr(lines[ln - 1][:90]) if ln <= len(lines) else "OUT OF RANGE")
