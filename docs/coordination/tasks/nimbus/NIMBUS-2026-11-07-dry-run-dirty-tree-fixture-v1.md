# The dry-run's dirty-tree warning is now pinned by a fixture that actually dirties the tree

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-dry-run-dirty-tree-fixture-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | ev-f2nyOwON-YqIiGwu_T3Ti, 703b40c |
| supersedes | |
| tags | dry-run, dirty-state, coverage, mutation, measured |

## The finding, and it was a coverage gap rather than a defect

Flux's verdict on the dry-run slice is `pass` with one unresolved criterion:
`dirty_state_report_asserted` FAIL. His mutation P-2 set `DIRTY=""` and the suite
stayed green with exit 0.

He measured why, and it was exactly right: **the fixture builds a clean
repository**, so only the "working tree clean" branch of the report is ever
exercised. The three things the report says about a dirty tree — that it lists the
changes, that it caps the listing at 20 with a "more" marker, and that it warns the
build would contain more than the commit — were never observed by any test.

That warning is the single most valuable line in the report for an acceptance: it
is what tells the operator "the thing you are about to accept is not the commit the
card names".

## What was added

Two fixture cases, both against the real dry run:

**3b — a dirty tree.** Appends to a *tracked* file, then asserts the run lists the
path, warns that the build is not just the commit, and does not claim the tree is
clean.

**3c — more than 20 changes.** Commits 25 tracked files, modifies all 25, and
asserts a truncation marker appears. Without it, a 40-file diff looks like the
complete list.

## Two things I got wrong on the first attempt, both caught by the fixture

**I dirtied an untracked file.** The script uses `grep -v '^??'` to exclude
untracked files — deliberately, because a file that was never `git add`ed was never
going to be in the build either, so it is not the risk this warning is about. My
fixture created a brand-new file, which `git status --porcelain` reports as `??`, so
it was filtered out and the script correctly said "clean". All four content
assertions went red while the script was behaving correctly. The fixture had built
the wrong state.

**I asserted the wrong exit code.** A dirty tree is a *blocker* in the script's own
model (`BLOCKERS=1`, "the working tree is dirty — commit or stash first"), so a dry run over a dirty tree
exits non-zero. My first version asserted exit 0. That one was me guessing the
intent instead of reading it — and reading it showed the script was right and my
assertion was wrong.

## Verified

| | result |
|---|---|
| clean baseline | all cases pass, exit 0 |
| Flux's P-2 (`DIRTY=""`) | **RED**, 7 assertions across 3b and 3c |
| the load-bearing P-1 (dry-run guard neutered) | still RED, exit 1 |
| no-side-effects oracle | unchanged and passing |
| argument-error cases | unchanged and passing |
| determinism | 3/3 |

The fixture restores itself to a clean tree at the end of 3c, so the later cases are
not polluted — and that restoration is itself asserted, because a fixture that leaks
state makes the following cases pass or fail for the wrong reason.

## Not done, and why

No build, no install, no restart, no card — the dispatch says an independent
follow-up is needed, so this is the bounded change only. Not merged to track.
