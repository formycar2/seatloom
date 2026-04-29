# Delivery: S6AB Shell Truth Fixes

| Field | Value |
|---|---|
| ID | MIRA-2026-04-28-s6ab-shell-truth-fixes-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-28 |

## 1. Scope completed

- Resolved Project Overview truth issues by removing fabricated health metrics.
- Extended Terminal auto-open behavior to the Inbox selection path.
- Corrected the Shortcut Help dialog to only list implemented shortcuts.
- Standardized touched UI surfaces to be consistently Chinese-first.

## 2. Changed files

- `ui/src/components/ProjectOverview.tsx`
- `ui/src/App.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`

## 3. Truth fixes applied

- **Project Overview**:
  - `unresolved review threads` now derived from `workItems.filter(wi => wi.status === 'InReview')`.
  - `budget alert state` now explicitly labeled as `未就绪 / 未索引` instead of a fake healthy state.
  - Localized remaining English shell headings (AUTHORITY HUB -> 项目权威源概览, etc.).
- **Inbox Selection**:
  - Updated `handleInboxSelect` to check session status; triggers `setIsTerminalOpen(true)` for `Running` or `InputRequired` sessions.
- **Shortcuts**:
  - Removed `Tab` from `ShortcutHelpDialog` as it is not currently implemented in `useGlobalShortcuts.ts`.

## 4. Build result

- `cd ui && pnpm build` => SUCCESS

## 5. Blockers

- none

## 6. Evidence paths

- `ui/src/components/ProjectOverview.tsx`
- `ui/src/App.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`
