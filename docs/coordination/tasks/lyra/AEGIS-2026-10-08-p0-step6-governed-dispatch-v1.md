# Dispatch: P0 step 6 — governed dispatch (G19), end-state + fixes the dropped-hop problem

[Aegis -> Lyra] step 4 (MCP emit, af3a0f7) is merged. Next end-state step: **governed dispatch
(G19)** — and it is the fix for the very problem that has made me merge/ nudge by hand all day
(seats finish one hop and stop; dispatches get dropped). Route: Onyx (schema/inbox) + Nimbus
(scheduler/delivery). Full refined spec is in `docs/coordination/ROADMAP.md` (G19 row + §2b/§3 step 6).

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-10-08-p0-step6-governed-dispatch |
| status | dispatched |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | P0 — governed dispatch (no dropped hops; ordered UI/Aegis commands) |
| depends_on | step 4 MCP emit (af3a0f7, merged), write API A1 |
| tags | p0, dispatch, g19, governed-dispatch, scheduler |

## What it is (refined G19 — not just a queue)
A per-seat inbox + a **ready-aware scheduler**, so a seat auto-pulls its next task instead of a
human nudging each hop:
1. **Inbox + `depends_on`** (Onyx, schema): a `dispatches` table — target seat, payload/ref, status
   `ready|blocked|in_flight|done`, `depends_on` (events/artifacts that must exist first). Additive,
   idempotent migration (next slot after 011); value-domain contract verbatim.
2. **Scheduler + delivery** (Nimbus): a seat, when idle, pulls the **next ready, highest-priority**
   item (blocked items do NOT head-of-line-block; one in-flight per seat). Delivery via the **MCP
   pull** model — extend step 4's `seatloom-mcp-emit` with a `get_next_dispatch()` tool the seat
   calls at end of turn — so there is NO mid-turn tmux injection. Every dispatch/delivery is itself
   an emitted event (reuse step 4).
3. **Cycle / stall detection** (Nimbus): detect a cross-seat `depends_on` cycle or an all-blocked
   stall and **escalate to Aegis** (emit an event + park in daily log) rather than hang.

## First slice (scope tight; decide the rest as reached)
Onyx: the `dispatches` inbox migration + `depends_on`. Nimbus: `get_next_dispatch()` MCP tool +
ready-aware pull for ONE seat, one dispatch end-to-end (Aegis/Lyra enqueue → seat idle → pulls →
acts → marks done, all as events). Auto-resolution/preemption/priority tuning → later.

## Acceptance (Flux, real Postgres)
Enqueue two dispatches for a seat, one `blocked` on an event not yet present; the seat idle-pulls the
**ready** one (skips the blocked), acts, marks done; when the dependency event lands, the blocked one
becomes ready and is pulled next. A fabricated cross-seat cycle is detected + surfaced, not hung.

## Why now
This removes the manual-nudge tax (Aegis had to merge step 4 by hand because Lyra dropped the merge
hop) and is the substrate for UI/Aegis-issued commands to be delivered safely and in order — the
"operate via Aegis" half of the end-state. Lyra: packetize to Onyx + Nimbus, dispatch both, drive to
merge. I am watching and will still intervene, but this is the step that lets the loop self-run.
