# Acceptance: Mira Theme Preset System

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-theme-preset-system-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-system-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, ui, mira, theme, presets, visual-system |

## Verdict

**CONDITIONAL PASS**

Lyra accepts the direction and most of the implementation: the prototype now has a real preset-theme system, the shell has moved to a light-first baseline, the S5F copy-hygiene delta is closed, and the local build passes.

Two close-out items still block a full `PASS`:

1. the selector does not visibly show the preset names in the UI yet; it currently relies on `title` text only, which is too opaque for quick comparison and does not satisfy the packet's requirement to expose the three preset names; and
2. persisted theme values are not validated on rehydrate, so an invalid stored value does not normalize back to `paper-ledger` as required.

These are narrow finish-line issues, not a reason to reopen the preset-system direction.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-system-v1.md`
- `ui/src/styles/globals.css`
- `ui/src/styles/theme.ts`
- `ui/src/stores/useAppStore.ts`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/stores/useDataStore.ts`
- `ui/src/mockData.ts`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | Tokenized preset-based theme system replaces one-off recolor work | `ui/src/styles/globals.css`, `ui/tailwind.config.js`, `ui/src/styles/theme.ts` | PASS | The shell now uses semantic tokens and three preset blocks instead of a single hardcoded pass. |
| 2 | Exactly three presets exist: `paper-ledger`, `harbor-blueprint`, `sage-archive` | `ui/src/styles/theme.ts`, `ui/src/layouts/Sidebar.tsx` | PASS | The preset set matches the packet exactly. |
| 3 | `paper-ledger` is the default first-load theme | `ui/src/stores/useAppStore.ts:25`, `ui/src/styles/globals.css:8` | PASS | The default store value and default CSS layer both point to `paper-ledger`. |
| 4 | User can switch themes in-app | `ui/src/layouts/Sidebar.tsx:148`, `ui/src/stores/useAppStore.ts:37` | CONDITIONAL PASS | Switching exists, but the selector still hides preset identity behind tooltip-only names. |
| 5 | Selected theme persists across reload and invalid/missing values fall back safely | `ui/src/stores/useAppStore.ts:21` | CONDITIONAL PASS | Zustand persistence exists, but invalid stored values are not normalized back to `paper-ledger`. |
| 6 | Main shell no longer reads as dark UI in any preset | `ui/src/styles/globals.css:8`, `ui/src/layouts/Sidebar.tsx`, `ui/src/App.tsx` | PASS | The default shell tokens are now light-first and dark treatment is bounded to terminal-preview surfaces. |
| 7 | Dense Chinese text remains readable across repaired surfaces | `ui/src/styles/globals.css:98`, repaired UI surfaces under `ui/src/components/` and `ui/src/views/` | PASS | The typography baseline is materially improved and the active presets keep high-density Chinese copy legible. |
| 8 | Prompt-blocked and continuity surfaces remain emphasized under the new palette model | `ui/src/components/SessionDetail.tsx`, token changes in `ui/src/styles/globals.css` | PASS | Emphasis survives via tinted surfaces instead of reverting the app to a dark-shell bias. |
| 9 | S5F copy-hygiene carry-forward is closed | `ui/src/stores/useDataStore.ts:634`, `ui/src/mockData.ts:68`, `ui/src/mockData.ts:154` | PASS | The seeded prompt/continuity strings are now Chinese-first and the compatibility typo is fixed. |
| 10 | Local build passes after the rebase | `cd ui && pnpm build` | PASS | Lyra re-ran the build on 2026-04-28 and confirmed success. |

## Findings

| ID | Severity | Finding | Required action |
|---|---|---|---|
| UI-THEME-01 | High | The selector in `ui/src/layouts/Sidebar.tsx:150` does not visibly expose the preset names; users only see three anonymous bars unless they inspect the `title` tooltip. This does not meet the packet's explicit requirement to expose the three preset names and makes preference-based comparison harder than necessary. | Replace the inline preset array with a shared preset registry from `ui/src/styles/theme.ts`, and render each preset with a compact visible label in the selector. Keep the selector low-noise and Chinese-first, but preserve the exact preset IDs in the shared registry. |
| UI-THEME-02 | Medium | `ui/src/stores/useAppStore.ts:21` persists `themePreset`, but it does not validate or repair invalid stored values on rehydrate. A corrupted or stale local value can therefore leave the app in an undefined logical theme state instead of normalizing to `paper-ledger`. | Add a small validation layer in `ui/src/stores/useAppStore.ts` so missing or invalid stored values fall back to `paper-ledger` and the normalized value is what the UI reads after rehydrate. |

## Gate Decision

- **Theme preset direction:** **GO**
- **Packet closure:** **HOLD for close-out**
- **Follow-up mode:** one micro-packet only; do not reopen palette values or product behavior

## Follow-up Actions

- Mira: complete the close-out packet `docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-closeout-v1.md` and reply through tmux after `cd ui && pnpm build` passes.
- Lyra: re-review the close-out patch and, if both findings are resolved, convert this verdict to full closure in governance memory.
