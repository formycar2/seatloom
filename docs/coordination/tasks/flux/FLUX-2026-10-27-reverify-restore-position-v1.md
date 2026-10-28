# Flux re-verdict — Mira's restart-restore fix, judged by mutation

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-reverify-restore-position-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-flux-reverify-restore-position-v1 |
| tags | reverification, mutation-testing, seat-flux |

- **Re-verifying dispatch**: `disp-IgwyJqKM_ef9T3j63sD-i` (Aegis → Flux)
- **Subject dispatch**: `disp-DFeH6Ut_aNfzIbuuR21V5` (Mira's state-memory + project-desync +
  legacy-fake-data work)
- **Superseded verdict**: `ev-mkwI3idecZKlZBGXOoC31` (`fail`, 2026-10-21, mine)
- **Fix commit**: `0e71598`, now on `packet/mira/knowledge-panel` and merged to
  `track/infra-foundation` via `7d738fa`
- **Verdict event**: **`ev-mAztaqFFzPSAQ-Fm3asBD`** — the pre-declared id

## Verdict: PASS

**The item-1 defect my `fail` named is genuinely fixed, and I did not take Mira's
246/246 on trust.** I reconstructed the exact pre-fix source and ran her new test suite
against it: **all 7 tests go red.** Then I mutated each load-bearing element of the fix
one at a time. The order contract is real, and the tests that pin it bite.

One **finding**: one of the three fixes the commit claims carries **no test coverage**
(F1, below). It is a coverage gap, not a live defect, and it is the third same-root
defect the dispatch did not name.

---

## 1. What my original fail actually said (read from the ledger, not from Aegis)

`ev-mkwI3idecZKlZBGXOoC31`, `verdict: fail`, one item:

> *"UseDataStore's initial activeProjectId is 'p-1' (the first row of
> SEED_PROJECTS_FOR_TESTS — a project in no database); App.tsx's write effect
> (lines 260-271) runs first and persists that value, overwriting the recorded project
> BEFORE the restore effect (lines 101-116) reads persistedProjectId. Observed: recorded
> 'ql_delivery' -> appStore ql_delivery -> p-1 -> p-1 -> seatloom; the launch lands on
> the default and the record left on disk is 'seatloom', so the next launch starts there
> again — the position is lost permanently, not once."*

Two root causes, then: the **launch default** was a test-only seed id, and the **write
effect won the race** against the restore.

## 2. The fix is real at source

| Root cause | Fix |
|---|---|
| `activeProjectId: 'p-1'` at launch | `activeProjectId: null` (`useDataStore.ts:1614`) — a real launch no longer opens on a project in no database |
| write effect runs first | `const [persistedProjectId] = useState(() => useAppStore.getState().activeProjectId)` — a snapshot, so no write of this component's can change the restore target |
| restore reads before the backend lands | restore gated on `backendHydrated`; a deleted project falls back to the first real backend project or `null` |
| previous project's saved tab overwrites the destination's | write effect gated on `restoredPosition && loadedUIProjectId === activeProjectId` |
| third same-root defect | `InitDialog`'s "initialize demo repo" no longer calls `setActiveProject('p-1')`; it re-reads the backend |

## 3. Mutation verification — the tests bite

**The decisive control first.** I checked out `0e71598~1`'s `App.tsx` **and**
`useDataStore.ts` verbatim (the exact pre-fix state) and ran Mira's new suite against it:

```
  × a launch does not begin on a test-only project id > opens useDataStore on no SEED_PROJECTS_FOR_TESTS id
  × restart lands back in the recorded project… > restores the recorded project after the backend read, not before it
  × restart lands back in the recorded project… > never persists the launch default while the real position is being restored
  × restart lands back in the recorded project… > lands on the recorded project's saved tab, not another project's
  × restart lands back in the recorded project… > does not restore a project the backend did not return
  × restart lands back in the recorded project… > a second launch lands back on the same project AND the same tab
  × restart lands back in the recorded project… > starts with no active project when the backend has none at all
  Tests  7 failed (7)
```

And the specific assertion reproduced my original measurement exactly:

```
AssertionError: expected 'seatloom' to be 'ql_delivery'
```

`expected 'seatloom' to be 'ql_delivery'` — the record on disk is `seatloom`, which is
precisely the `recorded 'ql_delivery' → … → seatloom` chain I measured on 10-21. The test
suite is pinned to the defect I named, not merely to the current code.

**Then each element, mutated one at a time** (tree confirmed byte-identical to Mira's HEAD
after every mutation):

| # | Mutation | Result |
|---|---|---|
| M2 | restore effect loses its `backendHydrated` gate | **RED, 6 of 7** |
| M3 | write effect loses both gates | **RED, 4 of 7** |
| M4 | write effect loses **only** the `loadedUIProjectId` gate | **RED, 1 of 7** — the saved-tab test |
| M5 | launch default back to `'p-1'` | **RED, 1 of 7** — the seed-id guard |
| M1 | snapshot → live subscription (`useAppStore((s) => s.activeProjectId)`) | **green — equivalent mutation** |
| M6 | `InitDialog` reverted to `setActiveProject('p-1')` | **green — see finding F1** |

M4 is the interesting one: dropping only the tab-acknowledgement gate kills exactly one
test, and it is the *second launch* test — the one that asserts the written tab set for the
project is `{'artifacts'}` and nothing else. That is the "invisible in the final state,
fatal in the window" case the fix was written for, and it is covered.

M1 is an honest equivalent mutation: with `restoredPosition` and `loadedUIProjectId` in
place, a live subscription and a one-shot snapshot behave identically, because the write
effect cannot run until restore has finished and the snapshot is only read inside the
restore effect. I record it as equivalent rather than as a miss.

## 4. The rest of the item-1 claim, independently

- **Mira's 246/246 is confirmed, at the right commit.** At `0e71598` I measure
  **246 passed (20 files)**. At her branch HEAD (`7ff9f0c`) I measure **277 passed (23
  files)**; the +31 is entirely later commits on her branch — `useGlobalShortcuts.test.ts`
  (+278), `SessionTerminal.geometry.test.tsx` (+475), `LiveSeatsView.test.tsx` (+86),
  `OperatorDecisionSection.test.tsx` (+134). So the number is real and current, not
  inflated. `tsc --noEmit` clean at HEAD.
- **14 unhandled errors are pre-existing** and unchanged (`App.restartRestore.test.tsx`'s
  deliberate unmocked Tauri invoke — the same 14 Mira flagged and declined to fix as out
  of scope, which I agree with).
- **The fix is deployed.** `0e71598` merged to track at `7d738fa` (2026-10-26 16:25). The
  running `/Applications/SeatLoom.app` binary is from **2026-10-27 00:03** and its
  process started 00:03. I decompressed the blobs embedded in that running executable and
  read the data-store's initial state: `activeProjectId:null`, and `activeProjectId:"p-1"`
  appears **nowhere**. The tilde-fix string is present too. So unlike the last dispatch,
  this one is not blocked on a rebuild.
- **No criteria were changed.** The dispatch's own acceptance criterion — "close the app,
  reopen, you should be back in project A" — is what the second-launch test asserts, and it is
  what M2/M3/M4 kill.

## 5. Finding F1 — one of the three fixes has no test coverage (P3)

The commit claims three fixes to the same root cause. Two are pinned by tests. The third
is not.

| Fix | Covered? |
|---|---|
| launch default `'p-1'` → `null` | **yes** — M5 kills a test |
| write effect ordering gates | **yes** — M2/M3/M4 kill four |
| `InitDialog`'s `setActiveProject('p-1')` → `hydrateFromBackend()` | **no** — M6 stays fully green |

M6 restored the exact old line and **all 7 tests passed.** I then wrote my own probe
(scratch, deleted) that mounts the real App with an empty backend project list, clicks the
dialog's button, and asserts nothing is planted. It **passes with the fix and fails with
M6**, which proves the path is reachable and the gap is real rather than unreachable code.

This is a coverage gap, not a live defect — the code is correct and the running app
carries the fix. But it is the same shape as the M4 self-finding Mira recorded in her own
delivery note ("my first IME test didn't catch the mutation, so I added the assertion"),
and it is worth naming because the claim in the commit message is stronger than the
evidence: "also fixed the third same-root instance (not named in the dispatch)" is true of the
code and unverified by the suite.

## 6. Implementation vs criterion — no ambiguity to report

The dispatch asked me to say so when I cannot tell whether the implementation or the
criterion is wrong. **Here I can, and it is the implementation that was wrong.** The
criterion ("close the app, reopen, you should be back in project A") is the operator's
own requirement and is exactly what the second-launch test asserts. My 10-21 `fail` was
against the implementation; the implementation has since changed and now satisfies the
criterion by measurement. Nothing about the criterion was revised, and I did not revise it.

## Reproduction

```bash
# worktree at the fix, isolated from mira's tree
git -C <mira> worktree add --detach /private/tmp/rr HEAD
ln -s <mira>/ui/node_modules /private/tmp/rr/ui/node_modules
cd /private/tmp/rr/ui

# 1. her suite, green
./node_modules/.bin/vitest run src/App.restartRestore.test.tsx      # 7 passed

# 2. THE CONTROL — exact pre-fix source, her suite against it
git show 0e71598~1:ui/src/App.tsx          > /tmp/pre-App.tsx
git show 0e71598~1:ui/src/stores/useDataStore.ts > /tmp/pre-store.ts
cp ui/src/App.tsx /tmp/keep-App.tsx && cp ui/src/stores/useDataStore.ts /tmp/keep-store.ts
cp /tmp/pre-App.tsx ui/src/App.tsx && cp /tmp/pre-store.ts ui/src/stores/useDataStore.ts
./node_modules/.bin/vitest run src/App.restartRestore.test.tsx      # 7 FAILED
#   AssertionError: expected 'seatloom' to be 'ql_delivery'   <- the original defect
cp /tmp/keep-App.tsx ui/src/App.tsx && cp /tmp/keep-store.ts ui/src/stores/useDataStore.ts

# 3. mutations (each reverted, tree verified identical afterwards)
#    M2 drop backendHydrated gate          -> 6 of 7 red
#    M3 drop both write gates              -> 4 of 7 red
#    M4 drop only loadedUIProjectId gate   -> 1 red (the second-launch tab test)
#    M5 launch default back to 'p-1'       -> 1 red (the seed-id guard)
#    M1 snapshot -> live subscription      -> green (equivalent)
#    M6 InitDialog back to setActiveProject('p-1') -> green  <-- F1, uncovered

# 4. the 246 claim, at the commit that made it
git -C <mira> worktree add --detach /private/tmp/rr2 0e71598
cd /private/tmp/rr2/ui && ./node_modules/.bin/vitest run          # 246 passed (20 files)
#    and at her branch HEAD: 277 passed (23 files); +31 from later commits

# 5. it is deployed
git merge-base --is-ancestor 0e71598 track/infra-foundation && echo "on track"
stat -f '%Sm' /Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri
python3 -c "
import glob,os,subprocess,re
binp='/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri'
binary=open(binp,'rb').read(); hits=[]
for d in glob.glob('/Users/…/seatloom/target/**/tauri-codegen-assets',recursive=True):
    for f in glob.glob(os.path.join(d,'*')):
        comp=open(f,'rb').read()
        if comp in binary:
            r=subprocess.run(['brotli','-d','-c',f],capture_output=True)
            if r.returncode==0 and r.stdout: hits.append(r.stdout)
print('launch default is null:', any(re.search(rb'activeProjectId:null,backendHydrated',b) for b in hits))
print('the seed id is absent :', not any(re.search(rb'activeProjectId:\"p-1\"',b) for b in hits))"
```

## Verdict event

`ev-mAztaqFFzPSAQ-Fm3asBD`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-DFeH6Ut_aNfzIbuuR21V5` in the payload, `reissue_of:
ev-mkwI3idecZKlZBGXOoC31` naming the fail it replaces.

## Files I did not change

No source file, no migration, no live-database write. Two worktrees were created at
`packet/mira/knowledge-panel` HEAD and at `0e71598`, both removed afterwards; `git status`
in mira's worktree is empty. Six mutations were applied and each reverted, with the tree
confirmed byte-identical to the ref after each. My one probe file was written under
`/private/tmp/` and deleted.
