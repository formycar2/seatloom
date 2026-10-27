# Delivery: write-in guard — live path proof NOT possible from this process (stop-with-report)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-11-writein-guard-delivery-v1 |
| status | delivered |
| author | vela |
| date | 2026-10-11 |
| version | v1 |
| depends_on | VELA-2026-10-10-washing-pipeline-p2-delivery-v1, VELA-2026-10-10-washing-pipeline-v1 |
| tags | washing, writein, guard, blocked, seatloom |

Dispatch `disp-onight-writein-guard`. Disposition: **stopped with report, per
the guard's own step 1. No bulk insert attempted. `disp-onight-writein` is NOT
attempted.**

## 1. Guard step 1 — process capability check (fails)

The guard requires confirming that this seat's MCP process reports
`open_topic` and the post-restart `remember` before any write is attempted,
and explicitly forbids attempting writes that will FK-fail if it does not.

Measured from this process:

| Tool | Present? | Evidence |
|---|---|---|
| `open_topic` | **NO** | `searchTools("open_topic")` returns only `mcp__seatloom__recall`; no topic-creation tool exists in this process's surface |
| `remember` (018-typed) | **NO** | its declaration has no `kind`/`confidence`/`l2_line_start`/`l2_line_end` fields |
| `remember` (pre-018 shape) | YES (old binary) | accepts the old signature, then fails at the database |

Root cause, confirmed by the operator: this seat's tool subprocess started
**2026-10-10 21:39** and holds the program image from that moment — a binary
that predates Lane A2. The fleet-wide program swap
(`scripts/land-knowledge-live.sh`) has not run yet, so no new tool is callable
from this process until it does. Aegis's restart patrol is the path.

## 2. Verbatim rejections (sanctioned paths attempted, not bypassed)

1. `remember` with the 018-typed signature:
   `Error: Validation failed for tool "mcp__seatloom__remember": -
   l2_line_start: schema is false - l2_line_end: schema is false - root: must
   not have additional properties`
2. `remember` with the pre-018 signature (what this binary accepts):
   `memory tool rejected: postgres error: db error`

Both are consistent with the two independent infrastructure gaps: the
deployed binary predates the typed memory tools, and the live database is
still at the 014 state (no memories/topics tables; the event-type vocabulary
is still 27 values without `TopicOpened`, so `open_topic` would fail the
CHECK even if the tool existed here).

## 3. Why no path proof was attempted

The guard's own ordering forbids it: step 1 fails, and the instruction for
that case is "emit SupervisorMessage and stop — do not attempt writes that
will FK-fail". A refused write is the system protecting itself; the failure
report is the deliverable. Reported via `SupervisorMessage`
(`ev-zwKRfzHN8tpvZcf5Ze_y0`) covering this dispatch, the Part 2 close-out,
and the 17-hour stall root cause.

## 4. Staged state (unchanged, ready)

The 88 typed records remain staged and validated in
`tools/wash/.local/wash/s3/records.jsonl`: idempotent by content digest,
resumable by per-session sha256 checkpoint, deep projects-tree cycle
detection unit-proven, every record carrying an L2 line range into `raw/`
with a recomputable sha256 body digest. When the live path exists, insertion
is one `execute_plan(plan, writer=remember)` call — no re-extraction needed.

## 5. Data discipline honored

No raw SQL. No bypass files. No live-DB action. No records without L2 line
ranges. Scope unchanged (seatloom only).
