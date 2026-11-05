# The deletion guard's acceptance: the guard isolated, built, and pointed at a throwaway repo

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-deletion-guard-acceptance-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | ev-hW75U60DweOY5-qM5pc5r, 655f40a, 1cee50c |
| supersedes | |
| tags | deletion-guard, acceptance, sandbox, measured, gate |

## What was asked and what was done

Flux re-verified the integrated deletion guard at `6f1c566` — clean pass, findings
empty, verdict `ev-hW75U60DweOY5-qM5pc5r`, naming the original implementation
dispatch `disp-AbQ4m1TKQZGLMeZ4bhwjy`. The dispatch was to prepare its acceptance:
inspect the branch and worktree, the current track containment, the full product
diff and scope, open the card for the ORIGINAL dispatch citing the NEW verdict,
and supply a matching isolated sandbox/build — without folding the unrelated
deploy-preview and work-list commits in.

Card `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy` is open, `pending`, branch
`packet/nimbus/deletion-guard-accept`, base `track/infra-foundation`, verdict
`ev-hW75U60DweOY5-qM5pc5r`. The sandbox is built from that branch and running,
with the card visible and the guard compiled in.

## Why the branch is not the one Flux verified

`packet/nimbus/project-scoped-reads` is 8 commits beyond track, and those 8 carry
three other dispatches: `close_topic`'s atomicity tests, the paste classifier, and
the 5-shipped-changes delivery. A card naming that tip would land all of it under
this one acceptance, including 765 lines of topic-close tests that have no card
and no verdict of their own.

So the guard's own change set was extracted — `git diff 6e04dae~1 cc34a8d --` over
its four files — and applied onto track's tip as
`packet/nimbus/deletion-guard-accept`. Nothing else is on it. Measured: 1 ahead,
0 behind, and the guard's own containment (c) check on it returns no deletions,
so the guard does not refuse the branch carrying it.

**The scope is measured, not assumed.** The `mcp-emit` diff is the exemption list
moving into `seatloom_core::git::ops` so both callers share one list, plus one
call site changing to call it. There is no behaviour change outside the guard.

## A claim of mine that was wrong, and the measurement that corrected it

I first recorded that merging the psr branch into track would REVERT two of
Aegis's files (`scripts/check-unclaimed-work.sh`,
`scripts/test-unclaimed-work.sh`) because the branch predates his commit `93ef3d3`.

That was a two-dot-diff artefact. The real question is the three-way merge, and
measured with `merge-tree --write-tree` it produces **0 conflicts and both files
identical to track's**. A two-dot diff shows what differs between two tips; it does
not show what a merge produces. The isolated branch was still the right call, but
for the scoping reason above, not the revert reason I first gave.

## The sandbox, and the thing it cannot do by itself

`prepare-sandbox-build.sh packet/nimbus/deletion-guard-accept --apply` builds and
installs `/Applications/SeatLoom-Sandbox.app`. The main app is untouched (pid
90805, still the 10-28 binary). The sandbox binary contains the guard's refusal
string; the running app's does not, so **no binary the operator can click today
has the guard** — that is the whole point of landing it.

But a click in the sandbox is not a test of this card on its own, for two reasons
measured rather than assumed:

1. **The sandbox runs on `seatloom_sandbox`, a snapshot.** A card opened in the
   live database does not appear there until the snapshot is refreshed. The first
   build I did had the card invisible for exactly this reason.
2. **The launcher never sets `SEATLOOM_REPO_ROOT`** (0 occurrences). Left unset,
   `main.rs`'s `HOME` heuristic resolves to `~/Documents/GitHub/seatloom` — the
   **main checkout**. So a sandbox click would merge into the real `track` branch,
   against the real working tree.

Both are fixable without touching anything real. The snapshot was refreshed after
the card was opened, and the sandbox was launched with
`SEATLOOM_REPO_ROOT=/tmp/sbclick`, a throwaway clone that has both branches, is
clean, and is on the base. A click there merges into the clone. **If the operator
wants to click the real thing, he must not pass that env var — and that is his
call to make, not mine, because it is the act that merges into track.**

## What the guard does, driven against real repositories

Not re-run from Flux's numbers, because this is a different base. Through
`merge_would_delete` + `undeleted_deletions` on live repositories:

| case | result |
|---|---|
| branch that deletes a base-tracked file | **refused**, paths listed: `["src/keep.txt"]`, `[".cargo/config.toml"]` |
| branch that only adds a file | allowed, `[]` |
| this card's own branch, on the real repo | allowed, `[]` |
| the guard's condition inverted (compiles) | suite RED, 1 of 5 — after asserting build errors 0 first |
| the guard's block deleted (does NOT compile) | suite RED — but the red was from the **stale binary** |

The last row is a trap worth naming. My first deletion mutation did not compile,
`cargo test` printed a red from the previously built binary, and my `git checkout
--` restore reverted the patch because it was never staged. I caught it because
the build-error count was non-zero, redid the mutation as a compiling inversion,
and asserted `build errors: 0` before believing the red.

## Gates

`cargo build --workspace` 0 errors · `cargo test --workspace` 372 passed, 0 failed
· `accept_deletion_guard` 5/5 · clippy 0 on the guard's three crates · sandbox
built and running, guard string present, main app untouched.

## Also fixed, in its own commit

`prepare-sandbox-build.sh` had the same `WROOT` defect I fixed additively in
`prepare-verification-build.sh`: run from a seat's own worktree — which its own
documentation tells the operator to do — it resolved to
`seatloom-worktrees/seatloom-worktrees`, a directory that does not exist. The
failure was worse than a wrong path: the "find the worktree with this branch
checked out" loop iterated a directory that wasn't there, so it never ran, and the
script reported `'packet/nimbus/deletion-guard-accept' is not a branch checked out
in any worktree` — false, and pointing at the wrong problem while a one-line path
error sat underneath. Commit `1cee50c`.

## Not done, and why

- **Not merged, not deployed, no live-DB write, no app install/restart.** The
  operator's Accept is the merge.
- **No user acceptance authored by Lyra.** Measured across her ten deletion-guard
  documents: zero `scenario`/`acceptance_plan`/`plan_version` occurrences. The
  card's plan is therefore mine, and it says so in its preconditions.
- **The topic-close tests on the psr branch have no card.** They are not part of
  this one, by design.
