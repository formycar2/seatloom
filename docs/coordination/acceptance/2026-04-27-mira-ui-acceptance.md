# Mira UI Acceptance Review - 2026-04-27

| Field | Value |
|---|---|
| Owner | Lyra |
| Review target | Mira redesign prototype |
| Review scope | `ui/src/App.tsx`, `ui/src/layouts/*`, `ui/src/views/*`, `ui/src/components/*`, `ui/src/mockData.ts`, `ui/src/styles/*` |
| Acceptance basis | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md` |
| Build evidence | `cd ui && pnpm build` -> pass on 2026-04-27 |

## 1. Acceptance verdict

**FAIL**

Reason: the prototype builds, but it does not meet the current product contract for the core shell, Inbox action semantics, Timeline replay contract, Handoff closure, or keyboard/continuity behavior.

## 2. Coverage matrix

| Required flow | Status | Evidence | Gap to contract |
|---|---|---|---|
| Project switcher, recent/pinned, switch protection, project memory | Partial | `ui/src/components/ProjectSwitcher.tsx`, `ui/src/components/SwitchProtectionDialog.tsx`, `ui/src/App.tsx:56` | `Cmd/Ctrl+K` opens an all-projects view instead of the switcher, and filter state is not restored |
| One-screen shell with bottom Terminal panel | Missing | `ui/src/App.tsx:125`, `ui/src/layouts/AppShell.tsx:28`, `ui/src/layouts/TopNav.tsx:15` | Terminal is a main tab, not a bottom toggle panel inside the core work surface |
| Initialize project flow with failure and retry | Partial | `ui/src/components/InitDialog.tsx` | Dialog exists, but trigger path and failure/retry feedback do not match interaction contract |
| Attach running session | Partial | `ui/src/components/AttachSessionDialog.tsx` | Happy-path mock exists; no unavailable/error fallback or pending state |
| Wrap launch new session | Partial | `ui/src/components/WrapLaunchDialog.tsx` | Happy-path mock exists; no pending/error/retry feedback or terminal docking behavior |
| Inbox action queue | Missing | `ui/src/views/InboxView.tsx:14`, `ui/src/components/InboxItem.tsx:38`, `ui/src/stores/useDataStore.ts:180` | Actions delete rows only; no object-state transition, no detail-first review loop |
| Timeline replay with seat/workitem/type/time filters and event detail | Missing | `ui/src/views/TimelineView.tsx:12`, `ui/src/components/EventRow.tsx:39` | Only one coarse event-type filter exists; no seat/workitem/time filters; no event detail surface |
| Create WorkItem | Partial | `ui/src/components/WorkItemForm.tsx` | Form exists and approximates state gates, but keyboard trigger and quick-vs-full behavior are incomplete |
| Create and process Handoff | Missing | `ui/src/components/HandoffForm.tsx:25`, `ui/src/components/HandoffDetail.tsx:43`, `ui/src/App.tsx:182` | Create/send is mocked; accept/return/complete chain is not executable |
| Switch runtime / LaunchPack preview / fallback | Partial | `ui/src/components/SwitchRuntimeDialog.tsx` | Preview is static; launch button does not launch; fallback path is incomplete |
| Interrupted session recovery chain | Missing | `ui/src/components/SessionDetail.tsx:58` | Recovery options are rendered, but none are actionable and no fallback state is shown |
| Morning digest on startup | Partial | `ui/src/components/MorningDigest.tsx` | Digest card exists, but drift/startup evidence is incomplete and mostly hard-coded |
| Keyboard contract and focus loop | Missing | `ui/src/hooks/useGlobalShortcuts.ts:5` | Only `Esc` and `Cmd/Ctrl+K` are wired; required shortcuts and focus loop are absent |

## 3. Findings by severity

### Critical

1. **Core shell is off-contract.** Terminal is implemented as a top-level tab instead of a bottom toggle panel, which breaks the required one-screen continuity surface. Files: `ui/src/App.tsx:25`, `ui/src/App.tsx:125`, `ui/src/layouts/AppShell.tsx:28`, `ui/src/layouts/TopNav.tsx:15`.
2. **Inbox cannot execute the product action loop.** Rows do not open a detail-driven review path, and actions only remove items from the list without updating the underlying WorkItem, Session, or Handoff state. Files: `ui/src/views/InboxView.tsx:14`, `ui/src/views/InboxView.tsx:36`, `ui/src/components/InboxItem.tsx:38`, `ui/src/stores/useDataStore.ts:180`.
3. **Handoff lifecycle is not acceptance-safe.** The prototype supports only a mocked create/send step; accept, return, and complete are missing from the detail pane and state flow. Files: `ui/src/components/HandoffForm.tsx:25`, `ui/src/components/HandoffDetail.tsx:43`, `ui/src/App.tsx:182`, `ui/src/types/index.ts:29`.
4. **Timeline replay contract is missing.** Required seat/workitem/type/time filtering and event-detail behavior are not implemented, so P-03 fails. Files: `ui/src/views/TimelineView.tsx:12`, `ui/src/views/TimelineView.tsx:62`, `ui/src/components/EventRow.tsx:39`.

### High

1. **Keyboard contract is largely absent.** Required shortcuts for tab switching, terminal toggle, create work item, create handoff, and reconcile are not wired. File: `ui/src/hooks/useGlobalShortcuts.ts:5`.
2. **Continuity fallback UX is incomplete across attach, wrap, switch, and recovery.** Current dialogs show happy-path mocks without `pending/success/error`, retry, or next-step fallback behavior. Files: `ui/src/components/AttachSessionDialog.tsx:20`, `ui/src/components/WrapLaunchDialog.tsx:24`, `ui/src/components/SwitchRuntimeDialog.tsx:29`, `ui/src/components/SessionDetail.tsx:58`.
3. **Forbidden MVP copy is still present.** The terminal hardcodes `seatloom run "Fix the project switching logic"`, which reintroduces an out-of-scope mode explicitly excluded by scenarios. File: `ui/src/components/TerminalPanel.tsx:84`; contract ref: `docs/archive/product-history/mvp-scenarios.md:154`.
4. **Schema and mock-state drift remains.** UI types add unsupported values such as `Received` in `HandoffStatus`, `Critical` in `Priority`, and `Developer` in `SeatRole`, which weakens contract alignment. Files: `ui/src/types/index.ts:14`, `ui/src/types/index.ts:21`, `ui/src/types/index.ts:29`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts`.

### Medium

1. **Sidebar information architecture is incomplete.** Contract-required Inbox count and standalone Session list entry points are missing. File: `ui/src/layouts/Sidebar.tsx:25`.
2. **Project-level state memory is only partial.** `activeTab` and a heuristic selection are restored, but filter state is not persisted and non-WorkItem selections drift on restore. Files: `ui/src/App.tsx:56`, `ui/src/stores/useAppStore.ts:14`.
3. **Morning Digest is only partially grounded in contract data.** Interrupted and blocked counts are present, but drift/startup narrative is incomplete and copy is partly hard-coded. File: `ui/src/components/MorningDigest.tsx:10`.
4. **Handoff detail is not data-driven.** Artifact display is hard-coded and not linked to actual handoff data. File: `ui/src/components/HandoffDetail.tsx:43`.
5. **Init and empty/error states are not fully contract-shaped.** Current dialogs and labels do not yet cover the required failure and recovery paths. Files: `ui/src/components/InitDialog.tsx`, `ui/src/components/AttachSessionDialog.tsx`, `ui/src/components/WrapLaunchDialog.tsx`.

### Low

1. **Some labels drift from contract language.** Examples include `Project Authority Dashboard` and `Init Demo`, which should be normalized before gate review. Files: `ui/src/views/AllProjectsView.tsx:17`, `ui/src/layouts/StatusBar.tsx:23`.

## 4. Required fixes for Mira

| Priority | Fix | Exact file targets |
|---|---|---|
| P0 | Rebuild the shell around Sidebar + Main + Detail + bottom Terminal panel; remove Terminal as a main tab | `ui/src/App.tsx`, `ui/src/layouts/AppShell.tsx`, `ui/src/layouts/TopNav.tsx`, `ui/src/components/TerminalPanel.tsx` |
| P0 | Make Inbox detail-first and action-correct in prototype state, not row-removal only | `ui/src/views/InboxView.tsx`, `ui/src/components/InboxItem.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Implement contract-required Timeline filters and an event-detail interaction | `ui/src/views/TimelineView.tsx`, `ui/src/components/EventRow.tsx`, add a dedicated event-detail component if needed |
| P0 | Complete Handoff UI chain for create, send, accept, return, complete | `ui/src/components/HandoffForm.tsx`, `ui/src/components/HandoffDetail.tsx`, `ui/src/components/WorkItemDetail.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Wire keyboard contract and terminal toggle behavior | `ui/src/hooks/useGlobalShortcuts.ts`, `ui/src/App.tsx`, relevant view components |
| P1 | Add pending/error/retry/fallback states to attach, wrap, switch-runtime, and recovery flows | `ui/src/components/AttachSessionDialog.tsx`, `ui/src/components/WrapLaunchDialog.tsx`, `ui/src/components/SwitchRuntimeDialog.tsx`, `ui/src/components/SessionDetail.tsx` |
| P1 | Align UI types and mock data with current contract | `ui/src/types/index.ts`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts` |
| P2 | Normalize off-contract labels and hard-coded copy | `ui/src/components/TerminalPanel.tsx`, `ui/src/views/AllProjectsView.tsx`, `ui/src/layouts/StatusBar.tsx`, related copy surfaces |

## 5. Go / No-Go recommendation for Nimbus handoff

**NO-GO** for full Nimbus implementation handoff.

Allowed now:
- Nimbus may prepare a schema/event alignment memo and implementation sequence.
- Nimbus should not treat the current prototype as baseline truth for shell, Inbox, Timeline, Handoff, or recovery flows.

Release condition for handoff:
- Mira rework must clear all P0 fixes above and return for Lyra acceptance review.
