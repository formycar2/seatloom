# Standing-duty finding: Mira's cold-start fix is real and better than what I prescribed — and one mutation survives it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-cold-start-fix-verified-with-one-gap-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, verification, mutation, ui, first-screen |

## What I verified, and how

Mira delivered `disp-kYbyTNIImBldOTZevYPU8` — the cold-start fix I ruled on. I re-ran her measurements
rather than accepting them:

| Her claim | My measurement |
|---|---|
| two new contract tests green, 10 passed | **confirmed** — 10 passed on her branch |
| full suite green, 370 tests / 31 files | **confirmed** — 370 passed / 31 files |
| the fix is in `App.tsx` | **confirmed** — `ui/src/App.tsx:484` |
| work not merged to track | **confirmed** — track still `2 failed | 8 passed` |

Her numbers are accurate. What follows is not a correction of her work; it is one gap her own mutation
testing did not cover, found by mutating it myself.

## Her invariant is better than the one I prescribed

I told her to split the effect, using `loadedUIProjectId === null` as the cold-start signal. She did not
do that. She wrote:

```ts
const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;
if (!skipColdStart) { /* apply the saved tab */ }
```

**This is the better design and I am recording why, because my prescription was the weaker one.** Her
gate does not ask *which project* — it asks *whether the restore has landed yet*. The skip is
**consumptive**: it fires once, on first sight, and clears itself. So:

- cold start → skip fires, the tab stays on Dispatch;
- switching to project B → skip already consumed, B's tab restores;
- **switching back to the restored project** → skip already consumed, its tab restores too.

My `loadedUIProjectId === null` version would have worked for the first two and would have needed a
second condition for the third. She found the single invariant that covers all three, and her delivery
note names exactly why ten rounds of my approach failed, in her own words: she kept adding a gate to the
third render frame, the gate did fire, but the second frame had already changed the tab — *"I installed a
correct gate at the wrong door."*
That is a more useful diagnosis than mine.

## The gap: one mutation survives

I ran three mutations against her fix.

**Mutation 1 — remove the cold-start skip** (`skipColdStart = false`, i.e. revert to the old behaviour):

```
Tests  2 failed | 8 passed (10)
```

Caught. Good.

**Mutation 2 — make the skip permanent** (delete the line that consumes it, so the restore never happens
for any project):

```
Tests  10 passed (10)
```

**Survived.** And this is the important one, because it breaks the *second half of the ruling*: with a
permanent skip, switching projects no longer restores that project's tab, and switching back never
restores either. The per-project memory — the feature the operator asked for, the half I explicitly said
was not optional — is silently gone, and **the suite is entirely green.**

## Why the suite misses it

The test named *"switching projects inside a session still restores THAT project's tab"* passes under the
mutation. Reading the ten test names, the covered cases are:

- cold start lands on Dispatch ✓
- switching projects restores that project's tab ✓
- a second launch lands back on the same project **and the same tab** ✓
- restores the recorded project after the backend read ✓
- never persists the launch default mid-restore ✓
- does not restore a project the backend did not return ✓
- no active project when the backend has none ✓

What is missing is the case her consumptive design specifically depends on: **switch away from the
restored project, then switch back to it, and its tab must still be there.** Without that test, a
mutation that destroys the restore for every project after the first looks identical to the correct
implementation.

**A green suite is not evidence of correctness; it is evidence that the suite's cases pass.** Her fix is
right — I am not claiming otherwise, and the mutation that survives is a gap in the *test*, not
necessarily a defect in the *code*. But this is the third time this week a mutation has survived a suite
that reported green, and the pattern is now consistent enough to name: **the surviving mutation is always
in the direction nobody thought to test, never the direction the fix was built for.**

## What is owed, in order

1. **One more test**: switch away from the restored project, switch back, assert that project's tab is
   still restored. That kills mutation 2 and closes the gap her design creates.
2. **Real-machine verification, which she correctly flagged as not done.** All her evidence is jsdom plus
   probes. The ruling's observable behaviour — quit, reopen, see Dispatch; switch projects, see that
   project's page — is a thing the operator does with his hands. Her own delivery note says so plainly.
3. **The verdict, then the card.** `disp-E06O6IAaWefF-BOW-CNJd` still has no verdict naming it, which I
   ruled on earlier today. She is right to wait rather than open a card against a half-finished state or a
   wrong pointer.

## What I did not do

I restored `App.tsx` from my backup after each mutation — verified `grep -c skipColdStart` back to 2 and
the suite back to 10/10, then 370/370. I did not edit her branch. I did not write the missing test,
because it is her surface and her fix, and a test I write for someone else's code is a test nobody owns.
I did not merge anything to track.
