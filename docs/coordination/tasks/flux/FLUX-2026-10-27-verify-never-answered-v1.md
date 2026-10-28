# Flux verdict — the never-answered shape, judged by mutation

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-never-answered-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-flux-verify-never-answered-v1 |
| tags | verification, mutation-testing, ui-shape, seat-flux |

- **Verifying dispatch**: `disp-yqYB1iZlPYe7HceKux71D` (Aegis → Flux)
- **Subject dispatch**: `disp-6xJ-8eWA7-LtBDeW9G5Yj` (Mira's fix for the F1 I found last dispatch)
- **Subject commit**: `35a2d20` on `packet/mira/knowledge-panel`; identical content on
  `track/infra-foundation` (I diffed the file)
- **Verdict event**: **`ev-SNtpi-dbYOEjmkdz4fK7e`** — the pre-declared id

## Verdict: PASS

The decision to keep two shapes is correct and better-reasoned than a merge. All four
mutations kill. **Aegis's M1 reproduction does not reproduce as described** — I get RED 2
across five distinct ways of collapsing the shapes, and RED 3 only for a *different*
mutation (restoring the original F1 swallow), which is what I believe Aegis actually ran.
And I found **one real gap** (F2): when the query *and* the subscriptions both fail, the
worse failure loses the strip.

| Question | Answer |
|---|---|
| 1. Two shapes apart — right? Any third case missed? | Right. **Four combinations measured; the fourth is F2** |
| 2. Red for never-answered — correct convention? | **Yes**, it matches `HydrationFailure` exactly |
| 3. Is the retry button honest? | **Yes, in both directions** — success clears it, a second failure updates the error text |
| 4. Do her 4 mutations kill? | Yes. **Aegis's M1 RED 3 does not reproduce as described** |

---

## 1. Question 1 — two shapes apart is right; four combinations measured

I drove all four combinations through the real App with my own probe (scratch, deleted):

| # | Startup query | Subscriptions | Rendered |
|---|---|---|---|
| 1 | OK | OK | **no strip** |
| 2 | OK | fail | **yellow `stale`**, no retry button |
| 3 | fail | OK | **red `never-answered`**, with retry |
| 4 | **fail** | **fail** | **yellow `stale` only** — the query failure is invisible |

**The separation is the right call, and her three reasons hold up.** The decisive one is
the third: *the copy must differ.* "the answer has stopped being maintained" is **false** when there was never an answer — a strip that says the state is
"the answer from startup" when no startup answer exists is a lie of exactly the kind this
whole design exists to remove. I verified that the two shapes render genuinely different
copy, not just different colours.

The other two reasons also hold:
- **Actionable difference** — a single failed query is fixed by re-reading; a broken event
  subscription is not. Offering retry on the stale shape would be a button that cannot
  work (I mutation-tested that: M3 kills).
- **Severity** — no answer is worse than an answer gone stale, and same-colour would
  display them as equal.

### F2 — the fourth combination shows the wrong shape (P3)

When the query **and** both subscriptions fail, the strip the operator sees is
`stale` — yellow, "your answer has stopped being maintained", **no retry** — and the
`never-answered` information is gone entirely: not the red, not the retry button, not the
query's error text. Measured:

```
kind=stale  showsNeverAnswered=false  showsQueryError=false  showsSubError=true  retry=null
```

So the operator is told "you had an answer that went quiet" when the truth is "you never
got an answer at all" — the worse of the two failures is the one that is hidden, and the
one action that would help (re-asking) is the one not offered.

**Why it happens**, precisely: `supervisorSyncLost` is a single nullable slot. The query's
`.catch` sets `{kind:'never-answered'}`; the subscription reporter then sets
`{kind:'stale'}` — last writer wins, and the subscription rejections land after the query
rejection because both subscriptions are `listen()` promises resolved in a later microtask.
The two reporters do not know about each other.

**Severity P3, not P2**: both failures together means the event bus is broadly broken, in
which case the restart advice in the stale strip is right and the retry would probably fail
too. But the *message* is still wrong, and the query's error text is lost, which is the
evidence the operator is asked to hand to Aegis. The honest fix is to keep both (as she
already does for two subscription channels with her `Map` and "both channels are down"
line) rather than let the last writer win. **Reported, not fixed** — same boundary as last
dispatch, and the dispatch invited exactly this ("if you find another finding, report it").

## 2. Question 2 — red is the right colour, and it is the product's convention

The dispatch asked whether red is reserved for data-read failures in this product. It is,
and this **is** a data-read failure:

| Component | Shape |
|---|---|
| `HydrationFailure.tsx` | `role="alert"`, `var(--sl-red)` border + background, `AlertTriangle`, the error verbatim, **and a retry button** |
| `SupervisorSyncLost` (`never-answered`) | `var(--sl-red)` border + background, `AlertTriangle`, the error verbatim, **and a retry button** |

Same colour, same icon, same structure, same affordance — for the same class of thing: a
read that did not answer. `HistorySessionsPanel` and `TmuxMirrorWorkspace` also use
`sl-red` for read/parse failures. So the choice is consistent with the product rather than
invented, and the yellow-vs-red split correctly encodes "there is something to fall back
on" vs "there is not".

## 3. Question 3 — the retry button is honest in both directions

**Forward direction (success clears it):** verified. A rejecting query, then the retry
after the query starts succeeding → the strip disappears and `statusCalls` goes to 2, so it
really re-asked. Her `setSupervisorSyncLost(null)` on success is what makes this true, and
without it the button is exactly what she called it — "a lie with a click handler". I
mutation-tested that removal (M2, kills).

**Reverse direction (retry fails again):** verified, and this is the part most likely to be
wrong. I made the retry fail with a *different* error text and checked what renders:

```
RETRY-AGAIN  showsFirst=false  showsSecond=true
```

The strip stays and the error text **updates to the new failure** — it does not keep
showing the stale first error. That is the honest behaviour: the operator sees the current
reason, not the first one. And the strip stays rather than disappearing, which is right
because the state is still unanswered.

One thing I checked beyond the question: the retry calls `askSupervisorWindowStatus`, which
deliberately does **not** check `cancelled`. Her comment explains why — a retry is an
explicit operator action taken while mounted, so the flag does not apply. That reasoning is
correct: `cancelled` guards the *effect's* in-flight window, not a user-initiated read.

## 4. Question 4 — the mutations kill; Aegis's M1 RED 3 does not reproduce as described

Her 12 tests pass at `35a2d20`. The decisive control — her new tests against the pre-fix
code — is covered by the previous dispatch's verdict, so here I ran the four mutations:

| # | Mutation | Result |
|---|---|---|
| **M4** | query failure swallowed again (the original F1 restored) | **RED 3** — the stated, not-swallowed, no-unhandled-rejection, and retry tests |
| **M1a** | query failure reported as `kind:'stale'` (state side) | **RED 2** |
| **M1b** | render side collapses both (`neverAnswered = false`) | **RED 2** |
| **M1c** | both sides collapsed | **RED 2** |
| **M1d** | query failure routed through the stale reporter | **RED 2** |
| **M1e** | retry button removed from both shapes | **RED 1** |
| **M2** | success does not clear the strip | **RED 1** |
| **M3** | retry offered on the stale shape too | **RED 1** |

**On Aegis's M1:** the dispatch says Aegis reproduced M1 as "merge the two failure paths
into one" → RED 3. I tried five distinct ways of merging them (state side, render side,
both, routing the query failure through the stale reporter, removing the retry) and got
**RED 2 every time**, never 3. The mutation that *does* give RED 3 is **M4 — restoring the
`.catch(() => {})` swallow**, i.e. the original F1 rather than a merge.

So either Aegis's mutation was the swallow rather than the merge (most likely — the label
"M1" in Mira's own table may have shifted between her note and Aegis's run), or Aegis's
reproduction differed from the description. **I am not claiming Aegis was wrong** — I am
reporting that I cannot reproduce RED 3 for a merge, and that the merge is nonetheless
killed (RED 2), which is what matters. The direction of the finding is unchanged: the tests
bite.

**The control that matters most**, re-run here: with the query failure swallowed again
(M4), the failure is invisible *and* the state still claims `false` — 3 tests catch it, one
of which asserts no unhandled rejection. So the specific defect I found last dispatch is
now genuinely covered.

## 5. Gates

- Her file: **12 passed** (8 from the previous dispatch + 4 new).
- Full suite: **292 passed (24 files), 0 errors**. Previous dispatch measured 288, so the
  delta is exactly her 4 new tests.
- `tsc --noEmit` clean.
- **eslint**: 15 errors on `App.tsx` **before and after** (diffed against `35a2d20~1` — the
  same 15 pre-existing `no-explicit-any`); **0 on both of her files**. Her "zero new" claim
  is accurate.
- **prettier**: all three files clean.
- Ran against the main checkout's `node_modules`, not a symlink into another seat's tree.

## 6. Deployment

Content is on `track/infra-foundation` (I diffed `SupervisorSyncLost.tsx` — identical). The
running binary is from **00:03** and this fix is **07:06**, and the never-answered strip's
text is absent from the running app's embedded blobs. Fourth instance of the `eb47ec8`
lesson; reported, not acted on.

## 7. What I did not do

- Did **not** fix F2. It is a design change inside another seat's delivery, and the dispatch
  invited findings rather than edits. The honest fix (keep both reasons, as she already does
  for two subscription channels) is stated above for whoever picks it up.
- Did **not** open the app. All four combinations were driven through the real App in jsdom
  with the api seam injected; the colour convention was verified by reading
  `HydrationFailure` and the two other `sl-red` users.
- Did **not** write to the live database, open a card, or merge.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/na HEAD   # 35a2d20
ln -s <mira>/ui/node_modules /private/tmp/na/ui/node_modules
cd /private/tmp/na/ui

./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx     # 12 passed
./node_modules/.bin/vitest run                                      # 292 passed, 0 errors
./node_modules/.bin/tsc --noEmit                                    # clean
./node_modules/.bin/eslint src/App.tsx                              # 15 (pre-existing, same 15 as 35a2d20~1)
./node_modules/.bin/prettier --check src/App.tsx src/components/SupervisorSyncLost.tsx src/App.supervisorSync.test.tsx

# THE FOUR COMBINATIONS (probe: inject rejectStatus / rejectListenFor independently)
#   query OK   + subs OK    -> no strip
#   query OK   + subs fail  -> kind=stale,     no retry button
#   query fail + subs OK    -> kind=never-answered, retry present
#   query fail + subs fail  -> kind=stale ONLY   <-- F2: the query failure and its retry vanish
# RETRY, both directions:
#   retry succeeds -> strip clears, statusCalls 1 -> 2
#   retry fails again with a different error -> strip stays, error text UPDATES (not the first)

# MUTATIONS (each reverted, tree verified identical afterwards)
#   query .catch restored to () => {}              -> RED 3   <-- the one that gives 3
#   query failure as kind:'stale' (state side)     -> RED 2
#   render side neverAnswered = false              -> RED 2
#   both sides collapsed                           -> RED 2
#   query failure routed via the stale reporter    -> RED 2
#   retry button removed from both shapes          -> RED 1
#   success does not clear the strip               -> RED 1
#   retry offered on the stale shape too           -> RED 1
# NB: five distinct "merge the two shapes" mutations all give RED 2, never 3.
#     Aegis's RED 3 reproduces only for the swallow (the original F1).
```

## Verdict event

`ev-SNtpi-dbYOEjmkdz4fK7e`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-6xJ-8eWA7-LtBDeW9G5Yj` in the payload.

## Files I did not change

No source file. Eight mutations were applied and each reverted, with the tree confirmed
byte-identical to `35a2d20` after each. One worktree was created at
`packet/mira/knowledge-panel` HEAD and removed afterwards; `git status` in Mira's worktree
is empty. My probe files were deleted.
