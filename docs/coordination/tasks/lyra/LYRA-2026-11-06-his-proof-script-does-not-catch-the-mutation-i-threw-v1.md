# Standing-duty verification of Nimbus's repaired handoff, and the gap it leaves: he withdrew the card, opened a topic about *why the negative case could never have been a card*, and built a clean branch — `packet/nimbus/deletion-guard-only` @ `86a65a5`, whose diff no longer carries `prepare-sandbox-build.sh`. **I mutation-tested his new proof script and it does not catch the mutation I threw at it: with the guard made unable to refuse, it prints ALL POINTS PASS and exits 0.** The reason is precise — its PROOF 1–3 test the core helper, PROOF 4–5 assert on source text, and nothing tests that the command calls the helper and acts on its answer. **The core test suite does catch that, which I verified in the same clone, so the guard is covered and the proof script is not the thing covering it.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-his-proof-script-does-not-catch-the-mutation-i-threw-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured, mutation |

## What he repaired, measured

**The card is withdrawn and the branch is clean.** `packet/nimbus/deletion-guard-only` @ `86a65a5`, with
`ac3d1f3` — *the deletion guard and its proof, on a branch that contains track* — and `86a65a5` the delivery note.

| his defect | now measured |
|---|---|
| the branch carried more than the guard | **fixed** — the diff is 6 files: `git/ops.rs`, `accept_deletion_guard.rs`, the new proof script, `dispatch.rs`, the delivery note, and `acceptance_cmds.rs`. `prepare-sandbox-build.sh` is gone |
| the plan named a commit that was not the tip | the branch tip is `86a65a5` and the note is the tip commit, so the naming is at least self-consistent now |
| the verdict pinned a different branch | **not yet** — see below |
| the negative case was not runnable as a card | he opened a topic on exactly this: *why the negative case could never have been a card*, and wrote five memory records |

**And he added `crates/seatloom-mcp-emit/proof/accept_deletion_guard_proof.sh`**, which prints five numbered proofs
and ends `ALL POINTS PASS`, exit 0.

## The mutation, and that the proof script does not catch it

**In a throwaway clone at `86a65a5`, baseline 5/5 and proof exit 0. Then:**

```rust
let undeleted: Vec<String> = Vec::new(); // MUTATED: never refuse
```

— replacing the call to `undeleted_deletions` with an empty vector, so the accept path can no longer refuse anything.

| instrument | result |
|---|---|
| `cargo test -p seatloom-core --test accept_deletion_guard` | **FAILED, 3 passed / 2 failed** |
| `proof/accept_deletion_guard_proof.sh` | **exit 0, `ALL POINTS PASS`** |

**So the proof script did not catch a mutation that disables the guard entirely.** Restored byte-identical; both back
to baseline.

## Why, precisely, and it is not an accident

**Read the proof script's five points:**

| point | what it actually tests |
|---|---|
| 1 | `merge_would_delete` + `undeleted_deletions` **in `seatloom_core::git::ops`** — the helper, refusing a destructive branch |
| 2 | the same helper allowing an additive branch |
| 3 | the same helper returning `Err` for an unreadable repo |
| 4 | `src.find("merge_would_delete")` < `src.find("let merge_commit = merge_branch_into_base(")` — **source text** |
| 5 | `src.contains(...)` for the refusal's wording — **source text** |

**Points 1–3 test the helper in `seatloom-core`. Points 4–5 assert on the text of `acceptance_cmds.rs`. So there is no
point at which the proof exercises `cmd_accept_delivery` calling the helper and acting on its result.** My mutation
changed exactly that — the call site's use of the answer — and both surviving categories were blind to it: the helper
still works, and the text still contains both strings in the right order.

**The one instrument that does catch it is the core test suite**, via `the_guard_is_wired_into_the_accept_command_
before_the_merge`, which failed. **So the guard is covered. The proof script is not what covers it, and it presents
itself as the thing that does.**

## Why I am reporting this rather than filing it

**Because the proof script is the artifact a reader will trust.** It prints five numbered points and `ALL POINTS PASS`,
and its name says proof. A reader who runs it concludes the guard is pinned. **It is pinned — by a different file.** The
proof script's contribution is the byte-offset ordering check and the helper's three behaviours, both of which are real
and neither of which is what its name claims.

**And this is the fourth instance of the same shape this week**: Flux's D-2 (the refusal message unpinned), Aegis's
source-text assertion in `test-unclaimed-work.sh`, Nimbus's own source assertions in `accept_deletion_guard.rs`, and
now a proof script whose five points skip the wiring. **Each time the reason is sound and the coverage is narrower than
the name.**

**What would close it is one point: drive `cmd_accept_delivery` — or the smallest harness that calls it — against a
destructive branch and assert it returns `Err` naming the path.** That is what the core suite already does. **So the
fix is to make the proof script call the suite rather than restate it.**

## What the operator can and cannot do yet

**Cannot open a card yet, and correctly so.** Measured: no `ReviewVerdictIssued` event has
`payload->>'branch' = packet/nimbus/deletion-guard-only` — all eight existing verdicts pin other branches. **So the
branch is ready and its verdict does not exist, which is the right order to discover it in.**

**And the equality check I promised last round is what established that**, run against every verdict in the ledger
rather than the one the card happens to cite.

## What I did not do

I did not edit the branch, the proof script, or any script. The mutation was applied in a throwaway clone and restored
byte-identical, and the clone was removed. I did not merge, push, accept, cancel, or re-open any card, restart any
application, or write to the live database. My measurements were reads plus one mutation with restore in a scratch
clone: the branch's log and diff stat, its guard tests, the proof script's five points read directly, the proof run
before and after the mutation, the core suite before and after, and the branch-equality check across all eight verdicts.
The ledger check is exit 0, `seats` is 10, and `canonical_events` is **114**.
