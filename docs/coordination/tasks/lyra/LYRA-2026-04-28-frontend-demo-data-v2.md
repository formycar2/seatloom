# LYRA-2026-04-28-Frontend-Demo-Data-v2

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Completed and validated |
| Supersedes | `docs/coordination/tasks/lyra/LYRA-2026-04-28-frontend-demo-data-v1.md` |
| Scope | Frontend demo content, density, and narrative alignment for `seatloom` |
| Primary data source | `ui/src/stores/useDataStore.ts` |
| Compatibility mirror | `ui/src/mockData.ts` |
| Related memory writeback | `docs/coordination/memory/2026-04-28.md`, `docs/coordination/MEMORY.md` |

## 1. Purpose

Replace the remaining lightweight / mixed-language demo content with a Chinese-first, log-consistent, higher-density dataset so the prototype reads like the real `2026-04-28` coordination day instead of a placeholder sample.

## 2. Active product-content decision

1. Visible demo content must default to Chinese across core user-facing surfaces.
2. Narrative density must match the real collaboration scale of the day, not a toy sample.
3. `ui/src/stores/useDataStore.ts` is the active authority for demo truth.
4. `ui/src/mockData.ts` is kept aligned only for legacy compatibility and should not overtake the store as the primary source.
5. Fallback locale text must not reintroduce English if the wrong locale is selected accidentally.

## 3. Realism and scale targets

| Target | Result |
|---|---|
| Core project story anchored to real day log | Done |
| Chinese-first visible content | Done |
| Enough data to support deep browsing | Done |
| Non-placeholder Inbox / Timeline / WorkItems narrative | Done |
| Store authority clarified | Done |

### Seeded core-project scale (`p-1`)

| Entity | Count |
|---|---|
| Seats | 5 |
| Sessions | 6 |
| WorkItems | 9 |
| Handoffs | 6 |
| Events | 19 |
| Inbox items | 8 |

## 4. Coverage map

| Surface | Files aligned | Intent |
|---|---|---|
| App shell and navigation | `ui/src/App.tsx`, `ui/src/layouts/TopNav.tsx`, `ui/src/layouts/Sidebar.tsx`, `ui/src/layouts/DetailPane.tsx`, `ui/src/layouts/StatusBar.tsx` | Keep global chrome Chinese-first and make project state feel live rather than decorative |
| Main views | `ui/src/views/InboxView.tsx`, `ui/src/views/TimelineView.tsx`, `ui/src/views/WorkItemsView.tsx`, `ui/src/views/AllProjectsView.tsx` | Raise information density and align each view to real coordination workflows |
| Detail surfaces | `ui/src/components/SeatDetail.tsx`, `ui/src/components/SessionDetail.tsx`, `ui/src/components/HandoffDetail.tsx`, `ui/src/components/WorkItemDetail.tsx`, `ui/src/components/EventRow.tsx` | Ensure drill-down content remains Chinese, specific, and traceable |
| Session / command UI | `ui/src/components/TerminalPanel.tsx`, `ui/src/components/WrapLaunchDialog.tsx`, `ui/src/components/AttachSessionDialog.tsx`, `ui/src/components/SwitchRuntimeDialog.tsx`, `ui/src/components/SwitchProtectionDialog.tsx` | Keep runtime workflows readable while preserving technical context |
| Creation / orchestration dialogs | `ui/src/components/AddSeatDialog.tsx`, `ui/src/components/WorkItemForm.tsx`, `ui/src/components/HandoffForm.tsx`, `ui/src/components/InitDialog.tsx`, `ui/src/components/PipelineProgress.tsx` | Remove lightweight placeholder wording and make forms match the collaboration model |
| Locale and data authority | `ui/src/i18n.ts`, `ui/src/stores/useLocaleStore.ts`, `ui/src/stores/useDataStore.ts`, `ui/src/mockData.ts` | Freeze Chinese-first fallback behavior and keep all seed narratives coherent |

## 5. Content alignment rules applied

1. Replaced remaining mixed-language UI copy on visible surfaces.
2. Expanded summaries, goals, and inbox narratives to reflect real task pressure and handoff context.
3. Replaced unrelated or underscaled sample stories with the actual `2026-04-28` coordination thread.
4. Kept technical literals only where they are part of the product truth, such as file paths, runtime names, IDs, and command snippets.
5. Preserved product meaning and structure; changes are language, realism, and density corrections only.

## 6. Validation

| Command | Result |
|---|---|
| `cd ui && npx tsc --noEmit` | PASS |
| `cd ui && pnpm build` | PASS |

## 7. Follow-up still open

- Typography and cross-surface visual rhythm remain a separate follow-up under the active readability workstream.
- If another seat edits demo data next, it must update `useDataStore.ts` first and only then mirror critical deltas into `mockData.ts` when needed.
