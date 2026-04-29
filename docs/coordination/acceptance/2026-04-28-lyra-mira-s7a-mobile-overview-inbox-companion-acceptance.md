# Acceptance: Mira S7A Mobile Overview + Inbox Companion

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s7a-mobile-overview-inbox-companion-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-delivery-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, ui, mobile, companion, mira, s7a |

## Verdict

**CONDITIONAL PASS**

Lyra accepts the structural direction of the new mobile companion surface, the in-app entry point, the bounded shell behavior, and the shared theme-token compliance.

The packet is **not closed yet** because two contract-critical gaps remain:

1. `Mobile Overview` does not yet expose explicit counts for `pending approvals / gates`, `Prompt blocked`, and `pending handoffs` as separate monitor signals.
2. `Mobile Inbox` still renders the first five inbox rows instead of a deterministic `high priority / action needed` filter.

A compact follow-up fix packet is required before `S7A` can close and before mobile action-card work can start.

## Scope Reviewed

- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-delivery-v1.md`
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/layouts/TopNav.tsx`
- `ui/src/layouts/AppShell.tsx`
- `ui/src/App.tsx`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | A prototype-visible mobile companion surface exists and is reachable in-app | `ui/src/layouts/TopNav.tsx:49`, `ui/src/App.tsx:253`, `ui/src/layouts/AppShell.tsx:40` | PASS | The `Smartphone` entry point is visible, the `mobile` tab renders a dedicated view, and the desktop sidebar is hidden for a bounded mobile shell. |
| 2 | Mobile shell stays distinct from the desktop detail-pane layout | `ui/src/App.tsx:384`, `ui/src/App.tsx:386`, `ui/src/layouts/AppShell.tsx:40` | PASS | The mobile tab suppresses sidebar + detail rail and presents a narrow single-column companion surface. |
| 3 | Project identity, sync freshness, and a deterministic recommended next action are visible from canonical state | `ui/src/views/MobileCompanionView.tsx:68`, `ui/src/views/MobileCompanionView.tsx:78`, `ui/src/views/MobileCompanionView.tsx:135` | PASS | The surface reads active project truth and derives the next-action block without LLM dependency. |
| 4 | Theme-token compliance is preserved | `ui/src/views/MobileCompanionView.tsx:65`, `ui/src/views/MobileCompanionView.tsx:88`, `ui/src/views/MobileCompanionView.tsx:136` | PASS | The view uses shared semantic tokens only; no one-off palette was introduced. |
| 5 | Mobile Overview exposes explicit `pending approvals / gates` signal | `ui/src/views/MobileCompanionView.tsx:87`, `ui/src/views/MobileCompanionView.tsx:111` | HOLD | The current cards show `Alerts`, `Blocked`, `WorkItems`, and `Inbox`, but not a distinct gate / approval count. |
| 6 | Mobile Overview exposes explicit `Prompt blocked` signal | `ui/src/views/MobileCompanionView.tsx:98`, `ui/src/views/MobileCompanionView.tsx:142` | HOLD | Prompt-blocked state influences the blocker count and next-action copy, but it is not surfaced as its own monitor metric. |
| 7 | Mobile Overview exposes explicit `pending handoff` signal | `ui/src/views/MobileCompanionView.tsx:111` | HOLD | There is no separate in-flight handoff count yet. |
| 8 | Mobile Inbox defaults to `high priority / action needed` rows rather than raw history order | `ui/src/views/MobileCompanionView.tsx:160` | HOLD | The current list uses `slice(0, 5)` over the full inbox and therefore depends on seed order instead of an urgency filter. |
| 9 | Mobile Inbox rows show priority, type, owner, object ref, summary, wait indicator, and a detail/open path | `ui/src/views/MobileCompanionView.tsx:163`, `ui/src/views/MobileCompanionView.tsx:173`, `ui/src/views/MobileCompanionView.tsx:179` | PASS | The row layout covers the required fields and provides existing drill-through to Detail. |
| 10 | Build stays green after the mobile slice lands | `cd ui && pnpm build` | PASS | Lyra re-ran the production build locally on 2026-04-28 and confirmed success. |

## Findings

### High

1. **Explicit mobile monitor truth is incomplete.**
   The packet contract requires standalone visibility for `pending approvals / gates`, `Prompt blocked`, and `pending handoffs`, but the current overview only shows aggregated `Alerts`, `Blocked`, `WorkItems`, and `Inbox` cards. This weakens the phone-first supervision story because the user still cannot tell which action class is waiting without opening rows manually.
   - Evidence: `ui/src/views/MobileCompanionView.tsx:87`
   - Evidence: `ui/src/views/MobileCompanionView.tsx:111`

### Medium

1. **Urgent mobile inbox filtering is not deterministic yet.**
   The current implementation renders `currentData.inboxItems.slice(0, 5)`, which means background or lower-value records can appear purely because of seed order. The contract requires a default `high priority / action needed` queue.
   - Evidence: `ui/src/views/MobileCompanionView.tsx:160`

## Gate Decision

- **Mobile companion structural direction:** **GO**
- **`S7A` packet closure:** **HOLD**
- **Next mobile packet after `S7A`:** **DO NOT START** until the monitor-truth fix is accepted

## Follow-up Actions

- Mira: complete the compact fix packet `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-v1.md`.
- Lyra: re-review the corrected mobile monitor surface and close `S7A` only after the overview metrics and urgency filter match the contract.
