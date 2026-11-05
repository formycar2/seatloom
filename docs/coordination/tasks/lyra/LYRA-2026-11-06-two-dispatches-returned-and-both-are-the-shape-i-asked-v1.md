# Standing-duty report that two of my three dispatched items came back while I was idle, and both are the shape I asked for: **Nimbus did the deletion-guard branch sync AND fixed the surviving mutation Flux and I both reported, and Flux re-verified with four mutations all red and zero findings.** Separately he has already started priority 1 — the deploy dry-run — on a branch, and I drove it far enough to confirm it refuses rather than builds. **What this round establishes is that the deletion guard is now one operator action from `track`, and that the two things I reported as gaps were both closed without being re-asked.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-two-dispatches-returned-and-both-are-the-shape-i-asked-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, measured, verification |

## The deletion guard is one action from `track`

**Nimbus merged `track` into `packet/nimbus/project-scoped-reads`** — measured, the branch is now `6f1c566`, and:

| | measured |
|---|---|
| branch contains `track`'s tip | **yes** — so `BranchNotSynced` will no longer refuse a card |
| ahead / behind `track` | **8 ahead / 0 behind** |
| `merge-tree` of the two | **0 deletions** — so his own guard would not refuse the branch carrying it |

**And he fixed the finding I passed on, which was Flux's and not mine.** `cc34a8d`, +20 lines to
`accept_deletion_guard.rs`, and his commit message reproduces the defect before changing anything: *with the
enumeration removed, 5 passed.* He identified the cause as his own test design, not the guard — the behavioural tests
assert on `guard_would_refuse()`, upstream of the message, and the wiring test asserted only that `Err` exists.

**Flux re-verified with four mutations, all red, `findings` empty:**

| mutation | result |
|---|---|
| R-1 — path enumeration replaced with `String::new()` | **RED, 1 of 5**, `the refusal must ENUMERATE the paths it is refusing` |
| R-2 — branch and base replaced with literals | **RED, 1 of 5**, `the refusal must name the branch and the base` |
| R-3 — whole guard block deleted, verified `grep -c` → 0 | **RED, 2 of 5** |
| R-4 — guard moved after the merge | **RED, 2 of 5**, `must run BEFORE the merge — at 5248 vs the merge at 5079` |

**R-1 is the mutation that survived 5/5 in his own first verdict. It is now red.** So the gap I reported and did not
find myself is closed, verified by the seat who found it, with the mutation that proves it.

**What Flux still does not claim, and recorded it himself**: no installed-application measurement, no user-level
acceptance, no merge, no card, and no probe against a real merge on the real repository — all his probes used throwaway
scratch repositories.

## Priority 1 is already in flight, and I drove it read-only

**`packet/nimbus/deploy-preview-dry-run`** — a new branch, 284 lines in `prepare-verification-build.sh` plus a
164-line test, plus the delivery note `NIMBUS-2026-11-07-deploy-preview-dry-run-v1.md`. **Not on `track`** — measured,
`grep -c dry-run scripts/prepare-verification-build.sh` on `track` returns 0.

**I fetched the script and read its dry-run path rather than running it in his worktree**, because running it would
have built or deleted things. What I verified by reading:

| required behaviour | present |
|---|---|
| refuses `--dry-run` without a branch or `--from-dispatch` | yes, line 79 |
| rejects an unknown or repeated `--dry-run` | yes, line 107 — *silently ignoring an argument it does not recognise* is refused |
| dry-run ends before any action, printing what would happen | yes, line 145 — *not one of them executed* |
| reports a dirty worktree, because dirty means the build is not the commit | yes |
| reports branch versus HEAD divergence, because that makes "this branch is installed" false | yes |
| names every path it would write, including the app it would delete | yes |
| names what requires human approval, separately | yes |
| ends with a blocker count that depends on preconditions, not luck | yes |

**And I ran it in his worktree once, with a branch name, and it stopped at the first real precondition** —
`no node_modules here, run pnpm install there first` — which is the correct behaviour in a worktree that cannot build.
**It did not reach any destructive step.**

## What this means for the operator, concretely

**The deletion guard's path is now: open a card citing `ev-flux-verdict-accept-deletion-guard-04`, then Accept.** Both
steps are his or a card-opening seat's, and the branch is synced so the first will not be refused.

**The deploy preview's path is: land the branch, then `--dry-run` is available to him**, and the rebuild itself remains
his approval — which is exactly what the slice was for.

## What I did not do

I did not merge, push, edit any script or product code, accept, cancel, or re-open any card, restart any application,
or write to the live database. I did not run the dry-run in the primary checkout. My measurements were reads: the
verdict payload, `merge-base --is-ancestor` and `rev-list --left-right --count` and `merge-tree` for the branch,
`git show` for `cc34a8d` and the dry-run script, `grep -c` on `track`'s copy, and one run of the dry-run in Nimbus's
worktree with a branch name. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **70**.
