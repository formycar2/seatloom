# Delivery: S7A Monitor Truth Fix

| Field | Value |
|---|---|
| ID | MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-28 |

## 1. Scope completed

- Resolved monitoring gaps in `Mobile Overview` by surfacing explicit counts for approvals, blocked prompts, and pending handoffs.
- Implemented a deterministic urgency filter for `Mobile Inbox` to ensure action-needed items appear by default.
- Maintained Chinese-first body copy and theme-token compliance.
- Verified build remains green after fixes.

## 2. Counting rules

- **Pending approvals / gates**: Count of items in `inboxItems` with types `待处理交接`, `验收待决策`, or `输入请求`.
- **Prompt blocked**: Count of active sessions where `prompt_state` is present.
- **Pending handoffs**: Count of project handoffs where status is neither `Completed` nor `Expired`.
- **Emergency alerts**: Explicit count of `Critical` priority items from the project inbox.

## 3. Urgent filter rule

The `Mobile Inbox` now uses a two-tier deterministic filter:
- **Tier 1 (Priority)**: Any item with `Critical` or `Normal` priority (Normal is treated as actionable in this bounded seed).
- **Tier 2 (Type)**: Any item with types `待处理交接`, `验收待决策`, `输入请求`, or `会话恢复`.
- **Sorting**: Items are sorted to put `Critical` priority at the top, regardless of type.

## 4. Changed files

- `ui/src/views/MobileCompanionView.tsx`

## 5. Build result

- `cd ui && pnpm build` => SUCCESS

## 6. Blockers

- none

## 7. Evidence paths

- `ui/src/views/MobileCompanionView.tsx`
