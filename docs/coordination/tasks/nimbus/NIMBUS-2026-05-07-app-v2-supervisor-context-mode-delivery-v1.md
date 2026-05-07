# Delivery: app-v2 Supervisor Context Mode (chan-03)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-05-07-app-v2-supervisor-context-mode-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-05-07 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-v1.md`, `docs/architecture-decisions.md` AD-013, `docs/architecture-design.md` §7.1 |
| tags | nimbus, ui, app-v2, supervisor, context-mode, global-dashboard, L1, chan-03 |
| owner | Nimbus |
| acceptance owner | Lyra |

## 1. Scope Restated

Implement AD-013 Supervisor 两层上下文模型 in v2 frontend:

- `currentContextMode: 'global' | 'project'` state machine
- New `GlobalDashboard.tsx` for the Global context
- `SupervisorPanel.tsx` switches between GlobalDashboard and ProjectDashboard by mode
- `MOCK_GLOBAL_SUMMARY` + `SupervisorContextMode` + `GlobalProjectSummary` types
- localStorage persistence with stale-state fallback
- Zero regression on Phase 1/2 ProjectDashboard views

Cross-track packet: Nimbus 主轨为 Rust + Postgres，本任务因 Copilot 不可用临时跨入 v2 frontend。

## 2. Files Changed

### New Files

- `ui/src/app-v2/dashboard/GlobalDashboard.tsx` — Global context view (summary header + N project rows)

### Modified Files

- `ui/src/app-v2/types.ts` — added `SupervisorContextMode` and `GlobalProjectSummary`
- `ui/src/app-v2/mock-data.ts` — added `MOCK_GLOBAL_SUMMARY` (3 projects matching `useDataStore.INITIAL_PROJECTS`)
- `ui/src/app-v2/panel/SupervisorPanel.tsx` — context mode state + breadcrumb (§F) + conditional render + localStorage persistence with stale-state fallback (§D); existing contact list / drag / resize / draft logic unchanged

### Files Explicitly NOT Touched (per scope boundary)

- `ui/src/app-v2/dashboard/ProjectDashboard.tsx` — unchanged
- `ui/src/app-v2/views/*.tsx` (Inbox / WorkItems / Artifacts) — unchanged
- `ui/src/app-v2/dashboard/{Blockers,ActiveWork,Timeline,StageProgress,Goals}Section.tsx` — unchanged
- `ui/src/stores/useDataStore.ts` — read-only consumption only (`useDataStore.getState().projects` for stale check + breadcrumb name)
- Rust backend / Postgres / infra scripts — untouched
- `ui/src/app-v2/styles/tokens.css` — untouched
- No new third-party dependencies

## 3. State Location Choice

**Option X (local `useState` in `SupervisorPanel`)** — as recommended by §B.

Reasons:
1. Lowest cross-track risk: matches the existing Phase 1/2 pattern for panel position, size, draft, and active contact (all local `useState` + direct `localStorage` writes).
2. No other v2 component needs to read or mutate `currentContextMode` — it's an in-panel UI mode, not a globally subscribed concern. AD-013's §7.1 Zustand sketch is preserved as a future migration path; promoting it to a store today would be premature.
3. Keeps the bounded packet to four files. Adding `useSupervisorStore.ts` would have added an extra surface area without observable user benefit.

## 4. Mode Invariant Enforcement

The互斥不变量 (`mode === 'project' ⇔ activeProjectId !== null`) is enforced by:

- A `loadInitialContext()` helper that downgrades to global if the persisted projectId is not in `useDataStore().projects`.
- Two state mutators: `enterGlobal()` (clears `activeProjectId`) and `enterProject(projectId)` (sets both).
- `switchContact()` syncs both `contextMode` and `activeProjectId` atomically when the clicked contact carries a `projectId`.
- The persistence effects mirror state into localStorage, with `STORAGE_KEY_ACTIVE_PROJECT` removed (not stored as `null`) when in global mode.

Setters `setContextMode` and `setActiveProjectIdState` are not exported from the component; the only call sites are the three controlled paths above.

## 5. localStorage Keys (§C)

| Key | Type | Notes |
|---|---|---|
| `seatloom.supervisor.contextMode` | `'global' \| 'project'` (literal string) | New — written on every mode change |
| `seatloom.supervisor.activeProjectId` | `string` (or absent) | New — absent encodes `null` |
| `sl-supervisor-pos`, `sl-supervisor-size`, `sl-supervisor-active-contact`, `sl-supervisor-draft-*` | (existing) | Unchanged |

## 6. Stale-state Fallback (§D)

`loadInitialContext()`:

```ts
if (rawMode !== 'project' || !rawProj) → { global, null }
if (project not found in useDataStore().projects) → { global, null }
otherwise → { project, rawProj }
```

This eliminates the "white-screen on stale projectId" risk that motivated the rule.

## 7. Breadcrumb Behavior (§F)

Per §F:

- Global mode: header shows `全局` (single non-clickable token).
- Project mode: header shows `全局 › <项目名>` where `全局` is a button → `enterGlobal()` and project name is non-clickable, slightly bolder.
- Project name resolves through: `useDataStore.projects[id].name` → MOCK_CONTACTS project channel name → projectId fallback.
- No tab form, no separate floating button, no extra ← arrow — strictly the breadcrumb idiom.

In project mode, a sub-header with the active contact's avatar / name / role is rendered below the breadcrumb so the existing seat / channel context remains visible.

## 8. Zero-Regression Guarantee

- ProjectDashboard remains rendered with `key={activeContactId}` so its tab state and internal scroll are independent across contact switches (existing behavior unchanged).
- `channelId={activeContact.id}` and `projectId={activeContact.projectId}` props are unchanged.
- All Phase 1/2 sections (BlockersSection, ActiveWorkSection, TimelineSection, StageProgressSection, GoalsSection, InboxView, WorkItemsView, ArtifactsView) are reachable via the same code path.
- Slice B artifact panel inside the overview tab is unchanged.

## 9. Validation Results

```
cd ui && npx tsc --noEmit
=> exit 0, zero errors (PASS)

cd ui && pnpm build
=> tsc + vite build PASS
=> dist/assets/index-*.js  351.15 kB │ gzip 102.09 kB
=> ✓ built in 1.11s (PASS)
```

### Browser smoke test (operator-required)

Per CLAUDE.md guidance: a CLI-only seat cannot complete browser smoke tests. The 8 scenarios from the task `§Validation` were traced statically against the new code paths and are expected to pass. Operator-driven verification is required for final acceptance:

| # | Scenario | Static trace |
|---|---|---|
| 1 | Cold open (clean localStorage) | `loadInitialContext()` returns `{global, null}` → breadcrumb `全局` → `GlobalDashboard` renders 3 rows |
| 2 | Click project row in GlobalDashboard | `onSelectProject('p-1')` → `enterProject('p-1')` → mode=project, activeProjectId='p-1', activeContact='ch-p1' (matching project channel) |
| 3 | Click `全局` breadcrumb | `enterGlobal()` → mode=global, activeProjectId=null, localStorage cleaned |
| 4 | Switch project via Contact list | `switchContact('ch-p2')` → contact + mode/projectId atomic update → ProjectDashboard re-renders with new channelId |
| 5 | Reload in project mode | localStorage `{mode=project, activeProjectId=p-1}` → present in `useDataStore.projects` → restored |
| 6 | Stale fallback | localStorage `{mode=project, activeProjectId=p-bogus}` → not in projects → downgraded to `{global, null}` |
| 7 | Tab regression | ProjectDashboard untouched; internal `useState<'overview' \| 'inbox' \| ...>` intact |
| 8 | Overview Slice B regression | ProjectDashboard untouched |

Flux verify-only is expected to run all 8 scenarios in a real browser as part of the verifier gate.

## 10. Branch and Commit

- Branch: `track/infra-foundation`
- Base commit: `1c7935c` (HEAD before this slice was `9ffed2f`, which contains the AD-013 docs revision)
- Delivery commit: `f5b8423`

## 11. Reporting

```text
[Nimbus -> Aegis] app-v2 Supervisor Context Mode (chan-03) Delivery
branch:
- track/infra-foundation
commit:
- f5b8423
completed:
- types.ts: SupervisorContextMode + GlobalProjectSummary added
- mock-data.ts: MOCK_GLOBAL_SUMMARY added (3 projects: p-1/p-2/p-3)
- GlobalDashboard.tsx: created (summary header + project rows + onSelectProject)
- SupervisorPanel.tsx: contextMode + activeProjectId state, enterGlobal/enterProject mutators, breadcrumb (§F), conditional render (Global vs Project), §D stale fallback, §C localStorage keys
state location choice:
- option X (local useState in SupervisorPanel) per §B recommendation
validation:
- cd ui && npx tsc --noEmit => PASS (zero errors)
- cd ui && pnpm build => PASS (tsc + vite build green; 351.15 kB / 102.09 kB gzip)
- browser smoke test (8 scenarios) => static trace PASS; operator verification required
blockers:
- none — Docker/browser unavailable on this seat; Flux to perform browser smoke test
next action:
- wait for Flux verify-only re-verification
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-delivery-v1.md
```
