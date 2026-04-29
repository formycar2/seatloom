# Review: Mira UI/UED/UX Readiness Note

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-29-mira-ui-ued-ux-readiness-note-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-v1.md` |
| tags | lyra, mira, ui, ued, ux, readiness, review |

## Conclusion

**READY WITH GATING**

Mira should reopen UI/UED/UX improvement work **after the current bounded `P1` verification loop finishes**. That means:

1. wait for Flux to finish the read-only verification of Mira's accepted five-defect `P1` packet;
2. if Flux returns `PASS`, open Mira's next UI/UED/UX cycle immediately;
3. if Flux returns `HOLD`, close those regressions first and only then reopen broader UI/UED/UX work.

`ENV-001` on the Nimbus track does **not** block Mira's next design-quality cycle. It is an engineering baseline issue, not a UI/UED/UX prerequisite.

## What is already closed and should not be reopened

The following five defects are treated as closed unless Flux's current verification reopens them with evidence:

- `D-01` broad `sl-clickable:hover *` cascade
- `D-02` hardcoded timeline event palette
- `D-03` hardcoded artifact template badge palette
- `D-07` inactive sidebar session contrast failure
- `D-10` event-row object-label contrast failure

These were accepted in:

- `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`

## Remaining UI/UED/UX issue inventory

### A. Open P2 quality issues from the accepted Flux audit

These remain the active backlog for Mira's next UI/UED/UX pass:

| ID | Issue | Current file target(s) | Notes |
|---|---|---|---|
| `D-04` | Secondary-button hover state is too weak on light presets | `ui/src/components/SessionDetail.tsx`, `ui/src/views/InboxView.tsx` | `hover:bg-accent` is still low-contrast on light backgrounds |
| `D-05` | Artifact chip icon contrast is too weak | `ui/src/components/ArtifactChip.tsx` | icon still uses `text-accent` |
| `D-06` | Primary action hover relies on opacity-only feedback | `ui/src/components/SessionDetail.tsx`, `ui/src/components/DelegationOverlay.tsx` | `hover:opacity-90` is still present |
| `D-08` | Terminal preview color is hardcoded and not theme-extensible | `ui/src/components/SessionDetail.tsx` | `text-green-400/90` still present |
| `D-09` | Sidebar icon-only hover affordance is still weak | `ui/src/layouts/Sidebar.tsx` | current fix adds background, but affordance still needs UED polish pass |
| `D-11` | Mobile companion rows have no desktop hover state | `ui/src/views/MobileCompanionView.tsx` | `active:bg-primary/5` only |
| `D-12` | Transition rhythm is inconsistent | `ui/src/components/InboxItem.tsx` | text/background timing still not unified |
| `D-13` | Dual text-token naming system remains in codebase | `ui/tailwind.config.js`, `ui/src/views/MobileCompanionView.tsx`, `ui/src/components/DelegationOverlay.tsx`, other mixed files | `text-ink*` and `text-text*` still coexist |
| `D-14` | Disabled buttons lose semantic identity | `ui/src/components/SessionDetail.tsx`, `ui/src/components/DelegationOverlay.tsx` | `disabled:grayscale` still present |
| `D-15` | Budget bar is still a static placeholder | `ui/src/components/SessionDetail.tsx` | `style={{ width: '45%' }}` still present |
| `D-16` | Event-row object-label contrast overlap needs re-baseline | `ui/src/components/EventRow.tsx` | may already be subsumed by the accepted `D-10` fix; recheck before reissuing |

### B. Sponsor-level dissatisfaction that is real but not yet packetized cleanly

The current codebase also still has a broader UED quality gap beyond the defect list:

1. **overall visual identity is still not persuasive enough** — the three presets are functional, but the palette/atmosphere still does not yet communicate a strong SeatLoom point of view;
2. **Chinese and English typography still feels uneven in several surfaces** — some screens still mix heavy uppercase labels with dense Chinese copy in a way that increases reading effort;
3. **hover and disabled states are still inconsistent across the shell** — some controls now feel fixed, but not yet systematized;
4. **theme system is working, but not yet governed as a coherent UED system** — tokens exist, but usage discipline is still incomplete.

These concerns are valid and should become explicit Mira scope, but they should be addressed **after** the current `P1` verification returns so we do not reopen the just-accepted packet blindly.

## Recommended opening sequence for Mira

### Step 1 — finish the current verification loop first

Owner: Flux  
Condition to proceed: `docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-delivery-v1.md` is delivered and reviewed by Lyra.

### Step 2 — open a read-only Mira design packet before more code edits

Do **not** send Mira directly into another styling patch first.

Open one bounded **read-only** packet for Mira to produce:

- a clean UI/UED/UX problem inventory,
- a typography-harmony recommendation,
- a preset-theme direction recommendation,
- a hover/disabled-state interaction policy,
- a proposed implementation split into 2–3 small packets.

Reason: the next cycle is no longer just bug repair; it is design-quality consolidation. That requires one contract pass before more patching.

### Step 3 — implement in two bounded Mira packets, not one broad redesign

Recommended split:

#### Packet A — Interaction affordance and state semantics

Target issues:

- `D-04`
- `D-06`
- `D-09`
- `D-11`
- `D-14`

Primary file targets:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/views/MobileCompanionView.tsx`

#### Packet B — Token/contrast/system cleanup

Target issues:

- `D-05`
- `D-08`
- `D-12`
- `D-13`
- `D-15`
- `D-16` only if still open after re-baseline

Primary file targets:

- `ui/src/components/ArtifactChip.tsx`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/InboxItem.tsx`
- `ui/tailwind.config.js`
- mixed `text-ink*` consumer files identified by grep

## Lyra operating position

Yes — the current UI/UED/UX problem set is understood well enough to drive Mira deliberately.

What I would avoid now is:

- reopening Mira immediately with a vague “make it prettier” request;
- mixing design-direction work with implementation in one large packet;
- widening into full aesthetic redesign before the current `P1` verification closes.

The correct move is:

1. close the current bounded verification,
2. open a read-only Mira UED packet,
3. then open two implementation packets from that accepted design brief.

## Suggested next owner

- **Immediate active owner:** Flux (finish current read-only `P1` verification)
- **Next owner after that:** Mira (read-only UI/UED/UX design brief)
- **Parallel non-blocking track:** Nimbus (`ENV-001` Tauri icon closure)
