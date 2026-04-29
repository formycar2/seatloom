# LYRA-2026-04-27-sg01-recovery-plan-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active |
| Horizon | 2026-04-27 to 2026-05-08 |
| Goal | Recover SG-01 UI Contract Baseline and create a clean Nimbus implementation handoff |

## 1. Decisions needed

1. Freeze LaunchPack fallback policy.
2. Freeze canonical Handoff status enum.
3. Freeze canonical review-request event mapping.
4. Freeze shell and keyboard contract for the baseline UI.

## 2. Decisions made

1. Active working product contract = `docs/archive/product-history/prd-v0.4.md` + `docs/archive/product-history/interaction-spec-v1.0.md` + `docs/archive/product-history/acceptance-spec-v1.0.md`.
2. Approved constraints = `docs/archive/product-history/mvp-scenarios.md` + `docs/architecture-decisions.md`.
3. `docs/architecture-design.md` is draft reference only.
4. `SG-01 UI Contract Baseline` stays on Hold until Mira clears the P0 acceptance gaps.

## 3. Two-week milestone plan

| Window | Milestone | Owner | Exit criteria |
|---|---|---|---|
| 2026-04-27 to 2026-04-28 | Product contract conflict freeze | Lyra | All P0 conflicts resolved or explicitly waived in files |
| 2026-04-27 to 2026-04-29 | Mira SG-01 recovery pass | Mira | Lyra re-review upgrades verdict from FAIL to PASS or CONDITIONAL PASS with no Critical findings |
| 2026-04-30 to 2026-05-02 | Nimbus implementation-readiness handoff | Nimbus | Contract-aligned schema/event plan accepted and implementation can start without UI truth drift |
| 2026-05-03 to 2026-05-06 | Flux acceptance runbook and evidence pass | Flux | Reproducible runbook plus evidence matrix exists for P0 flows |
| 2026-05-07 to 2026-05-08 | SG-01 gate review | Lyra + Aegis | Gate decision file updated to Go or Hold with evidence list |

## 4. Action packets

| Packet | Owner | Deadline | Done definition | Acceptance owner |
|---|---|---|---|---|
| UI contract recovery packet | Mira | 2026-04-29 18:00 | Bottom Terminal panel restored; Inbox actions mutate mocked state correctly; Timeline filters and event detail added; Handoff accept/return/complete chain represented; required keyboard shortcuts wired; `pnpm build` passes; off-contract copy removed | Lyra |
| Schema and event alignment prep packet | Nimbus | 2026-04-30 12:00 | Publish a contract-alignment memo for Handoff statuses, review events, Inbox projections, and LaunchPack fallback handling; no implementation assumptions taken from rejected UI surfaces | Lyra |
| Acceptance runbook prep packet | Flux | 2026-04-30 18:00 | Publish step-by-step SG-01 verification matrix, required screenshots/logs, and local run commands; mark blocked items that depend on Mira/Nimbus completion | Lyra |
| Core implementation packet | Nimbus | 2026-05-02 18:00 | After Mira acceptance, wire prototype-approved flows into implementation plan and confirm no schema drift versus accepted UI contract | Lyra |
| Integrated QA packet | Flux | 2026-05-06 18:00 | Execute accepted runbook against latest build, attach evidence for project switch, Inbox, Timeline, Handoff, and recovery paths | Lyra |

## 5. Dependencies and handoff sequence

1. Lyra resolves or escalates the four P0 contract conflicts.
2. Mira reworks the prototype against the frozen shell and interaction contract.
3. Lyra re-reviews Mira output and issues a Go or No-Go for engineering handoff.
4. Nimbus starts implementation only from accepted UI truth, not from the rejected prototype.
5. Flux validates the accepted implementation path with reproducible evidence.
6. Aegis reviews the stage gate only after all required artifacts exist.

## 6. Blockers

- Current Mira redesign is not safe for Nimbus implementation handoff.
- LaunchPack fallback, Handoff enum, review event semantics, and shell contract still require P0 alignment.
- Flux evidence work is partially blocked until Mira and Nimbus clear their handoffs.

## 7. Stage-gate status

- Current gate: `SG-01 UI Contract Baseline = HOLD`
- Next review trigger: Mira resubmits a contract-correct prototype and Lyra re-runs acceptance
