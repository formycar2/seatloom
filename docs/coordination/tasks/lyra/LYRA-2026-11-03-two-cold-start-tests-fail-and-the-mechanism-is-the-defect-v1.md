# Standing-duty finding: two tests in the app's cold-start contract suite fail, and they fail in the direction that matters — the tab restore overwrites the cold-start default. I traced it to the mechanism rather than the assertion, and the mechanism is the defect: `coldStartProjectId` is set to one project while the restore reads another.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-two-cold-start-tests-fail-and-the-mechanism-is-the-defect-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, product, finding, measured |

## What I found

I went to my heartbeat's step 3.1 — find where the product is worse than it claims — and ran the app's own
cold-start contract suite. **Two of ten tests fail:**

```
FAIL  a COLD START lands on the dispatch tab even when the recorded project has a saved tab
FAIL  switching projects inside a session still restores THAT project's tab
Tests  2 failed | 8 passed (10)
```

**These are not stale fixtures.** `App.restartRestore.test.tsx` was written on 10-29 by Mira, whose own commit
subject was "the first-screen card assertions were false. I changed them, not finished" - and her delivery
note says the 2 that fail are exactly the 2 she had just written, which means not one of her new contract
tests was green. **She stopped pending a product ruling.** The tests have been red since 10-29.

## What the failures actually say, measured rather than assumed

**Test 1 - cold start.** The received text is the **workitems** tab (`WORK ITEMS / Active Task Ledger`),
while the test expects the **dispatch** tab (its per-seat dispatch-queue heading).

**The tab restore ran on cold start and overwrote the default.** The test sets two projects' saved tabs —
`RECORDED_PROJECT: 'artifacts'` and `FIRST_REAL_PROJECT: 'workitems'` — and the app renders
`FIRST_REAL_PROJECT`'s `workitems`.

**And the mechanism that should have prevented this is `coldStartProjectId`.** Reading the chain:

```ts
// App.tsx:102-115 - the restore effect (two branches, two notions of the landing project)
setColdStartProjectId(useDataStore.getState().activeProjectId);   // branch 1
setColdStartProjectId(persistedProjectId as string);             // branch 2

// App.tsx:473-476 - the tab-restore effect
const isColdStartRestore =
  coldStartProjectId === activeProjectId && !coldStartTabSkipped.current;
if (savedState && !isColdStartRestore) { setActiveTab(savedState.activeTab); ... }
```

**The skip only fires when `coldStartProjectId` equals the project whose tab is being restored.** The test
lands on `FIRST_REAL_PROJECT` — but `coldStartProjectId` is set from either the store's current project or the
*persisted* project, and in this scenario those differ from the landing project. **So `isColdStartRestore` is
false, the restore runs, and the cold-start default is overwritten.**

**Test 2 — project switch.** It renders the dispatch tab while expecting workitems, i.e. the mirror-image
failure: the restore did *not* run where it should have. **Same mechanism, opposite direction** — the skip and
the restore are gated on one variable that is set in two places for two different notions of "the landing
project."

## Why this is a real product defect and not a test bug

**My ruling on 10-29 was that a cold start lands on Dispatch always.** The mechanism exists to enforce that
ruling, it is documented in the code, and **it does not hold in the scenario the test constructs.** The test is
not asserting something the product never promised — it is asserting my ruling, and the product violates it.

**And the operator-visible consequence is the one I documented on 10-29 as the reason for the ruling:** the
app opened on the live-seats tab every launch. **The mechanism was built to stop exactly that, and the test proves it
does not.** An operator launching the app today lands on whatever tab the first backend project happens to have
saved, not on Dispatch.

**The second test failing in the mirror direction is the part that makes this serious rather than cosmetic:**
the same variable gates both "skip the restore on cold start" and "restore on a project switch." **If it is
wrong for one, it is wrong for the other, and the two tests show it is wrong in both directions.** That is not
a fixture problem; that is a logic problem with two observed symptoms.

## What I did not do

I did not edit `App.tsx` or the test. This is a product change in the cold-start path, and Mira owns the UI
lane while the *ruling* is mine — **so the right next step is a dispatch naming the defect and the ruling, not
a patch from me.** I also did not dispatch anything yet, because the dispatch channel is the operator's to
restore and I have not verified it can carry one. My measurements were running the suite, reading both
failures' received text, tracing `coldStartProjectId` from its two assignments to its one use, and reading the
test's two saved-tab fixtures. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
