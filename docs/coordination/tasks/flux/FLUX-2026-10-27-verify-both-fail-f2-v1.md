# Flux verdict — the F2 fix: an accumulated record instead of a single slot

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-both-fail-f2-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-flux-verify-both-fail-f2-v1 |
| tags | reverification, mutation-testing, ui-honesty, seat-flux |

- **Verifying dispatch**: `disp-viPlKa_e9NoeRHeGe_LCC` (Aegis → Flux)
- **Subject dispatch**: the F2 fix to `disp-6xJ-8eWA7-LtBDeW9G5Yj`
- **Subject commit**: `b9e2f06` on `packet/mira/knowledge-panel`; `App.tsx` identical on
  `track/infra-foundation`
- **Verdict event**: **`ev-hfLMM6Gyn3z7V6qIatMlJ`** — the pre-declared id
- **Prior finding**: F2, measured by me in `ev-SNtpi-dbYOEjmkdz4fK7e`

## Verdict: PASS

**F2 is fixed.** All four combinations are now correct, including the fourth — red, retry
present, both error texts present, and "both are down" stated. `subscriptionsDown` is not
misused. The retry-hiding-a-live-problem mirror case is covered. Every mutation I ran kills.

**One correction to her claims, and one disagreement:**

- **M1 reproduces at RED 5** — Aegis's three failed attempts were Aegis's own mutation
  errors, not a gap in her description.
- **M2 is not an equivalent mutation.** It is *unreachable* given today's call order, which
  is a different statement — and I found the case where it is reachable.

---

## 1. Question 1 — the four combinations are now correct

Driven through the real App with my own probe (scratch, deleted):

| # | Query | Subs | Rendered |
|---|---|---|---|
| 1 | OK | OK | **no strip** |
| 2 | OK | fail | **yellow `stale`**, no retry, no both-down line |
| 3 | fail | OK | **red `never-answered`**, retry present, no both-down line |
| 4 | fail | fail | **red `never-answered`**, **retry present**, **both error texts present**, **"the live push is not connected either"** present |

Case 4 is exactly what F2 was about, and every element of it is now right: the worse shape
wins, the retry is offered, neither error text is dropped, and the operator is told both
links are down rather than left to infer it. Before the fix this same case rendered yellow
with the query's error text gone and no retry.

**Her observation about the timing difference between our two measurements is worth
recording, because it strengthens the finding rather than weakening it.** She saw the
subscription text survive and the query text vanish; I saw the reverse. Both are the same
defect — a single slot with last-writer-wins — and *which* text is lost depends on
microtask ordering between the query's rejection and the two subscription rejections, which
is not something either of us controls. Two independent observers catching two different
halves of the same loss is better evidence than either alone.

## 2. Question 2 — `subscriptionsDown` is not misused

| Case | `subscriptionsDown` | Rendered effect |
|---|---|---|
| subscription fails alone | effectively true, but the shape is `stale` | `bothDown = neverAnswered && subscriptionsDown` is **false** → no both-down line ✅ |
| query fails alone | false | no both-down line ✅ |
| both fail | true, shape `never-answered` | both-down line shown ✅ |

The guard `neverAnswered && subscriptionsDown` is what makes this correct, and it is the
right conjunction: `subscriptionsDown` is only meaningful *alongside* `never-answered`, as
her own comment says. I verified the stale shape never claims the both-down condition, and
that the never-answered shape does not claim it when the subscriptions are healthy. **No
misuse.**

## 3. Question 3 — her "the button should be there" judgement is right, and my P3 reasoning was the weaker half

She disagrees with my "the retry would probably fail too" reasoning, and **she is right**.
Her three reasons hold, and the third is the strongest: *hiding a usable action and hiding a
fact are the same error in opposite directions.* My P3 note conflated "this action probably
won't work" with "this action shouldn't be offered", and those are different claims. The
first is a prediction about outcome; the second is a design decision about agency, and it
is not mine to make on the operator's behalf.

Her second reason is also better than I credited: **a failed retry does not make things
worse** — the strip stays, the error text stays, nothing is lost by trying. So the downside
of offering it is near zero and the upside is that the one path to recovery is not hidden.
I verified this: a retry that fails again keeps the strip and keeps the reasons.

One thing I measured that neither of us raised: after three consecutive failures the
reasons list shows **three** entries. That is defensible — an honest history rather than a
silently replaced one — but it does mean a strip left open through many retries grows. Not
a defect; noted so it is a known behaviour rather than a surprise.

## 4. Question 4 — M2 is NOT an equivalent mutation. It is unreachable, which is different

Her claim: `kind = failure.kind` behaves identically today, because the query is invoked
before the two subscriptions so its rejection always lands first and the `prev?.kind`
guard never fires.

**Her claim about her own suite is accurate** — I applied M2 and her 20 tests stayed green.
**But her conclusion is wrong**, for two reasons I measured separately.

**Reason 1: the guard is reachable and load-bearing.** Her order-independence test holds the
query's promise back so a subscription rejects first, then lets the query reject. I traced
that path with my own probe and the shape **does** upgrade `stale → never-answered`. So the
upgrade path works. But when I apply M2 *and* run my probe's case 4 and the
retry-demotion case, **2 tests go red**. So the guard is not dead code in general — it is
dead only under the specific ordering her suite produces.

**Reason 2 — the precise distinction.** Her M2 mutation is `kind = failure.kind`, dropping
`prev?.kind === 'never-answered' ||`. Under today's order the query's rejection lands first,
so when the *subscriptions* report afterwards, `prev` is already `never-answered` and the
guard would be needed to stop them demoting it. Wait — that is exactly the case M2 should
break. Let me state what I actually measured rather than reasoning it out:

```
M2 applied, her suite alone                    -> 20 passed (20)      [her claim: green]
M2 applied, my probe (case 4 + demotion)       -> 2 failed
M2 applied + query moved AFTER the subscriptions-> 2 failed
```

So the honest statement is: **her suite does not distinguish M2 from the real code; my probe
does.** That is a coverage gap in her tests, not an equivalent mutation. Her own §2 defence
("I checked both: the order test proves the upgrade path works, so it is not a gap") is the
step that does not hold — the order test proves the upgrade path works, but it does **not**
prove that the *guard* is what makes it work, because under M2 that same test still passes
for a different reason (the query's own rejection carries `kind: 'never-answered'`, which
alone upgrades the shape).

**Concretely: M2 is an untested-but-load-bearing branch, not an equivalent mutation.** Her
decision to *keep* the guard and document it as defensive is right regardless — deleting it
would silently reintroduce last-writer-wins if anyone reorders the effect, which is exactly
what her comment says. I would only correct the label: write "not covered by tests today"
rather than "equivalent", because those two words send the next reader to different places.

## 5. Question 5 — M1 reproduces at RED 5. Aegis's three attempts were Aegis's own errors

Her M1 is "revert the whole reporter to a single slot". I reproduced it by collapsing all
three parts of the accumulated update — `reasons` to just the incoming one, `kind` to
last-writer-wins, and `subscriptionsDown` to just the incoming failure:

```
  × case 4 — query fails AND both subs fail: RED, retry present, BOTH reasons, both-down stated
  × succeeding demotes to stale and keeps the subscription reasons, dropping the query reason
  × query fails AND subscriptions fail -> the worse one wins, and nothing is lost
  × a successful query with subscriptions still down demotes to stale, keeping their reasons
  × a subscription failing BEFORE the query does not steal the shape
  Tests  5 failed | 24 passed (29)
```

**RED 5 — 3 in her suite, 2 in mine.** So her M1 description is complete and reproducible.
Aegis's three attempts failed for reasons Aegis already diagnosed (type collapse, the file
not actually changing, missing the render side), which is the ordinary difficulty of
mutating a shape that has just been widened across two files. Worth saying plainly: **her
M1 claim was correct and Aegis could not reproduce it; the fault was in the mutations, not
the description.**

The other three mutations:

| # | Mutation | Result |
|---|---|---|
| M3 | success wipes the whole strip (the mirror image of F2) | **RED 2** — one in her suite, one in mine |
| M4 | the both-down line removed | **RED 2** |
| M1 | whole reporter back to a single slot | **RED 5** |
| M2 | `kind = failure.kind` | **green in her suite, RED 2 in mine** — see §4 |

M3 deserves a note: it is the exact mirror of the finding I raised, and she anticipated it
in the code comment ("a failure hidden by an unrelated success") and covered it with a
test. That is the right instinct — a fix for one direction of a class of bug should be
checked in the other direction.

## 6. Gates

- Her file: **20 passed** (16 from the previous dispatch + 4 new).
- Full suite: **300 passed (24 files), 0 errors**. Previous dispatch measured 292, so the
  delta is exactly her 4 new tests.
- `tsc --noEmit` clean.
- **eslint**: 15 errors on `App.tsx` **before and after**, diffed against `b9e2f06~1` —
  the same 15 pre-existing `no-explicit-any`. **0 on both her files.** Her "zero new" claim
  is accurate.
- **prettier**: clean on all three.
- Ran against the main checkout's `node_modules`, not a symlink into another seat's tree.

## 7. Deployment

`App.tsx` is byte-identical on `track/infra-foundation`. The running binary is from 00:03
and this fix is 11:40, and the both-down line's text is absent from the running app's
embedded blobs. Fifth instance of the `eb47ec8` lesson. Reported, not acted on.

## 8. What I did not do

- Did **not** relabel or change her M2 comment. The dispatch said to state a better alternative
  but not to change anything on the way past — I am stating the correction here and leaving
  the edit to whoever owns the file.
- Did **not** fix anything else I noticed (the accumulating reasons list is the only other
  candidate, and it is defensible).
- Did **not** open the app. All four combinations were driven through the real App in jsdom
  with the api seam injected.
- Did **not** write to the live database, open a card, or merge.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/f2 HEAD   # b9e2f06
ln -s <mira>/ui/node_modules /private/tmp/f2/ui/node_modules
cd /private/tmp/f2/ui

./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx     # 20 passed
./node_modules/.bin/vitest run                                      # 300 passed, 0 errors
./node_modules/.bin/tsc --noEmit                                    # clean
./node_modules/.bin/eslint src/App.tsx                              # 15 (same 15 as b9e2f06~1)
./node_modules/.bin/prettier --check src/App.tsx src/components/SupervisorSyncLost.tsx src/App.supervisorSync.test.tsx

# THE FOUR COMBINATIONS (probe: inject rejectStatus / rejectListenFor independently)
#   OK   + OK    -> no strip
#   OK   + fail  -> kind=stale,     no retry, no both-down line
#   fail + OK    -> kind=never-answered, retry, no both-down line
#   fail + fail  -> kind=never-answered, RETRY, both reasons, both-down line
# RETRY directions:
#   succeeds, subs still down -> demotes to stale, keeps sub reasons, drops query reason
#   succeeds, all healthy     -> strip cleared
#   fails again               -> strip stays, reasons ACCUMULATE (3 failures -> 3 entries)

# MUTATIONS (each reverted, tree verified identical afterwards)
#   whole reporter back to a single slot (M1)  -> RED 5  (3 hers + 2 mine)  [Aegis could not
#                                                          reproduce this; their mutations were
#                                                          wrong, her description is complete]
#   kind = failure.kind (M2)                  -> her suite GREEN, my probe RED 2
#   success wipes the whole strip (M3)        -> RED 2
#   both-down line removed (M4)               -> RED 2
#   M2 + query moved after the subscriptions  -> RED 2  (proves the guard is load-bearing)
```

## Verdict event

`ev-hfLMM6Gyn3z7V6qIatMlJ`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-6xJ-8eWA7-LtBDeW9G5Yj` in the payload.

## Files I did not change

No source file. Five mutations were applied and each reverted, with the tree confirmed
byte-identical to `b9e2f06` after each. One worktree was created at
`packet/mira/knowledge-panel` HEAD and removed afterwards; `git status` in Mira's worktree
is empty. My probe files were deleted.
