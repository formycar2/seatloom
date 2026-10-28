# Delivery: V5 honesty fixes — three fabricated values replaced with real data or an explicit "no data"

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-20-honest-surfaces-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-20 |
| version | v1 |
| dispatch | disp-RWEf5ATPvICLkGtF__fEY |
| depends_on | AEGIS-2026-10-20-v5-honest-surfaces-v1 |
| tags | v5, honesty, fabricated-values, delivery, mira |

---

## 0. The one-paragraph version

All three located fabrications are gone: the hardcoded `budgetHealthPercent = 72`,
the render-time-as-sync-time clock, and `GlobalDashboard`'s `MOCK_GLOBAL_SUMMARY`.
None was replaced by another invented number — two now read from the store, and one
now says the backend has no such data and renders an em dash. The full `ui/src` grep
sweep the packet required is in §4.

## 1. The hardcoded budget-health number

`ui/src/components/SupervisionDashboard.tsx` — the constant is deleted.

The backend has no budget or health field today (no token-budget table, no health
column in the schema), so there was nothing real to put in the card. The `72%` is
replaced by an em dash (`value="—"`, `color="done"`), and the hover text now says the
backend has no such data and that this cell shows no number. No placeholder stands
in for a reading.

## 2. Render time presented as the last sync time

Same file. The label changed from "last sync time" to "data last updated", and the
value is a new `SyncFreshness` component (`ui/src/components/SyncFreshness.tsx`)
that separates the two facts:

- **data freshness** — "unknown (not recorded)", plus one line saying the read path
  does not record when a successful sync happened. `hydrateFromBackend` records
  success, not the instant, so the honest answer is unknown, not `Date.now()`.
- **page refresh time** — a ticking clock labelled "page refreshed at <time>", so a
  refresh moment is never again mistaken for data freshness.

When the read path starts recording its last-success timestamp, it is wired into
`SyncFreshness`; the component's own comment forbids going back to `Date.now()` for
the data line.

## 3. The global dashboard read pure fake data

`ui/src/app-v2/dashboard/GlobalDashboard.tsx` — the `MOCK_GLOBAL_SUMMARY` import is
gone (it survives only in the explanatory header comment) and rows now derive from
`useDataStore`:

- one row per real project from `projects`, with numbers from that project's own
  `workItems` / `sessions` / `inboxItems` / `events`;
- `activeBlockerCount` = blocked work items + input-required sessions;
- `healthStatus` is **derived, not fabricated** — blocked work exists → `blocked`,
  pending decisions only → `warning`, neither → `healthy` — and every row also shows
  the raw counts beside the colour, so the label cannot carry a claim the numbers
  do not;
- `lastActivityTime` = the project's most recent canonical event; a project with no
  events renders "no activity recorded", not an invented date.

Scope labelling (acceptance criterion 3) is now on screen: "scope: the N projects
currently read; blockers = blocked work items + input-required sessions; data from
the current backend read", so an aggregate never again passes one project's number
off as the fleet's. Two honest states were added: a failed backend read renders a red
panel with `backendHydrateError` verbatim, and a genuinely empty table says so rather
than rendering a blank that reads as "no projects".

## 4. The `ui/src` sweep the packet required (criterion 5)

Everything found; not all changed, because the packet only required listing them.

| Finding | Location | Status |
|---|---|---|
| `MOCK_GLOBAL_SUMMARY` — pure fake summary with projects that never existed | `app-v2/dashboard/GlobalDashboard.tsx:11,44` | **fixed** (§3) |
| `budgetHealthPercent = 72` — hardcoded health | `components/SupervisionDashboard.tsx:62` | **fixed** (§1) |
| `new Date().toLocaleTimeString()` as sync time | `components/SupervisionDashboard.tsx:101` | **fixed** (§2) |
| `MOCK_CHANNEL_DATA[channelId]` — mock read in the project dashboard | `app-v2/dashboard/ProjectDashboard.tsx:52` | **already fixed earlier** (2026-10-16, `GoalsSection` + project dashboard now say "no data source"); the remaining `data` binding feeds only the DAG scaffolding and timeline, reported in that delivery |
| `MOCK_PROCESSES` — fake PID/runtime/cwd list | `components/AttachSessionDialog.tsx:9` | **reachable** (via `TerminalPanel.tsx:201`) but not fixed — it is a process picker, not a summary number; out of this dispatch's three-located scope. Listed for a follow-up |
| `MOCK_CONTACTS` | `app-v2/AppV2.tsx:8` | **dead code** — `AppV2` is not mounted anywhere (`grep AppV2` finds only its own definition), so nothing the operator sees reads it |
| `MobileCompanionView.tsx:68` "synced at <event time>" | `views/MobileCompanionView.tsx` | already honest — derived from the last real event, and says "not yet synced" when there is none |
| The four forms' `new Date().toISOString()` | `SupervisorCommandBar` / `WorkItemForm` / `DelegationOverlay` / `AddSeatDialog` | **not a freshness claim** — creation timestamps for rows that are not persisted (reported earlier as write-only forms); not in scope here |

No fake data file was added, and no assertion was weakened to make a test pass.

## 5. Verification

- `tsc --noEmit`: clean.
- `vitest run`: **199/199**, 13 files.
- `vite build`: green (the >500 kB chunk advisory is pre-existing).
- `scripts/lint-changed.sh track/infra-foundation`: every changed file is delta-zero
  against a `track` worktree — `SupervisionDashboard.tsx` 2/2, `SyncFreshness.tsx` 0/0
  (new), `GlobalDashboard.tsx` 0/0, `seatAttribution.test.ts` 0/0 (one transient +1
  from a stray expression statement was fixed, not suppressed).

The first frontend run failed on the inherited `NODE_OPTIONS` pointing at a deleted
`.agentd` loader; tests were rerun in child shells with only that stale preload
unset. tmux global state, the session and the model were untouched.

## 6. Not verified

Nothing was rendered or clicked. `GlobalDashboard` is reached through the Supervisor
panel and now reads the store, but no pixel-level or click-level observation was made
from this seat (the screen is locked from a seat shell). The no-budget-health cell and
the "unknown (not recorded)" sync line are the operator's to confirm read as honest
rather than as empty.

## 7. Boundaries

No backend change, no database write, no seat-model or session change, no acceptance
card, no merge. Only the three files above plus the new `SyncFreshness` component were
touched; `ui/src/lib/api.ts` and `types-dto.ts` were not touched.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
