# Delivery: P1 Theme + Hover Fixes

| Field | Value |
|---|---|
| ID | MIRA-2026-04-29-p1-theme-hover-fixes-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-29 |

## 1. Scope Completed

Implemented the five targeted `P1` fixes identified in the Flux visual audit. The changes are strictly bounded to the identified defects and do not include P2 polish items or a broader restyle.

## 2. Files Changed

- `ui/src/styles/globals.css`
- `ui/src/components/EventRow.tsx`
- `ui/src/components/ArtifactDetail.tsx`
- `ui/src/layouts/Sidebar.tsx`

## 3. Fix-by-Fix Mapping

### D-01 — Hover Cascade Bug
- **File:** `ui/src/styles/globals.css`
- **Action:** Removed the broad `.sl-clickable:hover *` descendant selector that was forcing `!important text-primary` on all children. This ensures that semantic status indicators (dots, badges) retain their correct colors when a row is hovered.

### D-02 — Hardcoded Colors in Timeline
- **File:** `ui/src/components/EventRow.tsx`
- **Action:** Replaced hardcoded Tailwind palette classes in `getTypeColor()` with theme-aware semantic tokens.
  - `Artifact*` and `WorkItemDelegated` now use `text-status-drifted`.
  - Generic `WorkItem*` now uses `text-status-warning`.

### D-03 — Hardcoded Template Badge Colors
- **File:** `ui/src/components/ArtifactDetail.tsx`
- **Action:** Replaced hardcoded background and text color classes for T1–T7 families with a consistent mapping to theme semantic tokens (`done`, `drifted`, `primary`, `warning`, `success`, `secondary`, `error`).

### D-07 — Low-Contrast Sidebar Session Text
- **File:** `ui/src/layouts/Sidebar.tsx`
- **Action:** Removed low-opacity overrides for inactive session labels. These now use solid `text-text-secondary` and `text-text-muted` tokens, providing stable and accessible contrast across all three theme presets.

### D-10 — Low-Contrast Object Label in Timeline
- **File:** `ui/src/components/EventRow.tsx`
- **Action:** Replaced `text-primary/80` (failing WCAG AA on light backgrounds) with `text-text-secondary` at full opacity. This improves readability for tiny 10px monospace labels.

### Normalization (R-05)
- **Action:** In all touched files, standardized text color naming to use the `text-text-*` convention instead of `text-ink*` aliases, reducing architectural drift.

## 4. Validation Commands and Results

```bash
cd ui && pnpm build
```
- **Result:** **SUCCESS** (Built in 1.38s, 0 TypeScript errors).

## 5. Residual Notes
- **P2 Polish:** All `P2` findings from the Flux audit (D-04, D-05, D-06, D-08, D-09, D-11, D-12, D-13, D-14, D-15) remain untouched to keep this packet focused on immediate blockers.
- **Theme Extensibility:** By removing hardcoded colors in `EventRow` and `ArtifactDetail`, the prototype is now fully responsive to theme preset switching in these areas.

## 6. Recommended Next Owner

**Flux** for verification of the P1 repairs and subsequent P2 audit prioritization.
