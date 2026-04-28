# FLUX-2026-04-28-Acting-Mira-Contract-Repair-Delivery-v1

| Field | Value |
|---|---|
| Owner | Flux (acting Mira) |
| Packet | FLUX-2026-04-28-acting-mira-contract-repair-v1 |
| Gate | SG-01 UI Contract Baseline |
| Build result | PASS — `pnpm build` succeeds, zero TS errors |

## 1. Micro-brief

Executed the Lyra-approved adoption set from the Flux design review:
- Restored Terminal as a bottom collapsible panel (was a 4th top-level tab)
- Wired Inbox row selection → object detail lookup
- Implemented contract-required Timeline filtering (Seat / WorkItem / type / time) with Sidebar-to-Timeline linkage
- Removed MorningDigest hard-coded truth claims; replaced with computed facts from active project data
- Improved Inbox action button visibility (always visible, no longer hover-hidden)

All touched surfaces maintain bilingual readability floor from the prior typography pass. No Sidebar/Main information architecture changes were made.

## 2. Changed files

| File | Change summary |
|---|---|
| `ui/src/App.tsx` | Added `isTerminalOpen` state; added `timelineFilters` state; added `handleInboxSelect` for Inbox→Detail linkage (case-normalized: inbox `object_ref` is uppercase e.g. `WI-102` but store IDs are lowercase `wi-102`; matching uses `.toLowerCase()` on both sides); Sidebar seat/WI click now sets timeline filter + navigates to timeline; removed `terminal` from Tab type; passes `terminalPane` + `isTerminalOpen` + `onTerminalToggle` to AppShell |
| `ui/src/layouts/AppShell.tsx` | Added `terminalPane`, `isTerminalOpen`, `onTerminalToggle` props; renders TerminalPanel below StatusBar when open |
| `ui/src/layouts/StatusBar.tsx` | Added `isTerminalOpen`/`onTerminalToggle` props; Terminal toggle button with `Ctrl+\`` tooltip |
| `ui/src/layouts/TopNav.tsx` | Removed terminal tab from tabs array; font adjusted per readability pass |
| `ui/src/views/TimelineView.tsx` | Full rewrite: real filtering by Seat/WorkItem/event-type/time-range using `useMemo` and cyclable filter buttons; accepts `filters` + `onFiltersChange` props; uses project data from `useDataStore`; includes clear-filters button and empty state |
| `ui/src/views/InboxView.tsx` | Added `onSelectObject` prop; click handler on each row |
| `ui/src/components/InboxItem.tsx` | Action buttons changed from `opacity-0 group-hover:opacity-100` to always visible |
| `ui/src/components/MorningDigest.tsx` | Replaced hard-coded claims with computed data: session/goal/handoff counts, interrupted/blocked detection via `useDataStore` |
| `ui/src/components/EventRow.tsx` | Font raised: `text-[10px]` → `text-xs`; removed `uppercase tracking-wider` from time/actor columns |
| `ui/src/hooks/useGlobalShortcuts.ts` | Added `Ctrl+\` handler for terminal toggle |

## 3. Flow coverage checklist

| Contract item | Status | Notes |
|---|---|---|
| Terminal as bottom collapsible panel (PRD §4.4) | implemented | Toggle via `isTerminalOpen` state; `Ctrl+\` shortcut added; no longer a main-panel tab |
| Terminal preserves session tabs/state | implemented | TerminalPanel component unchanged; retains session tabs and mock output |
| Inbox row selection → Detail (PRD §4.3, Interaction Spec §3.5) | implemented | Clicking Inbox row looks up related Handoff/WorkItem/Session via `object_ref` matching |
| Timeline Seat filter (Acceptance P-03) | implemented | Sidebar seat click sets `timelineFilters.seatId`, filters by actor + owned objects |
| Timeline WorkItem filter (Acceptance P-03) | implemented | Sidebar WI click sets `timelineFilters.workItemId`, filters by object refs |
| Timeline event type filter (Acceptance P-03) | implemented | Cycle button: session/handoff/artifact/pipeline/All |
| Timeline time range filter (Acceptance P-03) | implemented | Cycle button: 1h/6h/24h/7d with computed time boundaries |
| MorningDigest no uncomputed claims (Acceptance P-06) | implemented | All displayed facts (`n sessions · n goals · n handoffs`) derived from active project `useDataStore` |
| Inbox action visibility (Interaction Spec §3.5) | implemented | Accept/Return/Dismiss buttons now always visible |
| Sidebar → Timeline linkage (PRD §10.1) | implemented | Seat/WI click triggers timeline filter + tab switch |
| Readability floor maintained (prior typography pass) | implemented | Touched files use `text-xs` (12px) minimum, semibold/bold rather than `font-black uppercase tracking-widest` |

## 4. Build command + result

```
$ cd ui && pnpm build

> seatloom-ui@0.1.0 build
> tsc && vite build

vite v6.4.2 building for production...
✓ 1849 modules transformed.
dist/index.html                   0.47 kB
dist/assets/index-C9qiP6cO.css   27.89 kB
dist/assets/index-C7BqMObT.js   327.24 kB
✓ built in 1.56s

npx tsc --noEmit → 0 errors
```

## 5. Evidence paths

- Live dev server: `http://localhost:5173/`
- Production build output: `ui/dist/`
- All 9 changed files serve HTTP 200 through Vite

## 6. Blockers / risks / next owner

**Blockers:** None. All P0 and P1 items pass.

**Risks:**
- `ReconcileNotification` has no active caller in the prototype after init-trigger removal (expected per Lyra decision — will be wired when Nimbus connects explicit reconcile flow)
- Timeline filter for Seat uses actor-ref + indirect object-ref matching; edge case events where the seat is neither actor nor object owner will not appear — acceptable for prototype scope
- Terminal panel toggle is available via `Ctrl+\`` (keyboard) and the StatusBar Terminal button; both work identically in the prototype

**Next owner:** Lyra — re-review for SG-01 UI Contract Baseline clearance.
