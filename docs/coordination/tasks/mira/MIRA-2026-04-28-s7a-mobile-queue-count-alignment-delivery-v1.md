# Delivery: S7A Mobile Queue + Count Alignment

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| packet_ref | `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-v1.md` |
| takeover_reason | Mira seat offline; Lyra completed the bounded fix directly to close the active `S7A` hold |
| tags | ui, mobile, companion, queue, count, truth, s7a, fix |

## 1. Scope completed

Lyra completed the remaining `S7A` truth-alignment fix directly in `ui/src/views/MobileCompanionView.tsx` after the Mira seat went offline again.

The fix stayed inside the issued packet boundary:

- tightened the `pending approvals / gates` count to the seed-backed action set,
- removed background and FYI rows from the urgent mobile queue by default, and
- made the queue ranking deterministic without changing the accepted mobile shell or theme system.

## 2. Approval / gate count rule

The mobile `待审批 / 闸门` card now counts only the canonical action-bearing Inbox types listed in the active fix packet:

- `待处理交接`
- `需要验收`
- `需确认范围`
- `验收待决策`
- `输入请求`

The count no longer relies on an incomplete subset and continues to exclude FYI-only rows such as:

- `背景记录`
- `已记录决策`
- `需要同步`

Implementation shape:

- `MOBILE_APPROVAL_GATE_TYPES` is defined once and reused for the monitor card.

## 3. Urgent queue rule

The `紧急收件箱 (URGENT ONLY)` list now includes only:

- any `Critical` Inbox item, or
- an item whose type belongs to the bounded mobile action set:
  - `待处理交接`
  - `需要验收`
  - `需确认范围`
  - `验收待决策`
  - `输入请求`
  - `会话恢复`
  - `待补产物`

This removes the earlier leakage where every `Normal` item could appear even when it was only background context.

Rows such as `背景记录`, `已记录决策`, and `需要同步` are therefore excluded unless they become `Critical` by explicit priority.

## 4. Ranking rule

Queue ordering is now deterministic:

1. `Critical` items first
2. then non-critical action-needed rows
3. newest timestamp first inside the same priority band

The timestamp comparison uses the seeded `YYYY-MM-DD HH:mm` format already present in `useDataStore`, so lexical descending comparison stays stable for the prototype without adding parsing ambiguity.

## 5. Changed files

- `ui/src/views/MobileCompanionView.tsx`

## 6. Build result

Validation run:

```bash
cd ui && pnpm build
```

Result:

- `SUCCESS`

## 7. Blockers

- none

## 8. Evidence paths

- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/stores/useDataStore.ts`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-v1.md`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-monitor-truth-fix-acceptance.md`
