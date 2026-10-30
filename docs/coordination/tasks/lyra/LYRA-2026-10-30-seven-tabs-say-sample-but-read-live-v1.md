# Product finding: seven tabs read live PostgreSQL and tell the operator they are "SAMPLE DATA — seed fixtures"

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-seven-tabs-say-sample-but-read-live-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, product-finding, blocking, provenance, ui |

## Why this round looks different

My heartbeat's item 3.1 is *find where the product is worse than it claims*. I spent the last many rounds on
delivery-chain and instrument problems, which are real but are item 3.2-adjacent. **This round I did the
product-audit duty**, and the finding is the one I first reported on 2026-10-29 — **still live, still
unfixed, and now measured more precisely.**

## The finding

**Seven of the ten nav tabs read live PostgreSQL for real, and every one of them wears a badge reading
`SAMPLE DATA — not yet from live capture`, whose own detail text says the rows are seed fixtures.**

The nav, from `ui/src/components/NavRail.tsx`:

```
live-seats  dashboard  inbox  workitems  timeline  dispatches  seats  artifacts  playbook  sessions
```

`ui/src/lib/provenance.ts` classifies three:

```ts
export const LIVE_TAB_BACKING: Record<string, 'tmux' | 'database'> = {
  'live-seats': 'tmux',
  sessions: 'database',
  dispatches: 'database',
};
export const LIVE_TABS = new Set(['live-seats', 'sessions', 'dispatches']);
export const LEDGER_TABS = new Set(['timeline']);
```

**Unclassified: `dashboard`, `inbox`, `workitems`, `seats`, `artifacts`, `playbook` — six — plus `timeline`
is ledger, so seven tabs fall through to the `seed` branch.**

`provenanceForTab` then does, verbatim:

```ts
if (LIVE_TAB_BACKING[tab] === 'database') return backendHydrated ? 'live' : 'fixture';
if (LIVE_TABS.has(tab)) return 'live';
if (LEDGER_TABS.has(tab)) return backendHydrated ? 'ledger' : 'fixture';
return backendHydrated ? 'seed' : 'fixture';      <-- the seven land here
```

## The proof that they are live, not seed

**The read path.** All six views read through the same store, and `hydrateProjectData`
(`ui/src/stores/useDataStore.ts:1908-1914`) calls:

```ts
api.listSeats(projectId),
api.listWorkitems(projectId),
api.listHandoffs(projectId),
api.listArtifacts(),
api.listSessions(projectId),
api.listEvents(projectId, 400),
api.listDocuments({ projectId }),
```

**Every one is a real Postgres read.** And what those reads return, measured on live just now:

| | live rows |
|---|---|
| `seats` | **10** |
| `workitems` | **7** |
| `artifacts` | **15** |
| `canonical_events` | **2942** |

So the Seats tab shows 10 real seats, the Work Items tab shows 7 real workitems, the Artifacts tab shows 15
real artifacts. **None of that is a hand-written baseline.**

## What the operator actually reads

The `seed` branch, verbatim:

```ts
seed: {
  badge: 'SAMPLE',
  headline: SAMPLE_HEADLINE,                       // 'SAMPLE DATA — not yet from live capture'
  detail:
    'These rows are read from PostgreSQL for real, but they are seed fixtures — ' +
    'a hand-written baseline, not captured activity. Nothing here reflects what the seats ' +
    'are doing now. The live mirror is under the Live Seats tab.',
```

**Three statements, and the first two are true while the third is false.** The rows *are* read from
PostgreSQL. They are *not* seed fixtures — they are 10 seats, 7 workitems, 15 artifacts and 2942 events of
captured activity. **And the last sentence actively misdirects him:** it tells him to look under the Live Seats tab for
the live mirror, when the data he is looking at *is* live.

**This is worse than a wrong badge.** A wrong badge costs a glance. **A badge whose explanation is confidently
false costs him the conclusion that the surface is empty when it is not** — and he has been making merge
decisions all day off surfaces that tell him nothing is real.

## What changed since I first reported it, and what did not

**Changed, and it is a real improvement.** The classification is now keyed on *what backs the tab*
(`LIVE_TAB_BACKING`) rather than on a tab-name list consulted before `backendHydrated`, and a failed
hydration now correctly falls to `fixture` instead of wearing a green LIVE badge. The comment records the
old bug. **Someone fixed the half that was lying in the dangerous direction.**

**Unchanged:** the seven tabs that read live data are still not in any set, so they still fall through to
`seed`. **The fix stopped exactly at the boundary where the lie became harmless-looking instead of where it
became true.**

## The ask, and it is small

**Two lines in `provenance.ts`:**

```ts
export const LIVE_TAB_BACKING: Record<string, 'tmux' | 'database'> = {
  'live-seats': 'tmux',
  sessions: 'database',
  dispatches: 'database',
  dashboard: 'database',      <-- add
  inbox: 'database',          <-- add
  workitems: 'database',      <-- add
  seats: 'database',          <-- add
  artifacts: 'database',      <-- add
  playbook: 'database',       <-- add
};
```

**And then correct the `seed` branch's detail text**, because it currently describes a surface that does not
exist. If no tab lands on `seed` after the change, the branch is dead code and should say so rather than
describe a fixture read.

This is Mira's surface, so this is an ask and not an edit.

## What I did not do

I did not edit `provenance.ts`. I did not dispatch anyone — this is a report, and the fix is two lines that
the owning seat can make in minutes once told. I did not change any card.
