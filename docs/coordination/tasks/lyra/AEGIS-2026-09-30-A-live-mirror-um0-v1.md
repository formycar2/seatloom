# Dispatch: A — make UM0 real (live tmux mirror + mark/hide mock) → Mira, in a worktree

[Aegis -> Lyra] Route this to **Mira** when you reach it (she is idle, freshly restarted at 0k
context — the packet must be self-contained for her). This is a **pure-frontend** slice,
disjoint from the in-flight step 2 (Nimbus, Rust write API). Toward user-milestone **UM0**.

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-09-30-A-live-mirror-um0-v1 |
| status | dispatched |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| priority | P0-parallel |
| milestone | UM0 — one screen for the whole team (made real) |
| depends_on | `docs/coordination/ROADMAP.md` |
| tags | p0, dispatch, ui, um0, live-mirror |

## Why

Mr. Zhang opened the running UI and everything reads as old/mockup data except a sync
timestamp — he can't tell real from mock. This is the "read-only viewer over fixtures"
state. The live-tmux-mirror path can be made **genuinely real without capture**, and mock data
must be labelled so it is distinguishable. That is this task.

## Scope — MIRA, pure frontend (`ui/src` only)

1. **Live mirror of the real seats.** The backend commands already exist and are already
   wrapped in `ui/src/lib/api.ts`:
   - `listTmuxSessions()` → `cmd_list_tmux_sessions` (api.ts:145) — discovers the live seat
     tmux sessions (Flux-…, Lyra-…, Mira-…, Nimbus-…, Onyx-…-seatloom).
   - `attachTmuxSession(request)` → `cmd_attach_tmux_session` (api.ts:153), plus `pty_write` /
     `pty_resize` / `listLiveSessions` — byte-level pane mirror.
   Wire the main/live view to list the **real** live seats from `listTmuxSessions()` and render
   their live panes (attach + stream). Typing in a seat's real tmux must reflect in the UI.
2. **Mark or hide mock.** Seed/projection rows (workitems, sessions, documents, events) are
   fixtures with no live backing and **no `source` column** to key off. Give any seed-backed
   view an unmistakable **"SAMPLE DATA — not yet from live capture"** badge/banner, or hide it,
   so real (live mirror) vs mock (seed) is obvious at a glance. Exact UX is yours (you own UX);
   the V1 AppShell is the reference, do not replace it.

## Hard constraints

- **No backend / Rust change.** Every command you need already exists. If you conclude a
  backend change is required, **STOP and escalate to Aegis** — do not touch Rust or any
  write-API file; step 2 (Nimbus) is live in Rust and must not be disturbed.
- **Work in your OWN git worktree**, not the shared checkout (it is hosting a running
  `pnpm tauri:dev`) and not Nimbus's step-2 worktree. Create it with
  `git worktree add ../seatloom-worktrees/mira -b packet/mira/MIRA-2026-09-30-live-mirror-um0-v1 track/infra-foundation`.
  If packet files seem to vanish on checkout, recover with `git show track/infra-foundation:<path>`.
- Your session was just restarted (0k context) — everything you need is in this packet + `api.ts`.

## Acceptance (Flux, in a worktree)

- Opening the app shows the **5 real seats' live terminals**; typing in a seat's tmux reflects
  in the UI pane.
- Seed/mock data is unmistakably labelled or hidden — real vs mock is distinguishable.
- Frontend gates pass: `pnpm --dir ui typecheck`, `eslint`, `prettier --check`.
- Verified in the worktree; the shared-tree running UI is restarted onto the merged branch by
  Aegis afterward.

## Coordination note

This runs in parallel with step 2 safely because it is frontend-only, in its own worktree, on a
disjoint file set (ui/src vs Rust db/), on a disjoint data path (reads live tmux; writes
nothing). Lyra: track Mira as occupied on this so no second packet double-books her. This manual
serialization is exactly what G19 will later automate.
