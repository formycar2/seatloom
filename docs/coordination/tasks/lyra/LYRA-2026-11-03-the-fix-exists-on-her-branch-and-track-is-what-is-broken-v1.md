# Standing-duty verification: I read the mechanism that replaced mine on Mira's branch and it is better than what I dispatched against — a ref set before the project changes, consumed on first sight, plus a `!restoredPosition` guard. Her branch passes 20 of 20; track fails 2 of 10. The fix exists, is verified, and the integration line is what is broken.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-fix-exists-on-her-branch-and-track-is-what-is-broken-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, fix-exists |

## Why this round

Last round I established that Mira was right about her branch and I was right about track, and that the
divergence was the finding. **What I had not done is read the mechanism she built** — I had only counted
references to the old variable. **A count of references proves a mechanism changed; it does not prove the new
one is correct.**

**So I read it, and ran her suite.**

## The mechanism on her branch, and it fixes the defect I named

Track's version gates the skip on a state variable set in two branches:

```ts
coldStartProjectId === activeProjectId && !coldStartTabSkipped.current
```

**Hers replaces state with a ref, sets it before the project changes, and consumes it on first sight:**

```ts
const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;   // consumed on first sight
```

**And the assignment carries the reasoning I would have asked for** — `coldStartSkip.current =
persistedProjectId` is set *before* `setActiveProject`, with the comment stating why: the project id is derived
from the store, so it changes as soon as `setActiveProject` runs, and the tab effect re-runs within the same
flush — **the ref is what tells that re-run "this arrival is the cold-start restore."** Setting it after would
be a render too late.

**That is the exact defect I dispatched.** My finding was that the two notions of "the landing project" could
differ; hers removes the second notion entirely by marking *before* the change rather than comparing after it.

**And the `!restoredPosition` term is a second guard mine did not have:** before the restore lands, no arrival
at a project is the operator's, so no saved tab may be applied. **The comment states the measured case — the app
showing defaults (`'p-1'`) and then whatever the store picked while the backend read was in flight — and says
applying a saved tab to either would be answering "where was the operator?" with a project they never opened.**

## Measured, both trees

```
mira  packet/mira/knowledge-panel   npx vitest run src/App.restartRestore.test.tsx
      Tests  20 passed (20)          EXIT=0

track track/infra-foundation         npx vitest run src/App.restartRestore.test.tsx
      Tests  2 failed | 8 passed (10)  EXIT=1
```

**Twenty tests on her branch, ten on track — the suites are different sizes, which is itself the point.** Her
branch has the newer, larger, green suite. Track has the older, smaller, two-red suite that its own author
marked "not finished" on 10-29.

**I also checked her worktree before running anything in it: `git status --porcelain` returned zero lines.** A
test run does not write, but I have nearly destroyed a seat's uncommitted work once and the check is cheap.

## What this means for my dispatch

**My dispatch was correct and is now obsolete.** It named a real defect on track, with the mechanism and the
ruling, and it asked for a mutation proof in both directions. **She could not reproduce the defect because she
had already fixed it, and she ran my two mutations on her code and reported both red — which is what a
correctly-fixed defect looks like when you mutate it.**

**So the finding I dispatched is closed on her branch and open on track, and the thing that closes it on track
is the merge.** That is the same ask that has been open all stretch, and this is now the third concrete cost I
have measured: the operator sees one seat of seven (fixed tonight by the binding), the dispatch channel carried
my work (fixed tonight by the registration), and this defect (fixed on 10-29 or later, still open on the line
the operator runs).

## What I did not do

I did not merge. I did not edit either file. I did not write to Mira's worktree — the test run is read-only
and her worktree was clean before and after. I did not close my finding, because it is about track and track is
unchanged. My measurements were reading the mechanism from her ref (lines 470-530 and 101-130), the two
`vitest` runs, and the pre-run `git status`. `seats` is 10, `canonical_events` is 4, `check-ledger-exists.sh`
is exit 0.
