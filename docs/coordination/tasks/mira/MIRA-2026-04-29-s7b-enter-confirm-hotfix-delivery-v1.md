# Delivery: S7B Enter Confirm Hotfix

| Field | Value |
|---|---|
| ID | MIRA-2026-04-29-s7b-enter-confirm-hotfix-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-29 |

## 1. Scope completed

- Repaired the `Enter` confirm path for the Supervisor Command Bar proposal surface.
- Resolved the issue where a disabled input field prevented keyboard confirmation of proposals.
- Preserved existing keyboard behaviors for initial proposal generation, inline editing, and modal closing.

## 2. Changed files

- `ui/src/components/SupervisorCommandBar.tsx`

## 3. Keyboard-path coverage

- **Global Listener**: Moved `Enter` and `Escape` handling to a `window` event listener while the modal is open. This ensures events are captured even when the main input is `disabled` by an active proposal.
- **Context-Aware Enter**:
  - If a valid proposal is open and NOT in edit mode: `Enter` triggers `handleConfirm()`.
  - If no proposal exists and input is present: `Enter` triggers `generateProposal()`.
  - If in inline edit mode: The global listener yields to the local input handler.
- **Escape Integrity**: `Escape` remains a reliable way to close the Command Bar at any stage.

## 4. Build result

- `cd ui && pnpm build` => **SUCCESS**

## 5. Blockers

- none

## 6. Evidence paths

- `ui/src/components/SupervisorCommandBar.tsx`
