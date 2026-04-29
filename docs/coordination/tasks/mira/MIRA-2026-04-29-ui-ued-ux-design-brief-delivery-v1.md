# Delivery: Mira UI/UED/UX Design Brief After P1 Repair

| Field | Value |
|---|---|
| ID | MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1 |
| Status | Delivered |
| Author | Mira |
| Date | 2026-04-29 |

## 1. Baseline Assumptions and Closed Items

The following P1 defects are treated as **closed baseline** and will not be reopened in this cycle unless explicitly directed by a HOLD verdict from Flux verification:
- `D-01`: Shared clickable hover cascade (fixed in `globals.css`).
- `D-02`: Hardcoded timeline event colors (fixed in `EventRow.tsx`).
- `D-03`: Hardcoded artifact badge colors (fixed in `ArtifactDetail.tsx`).
- `D-07`: Sidebar session contrast (fixed in `Sidebar.tsx`).
- `D-10`: Timeline object label contrast (fixed in `EventRow.tsx`).

## 2. Open Issue Inventory

The next cycle addresses the remaining P2 backlog and sponsor concerns, grouped by systemic problems:

### 2.1 Interaction Affordance and State Semantics
- **Weak Hover Feedback**: `D-04` (Secondary buttons), `D-09` (Sidebar icons), `D-11` (Mobile companion desktop).
- **Incomplete Affordance**: `D-06` (Opacity-only hover on primary actions).
- **Degraded Semantic Identity**: `D-14` (Disabled grayscale stripping color meaning).

### 2.2 Token, Contrast, and System Integrity
- **Contrast Gaps**: `D-05` (Artifact chip icon pale color).
- **Hardcoding/Static Artifacts**: `D-08` (Terminal color), `D-15` (Budget bar static placeholder).
- **Systematic Drift**: `D-12` (Inconsistent transition rhythms), `D-13` (Dual naming families `ink` vs `text`).

### 2.3 Identity and Harmony (Sponsor/PO Feedback)
- **Persuasiveness**: Visual identity across presets lacks a strong "SeatLoom" point of view.
- **Typographic Unevenness**: Mixed Chinese/English weights and sizes create reading friction.
- **Discipline**: Token usage is functional but lacks coherent UED governance.

## 3. Typography Recommendations

**Problem**: Mixed Chinese/English text currently feels disconnected; dense Chinese copy adjacent to bold English labels increases cognitive load.
- **Recommendation**: Standardize on a three-tier typographic scale. 
  - **Tier 1 (Semantic Labels)**: 9-10px bold English (uppercase with 0.1em tracking) paired with 11px Medium Chinese.
  - **Tier 2 (Primary Body)**: 13-14px Regular Chinese with 1.6 leading for flow, ensuring English monospace identifiers (IDs) are vertically centered.
  - **Tier 3 (Headings)**: Use the `--font-heading` token strictly for component headers, ensuring weight consistency across presets (900 for Sans, 700 for Serif).
- **Pain Solved**: Reduces "shouting" labels and "whispering" body text; improves skim-reading efficiency.
- **Risk of Omission**: Visual clutter and reading fatigue persist.
- **Behavior Change**: Faster identification of task IDs and role assignments.

## 4. Theme Preset Direction Recommendations

**Problem**: Presets are "color swaps" but don't feel like distinct operational environments.
- **Recommendation**: Deepen structural differentiation.
  - **Paper Ledger**: Lean into "Paper" by adding subtle grain textures and slightly uneven (organic) border-radius values.
  - **Harbor Blueprint**: Lean into "Blueprint" by using a dark-blue primary background with cyan/white lines and grid-aligned metrics.
  - **Sage Archive**: Lean into "Archive" by using dual-line borders (`border-double`) and wider margins to simulate print-ledger layouts.
- **Pain Solved**: Solves the "toy-like" or "default" feel reported by the sponsor.
- **Risk of Omission**: The product fails to establish a premium, professional identity.
- **Behavior Change**: Immediate cognitive context-switching when changing projects or roles.

## 5. Interaction State Policy

**Problem**: Hover and disabled states are currently handled per-file, leading to inconsistent feedback.
- **Policy**:
  - **Hover**: Shift from `opacity` to `brightness(1.05)` or a specific `--hover-tint` (typically `primary/10`).
  - **Active**: Preserve the "Reverse Contrast" (反显) baseline implemented in S6/S7.
  - **Disabled**: Replace `grayscale` with `opacity-50`. Use `cursor-not-allowed` and always provide a `title` (tooltip) explaining the lock reason.
- **Pain Solved**: Uncertainty about whether an action is interactive or why it is blocked.
- **Risk of Omission**: Intermittent "broken" feel where some hovers work and others don't.
- **Behavior Change**: Predictable system-wide feedback loop for all mouse and touch actions.

## 6. Implementation Packet Split

### Packet A — Interaction Affordance and State Semantics
**Focus**: SYSTEM REPAIR. Fix how the app feels when touched.
- **Targets**: `D-04`, `D-06`, `D-09`, `D-11`, `D-14`.
- **Files**: `SessionDetail.tsx`, `DelegationOverlay.tsx`, `Sidebar.tsx`, `MobileCompanionView.tsx`, `InboxView.tsx`.
- **Criteria**: Build passes; no opacity-only hovers on P0 CTAs; disabled buttons keep color but dim.

### Packet B — Token/System Cleanup and Typography
**Focus**: SYSTEM PURITY. Standardize naming and data-driving.
- **Targets**: `D-05`, `D-08`, `D-12`, `D-13`, `D-15`, `D-16`.
- **Files**: `ArtifactChip.tsx`, `SessionDetail.tsx`, `InboxItem.tsx`, `tailwind.config.js`, `EventRow.tsx`, plus all files found via `grep text-ink`.
- **Criteria**: Build passes; zero occurrences of `text-ink` in codebase; budget bar animated and data-bound.

## 7. Next Owner

- **Mira**: Fulfill implementation packets A and B sequentially.
- **Flux**: Verify after each implementation packet.
