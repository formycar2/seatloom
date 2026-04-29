# Acceptance: Mira S5F Seeded Visibility Patch

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s5f-seeded-visibility-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5f-seeded-visibility-patch-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, ui, mira, seeded-visibility, proof-state, v0.5 |

## Verdict

**CONDITIONAL PASS**

Lyra accepts the core scope of `S5F`: the repaired UI surfaces now have credible seeded proof states, the data lands in the allowed files, and the local build passes.

Close-out is still required for a small copy-hygiene delta before the visual rebase is considered complete:

1. localize the remaining operator-facing English strings in the newly seeded `prompt_state` and `continuity_pack`, and
2. correct the compatibility typo in `ui/src/mockData.ts` (`中英混杂福利` -> `中英混杂`).

These are narrow content-fidelity issues, not a reason to reopen the seeded-visibility scope.
The next Mira packet may proceed, but it must absorb this cleanup first.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5f-seeded-visibility-patch-v1.md`
- `ui/src/stores/useDataStore.ts`
- `ui/src/mockData.ts`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Contract refs | Prototype evidence | Result | Notes |
|---|---|---|---|---|---|
| 1 | Seed one WorkItem with a realistic `change_tier_record` | Task packet; `docs/prd-v0.5.md` `US-P0-08`; `docs/interaction-spec-v1.1.md` review tiering flows | `ui/src/stores/useDataStore.ts` `wi-411`; `ui/src/mockData.ts` `wi-411` | PASS | The seeded WorkItem exposes `L2`, changed clauses, reviewer/executor, ack mode, and evidence references. |
| 2 | Seed one Session with a realistic `prompt_state` | Task packet; `docs/prd-v0.5.md` `US-P0-11`; `docs/interaction-spec-v1.1.md` `INT-16`; `docs/ux-spec-v1.1.md` `UX-12` | `ui/src/stores/useDataStore.ts` `ses-407`; `ui/src/mockData.ts` `ses-407` | PASS | The session now drives the accepted prompt-blocked surface with classification, policy, preview, expected next step, and budget. |
| 3 | Seed one Session with a complete continuity preview | Task packet; `docs/prd-v0.5.md` `US-P0-06`; `docs/ux-spec-v1.1.md` `UX-06` | `ui/src/stores/useDataStore.ts` `ses-408`; `ui/src/mockData.ts` `ses-408` | PASS | Tier 0 / 1 / 2, skills, playbook, budget estimate, and fallback path are all present. |
| 4 | Seed one Seat with capability-truth fields populated | Task packet; `docs/prd-v0.5.md` seat capability truth stories; `docs/ux-spec-v1.1.md` seat surfaces | `ui/src/stores/useDataStore.ts` `seat-2`; `ui/src/mockData.ts` `seat-2` | PASS | Capabilities, input/output types, budgets, constraints, and attached skills are populated for Nimbus. |
| 5 | Preserve typed artifact evidence and link repaired flows back to artifacts | `docs/prd-v0.5.md` artifact objectization stories; `docs/interaction-spec-v1.1.md` artifact drill-through; `docs/acceptance-spec-v1.1.md` typed artifact checks | `ui/src/stores/useDataStore.ts` `P1_SEEDED_ARTIFACTS`, inbox `linked_artifact_ids` additions | PASS | The patch strengthens proof visibility rather than overwriting existing typed artifact data. |
| 6 | Keep seeded copy Chinese and aligned with the 2026-04-28 narrative | Task packet guardrail; prior Chinese-density baseline decision | `ui/src/stores/useDataStore.ts`; `ui/src/mockData.ts` | CONDITIONAL PASS | Most new narrative content is Chinese and log-aligned, but some operator-facing strings remain English and one compatibility string contains a typo. |
| 7 | Stay within file scope and keep the build green | Task packet done definition | `ui/src/stores/useDataStore.ts`; `ui/src/mockData.ts`; `cd ui && pnpm build` | PASS | Lyra re-ran the build locally on 2026-04-28 and confirmed success. |

## Findings

| ID | Severity | Finding | Required action |
|---|---|---|---|
| UI-S5F-01 | Medium | The newly seeded prompt preview / expected-next strings in `ses-407` and the continuity proof-state strings in `ses-408` remain partially English, which does not fully satisfy the packet's Chinese-first copy guardrail. | Localize those operator-facing strings in `ui/src/stores/useDataStore.ts` and mirror the same fix in `ui/src/mockData.ts` before closing the next visual packet. |
| UI-S5F-02 | Low | `ui/src/mockData.ts` contains a compatibility typo: `不再出现明显中英混杂福利。` | Correct the compatibility mirror string to `不再出现明显中英混杂。` |
| UI-S5F-03 | Low | Seeded artifact titles and metadata in the typed artifact catalog remain English-heavy. | Accept for this slice because persistent coordination artifacts are governed as English documents; do not relabel them in the UI seed unless the product contract changes. |

## Gate Decision

- **S5F:** **GO with condition**
- **Next packet:** **GO now**
- **Condition carried forward:** the first step of the next Mira packet must close `UI-S5F-01` and `UI-S5F-02` before the visual rebase is considered done.

## Follow-up Actions

- Mira: start `docs/coordination/tasks/mira/MIRA-2026-04-28-light-theme-rebase-v1.md` after reading the updated packet; complete the S5F copy-hygiene delta first, then execute the light-theme rebase.
- Lyra: review the next packet as one combined close-out for copy harmony plus theme direction.
