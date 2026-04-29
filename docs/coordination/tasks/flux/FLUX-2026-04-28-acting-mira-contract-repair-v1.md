# FLUX-2026-04-28-Acting-Mira-Contract-Repair-v1

| Field | Value |
|---|---|
| Owner | Flux |
| Acting role | Temporary UI/UED repair seat replacing Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-28 18:00 |
| Acceptance owner | Lyra |
| Gate | SG-01 UI Contract Baseline |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. Act as temporary Mira replacement for constrained UI contract repair only.
2. Read only the contract pack and named UI files unless blocked.
3. Adopt only Lyra-approved changes from the Flux design review.
4. Do not redesign Sidebar/Main information architecture.
5. Preserve or improve bilingual readability on every touched surface.
6. Persist the delivery artifact in English; keep terminal progress summaries in Chinese.

## 2. Decisions needed

No new product decision is requested by default.

Escalate to Lyra only if:
- a required contract behavior cannot be implemented without changing product meaning;
- a structural change beyond the approved action packet appears necessary; or
- mock data is too incomplete to show the required flow truthfully.

## 3. Decisions made

These decisions are already frozen for this packet:

- `Terminal` is a bottom collapsible panel, not a fourth full-page tab.
- `Sidebar` keeps its active contract role: Inbox badge + Seats + Sessions + WorkItems.
- `Timeline` must support Seat / WorkItem / event type / time filtering.
- `MorningDigest` must not claim facts that the prototype does not compute.
- `Inbox` action visibility may improve, but not before row-selection-to-Detail behavior is repaired.
- Do not add a new Sidebar/Main redesign without explicit Lyra approval.

## 4. Contract pack

Mandatory reads, in this order:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`
4. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
5. `docs/coordination/COORDINATION_RULES.md`
6. `docs/coordination/reviews/2026-04-27-flux-acting-mira-design-suggestion-review.md`
7. `docs/coordination/tasks/mira/MIRA-2026-04-27-typography-harmony-fix-v1.md`

## 5. Need-to-know scope

Read only:
- this packet;
- the contract pack above;
- files directly named in the required work table;
- adjacent imports only when required to complete the assigned fix.

Do not consume by default:
- full repo history;
- unrelated seat activity;
- broad architecture exploration beyond the contract pack;
- unrelated QA/evidence work.

If blocked, stop and return one compact blocker note with exact file path and missing truth.

## 6. Token-efficiency rules

| Item | Rule |
|---|---|
| Input budget target | <= 7,000 tokens live context |
| Output budget target | <= 1,000 words in the delivery artifact body, excluding tables |
| Sync policy | Delta-only from the current accepted contract and Lyra review artifact |
| Preferred output | changed files, flow checklist, build result, screenshot/evidence paths, blockers |
| Truncation strategy | Keep P0/P1 evidence first; collapse commentary into short risk notes |

## 7. Action packet

| Priority | Requirement | Exact file targets |
|---|---|---|
| P0 | Restore `Terminal` to a bottom collapsible panel with preserved session tabs/state; remove it as a top-level main tab | `ui/src/App.tsx`, `ui/src/layouts/AppShell.tsx`, `ui/src/layouts/TopNav.tsx`, `ui/src/components/TerminalPanel.tsx`, `ui/src/hooks/useGlobalShortcuts.ts` |
| P0 | Repair Inbox selection-to-Detail behavior so clicking a row opens the related object detail; do not keep Inbox as action-only row mutation | `ui/src/views/InboxView.tsx`, `ui/src/components/InboxItem.tsx`, `ui/src/App.tsx`, `ui/src/stores/useDataStore.ts` |
| P0 | Complete contract-required Timeline filtering: Seat / WorkItem / event type / time, plus Sidebar-to-Timeline linkage where applicable | `ui/src/views/TimelineView.tsx`, `ui/src/layouts/Sidebar.tsx`, `ui/src/stores/useAppStore.ts`, `ui/src/App.tsx` |
| P1 | Remove or neutralize fake truth claims in `MorningDigest`; show only computed or safely generic prototype copy | `ui/src/components/MorningDigest.tsx`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts` |
| P1 | Improve Inbox action visibility after the selection/detail repair; touched surfaces must also respect bilingual readability guidance | `ui/src/components/InboxItem.tsx`, `ui/src/views/InboxView.tsx`, `ui/src/styles/globals.css`, additional touched UI files only if required |
| P1 | Keep touched navigation/list/detail text comfortably readable in both `en` and `zh`; avoid extreme tiny uppercase/tracking on modified surfaces | only files touched by the work above |

## 8. Blockers to avoid turning into silent scope drift

Do not do these in this packet unless Lyra explicitly re-approves them:
- simplifying Sidebar into a Seat-only monitor;
- adding a default project-summary Detail pane as a new product behavior;
- broad visual redesign outside touched surfaces;
- speculative reconcile-flow redesign.

## 9. Stage-gate status

Current gate state: `SG-01 UI Contract Baseline = HOLD`

This packet helps clear specific UI blockers, but it does not itself grant gate PASS.
Lyra will re-review after Flux submits the delivery artifact.

## 10. Done definition

All items below must be true:

- `Terminal` is no longer a main-panel tab.
- Bottom terminal panel can be toggled independently and remains available alongside Inbox / Timeline / WorkItems.
- `Inbox` row selection opens Detail for the related object or handoff context.
- `Timeline` supports Seat / WorkItem / event type / time filtering in the prototype.
- `MorningDigest` no longer states uncomputed factual claims.
- Touched surfaces remain readable in both `en` and `zh` and do not regress the typography floor.
- No off-contract Sidebar/Main redesign is introduced.
- `cd ui && pnpm build` is run and the result is recorded.
- Delivery artifact is persisted at the required path.

## 11. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-delivery-v1.md`

Required sections:

1. Micro-brief
2. Changed files
3. Flow coverage checklist (`implemented` / `partial` / `missing`)
4. Build command + result
5. Evidence paths (screenshots if UI changed visibly)
6. Blockers / risks / next owner

## 12. Reporting rules

- Durable artifact: English only.
- Terminal progress summary: Chinese only.
- Keep tmux updates short and delta-based.
- Do not paste long code listings into the delivery artifact.
- If blocked, report exact file, missing truth, and recommended owner.
