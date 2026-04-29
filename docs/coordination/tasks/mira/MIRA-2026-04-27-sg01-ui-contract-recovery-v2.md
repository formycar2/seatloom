# MIRA-2026-04-27-sg01-ui-contract-recovery-v2

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-29 18:00 |
| Acceptance owner | Lyra |
| Gate | SG-01 UI Contract Baseline |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. Recover the prototype from `FAIL` to SG-01-ready baseline.
2. Use only the contract pack and named UI files unless blocked.
3. Keep the shell as one screen with a bottom Terminal panel.
4. Fix Inbox state mutation, Timeline filters/detail, Handoff lifecycle, and SG-01 shortcuts.
5. Remove forbidden MVP copy and off-contract labels.
6. Persist the delivery report in English; keep terminal progress summaries in Chinese.

## 2. Contract pack

Mandatory reads, in this order:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`
4. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
5. `docs/coordination/COORDINATION_RULES.md`
6. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`
7. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`

## 3. Need-to-know scope

Read only:

- this packet
- the contract pack above
- files directly listed in the required work table
- any adjacent component imported by those files when needed to complete the fix

Do not consume by default:

- full repo history
- unrelated seat packets
- unrelated source directories
- broad architecture docs outside the contract pack unless a blocker requires them

If blocked by missing product truth, stop and return a blocker note instead of widening scope silently.

## 4. Token-efficiency rules

| Item | Rule |
|---|---|
| Input budget target | <= 7,000 tokens total live context |
| Output budget target | <= 1,200 words in the delivery artifact body, excluding tables |
| Sync policy | Delta-only from `MIRA-2026-04-27-sg01-ui-contract-recovery-v1.md` |
| Preferred output | changed files, flow checklist, build result, blockers |
| Truncation strategy | If output grows too long, keep P0/P1 evidence and move optional commentary to a short risk note |

## 5. Delta from v1 packet

New requirements in this v2 packet:

- `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md` is now mandatory authority for execution behavior.
- Micro-brief plus contract-pack workflow is now required.
- Need-to-know scope and token budgets are explicit.
- Terminal progress must be Chinese; durable artifacts must remain English.

## 6. Required work

| Priority | Requirement | Exact file targets |
|---|---|---|
| P0 | Rebuild shell around Sidebar + Main + Detail + bottom Terminal panel | `ui/src/App.tsx`, `ui/src/layouts/AppShell.tsx`, `ui/src/layouts/TopNav.tsx`, `ui/src/components/TerminalPanel.tsx` |
| P0 | Make Inbox detail-first and state-correct in mocked behavior | `ui/src/views/InboxView.tsx`, `ui/src/components/InboxItem.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Implement Timeline filters and event-detail behavior | `ui/src/views/TimelineView.tsx`, `ui/src/components/EventRow.tsx`, add a component under `ui/src/components/` if needed |
| P0 | Complete Handoff create/send/accept/return/complete path | `ui/src/components/HandoffForm.tsx`, `ui/src/components/HandoffDetail.tsx`, `ui/src/components/WorkItemDetail.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Wire SG-01 keyboard contract and terminal toggle | `ui/src/hooks/useGlobalShortcuts.ts`, `ui/src/App.tsx`, relevant view components |
| P1 | Add pending/success/error/retry/fallback states to continuity flows | `ui/src/components/AttachSessionDialog.tsx`, `ui/src/components/WrapLaunchDialog.tsx`, `ui/src/components/SwitchRuntimeDialog.tsx`, `ui/src/components/SessionDetail.tsx`, `ui/src/components/InitDialog.tsx` |
| P1 | Align UI types and mock data with contract | `ui/src/types/index.ts`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts` |
| P2 | Normalize off-contract labels and hard-coded copy | `ui/src/components/TerminalPanel.tsx`, `ui/src/views/AllProjectsView.tsx`, `ui/src/layouts/StatusBar.tsx` |

## 7. Done definition

All items below must be true:

- No Critical findings remain against the acceptance basis.
- `ui` shell matches the one-screen contract with a bottom Terminal panel.
- Inbox actions mutate mocked object state, not just visible rows.
- Timeline supports seat, work item, event type, and time filtering plus event-detail review.
- Handoff detail and state flow cover create, send, accept, return, and complete.
- Required SG-01 shortcuts are wired and usable.
- Forbidden MVP copy is removed.
- `cd ui && pnpm build` passes.
- Delivery report is persisted under the required artifact path.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-contract-recovery-delivery-v2.md`

Required sections:

1. Micro-brief
2. Changed files
3. Flow coverage checklist (`implemented` / `partial` / `missing`)
4. Build command + result
5. Blockers / risks / next owner

## 9. Reporting rules

- Durable artifact: English only.
- Terminal progress summary: Chinese only.
- Do not paste long code listings into the delivery report.
- If blocked, report exact file, missing truth, and recommended owner.
