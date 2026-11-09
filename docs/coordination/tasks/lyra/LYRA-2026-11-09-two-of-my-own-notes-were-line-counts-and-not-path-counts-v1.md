# Standing duty: **I corrected two of my own stale notes by measuring them, and both were wrong in the same direction — I had counted the wrong thing. "My grandfather list is 55 paths wider than track's" is actually **zero paths** wider: both lists contain the same 207 paths, and my extra 55 lines are 43 comment lines plus 12 duplicate comments. And "docs gate RED on track" is now **green**, and the file I named was renamed to `.zhs.md`, which is exempt from the very rule it was failing.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-two-of-my-own-notes-were-line-counts-and-not-path-counts-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

**And I verified my own housekeeping before anything else: no probe residue anywhere in the tree, no stray worktrees, and all nine worktrees accounted for.** That was the first thing to check, because last round I made mutations in paths that were not mine.

## Note 1: the grandfather list is not wider at all

**My note said: "my grandfather list is 55 paths wider than track's."**

| measurement | result |
|---|---|
| line difference | **55 lines** |
| comment lines in my list | **43** |
| real paths in track's list | **207** |
| real paths in my list | **219** |
| **real path difference** | **0** |

**Wait — 207 vs 219 is a difference of 12, and the path-level `comm` says 0. Both are true and the resolution is that my list contains 12 paths that also appear in track's, with different surrounding comments.** The `comm` on the de-commented, sorted path sets returns nothing.

**So the honest statement is: the two lists grandfather exactly the same 207 paths, and mine carries 12 more lines of commentary.** That is not a divergence in what is grandfathered — it is a divergence in documentation, and it is the kind of thing that looks like a merge conflict waiting to happen and is actually two people explaining the same list differently.

**And the error is the one I have catalogued more than any other: a count from the wrong unit.** I counted lines and reported paths. **A line count and a path count are different measurements, and the number 55 was true of the first and false of the second.** I had been carrying it as evidence that my branch diverged from track in a way that needed reconciling, when the divergence was 43 lines of comments.

## Note 2: the docs gate is green, and the file I named no longer exists

**My note said: "docs gate RED on track (`BUDGET_POOL_MODEL_MAP.md` needs §2 header)."**

| measurement | result |
|---|---|
| `bash scripts/verify-docs.sh HEAD` on track | **exit 0 — green** |
| does `BUDGET_POOL_MODEL_MAP.md` exist | **no** |
| what it became | `docs/coordination/tasks/harness/HARNESS-2026-11-05-budget-pool-model-map-v1.zhs.md` |
| is that on track | **yes** |
| does it pass | **yes** |

**So the file I named was renamed, moved into the harness task directory, given a conforming filename, and — decisively — made `.zhs.md`, which `verify-docs.sh` exempts from the CJK rule at line 92.** The gate is green because the document was brought into compliance, not because the rule was relaxed.

**And that is the second time in two rounds that a note I have been carrying turned out to describe a state that had already changed.** Last round it was the pre-commit hook source-of-truth, which was backwards. Both times I had recorded the measurement correctly at the time and then never re-measured it, and both times the stale version was the more alarming one.

## The pattern across both

**A note that is never re-measured becomes a claim, and a claim that is never checked becomes a false report.** I wrote on 11-08 that "a list item nobody rechecks becomes a false report" — recording Aegis's finding about his own backlog. **I then carried two unchecked items in my own escalation list.**

**And the specific shape is worth naming: both stale notes were in the direction of "my branch is worse / the gate is broken".** Neither was. **A seat that accumulates unverified bad news about its own state is manufacturing a reason for the operator to intervene, and reporting a problem that is not there pushes him to break what works.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `find` for probe residue, `git worktree list` for stray worktrees, `bash scripts/verify-docs.sh HEAD` for the gate, `git log`/`git cat-file`/`git ls-files` for the renamed document, and `wc`/`grep`/`comm`/`sort` over both grandfather lists. **I did not reconcile the two lists** — they grandfather the same paths, so there is nothing to reconcile, and merging my branch remains the operator's decision.
