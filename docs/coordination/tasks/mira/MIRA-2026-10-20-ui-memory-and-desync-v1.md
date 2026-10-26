# Delivery: restart restores the last project and tab; the timeline type filter is real and remembered; two legacy fakes removed

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-20-ui-memory-and-desync-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-20 |
| version | v1 |
| dispatch | disp-DFeH6Ut_aNfzIbuuR21V5 |
| depends_on | AEGIS-2026-10-20-ui-memory-and-desync-v1 |
| tags | restart-restore, timeline-filter, mock-removal, delivery, mira |

---

## 0. One paragraph

Three independent things, as the packet requires them kept separate. One: the app now
restarts into the project and tab the operator was last in, by persisting the active
project id in the store that already persists the per-project tab and selection. Two: the
timeline's event-type filter is real for the first time (the old `templateFilter` state
was written but never read), memory and topic traffic is collapsed by default, the
operator's choice is remembered per project, and what is collapsed is named on screen
with a one-click expand. Three: two legacy fake datasets are gone — a fake process list
that now says "no data" and names the missing capability, and a dead component's fake
contacts and messages, deleted.

## 1. Item 1 — restart restores the last project and tab

**Before.** `App.tsx` already restored each project's tab and selected object on project
switch — but from `projectUIStatesRef`, an in-memory mirror of `useAppStore`'s state. The
app closed and it was gone. So a restart always landed on the default `dispatches` tab
even though the destination was on disk.

**After.** `useAppStore` was already wrapped in zustand's `persist` (key
`seatloom-app-store`, localStorage), so the per-project tab and selection already
survived a restart. Only the **project id** was missing: it lived in `useDataStore`'s
in-memory state. `activeProjectId` is now a persisted field of the same store, with a
`setActiveProjectId` action.

- `App.tsx` writes it whenever the active project changes (after hydration, so a
  transient initial `null` is never recorded).
- A restore effect, run once when `backendHydrated` flips true, calls `setActiveProject`
  only if the recorded id still names a project that exists. The existing per-project tab
  restore then runs unchanged and lands on the recorded tab and selection.
- **Failure cannot blank the app.** No record, an unreadable record, or a record naming a
  deleted project all fall through to the normal default. `persist` hands back its default
  state when storage is unusable, and the effect only calls `setActiveProject` on a hit.

No new localStorage key was introduced; the mechanism is the one already in the store.

**Files:** `ui/src/stores/useAppStore.ts` (persisted field + action),
`ui/src/App.tsx` (write + guarded restore).

**Not verified:** no restart was performed from this seat. The restore is wired through
the existing persist mechanism and guarded, but "close and reopen lands on the same
project" has not been observed.

## 2. Item 2 — the event-type filter is real, collapses by default, and is remembered

### 2a. The dead switch, fixed first

**Before.** `TimelineView.tsx` held `const [templateFilter, setTemplateFilter] =
useState<string>('all')` and no filter ever read it — the `filteredEvents` memo depended
only on `events`, `searchTerm` and `typeFilter`. `clearFilters` reset it. A switch that
changed nothing while looking live is worse than no switch, because it tells the operator
the filter is already applied.

**After.** The state is a `hiddenTypes: Set<string>` that the filter actually consults:
`const matchesVisibility = !hiddenTypes.has(event.event_type)` in the memo. `clearFilters`
and the reset buttons now clear it through the same setter.

### 2b. The default is "collapsed", for a measured reason

The packet's rationale, measured on live before this change: of the newest 400 ledger rows,
106 are `MemoryRecalled`, 68 `MemoryCreated` and 6 `TopicOpened` — 180 of 400 — while every
dispatch, acceptance and verdict row the operator opens the timeline to read is a minority.
The operator's ruling was "collapsed on the first run", so `COLLAPSED_BY_DEFAULT` holds
exactly those three and the source comment carries the counts.

### 2c. The choice is remembered — per project

The hidden set is persisted to
`localStorage['seatloom.timeline.visibility.<projectId>']`, read once when the active
project becomes known. Per project, not global: the timeline is per-project, and a global
key would carry one project's preference into another project's timeline and hide rows the
operator never chose to hide there. A first run (no key) uses the collapsed default; a
corrupt or unreadable value falls back to the same default rather than throwing.

### 2d. Collapsing is not hiding

When any type is collapsed, a strip names the collapsed types, the count of rows removed,
offers a chip per type to bring just that one back, and a single "expand all" control for
all of them.
The filter panel gained the same per-type toggles, so the visibility state is editable
from either place and both show the same truth.

### 2e. Verification

Five tests in `ui/src/views/TimelineVisibility.test.tsx`: memory/topic collapsed by
default with the counts named; the toggle really filters (chip → row appears, chip
disappears, count updates); the choice survives a remount; each project keeps its own
choice; a corrupt stored value falls back instead of throwing.

**Files:** `ui/src/views/TimelineView.tsx`, `ui/src/views/TimelineVisibility.test.tsx`.

**Not verified:** persistence across a real restart, and the panel's appearance with the
operator's own event mix.

## 3. Item 3 — two legacy fakes

### 3a `AttachSessionDialog`'s `MOCK_PROCESSES`

**Before.** Three hardcoded `{pid, runtime, cwd}` entries rendered under the label "detected
local processes", selectable, with an "attach and sync" button. All three were invented,
and the button only closed the dialog — nothing was bound or restored.

**After.** The list is a no-data state that names the missing capability: the backend has
no process registry, so there is no real list and no command to call; what SeatLoom can
mirror is tmux sessions, and binding an arbitrary PID to a seat is not implemented. The
confirm button is disabled and labelled "not implemented" with a tooltip saying the same.
The component itself is kept: it marks a real gap, and deleting it would remove the gap
from view.

### 3b `AppV2`'s `MOCK_CONTACTS`

**Before.** `ui/src/app-v2/AppV2.tsx` imported `MOCK_CONTACTS`, and `mock-data.ts` also
carried `MOCK_MESSAGES` and `MOCK_GLOBAL_SUMMARY`.

**After.** `AppV2.tsx` is deleted — grep confirms it is dead (its only occurrences were
its own definition and historical coordination docs). With no consumer, `MOCK_CONTACTS`,
`MOCK_MESSAGES` and `MOCK_GLOBAL_SUMMARY` were removed from `mock-data.ts` as well (the
last was already orphaned by the 2026-10-20 V5 change that made `GlobalDashboard` derive
from the store). The remaining `MOCK_CHANNEL_DATA` has known consumers and is left alone;
the file header now records what was removed and why, and warns against adding fake data
without naming its consumer.

## 4. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest run` | **204/204**, 14 files (5 new for item 2) |
| `vite build` | green (the >500 kB chunk advisory is pre-existing) |
| `lint-changed.sh` | every changed file delta-zero vs a `track` worktree: `App.tsx` 15/15, `useAppStore.ts` 0/0, `AttachSessionDialog.tsx` 0/0, `mock-data.ts` 0/0, `TimelineVisibility.test.tsx` 0/0 (new); `TimelineView.tsx` 10/9 (one pre-existing unused import now genuinely unused after the filter became real — an improvement, not a new issue) |
| `verify-docs.sh` | this note clean |

**Runtime note:** the codemode sandbox failed on the inherited `NODE_OPTIONS` (a deleted
`.agentd` loader) at the start of this dispatch; it recovered on its own and no
environment, session or model change was made.

## 5. Boundaries

No backend change, no database write, no acceptance card, no merge. No test assertion was
weakened. `api.ts` and `types-dto.ts` untouched. No new navigation.

## 6. What is not verified

No restart was performed, no timeline was opened, and no dialog was clicked from this
seat. Item 1's restore is wired through the existing persist mechanism and guarded;
items 2 and 3 are covered by unit tests. Clicking through is the operator's.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
