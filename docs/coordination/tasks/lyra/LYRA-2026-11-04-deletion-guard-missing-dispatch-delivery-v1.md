# Delivery — dispatch `disp-AbQ4m1TKQZGLMeZ4bhwjy` (Lyra → Nimbus): the deletion guard is missing from the accept path

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | LYRA-2026-11-04-deletion-guard-missing-dispatch-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | dispatch, acceptance, measured |

**Filename note.** AGENTS.md names this `<dispatch-id>-delivery.md`; `scripts/verify-docs.sh:74` requires
`ROLE-YYYY-MM-DD-<topic>-vN.md` under `docs/coordination/tasks/`. The two conflict; I follow the gate and record
it here. The dispatch id is in the title and tags.

## What I dispatched

`disp-AbQ4m1TKQZGLMeZ4bhwjy` → `seat-nimbus-001`, priority 9. Landed `ready`.

**Title:** `merge_would_delete` runs only at card-open and never at Accept: add it to the accept path, where the
branch may have changed since.

## Why this dispatch exists

**Last round I found that the deletion guard is evaluated at the one moment when its answer is already stale.** I
recorded it and said I would not edit `acceptance_cmds.rs` myself. **That was the right boundary and the wrong
conclusion about what to do next** — a finding delivered only in a file is a finding nobody acts on, which is the
failure I have measured repeatedly today. **So this round I checked my own contract and then dispatched it.**

## The boundary, checked against my own role before dispatching

`docs/coordination/roles/LYRA.md`:

- **Owns:** authority-chain interpretation, product boundary, acceptance criteria, review findings and sign-off
  conditions, cross-role task packaging quality.
- **Primary outputs:** versioned task packets, review findings, product clarifications, sign-off decisions.
- **Must not:** silently expand scope, accept evidence-free completion claims.

**Product code is not in my outputs.** So the finding becomes a task packet with the file, the line, the pattern to
copy and the acceptance criteria — which is what this dispatch is. **And Nimbus is the right seat**: `src-tauri`'s
command layer is his, and the function he needs already exists in core with two tests.

## What the dispatch carries, and why each part is in it

- **Both measured directions**, in scratch repositories, so the recipient can distinguish an argument from a
  measurement — including the direction where the current gate refuses a safe merge, and the dangerous one where it
  permits a merge that deletes a tracked file.
- **The grep that establishes the gap** — the definition, its two tests, one call site in `request_acceptance`, and
  zero occurrences in `acceptance_cmds.rs`.
- **The exact insertion point** — `acceptance_cmds.rs:89`, immediately before the existing
  `merge_branch_into_base` call.
- **The pattern to copy**, `dispatch.rs:1820-1840`, **including the part people skip**: subtract the operator-ruled
  exemptions *before* deciding to refuse, because an empty remainder means every deletion was ruled on and there is
  nothing to defend. Reuse `is_deletion_exempt`; do not invent a second list.
- **Refuse with a message that names the paths, the branch and the base** — a refusal the operator cannot act on is
  not a refusal.
- **Why it matters beyond the guard**: with the deletion check on the accept path, the level-with-base proxy can be
  dropped from the clickability gate without losing anything. That is last round's ruling, and this is its
  prerequisite. **Two changes, not one — explicitly.**
- **His uncommitted work named and protected**: two files, neither overlapping `acceptance_cmds.rs`, which I
  checked. *Leave them alone; do not stash, reset, rebase or clean to make room.*
- **Boundary**: no merge into track, no push, no application restart, no live-database write.

## The acceptance criteria, and the one that protects the operator

1. The accept path refuses a non-exempt deletion, naming paths, branch and base.
2. **Mutation-proven in both directions with counts and test names** — delete the new guard and a test must fail;
   make the guard ignore the deletion list and a test must fail.
3. The exemption list is reused, not duplicated.
4. **The two real pending cards must still accept cleanly** — I measured their merge at 24 added / 17 modified /
   **0 deleted**, so a guard that refuses them is a broken guard. That criterion is in there because the most
   likely way this change goes wrong is by being too aggressive.
5. Gate exit codes stated, not implied.
6. **Two separate commits or dispatches** — the guard first, the clickability change second.

## Why I dispatched rather than reporting upward only

**Because the finding's whole value is that someone acts on it, and the seat that can act is idle.** Aegis has
carried findings into the operator's push channel before and that worked; but this one has a precise file, a
precise line, an in-repo pattern and a testable acceptance — **which makes it a task packet, not an escalation.**
The ruling that depends on it is already on record and awaiting the operator's approval; this dispatch does not
pre-empt that approval, it removes the blocker in front of it.

## Verification of the dispatch itself

```
dispatches: disp-AbQ4m1TKQZGLMeZ4bhwjy | seat-nimbus-001 | ready
canonical_events: SeatDispatchIssued | seat:seat-lyra-001   (newest)
```

## What I did not do

I did not edit `acceptance_cmds.rs` or any product code. I did not merge, push, accept, cancel, or re-open any
card, restart any application, or write to the live database beyond this dispatch. I did not touch Nimbus's
worktree — his two uncommitted files are still uncommitted, which I verified rather than assumed. My measurements
were reads: `roles/LYRA.md` in full, `git log` for the recent authors of `acceptance_cmds.rs`, `git status
--porcelain` and an overlap check on Nimbus's worktree, `psql` for his in-flight dispatches, and
`acceptance_cmds.rs:13, 60-95, 311-378`, `git/ops.rs:110-175`, `dispatch.rs:1820-1845`. The ledger check is exit 0,
`seats` is 10, `canonical_events` is 26.
