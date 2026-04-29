# Delivery: S6B Shortcut Discovery

| Field | Value |
|---|---|
| ID | MIRA-2026-04-28-s6b-shortcut-discovery-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-28 |

## 1. Scope completed

- Implemented a compact `ShortcutHelpDialog` to list all supported keyboard shortcuts.
- Wired the shortcut discovery to `?` (Shift+/) and `F1` via the `useGlobalShortcuts` hook.
- Added a visual discovery hint in the Status Bar.
- Ensured the help surface is Chinese-first and non-intrusive.

## 2. Changed files

- `ui/src/components/ShortcutHelpDialog.tsx` (New)
- `ui/src/hooks/useGlobalShortcuts.ts` (Updated to handle help keys)
- `ui/src/layouts/StatusBar.tsx` (Updated to include help hint)
- `ui/src/App.tsx` (Integrated dialog and updated hook usage)

## 3. Shortcut list exposed

- `Esc`: Close details / forms
- `Cmd/Ctrl + K`: Project Switcher
- `Cmd/Ctrl + \``: Toggle Operator Console
- `?` or `F1`: Open this help dialog

## 4. Build result

- `cd ui && pnpm build` => SUCCESS

## 5. Blockers

- none

## 6. Evidence paths

- `ui/src/components/ShortcutHelpDialog.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/layouts/StatusBar.tsx`
