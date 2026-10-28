"""Measure the supervisor feed's 400 newest rows: how many would have hit the old
`default: content = '• ${etype}'` branch, and how many the new translator still
leaves as a bare type name.

Pure read. Drives the same MCP binary the Tauri app spawns, over stdio, against
the live database.
"""
import json
import subprocess

BIN = "/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira/target/debug/seatloom-mcp-emit"
ENV = {
    "PATH": "/usr/bin:/bin:/usr/local/bin",
    "DATABASE_URL": "postgresql://seatloom:seatloom@localhost:5432/seatloom",
    "SEATLOOM_ACTOR_REF": "seat:seat-flux-001",
    "SEATLOOM_PROJECT_ID": "seatloom",
}
INIT = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED = '{"jsonrpc":"2.0","method":"notifications/initialized"}'

# The 22 types the OLD switch recognised (from the removed code), and the
# vocabulary the database actually accepts.
OLD_SWITCH = {
    "SessionStarted", "SessionCompleted", "SessionFailed", "SessionInterrupted",
    "ArtifactCreated", "HandoffDrafted", "HandoffSent", "HandoffReceived",
    "HandoffAccepted", "HandoffWorking", "HandoffReturned", "HandoffCompleted",
    "WorkItemCreated", "WorkItemStatusChanged", "ReviewVerdictIssued",
    "SeatDelegationIssued", "SeatDelegationClosed", "CheckpointCreated",
    "PromptDetected", "SupervisorMessage", "SeatResponse",
}


def fetch(limit):
    # The supervisor column loads through list_supervisor_messages; the closest
    # read-only equivalent is query_ledger newest-first, which is what the
    # backend itself does before reversing.
    payload = json.dumps({"jsonrpc": "2.0", "id": 9, "method": "tools/call",
                          "params": {"name": "query_ledger", "arguments": {"limit": limit}}})
    p = subprocess.run([BIN], input="\n".join([INIT, INITED, payload]) + "\n",
                       env=ENV, capture_output=True, text=True)
    last = [l for l in p.stdout.strip().splitlines() if l.strip()][-1]
    r = json.loads(last)["result"]
    if r["isError"]:
        raise SystemExit("query_ledger refused: " + r["content"][0]["text"])
    t = json.loads(r["content"][0]["text"])
    return t if isinstance(t, list) else {"events": t.get("events") or t.get("rows") or []}


d = fetch(200)
rows = d if isinstance(d, list) else (d.get("events") or d.get("rows") or [])
print("rows fetched:", len(rows))
if rows:
    print("row keys:", list(rows[0].keys()))

import collections
c = collections.Counter()
for r in rows:
    c[r.get("eventType") or r.get("event_type")] += 1

print()
print("--- composition of the %d newest rows (query_ledger ceiling is 200) ---" % len(rows))
for t, n in c.most_common():
    print("  %-26s %3d" % (t, n))

fell_through = {t: n for t, n in c.items() if t not in OLD_SWITCH}
print()
print("rows the OLD switch would have rendered as a bare `• Type`: %d / %d (%.0f%%)"
      % (sum(fell_through.values()), len(rows), 100.0 * sum(fell_through.values()) / max(len(rows), 1)))
for t, n in sorted(fell_through.items(), key=lambda kv: -kv[1]):
    print("   %-26s %3d" % (t, n))

# The new translator's default branch: does it still produce a bare type name?
print()
print("--- which of these types does the NEW translator still leave to its default branch? ---")
NEW_COVERED = {
    "SeatDispatchIssued", "SeatDispatchDelivered", "SeatDispatchCompleted",
    "SeatDispatchCancelled", "SeatDispatchFailed", "SeatDispatchRetried",
    "AcceptanceRequested", "AcceptanceGranted", "AcceptanceRejected",
    "AcceptanceCancelled", "MemoryCreated", "MemoryRecalled", "MemoryPromoted",
    "MemoryForgotten", "MemoryResolved", "TopicOpened", "SessionStarted",
    "ArtifactCreated", "HandoffSent", "HandoffAccepted", "HandoffCompleted",
    "WorkItemCreated", "WorkItemStatusChanged", "ReviewVerdictIssued",
    "SeatDelegationIssued", "SeatDelegationClosed", "SupervisorMessage",
    "SeatResponse",
}
uncovered = {t: n for t, n in c.items() if t not in NEW_COVERED}
if not uncovered:
    print("  none — every type present in these %d rows has a dedicated line" % len(rows))
else:
    print("  still on the default branch:", uncovered)
    print("  (and the default now prints the type AND its payload keys, not a bare word)")
