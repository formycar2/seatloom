# Flux verdict — the accumulated record replacing the single slot (F2)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-both-fail-f2-reissue-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, ui-honesty, seat-flux |

- **Verifying dispatch**: `disp-nQ-SnADq1jXvYN16ucSmv` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-J9ExyK_2-3ajGuXjBvlKj` (Mira)
- **Subject commit**: `b9e2f06` on `packet/mira/knowledge-panel`
- **Verdict event**: **`ev-Q1LjVPXRhHH4_xTQEjlim`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**F2 is fixed.** All four combinations are correct including the fourth — red, retry present,
both error texts present, and "both are down" stated. `subscriptionsDown` is not misused. The
retry-hiding-a-live-problem mirror case is covered. Every mutation I ran kills.

**This is a re-issue of a verification I already performed** (under `disp-viPlKa_e9NoeRHeGe_LCC`,
event `ev-hfLMM6Gyn3z7V6qIatMlJ`), naming a different implementation dispatch. I re-ran the whole
verification from scratch at `b9e2f06` rather than relying on my earlier measurements, and every
number reproduced identically. **The one thing that changed since is the delivery note's wording,
and the change is an improvement — see §6.**

---

## 1. What the defect was, measured rather than accepted

I re-measured the old behaviour as a baseline rather than trusting the "40 lines" figure — the
same discipline as the P0 scrollback dispatch:

```
the defect (single slot, last writer wins), 1500-line history:
  query fails AND both subs fail  ->  yellow 'stale', the query's error text GONE, no retry
```

That is the whole of F2 in one line: when the worse failure arrives second, the operator is told
"you had an answer that went quiet" when the truth was "you never got one", and the error text
they are asked to hand over is gone.

## 2. The four combinations — verified through the real App

Driving the real component with my own probe (scratch, deleted):

| # | Query | Subs | Rendered |
|---|---|---|---|
| 1 | OK | OK | **no strip** |
| 2 | OK | fail | **yellow `stale`**, no retry, no both-down line |
| 3 | fail | OK | **red `never-answered`**, retry present, no both-down line |
| 4 | fail | fail | **red `never-answered`**, **retry present**, **both error texts present**, **"the live push is not connected either"** present |

Case 4 is exactly what F2 was about, and every element is right: the worse shape wins, the retry
is offered, neither error text is dropped, and the operator is told both links are down rather
than left to infer it.

**Her note that our two measurements differed strengthens the finding rather than weakening it.**
She saw the subscription text survive and the query text vanish; I saw the reverse. Both are the
same defect — a single slot with last-writer-wins — and *which* text is lost depends on microtask
ordering between the query's rejection and the two subscription rejections, which neither of us
controls. Two independent observers catching different halves of one loss is better evidence than
either alone.

## 3. `subscriptionsDown` is not misused

| Case | `subscriptionsDown` | Rendered effect |
|---|---|---|
| subscription fails alone | effectively true, but the shape is `stale` | `bothDown = neverAnswered && subscriptionsDown` is **false** → no both-down line |
| query fails alone | false | no both-down line |
| both fail | true, shape `never-answered` | both-down line shown |

`neverAnswered && subscriptionsDown` is the right conjunction, and `subscriptionsDown` is only
meaningful *alongside* `never-answered`, as her comment says. All three verified.

## 4. The retry paths — including the mirror of F2

| Direction | Measured |
|---|---|
| retry succeeds, subscriptions still down | demotes to `stale`, keeps the subscription reasons, **drops** the query reason, no retry button |
| retry succeeds, everything healthy | the strip clears entirely |
| retry fails again | the strip stays and the reasons **accumulate** (three failures → three entries) |

The first row is the important one: it is the exact mirror of F2 (a failure hidden by an unrelated
success), she anticipated it in the code comment, and she covered it with a test. That is the right
instinct — a fix for one direction of a class of bug should be checked in the other.

The third row is a behaviour neither of us raised in the earlier round: reasons accumulate across
retries. Defensible as an honest history, but it means a strip left open through many retries
grows. Not a defect; recorded so it is known rather than surprising.

## 5. Mutations — all kill, and the counts are unchanged from my earlier round

| # | Mutation | Result |
|---|---|---|
| M1 | the whole reporter back to a single slot | **RED 5** — 3 in her suite, 2 in mine |
| M2 | `kind = failure.kind`, order unchanged | **her suite GREEN (20 passed), my probe RED 2** |
| M3 | success wipes the whole strip (the mirror of F2) | **RED 2** |
| M4 | the both-down line removed | **RED 2** |

**M2 is the interesting one and her relabelled note now describes it correctly.** Her own suite
cannot distinguish M2 from the real code; my probe can. That is a coverage gap, not an equivalent
mutation — and she has now changed the word from "equivalent" to "not covered today", which is
the correct label and the one that points at the right action.

## 6. What changed since my earlier verdict — the note, and it is an improvement

The delivery note at `b9e2f06` called M2 an "equivalent mutation". The note at the current branch
head (after the relabel dispatch, `ad31c66`) calls it "not covered today", separates the two
claims explicitly, and — most importantly — **records that my control mutation does not reproduce
against her suite**, with both outputs printed:

```
$ M2 alone                                       ->  20 passed (20)
$ M2 + query moved after the subscriptions          ->  20 passed (20)
```

That is my false `RED (2 failures)` being caught by the person best placed to catch it, and being
written down rather than quietly dropped. **Her self-correction is better evidence of rigour than
a clean delivery would have been.**

**One residual I must flag, because it is my error and this artifact still carries it.** The
delivery note still quotes my control mutation as "→ RED 2" in the section explaining why the
guard is load-bearing (line 171), immediately before printing the measurement that contradicts it.
The note does say both, so it is not a lie — but the false number appears first and a skimmer
takes it. I have verified separately (`97a0e0e`, dispatch `disp-x7QjIhw2ZsvokB_yidBj1`) that the
**code comment** no longer contains it, and that the ordering she identified is now covered by a
real test under which the same mutation is RED 1. **The delivery note's line 171 was not updated
by that dispatch.** Reported here; it is her file and the dispatch said report rather than fix.

## 7. Gates — every figure exact

At `b9e2f06`:
- Her file: **20 passed**.
- Full UI suite: **300 passed / 24 files / 0 errors**. Baseline is 292, so the delta is exactly
  her 8 new tests.
- `tsc --noEmit` clean.
- **eslint: 143 problems on the whole `src` tree, and 15 on her two files before AND after** —
  both diffed against `b9e2f06~1`. Her "no new lint" claim holds, and the 15 are the same
  pre-existing `no-explicit-any`.
- **prettier**: all three changed files clean.
- `vite build` succeeds.

## 8. What I did not do

- Did **not** edit the delivery note's line 171 to remove the false number, and did not edit any
  code. Five mutations were applied and each reverted, with the tree confirmed byte-identical to
  `b9e2f06` after each.
- Did **not** merge. `b9e2f06` is not on `track`; the running binary is from 14:03 against a fix
  committed 11:40, and I confirmed in an earlier dispatch that the 14:03 build predates this fix.
- Did **not** touch Mira's worktree beyond a detached worktree at `b9e2f06`, removed afterwards;
  her `git status` is empty.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/f2r b9e2f06
ln -s <mira>/ui/node_modules /private/tmp/f2r/ui/node_modules
cd /private/tmp/f2r/ui

./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx   # 20 passed
./node_modules/.bin/vitest run                                    # 300 / 24 files
./node_modules/.bin/tsc --noEmit                                  # clean
./node_modules/.bin/eslint src                                    # 143 (same as b9e2f06~1)
./node_modules/.bin/eslint src/App.tsx src/components/SupervisorSyncLost.tsx  # 15 (same)
./node_modules/.bin/prettier --check src/App.tsx src/components/SupervisorSyncLost.tsx \
    src/App.supervisorSync.test.tsx                               # clean
./node_modules/.bin/vite build                                    # ok

# THE FOUR COMBINATIONS (probe: inject rejectStatus / rejectListenFor independently)
#   OK   + OK    -> no strip
#   OK   + fail  -> kind=stale,     no retry, no both-down line
#   fail + OK    -> kind=never-answered, retry, no both-down line
#   fail + fail  -> kind=never-answered, RETRY, both reasons, both-down line
# RETRY directions:
#   succeeds, subs still down -> demotes to stale, keeps sub reasons, drops the query reason
#   succeeds, all healthy     -> strip cleared
#   fails again               -> strip stays, reasons ACCUMULATE

# MUTATIONS (each reverted, tree verified identical afterwards):
#   whole reporter back to a single slot        -> RED 5   (3 hers + 2 mine)
#   kind = failure.kind (order unchanged)       -> her suite 20 passed, my probe RED 2
#   success wipes the whole strip               -> RED 2
#   the both-down line removed                  -> RED 2
```

## Verdict event

`ev-Q1LjVPXRhHH4_xTQEjlim`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-J9ExyK_2-3ajGuXjBvlKj` in the payload.

## Files I did not change

No source file. Five mutations were applied and each reverted, with the tree confirmed
byte-identical to `b9e2f06` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
