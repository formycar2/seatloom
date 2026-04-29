# Acceptance: Mira S7A Monitor Truth Fix Recheck

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s7a-monitor-truth-fix-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1.md` |
| verdict | FAIL |
| tags | acceptance, ui, mobile, companion, monitor, truth, mira, s7a |

## Verdict

**FAIL**

Lyra accepts that Mira closed part of the original `S7A` hold: the mobile monitor now exposes separate cards for `pending approvals / gates`, `Prompt blocked`, and `pending handoffs`, and the build remains green.

The recheck still does **not** close `S7A`, because the current counting and filtering rules are not yet aligned to the active mobile contract:

1. the `pending approvals / gates` count under-represents approval-worthy objects that already exist in the seeded canonical Inbox, and
2. the `Urgent Mobile Inbox` still includes non-actionable background records because all `Normal`-priority rows are admitted by default.

The earlier structural verdict remains unchanged: `S7A` is directionally correct, but packet closure stays on hold until the truth rules are tightened.

## Scope Reviewed

- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md` (`INT-17`)
- `docs/ux-spec-v1.1.md` (`UX-13`)
- `docs/acceptance-spec-v1.1.md` (`US-P0-12`, `P-13`, `U-11`)
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/stores/useDataStore.ts`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | Mobile Overview exposes separate monitor signals for approvals/gates, prompt blocks, and pending handoffs | `ui/src/views/MobileCompanionView.tsx:36`, `ui/src/views/MobileCompanionView.tsx:110` | PASS | The three monitor cards are now explicit and visually separate. |
| 2 | `Prompt blocked` count derives from canonical session state | `ui/src/views/MobileCompanionView.tsx:37`, `ui/src/stores/useDataStore.ts:641` | PASS | The count is derived from sessions carrying `prompt_state`. |
| 3 | `pending handoffs` count derives from in-flight canonical handoff state | `ui/src/views/MobileCompanionView.tsx:38`, `ui/src/stores/useDataStore.ts:895`, `ui/src/stores/useDataStore.ts:921`, `ui/src/stores/useDataStore.ts:934` | PASS | The count is derived from handoffs that are not `Completed` or `Expired`. |
| 4 | `pending approvals / gates` count truthfully reflects approval-worthy and gate-waiting objects | `ui/src/views/MobileCompanionView.tsx:40`, `ui/src/stores/useDataStore.ts:1137`, `ui/src/stores/useDataStore.ts:1166` | FAIL | The current rule only counts `待处理交接`, `验收待决策`, and `输入请求`, but the seeded canonical Inbox also contains `需确认范围` and `需要验收` records that are still approval/gate work. |
| 5 | Mobile Inbox defaults to urgent/action-needed rows and excludes background/FYI rows | `ui/src/views/MobileCompanionView.tsx:63`, `ui/src/stores/useDataStore.ts:1147`, `ui/src/stores/useDataStore.ts:1176`, `ui/src/stores/useDataStore.ts:1195` | FAIL | The filter admits every `Normal` row, which still allows non-actionable `背景记录` rows into the urgent mobile queue. |
| 6 | Build stays green after the truth-fix patch | `cd ui && pnpm build` | PASS | Lyra re-ran the production build locally on 2026-04-28 and confirmed success. |

## Findings

### High

1. **The gate / approval count is still semantically incomplete.**  
   The overview card is present, but the implemented allow-list is narrower than the canonical data model currently used by the prototype. `需确认范围` and `需要验收` are gate-bearing or approval-bearing records in the seed set, yet they are excluded from the count. This means the user can still see a lower approval number than the actual mobile action load.
   - Evidence: `ui/src/views/MobileCompanionView.tsx:40`
   - Evidence: `ui/src/stores/useDataStore.ts:1137`
   - Evidence: `ui/src/stores/useDataStore.ts:1166`

2. **The urgent queue still leaks non-actionable rows.**  
   The current filter treats all `Normal` items as urgent, which allows background-only records such as `背景记录` to surface in the `URGENT ONLY` queue. This breaks the monitor-first mobile promise and weakens the bounded action model required by `INT-17` / `UX-13`.
   - Evidence: `ui/src/views/MobileCompanionView.tsx:64`
   - Evidence: `ui/src/stores/useDataStore.ts:1147`

## Gate Decision

- **`S7A` monitor-truth recheck:** **NO-GO**
- **Overall `S7A` packet closure:** **HOLD**
- **Next mobile packet (`S7B`)**: **DO NOT START** until the queue/count truth patch is accepted

## Follow-up Actions

- Mira: tighten the action-needed allow-list for both the approval count and the urgent mobile queue, then return a bounded delivery artifact.
- Lyra: re-review the patch and close `S7A` only after the urgent queue excludes background/FYI rows and the gate count matches the canonical mobile action set.
