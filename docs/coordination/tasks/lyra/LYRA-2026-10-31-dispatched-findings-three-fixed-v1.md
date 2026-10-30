# Standing-duty closure: dispatching the findings instead of writing them worked — three of four fixed in 15 minutes, and I mutation-tested each

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-dispatched-findings-three-fixed-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, closure, verification, mutation, dispatch |

## The result

Last round I stopped writing findings as documents — after measuring that they reach nobody — and
dispatched them instead. **`disp-OQyilLqRjR1fDSWsWE-5n` went `done` in 15 minutes**, in four commits, with
three of the four findings fixed and the fourth correctly identified as not a code change.

| Finding | State | Commits |
|---|---|---|
| 1 — the SAMPLE-badge lie a test enforces | **fixed** | `da20424` |
| 2 — his three cards are invisible | **fixed** | `e700d34` |
| 4 — his active project does not exist | **fixed** | `a9a00f7` |
| 3 — the installed app predates the fix | **correctly identified as process, not code** | `8c71c88` |

**375/375 across 31 files**, worktree clean.

## Finding 1, and I mutation-tested the fix

`LIVE_TAB_BACKING` now declares all eight tabs, and the reasoning is written into the source rather than left
implicit:

```ts
// The set below used to be exactly [sessions, dispatches], and a test asserted
// that exact set — which is how the SAMPLE badge stayed on seven surfaces that
// read live PostgreSQL. The premise was that a tab fed from the zustand store is
// somehow different from a database-backed one. It is not: hydrateProjectData
// fills the store from api.listSeats / listWorkitems / listHandoffs /
// listArtifacts / listSessions / listEvents / listDocuments, every one a real
// Postgres read, and it builds inboxItems from that data too. There is no
// store-backed tier. The distinction the badge logic was built on does not exist.
//
// Measured against the live database: 10 seats, 7 workitems, 15 artifacts,
// 2946 events — all behind tabs that were telling the operator "SAMPLE DATA —
```

**And the test that used to enforce the lie is gone, replaced by its negation**:
`'a tab with NO database backing is unaffected by hydration — there is no store-backed tier'`.

**I mutation-tested it rather than accepting the green.** Restored the old three-tab set:

```
× every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name
× a tab with NO database backing is unaffected by hydration — there is no store-backed tier
  Tests  2 failed | 11 passed (13)
```

**The lie is now caught by two tests.** Last round the same mutation produced *three* failures because the
suite defended the lie; now it produces two because the suite defends the truth. **That inversion is the
whole fix, and it is the first time this week a check has flipped from defending a falsehood to defending a
fact.** Restored, worktree clean, 13/13.

## Finding 2, fixed with the scoping bug named in the code

A pending-acceptance count now renders on the nav, and the comment states exactly why it reads every
project:

```ts
// `listAcceptances` takes a projectId and the backend filters by it — so
// one call scoped to the active project is exactly the bug this badge
// exists to fix: measured, every acceptance belongs to seatloom while the
// operator's active project had none, so the scoped read returned an
// empty "no pending cards" while three waited elsewhere.
//
// Read EVERY project and take the union. That is the honest count of
// cards waiting for him, and it is the only thing a badge can show
// without lying. The per-card scoping bug in the panel itself is
// reported, not patched here: the panel's read is Nimbus's fence.
```

**And she declined to patch a surface that is not hers, naming whose it is.** That is the boundary discipline
working in both directions — she took her three findings and left the fourth file alone.

## Finding 4, fixed with both ids named

`App.tsx` now drops a persisted `activeProjectId` that names no existing project, falls back to a real one,
and **names both ids to the operator** — the comment explains why naming matters: *"The project you were
last in no longer exists" without naming it leaves the operator unable to tell* which one. **A silent
fallback would have fixed the data and hidden the event**, which is what I asked her not to do.

## Finding 3, and she owned it rather than deflecting

Her delivery note, verbatim in substance:

> **This was my omission at card-open time, not ignorance of the rule.** The two cards
> `acc-disp-faEWBt8kSKgv-6pJs_Rqw` and `acc-disp-E06O6IAaWefF-BOW-CNJd` did not run the script. The reason
> is measured: the installed app is ~43 hours older than the fix, so the card's own step 1 — quit and
> reopen, tab must be Dispatch — **fails on the installed app**, because the write effect is still there.
> Both cards are pending now; I should have prepared the build first. Reported to Lyra/Aegis, not hidden.

**That is the right shape.** She named the defect, named her part in it, named what she did not do, and did
not dress it as blocked.

## What this proves about the delivery decision

**Three findings fixed in 15 minutes that had been sitting unreachable for five rounds.** The documents were
correct, committed, and pushed to two remotes the whole time. **The dispatch was the only thing that
delivered them.**

I am recording this as the operative rule for my own output, not as a one-off success:

> **A finding is delivered when the seat that can act on it has it in front of them. For a governed seat
> that means a dispatch. A document on a branch that is not on track is a note to myself.**

## What is still open, and it is now short

1. **The panel's acceptance scoping** — the badge works around it correctly, but the panel itself still
   shows "nothing to accept" when the active project has none. That is Nimbus's fence and Mira named it
   rather than crossing it.
2. **The build for the two cards** — hers to run at card-open time, and she has owned it.
3. **The three cards themselves** — still pending, still invisible to him until findings 1, 2 and 4 land on
   track and he is given a build.
4. **`land-standing-duty-docs.sh`** — still unrun.

## What I did not do

I did not edit her tree beyond the one mutation, restored from backup with the worktree verified clean and
13/13 green. I did not merge anything, grant any card, or touch the operator's session state.
