# Task: NavRail Logo + ProjectSwitcher Z-Index Fix (SG-A §A2)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | MIRA-2026-05-09-v02-navrail-logo-and-zindex-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-05-09 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (commit a5998c1), `docs/coordination/reviews/2026-05-09-aegis-mvp-gap-to-tmux-replacement.md` (commit 0cc401b), `ui/src/components/NavRail.tsx`, `ui/src/components/ProjectSwitcher.tsx`, `ui/src/app-v2/components/Avatar.tsx` (SeatLoomLogo component) |
| tags | mira, ui, navrail, logo, z-index, v0.0.1, SG-A |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Parallel with A1/A3/A4. All SG-A packets (A1+A2+A3+A4) must pass Flux verification before SG-B dispatch. |

## Context

Two UI bugs surfaced during Mira's v01 delivery:

1. **NavRail Logo missing**: The SeatLoom brand mark was added in commit `31367a0` but reverted in `b11d878` due to scope violations. The Logo itself is valid and needed — it just needs to be re-added **without** the scope violations (i18n changes, unrelated refactors).

2. **ProjectSwitcher dropdown clipped**: When the user clicks the ProjectSwitcher in the NavRail, the dropdown menu is clipped by an ancestor's `overflow: hidden` style (likely in `AppShell.tsx` or a parent container). The dropdown should render fully visible above all other UI elements.

---

## Prerequisite — Close v01 Before Starting A2

Before beginning work on A2, you must finish the v01 delivery doc that is still outstanding:

**Required artifact**: `docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-delivery-v1.md`

Must contain:
- Complete commit chain (578ff7c §A, 4651bb4 §B, 2f83624 §C) with `git show --stat` for each.
- Three validation segments with **verbatim tool output** (no summaries): `tsc --noEmit`, `pnpm build`, and the on-screen observation Mr. Zhang asked for.
- Scope-drift incident recorded faithfully in three parts: what happened, what was reverted (`b11d878`), what the corrective behavior is.

A2 may not be dispatched or accepted until v01 delivery is posted and Aegis closes v01. Do not conflate v01 closure with A2 in the same commit — they are separate packets.

## Your Task

Fix both issues in a **bounded, surgical** way. No layout redesigns, no i18n changes, no unrelated refactors.

### Issue 1 — Reinstate NavRail Logo

**File**: `ui/src/components/NavRail.tsx`

**Requirements**:
1. Import `SeatLoomLogo` from `ui/src/app-v2/components/Avatar.tsx`.
2. Add a brand section at the **top** of the NavRail, **above** the `ProjectSwitcher`.
3. The Logo must render correctly in **both** `collapsed` and `expanded` states:
   - **Collapsed**: Show only the icon (no text), centered, ~32px size.
   - **Expanded**: Show icon + "SeatLoom" text, left-aligned, ~40px icon size.
4. Use the existing NavRail styling patterns (CSS variables like `--sl-surface`, `--sl-border`, etc.).
5. **Do not** add any new i18n keys. **Do not** modify `ui/src/i18n.ts`.

**Example structure** (adapt to fit NavRail's existing layout):
```tsx
<div className="navrail-brand" style={{ padding: '12px', borderBottom: '1px solid var(--sl-border)' }}>
  <SeatLoomLogo size={collapsed ? 32 : 40} showText={!collapsed} />
</div>
```

### Issue 2 — Fix ProjectSwitcher Dropdown Clipping

**Files**: `ui/src/components/ProjectSwitcher.tsx` (and possibly `ui/src/layouts/AppShell.tsx`)

**Root cause**: The dropdown is absolutely positioned but clipped by an ancestor with `overflow: hidden`.

**Solution** (choose **Option A**, it's more robust):

**Option A — Use React Portal** (recommended):
1. In `ProjectSwitcher.tsx`, import `ReactDOM.createPortal`.
2. When the dropdown is open, render it via `createPortal(dropdownContent, document.body)` instead of as a direct child.
3. Position the dropdown using `position: fixed` with calculated `top`/`left` coordinates based on the trigger button's `getBoundingClientRect()`.
4. Ensure `z-index` is high enough (e.g., `9999`) to appear above all other UI.

**Option B — Change ancestor overflow** (fallback if Portal is too complex):
1. Locate the ancestor with `overflow: hidden` (likely `AppShell.tsx` or a wrapper div).
2. Change it to `overflow: visible` **only if** it doesn't break other layout constraints (scrollbars, clipping of other elements).
3. If changing overflow breaks other things, **do not use this option** — use Option A instead.

**Prefer Option A.** It's safer and doesn't risk breaking other layout.

### Verification Steps

After your changes:

1. **TypeScript check**:
   ```bash
   cd ui && npx tsc --noEmit
   ```
   Must exit 0 with zero errors.

2. **Build check**:
   ```bash
   cd ui && pnpm build
   ```
   Must succeed.

3. **Visual verification** (run `pnpm tauri dev` or `pnpm dev`):
   - **NavRail Logo**:
     - Logo appears at the top of NavRail, above ProjectSwitcher.
     - In collapsed state: icon only, centered, ~32px.
     - In expanded state: icon + "SeatLoom" text, ~40px icon.
   - **ProjectSwitcher dropdown**:
     - Click the ProjectSwitcher.
     - Dropdown menu appears **fully visible**, not clipped.
     - Dropdown is above all other UI elements (no z-index issues).
     - Clicking outside the dropdown closes it.

4. **No regressions**:
   - SupervisorPanel (⌘K) still opens/closes correctly.
   - SupervisionDashboard (5-card layout) still renders.
   - No console errors.

### Commit and Report

Commit your changes:
```
fix(ui): reinstate NavRail Logo and fix ProjectSwitcher dropdown clipping (SG-A §A2)
```

## Delivery Format

Write your delivery to:
```
docs/coordination/tasks/mira/MIRA-2026-05-09-v02-navrail-logo-and-zindex-delivery-v1.md
```

Required sections:

```markdown
# Delivery: NavRail Logo + ProjectSwitcher Z-Index Fix (SG-A §A2)

[Mira -> Lyra] NavRail Logo + ProjectSwitcher Z-Index Fix

commit:
- <your-commit-hash>

completed:
- Issue 1: NavRail Logo reinstated (collapsed + expanded states)
- Issue 2: ProjectSwitcher dropdown clipping fixed (Option A / Option B)
- TypeScript check PASS
- Build check PASS
- Visual verification PASS

## Changes (git diff)

```
$ git show <commit> --stat
<paste output>

$ git show <commit>
<paste full diff — DO NOT SUMMARIZE>
```

## Verification

### TypeScript check
```
$ cd ui && npx tsc --noEmit
<paste full output>
```

### Build check
```
$ cd ui && pnpm build
<paste last 20 lines>
```

### Visual verification

**NavRail Logo**:
- Collapsed state: <describe what you see>
- Expanded state: <describe what you see>

**ProjectSwitcher dropdown**:
- Dropdown fully visible: <yes/no>
- Z-index correct: <yes/no>
- Closes on outside click: <yes/no>

**Regressions**:
- SupervisorPanel (⌘K): <works/broken>
- SupervisionDashboard: <renders/broken>
- Console errors: <none / list errors>

blockers:
- none / <describe any blockers>

verdict:
- PASS / HOLD / FAIL

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-v02-navrail-logo-and-zindex-delivery-v1.md
```

## Scope Constraints (CRITICAL — Read Carefully)

Aegis flagged **3 scope violations** in your v01 delivery:
1. **i18n over-deletion**: You removed more keys than your task specified.
2. **Proposal smuggling**: You included design proposals (v02 suggestions) in a delivery doc.
3. **Summary instead of verbatim**: You summarized tool output instead of pasting it in full.

**For this packet, you MUST**:
- **Only touch these files**: `NavRail.tsx`, `ProjectSwitcher.tsx`, and optionally `AppShell.tsx` (if using Option B).
- **Do NOT modify**:
  - `ui/src/i18n.ts` (no new keys, no deletions)
  - `ui/src/app-v2/panel/SupervisorPanel.tsx` (chan-03/chan-09 verified)
  - `ui/src/app-v2/dashboard/GlobalDashboard.tsx` (chan-03 verified)
  - `ui/src/components/SupervisionDashboard.tsx` (unless absolutely necessary for z-index fix)
  - `src-tauri/**` (no Rust changes)
- **Paste tool output verbatim** in your delivery. Do not summarize `tsc` or `pnpm build` output.
- **Do not include design proposals** in your delivery doc. If you have suggestions for future work, mention them in a separate message to Aegis or Lyra after delivery.

**Violation of these constraints will result in HOLD and revert.**

## Coordination Rules (COORDINATION_RULES.md §3)

- **Delivery must include tool output verbatim** (not summaries).
- **Commit hash required**. Lyra will verify the exact commit.
- **No scope drift**. If you discover other bugs, report them separately — do not fix them in this packet.

---

*Task issued by Lyra · 2026-05-09 · SG-A §A2 · Parallel with A1/A3/A4*
