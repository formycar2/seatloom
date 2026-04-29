# Acceptance: Mira S5B-S5D UI Realignment Slices

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s5b-s5d-ui-slices-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5b-workitem-review-tier-strip-v1.md`; `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5c-session-prompt-blocked-surfaces-v1.md`; `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5d-seat-card-capability-truth-v1.md` |
| verdict | PASS |
| tags | acceptance, ui, mira, v0.5, review-tier, prompt-state, seat-card |

## Verdict

**PASS**

Lyra accepts the three scoped UI slices as delivered for the current queue stage.

- `S5B` closes the visible review-tier gap on `WorkItemDetail` and surfaces the required `change_tier_record` fields.
- `S5C` closes the prompt-blocked visibility baseline on the session list and session detail surface, including classification, policy, bounded preview, and action affordances.
- `S5D` upgrades the existing seat detail into a capability-truth surface that is materially closer to the v0.5 seat contract than the previous role-only view.

This acceptance is sufficient to unlock `S5E Session Continuity Preview`.
`S5F Seeded Visibility Patch` remains blocked until `S5E` is accepted.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5b-workitem-review-tier-strip-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5c-session-prompt-blocked-surfaces-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5d-seat-card-capability-truth-v1.md`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/utils/display.ts`

## Coverage Matrix

| # | Slice | Contract refs | Prototype evidence | Result | Notes |
|---|---|---|---|---|---|
| 1 | WorkItem review tier strip is visible in detail view | `docs/prd-v0.5.md` `US-P0-05`; `docs/interaction-spec-v1.1.md` `INT-05`; `docs/ux-spec-v1.1.md` `UX-05` | `ui/src/components/WorkItemDetail.tsx` | PASS | `L1/L2/L3` tier badges are visible in the work item detail surface. |
| 2 | Structured `change_tier_record` fields are rendered for review follow-up | `docs/prd-v0.5.md` §6.4; `docs/interaction-spec-v1.1.md` §1.8; `docs/acceptance-spec-v1.1.md` `US-P0-05` | `ui/src/components/WorkItemDetail.tsx` | PASS | Reason, reviewer, executor, changed clauses, impact, ack mode, and evidence references are all surfaced. |
| 3 | `L2` compact acknowledgment and `L3` full-gate escalation are distinguishable | `docs/prd-v0.5.md` §6.4; `docs/coordination/COLLABORATION_PROTOCOL.md` §4.6 | `ui/src/components/WorkItemDetail.tsx` | PASS | The UI visibly differentiates the low-cost `L2` path from the heavier `L3` gate path. |
| 4 | Prompt-blocked state is discoverable from the session list and detail view | `docs/prd-v0.5.md` `US-P0-11`; `docs/interaction-spec-v1.1.md` `INT-16`; `docs/ux-spec-v1.1.md` `UX-12` | `ui/src/layouts/Sidebar.tsx`; `ui/src/components/SessionDetail.tsx` | PASS | Blocked sessions now expose an explicit prompt-state cue before the user opens the session. |
| 5 | Prompt detail shows classification, policy, bounded preview, and action choices | `docs/prd-v0.5.md` `US-P0-11`; `docs/interaction-spec-v1.1.md` `INT-16`; `docs/ux-spec-v1.1.md` `UX-12` | `ui/src/components/SessionDetail.tsx` | PASS | Approve / Human takeover / Supervisor assist / Stop actions are visible, with assist disable logic applied when required. |
| 6 | Seat detail exposes capability truth instead of role-only labels | `docs/prd-v0.5.md` `US-P0-03`; `docs/prd-v0.5.md` `US-P0-08`; `docs/interaction-spec-v1.1.md` `INT-02`; `docs/ux-spec-v1.1.md` `UX-04` | `ui/src/components/SeatDetail.tsx` | PASS | Capabilities, input/output types, budgets, constraints, and attached skills are now visible. |
| 7 | Seat status signals assignable / under-specified / budget-constrained using current data | `docs/prd-v0.5.md` `US-P0-03`; `docs/ux-spec-v1.1.md` `UX-04`; `docs/acceptance-spec-v1.1.md` `P-04` | `ui/src/components/SeatDetail.tsx` | PASS | The state badge is derived from current seat fields rather than fabricated helper data. |
| 8 | Local verification still passes after the three slices land | Task packet done definitions; `docs/acceptance-spec-v1.1.md` baseline validation expectation | `cd ui && pnpm build` | PASS | Lyra re-ran the build locally after the latest slice set. |

## Findings

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| UI-S5-01 | Low | Some new operator-facing labels still mix Chinese and English (`Prompt`, `Full Gate Required`, `Seat Card`, `Capabilities`). | Accept for now. Keep future edits Chinese-first and fold lightweight copy normalization into later UI polish rather than reopening these slices immediately. |
| UI-S5-02 | Low | `S5C` only closes prompt-state visibility. Continuity-pack preview is still missing and remains the next required session-level slice. | Expected. `S5E` stays mandatory and is now unlocked. |

## Gate Decision

- **S5B:** **GO / accepted**
- **S5C:** **GO / accepted**
- **S5D:** **GO / accepted**
- **S5E:** **GO / unlocked now**
- **S5F:** **HOLD** until `S5E` is accepted

## Follow-up Actions

- Mira: start `S5E Session Continuity Preview` now and keep the edit scoped to `ui/src/components/SessionDetail.tsx`.
- Mira: keep overall concurrency at three packets maximum, but do not overlap edits on the same file.
- Lyra: review `S5E` next, then decide whether copy normalization needs a dedicated follow-up micro-slice or can stay bundled into later polish.
