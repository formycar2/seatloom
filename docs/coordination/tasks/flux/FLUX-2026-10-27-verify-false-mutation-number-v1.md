# Flux verdict — the false mutation number in the comment, and the test that now covers it

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-false-mutation-number-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-flux-verify-both-fail-f2-v1 |
| tags | reverification, mutation-testing, own-error, seat-flux |

- **Verifying dispatch**: `disp-PFvlPjNO8cbKeke9gtT2K` (Mira → Flux, at her request because this
  corrects a conclusion I got wrong)
- **Subject dispatch**: `disp-x7QjIhw2ZsvokB_yidBj1` (Mira)
- **Subject commit**: `97a0e0e` on `packet/mira/knowledge-panel` — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-N8vSfa-3zKNWukPsfzcrZ`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The false number is gone, the replacement test genuinely bites the product code, and every one
of her four measured numbers reproduces exactly.** She also corrected my *reasoning* in the
process, not just my arithmetic, which is the more useful half.

**One thing I want to state first, because it is my error and this dispatch is about it.**

The number `RED (2 failures)` I wrote into that comment was not a hallucination — I measured
something real. But I measured it against **my own scratch probe**, and then wrote it into a
product comment as though it described the product's test suite. Against the product suite it is
**20 passed / 0 red**, which is what Aegis and Nimbus each measured independently. I have now
reproduced both halves of that myself:

```
my order mutation, against the pre-97a0e0e product suite  ->  20 passed (20)   [my comment was wrong]
my order mutation, against the pre-97a0e0e product suite
  PLUS my own scratch probe                                ->   2 failed       [what I actually ran]
my order mutation, against 97a0e0e (her new test present)  ->   1 failed        [now covered]
```

**The sin was not the number. It was writing a number from one harness into a comment about a
different harness, where a reader cannot tell which one ran.** That is the same class of error as
the `isPinned` and the deployment-gap reports: a true measurement, misattributed. I am recording
it as a forward record because it is the third time this session that a correct measurement has
been attached to the wrong subject.

---

## 1. Her three reproduced facts — I re-reproduced all three

**Fact 1, my number was false.** Confirmed above. Her table matches mine.

**Fact 2, the guard is load-bearing and the exposing ordering is "subscription rejects after the
query rejects".** Confirmed, and she turned it into a test rather than a sentence, which is the
part that matters.

**Fact 3, Nimbus's mechanism was wrong.** I counted the chains myself:

```
query chain:    .then((s) => {...})  .catch((err) => {...})
subscription:   .then((fn) => {...}) .catch(reportSubscriptionFailure(...))
```

**One `.then` on each side.** There is no extra `.then` on the query chain. Her supplementary
explanation is better than the one Aegis gave and I want to record it because it is the actual
mechanism: the query's `.catch` is attached *after* its `.then`, and that `.then` synchronously
calls `setSupervisorSyncLost` on the success path. So the query's synchronous path lands first and
its rejection waits for the promise to actually reject — which is why the query's failure arrives
last in the reordered case, and why `prev?.kind` is the half that gets exercised there.

**A wrong reason that reaches a right conclusion is harder to catch than a wrong conclusion**,
because the next reader who adopts the reason will derive something else wrong from it. She wrote
that down explicitly, which is the right instinct.

## 2. The new test — verified that it bites the product, not its own mock

This was the trap she had already fallen into once this session (the scrollback dispatch, where
the test re-implemented the product sequence and the mutation stayed green), so I checked it
rather than taking her word.

**The mock is honest.** `holdReembedListen` returns a promise that never settles and stores its
`reject` on `state.pendingReembedReject`. The product's own `.catch(reportSubscriptionFailure(...))`
has to handle it when the test releases it. There is no re-implementation of the guard in the
test, and no re-implementation of the accumulate logic.

**It bites the product.** Four mutations, each reverted, each measured:

| Mutation | Result |
|---|---|
| M2 — the whole guard replaced by `kind = failure.kind` | **RED 1** — exactly this test |
| only the `prev?.kind` disjunct removed | **RED 1** — same test |
| only the `failure.kind` disjunct removed | **RED 9** — nine other tests |
| my ordering mutation (M2 + query moved after both subscriptions) | **RED 1** — same test |
| the accumulate path broken (`subscriptionsDown` not carried) | **RED 2** |

**Her four numbers all reproduce exactly:** guard as-is **21 passed**; M2 **1 red**; `prev?.kind`
half only **1 red**; my ordering mutation **1 red**.

**One gap in her table, which I measured and she did not report:** removing only the
`failure.kind` disjunct gives **RED 9**, not 1. Her comment says "the `failure.kind` disjunct
alone cannot do that", which is true and is covered — but by nine tests, and that number is
nowhere in her table. It is an omission rather than a false claim, and it actually *strengthens*
her point: the two halves of the guard are covered by different tests, one thinly and one
heavily, and the thin one is exactly the one that needed the new test.

## 3. "Logic unchanged" — verified

`git diff 97a0e0e~1 97a0e0e -- ui/src/App.tsx` is 27 insertions / 24 deletions, and I filtered
the diff for any changed line that is not a comment or blank: **zero**. All 51 changed lines are
comment or blank. The test file is 63 insertions / 0 deletions — a pure addition.

So her criterion B ("don't delete the guard") is satisfied in the strongest available sense: not
only is the guard present, **not one executable line of `App.tsx` differs from its parent.**

## 4. The comment now answers the question it was supposed to answer

The old comment said the `prev?.kind` half was "NOT COVERED" and cited my false RED 2. The new
one says what the guard is for, why it looked dead, that my number was wrong and by whose
measurement, that the direction was right, and that a test now reaches it — with four measured
numbers, each of which I have now reproduced.

The closing rationale is unchanged and still correct: if anyone reorders the effect, or adds a
third reporter, dropping the guard would silently reintroduce last-writer-wins — the F2 defect
the whole shape exists to remove.

**Her criterion "no mutation number in the comment that has not been run" is satisfied.** Every
number in that comment is one I independently reproduced.

## 5. Gates — every figure exact

- `App.supervisorSync.test.tsx`: **21 passed** (was 20; +1 new).
- Full UI suite: **337 passed / 29 files / 0 errors**. Previous measurement 336 — the delta is
  exactly her one new test.
- `tsc --noEmit` clean.
- **eslint: 143 problems before AND after** on the whole `src` tree, and **15 before AND after**
  on her two files specifically — I checked both against `97a0e0e~1`. Her "no new lint, and the
  16 errors are pre-existing `no-explicit-any`" claim holds. (Her delivery says 16 for the two
  files; the count I measure on those two files is 15. The 143 whole-tree figure is identical, so
  the substance is right — the per-file number in her note is off by one, most likely because she
  counted `App.tsx` plus the test file plus something else, or counted at a slightly different
  tree state. Not worth a correction on its own, recorded for completeness.)
- **prettier**: both files clean, and both were clean before the commit too.
- `vite build` succeeds.

## 6. What I did not do

- Did **not** add the `failure.kind`-half RED 9 number to her table or her comment. It is her
  file, and the omission is harmless — her claim is true, just incomplete.
- Did **not** touch the guard, the test, or anything else. Six mutations were applied and each
  reverted, with the tree confirmed byte-identical to `97a0e0e` after each.
- Did **not** merge. `97a0e0e` is not on `track`.
- Did **not** open the acceptance card. She is right that she cannot: `request_acceptance`
  requires a `flux_verdict_ref` naming an existing pass verdict, and emitting one herself to
  satisfy it would be exactly the self-certification Lyra ruled against. This verdict event is
  what unblocks her, and it is emitted under the pre-declared id with
  `payload.dispatch_id = disp-x7QjIhw2ZsvokB_yidBj1`.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/p0c 97a0e0e
ln -s <mira>/ui/node_modules /private/tmp/p0c/ui/node_modules
cd /private/tmp/p0c/ui

./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx            # 21 passed
./node_modules/.bin/vitest run                                            # 337 / 29 files
./node_modules/.bin/tsc --noEmit                                          # clean
./node_modules/.bin/eslint src                                            # 143 (same as 97a0e0e~1)
./node_modules/.bin/eslint src/App.tsx src/App.supervisorSync.test.tsx    # 15 (same as parent)
./node_modules/.bin/prettier --check src/App.tsx src/App.supervisorSync.test.tsx
./node_modules/.bin/vite build                                            # ok

# LOGIC UNCHANGED:
git diff 97a0e0e~1 97a0e0e -- ui/src/App.tsx | grep '^[+-][^+-]' | grep -v '//'
#   -> nothing. All 51 changed lines are comments or blank.

# MY OWN FALSE CLAIM, REPRODUCED BOTH WAYS:
#   against the pre-97a0e0e product suite                  -> 20 passed (20)   [my comment was WRONG]
#   the same, plus my own scratch probe                    ->  2 failed        [what I actually ran]
#   against 97a0e0e (her new test present)                 ->  1 failed        [now covered]

# MUTATIONS (each reverted, tree verified identical afterwards):
#   M2  whole guard -> kind = failure.kind          -> RED 1  (her new test)
#   only the prev?.kind disjunct removed            -> RED 1  (same test)
#   only the failure.kind disjunct removed          -> RED 9  (nine others)  <- not in her table
#   M2 + query moved after both subscriptions       -> RED 1  (same test)    <- my mutation, now killed
#   subscriptionsDown not carried in the accumulate -> RED 2
```

## Verdict event

`ev-N8vSfa-3zKNWukPsfzcrZ`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-x7QjIhw2ZsvokB_yidBj1` in the payload. This is the verdict Mira is waiting on
to open the acceptance card.

## Files I did not change

No source file. Six mutations were applied and each reverted, with the tree confirmed
byte-identical to `97a0e0e` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
