# Acceptance: Artifact Objectization Baseline

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-artifact-objectization-baseline-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | LYRA-2026-04-28-artifact-objectization-baseline-v1 |
| verdict | PASS |
| tags | acceptance, artifact, template-subtype, prototype |

## Verdict

**PASS**

Reason: the scoped baseline is now closed in both contract and prototype layers. Seeded coordination documents are modeled as typed Artifact objects, Timeline/Inbox/Detail surfaces resolve them as openable evidence instead of raw path strings when metadata is valid, dual-key allow-list validation is enforced, and Timeline exposes working `template` / `subtype` filters.

This verdict applies to the artifact objectization baseline only. It does not lift the broader pre-implementation freeze and does not claim completion of the full Artifact Review module defined elsewhere in the v0.5 contract.

## Coverage Matrix

| # | Feature point | Document constraints | Prototype evidence | Result | Notes |
|---|---|---|---|---|---|
| 1 | Artifact schema carries canonical `template+subtype` and header-derived metadata | `docs/coordination/DOCUMENT_TEMPLATES.md` §11.1; `docs/interaction-spec-v1.1.md` §1.7; `docs/acceptance-spec-v1.1.md` `E-08` | `ui/src/types/index.ts`; `ui/src/utils/artifacts.ts` | PASS | Allow-list validation is implemented in code and invalid typing degrades visibly. |
| 2 | Representative coordination docs across T1-T7 are mapped into seeded Artifact objects | `docs/prd-v0.5.md` §6.5; `docs/ux-spec-v1.1.md` `UX-07`; `docs/acceptance-spec-v1.1.md` `P-10` | `ui/src/stores/useDataStore.ts`; `ui/src/mockData.ts` | PASS | Baseline uses curated seed coverage, not repo-wide automatic parsing. |
| 3 | Timeline evidence no longer depends on raw path-only display when mapped metadata exists | `docs/interaction-spec-v1.1.md` §1.7; `docs/interaction-spec-v1.1.md` `INT-04`; `docs/ux-spec-v1.1.md` `UX-10` | `ui/src/views/TimelineView.tsx`; `ui/src/components/EventRow.tsx` | PASS | Timeline shows typed artifact chips, searchable metadata, and `template` / `subtype` filters. |
| 4 | Inbox evidence is projected as typed Artifact references that can open detail | `docs/interaction-spec-v1.1.md` §1.7; `docs/interaction-spec-v1.1.md` `INT-04`; `docs/ux-spec-v1.1.md` `UX-03` | `ui/src/views/InboxView.tsx`; `ui/src/components/InboxItem.tsx`; `ui/src/App.tsx` | PASS | Linked artifacts open through the same Artifact detail route as primary objects. |
| 5 | WorkItem / Session / Handoff detail surfaces render typed artifact references instead of evidence-path-only sections | `docs/prd-v0.5.md` §6.5; `docs/ux-spec-v1.1.md` `UX-05`; `docs/ux-spec-v1.1.md` `UX-07` | `ui/src/components/WorkItemDetail.tsx`; `ui/src/components/SessionDetail.tsx`; `ui/src/components/HandoffDetail.tsx`; `ui/src/components/ArtifactReferenceList.tsx` | PASS | Unmapped evidence remains visible as warnings rather than being silently hidden. |
| 6 | Artifact detail exposes title, family, subtype, path, metadata strip, and subtype-aware summary framing | `docs/prd-v0.5.md` §6.5; `docs/ux-spec-v1.1.md` `UX-07`; `docs/acceptance-spec-v1.1.md` `U-08`; `docs/acceptance-spec-v1.1.md` `P-10` | `ui/src/components/ArtifactDetail.tsx`; `ui/src/components/ArtifactChip.tsx`; `ui/src/App.tsx` | PASS | The baseline is summary-oriented and metadata-aware; it is not yet the full markdown reader/review rail. |
| 7 | At least one main browsing surface filters and searches by artifact family/subtype | `docs/prd-v0.5.md` `US-P0-10`; `docs/interaction-spec-v1.1.md` `INT-13`; `docs/ux-spec-v1.1.md` `UX-10` | `ui/src/views/TimelineView.tsx`; `ui/src/utils/artifacts.ts` | PASS | Timeline is the required lightweight filter surface for the baseline. |
| 8 | Active product contract explicitly covers the objectization baseline | `docs/PRODUCT_TRUTH.md`; `docs/prd-v0.5.md`; `docs/interaction-spec-v1.1.md`; `docs/ux-spec-v1.1.md` | `docs/prd-v0.5.md`; `docs/interaction-spec-v1.1.md`; `docs/ux-spec-v1.1.md` | PASS | Contract text now names typed evidence chips, dual-key taxonomy, filter behavior, and fallback warnings. |
| 9 | Local verification passes after the prototype update | `docs/acceptance-spec-v1.1.md` `E-08` and the task packet done definition | `cd ui && npx tsc --noEmit`; `cd ui && pnpm build` | PASS | Both commands passed on 2026-04-28. |

## Issues Found

| ID | Severity | Description | Disposition |
|---|---|---|---|
| AO-01 | Low | The current Artifact detail is a metadata-aware baseline, not yet the full markdown reader with review rail, inline anchors, or persisted thread actions described by the broader Artifact Review module. | Accepted non-goal for this baseline. Keep under the broader `US-P0-07` implementation track. |
| AO-02 | Low | Artifact ingestion is seeded and representative. The prototype does not yet auto-index all markdown files in the repository. | Accepted non-goal for this baseline. |
| AO-03 | Low | `template` / `subtype` filtering is implemented on Timeline only, not yet on a dedicated artifact explorer. | Accepted baseline scope because at least one main surface filter was the explicit requirement. |

## Gate Decision

**GO for the artifact objectization baseline.**

This closes the scoped `template+subtype` mapping milestone and gives Nimbus/Flux a stable typed-artifact baseline to build against. The broader project-level implementation freeze remains unchanged until the remaining product-scope and full Artifact Review concerns are separately cleared.

## Follow-up Actions

- Lyra: keep future artifact-family additions blocked until `docs/coordination/DOCUMENT_TEMPLATES.md` updates the allow-list first.
- Nimbus: carry the same dual-key taxonomy into architecture/indexing follow-up work so storage, retrieval, and projection stay aligned.
- UI owner: treat full markdown-body reading, comment-thread authoring, and version/diff review as the next Artifact Review module scope rather than retroactively folding them into this baseline.
