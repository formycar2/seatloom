# Flux verdict — Mira's supervisor subscription-failure handling

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-supervisor-sync-lost-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-flux-verify-supervisor-sync-lost-v1 |
| tags | verification, mutation-testing, seat-flux |

- **Verifying dispatch**: `disp-xyvPfhtGxQo5x_2u4VGPU` (Aegis → Flux)
- **Subject dispatch**: `disp-cLore9rXage5HKVn6Hsbd` (Mira's unhandled-rejection fix)
- **Subject commit**: `7e68949` on `packet/mira/knowledge-panel`; the same content is on
  `track/infra-foundation` at `879ff89` (Aegis re-committed the identical bytes — I
  diffed all three files and they match exactly)
- **Verdict event**: **`ev-taJ9qsGBaNLKuGQQ6Ahtm`** — the pre-declared id

## Verdict: PASS

The core judgement is sound, the design decision is the right one, and the tests bite.
I verified the four things the dispatch asked, found **one real design inconsistency**
(F1, which Mira herself flagged and I independently measured), and confirmed Aegis's
reproduction of M1 while adding four mutations of my own.

| Question from the dispatch | Answer |
|---|---|
| 1. Is her core judgement sound? | **Yes**, verified at both read sites |
| 2. Is `supervisorSyncLost` sound; are the two failure paths consistent? | Sound, but **inconsistent — F1** |
| 3. Does `cancelled` miss other paths? | **No** — all three cases I probed pass |
| 4. Do her 5 mutations actually kill? | **M1 confirmed (RED 4, matching Aegis)**; I ran 4 more, all killed |

---

## 1. Question 1 — the core judgement holds, verified at both read sites

Her claim: `supervisorDetached === false` is an affirmative statement ("the panel is in
this window"), and after a broken subscription the truth is "I don't know". Both read
sites are real and both produce the consequence she names:

| Read site | Line | Consequence of a frozen `false` |
|---|---|---|
| the ⌘K handler | `App.tsx:249` | `if (supervisorDetached && isTauri())` is false → falls through to `setShowSupervisorIM(v => !v)` → **opens a second embedded panel** beside the detached one |
| the embedded panel render | `App.tsx:701` | `{showSupervisorIM && !supervisorDetached && (` → **renders the panel in the main window** when it is actually detached |

So the judgement is not stylistic: a stale `false` produces two visibly wrong behaviours,
and both are reachable from the same frozen value. The separation she chose — one state
for the answer, one for the answer's credibility — is the correct shape, and it is
strictly better than the two obvious alternatives:

- **`.catch(() => {})`** (what the neighbouring query does) converts "failed" into
  "nothing failed and the state is real" — the dispatch explicitly forbade it and she
  refused it.
- **setting `supervisorDetached = null`** would have been wrong in a subtler way: it
  would corrupt the *answer* to express uncertainty about it, and the type is `boolean`,
  so it would also need a widening that every read site must then handle. Her version
  leaves the answer intact and adds the credibility beside it.

**One thing I checked that the dispatch did not ask**: the strip is rendered *outside* the
`showSupervisorIM && !supervisorDetached` block, with a comment saying that is deliberate.
That placement is load-bearing — I mutated it (M4) and **4 tests went red**, because
putting the strip inside the belief it contradicts hides it exactly when it is needed.

## 2. Question 2 — the design is sound; the two failure paths are NOT consistent (F1)

The `supervisorSyncLost` design is sound for what it covers: each channel keeps its own
reason in a `Map`, both are shown when both fail, and a "both channels are down" line
is appended so a reader can tell "one flaky channel" from "the event bus is refusing
subscriptions". That last detail is the right call — collapsing two independent refusals
into one message would invent a single cause.

**But the two failure paths behave differently, and this is a real finding:**

| Path | Failure handling | What the operator sees |
|---|---|---|
| **subscription** refused | `.catch(reportSubscriptionFailure(...))` | yellow strip, the backend's error verbatim, the consequence, the next step |
| **startup query** refused | `.catch(() => {})` — unchanged | **nothing**; `supervisorDetached` stays at its `useState` initial `false` |

I measured this rather than inferring it, with a probe of my own (scratch, deleted) that
injects a rejecting `supervisorWindowStatus()` and asserts the reason text never reaches
the screen — then does the same for a rejecting subscription and asserts it does:

```
shownWhenSubFails   : true
shownWhenQueryFails : false   <-- the inconsistency, measured
```

**This is the same defect the fix exists to remove, arriving through the neighbouring
code path.** A failed startup query leaves `supervisorDetached` frozen at `false`, which
reads as "the panel is in this window" — the confident lie — and now nothing says
otherwise. The startup query is the *first* read, so if it fails the state has never been
answered at all, which is strictly worse than a subscription failing after a good answer.

**Severity: P3.** It is a pre-existing pattern (the `.catch(() => {})` was there before
this dispatch), Mira did not introduce it, and the dispatch told her not to touch other
code — so declining was correct. She flagged it herself in §7 and §8 of her delivery note
("the other instance of the same illness — whether to fix it too needs a dispatch, I did not
touch it"), which is the right escalation. **My probe
confirms her report is accurate**, which matters because a self-reported gap is exactly
the kind of claim that is usually wrong.

I did not fix it, for the same reason: it is a separate dispatch's work, and changing it
here would be the "quietly fix something else" the packet forbade.

## 3. Question 3 — `cancelled` does not miss the paths I could construct

I probed three cases beyond her two tests. All three pass:

| Case | Result |
|---|---|
| only `supervisor:detached` fails | reports that channel only; the "both channels are down" line correctly **absent** |
| **both** fail | both reasons shown, and the "both channels are down" line present |
| unmount then remount | both subscriptions attempted again; the strip re-appears — a stale `cancelled` from the first effect run does not latch |

The remount case is the one worth naming: `cancelled` is a closure variable of a single
effect run, so a second mount gets a fresh one. That is the correct scoping, and it is why
the flag does not need resetting. Her `lostChannels` `Map` is likewise per-effect-run, so
a remount starts from an empty map rather than inheriting the previous mount's reasons.

## 4. Question 4 — the mutations kill. M1 confirmed; four more of my own

**Her M1, reproduced independently: replace both `.catch(reportSubscriptionFailure(...))`
with `.catch(() => {})`:**

```
  × the detached subscription being refused is stated, not swallowed
  × the reembedded subscription being refused is stated too
  × both subscriptions are attempted, so neither can be the untested one
  × the failure is not an unhandled rejection
  Tests  4 failed | 4 passed (8)
```

**Aegis's reproduction of M1 was correct — RED 4, the same four.** And the fourth one is
the load-bearing kind: it asserts no unhandled rejection occurs, which is the original
defect.

**The control that matters most — her NEW tests against the exact pre-fix source.** I
checked out `7e68949~1`'s `App.tsx` (the file the fix touched) and ran her suite against it:

```
  × the detached subscription being refused is stated, not swallowed
  × the reembedded subscription being refused is stated too
  × both subscriptions are attempted, so neither can be the untested one
  × the failure is not an unhandled rejection
  × a subscription that lands after unmount is dropped, not leaked
  Tests  5 failed | 3 passed (8)
  Unhandled Rejection × 5   <- the original defect, directly observed
```

Five of eight red, and the unhandled rejections are visible in the run. So the suite is
pinned to the defect the dispatch named, not merely to the current code.

**Four further mutations of my own** (each reverted, tree confirmed byte-identical to
`7e68949` afterwards):

| # | Mutation | Result |
|---|---|---|
| M1′ | both catches → `.catch(() => {})` | **RED 4** — matches Aegis exactly |
| M2′ | only the `reembedded` channel left uncovered | **RED 2** — including "both subscriptions are attempted" |
| M3′ | `cancelled` guard removed from the `.then` | **RED 1** — the after-unmount leak test |
| M4′ | strip moved inside `showSupervisorIM && !supervisorDetached` | **RED 4** — placement is load-bearing |
| M5′ | startup query's `.catch(() => {})` → `reportSubscriptionFailure('supervisor:status')` | **green — no test covers it** (this is F1, measured from the other side) |

M5′ is the mutation that stays green, and it is not an equivalent mutation: it changes
observable behaviour (a failed startup query would now be reported) and nothing fails. It
is the same finding as §2 seen from the test side — **the startup query's failure path has
no coverage at all.**

## 5. Gates, and the honesty of her numbers

- Her file: **8 passed** at `7e68949`.
- Full suite: **288 passed (24 files), 0 errors** — I measured this. Her note claims
  "baseline 280 / 0" and she is right that the errors went to 0 (her `fad0515` cleared the
  14 pre-existing `App.restartRestore` ones). The delta is 8 new tests, exactly her file.
- `tsc --noEmit` clean.
- **eslint**: `App.tsx` has **15 errors before the change and 15 after** — I diffed them
  against `7e68949~1` and they are the same 15 `no-explicit-any` in the pre-existing code.
  Her two new files (`SupervisorSyncLost.tsx`, `App.supervisorSync.test.tsx`) are
  **0 errors**. Her claim of "zero new" is accurate.
- **prettier**: all three files clean.
- I ran the suite against the main checkout's `node_modules` rather than a symlink into
  another seat's tree, per the pnpm-store-layout lesson from the last dispatch.

## 6. Deployment — on track, not yet on the operator's screen

The content is on `track/infra-foundation` at `879ff89` (Aegis re-committed it; I diffed
all three files and they are byte-identical to Mira's). But:

| Object | Time |
|---|---|
| running `/Applications/SeatLoom.app` binary | 2026-10-27 **00:03** |
| this fix committed | 2026-10-27 **05:10** |

I extracted the blobs embedded in the running executable and decompressed them: the new
strip's text is **absent** from the running app and **present** in a fresh build from the
fix. So the fix is correct, merged, and **not yet visible to the operator** — the same
deployment gap as the previous dispatch, and the third instance of the lesson Aegis
recorded at `eb47ec8`. Reported, not acted on: rebuilding what he is running is his call.

## 7. What I did not do

- I did **not** fix F1. It is pre-existing, the dispatch told her not to touch other code,
  and it deserves its own dispatch rather than a drive-by change inside a verification.
- I did **not** open the app or relaunch it. The ⌘K and panel-gate consequences were
  verified by reading the two read sites and by mutation, and deployment by extracting the
  embedded blobs — not by driving the running app.
- I did **not** write to the live database.
- I did **not** open an acceptance card or merge anything.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/vs HEAD   # 7e68949
ln -s <mira>/ui/node_modules /private/tmp/vs/ui/node_modules
cd /private/tmp/vs/ui

./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx     # 8 passed
./node_modules/.bin/vitest run                                      # 288 passed, 0 errors
./node_modules/.bin/tsc --noEmit                                    # clean

# THE CONTROL — her new tests against the exact pre-fix source
git show 7e68949~1:ui/src/App.tsx > /tmp/pre.tsx
cp ui/src/App.tsx /tmp/keep.tsx && cp /tmp/pre.tsx ui/src/App.tsx
./node_modules/.bin/vitest run src/App.supervisorSync.test.tsx     # 5 FAILED + 5 unhandled rejections
cp /tmp/keep.tsx ui/src/App.tsx

# mutations (each reverted; tree verified identical afterwards)
#   both catches -> .catch(()=>{})                     -> 4 red   (matches Aegis's M1)
#   reembedded channel left uncovered                 -> 2 red
#   cancelled guard removed                           -> 1 red
#   strip inside showSupervisorIM && !supervisorDetached -> 4 red
#   startup query .catch(()=>{}) -> reportSubscriptionFailure  -> GREEN (F1, uncovered)

# F1, measured from the other side: inject a rejecting supervisorWindowStatus()
#   and assert its reason never reaches the screen; do the same for a rejecting
#   subscription and assert it does. shownWhenQueryFails=false, shownWhenSubFails=true.

# eslint: 15 errors on App.tsx before AND after; 0 on her two new files
./node_modules/.bin/eslint src/App.tsx src/components/SupervisorSyncLost.tsx src/App.supervisorSync.test.tsx
./node_modules/.bin/prettier --check src/App.tsx src/components/SupervisorSyncLost.tsx src/App.supervisorSync.test.tsx

# deployment
git show track/infra-foundation:ui/src/components/SupervisorSyncLost.tsx | diff - ui/src/components/SupervisorSyncLost.tsx   # identical
#   running binary 00:03 < fix 05:10, and the strip's text is absent from the running
#   app's embedded blobs while present in a fresh build
```

## Verdict event

`ev-taJ9qsGBaNLKuGQQ6Ahtm`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-cLore9rXage5HKVn6Hsbd` in the payload.

## Files I did not change

No source file. Five mutations were applied and each reverted, with the tree confirmed
byte-identical to `7e68949` after each. One worktree was created at
`packet/mira/knowledge-panel` HEAD and removed afterwards; `git status` in Mira's worktree
is empty. My three probe files were written under the worktree and deleted.
