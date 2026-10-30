# Standing-duty verification: the surviving mutation is closed, and the one that remains is a test stub she correctly classified

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-restore-exception-now-tested-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, closure, mutation, verification |

## What I asked for and what arrived

Last round I found a surviving mutation: making the restore at `App.tsx:489` persist produced **12 passed, 0
failed** — the exception the ruling exists for was untested. I asked for one test. It arrived in **three
minutes**: `3bf314b` plus a delivery note `0f420bb`, 43 insertions in the contract file, worktree clean.

The mechanism she chose is better than what I specified. I asked for a spy on `updateProjectUIState`; she
wired `restoreFrameWrites` to record every call **and still call the real function**, then asserted
`expect(restoreFrameWrites).toEqual([])` on the restore frame — and also asserted it empty after the cold
start. Recording without calling would have made the assertion about a dead path.

## I re-ran the mutation that survived, and it is now caught

```
× switching projects inside a session still restores THAT project's tab
  Tests  1 failed | 11 passed (12)
```

**Red.** The `:489` restore-persists mutation — which survived a 12-test suite last round — now fails.
Restored from backup, worktree clean, and the full suite is **372/372 across 31 files.**

## She ran six mutations, not the five I asked for, and reported the survivor herself

| # | Mutation | Result |
|---|---|---|
| 1 | my own `:489` mutation | **RED** — the new test |
| 2 | restore deletes `setActiveTab` | **RED** ×2 |
| 3 | unconditional write after `setLoadedUIProjectId` | **RED** ×4 |
| 4 | `chooseTab` does not call `setActiveTab` | **RED** |
| 5 | `skipColdStart` always false | **RED** ×4 |
| 6 | the spy records but never calls the real function | **12 passed — SURVIVED** |

**I verified mutation 6 myself rather than taking her word.** It survives, and **her classification of it is
correct**: the test asserts *whether the write path is called*, not *whether the write path works*. Removing
the real call from the mock does not affect the first claim, and the second claim is covered by the other
372 tests — including the test that caught mutation 1 last round. **A surviving mutation in the direction of
"the instrument stopped measuring" is not a product defect; it is an instrument limit, and it is the one
she named.**

That distinction is the whole discipline. A surviving mutation is only a finding when it removes product
behaviour nothing checks. This one removes test behaviour that a different test checks.

## What this closes

**The `:489` exception — the entire point of the persist-from-input ruling — is now tested, in both
directions.** The suite can tell the difference between implementing the exception and not implementing it.
That is the fourth surviving mutation this week to be closed, and the third one that turned out to be the
assertion nobody thought to write.

## What is still owed, unchanged, and I want the list short

1. **No acceptance card.** `disp-E06O6IAaWefF-BOW-CNJd` still has no verdict naming it. The guard is correct
   to refuse a verdict about a different dispatch; what is owed is a verdict whose `payload.dispatch_id`
   names this one.
2. **Real-machine verification.** All evidence is jsdom plus probes. "The operator's saved tab survives a
   real restart" is a real-machine claim and has never been tested on a real machine.
3. **Both remotes unreachable at her finish time.** They recovered for me earlier today and I pushed her
   previous commit to both; I will push these two as well, because a complete commit nobody can fetch is
   not delivered either.

## The merge question is now the only thing between this and a green line

The code is done and verified. **Track is still red at 2 failed / 370 passed** — that is not a code
question, it is a merge question, and it is the same one I have been asking for: `packet/mira/knowledge-panel`
into track. Nothing about the work is outstanding except the decision.

## What I did not do

I did not edit her tree beyond the two mutations, both restored from backup, and I verified her worktree is
clean afterwards. I did not open a card, and I did not merge anything.
