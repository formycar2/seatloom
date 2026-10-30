# Product finding, escalated: the SAMPLE-badge lie is not merely unfixed — a test enforces it, and I proved it in both directions

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-a-test-enforces-the-lie-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, product-finding, blocking, mutation, provenance |

## Why this is a separate document

Last round I reported that seven nav tabs read live PostgreSQL and wear `SAMPLE DATA — seed fixtures`, and
that the fix is two lines in `provenance.ts`. **That was the finding but not the worst part of it.** This
round I found what makes it hard to fix: **a test pins the lie, and I proved it in both directions.**

## The test that enforces it

`ui/src/lib/provenance-honesty.test.ts`, verbatim:

```ts
it('a store-backed tab is unaffected: it still says seed/fixture either way', () => {
  expect(provenanceForTab('inbox', true)).toBe('seed');      <-- hydrated, so "seed"
  expect(provenanceForTab('inbox', false)).toBe('fixture');
});
```

and, one test above it:

```ts
it('every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name', () => {
  for (const tab of LIVE_TABS) expect(LIVE_TAB_BACKING[tab], tab).toBeDefined();
  const dbBacked = Object.entries(LIVE_TAB_BACKING).filter(([, v]) => v === 'database');
  expect(dbBacked.map(([k]) => k).sort()).toEqual(['dispatches', 'sessions']);   <-- exactly two, NAMED
});
```

**The second one enumerates the database-backed set as exactly `['dispatches', 'sessions']`.** So the seven
tabs that read live PostgreSQL are excluded **by name, deliberately, with a test asserting it.**

## The premise is false, and it is checkable

The test calls `inbox` *store-backed* as though that were a different thing from database-backed. **It is
not.** `hydrateProjectData` (`ui/src/stores/useDataStore.ts:1908-1914`) fills the store from:

```ts
api.listSeats(projectId), api.listWorkitems(projectId), api.listHandoffs(projectId),
api.listArtifacts(), api.listSessions(projectId), api.listEvents(projectId, 400),
api.listDocuments({ projectId }),
```

**Every one a real Postgres read.** Live holds 10 seats, 7 workitems, 15 artifacts, 2946 events. So
`inbox` hydrated is not "a store with fixtures in it" — it is a store holding rows that came from
PostgreSQL a moment ago. **The distinction the test is built on does not exist.**

## Proven in both directions, which is the only reason I am escalating rather than reporting

**Direction 1 — the lie can be made worse and the test catches it.** Removed `sessions` and `dispatches`
from `LIVE_TAB_BACKING`:

```
× every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name
  Tests  2 failed | 11 passed (13)
```

**Direction 2 — the fix can be applied and the test refuses it.** Added `dashboard`, `inbox`, `workitems`,
`seats`, `artifacts`, `playbook` as `'database'`:

```
× every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name
× a store-backed tab is unaffected: it still says seed/fixture either way
  Tests  3 failed | 10 passed (13)
```

**So the suite is green today and turns red the moment anyone tells the truth.** Both mutations restored
from backup, worktree clean, 13/13 green again.

**That is the definition of a check that cannot be satisfied by the correct answer** — and it is the exact
failure my own heartbeat names as worse than a missing check: *a check that can never pass is worse than no
check, because it converts the fix into a regression.*

## Why this is worse than the badge alone

A wrong badge costs the operator a glance. **This costs him a fight**: the seat that fixes it will see three
red tests, and every one of them will read as "you broke the provenance logic" rather than "you made the
badge honest." **The path of least resistance is to revert the fix and keep the green**, which is exactly
what has happened — the principle landed on 10-17 and the enumeration never did.

And the person most likely to hit it is the one who reads tests as truth. **The test says the behaviour is
correct. The database says seven surfaces are live. Both cannot be right, and the test is winning.**

## The fix, and it is three edits not two

```ts
// 1. provenance.ts — declare the six database-backed tabs
  dashboard: 'database', inbox: 'database', workitems: 'database',
  seats: 'database', artifacts: 'database', playbook: 'database',

// 2. provenance-honesty.test.ts — the enumeration must include them
  expect(dbBacked.map(([k]) => k).sort()).toEqual(
    ['artifacts','dashboard','dispatches','inbox','playbook','seats','sessions','workitems']);

// 3. provenance-honesty.test.ts — the 'store-backed' test is testing a distinction that
//    does not exist. inbox hydrated reads Postgres, so it is 'live', not 'seed'.
//    Rewrite it as: a tab with NO database backing is unaffected by hydration.
```

**And then the `seed` branch's detail text**, which currently says the rows are seed fixtures and misdirects
him to the Live Seats tab. If no tab lands on `seed` after the change, that branch is dead code and should
say so rather than describe a fixture read.

This is Mira's surface and Aegis's branch already carries the principle, so this is an ask.

## What I did not do

I did not edit either file — I applied both mutations from a backup and restored them, verifying the
worktree clean and 13/13 green afterwards. I did not dispatch anyone: this is a report, the fix is three
edits, and a dispatch would spend a seat's attention on something that needs no discovery. I did not grant,
cancel, or re-open any card.

## One thing I verified that is good news

**All three pending cards are clickable right now.** I ran `scripts/check-cards-clickable.sh` rather than
assuming: `OK: all 3 pending cards are clickable`, and I verified independently that both branches
(`packet/mira/knowledge-panel`, `packet/nimbus/project-scoped-reads`) contain track and are **0 commits
behind**. The app is running with `SEATLOOM_OPERATOR_REF=human:supervisor` and the correct
`SEATLOOM_REPO_ROOT`. **So nothing is technically blocking his click. He has three working cards and has
not clicked in about five hours**, which is his to do and not something I should chase.
