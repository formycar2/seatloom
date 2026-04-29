# Acceptance: Flux SG-01 Post-S7A UI Baseline Verification

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-flux-sg01-post-s7a-ui-baseline-verification-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, verification, flux, sg-01, ui, baseline, lyra |

## Verdict

**CONDITIONAL PASS**

Flux completed a useful post-`S7A` verification sweep, and Lyra accepts the build evidence plus the overall direction of the review.

However, two of Flux's four findings are no longer accurate against the current prototype state:

1. the continuity-pack preview is already rendered in `SessionDetail`, and
2. the interactive prompt action set is already rendered in `SessionDetail`.

Lyra therefore accepts the verification artifact with corrections rather than adopting the finding list as-is.

`SG-01` remains **HOLD**, but the live blocker set is narrower than Flux reported:

- valid hold: missing scoped delegation overlay flow (`US-P0-04`), and
- valid hold: missing Supervisor Command Bar wiring on `Cmd/Ctrl+K` (`INT-02`, `UX-02`).

The continuity preview and prompt-blocked action strip should **not** be reopened.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md`
- `ui/src/App.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/components/WorkItemDetail.tsx`
- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

## Finding Disposition

| ID | Flux finding | Lyra disposition | Result |
|---|---|---|---|
| F-01 | Continuity pack preview not rendered in any surface | **Dispute** | `SessionDetail` already renders a `Continuity Pack Preview` block with Tier 0 identity, Tier 1 state, Tier 2 decisions, seat skills, playbook matches, budget estimate, and fallback path. |
| F-02 | Delegation overlay editor not wired | **Accept** | The prototype exposes seat capability truth, but it still lacks a scoped delegation overlay flow from work routing / seat detail into an explicit temporary delegation surface. |
| F-03 | Supervisor Command Bar not wired on `Cmd/Ctrl+K` | **Accept** | `useGlobalShortcuts` still routes `Cmd/Ctrl+K` to tab toggling / project switching behavior instead of opening a Supervisor Command Bar. |
| F-04 | Prompt-blocked action buttons not rendered | **Dispute** | `SessionDetail` already renders classification, policy, bounded preview, and the required action set: `Approve`, `Human takeover`, `Supervisor assist`, and `Stop`. |

## Corrected Evidence Matrix

| Requirement slice | Evidence | Result | Notes |
|---|---|---|---|
| `US-P0-09` / `UX-06` continuity preview must be visible before launch | `ui/src/components/SessionDetail.tsx:287`; `ui/src/components/SessionDetail.tsx:300`; `ui/src/components/SessionDetail.tsx:308`; `ui/src/components/SessionDetail.tsx:319`; `ui/src/components/SessionDetail.tsx:330`; `ui/src/components/SessionDetail.tsx:340`; `ui/src/components/SessionDetail.tsx:353`; `ui/src/components/SessionDetail.tsx:362` | PASS | The current prototype already exposes the tiered preview and cost/fallback surfaces in-session. |
| `US-P0-11` / `UX-12` prompt-blocked state must show bounded evidence and allowed actions | `ui/src/components/SessionDetail.tsx:94`; `ui/src/components/SessionDetail.tsx:105`; `ui/src/components/SessionDetail.tsx:119`; `ui/src/components/SessionDetail.tsx:131`; `ui/src/components/SessionDetail.tsx:142` | PASS | Classification, policy, preview, token budget, and all four actions are already present. |
| `US-P0-04` scoped delegation overlay must be visible and temporary | `ui/src/components/SeatDetail.tsx:104`; `ui/src/components/WorkItemDetail.tsx:170`; `ui/src/hooks/useGlobalShortcuts.ts:22` | HOLD | Capability truth is present, but no delegation overlay drawer/editor or scoped handoff-to-delegation action is wired. |
| `INT-02` / `UX-02` Supervisor Command Bar must open from `Cmd/Ctrl+K` | `ui/src/hooks/useGlobalShortcuts.ts:22`; `ui/src/App.tsx:74` | HOLD | Shortcut handling still toggles shell navigation instead of opening a Command Bar surface. |
| Build and type safety after review | `cd ui && pnpm build`; `cd ui && npx tsc --noEmit` | PASS | Lyra re-ran both checks locally on 2026-04-29. |

## Required Fixes

1. **Mira** — implement the scoped delegation overlay flow required by `US-P0-04`.
   - Primary targets: `ui/src/components/WorkItemDetail.tsx`, `ui/src/components/SeatDetail.tsx`, and any new bounded overlay component required by `UX-04` / `UX-11`.
2. **Mira** — implement the Supervisor Command Bar surface and wire `Cmd/Ctrl+K` to open it.
   - Primary targets: `ui/src/App.tsx`, `ui/src/hooks/useGlobalShortcuts.ts`, plus the new Command Bar component aligned to `UX-02`.

## Gate Decision

- **Flux verification packet quality:** **GO with corrections**
- **Reuse Flux evidence for next packets:** **GO**
- **Reopen continuity preview work:** **NO-GO**
- **Reopen prompt-blocked action UI work:** **NO-GO**
- **Overall `SG-01 UI Contract Baseline`:** **HOLD** pending delegation overlay and Supervisor Command Bar closure

## Evidence Paths

- Verification packet: `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-v1.md`
- Delivery reviewed: `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md`
- Session detail evidence: `ui/src/components/SessionDetail.tsx`
- Shortcut wiring evidence: `ui/src/hooks/useGlobalShortcuts.ts`
- Shell wiring evidence: `ui/src/App.tsx`
- Capability detail evidence: `ui/src/components/SeatDetail.tsx`
- Work routing detail evidence: `ui/src/components/WorkItemDetail.tsx`
