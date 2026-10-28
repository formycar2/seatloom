# Flux verdict — the M2 relabel ("equivalent mutation" → "not covered today")

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-m2-relabel-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, wording, seat-flux |

- **Verifying dispatch**: `disp-kaaiuujze3aPbYCyRReWX` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-zsKFSIHHgv5v7wLG-94UR` (Mira)
- **Subject commit**: `ad31c66` on `packet/mira/knowledge-panel`
- **Verdict event**: **`ev-1mD-lbquk7qa_0venCWXO`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The relabel is correct, the reasoning behind it is correct, and her §4 finding is the most
valuable thing in the delivery.** Every claim I could check holds.

**One thing that must be stated, because it is a factual error in the artifact this dispatch
certifies, and it is *my* error being carried forward:**

The `ad31c66` comment — the artifact under verification — still contains my false number
`turns RED (2 failures)` in its `NOT EQUIVALENT` section. She reproduced the control mutation
against her own suite, got **20 passed / 0 red**, and wrote that discrepancy down honestly in her
delivery note (§4) rather than repeating my number as fact. But the **code comment at `ad31c66`
still asserts the RED 2**, and a reader of the code does not see her delivery note.

**That false number was subsequently removed by a later dispatch (`disp-x7QjIhw2ZsvokB_yidBj1`,
commit `97a0e0e`), which I have already verified as PASS and which also added the test that makes
my control mutation RED 1.** So the error is fixed on the branch — but not by *this* dispatch.

**This verdict is for `ad31c66`, and at `ad31c66` the false number is present.** I am issuing a
pass on the relabel itself, which is what the dispatch asked for and which is correct, and I am
flagging the residual so nobody reads this verdict as certifying the RED 2. See §4.

---

## 1. The relabel itself — verified

At `ad31c66`, `ui/src/App.tsx` now carries two explicitly separated claims:

- **NOT COVERED** — the query is invoked before the two subscriptions, so its rejection lands
  first and `failure.kind` alone gives the same answer. Mutation M2 therefore passes every test
  in this suite.
- **NOT EQUIVALENT** — the control mutation (M2 plus moving the query after the subscriptions)
  "turns RED (2 failures)" *(this is the false number, see §4)*. Under that ordering the query's
  rejection lands last, and only `prev?.kind` keeps the shape at never-answered.

**Her reasoning for why the words matter is exactly right, and better than how I put it when I
raised it.** "Equivalent" carries a second meaning I did not intend and she did not either: *this
code is redundant, deleting it changes nothing*. A reader who believes that will reasonably delete
`prev?.kind` — and that would silently reintroduce last-writer-wins the moment anyone reorders the
effect. "Not covered today" says something different and points at an action: there is an
unpinned behaviour here, either test it or write down why it is there.

**One word's difference is the difference between "should be deleted" and "should be
completed".** That is the right way to frame it and it is why this dispatch was worth running even
though it changes no behaviour.

## 2. The guard and the tests are untouched — verified in the strongest sense

- `git diff ad31c66~1 ad31c66 -- ui/src/App.tsx`, filtered for changed lines that are not
  comments or blank: **zero**. The guard line
  `prev?.kind === 'never-answered' || failure.kind === 'never-answered'` is character-identical.
- `git diff` for `App.supervisorSync.test.tsx` and `SupervisorSyncLost.tsx`: **empty**. Zero
  change to either.

So her "guard not deleted, tests not changed" claim holds, and it holds the way the dispatch
required: not "the tests still pass" but "not one executable line differs from the parent".

## 3. Her §4 finding — reproduced, and her diagnosis is exactly right

This is the part of the delivery that matters most, and she volunteered it rather than burying it.

**She reproduced my control mutation and it did not reproduce.** I did the same at `ad31c66`:

```
M2 alone (guard dropped, order unchanged)                 -> 20 passed (20)
M2 + query moved after the two subscriptions               -> 20 passed (20)     <-- NOT RED 2
```

**Her number is right and my comment's number is wrong.** Confirmed at the commit under
verification.

**And her diagnosis of *why* is right too.** I tested it directly: with the guard dropped and the
order unchanged, her order-independence test (`a subscription failing BEFORE the query does not
steal the shape`) stays **green**. The reason is the one she identified — the query's own rejection
carries `kind: 'never-answered'`, which upgrades the shape by itself, so `prev?.kind` is never
exercised. That test proves the *upgrade path works*, not that the *guard is what makes it work*.
**Its assertion is weaker than its name.** She wrote that down explicitly instead of letting it
sit inside "my result and Flux's disagree".

**Her conclusion — that what her suite lacks is not one case but a whole class of ordering
scenarios, and that a new dispatch is needed to build the ordering where the query rejects last —
is the right escalation.** That dispatch was subsequently issued and executed
(`disp-x7QjIhw2ZsvokB_yidBj1`), and it produced exactly the test she described. I verified that
separately: with `holdReembedListen` holding the reembedded subscription so it can reject after the
query has already failed, my control mutation is **RED 1**.

So her §4 prediction was not merely honest — it was **correct and actionable**, and the follow-up
dispatch confirmed it.

## 4. The residual: the false RED 2 is still in the `ad31c66` comment

Stated plainly, because this verdict certifies the commit that contains it.

At `ad31c66`, `App.tsx:208-209` reads:

```
// NOT EQUIVALENT: Flux's control mutation — M2 PLUS moving the query after
// the two subscriptions — turns RED (2 failures).
```

**That number is false against this suite.** She measured 20 passed / 0 red at this commit; so did
I; so did Aegis and Nimbus independently.

**What is true:** the number came from my own scratch probe, not from the product suite, and the
*direction* it points at is correct — that ordering is the one that exposes the guard. She wrote
both halves of that into her delivery note §4 with the discrepancy visible, which is why I am not
treating this as a defect in her work: she did not repeat my number as fact, she recorded that it
did not reproduce and why.

**The code comment, however, still asserts it**, and a reader of the code never sees her delivery
note.

**Current branch state:** the later dispatch `disp-x7QjIhw2ZsvokB_yidBj1` (`97a0e0e`, 23:11, ten
hours after `ad31c66`) removed that sentence and replaced it with four measured numbers, one of
which is that same control mutation at **1 red** — because by then the test she called for
existed. I verified `97a0e0e` separately as PASS. **So the false number is gone from the branch;
it is not gone from `ad31c66`, which is the commit this verdict names.**

**Therefore:** this verdict certifies the *relabel and the guard-unchanged claim* at `ad31c66`. It
does **not** certify the RED 2 sentence in that commit's comment, and any acceptance card opened
against it should be read as depending on `97a0e0e` being present as well. Since `97a0e0e` is a
descendant of `ad31c66` on the same branch, a merge of the branch tip resolves this — but the
card's `flux_verdict_ref` should name this verdict for the relabel, and the branch tip must be
what gets merged.

## 5. Gates — every figure exact

At `ad31c66`:
- `App.supervisorSync.test.tsx`: **20 passed** (the new test does not exist yet at this commit —
  it arrives with `97a0e0e`).
- Full UI suite: **300 passed / 24 files / 0 errors**. Baseline is 292, so the delta is 8 — the
  tests from the F2 fix commit `b9e2f06` that this branch already contains. Matches her figure.
- `tsc --noEmit` clean.
- **eslint: 143 problems on the whole `src` tree, before AND after**, diffed against `ad31c66~1`.
  Her "identical to baseline" claim holds. Her file alone carries the same 15 pre-existing
  `no-explicit-any` errors as its parent.
- **prettier**: `App.tsx` clean, and clean at the parent too.
- `vite build` succeeds.

## 6. What I did not do

- Did **not** edit the `ad31c66` comment to remove the false number. It was already removed by the
  later dispatch on the branch, and editing a verified commit's ancestor is not a thing I should
  do from a verification seat.
- Did **not** merge. `ad31c66` is not on `track`.
- Did **not** touch Mira's worktree beyond a detached worktree at `ad31c66`, removed afterwards;
  her `git status` is empty.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/relabel ad31c66
ln -s <mira>/ui/node_modules /private/tmp/relabel/ui/node_modules
cd /private/tmp/relabel

# THE RELABEL, AND THE TWO CLAIMS IT SEPARATES:
grep -n 'NOT COVERED\|NOT EQUIVALENT' ui/src/App.tsx          # lines 199, 208

# THE GUARD AND TESTS ARE UNTOUCHED:
git diff ad31c66~1 ad31c66 -- ui/src/App.tsx | grep '^[+-][^+-]' | grep -v '//'
#   -> nothing. Only comments changed.
git diff --stat ad31c66~1 ad31c66 -- ui/src/App.supervisorSync.test.tsx \
    ui/src/components/SupervisorSyncLost.tsx
#   -> empty. Zero change to either.

cd ui
./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx   # 20 passed
./node_modules/.bin/vitest run                                    # 300 / 24 files
./node_modules/.bin/tsc --noEmit                                  # clean
./node_modules/.bin/eslint src                                    # 143 (same as parent)
./node_modules/.bin/prettier --check src/App.tsx                  # clean
cd .. && ./node_modules/.bin/vite build                           # ok

# HER §4 FINDING, REPRODUCED AT ad31c66 (the false RED 2):
#   M2 alone                                     -> 20 passed (20)
#   M2 + query moved after the subscriptions      -> 20 passed (20)     <-- NOT RED 2
# and her diagnosis of WHY, also reproduced:
#   with the guard dropped and the order unchanged, the order-independence test
#   ('a subscription failing BEFORE the query does not steal the shape') is GREEN,
#   because the query's own rejection carries kind never-answered and upgrades
#   the shape by itself. The assertion is weaker than its name.

# THE RESIDUAL, AND HOW IT WAS LATER RESOLVED (verified separately as PASS):
git log --oneline ad31c66..97a0e0e -- ui/src/App.tsx | cat
#   97a0e0e removes the false sentence, adds four measured numbers,
#   and adds 'a subscription failing AFTER the query does not demote the shape'
#   -> under that test, the same control mutation is RED 1
```

## Verdict event

`ev-1mD-lbquk7qa_0venCWXO`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-zsKFSIHHgv5v7wLG-94UR` in the payload.

## Files I did not change

No source file. Three mutations were applied and each reverted, with the tree confirmed
byte-identical to `ad31c66` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
