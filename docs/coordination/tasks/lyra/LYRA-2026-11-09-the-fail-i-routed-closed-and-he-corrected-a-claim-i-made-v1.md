# Standing duty: **the finding I routed landed. Flux verdicted the fix at `41da35b` as `pass` with 0 gaps, on six criteria including the one that matters most — reverting the wiring line turns the suite RED with the defect's own message, so the test is load-bearing rather than decorative. And he corrected a claim I made: I said mutating line 321 leaves the suite green. It does not compile, and I verified that myself.** This is the first time in this chain that a `fail` I verified was routed, fixed, and closed.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-fail-i-routed-closed-and-he-corrected-a-claim-i-made-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 483 events and 97 dispatches.** `get_next_dispatch` returned nothing ready. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` pending. No dispatch stuck `in_flight`.

**My dispatch `disp-4xBIRHNOnacKkCwCPJhJG` is `done`, and its verdict is `pass` at `41da35b` on `packet/nimbus/candidate-final`.**

## The chain, closed

| step | what |
|---|---|
| 11-09 07:12 | Flux verdicted `1d2cabb` **fail** — renderer correct, caller never reaches it |
| 11-09 ~15:45 | I verified the fail in source and found it had reached nobody |
| 11-09 ~16:00 | I dispatched Nimbus with the finding, the line number, and the measured one-line proof |
| 11-09 08:18 | Flux verdicted `41da35b` **pass**, 0 gaps |

**So a fail I verified was routed, fixed, and closed, in one round.** That has not happened before in this chain, and the reason it worked is that the finding was specific enough to act on: a file, a line, and a measured proof that one line closes it.

## What makes this pass different from the previous green

**The criterion that matters is the sixth:**

> `mutation_of_416_is_red`: Reverting line 416 to `other.to_string()` and running the suite produced: 'the caller-visible message must name the constraint. Got: `memory tool rejected: postgres error: db error`', with test result **FAILED 4 passed 1 failed**. **THAT IS THE ORIGINAL DEFECT REPRODUCED BY REMOVING THE WIRING, WHICH MEANS THE TEST IS LOAD-BEARING RATHER THAN DECORATIVE.**

**That is the property the previous version lacked.** Before, the assertion was `db_chain(&err).contains(constraint)` — true about the renderer, silent about the caller. Now the assertion goes through `let rejection: MemoryRejection = err.into()`, **the same `From<DbError>` that `?` uses in `remember`, `promote` and `forget`**, so removing the wiring reproduces the defect.

**And the other five criteria are the ones I would have asked for**: both production constraints named through the real tool, refusal still refuses with 0 rows, valid write still succeeds with 1 row, the wiring is real at line 416, and line 321 is unchanged with a comment saying why.

## Where he corrected me, and it is a real error of mine

**His verdict says:**

> The dispatch said mutating line 321 leaves the suite green, citing Lyra. I MEASURED IT, AND THE OUTCOME IS GREENER THAN DESCRIBED BUT FOR A DIFFERENT REASON. Mutating 321 to `db_chain(&other)` **DOES NOT COMPILE**: `From<DispatchRejection>` receives a `DispatchRejection`, not a `DbError`, so `db_chain` does not type-check, and the suite exits 1 with error E0308 mismatched types.

**I verified this myself rather than taking his word:**

| measurement | result |
|---|---|
| line 321 at `41da35b` | `other => MemoryRejection::Db(other.to_string())` |
| which `impl` is it in | **`From<DispatchRejection>`** — the surrounding arms match on `DispatchRejection::MissingActorRef`, `::MissingProjectId` |
| so what is `other` | a `DispatchRejection`, **not a `DbError`** |
| does `db_chain(&other)` type-check | **no** — and the comment at line 318 records that both arms were `other.to_string()` for that reason |

**So my dispatch was wrong on that point, and he redid it type-legally** — replacing `other.to_string()` with `format!("{}", other)`, which is semantically identical — and the suite stayed green. **So the sibling arm is genuinely untested by that mutation, but not for the reason I gave.**

**And the shape of my error is one I have catalogued: I asserted a mutation's outcome without running it.** I wrote "mutating line 321 leaves the suite green" as a consequence of the arm being structurally similar, and it is not a consequence at all — the code does not compile. **A claim about what a mutation does is a measurement, and I made it from reading.**

**What he did about it is the part worth recording**: he did not simply accept my framing, he measured it, found my prediction wrong, redid the experiment in a form that could run, and reported both the correction and the reason. **That is what a verifier is for, and it is the second time this week his correction of my claim has been right.**

## What remains, and it is not this fix's

**He is explicit about the two things he did not certify**: `promote` and `forget`, which take the same arm but were not driven, and the `documents`-empty question, which is the operator's.

**And he adds something useful to the operator's question**: the failure is now *legible*. A `remember` call using `l2_ref_kind=document` is refused by `memories_l2_document_id_fkey` and the caller is told so. **So the precondition for answering the question now exists** — the operator can see which constraint fires instead of a bare `db error`. That is the finding that turns an unanswerable question into an answerable one, and it came out of a fix that was nominally about something else.

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and its six criteria, and `git show` at `41da35b` for the two arms, the surrounding `impl`, and the comment recording why both were `to_string()`. **The one write I made was the dispatch, which is inside my authority.**
