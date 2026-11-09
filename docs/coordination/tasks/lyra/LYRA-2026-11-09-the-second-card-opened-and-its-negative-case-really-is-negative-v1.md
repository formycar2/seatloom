# Standing duty: **a second card opened, `acc-disp-4xBIRHNOnacKkCwCPJhJG`, for the db-error fix I routed, and I ran its verification myself in an isolated worktree before the operator acts on it. Both steps pass exactly as written — and the load-bearing one is genuinely load-bearing: reverting the wiring at line 416 turns the suite RED with the defect's own message.** The card is correctly wired and its branch containment holds.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-second-card-opened-and-its-negative-case-really-is-negative-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready.

**Two cards now pending**: `acc-disp-MlhRWjAlGcaPUj2BqMpOr` (Vela's gate change, description corrected) and **`acc-disp-4xBIRHNOnacKkCwCPJhJG`** (the db-error fix I routed).

## The card's wiring, checked before its steps

**A reference that resolves is not a reference that applies, so I checked the pairing before running anything:**

| check | measurement |
|---|---|
| `flux_verdict_ref` | `ev-nRH3MeASZN_sK3E6Up1Ku` |
| does that event exist | **yes** |
| is it `pass` | **yes** |
| does it name `41da35b` | **yes** |
| does its `branch` match the card's | **yes** — `packet/nimbus/candidate-final` |
| does its `dispatch_id` match the card's | **yes** — `disp-4xBIRHNOnacKkCwCPJhJG` |
| is `41da35b` an ancestor of the branch | **yes** |

**And the branch has moved 49 commits past the verdict, to `26a5d94` — so I checked whether that matters:**

| deliverable | at `41da35b` vs branch tip |
|---|---|
| `crates/seatloom-mcp-emit/src/memory.rs` | **IDENTICAL** |
| `scripts/test-memory-rejection-message.sh` | **IDENTICAL** |

**None of the 49 touches either file.** So the verdict still describes what the branch contains, and the 49 is unrelated work rather than drift. **That is the third time this week I have measured a growing behind-count and found the deliverable blobs unchanged** — which is why I check it every time rather than assuming either answer.

## The two steps, run by me

**The card's `how_to_verify` is two sentences: run the suite and read the two tests; then change line 416's wiring back to `to_string()` and run again, and it must go red with the message that the caller-visible message must name the constraint.**

**I ran both in an isolated detached worktree at `41da35b`:**

| step | card expects | I measured |
|---|---|---|
| `bash scripts/test-memory-rejection-message.sh` | green | ✓ **`test result: ok. 5 passed; 0 failed`**, exit 0 |
| revert line 416 to `to_string()`, re-run | red, defect's own message | ✓ **RED**, exit 1 |

**And the red message is the thing that makes this card honest:**

> `the caller-visible message must name the constraint. Got: memory tool rejected: postgres error: db error`

**That is the original defect, reproduced by removing the wiring.** So the test fails for the right reason and not because the harness broke — which is the difference between a load-bearing assertion and a decorative one. **This is the exact property the previous version lacked**, where the assertion was true about the renderer and silent about the caller.

**I also verified the part that is easiest to get wrong**: that my mutation actually mutated, and that it was actually restored. I asserted the old string occurred exactly once before replacing it, and afterwards confirmed with `git diff --quiet HEAD` that the tree was byte-identical again.

## What is not certified, stated so the operator is not misled

**The verdict is explicit and so am I**: this does not certify `promote` and `forget`, which take the same arm and were not driven, and it does not resolve the `documents`-empty question.

**What it does establish is the precondition for that question being answerable**: a `remember` call with `l2_ref_kind=document` is refused by `memories_l2_document_id_fkey` and the caller is now told so. **So the operator can see which constraint fires, which he could not before this fix.**

## And nothing real was touched

| check | measurement |
|---|---|
| worktree | isolated, detached, at `41da35b`, **removed** |
| mutation | applied, verified to have mutated, **restored from a byte-identical backup** |
| sandbox | md5 `7c0a6e2a…` — **unchanged** |
| live database | 67 memories rows; the suite uses scratch databases and refuses the live one by name |

**The live row count moved from 66 to 67 since Flux's verdict, and that is other seats' legitimate work rather than mine** — my experiments were all in the isolated worktree against scratch databases.

## What I did not do

I did not modify any file in any shared worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one isolated worktree: `git worktree add --detach`, the suite run, a single-line mutation verified to have mutated, and a restore verified with `git diff --quiet`.
