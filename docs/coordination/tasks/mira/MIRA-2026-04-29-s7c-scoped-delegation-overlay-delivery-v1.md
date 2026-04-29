# Delivery: S7C Scoped Delegation Overlay

| Field | Value |
|---|---|
| ID | MIRA-2026-04-29-s7c-scoped-delegation-overlay-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-29 |

## 1. Scope completed

- Implemented the `DelegationOverlay` to capture temporary scoped responsibility for WorkItems.
- Added a `Delegate this work` entrypoint in `WorkItemDetail` identity section.
- Implemented `delegateWorkItem` store action to persist delegation state and emit audit events.
- Surfaced active delegation banners in `WorkItemDetail` showing delegate, scope, issuer, and expiry.
- Surfaced active delegation badges in `SeatDetail` for seats acting as delegates.
- Wired delegation events to the Timeline with human-readable labeling and distinctive styling (indigo).
- Ensured original owner seat identity is preserved (non-mutating).

## 2. Changed files

- `ui/src/components/DelegationOverlay.tsx` (New)
- `ui/src/components/WorkItemDetail.tsx` (Entrypoint and state visibility)
- `ui/src/components/SeatDetail.tsx` (Delegate state visibility)
- `ui/src/stores/useDataStore.ts` (State persistence and event emission)
- `ui/src/types/index.ts` (Data model extension)
- `ui/src/utils/display.ts` (Readable labeling)
- `ui/src/components/EventRow.tsx` (Timeline styling)

## 3. Overlay fields and validation

The overlay captures:
- **Delegate Seat**: Selected from current project seats.
- **Scope**: Multi-line description of delegated tasks.
- **Issuer**: Identity of the seat or human assigning the delegation.
- **Expiry**: Date-picker for delegation termination.
- **Authority Limits**: Specific constraints on the delegate's power.

Validation:
- Blocks confirmation if any mandatory field (Delegate, Scope, Expiry) is missing.
- Blocks "self-delegation" (Delegate Seat == Source Owner).

## 4. Delegated-state visibility coverage

- **WorkItem Detail**: A persistent amber banner appears below the header when a delegation is active, showing the delegate name, scope, and expiry date.
- **Seat Detail**: A new section "活动中的受托任务 (ACTIVE DELEGATIONS)" appears at the top of the detail pane when a seat is acting as a delegate, listing the WorkItem ID, original owner, and scope.

## 5. Timeline projection behavior

- **Event Type**: `WorkItemDelegated`
- **Visuals**: Distinctive indigo color badge.
- **Labeling**: "工作项委派"
- **Summary**: Human-readable restatement of the delegation source and target.

## 6. Build result

- `cd ui && pnpm build` => **SUCCESS**

## 7. Blockers

- none

## 8. Evidence paths

- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/stores/useDataStore.ts`
