# Scenario 2.2 — wire one seat to PULL governed dispatches (seat-MCP rollout)

**Status: READY TO RUN — needs Mr. Zhang in the loop (edits a seat's pi config + restarts that
seat's pi session). Do NOT run autonomously.** This is the step that turns "Aegis enqueues a
dispatch" into "the seat actually receives it and works" — the second half of Scenario 2.

Prepared by Aegis during the 2026-10-09 autonomous window. The first half (Aegis/UI → `enqueue_dispatch`)
and the inbox view (Scenario 3) are already live; three real dispatches sit in the live `dispatches`
table waiting for a seat to pull them.

| Field | Value |
|---|---|
| author | aegis |
| date | 2026-10-09 |
| scenario | 2.2 (seat pulls via get_next_dispatch) |
| risk | **touches a seat's pi config + requires restarting that seat's pi session (loses its live in-terminal context)** |
| recommended first seat | **Mira** (frontend/step-5-preview, lowest-blast-radius) or **Nimbus** |

## The identity constraint (why per-seat, not global)

step6's non-forgeable identity requires each seat's MCP server to be launched with **its own**
`SEATLOOM_ACTOR_REF`. pi's **global** `~/.pi/agent/mcp.json` is shared by every pi session, so a
single global entry would give every seat the same identity — wrong. Each seat runs in its own
worktree (`seatloom-worktrees/<seat>`), so the clean mechanism is a **per-project** `.pi/mcp.json`
in that seat's worktree, carrying that seat's actor ref. pi reads `.pi/mcp.json` only in **trusted
projects**, so the worktree must be trusted.

## Exact steps (for ONE seat — example: Mira)

1. **Build is current:** `target/debug/seatloom-mcp-emit` exists (rebuilt 2026-10-09). Confirm:
   `ls -l /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit`

2. **Add the MCP server in that seat's worktree** (writes `.pi/mcp.json` local to the worktree):
   ```bash
   cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira
   pi mcp add seatloom -l \
     --env SEATLOOM_ACTOR_REF=seat:seat-mira-001 \
     --env DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom \
     --env SEATLOOM_PROJECT_ID=seatloom \
     -- /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit
   ```
   Then `pi mcp list` should show `seatloom` with its tools (emit_event, enqueue_dispatch,
   get_next_dispatch, complete_dispatch). If the worktree isn't trusted, pi will say so — trust it.

3. **Restart that seat's pi session** so it loads the new MCP server. **This is the disruptive bit**
   — the seat's current in-terminal context is lost. Do it when the seat is idle (all five are idle
   now). Re-launch its pi in the same tmux window (`Mira-UX/UED-seatloom`).

4. **Give the seat the pull convention.** In its launch/system prompt (or a one-line instruction):
   "When idle at end of turn, call the `get_next_dispatch` MCP tool; if it returns a dispatch, do
   the work in its payload, then call `complete_dispatch`." This is what makes the loop self-run.

5. **Watch it pull.** Mira has a real ready dispatch waiting (`disp-sc13-polish`). On its next idle
   pull it should receive that dispatch, the inbox view flips it in_flight → done, and the lifecycle
   events (`SeatDispatchDelivered`/`Completed`) appear in the timeline. That is the full Scenario 2
   loop, visible end to end.

## Verify / rollback

- Verify: `cmd_list_dispatches` (派单 tab) shows the dispatch move ready → in_flight → done; the
  timeline shows the lifecycle events.
- Rollback: `pi mcp remove seatloom -l` in that worktree + restart its pi session. The dispatch rows
  are untouched (a stuck in_flight can be reset with `UPDATE dispatches SET status='ready', started_at=NULL WHERE id=...`).

## Then

Once one seat proves the loop, roll the same per-worktree `.pi/mcp.json` out to the other seats
(each with its own `SEATLOOM_ACTOR_REF`), and wire `cmd_append_supervisor_message` so a UI message
targeted at a seat also calls `enqueue_dispatch` (so the whole thing runs from the UI, not just from
Aegis's CLI). That is the full end-state: talk in the UI → Aegis/dispatch → seat pulls → visible.
