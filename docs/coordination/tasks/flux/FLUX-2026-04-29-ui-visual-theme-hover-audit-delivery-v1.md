# UI Visual Theme + Hover Audit — Delivery Artifact

| Field | Value |
|---|---|
| Packet | FLUX-2026-04-29-ui-visual-theme-hover-audit-v1 |
| Status | Complete |
| Auditor | Flux |
| Date | 2026-04-29 |
| Scope | Read-only visual QA, no code patching |
| Method | Static code inspection + `cd ui && pnpm build` |

---

## 1. Scope Completed

- Read all 10 input documents listed in the task packet
- Inspected 22 UI source files across layouts, components, views, stores, styles, and config
- Ran `cd ui && pnpm build` — **PASS** (1.72s, 0 TS errors)
- Could not run a live preview (`localhost:5173` was not available during this pass); all findings are code-backed
- No product code was patched

---

## 2. Runtime / Code Inspection Method

| Step | Method | Result |
|---|---|---|
| UI build | `cd ui && pnpm build` | PASS — 1554 modules, 0 TS errors |
| Live preview | `http://localhost:5173/` | Not running; headless fallback used |
| Screenshot evidence | Screenshot capture | Not captured (no live preview) |
| Static code inspection | Read all 22 source files | Complete |

All defects below are reproduced from source code with exact file paths and line numbers.

---

## 3. Visual Defect Matrix

### D-01 — `sl-clickable:hover *` Forces `!important text-primary` on All Descendants

| Field | Value |
|---|---|
| ID | D-01 |
| Severity | **P1** |
| Category | `token inconsistency` |
| Affected presets | All three (paper-ledger, harbor-blueprint, sage-archive) |
| Affected surface | Sidebar session rows, any component nesting `sl-clickable` around status-bearing children |
| Reproduction path | Hover over a Sidebar session row with a blocked/error status dot (e.g., session with `status: 'blocked'`) |
| Expected behavior | Hover changes the background of the row; status dot color stays semantic (amber for warning, red for error) |
| Actual behavior | `.sl-clickable:hover * { @apply !text-primary; }` in `globals.css:129` forces all descendant text to primary teal, overriding status dot colors on hover |
| Suspected file target(s) | `ui/src/styles/globals.css` (lines 122–130), `ui/src/layouts/Sidebar.tsx` (lines 43, 74, 126) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-02 — Hardcoded Tailwind Color Classes in EventRow.tsx `getTypeColor()` (Complete Theme Bypass)

| Field | Value |
|---|---|
| ID | D-02 |
| Severity | **P1** |
| Category | `theme`, `token inconsistency` |
| Affected presets | All three |
| Affected surface | Timeline event rows — Artifact events, WorkItemDelegated events, generic WorkItem events |
| Reproduction path | Switch theme to `harbor-blueprint` or `sage-archive`, observe Timeline event rows; Artifact badge stays violet, WorkItemDelegated stays indigo |
| Expected behavior | Event-type badge colors adapt to active theme palette via CSS variables |
| Actual behavior | Lines 42–47 use `text-violet-600`, `text-indigo-600`, `text-orange-500` — hardcoded Tailwind colors that do not respond to theme changes |
| Suspected file target(s) | `ui/src/components/EventRow.tsx` (lines 40–47) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-03 — Hardcoded Template Badge Colors in ArtifactDetail.tsx (Complete Theme Bypass)

| Field | Value |
|---|---|
| ID | D-03 |
| Severity | **P1** |
| Category | `theme`, `token inconsistency` |
| Affected presets | All three |
| Affected surface | Artifact detail header — template family badge (T1–T7) |
| Reproduction path | Open any artifact in detail view, switch to a non-default theme; badge background and text colors remain hardcoded (slate, cyan, amber, rose, emerald, violet, sky) |
| Expected behavior | Template badge colors should derive from theme semantic tokens (e.g., drifted, primary, warning, etc.) |
| Actual behavior | `toneByTemplate` on lines 18–26 uses literal Tailwind classes (`bg-slate-100`, `text-slate-700`, etc.) |
| Suspected file target(s) | `ui/src/components/ArtifactDetail.tsx` (lines 18–26) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-04 — Low-Contrast Hover State on Secondary Button (`hover:bg-accent` on `paper-ledger`)

| Field | Value |
|---|---|
| ID | D-04 |
| Severity | **P2** |
| Category | `theme`, `hover` |
| Affected presets | `paper-ledger` (primary risk), `sage-archive` (secondary risk) |
| Affected surface | SessionDetail secondary button ("人工接管 / TAKEOVER"), InboxView clear-search button |
| Reproduction path | Switch to `paper-ledger`, hover over the secondary button in SessionDetail or the clear-X in InboxView search |
| Expected behavior | Hover background is visibly darker/lighter than the resting state, providing clear feedback |
| Actual behavior | `hover:bg-accent` resolves to `--primary-tint` (#DCECEF on paper-ledger), which is nearly indistinguishable from the `--surface-subtle` background (#F0E8DC). The effective contrast change is ~1.3:1 — imperceptible |
| Suspected file target(s) | `ui/src/components/SessionDetail.tsx` (line 133), `ui/src/views/InboxView.tsx` (line 54) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-05 — Low-Contrast Icon Color in ArtifactChip (`text-accent` on Light Background)

| Field | Value |
|---|---|
| ID | D-05 |
| Severity | **P2** |
| Category | `theme`, `contrast` |
| Affected presets | All three (paper-ledger worst: `--primary-tint` = #DCECEF on #FFFDF8) |
| Affected surface | ArtifactChip file icon in Inbox, Timeline, and WorkItem detail |
| Reproduction path | Switch to `paper-ledger`, inspect an artifact chip in Inbox or Timeline; icon color is `text-accent` (= `--primary-tint`) which is very pale |
| Expected behavior | Icon color provides ≥ 3:1 contrast against the background per WCAG AA for non-text symbols |
| Actual behavior | On `paper-ledger`, `--primary-tint` (#DCECEF, ~90% lightness) on `--surface` (#FFFDF8, ~98% lightness) has an estimated contrast ratio of ~1.8:1, **failing WCAG AA non-text contrast** |
| Suspected file target(s) | `ui/src/components/ArtifactChip.tsx` (line 14) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-06 — Opacity-Only Hover on Primary/Supervisor/Stop Buttons (Low Discoverability)

| Field | Value |
|---|---|
| ID | D-06 |
| Severity | **P2** |
| Category | `hover`, `state semantics` |
| Affected presets | All three |
| Affected surface | SessionDetail action buttons (lines 132, 136, 140), DelegationOverlay CTA (line 183) |
| Reproduction path | Hover over "确认 / APPROVE", "SUPERVISOR 辅助", or "停止 / STOP" buttons in SessionDetail |
| Expected behavior | Hover provides a visible background or color transition signaling interactivity |
| Actual behavior | `hover:opacity-90` reduces opacity by 10% only. No color shift, no brightness change. This is the weakest possible hover affordance and is not clearly distinguishable from the resting state at a glance |
| Suspected file target(s) | `ui/src/components/SessionDetail.tsx` (lines 132, 136, 140), `ui/src/components/DelegationOverlay.tsx` (line 183) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-07 — Low-Contrast Text in Inactive Sidebar Session Items (Fails WCAG AA at 10px)

| Field | Value |
|---|---|
| ID | D-07 |
| Severity | **P1** |
| Category | `typography`, `contrast` |
| Affected presets | All three (paper-ledger worst-case) |
| Affected surface | Sidebar session list — inactive session status label and runtime label |
| Reproduction path | Look at any inactive sidebar session in `paper-ledger` theme; inspect status text and runtime text |
| Expected behavior | All text ≥ 10px should meet WCAG AA 4.5:1 contrast |
| Actual behavior | Lines 88–91: status uses `text-text-secondary opacity-70` (= #5E6A75 at 70% over #FFFDF8 ≈ 2.8:1). Runtime uses `opacity-60` on `text-text-muted` (= #8C97A3 at 60% ≈ 1.5:1). Both **fail WCAG AA** for 10px body text |
| Suspected file target(s) | `ui/src/layouts/Sidebar.tsx` (lines 88–91) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-08 — Hardcoded `text-green-400/90` Terminal Preview (Theme-Independent, Acceptable but Not Extensible)

| Field | Value |
|---|---|
| ID | D-08 |
| Severity | **P2** |
| Category | `token inconsistency` |
| Affected presets | All three |
| Affected surface | SessionDetail terminal preview panel |
| Reproduction path | Open any session detail with terminal content; inspect text color |
| Expected behavior | Terminal text color should ideally be theme-extensible via a CSS variable |
| Actual behavior | `text-green-400/90` is hardcoded in `SessionDetail.tsx:120`. While it currently produces acceptable contrast (green #4ade80 on dark #20262E ≈ 9:1), it cannot be customized per-theme. Any future terminal theme change (e.g., amber terminal, white terminal) would require manual component edits |
| Suspected file target(s) | `ui/src/components/SessionDetail.tsx` (line 120) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-09 — Low-Discoverability Hover on Sidebar Icon-Only Buttons

| Field | Value |
|---|---|
| ID | D-09 |
| Severity | **P2** |
| Category | `hover` |
| Affected presets | All three |
| Affected surface | Sidebar "Add Seat" (+) button and "Add WorkItem" (+) button |
| Reproduction path | Hover over the small icon-only plus buttons in Sidebar header |
| Expected behavior | Icon-only action buttons should have a clear hover background or border highlight |
| Actual behavior | Lines 31, 106: `hover:text-primary transition-colors` only changes icon color, no background fill or border change. On the light `--surface-subtle` background, a 12px icon color shift is minimally discoverable |
| Suspected file target(s) | `ui/src/layouts/Sidebar.tsx` (lines 31, 106) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-10 — Low-Contrast Object Label in EventRow (`text-primary` at 80% Opacity)

| Field | Value |
|---|---|
| ID | D-10 |
| Severity | **P2** |
| Category | `typography`, `contrast` |
| Affected presets | `paper-ledger` (fails), `harbor-blueprint` (passes), `sage-archive` (passes) |
| Affected surface | Timeline event row — object reference label (line 65) |
| Reproduction path | Switch to `paper-ledger` theme, hover or inspect an EventRow with an object label; `text-primary font-mono opacity-80` renders at ~4.2:1 on the surface background |
| Expected behavior | Text at 10px should achieve ≥ 4.5:1 contrast |
| Actual behavior | `text-primary/80` over `--surface` (#FFFDF8) yields ~4.2:1 on paper-ledger, **failing WCAG AA** for 10px text |
| Suspected file target(s) | `ui/src/components/EventRow.tsx` (line 65) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-11 — No Hover State on MobileCompanionView List Items (Desktop Inconsistency)

| Field | Value |
|---|---|
| ID | D-11 |
| Severity | **P2** |
| Category | `hover`, `state semantics` |
| Affected presets | All three |
| Affected surface | MobileCompanionView item rows |
| Reproduction path | Open MobileCompanionView on desktop; hover over any list item |
| Expected behavior | Clickable list items should have a hover background change |
| Actual behavior | Line 212: `active:bg-primary/5 transition-colors cursor-pointer` provides only an active (mouse-down) state; no `hover:` class is defined. On desktop, there is no hover feedback |
| Suspected file target(s) | `ui/src/views/MobileCompanionView.tsx` (line 212) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-12 — Hardcoded `text-text-primary` Transition Speed vs. `text-text-secondary` in InboxItem (Minor Inconsistency)

| Field | Value |
|---|---|
| ID | D-12 |
| Severity | **P2** |
| Category | `hover` |
| Affected presets | All three |
| Affected surface | Inbox item text color transition |
| Reproduction path | Inspect InboxItem text elements and their CSS transition properties |
| Expected behavior | Hover transitions should use consistent timing across all state properties |
| Actual behavior | InboxItem text elements use `transition-colors duration-200` for text color while background uses `transition-all`. Other components use `transition-colors` with no explicit duration. This creates a perceptibly faster text-color fade than background fade on hover |
| Suspected file target(s) | `ui/src/components/InboxItem.tsx` (line 46) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-13 — Dual Token Naming System (`text-ink*` vs. `text-text*`) Creates Architectural Drift

| Field | Value |
|---|---|
| ID | D-13 |
| Severity | **P2** |
| Category | `token inconsistency` |
| Affected presets | All three |
| Affected surface | System-wide — any file using `text-ink`, `text-ink-soft`, or `text-ink-faint` |
| Reproduction path | Grep for `text-ink` across the codebase |
| Expected behavior | One consistent naming convention for semantic text color tokens |
| Actual behavior | The codebase uses **two parallel naming systems** that resolve to the same CSS variables: `text-ink` / `text-ink-soft` / `text-ink-faint` and `text-text-primary` / `text-text-secondary` / `text-text-muted`. Both are defined in `tailwind.config.js` (lines 47–56). Files mixing both: Sidebar.tsx, SeatDetail.tsx, WorkItemDetail.tsx, ProjectOverview.tsx, SupervisorCommandBar.tsx, MobileCompanionView.tsx, InboxItem.tsx, EventRow.tsx, and others. This creates architectural drift and makes it harder to audit coverage |
| Suspected file target(s) | `ui/tailwind.config.js` (lines 47–56), `ui/src/layouts/Sidebar.tsx`, `ui/src/components/SeatDetail.tsx`, `ui/src/components/ProjectOverview.tsx`, and 6+ others |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-14 — `disabled:grayscale` Removes Semantic Color Identity from Buttons

| Field | Value |
|---|---|
| ID | D-14 |
| Severity | **P2** |
| Category | `state semantics` |
| Affected presets | All three |
| Affected surface | SessionDetail Supervisor Assist button, DelegationOverlay CTA when disabled |
| Reproduction path | Set the Supervisor Assist button to disabled state (e.g., session is not in a prompt-blocked state); observe button color |
| Expected behavior | Disabled state should communicate *why* the action is unavailable (via tooltip, label change, or icon), not just reduce visibility |
| Actual behavior | `disabled:grayscale` in SessionDetail.tsx:136 and DelegationOverlay.tsx:183 strips the button's primary color identity entirely, rendering it as a gray box. The semantic difference between "primary action, currently disabled" and "secondary action" is visually erased |
| Suspected file target(s) | `ui/src/components/SessionDetail.tsx` (line 136), `ui/src/components/DelegationOverlay.tsx` (line 183) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-15 — Hardcoded Budget Progress Bar Width in SessionDetail

| Field | Value |
|---|---|
| ID | D-15 |
| Severity | **P2** |
| Category | `token inconsistency` |
| Affected presets | All three |
| Affected surface | SessionDetail budget meter progress bar |
| Reproduction path | Inspect SessionDetail budget section; the progress bar fill has a hardcoded `style={{ width: '45%' }}` |
| Expected behavior | Progress bar width should be driven by a computed value from session budget data |
| Actual behavior | Line 356: `style={{ width: '45%' }}` is a static prototype placeholder. The bar does not reflect actual budget consumption. Additionally, no `transition-all` or animation class is applied, so changes (when wired) would not animate smoothly |
| Suspected file target(s) | `ui/src/components/SessionDetail.tsx` (line 356) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

### D-16 — `text-accent` (Primary Tint) on Event Row Object Label (Contrast Risk)

| Field | Value |
|---|---|
| ID | D-16 |
| Severity | **P2** |
| Category | `contrast`, `theme` |
| Affected presets | `paper-ledger` (fails), `sage-archive` (fails), `harbor-blueprint` (marginal) |
| Affected surface | Timeline event row — object reference label |
| Reproduction path | Inspect the object reference label on any EventRow; `text-primary font-mono opacity-80` on line 65 |
| Expected behavior | Object label should have ≥ 4.5:1 contrast on all themes |
| Actual behavior | `text-primary/80` on `--surface`: paper-ledger = ~4.2:1 (fails AA), sage-archive = ~3.9:1 (fails AA). Harbor-blueprint with darker primary (#245A7A) may pass at ~5.1:1 at 80% opacity |
| Suspected file target(s) | `ui/src/components/EventRow.tsx` (line 65) |
| Evidence path(s) | `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` |

---

## 4. Triage Summary

### 4.1 Immediate Bug Fixes (P1 — Fix in Next Cycle)

| ID | Title | Type |
|---|---|---|
| D-01 | `sl-clickable:hover *` `!important` cascade overriding child status colors | Token-layer fix |
| D-02 | Hardcoded Tailwind colors in `EventRow.getTypeColor()` bypass theme | Component-level state fix |
| D-03 | Hardcoded template badge colors in `ArtifactDetail.toneByTemplate` bypass theme | Token-layer fix |
| D-07 | Low-contrast inactive sidebar session text (opacity × small text) | Typography/system cleanup |
| D-10 | Low-contrast object label in EventRow (`text-primary/80` at 10px) | Typography/system cleanup |

### 4.2 Visual-System Fixes (P2 — Fix in Next UI Pass)

| ID | Title | Type |
|---|---|---|
| D-04 | Low-contrast hover on secondary button (`hover:bg-accent`) | Token-layer adjustment |
| D-05 | Low-contrast icon in ArtifactChip (`text-accent` on light bg) | Token-layer adjustment |
| D-06 | Opacity-only hover on primary/stop/supervisor buttons | Component-level state fix |
| D-09 | Low-discoverability icon-only button hover in Sidebar | Component-level state fix |
| D-13 | Dual naming system (`text-ink*` vs. `text-text*`) causes architectural drift | Typography/system cleanup |
| D-14 | `disabled:grayscale` erases semantic button identity | Component-level state fix |

### 4.3 Optional Polish Items (P2 — Non-blocking)

| ID | Title | Type |
|---|---|---|
| D-08 | Hardcoded terminal text color not theme-extensible | Token-layer adjustment |
| D-11 | No hover state on MobileCompanionView items | Component-level state fix |
| D-12 | Inconsistent transition speeds in InboxItem text vs. background | Typography/system cleanup |
| D-15 | Hardcoded budget bar width (prototype placeholder) | Token-layer adjustment |
| D-16 | `text-primary/80` contrast risk in EventRow (partial overlap with D-10) | Typography/system cleanup |

---

## 5. Fix-Ready Recommendation Set

### R-01 — Fix `sl-clickable` CSS to Remove `!important` Cascade

**File:** `ui/src/styles/globals.css:128–129`
**Type:** Token-layer adjustment
**Action:** Remove `.sl-clickable:hover * { @apply !text-primary; }`. Replace with a targeted selector that only applies to direct child text elements, or remove entirely and let child components manage their own hover text colors. The `sl-clickable` background highlight (`bg-primary/5`) is sufficient to signal hover state without forcing child text color.

---

### R-02 — Replace Hardcoded Colors in `EventRow.getTypeColor()` with Theme Tokens

**File:** `ui/src/components/EventRow.tsx:40–47`
**Type:** Component-level state fix
**Action:** Map the three hardcoded color schemes to theme tokens:
- `Artifact*` → `text-status-drifted bg-status-drifted/10 border-status-drifted/20` (drifted violet maps semantically to "artifact evidence")
- `WorkItemDelegated` → `text-status-drifted bg-status-drifted/10 border-status-drifted/20`
- Generic `WorkItem*` → `text-warning bg-warning/10 border-warning/20`
All colors will then derive from CSS variables defined per theme.

---

### R-03 — Replace Hardcoded Template Badge Colors with Theme Tokens

**File:** `ui/src/components/ArtifactDetail.tsx:18–26`
**Type:** Token-layer adjustment
**Action:** Map each T-family badge to an existing semantic color or define a new CSS variable for template-family identity:
```typescript
const toneByTemplate: Record<Artifact['template'], string> = {
  T1: 'bg-status-done/10 text-status-done border-status-done/20',      // Authority Doc → done (stable reference)
  T2: 'bg-status-drifted/10 text-status-drifted border-status-drifted/20', // Role Profile → drifted
  T3: 'bg-primary/10 text-primary border-primary/20',                  // Task Packet → primary
  T4: 'bg-status-warning/10 text-status-warning border-status-warning/20', // Review → warning
  T5: 'bg-status-active/10 text-status-active border-status-active/20',    // Acceptance → active/success
  T6: 'bg-surface-subtle text-ink-soft border-border',                 // Daily Memory → neutral
  T7: 'bg-status-error/10 text-status-error border-status-error/20',     // Governance → error (policy)
};
```

---

### R-04 — Fix Low-Contrast Inactive Sidebar Session Text

**File:** `ui/src/layouts/Sidebar.tsx:88–91`
**Type:** Typography/system cleanup
**Action:** Replace `opacity-70` and `opacity-60` with fixed color tokens that maintain contrast:
- Status label: use `text-ink-soft` (without opacity) for inactive items
- Runtime label: use `text-ink-faint` (without opacity) for inactive items
In the active state, keep the current `text-surface/80` and `text-surface/60` approach (contrast is good on the dark active background).

---

### R-05 — Standardize Naming: `text-ink*` → `text-text*`

**File:** `ui/tailwind.config.js:47–56`, all component files using `text-ink*`
**Type:** Typography/system cleanup
**Action:** Decide on one naming convention and deprecate the other. Recommended: keep `text-text-primary`, `text-text-secondary`, `text-text-muted` (these follow the semantic naming pattern used by `primary`, `secondary`, `muted`, `accent`). Remove the `ink` alias keys from `tailwind.config.js` and update all component files to use `text-text-*` classes. Run a project-wide grep-and-replace for `text-ink`, `text-ink-soft`, `text-ink-faint`.

---

### R-06 — Replace `hover:opacity-90` with `hover:brightness-110` on Action Buttons

**File:** `ui/src/components/SessionDetail.tsx` (lines 132, 136, 140), `ui/src/components/DelegationOverlay.tsx` (line 183)
**Type:** Component-level state fix
**Action:** Change `hover:opacity-90` to `hover:brightness-110` on primary/stop/supervisor buttons. For the secondary button (line 133), change `hover:bg-accent` to `hover:bg-surface-subtle` or `hover:bg-border` (a slightly darker shade of the resting background). Remove `disabled:grayscale` and replace with `disabled:opacity-50` plus an `aria-label` or `title` explaining the disabled reason.

---

### R-07 — Add Hover Background to Sidebar Icon-Only Buttons

**File:** `ui/src/layouts/Sidebar.tsx` (lines 31, 106)
**Type:** Component-level state fix
**Action:** Add `hover:bg-primary/10 hover:rounded-full` to the `Plus` button className to provide a visible circular highlight on hover.

---

### R-08 — Use `text-primary` (Not `text-accent`) for ArtifactChip Icon

**File:** `ui/src/components/ArtifactChip.tsx` (line 14)
**Type:** Token-layer adjustment
**Action:** Change `text-accent` to `text-primary/70` or `text-primary`. The `text-primary` color (e.g., #1F6B75 on paper-ledger) provides sufficient contrast against the surface background while remaining visually distinct from regular text.

---

### R-09 — Add `hover:bg-primary/5` to MobileCompanionView List Items

**File:** `ui/src/views/MobileCompanionView.tsx` (line 212)
**Type:** Component-level state fix
**Action:** Add `hover:bg-primary/5` to the `className` of the list item container to provide consistent desktop hover feedback matching the rest of the app.

---

### R-10 — Wire Budget Progress Bar to Actual Data and Add Transition

**File:** `ui/src/components/SessionDetail.tsx` (line 356)
**Type:** Token-layer adjustment
**Action:** Replace `style={{ width: '45%' }}` with a computed width from session budget data (e.g., `style={{ width: `${(session.budgetUsed / session.budgetLimit) * 100}%` }}`). Add `transition-all duration-300` to animate changes.

---

## 6. Validation Commands and Results

```bash
cd ui && pnpm build
```
**Result:** PASS — built in 1.72s, 1554 modules transformed, 0 TypeScript errors.

```bash
cd ui && npx tsc --noEmit
```
**Result:** Not explicitly run in this pass. The build command includes `tsc` (via `tsc && vite build`), so the type-check result is PASS by implication.

```bash
# Contrast calculations (WCAG AA 4.5:1 threshold)
# paper-ledger text-ink-soft (#5E6A75) on surface (#FFFDF8) @ 70% opacity: ~2.8:1 → FAIL
# paper-ledger text-ink (#1F2A37) on surface (#FFFDF8): ~14.5:1 → PASS
# paper-ledger primary-tint (#DCECEF) on surface (#FFFDF8): ~1.8:1 → FAIL (non-text icon)
# paper-ledger text-primary/80 (#1F6B75 @ 80%) on surface (#FFFDF8): ~4.2:1 → FAIL (10px text)
```

---

## 7. Evidence Paths

| File | Description |
|---|---|
| `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/build.log` | UI build output (SUCCESS, 1.72s) |
| `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/inspection-log.txt` | Code inspection log with defect summary |
| `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md` | This delivery artifact |

---

## 8. Recommended Next Owner

**Mira** (UI implementation seat) is the recommended next owner for all P1 fix recommendations (R-01 through R-05). These are component-level changes that require careful styling work to ensure no regression in the accepted SG-01 baseline.

**Flux** should re-verify after Mira's fixes via a bounded read-only QA pass using the same packet format.

---

*Audit completed 2026-04-29. All findings are code-backed. No live preview was available; runtime-based defects (e.g., cursor behavior, animation smoothness) are noted where detectable from code but may require live verification for full confirmation.*
