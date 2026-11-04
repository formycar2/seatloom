# Delivery — dispatch `disp-MQwgyf-XTKRKARKMu_jjR` (Lyra → Mira): open the two acceptance cards, and the one prerequisite blocking it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | LYRA-2026-11-04-open-the-two-cards-dispatch-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | dispatch, acceptance, measured |

**Filename note.** AGENTS.md names this `<dispatch-id>-delivery.md`; `scripts/verify-docs.sh:74` requires
`ROLE-YYYY-MM-DD-<topic>-vN.md` under `docs/coordination/tasks/`. The two conflict; I follow the gate and record
it here. The dispatch id is in the title and tags.

## What I dispatched

`disp-MQwgyf-XTKRKARKMu_jjR` → `seat-mira-001`, priority 8. Landed `ready`.

**Title:** Open the two acceptance cards: your branch is 10 commits behind track, which is the only thing
stopping you.

## Why this dispatch exists

The previous round defined the acceptance scope for her two project-scoping deliveries — the user-perceivable
check, the project to use, the build command. **It is written down and it is on my branch, which she cannot
read.** That is the failure family I have now measured several times: *a ruling not carried by a dispatch is not
delivered.*

**And while measuring the prerequisites for opening the cards, I found the actual blocker — which nobody had
named.**

## The blocker, measured

`request_acceptance`'s `BranchNotSynced` guard requires the delivery branch to **contain** the base:

```
git merge-base --is-ancestor track/infra-foundation packet/mira/knowledge-panel   ->  NO
```

**`packet/mira/knowledge-panel` is 10 commits behind `track/infra-foundation`.** All ten are Aegis's —
`fc6446a`, `301d7f2`, `b5762c4` (the three sandbox-guard fixes) plus seven docs commits. **None touches
`src-tauri` or `ui/src`, so there is nothing to reconcile; it is a sync, not a conflict resolution.**

**So she has two done dispatches, two pass verdicts that name them correctly on both channels, zero pending
cards — and still cannot open either one, because her branch does not contain the base.**

**Every other prerequisite I measured and found satisfied:**

| guard | measured |
|---|---|
| dispatch done and targets her | 3 done dispatches are hers |
| verdict exists, is `ReviewVerdictIssued`, says `pass` | both do |
| verdict is about **that** dispatch | `payload.dispatch_id` matches, **and** an `event_object_refs` row with `ref_type='dispatch'` matches — either channel satisfies it |
| no pending acceptance for the dispatch | `acceptances` = 0 |
| base and branch exist as local branches | both do |
| merge would delete no file | `git diff --diff-filter=D` across the range is empty |

**The one red was the sync, and it is hers to fix** — I am not merging into her worktree.

## What the dispatch tells her, and why each constraint is in it

- **Sync track into her branch first**, with the exact command and the note that all ten commits are Aegis's
  docs-and-scripts, so nothing needs reconciling.
- **The acceptance scope, restated in full** — because my document is on my branch and she cannot read it. I
  told her to read the document rather than trust my summary if the two disagree.
- **`ql_delivery`, not `project_delivery/QL`.** I measured the sandbox database: its QL project is named
  `ql_delivery` (the snapshot predates the rename), and `project_delivery/QL` has **no seat binding** there, so
  its dropdown would be empty and the form could not be submitted. This is the kind of detail that makes the
  difference between a card the operator can run and one that dies on step 2.
- **The build command with the branch name, never the hash** — and why: a hash resolves to a worktree 868
  commits behind that lacks the fix while the banner prints the hash you passed. Also that the script comes from
  the primary checkout on track (so Aegis's guard fixes are in it) while the compiled tree comes from her
  worktree, which is why the tooling is fixed even though her branch does not carry those fixes yet.
- **Verify the built commit line names her tree** — `758c8a4`, or her HEAD after the sync. Not another
  worktree's.
- **Keep the worktree clean until the build is done**, so the dirty guard passes without `--allow-dirty`.
- **Do not fix the sandbox database.** `project_delivery/QL` has no binding there and two of the three affected
  projects are absent entirely. `ql_delivery` is a valid stand-in for the criterion, and whether the sandbox
  should mirror those three is the operator's decision, not a task.
- **Do not fold in the cold-start acceptance.** That delivery has no verdict yet and its own A-D steps are
  undone.

## The acceptance criteria I set, so the card is checkable

1. Her branch contains `track/infra-foundation` — measured with the command, not asserted.
2. Two cards, one per dispatch, each citing the verdict event id that names it.
3. Each card's `acceptance_plan` carries the four steps in plain language with pass and fail criteria spelled
   out, naming `ql_delivery`.
4. Each card names the build command and the commit its sandbox must have been built from, **so the operator can
   tell whether the app in front of them is the thing being accepted** — that is the whole point of a card.
5. A `select` against `acceptances` showing the rows that landed, pasted as output.

## Why I did not do this myself

**I cannot open these cards.** The guard is on `target_seat_id` — all three dispatches target
`seat-mira-001` — so a Lyra call would be refused, correctly: *the seat that did the work opens the card, not
the seat that asked for it.* What I can do is remove every obstacle in front of her and state the scope, which
is what this dispatch does.

## Verification of the dispatch itself

```
dispatches: disp-MQwgyf-XTKRKARKMu_jjR | seat-mira-001 | ready
canonical_events: SeatDispatchIssued | seat:seat-lyra-001   (newest)
```

## What I did not do

I did not merge track into her branch, open a card, modify her worktree, run `--apply`, build, install, or
launch anything. I did not modify the sandbox database. I did not restart the operator's application or write to
the live database beyond this dispatch. My measurements were reads: `psql` for dispatches, verdicts and their
payloads and refs, acceptances; `git merge-base --is-ancestor` and `git log` for the ten commits and their
paths; `git diff --diff-filter=D`; `git show-ref --verify` for both branches; and reading
`dispatch.rs:1843-1875` for the guard's criteria. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 16.
