# MIRA-2026-04-27-sg01-ui-contract-recovery-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-29 18:00 |
| Acceptance owner | Lyra |
| Gate | SG-01 UI Contract Baseline |

## 1. Objective

Recover the SeatLoom prototype from Lyra verdict `FAIL` to a contract-safe baseline for SG-01.

## 2. Authority and evidence

Required contract set:

1. `docs/prd-v0.4.md`
2. `docs/interaction-spec-v1.0.md`
3. `docs/acceptance-spec-v1.0.md`
4. `docs/mvp-scenarios.md`
5. `docs/architecture-decisions.md`
6. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`
7. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`

## 3. Decisions already frozen

1. Terminal must be a bottom toggle panel inside the main shell, not a top-level tab.
2. Inbox must be detail-first and action-correct; row deletion alone is not valid behavior.
3. Timeline must support seat, work item, event type, and time filtering plus event-detail review.
4. Handoff UI must represent create, send, accept, return, and complete.
5. Forbidden MVP copy and off-contract modes must be removed.

## 4. Required work

| Priority | Requirement | Exact file targets |
|---|---|---|
| P0 | Rebuild shell around Sidebar + Main + Detail + bottom Terminal panel | `ui/src/App.tsx`, `ui/src/layouts/AppShell.tsx`, `ui/src/layouts/TopNav.tsx`, `ui/src/components/TerminalPanel.tsx` |
| P0 | Make Inbox detail-first and state-correct in mocked behavior | `ui/src/views/InboxView.tsx`, `ui/src/components/InboxItem.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Implement Timeline filters and event-detail behavior | `ui/src/views/TimelineView.tsx`, `ui/src/components/EventRow.tsx`, add component if needed under `ui/src/components/` |
| P0 | Complete Handoff create/send/accept/return/complete path | `ui/src/components/HandoffForm.tsx`, `ui/src/components/HandoffDetail.tsx`, `ui/src/components/WorkItemDetail.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Wire SG-01 keyboard contract and terminal toggle | `ui/src/hooks/useGlobalShortcuts.ts`, `ui/src/App.tsx`, relevant views/components |
| P1 | Add pending, success, error, retry, and fallback states to continuity flows | `ui/src/components/AttachSessionDialog.tsx`, `ui/src/components/WrapLaunchDialog.tsx`, `ui/src/components/SwitchRuntimeDialog.tsx`, `ui/src/components/SessionDetail.tsx`, `ui/src/components/InitDialog.tsx` |
| P1 | Align UI types and mock data with contract | `ui/src/types/index.ts`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts` |
| P2 | Normalize off-contract labels and hard-coded copy | `ui/src/components/TerminalPanel.tsx`, `ui/src/views/AllProjectsView.tsx`, `ui/src/layouts/StatusBar.tsx` |

## 5. Done definition

All items below must be true:

- No Critical findings remain against the current acceptance basis.
- `ui` shell matches the one-screen contract with a bottom Terminal panel.
- Inbox actions mutate mocked object state, not just visible rows.
- Timeline filter set includes seat, work item, event type, and time range, with event-detail review.
- Handoff detail and state flow cover create, send, accept, return, and complete.
- Required SG-01 shortcuts are wired and usable.
- Forbidden MVP copy is removed.
- `cd ui && pnpm build` passes.
- Mira publishes a delivery report with changed file list and flow coverage checklist under `docs/coordination/tasks/mira/`.

## 6. Constraints

- Do not redefine product semantics.
- Do not treat current rejected prototype behavior as acceptable by momentum.
- If a product conflict blocks implementation, escalate it to Lyra in file-backed form.
- Preserve the existing visual language where it does not conflict with contract truth.

## 7. Delivery artifact required

Publish completion as:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-contract-recovery-delivery-v1.md`

Include:

- changed files
- flow coverage checklist (`implemented` / `partial` / `missing`)
- build command + result
- known risks or follow-up notes
