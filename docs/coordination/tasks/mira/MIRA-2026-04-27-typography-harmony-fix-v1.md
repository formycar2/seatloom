# MIRA-2026-04-27-Typography-Harmony-Fix-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Next UI resubmission on 2026-04-27 |
| Priority | P1 usability polish with direct readability impact |

## 1. Objective

Improve bilingual typography quality in the SeatLoom prototype.

Current problem observed by PO review:
- Chinese and English typography do not feel coordinated;
- several surfaces are visually cramped;
- very small type plus heavy uppercase/tracking makes the UI tiring to read.

Do **not** change product meaning or flow structure in this packet. This is a typography/readability pass.

## 2. Active contract references

Required references for this pass:
- `docs/prd-v0.4.md`
- `docs/interaction-spec-v1.0.md`
- `docs/acceptance-spec-v1.0.md`
- `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
- `docs/coordination/tasks/lyra/LYRA-2026-04-27-drift-signal-decision-v1.md`
- `docs/coordination/acceptance/2026-04-27-mira-v52-drift-signal-verification.md`

## 3. Need-to-know scope only

You only need to touch typography, spacing coupled to typography, and localized readability.

Primary target files:
- `ui/src/styles/globals.css`
- `ui/src/layouts/TopNav.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/layouts/StatusBar.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/views/WorkItemsView.tsx`
- `ui/src/components/MorningDigest.tsx`
- `ui/src/components/InboxItem.tsx`
- `ui/src/components/ReconcileNotification.tsx`
- any dialog/detail component where text remains visibly cramped after the system fix

Avoid broad flow rewrites.

## 4. Required changes

### A. Establish a bilingual type system

Create a clearer typography foundation for mixed English + Chinese UI:
- define a robust app sans stack that reads well for Latin and Simplified Chinese;
- keep monospace only for IDs, paths, branch names, and terminal-like data;
- centralize typography defaults in shared style tokens/classes instead of solving each surface ad hoc.

Preferred direction:
- app sans should support macOS + common cross-platform CJK fallback cleanly;
- mono stack should remain technical but not overpower surrounding UI.

### B. Raise the readability floor

The current prototype overuses very small text.

Required minimums:
- no primary interactive or navigational text below `12px`;
- secondary metadata should generally stay at `11-12px`, not `8-10px` by default;
- only truly minor technical labels may remain at `10px`, and only when contrast and spacing stay comfortable.

Targets that likely need adjustment:
- sidebar labels and sublabels;
- status bar items;
- inbox metadata rows;
- workitem badges and list meta;
- morning digest supporting copy;
- toast CTA text.

### C. Reduce over-stylization that hurts bilingual reading

Current issues include heavy uppercase, aggressive tracking, and cramped line-height.

Required:
- reduce excessive all-caps usage on dense surfaces;
- reduce tracking on small labels;
- do not rely on uppercase styling as the default for localized Chinese labels;
- use more comfortable line-height for body/supporting text;
- avoid italic styling for Chinese-supporting body copy when it reduces readability.

### D. Keep hierarchy, but make it calmer

The UI should still feel sharp and intentional, but easier to scan.

Required hierarchy intent:
- navigation and section headers stay clearly stronger than metadata;
- object titles stay readable at a glance;
- IDs remain monospace but visually secondary to titles;
- notification copy should be legible without shouting.

### E. Verify both locales

You must validate both `en` and `zh` modes.

Required screens for check:
- Inbox
- WorkItems
- Morning Digest
- Status bar
- Reconcile summary notification
- Init dialog / one form dialog

## 5. Acceptance criteria

This packet is done only when all are true:

1. English and Chinese UI look coordinated rather than mixed from different systems.
2. Reading effort is visibly lower on the main screens.
3. No obvious `8-9px` critical UI text remains.
4. Small labels no longer depend on extreme uppercase + wide tracking to carry hierarchy.
5. No new truncation or clipping regressions appear in nav, sidebar, status bar, inbox rows, or toast.
6. Typography rules are more centralized than before, not more fragmented.
7. Screenshots are provided for both `en` and `zh` after the pass.

## 6. Evidence required back to Lyra

Please return:
- changed file list;
- one short delta note describing the new type system;
- before/after screenshots for `en` and `zh`;
- any remaining typography compromises you intentionally left.

Store evidence under a durable artifact path and reference it in your handoff.

## 7. Notes from Lyra

This request is driven by direct PO review feedback, not by aesthetic preference alone.
The bar is: **readable, coordinated, lower-friction bilingual UI**.
Do not spend tokens on unrelated feature ideas in this pass.
