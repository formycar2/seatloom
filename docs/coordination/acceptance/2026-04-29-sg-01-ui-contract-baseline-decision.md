# Acceptance: SG-01 UI Contract Baseline

| Field | Value |
|---|---|
| template | T5 |
| subtype | gate_decision |
| id | LYRA-2026-04-29-sg01-ui-contract-baseline-decision-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `SG-01 UI Contract Baseline` |
| verdict | GO |
| tags | acceptance, gate, sg-01, ui, baseline |

## Verdict

**GO**

`SG-01 UI Contract Baseline` is now closed at `GO`.

The last hold-level gap was the proposal-surface `Enter` confirm path in the Supervisor Command Bar. Mira's bounded hotfix closes that gap, `S7C` is already accepted, and Lyra reverified the shared UI build locally.

This gate decision clears the UI-baseline blocker only. It does **not** by itself close the broader `Product Baseline Freeze` or replace any later Aegis stage-gate review.

## Coverage Matrix

| Criterion | Result | Evidence | Gap |
|---|---|---|---|
| Terminal remains a bottom toggle panel inside the main shell | PASS | `docs/coordination/acceptance/2026-04-28-lyra-mira-s6ab-shell-truth-fixes-acceptance.md`; `ui/src/App.tsx`; `ui/src/components/TerminalPanel.tsx` | None |
| Inbox behaves as an action queue with state-correct next-action behavior | PASS | `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md`; `ui/src/views/InboxView.tsx`; `ui/src/components/MorningDigest.tsx` | None |
| Timeline supports replay, drill-through, and accepted event truth | PASS | `docs/coordination/acceptance/2026-04-29-lyra-mira-reentry-delta-scan-acceptance.md`; `ui/src/views/TimelineView.tsx`; `ui/src/components/EventRow.tsx` | None |
| Handoff lifecycle truth is visible in the UI | PASS | `docs/coordination/acceptance/2026-04-28-lyra-mira-s5b-s5d-ui-slices-acceptance.md`; `ui/src/components/HandoffDetail.tsx` | None |
| Supervisor creation flow and required keyboard behavior work on the claimed surfaces | PASS | `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`; `ui/src/components/SupervisorCommandBar.tsx`; `ui/src/hooks/useGlobalShortcuts.ts` | None |
| Scoped delegation overlay is visible and issuer-safe without mutating original seat identity | PASS | `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`; `ui/src/components/DelegationOverlay.tsx`; `ui/src/components/WorkItemDetail.tsx`; `ui/src/components/SeatDetail.tsx` | None |
| Prompt-blocked state and continuity preview remain visible on accepted session surfaces | PASS | `docs/coordination/acceptance/2026-04-28-lyra-mira-s5e-session-continuity-acceptance.md`; `docs/coordination/acceptance/2026-04-28-lyra-mira-s5b-s5d-ui-slices-acceptance.md`; `ui/src/components/SessionDetail.tsx` | None |
| Typed artifact rendering and deterministic filters remain visible in-product | PASS | `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`; `ui/src/components/ArtifactDetail.tsx`; `ui/src/components/ArtifactChip.tsx`; `ui/src/views/TimelineView.tsx` | None |
| Shared UI build passes on the current accepted branch | PASS | `cd ui && pnpm build` | None |

## Issues Found

No hold-level issues remain in the `SG-01` UI baseline after the S7B hotfix acceptance.

## Gate Decision

- **Gate status:** **GO**
- **What closes here:** the UI-baseline blocker that previously prevented a contract-safe shared prototype baseline
- **What remains outside this gate:** `Product Baseline Freeze`, `ENV-001` Rust-tooling verification on Nimbus's seat, and any later Aegis stage-gate/risk review

## Follow-up Actions

- Keep Nimbus on the currently issued seat-registry + delegation-storage packet; no scope widening from this gate decision alone.
- Issue Flux a final v0.5 UI evidence-pack / QA pass against the now-accepted baseline.
- Preserve `docs/coordination/acceptance/2026-04-27-sg-01-ui-contract-baseline-decision.md` as historical hold-state evidence; use this file as the current gate truth.

## Evidence Paths

- Historical hold decision: `docs/coordination/acceptance/2026-04-27-sg-01-ui-contract-baseline-decision.md`
- Supervisor acceptance: `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
- Delegation acceptance: `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`
- Session continuity acceptance: `docs/coordination/acceptance/2026-04-28-lyra-mira-s5e-session-continuity-acceptance.md`
- Session prompt / review-tier / seat-card acceptance: `docs/coordination/acceptance/2026-04-28-lyra-mira-s5b-s5d-ui-slices-acceptance.md`
- Shell truth acceptance: `docs/coordination/acceptance/2026-04-28-lyra-mira-s6ab-shell-truth-fixes-acceptance.md`
- Artifact objectization acceptance: `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`
