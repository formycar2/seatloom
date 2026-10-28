# Flux verdict — the InitDialog coverage gap (my F1), closed

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-initdialog-coverage-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, coverage, own-error, seat-flux |

- **Verifying dispatch**: `disp-w_-TIYqGYCf7yHrmSFqHP` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-B4Y5oAhMQwtXm5Vva1fw` (Mira)
- **Subject commit**: `fad0515` on `packet/mira/knowledge-panel`
- **Verdict event**: **`ev-usP7dX7geulRhaDy1eXwD`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**My F1 is closed.** The `InitDialog` `onInitialize` path — which I found had zero test coverage —
now has two tests that drive the real dialog and the real button, and **every mutation I ran dies,
including the two she reported as having been false greens on her first attempt.** Source is
untouched: the commit is one test file and one delivery note, nothing else.

**The most valuable thing in this delivery is that she caught a second instance of the
"final-value assertion cannot see a window-period error" pattern, in her own new test, before
shipping it** — and then wrote down both the miss and the fix. See §4.

---

## 1. The gap she was asked to close — and why my 7 tests could not see it

My original finding was that reverting `InitDialog`'s `onInitialize` from
`hydrateFromBackend()` back to `setActiveProject('p-1')` left all 7 of my tests green. Her
diagnosis of why is correct and I verified it in the source: `InitDialog` opens only when
`restoredPosition && !activeProjectId`, which only holds when the backend has **no projects at
all**. My 7 tests all mounted with the `CANNED` fixture that has projects, so the dialog never
appeared and the button was never reachable.

**She also corrected a transcription error in the dispatch.** The dispatch said M6 was in the
`mutation_table` field of my verdict event; it is in `finding_F1`. She read the ledger original
rather than the dispatch's paraphrase, which is the right instinct and the reason she was looking
in the right place.

## 2. What she added — verified by driving it

Two tests appended to `src/App.restartRestore.test.tsx`, using the file's existing
instrumentation (on-disk write log + call-order log) rather than new machinery:

1. **no backend projects at all** — click → invents nothing (`activeProjectId` stays `null`),
   writes nothing to disk, **re-reads the backend** (`cmd_list_projects` appears more than once),
   and the dialog closes.
2. **projects have appeared since launch** — click → lands on the real project `seatloom`, an id
   the backend actually returned, and re-reads the backend.

The second is the anti-regression guard for the opposite direction: an implementation that only
closes the dialog without re-reading would trap the operator permanently in "no projects", which
is the failure shape on that side.

**Full suite: 280 passed / 0 errors, against a baseline of 278 passed / 14 errors.** Both numbers
I reproduced myself. So: two more tests, fourteen fewer errors, and the 14 were pre-existing.

## 3. The mock consolidation — the reason the error count dropped

Her explanation is precise and I verified it. The file had two mocks that disagreed:

- `vi.mock('@tauri-apps/api/core')` answered from the `CANNED` table;
- `window.__TAURI_INTERNALS__.invoke` threw `unmocked Tauri invoke` for **every** command.

`isTauri()` takes the internals path, and `App.tsx:160/166`'s `onSupervisorDetached` /
`onSupervisorReembedded` subscriptions have **no `.catch()`** — so each mount produced two
unhandled rejections. Seven existing tests × 2 = the 14. Her two new tests each mount once, which
is why her first run showed 18.

She unified both mocks onto one `answerCommand` — which the file's own header comment already
claimed was "one source of truth for the backend" but the implementation did not honour — and
added the event plugin's internals.

**This was the right call and I want to be explicit about why, because "the errors were already
there" is the argument that usually wins and is usually wrong.** A monotonically growing error
count means the next person cannot tell whether they broke something. Zeroing it costs one mock
unification and touches no product code. Her note that she did **not** add `.catch()` to the
product to make the number look better is also right — that is a separate defect and she escalated
it instead of fixing it quietly.

## 4. Her second catch of the window-period pattern — reproduced, and her claim is exact

This is the part that makes the delivery worth a PASS rather than a pass-with-notes.

**Mutation M-D** is `hydrateFromBackend()` followed *by* `setActiveProject('p-1')`. On her first
test version it was **fully green**, because `hydrateFromBackend` awaits the backend list and the
`nextActive` logic then notices `'p-1'` is not among the returned projects and corrects to
`projects[0]` — **the final value looks right.**

She added a trace assertion instead of a value assertion:

```ts
expect(readProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
```

**I reproduced both halves.** With M-D applied and the trace assertion present: **RED 1**, with
`expected [ 'p-1', 'seatloom', 'seatloom' ] to not include 'p-1'`. With the trace assertion
removed and M-D still applied: **9 passed** — a false green, exactly as she said.

So her claim is exact: the final-value assertion cannot see a window-period error, and the fix is
to assert the *trace* (which project ids the backend was asked to read) rather than the endpoint.
For `InitDialog` the window-period cost is real — that scoped read goes looking for a project that
does not exist.

**This is her second encounter with the same shape** (the earlier one was a
`loadedUIProjectId` gate on 10-22), and she recorded it in the same delivery rather than letting
it pass as a one-off. A pattern caught twice by the same person, with the generalisation written
down, is worth more than a clean run.

## 5. Mutations — all kill, and her counts are accurate

| # | Mutation | Her claim | My measurement |
|---|---|---|---|
| M6 | the exact old line restored (`setActiveProject('p-1')`) | RED 2 | **RED 2** — and the 7 pre-existing tests stay green |
| M-A | close only, no re-read | RED 1 | **RED 2** |
| M-B | `setActiveProjectId(null)`, no backend read | RED 1 | **RED 2** |
| M-C | `setActiveProject('p-2')`, another seed id | RED 2 | **RED 2** |
| M-D | re-read **then** `setActiveProject('p-1')` | RED 1 after the trace fix | **RED 1** |
| V1 | the restore effect loses its `backendHydrated` gate | RED 6 | **RED 7** |
| V2 | `useDataStore`'s launch default back to `'p-1'` | RED 1 | **RED 1** |

**Two of her counts are slightly low, both in the safe direction.** M-A and M-B each kill **2**
tests, not 1 — the "invents nothing and records nothing" case dies as well as the "lands on a real
one" case, because a close-only or null-only handler also fails to re-read the backend. And V1
kills **7**, not 6, because her two new tests are also covered by that guard. **Both discrepancies
mean her tests are stronger than she claimed**, which is the direction that does not matter. I am
recording them because a verification that silently rounds a kill count down in the unsafe
direction would be the thing to catch, and these are the opposite.

**V1 and V2 are the checks that matter most here and she ran them deliberately.** Adding tests and
rewriting mocks is exactly how existing guards get quietly weakened, so re-running the two
pre-existing guards under mutation is the right discipline. Both still bite.

## 6. Source is untouched — verified

The commit is `App.restartRestore.test.tsx` (178 lines changed) and the delivery note. **Zero
changes to `App.tsx`, `useDataStore.ts`, or any product file.** I confirmed by diffing the commit's
file list. Her criterion "don't change the implementation to make M6 go red" is satisfied in the
strongest sense.

## 7. Gates — every figure exact

- Her file: **9 passed** (7 existing + 2 new).
- Full UI suite: **280 passed / 23 files / 0 errors**. Baseline: **278 passed / 14 errors**.
  Both reproduced.
- `tsc --noEmit` clean.
- **eslint: 143 problems on the whole `src` tree, and 0 on her file before AND after** — she added
  no lint.
- **prettier**: her file clean.
- `vite build` succeeds.

## 8. What I did not do

- Did **not** add the `.catch()` to the two subscriptions at `App.tsx:160/166`. Her escalation is
  correct and I am re-escalating it: `supervisorWindowStatus()` has one at line 156, these two do
  not, and a real `listen()` failure in production is an unhandled rejection rather than a sentence
  on screen. That is a product change and needs its own dispatch.
- Did **not** change her kill counts in the delivery note. M-A, M-B and V1 are each one higher
  than she wrote, in the safe direction.
- Did **not** merge. `fad0515` is not on `track`.
- Did **not** touch Mira's worktree beyond a detached worktree at `fad0515`, removed afterwards;
  her `git status` is empty.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/init fad0515
ln -s <mira>/ui/node_modules /private/tmp/init/ui/node_modules
cd /private/tmp/init/ui

./node_modules/.bin/vitest run src/App.restartRestore.test.tsx   # 9 passed
./node_modules/.bin/vitest run                                    # 280 passed / 0 errors
./node_modules/.bin/tsc --noEmit                                  # clean
./node_modules/.bin/eslint src                                    # 143 (same as parent)
./node_modules/.bin/eslint src/App.restartRestore.test.tsx        # 0 problems
./node_modules/.bin/prettier --check src/App.restartRestore.test.tsx
./node_modules/.bin/vite build                                    # ok

# THE BASELINE SHE IMPROVED:
git checkout fad0515~1 -- ui/src && ./node_modules/.bin/vitest run
#   -> 278 passed, 14 errors   (7 tests x 2 unhandled rejections from the
#                               un-caught supervisor subscriptions)

# SOURCE UNTOUCHED:
git show --stat fad0515 | cat    # one test file + the delivery note. Nothing else.

# MUTATIONS (each reverted, tree verified identical afterwards):
#   the exact old line setActiveProject('p-1')     -> RED 2   (7 existing stay green)
#   close only, no re-read                         -> RED 2   [she said 1]
#   setActiveProjectId(null), no read              -> RED 2   [she said 1]
#   setActiveProject('p-2')                        -> RED 2
#   re-read THEN setActiveProject('p-1')           -> RED 1   (the trace assertion)
#   restore effect loses its backendHydrated gate  -> RED 7   [she said 6]
#   useDataStore launch default back to 'p-1'      -> RED 1

# HER M-D FALSE GREEN, REPRODUCED:
#   with the trace assertion expect(readProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT)
#   removed and M-D still applied  ->  9 passed. The final value is correct because
#   hydrateFromBackend's nextActive logic corrects it; the trace shows it briefly
#   landed on 'p-1' and the backend was asked to read it.

# THE ESCALATION SHE RAISED AND I AM RE-RAISING:
sed -n '155,172p' src/App.tsx
#   supervisorWindowStatus() has .catch(() => {}) at line 156;
#   onSupervisorDetached / onSupervisorReembedded have none.
```

## Verdict event

`ev-usP7dX7geulRhaDy1eXwD`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-B4Y5oAhMQwtXm5Vva1fw` in the payload.

## Files I did not change

No source file. Seven mutations were applied and each reverted, with the tree confirmed
byte-identical to `fad0515` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
