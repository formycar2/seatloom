# Acceptance: Flux v0.5 UI Baseline Evidence Pack

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-flux-v05-ui-baseline-evidence-pack-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, flux, verification, ui, baseline, sg-01 |

## Verdict

**PASS**

Flux completed the requested read-only evidence pack against the accepted `SG-01` UI baseline.

Lyra accepts this packet because:

1. all 10 required UI surfaces are covered with concrete evidence paths;
2. the packet stays inside read-only QA scope and does not reopen accepted surfaces;
3. the raw evidence folder exists at the declared path; and
4. Lyra re-ran the shared UI checks locally: `cd ui && pnpm build` and `cd ui && npx tsc --noEmit`.

This packet is accepted as the durable verification close-out for the current v0.5 UI baseline.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`
- `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
- `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md`
- `.local/evidence/2026-04-29-v05-ui-baseline/build.log`
- `.local/evidence/2026-04-29-v05-ui-baseline/build_result.txt`
- `.local/evidence/2026-04-29-v05-ui-baseline/surfaces_inventory.txt`
- `ui/src/components/ProjectOverview.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/components/HandoffDetail.tsx`
- `ui/src/components/SupervisorCommandBar.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/ArtifactDetail.tsx`
- `ui/src/views/MobileCompanionView.tsx`
- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Shell truth, Project Overview default state, terminal auto-open, and shortcut help remain accepted and visible | `ui/src/components/ProjectOverview.tsx`; `ui/src/components/TerminalPanel.tsx`; `ui/src/layouts/AppShell.tsx`; `ui/src/components/ShortcutHelpDialog.tsx` | PASS | The shell closes cleanly around the accepted default overview and terminal behavior. |
| Inbox next-action loop and deterministic list search remain model-free and actionable | `ui/src/views/InboxView.tsx`; `ui/src/components/MorningDigest.tsx`; `.local/evidence/2026-04-29-v05-ui-baseline/surfaces_inventory.txt` | PASS | The evidence pack confirms deterministic filtering and action-queue behavior. |
| Timeline replay and drill-through remain wired to typed objects and artifact open paths | `ui/src/views/TimelineView.tsx`; `ui/src/components/EventRow.tsx`; `ui/src/components/ArtifactReferenceList.tsx` | PASS | The accepted replay and object drill-through flow is preserved. |
| Handoff lifecycle state strip remains visible with accepted detail truth | `ui/src/components/HandoffDetail.tsx` | PASS | The state strip is still present and in accepted scope. |
| Supervisor Command Bar keyboard and proposal behavior remain closed at the accepted baseline | `ui/src/components/SupervisorCommandBar.tsx`; `ui/src/hooks/useGlobalShortcuts.ts`; `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md` | PASS | `Cmd/Ctrl+K`, proposal generation, `Enter` confirm, and `Esc` close all remain aligned. |
| Seat capability truth and scoped delegation overlay remain visible without mutating original seat identity | `ui/src/components/SeatDetail.tsx`; `ui/src/components/DelegationOverlay.tsx`; `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md` | PASS | Delegation visibility stays inside the accepted overlay model. |
| Session prompt-blocked banner and continuity-pack preview remain visible and contract-aligned | `ui/src/components/SessionDetail.tsx`; `docs/coordination/acceptance/2026-04-28-lyra-mira-s5e-session-continuity-acceptance.md` | PASS | The packet correctly treats these surfaces as already closed. |
| Typed Artifact metadata, family badges, and filterable object paths remain visible | `ui/src/components/ArtifactDetail.tsx`; `ui/src/components/ArtifactChip.tsx`; `ui/src/views/TimelineView.tsx` | PASS | Template + subtype objectization remains intact. |
| Mobile companion monitor surface remains aligned to the accepted read-only boundary | `ui/src/views/MobileCompanionView.tsx`; `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`; `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-monitor-truth-fix-acceptance.md` | PASS | Mobile stays within monitor + feedback boundary, not full workstation scope. |
| Shared UI validation stays green on the current branch | `.local/evidence/2026-04-29-v05-ui-baseline/build.log`; `cd ui && pnpm build`; `cd ui && npx tsc --noEmit` | PASS | Lyra reproduced the build and type-check locally. |

## Findings

| ID | Severity | Finding | Why it matters | Status |
|---|---|---|---|---|
| FVP-01 | Low | The accepted mobile companion slice is still a read-only monitor surface and does not yet include future mobile action-card flows. | This is a boundary reminder only, not a regression in the verified baseline. | Residual note only |
| FVP-02 | Low | Continuity preview truth still depends on seeded prototype data until backend wiring lands. | The UI baseline is still valid because the evidence pack is verifying the current accepted prototype surface, not backend completeness. | Residual note only |

## Required Fixes for Flux

None.

## Go / No-Go Recommendation

- **Evidence pack closure:** **GO**
- **Keep `SG-01` closed at `GO`:** **GO**
- **Reopen accepted UI surfaces from this packet:** **NO-GO**

## Evidence Paths

- Packet issued: `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-v1.md`
- Delivery reviewed: `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md`
- Gate truth: `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`
- Supervisor acceptance: `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
- Delegation acceptance: `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`
- Raw evidence folder: `.local/evidence/2026-04-29-v05-ui-baseline/`
- UI build evidence: `.local/evidence/2026-04-29-v05-ui-baseline/build.log`
