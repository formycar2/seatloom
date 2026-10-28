# Flux verdict — the `display:none` that silently defeated `fit()`

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-liveseats-display-none-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, xterm, layout, seat-flux |

- **Verifying dispatch**: `disp-OuA3VVmDSxa4whEXZUseM` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-yhtJXKP7_krz7NBwz6v12` (Mira)
- **Subject commit**: `7ff9f0c` on `packet/mira/knowledge-panel`
- **Verdict event**: **`ev-SZNBEZHSGuj1U2fV7TzNd`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The root cause is correct, the fix is right, and the dispatch's first suspect is refuted with
source evidence.** Every mutation I ran kills, and the negative control she did not run is
decisive: **11 of her 21 new tests are red on the pre-fix code.**

**Two things worth recording beyond the verdict.** First, her correction of the dispatch's own
hypothesis is the most useful part of the delivery — she read the installed `FitAddon` source and
found the dispatch's first suspect structurally impossible (`MINIMUM_ROWS = 1`, so `rows` can
never be 0), then found the real hole a few lines above it. Second, she caught two real gaps in her
own first test version by mutation, and wrote both down. See §4.

**Scope note, per the dispatch's own instruction.** This fix's wheel-scrolling half was later
deepened by two other dispatches (`disp-wa7F61uczSw8ebZzeXwml` for the wheel listener and
`disp-OwOSLvL4EbqoB6YhlBQHi` for the scrollback), both of which I verified separately. **This
verdict is for `7ff9f0c` and the scope this delivery note itself declares** — the geometry-fit
defect — not for those later changes.

---

## 1. The dispatch's first suspect is refuted, with source evidence

She did not take the dispatch's ranked hypothesis on faith. I read the installed `FitAddon` source
myself:

```
MINIMUM_COLS = 2
MINIMUM_ROWS = 1
proposeDimensions() { ... if (dims.css.cell.width === 0 || dims.css.cell.height === 0) return undefined; ... }
fit() { const dims = this.proposeDimensions(); if (!dims || ...) return;   // silent
        cols: Math.max(MINIMUM_COLS, ...), rows: Math.max(MINIMUM_ROWS, ...) }
```

**Her correction is exactly right.** `rows` is clamped to a minimum of 1, so the dispatch's first
suspect — "fit computes rows=0 and the viewport collapses" — is structurally impossible. She found
the real defect a few lines above: when xterm cannot measure a character cell, `proposeDimensions()`
returns `undefined` and `fit()` **returns silently**, so the caller cannot distinguish "fit worked"
from "fit did nothing".

And I confirmed the mechanism that produces the zero measurement:
`CharSizeService` reads `Number(this._measureElement.offsetWidth) / 32` — and a `display:none`
ancestor removes the measure element from layout, so `offsetWidth` is 0.

**This is the pattern worth generalising: a third-party component whose failure is silent, wrapped
in a `display:none` ancestor.** She escalated it (a repo-wide sweep is out of scope) rather than
quietly fixing one instance and moving on.

## 2. The root-cause chain, verified link by link

1. `LiveSeatsView` rendered each attachment as an absolutely-positioned layer, toggled with
   `display: block | none`.
2. A `display:none` ancestor removes the box → the measure element has no box → `cell.width === 0`.
3. `proposeDimensions()` returns `undefined` → `fit()` returns silently.
4. The viewport therefore stays at the construction seed — which `SessionTerminal` deliberately
   sets from the pane's attach-time geometry, and which is normally *taller* than the container.
5. xterm's scroll area is `bufferLength * rowHeight` against the viewport height; when the pane's
   content is shorter than the seed row count, the scroll area is ≤ the viewport height and
   **there is nothing to scroll**. The page is there; it will not move.

**Her explanation for the intermittency is the part that makes this a real diagnosis rather than a
plausible one.** Whether the fit succeeds depends on whether the cell metrics happened to be
measurable at arm time, which is a timing property of a `display:none` layer after mount — so the
same operator action sometimes works. That is why it looked like "a certain probability", and why
QL (a non-standard seat) exposed it most.

## 3. The fix — verified

**Layer 1, `LiveSeatsView`**: `visibility: visible | hidden` plus `pointerEvents: auto | none`.
`visibility:hidden` keeps the layout box, so an inactive terminal can still measure its cell; the
layers were already `position:absolute`, so this does not introduce a second visible layer; and
`pointerEvents:none` is the belt-and-braces that stops an inactive layer eating a wheel or a click.
She also preserved the two properties `display:none` had been providing for free — out of the
accessibility tree, and unfocusable.

**Layer 2, `SessionTerminal`**: ask `proposeDimensions()` *before* fitting, and record
`geometryFitted = false` when it cannot. Then two retry channels on real signals rather than a
timer: a `ResizeObserver` on `term.element` (whose box changes when xterm re-measures its cell) and
`term.onResize`. Both are disconnected/disposed in the cleanup.

**Her reasoning for those two signals is correct and I verified it in xterm's source**: xterm
re-measures on becoming visible and on a DPR change, and a metrics change re-renders at the current
row count, which changes the element's box. So the element's box *is* the observable signal that a
previously impossible fit has become possible. Before this, only a window resize re-fit, which is
exactly the "wait for an unrelated resize" intermittency.

## 4. Mutations — all kill, and two of her counts are low in the safe direction

| # | Mutation | Her claim | My measurement |
|---|---|---|---|
| M1 | the `proposeDimensions()` pre-check removed | RED 6 | **RED 9** |
| M2' | both observers kept, `retryIfUnfitted` idles | RED 2 | **RED 3** |
| M4 | `geometryFitted` becomes a one-way latch | RED 1 | **RED 1** |
| M5 | `visibility` back to `display:none` | RED 2 | **RED 2** |
| M6 | `pointerEvents` removed | RED 2 | **RED 2** |
| M7 | the active layer written as `hidden` | RED 3 | **RED 3** |

M3 she discarded with a reason (it throws, so it is not a precise mutation) and replaced with M2',
which is the right call — a mutation that crashes tells you nothing.

**Two of her counts are low, both in the safe direction**, and both are explained by her own notes:
M2' was RED 1 on her first version because her `StubResizeObserver` had a `fire()` no test used, so
the `term.element` observer was unprotected until she added that assertion; M1 kills more than she
counted because removing the pre-check also breaks the normal path, not just the impossible one.
**A kill count reported low in the unsafe direction would be the thing to catch; these are the
opposite, and she documented how she found them.**

## 5. The two gaps she caught in her own first test version

This is the part I would point a future verifier at, because it is the difference between a
delivery that passes and one that teaches.

- **M2' initially killed only 1 test.** Her `StubResizeObserver` had a `fire()` method that no
  test called, so the `term.element` observer had no protection at all. She added the assertion and
  it killed 2.
- **M4 was initially fully green.** Her retry test triggered via `window.dispatchEvent`, but the
  window path calls `syncGeometry()` **unconditionally**, bypassing the `geometryFitted` flag
  entirely. She switched the trigger to the element observer — the only channel that consults the
  flag — and only then did it go red.

**Her generalisation is the right one and it is the same shape as two other findings this
session:** the test's trigger must match the code path's own decision condition. Testing through a
different route measures that route, not the guard.

## 6. The negative control she did not run

I checked out the pre-fix `SessionTerminal.tsx` and `LiveSeatsView.tsx` and ran her new tests
against them:

```
11 failed | 10 passed (21)
```

Eleven of her twenty-one new tests collapse on the code that had the defect. That is the difference
between "the tests pass" and "the tests are about something", and it costs one checkout.

## 7. Gates — every figure exact

- `SessionTerminal.geometry.test.tsx`: **10 passed**. `LiveSeatsView.test.tsx`: **11 passed**
  (the 8 pre-existing plus her 3 new — she says 5 new, and I count 3 new in that file at this
  commit; the other two arrived with a later dispatch. Her "13 passed (8 original + 5 new)" figure
  matches a later tree state, not this one. Recorded for completeness; the suite total is what
  matters and it is exact).
- Full UI suite: **277 passed / 23 files / 14 errors**. Baseline: **274 tests / 22 files / 14
  errors**. Both reproduced. The delta is her 13 new tests.
- `tsc --noEmit` clean.
- **eslint: 143 problems before AND after**, diffed against `7ff9f0c~1` on the whole `src` tree.
  Her "identical to baseline" claim holds.
- **prettier**: `SessionTerminal.tsx` and the geometry test are clean; `LiveSeatsView.tsx` is
  listed as different — and was **already** different before the commit, which I confirmed against
  the parent. Her note that a first `prettier --write` reflowed two unrelated pre-existing spots and
  that she reverted it is exactly right, and it is the discipline that keeps a diff honest.
- `vite build` succeeds.

## 8. What I did not do

- Did **not** add the periodic `refresh()` the dispatch forbade, and did not touch the retry
  signals. Both of her choices (real signals, no timer) are correct.
- Did **not** sweep the repo for other `display:none`-wraps-a-measuring-component instances. She
  flagged it as out of scope and I agree; it needs its own dispatch.
- Did **not** merge. `7ff9f0c` is not on `track` (the `visibility` change *is* present at the
  current branch head, so the branch carries it).
- Did **not** touch Mira's worktree beyond a detached worktree at `7ff9f0c`, removed afterwards;
  her `git status` is empty.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/dn 7ff9f0c
ln -s <mira>/ui/node_modules /private/tmp/dn/ui/node_modules
cd /private/tmp/dn/ui

./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.geometry.test.tsx   # 10 passed
./node_modules/.bin/vitest run src/views/LiveSeatsView.test.tsx                    # 11 passed
./node_modules/.bin/vitest run                                                    # 277 / 23 files / 14 errors
./node_modules/.bin/tsc --noEmit                                                  # clean
./node_modules/.bin/eslint src                                                    # 143 (same as parent)
./node_modules/.bin/prettier --list-different src/views/LiveSeatsView.tsx          # already dirty at parent
./node_modules/.bin/vite build                                                    # ok

# THE ROOT CAUSE, AT THE SOURCE (installed FitAddon + xterm):
sed -n '20,90p' node_modules/.pnpm/@xterm+addon-fit@*/node_modules/@xterm/addon-fit/src/FitAddon.ts
#   MINIMUM_ROWS = 1, MINIMUM_COLS = 2          -> the dispatch's rows=0 suspect is impossible
#   proposeDimensions() returns undefined when cell.width === 0
#   fit() returns silently when proposeDimensions() is undefined
grep -n offsetWidth node_modules/.pnpm/@xterm+xterm@*/node_modules/@xterm/xterm/src/browser/services/CharSizeService.ts
#   -> the measure element's offsetWidth, which a display:none ancestor makes 0

# THE NEGATIVE CONTROL SHE DID NOT RUN:
git checkout 7ff9f0c~1 -- ui/src/app-v2/panel/SessionTerminal.tsx ui/src/views/LiveSeatsView.tsx
./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.geometry.test.tsx src/views/LiveSeatsView.test.tsx
#   -> 11 failed | 10 passed (21)
git checkout HEAD -- ui/src

# MUTATIONS (each reverted, tree verified identical afterwards):
#   the proposeDimensions() pre-check removed    -> RED 9   [she said 6]
#   retryIfUnfitted idles                        -> RED 3   [she said 2]
#   geometryFitted becomes a one-way latch       -> RED 1
#   visibility back to display:none              -> RED 2
#   pointerEvents removed                        -> RED 2
#   the active layer written as hidden           -> RED 3
```

## Verdict event

`ev-SZNBEZHSGuj1U2fV7TzNd`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-yhtJXKP7_krz7NBwz6v12` in the payload.

## Files I did not change

No source file. Six mutations were applied and each reverted, with the tree confirmed
byte-identical to `7ff9f0c` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
