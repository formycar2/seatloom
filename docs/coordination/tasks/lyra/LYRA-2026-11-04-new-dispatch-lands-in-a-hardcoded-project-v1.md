# Standing-duty finding, product line: the "new dispatch" form dispatches into a hardcoded project, not the one on screen. In `project_delivery/networkOrchestrator` the operator's only dispatchable seat is `seat-networkorchestrator-001`, bound to that project alone — and the dispatch it creates lands in `seatloom`, so it appears on neither side. This is a click that succeeds and a thing that vanishes.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-new-dispatch-lands-in-a-hardcoded-project-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, product, measured, live, gate |

## Why this round was on the product line

The heartbeat's rule for an idle round is to find where the product is worse than it claims, judged always by
*what happens when the user clicks*. The previous rounds were on the tooling line — sandbox build guards, Aegis's
fixes, my own retractions. Those were necessary and I do not retract them, but **none of them made anything
clickable for the operator**. This round went back to the screens.

## The defect, measured end to end

**The form is correct about its own scope. The backend is not.**

```ts
// ui/src/views/DispatchInboxView.tsx:665 — the seat list is scoped to the project on screen
.listSeats(activeProjectId)
// :874 — and the seat id is passed straight through
await api.enqueueDispatch(newSeat, newTitle.trim(), newInstr.trim(), newPrio);
```

**`api.enqueueDispatch` does not take a project id at all:**

```ts
// ui/src/lib/api.ts:181-184
enqueueDispatch: (targetSeatId: string, title: string, instructions: string, priority?: number) =>
  invoke('cmd_enqueue_dispatch', { targetSeatId, title, instructions, priority }),
```

**And the backend command resolves the project from a constant:**

```rust
// src-tauri/src/main.rs:120
AppState::new(db, root, "seatloom".to_string())
// src-tauri/src/commands/dispatch_cmds.rs:131 — used when spawning the emit binary
.env("SEATLOOM_PROJECT_ID", &state.default_project_id)
```

**So every dispatch created from the UI is filed under `seatloom`, whatever project the operator is looking at.**

## Why this is not theoretical, measured against the live database

| measurement | value |
|---|---|
| projects in live | **40** |
| the project the operator is persisted on | `seatloom` |
| bindings for `project_delivery/networkOrchestrator` | **1** — `seat-networkorchestrator-001` |
| bindings for `seat-networkorchestrator-001` in **any** project | **only** `project_delivery/networkOrchestrator` |
| that seat's tmux session | `NetworkOrchestrator-seatloom`, running now |

**So the walk is real:** the operator switches to `project_delivery/networkOrchestrator`. The dispatch page's seat
dropdown lists exactly one entry, because `listSeats(activeProjectId)` returns the bindings for that project. He
picks it, writes a task, clicks. The dispatch is created with `project_id = 'seatloom'`.

**And then it is invisible from both sides:**

- **On the project he was on** — the page reads `cmd_list_dispatches({ projectId: 'project_delivery/networkOrchestrator' })`. The new dispatch's `project_id` is `seatloom`, so it is not returned. The form closes, the list refreshes, and nothing is there.
- **On `seatloom`** — the dispatch *is* returned, but its target seat has no binding under `seatloom`, so the seat filter cannot name it. `seatName` falls back to the raw seat id at `DispatchInboxView.tsx:752`.

**A click that succeeds, and a thing that vanishes.** That is the exact class of failure the heartbeat tells me to
look for.

## What is honest nearby, so the finding is not overstated

I measured the surfaces around this and they behave correctly, which is why the defect is narrow rather than
systemic:

| surface | measured behaviour |
|---|---|
| The dispatch page's empty state | When all matches are done it says **no dispatches pending**, states that N are folded as completed, and offers a button to reveal them. It does not pretend the table is empty when it is not. |
| The acceptance section's empty state | Distinguishes live-but-empty (**nothing awaiting acceptance; deliveries appear here after a seat delivers and Flux verifies**) from paused-refresh. Both honest. |
| The live-seats empty state | States plainly that it shows only seats bound to the current project, and that the backend filters by the `-seatloom` suffix, so empty means genuinely nothing online. |
| The live-seats list itself | 9 `-seatloom` sessions are running; 6 survive `boundMirrorNames` under `seatloom` because `NetworkOrchestrator`, `QL` and `Stepaicloud` are bound to no project at all. **Correct scoping, stated in the UI.** |
| `Aegis-Supervisor` and `seatloom-waker` | Never reach the frontend — the Rust discovery drops any session not ending in `-seatloom` (`pty/mod.rs:383`). The frontend's degrade-to-stem branch is therefore unreachable from this backend, and I am not reporting it as a defect. |

**And one correction to myself, recorded because I nearly published it:** I first computed the
`boundMirrorNames` filter by hand and concluded all 9 sessions were dropped. That was wrong — `parseSeatSession`
takes the part *before* the first dash of the suffix-stripped name, so `Flux-Quality&Ops-seatloom` → `Flux`, which
is bound. **The real answer is 6 kept, 3 dropped, and the 3 are dropped because they are bound to no project at
all.** I re-ran the derivation before writing it down.

## What this needs, and who can do it

**The backend must take the project from the request, not from a constant.** That means
`cmd_enqueue_dispatch` gains a `project_id` parameter (like `cmd_list_dispatches` already has), `api.enqueueDispatch`
passes `activeProjectId`, and the emit binary is spawned with that project rather than
`state.default_project_id`.

**This is product code in `src-tauri` and `ui/src`, and it is not mine to write.** I am recording it rather than
patching it, and I am not opening a card — the heartbeat's rule is that a card is not opened without a measured
pass verdict, and nothing here has been verified as fixed.

**One thing I want on record about the constant itself.** `"seatloom".to_string()` at `main.rs:120` is not a bug
in isolation; it is the assumption that the operator lives in exactly one project. Live now holds **40**. **The
assumption was true when it was written and is not true now, and nothing in the code fails loudly to say so.**

## What I did not do

I did not modify `src-tauri`, `ui/src`, or any script. I did not create a dispatch from the UI, did not run
`--apply`, did not merge, did not restart the operator's application, and did not write to the live database. My
measurements were all reads: `ui/src/lib/api.ts:176-186`, `DispatchInboxView.tsx:633-884, 1145-1170, 1326-1360`,
`src-tauri/src/commands/dispatch_cmds.rs:22-40, 83-131`, `src-tauri/src/main.rs:120`, `src-tauri/src/state.rs:18-28`,
`pty/mod.rs:378-390`, `ui/src/lib/tmux-seats.ts:33-47`, `ui/src/lib/project-scope.ts:7-12`,
`SandboxBanner.tsx`/`AcceptanceSection.tsx` empty-state blocks, plus `psql` against live for projects, bindings,
dispatches, acceptances, operator_decisions, workitems and handoffs, and `tmux list-sessions`. The ledger check is
exit 0, `seats` is 10, `canonical_events` is 4.
