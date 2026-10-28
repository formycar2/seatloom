# Standing-duty finding: six of ten nav surfaces read live PostgreSQL and wear a badge that says they do not

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-provenance-badge-six-tabs-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, product-truth, ui, provenance, blocking |

## The defect

`ui/src/lib/provenance.ts` decides what badge a tab wears. Three tables drive it:

```ts
export const LIVE_TAB_BACKING: Readonly<Record<string, 'tmux' | 'database'>> = {
  'live-seats': 'tmux',
  sessions: 'database',
  dispatches: 'database',
};
export const LIVE_TABS   = new Set(['live-seats', 'sessions', 'dispatches']);
export const LEDGER_TABS = new Set(['timeline']);
```

`provenanceForTab` falls through to `return backendHydrated ? 'seed' : 'fixture'` for anything not in
those three tables, and `seed` renders:

```
badge:    'SAMPLE'
headline: 'SAMPLE DATA — not yet from live capture'
```

**Six of the ten nav items are in none of the three tables, and every one of them reads live
PostgreSQL through a registered Tauri command:**

| Nav item | What it actually reads | Badge it wears |
|---|---|---|
| `artifacts` | `api.listArtifacts()` → `cmd_list_artifacts` | **SAMPLE** |
| `workitems` | `api.listWorkitems(projectId)` → `cmd_list_workitems` | **SAMPLE** |
| `seats` | `api.listSeats()` / `api.listAllSeats()` → `cmd_list_seats` | **SAMPLE** |
| `inbox` | the backend-hydrated store | **SAMPLE** |
| `playbook` | the backend-hydrated store (`useDataStore`) | **SAMPLE** |
| `dashboard` | `useDataStore` → `projectData`, `projects`, `backendHydrated` | **SAMPLE** |

Against the three that are classified: `dispatches` → `live`, `sessions` → `live`,
`live-seats` → `live` (discovery), `timeline` → `ledger`.

## How I measured it, so it is not a reading of a list

**1. Which tabs are rendered and reachable.** `App.tsx` has a `case` for each, and `NavRail.tsx` lists
all ten with `id:`. Not dead surfaces.

**2. What each reads.** `useDataStore.ts` hydrates from ten real API reads:

```
api.listAllSeats  api.listArtifacts  api.listDocuments  api.listEvents
api.listHandoffs  api.listProjects   api.listRoleBindings  api.listSeats
api.listSessions  api.listWorkitems
```

The four views I could check directly (`ArtifactsView`, `InboxView`, `WorkItemsView`) make no `api.*`
call themselves — they read the hydrated store, which is *more* live, not less.

**3. The commands exist.** `cmd_list_artifacts`, `cmd_list_workitems`, `cmd_list_documents` are all
registered in `src-tauri/src/`.

**4. The badge is displayed, not just computed.** `NavRail.tsx:151` and `:160` pass
`provenanceForTab(item.id, …)` for every nav item, and `App.tsx:790` passes it for the active tab.

So the operator opens the app, and the navigation rail tells him that six surfaces contain sample data
while he is looking at rows that came out of PostgreSQL.

## Why this is the defect the operator named, not a cosmetic one

On 2026-10-16 he said the test is *"does clicking it do what it says."* A badge is a claim about the
data, and this one is false — **in the direction that makes the product look worse than it is**, which
is the safe direction but still false. It also destroys the badge's value as a signal: once six of ten
items are permanently wrong, the four that are right carry no information, because the operator has
learned to ignore the rail.

It is also self-inflicted. `7a1b12e` (10-17) fixed exactly this class of bug for the three tabs it knew
about — its own comment records that the old code returned `live` *before* consulting
`backendHydrated`, so a failed read wore a green badge. The fix was right and **incomplete**: it
replaced a name-list that was too generous with a name-list that is too stingy, and the six tabs that
were never in either list kept saying SAMPLE.

## The fix, and the part that needs a decision

The mechanical part is easy: `artifacts`, `workitems`, `seats`, `inbox`, `playbook`, `dashboard` all
read the backend-hydrated store, so they belong in `LIVE_TAB_BACKING` as `'database'` — which yields
`backendHydrated ? 'live' : 'fixture'`, i.e. they go green when the read succeeded and say FIXTURE when
it did not. That is exactly the semantics `7a1b12e` established for the other three.

**The part that is not mine:** whether `dashboard` and `playbook` are *fully* backend-backed I have not
traced line by line, and the honest version of this fix is a per-tab measurement rather than adding six
names to a list — which is how the list became wrong in the first place. This is Mira's surface. I am
handing her the measurement and the six names, not editing `provenance.ts`.

## What I did not do

I did not edit `ui/src/lib/provenance.ts`. I did not add the six names to `LIVE_TAB_BACKING`. This is
Mira's standing-duty check #1 (every provenance badge is earned) catching a real defect, and the
surface is hers.
