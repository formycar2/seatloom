# Flux verdict — the P0 live-seat scrollback fix (`dropStaleSnapshotViewport`)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-p0-restore-seat-scrollback-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, xterm, p0, seat-flux |

- **Verifying dispatch**: `disp--Hvv9ROFzarJ7OrPUCgsT` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-OwOSLvL4EbqoB6YhlBQHi` (Mira)
- **Subject commit**: `9719937` on `packet/mira/knowledge-panel` — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-nbpmWap9wlZue7V_A70D1`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The P0 is fixed on the UI side, within the constraints she was given.** The operator's
report — every seat but the one being looked at cannot scroll its live pane — is explained by
one branch and fixed by one function, and I confirmed all five of her claims by driving the
real component and the real xterm, not by reading her note.

**Two things I found that she did not state**, one of which is a genuine gap:

1. **Her "component decision" and "component behaviour" are verified by two different
   harnesses, and the boundary between them is load-bearing.** I measured that the
   component's clear branch is *unreachable* in jsdom and found the precise cause
   (see §3). That is the reason the split exists, and it means the fix's **call site** is
   verified only by a transcribed FitAddon — which is honest, but it is the weakest link in
   the evidence and nobody had said so out loud.
2. **Her declared residual (reflow debris) is measured in prose but pinned by no test.** I
   reproduced it exactly (a 100-character line into 40 columns becomes 3 rows, all retained),
   and confirmed her boundary claim (45 characters at 80→78: zero reflow). Both should be
   tests, because they are the two numbers a future change could silently break.

---

## 1. What the defect was, measured rather than accepted

I measured the old behaviour as a baseline rather than trusting the "40 lines" figure:

```
defect baseline (same component mount, 1500-line history, clear()+reset()):
  before = 1501 lines      after = 25 lines   (== rows)
```

`reset()` on its own leaves exactly `rows` lines; the combined pair leaves one more than the
row count. Her "40 lines of a 300-line history at 40 rows" is the same measurement at a
different size. **Confirmed, and I reproduced it independently.**

Her added explanation for why the active seat worked is the part worth keeping: the clear
branch is conditional on `seedDims.rows !== rows || seedDims.cols !== cols`, so the seat the
operator happened to be looking at fitted to its seed, the branch never ran, and its history
survived. **The difference between seats was never about the seats** — that is a better root
cause than "Aegis was looking at it", and it is what makes the fix the right shape.

## 2. The fix, verified through the real component

Driving the real `SessionTerminal` with the real xterm (real `.open()`, real buffer, real
parser — the only stubs are the two jsdom lacks: `matchMedia` and the canvas 2d context, both
stated):

| Criterion | Measured |
|---|---|
| 1. a long history survives and scrolls past 1000 lines | 1501 lines, oldest line `L000 a a a a` intact; `scrollLines(-1200)` moves `ydisp` to 276 off a `ybase` of 1476, landing on `L276 a a a a a` — real old history, not a blank buffer |
| 2. the input box is not misplaced | after the drop and `scrollToBottom`, a repaint of `SAIH PROMPT > ` is visible in the viewport (it appears at the last viewport row, since the history fills the screen) |
| 3. switch away and back | a fresh mount re-attaches: 1501 lines again, `L000` intact. `clearedStaleSnapshotRef` is per-mount and the snapshot is re-written each attach, so the history is re-acquired, not re-accumulated |
| 4. existing tests | her 20 pass at `9719937`; full suite **336 passed / 29 files / 0 errors** |
| 5. real-machine verification | **still open** — she cannot click the GUI, and the fix is not in the running binary (see §6) |

Criterion 5 is genuinely blocked and she said so plainly rather than claiming it. That is the
right call: the other four are measurable and she measured them; this one needs a human at a
keyboard.

## 3. The reachability limit — the thing nobody had stated

Her delivery note says jsdom cannot reach the component's fit branch, and gives the reason. I
verified that by **instrumenting the product function with a counter** and driving the real
component: **the function is called 0 times.** The cause is precise:

`FitAddon.proposeDimensions()` returns `undefined` when `dimensions.css.cell.width === 0`,
and the cell size comes from xterm's `CharSizeService`, which in jsdom has no `OffscreenCanvas`
(undefined) and no canvas 2d context. So `syncGeometry()` bails at `if (!proposed)` and the
clear branch never executes. Her note's phrasing ("jsdom cannot give xterm the character-cell
metrics") is correct; the mechanism is `CharSizeService` → zero cell size → fit is a no-op.

**Why this matters and is not just a footnote:** it means the two halves of her evidence are
not interchangeable.

- `SessionTerminal.scrollback.test.tsx` — real xterm, but it **calls the exported function
  directly** after a `resize` (lines 250, 276, 311). It proves what the function does to a
  real buffer. It says nothing about whether the component reaches it.
- `SessionTerminal.geometry.test.tsx` — a transcribed FitAddon + FakeTerminal. It proves the
  component's **decision**: `clearCalls === 0`, `resetCalls === 0`, an `ESC[2J` write, one
  `scrollToBottom`, and the geometry still pushed to the pane. But it is a transcription of
  the contract, so it would not catch a change in xterm's own behaviour.

Both halves are needed and she split them deliberately, with the reasoning written down. My
only correction is to the framing: the split is not a convenience, it is **forced** by the
reachability limit, and the fix's call site is therefore only verified by a transcription.
That is worth writing in the note so the next person does not mistake it for a stylistic
choice, and so a future xterm upgrade knows which test to re-check.

I tried to close the gap by injecting real cell metrics into `CharSizeService` from the test.
It did not work — the service is constructed inside `Terminal.open()` and the metrics are read
through the private core, so the injection lands after the fit has already bailed. I am
reporting the attempt rather than hiding it, because "I could not reach it either" is the
evidence that her split is necessary rather than lazy.

## 4. Mutations — all three kill

| # | Mutation | Result | Caught by |
|---|---|---|---|
| M1 | the function back to `clear()+reset()` | **RED 5** | 2 geometry + 3 scrollback — exactly her claim |
| M2 | drop `scrollToBottom()` | **RED 2** | geometry (both the differing-geometry and the clear-once tests) |
| M3 | drop the `ESC[2J` viewport clear | **RED 3** | geometry ×2 + scrollback ×1 (the "not garbled" one) |

M1's kill count matches her claim of 5 exactly. M2 and M3 kill in `geometry.test.tsx` — the
transcription half — which is the right place: those two mutations are about what the
component asks for.

**One thing I checked that she did not, and it changes nothing but is worth recording:** under
M1, *my* component-driven probe (the real-xterm half) stays green, because the component's
clear branch never runs in jsdom at all. So the mutation that destroys the scrollback is
killed only by tests that call the function directly. That is a real consequence of the §3
limit: **the "does it destroy the history" mutation is only caught in a harness that bypasses
the component.** If someone later "simplifies" `scrollback.test.tsx` to drive the component
instead of the function, M1 would go undetected.

## 5. Her declared residual, measured precisely

Her claim: `resize` reflows a line wider than the new column count onto extra rows, and those
rows stay in the scrollback. Measured boundary: 45-character lines at 80→78, zero reflow.

```
100-char line into 40 columns:  before = 302 lines  ->  after = 304 lines
                                 rows 300, 301, 302 are all 'W' (the reflow)
45-char lines at 80->78:         before = 301 lines  ->  after = 301 lines, oldest intact
resize alone (80->120 and 80->78): 301 -> 301, oldest intact, both directions
```

**Her residual claim is exactly right**, and her boundary is right too. Both numbers are
correct and reproducible. Neither is pinned by a test — they exist only in the file's header
comment and her delivery note. Two concrete suggestions, neither of which I acted on:

- a test that a 100-character line into 40 columns produces 3 rows and all 3 survive (pins
  the residual so a future "optimisation" cannot silently increase it);
- a test that 45-character lines at 80→78 produce zero extra rows (pins her boundary, which
  is the claim that makes the residual acceptable for real TUI output).

Her reasoning for why this is the honest limit is also correct: eliminating it would need a
re-capture of history at the new geometry, which lives in the backend (`pty/mod.rs` backfill
is attach-time and sized to the pane), and the dispatch forbade touching the backend. She
took the constraint and stated the limit rather than quietly shipping a partial fix as a
complete one.

## 6. Gates and deployment

**Gates at `9719937`:** full suite **336 passed / 29 files / 0 errors**; `tsc --noEmit` clean;
`eslint src` **143 problems (133 errors + 10 warnings)**; `prettier` clean on all four changed
files; `vite build` succeeds.

Her eslint claim of "143, identical to baseline" is accurate in count. I diffed the
`SessionTerminal.tsx` file specifically against the pre-fix commit and the 15
`no-explicit-any` errors are unchanged; the 143 is the whole-`src` figure and is dominated by
pre-existing `App.tsx` debt. Her claim to have added zero new lint is correct, and the
`react-refresh/only-export-components` warning she hit mid-way was genuinely resolved by
extracting the file rather than by a disable comment — I checked that no `eslint-disable`
appears in any of the four files.

**Deployment — three separate facts, none of which is Flux's to act on:**

1. `9719937` is **not an ancestor of `track/infra-foundation`** and the file does not exist in
   the main repo. The fix is not merged.
2. The running binary was built **14:03**, the fix was committed **17:14** — three hours
   later.
3. The `ESC[2J ESC[H` sequence is **absent** from the running app's embedded assets, while
   the F2 both-down string (committed 11:40) **is present**. So the 14:03 build predates this
   fix and the earlier F2 deployment gap is now closed.

**So the operator cannot observe this fix yet, and the blocker she flagged (the bundle missing
frontend assets) is no longer it** — the assets are embedded and decompressible now. The
blocker is simply that nothing has been rebuilt or merged since 17:14. Reported, not acted on.

## 7. What I did not do

- Did **not** merge anything, rebuild, or reinstall. Deployment is the operator's call.
- Did **not** touch Mira's worktree beyond a detached worktree at `9719937`, which was
  removed afterwards; her `git status` is empty.
- Did **not** open the app or drive the GUI. Criterion 5 remains open and is a human task.
- Did **not** write to the live database, open a card, or close anything on her behalf.
- Did **not** add the two residual tests suggested in §5 — that is her file, and the dispatch
  said to report rather than fix.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/p0 9719937
ln -s <mira>/ui/node_modules /private/tmp/p0/ui/node_modules
cd /private/tmp/p0/ui

./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.scrollback.test.tsx   # 3 passed
./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.geometry.test.tsx     # 3 passed
./node_modules/.bin/vitest run                                                        # 336 passed / 29 files
./node_modules/.bin/tsc --noEmit                                                       # clean
./node_modules/.bin/eslint src                                                         # 143 (same as baseline)
./node_modules/.bin/prettier --check src/app-v2/panel/{dropStaleSnapshotViewport.ts,SessionTerminal.tsx,SessionTerminal.scrollback.test.tsx,SessionTerminal.geometry.test.tsx}
./node_modules/.bin/vite build                                                         # ok

# THE DEFECT, MEASURED AS A BASELINE (real component, real xterm, 1500 lines):
#   term.clear(); term.reset()   ->  1501 lines become 25 (== rows)
#
# THE FIX, THROUGH THE REAL COMPONENT:
#   oldest line survives, ydisp leaves ybase, scrollLines(-1200) lands on L276
#   remount -> 1501 lines again, L000 intact
#
# THE REACHABILITY LIMIT (instrument the product function with a counter):
#   the component calls dropStaleSnapshotViewport ZERO times in jsdom,
#   because CharSizeService has no metrics -> proposeDimensions() returns
#   undefined -> syncGeometry() bails before the clear branch.
#   This is why the call site is verified only by a transcription.
#
# MUTATIONS (each reverted, tree verified identical afterwards):
#   back to clear()+reset()      -> RED 5   (2 geometry + 3 scrollback)
#   drop scrollToBottom()        -> RED 2   (geometry)
#   drop the ESC[2J clear        -> RED 3   (geometry x2 + scrollback x1)
#   ...and under the first mutation MY component-driven probe stays green,
#      because the component's branch never runs in jsdom at all.
#
# THE DECLARED RESIDUAL:
#   100-char line into 40 cols -> 3 reflow rows, all retained in the scrollback
#   45-char lines at 80->78    -> zero reflow, oldest line intact
```

## Verdict event

`ev-nbpmWap9wlZue7V_A70D1`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-OwOSLvL4EbqoB6YhlBQHi` in the payload.

## Files I did not change

No source file. Four mutations were applied and each reverted, with the tree confirmed
byte-identical to `9719937` after each. One detached worktree was created at Mira's packet
branch and removed afterwards. My probe files were deleted.
