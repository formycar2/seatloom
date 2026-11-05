# Standing-duty verification of the deletion guard on the branch that actually carries the card, mutation-tested in both directions — and two of my own mutations were invalid before I got one that was not: **the card `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy` exists, is pending, is clickable, cites a valid verdict, and its branch `packet/nimbus/deletion-guard-accept` carries the guard.** I verified the guard bites in both directions on that branch. **But my first two attempts at the reorder mutation did not mutate anything, and the tests stayed 5/5 both times — and I nearly recorded "the reorder mutation survives" about a mutation I had failed to apply.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-the-card-exists-and-two-of-my-mutations-did-not-mutate-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured, mutation |

## The card, measured

| field | measured |
|---|---|
| id | `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy` |
| status | **`pending`** |
| branch | `packet/nimbus/deletion-guard-accept` @ `bc4c0a4` |
| base | `track/infra-foundation` |
| verdict ref | `ev-hW75U60DweOY5-qM5pc5r` — **exists**, `ReviewVerdictIssued`, from `seat-flux-001` |
| clickable | **yes** — `check-cards-clickable.sh` exit 0, `OK: all 1 pending cards are clickable` |
| branch contains `track` | **yes** — so `BranchNotSynced` did not refuse it |
| guard on that branch | **yes** — `merge_would_delete` present in `acceptance_cmds.rs` |

**So the deletion guard's card is open, clickable, and correctly referenced.** That is the state I reported two steps
from on 11-06 04:10, and it has arrived.

**One thing to note for the record**: the card's branch is `packet/nimbus/deletion-guard-accept`, not
`packet/nimbus/project-scoped-reads` which is the branch I verified on 11-05. **Nimbus isolated the guard onto track's
tip on a fresh branch** — `655f40a`, *the deletion guard, isolated onto track's tip so a card can be opened*. So the
thing being accepted is a clean single-purpose branch, and my earlier verification of the guard's behaviour still
applies to its content but not to its exact commit.

## My mutations, and the two that were not mutations

**Direction 1 — make the guard never refuse** (`let undeleted: Vec<String> = Vec::new()`):

```
test result: FAILED. 3 passed; 2 failed
```

**So the refusal logic is covered.** Restored, verified byte-identical, back to 5/5.

**Direction 2 — move the guard after the merge.** This took three attempts, and the first two were not mutations at
all:

| attempt | what happened |
|---|---|
| 1 | I deleted the `let would_delete = ...` statement by slicing from the `let` to the first `;`. **It broke the syntax** — `unexpected closing delimiter` — and the test binary never built, so "5 passed" meant *nothing ran*. |
| 2 | I moved a block by line indices, but placed it after `let merge_commit` in a way that left the guard's own text still **before** the merge. Verified by offset: guard at 5077, merge at 6068. **`5/5` again meant nothing ran**, because the crate had 2 compile errors from the tauri context macro. |
| 3 | Moved the block by statement boundaries and **verified the offsets after applying**: guard at 5281, merge at 5083 — `guard BEFORE merge? False`. Then: `test result: FAILED. 3 passed; 2 failed`, failing on `the_guard_is_wired_into_the_accept_command_before_the_merge`. |

**So the reorder direction is covered, and it took verifying that my mutation had mutated before the result meant
anything.**

## Why this is the important part of the round

**A mutation that does not mutate produces a green suite, and a green suite is indistinguishable from a pass.** I have
written that sentence about other people's instruments four times this week. **This round I produced two of them
myself, in a row, and both times the output said `5 passed`.**

**What caught me was not the test — the test was silent because nothing compiled.** What caught me was checking the
offsets: *did the thing I intended to change actually change?* Aegis recorded the same discipline on 11-05 when his
first mutation hit the wrong branch and he nearly reported "mutation survived", and Flux recorded it when his first lag
probe failed for its own reason and was indistinguishable from a product defect. **This is the third instance in two
days, and the fleet's answer is always the same: prove the mutation mutated, then read the result.**

**And the specific reason my attempts failed is worth naming, because it will recur**: the test asserts order with
`src.find(...)`, which finds the **first** occurrence. A mutation that moves code around can easily leave an earlier
occurrence of the same string in place — a doc comment, a different call site — so a text-order assertion can be
satisfied by a mutation that did not do what it looks like. **The offset check is what distinguishes them, and it is
the only reason I did not publish a false finding.**

## What the operator can now do

**Click Accept on `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy`.** Its branch is synced, its verdict is valid, its merge produces 0
deletions so the guard will not refuse it, and I have verified the guard bites in both directions on that exact branch.
**That is the last step between the deletion guard and the running application.**

## What I did not do

I did not edit the branch, the card, or any script. Both mutations were applied in a throwaway clone and restored
byte-identical, and the clone was removed. I did not merge, push, accept, cancel, or re-open any card, restart any
application, or write to the live database. My measurements were reads plus two mutations with restores in a scratch
clone: the card's fields and verdict reference, `merge-base --is-ancestor` for the branch against `track`,
`check-cards-clickable.sh`, the branch's log and stat, `include_str!` in the test to establish how order is asserted,
and the offset checks that proved each mutation had mutated. The ledger check is exit 0, `seats` is 10, and
`canonical_events` is **94**.
