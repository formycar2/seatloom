# Standing duty: **Flux issued a `fail` on Nimbus's db-error message fix at `1d2cabb`, and I verified the finding independently: the rendering exists, is correct, and has exactly one caller — the test that asserts it. The path a caller actually takes never reaches it, so the caller-visible message is still the literal `db error`.** Flux measured that one line closes it. This is the same shape as three previous defects and it is the clearest instance yet.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-rendering-exists-and-the-caller-never-reaches-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 468 events and 94 dispatches.** `get_next_dispatch` returned nothing ready. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` still pending with its corrected description. No dispatch stuck `in_flight`.

**And a `fail` verdict is now on the integration line**: `ev-…` for `disp-Pca6m4qu1RERbP37cwlTL`, commit `1d2cabb`, branch `packet/nimbus/candidate-final`.

## The finding, verified in source rather than relayed

**Flux's verdict says the fix was written, tested, and left one function call away from the caller. I checked each claim:**

| claim | my measurement |
|---|---|
| `db_chain` is defined | `crates/seatloom-mcp-emit/src/memory.rs:335` |
| it has exactly one caller in the workspace | **line 1047 — inside a test**, immediately followed by `assert!` |
| the caller's path uses a fallback arm | line 386: `other => MemoryRejection::Db(other.to_string())` |
| that Display is near-useless | **the fix's own comment at line 322 says so**: "so a CORRECT [refusal] … reached the caller as the literal `db error`" |
| the sibling arm at 312 | same shape — `other.to_string()`, with a comment saying the chain is already rendered upstream |

**So the commit adds 266 lines that document the defect and render a correct message, and then does not call the renderer from the arm the caller takes.** The comment at line 907 describes exactly the failure mode — "Every foreign-key, CHECK and constraint failure in the [memory path] … string `db error`" — inside the same file that still produces `db error`.

**And Flux drove the real `remember` tool against both production constraints** and got `memory tool rejected: postgres error: db error` for both. **That is the defect the commit set out to fix, still present after the commit.**

## Why the green suite is not evidence

**This is the part worth recording, because the suite is not broken.** Flux checked and said so plainly:

> The two behavioural tests are ignore-d and driven by `scripts/test-memory-rejection-message.sh`, which refuses to run if nothing ran — it fails when PASSED is 0 and FAILED is 0, and its comment records that its first version printed `passed (0 ok)` because grep matched on a run that executed nothing. **THAT IS A REAL GATE AND IT IS CORRECTLY BUILT.** … **THE SUITE'S GATE IS SOUND. WHAT IT GUARDS IS THE WRONG THING**: the assertion is `db_chain(&err).contains(constraint)`, which is true, while the caller-visible message is built from `to_string()`. **THE TEST PROVES THE RENDERER WORKS; NOTHING PROVES [THE CALLER SEES IT].**

**So the gate refuses to pass vacuously, refuses the live database by name, and runs both ignored tests on a real temporary database — and it still certifies a change that does not reach the user.** A gate that cannot fail vacuously and a gate that guards the right thing are two different properties, and this one has the first without the second.

**That is the fourth time this week I have seen a green instrument answer about a different subject than the one under test.** The others were a vitest run that does not type-check, a `--build-only` probe that exits before the gate, and a per-file count read as a per-rule count.

## The pattern, and why this instance is the clearest

**Flux names the lineage himself**: "That is the same shape as the `MUST_HAVE` defect at `6dd68c5` and the two-pass guard at `0a7f1a7`: a capability that exists, is verified, [and is not connected]."

**What makes this one clearest is that the disconnection is one line and he measured it.** He wired `db_chain` into the fallback arm in his isolated worktree only, reverted, and the same probe produced:

> `memory tool rejected: db error | ERROR: insert or update on table memories violates foreign key constraint memories_topic_id_fkey | sqlstate=23503 constraint=Some(memories_topic_id_fkey) table=Some(memories)`

**Both constraints legible, the valid write still succeeding, both refusals still inserting 0 rows.** So the work is complete and correct and one call away from mattering. **A defect whose fix is measured to be one line is the strongest possible evidence that the work was done and left unconnected** — it is not a design gap or a missing capability.

## The three things he asks for, and the one that is not a code question

**His `next_steps`: wire `db_chain` into the fallback arm at line 386 and check the sibling at 312; change the assertion to the caller-visible message; decide what `l2_ref_kind=document` means while `documents` is empty; re-verify after the wiring lands.**

**The third is the operator's and he says so.** Today every `l2_ref_kind=document` call is refused by the FK with a message that cannot say so — and that is a question about what the schema means, not about code.

**And one more thing he confirmed, which closes a thread of mine**: `52cdb8b`, the path-spelling fix I verified myself, is on this branch and is real — `prepare-sandbox-build.sh` now resolves both sides with `readlink -f` before comparing. **So the containment chain is holding while a separate defect sits two files away on the same branch.** A branch is not uniformly good or bad, and a `fail` on one subject does not retract a `pass` on another.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and fleet state, `git show` at `1d2cabb` for the stat and the source lines, and `grep`/`sed` over the file at that commit for `db_chain`'s definition, its single caller, both fallback arms, and the comments describing the defect. **I did not wire the line myself** — the change is Nimbus's and dispatching him is inside my authority, but taking it in his worktree is not.
