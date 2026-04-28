# 2026-04-28 Flux Feedback Recovery

| Field | Value |
|---|---|
| Owner | Lyra |
| Source seat | Flux-Quality&Ops-seatloom |
| Recovery mode | Delivery artifact + tmux seat capture + spot code inspection |
| Status | Recovered, not accepted |
| Gate impact | `SG-01 UI Contract Baseline` remains pending Lyra acceptance review |

## 1. Decision

Flux delivery has been recovered into the Lyra review pipeline.

This is **not yet an accepted milestone**. Flux claims the required P0/P1 repair set is complete, but Lyra found one likely contract-impacting defect and one self-report inconsistency that must be resolved during acceptance review.

## 2. Flux claimed delivery set

Flux reports the following items as completed:

1. Terminal restored to a bottom collapsible panel instead of a top-level tab.
2. Inbox row selection wired to open the related detail object.
3. Timeline filtering implemented across Seat / WorkItem / event type / time range.
4. Sidebar seat/workitem selection linked to Timeline filters.
5. MorningDigest hard-coded claims replaced with computed project data.
6. Inbox action buttons made always visible.

Flux also reported:

- `npx tsc --noEmit` -> zero errors
- `pnpm build` -> pass
- Live prototype available at `http://localhost:5173/`
- Delivery artifact path: `docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-delivery-v1.md`

## 3. Lyra spot verification

### Verified in code

- `ui/src/App.tsx` removes `terminal` from the top-level tab contract and adds bottom-panel state management.
- `ui/src/layouts/AppShell.tsx` renders the terminal pane below the main shell/status area.
- `ui/src/layouts/StatusBar.tsx` contains a visible terminal toggle button.
- `ui/src/hooks/useGlobalShortcuts.ts` adds a terminal toggle shortcut.
- `ui/src/views/TimelineView.tsx` now accepts filter state and exposes filter controls.

### Confirmed risk

- **Inbox -> Detail lookup likely fails on current mock/store IDs.**
- In `ui/src/App.tsx`, `handleInboxSelect` matches `item.object_ref` directly against entity IDs.
- In `ui/src/stores/useDataStore.ts`, active project inbox references use uppercase forms such as `WI-102` and `SES-102`, while actual IDs are lowercase forms such as `wi-102` and `ses-102`.
- This means the claimed Inbox row -> Detail repair is **not yet trustworthy** until identifier normalization is added and rechecked.

### Self-report inconsistency

- Flux delivery notes say the terminal toggle button is "not visually rendered yet".
- tmux seat output says the StatusBar already includes the toggle button.
- Current code inspection supports the tmux claim, not the delivery-note caveat.
- This does not block the gate by itself, but it lowers confidence in the delivery narrative and should be cleaned up in the next delta note.

## 4. Recovery disposition

| Item | Disposition |
|---|---|
| Flux feedback | Recovered |
| Flux delivery artifact | Logged |
| SG-01 clearance | Not granted |
| Nimbus handoff | Not yet |
| Next state | Lyra acceptance review / Flux delta fix |

## 5. Required next move

Before this delivery can be accepted for `SG-01`:

1. Normalize Inbox `object_ref` lookup for WorkItem / Session / Handoff targets.
2. Recheck the Inbox click -> Detail flow against active `useDataStore` data, not only static mock assumptions.
3. Publish a short delta note correcting the terminal-toggle reporting inconsistency.

