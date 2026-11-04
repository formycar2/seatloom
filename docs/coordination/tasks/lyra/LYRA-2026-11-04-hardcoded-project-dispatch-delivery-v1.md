# Delivery — dispatch `disp-XpAdx0a9YLL83JVoPesTj` (Lyra → Mira): four write paths still file into a hardcoded project

**Filename note.** The AGENTS.md completion protocol names this file `<dispatch-id>-delivery.md`, but
`scripts/verify-docs.sh:74` requires every file under `docs/coordination/tasks/` to match
`ROLE-YYYY-MM-DD-<topic>-vN.md`. The two rules conflict. **I am following the gate, not the protocol**, and
recording the conflict here rather than quietly grandfathering the filename: the six earlier `disp-*-delivery.md`
notes in this directory pass only because they are unchanged relative to the base ref, so the ratchet has never
looked at them — which is precisely the "a check that cannot fail" pattern. The dispatch id is in the title and
in the `status`/`tags` below, so nothing is lost by the rename.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | LYRA-2026-11-04-hardcoded-project-dispatch-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | dispatch, product, measured |

## What I dispatched

`disp-XpAdx0a9YLL83JVoPesTj` → `seat-mira-001`, `status=ready`, `project_id=seatloom`, priority 6.

**Title:** Four write paths still file into a hardcoded project: the read paths were fixed, these were not.

The payload carries two keys, matching the shape the previous dispatch used: `note` (the measured defect and
the fix shape) and `acceptance` (what a person who cannot read code must be able to run).

## Why this and not something else

The previous rounds were on the tooling line — sandbox build guards, Aegis's three fixes, my own retractions.
Those were necessary, but none of them made anything clickable. This round went back to the product line per
the heartbeat's rule, found the hardcoded-project defect, measured its blast radius, corrected my own
overstatement about it, and **this dispatch is the step that turns it into work a seat can do**.

## The finding, as measured

**The read paths were fixed project-by-project** with `require_project_id`
(`src-tauri/src/commands/mod.rs:42-48`), which **refuses** an absent `project_id` rather than falling back. Its
own doc comment records the reasoning verbatim: *a fallback that silently substitutes another project's rows is
worse than an error — the error says something is missing, while the fallback says nothing at all.*

**Four write/spawn paths never got that treatment:**

| location | command | project source |
|---|---|---|
| `src-tauri/src/commands/dispatch_cmds.rs:128` | `cmd_enqueue_dispatch` | `.env("SEATLOOM_PROJECT_ID", &state.default_project_id)` |
| `src-tauri/src/commands/dispatch_cmds.rs:197` | `cmd_plan_intent` | same |
| `src-tauri/src/commands/history_cmds.rs:59` | `call_tool` (shared helper) | same |
| `src-tauri/src/commands/session_work_cmds.rs:43` | `call_tool` (shared helper) | same |

**and `state.default_project_id` is a constant** — `src-tauri/src/main.rs:120`,
`AppState::new(db, root, "seatloom".to_string())`.

**This is the same defect `cmd_list_dispatches` had and already fixed.** Its comment at
`dispatch_cmds.rs:12-19` records it verbatim: *This command is the one that hid the defect... A fallback that
silently substitutes another project's rows is worse than an error.* **It was fixed on the read path and never
on the write path, in the same file, by the same mechanism.** 14 command files use `require_project_id`; these
four spawn points do not.

**Measured blast radius, against live today:**

| measurement | value |
|---|---|
| projects in live | 40 |
| projects with any seat binding | 4 |
| of those, not `seatloom` | 3 — `project_delivery/stepaicloud`, `project_delivery/QL`, `project_delivery/networkOrchestrator` |
| seats bound to each | exactly 1 each |
| any of those 3 also bound under `seatloom`? | no |
| their tmux sessions | all three running now |

**So every dispatch the operator creates while viewing any of those three projects is filed under
`seatloom`.** He clicks, the form closes, the list refreshes, and it is empty. Nothing tells him where it went.

## What the dispatch tells Mira, and why those constraints are in it

- **The fix shape, with the in-file pattern to copy** — `cmd_list_dispatches:22-32` already resolves and
  forwards the project. She is not asked to invent it.
- **Do not delete `state.default_project_id` or `main.rs:120`** — it has other readers and that is a separate
  decision. Leave it; stop these four from using it.
- **Mutation-test in both directions and report the counts** — delete the added `project_id` and a test must
  fail; reintroduce the silent fallback and a test must fail. This is the discipline the last three rounds
  established and it is written into the acceptance.
- **Nimbus's 124 uncommitted Rust lines are named explicitly** so nobody rebases, merges or resets his worktree.
- **No merge to track, no app restart, no live-DB write** — product code only.

## The correction I made before dispatching, and why it matters

I first wrote that the dispatch "appears on neither side." **That was wrong on one side**, and I corrected it in
a separate document (`LYRA-2026-11-04-correction-the-dispatch-is-not-invisible-on-both-sides-v1.md`) rather than
editing the delivered one. Under `seatloom` the dispatch **is** listed — grouped under its raw seat id, because
`DispatchInboxView.tsx:751-753` falls back to `seatId` and the group header at `1291-1304` prints both label and
id.

**So the accurate statement is: misfiled and undisclosed, not invisible.** I put the corrected version in the
dispatch payload, not the dramatic one. **A dispatch that overstates its defect costs the receiving seat time
distinguishing the claim from the reality.**

## Verification of the dispatch itself

```
dispatches: disp-XpAdx0a9YLL83JVoPesTj | seat-mira-001 | ready | seatloom | 2026-11-04 05:12:59
canonical_events: SeatDispatchIssued | seat:seat-lyra-001 | 2026-11-04 05:12:59   (newest)
```

The ledger carried it, which is the only proof that a dispatch exists — a dispatch described only in a reply is
a dispatch that does not exist.

## What I did not do

I did not modify `src-tauri` or `ui/src`. I did not touch Mira's worktree, Nimbus's worktree, or the primary
checkout's untracked `scripts/bind-seatloom-seats.sh`. I did not merge, restart the operator's application, or
write to the live database beyond the dispatch itself. My measurements were reads: `psql` against live for
projects, all bindings grouped by project, the three seats' binding sets, dispatches and `canonical_events`;
`tmux list-sessions`; and reading `commands/mod.rs:42-54`, `dispatch_cmds.rs:12-32, 83-132, 173-200`,
`history_cmds.rs:35-62`, `session_work_cmds.rs:19-46`, `main.rs:120`, `ui/src/lib/api.ts:176-191`,
`DispatchInboxView.tsx:751-753, 1290-1304`.
