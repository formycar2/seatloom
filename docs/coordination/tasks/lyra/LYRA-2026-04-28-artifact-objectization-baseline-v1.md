# Task: Artifact Objectization Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | integration |
| id | LYRA-2026-04-28-artifact-objectization-baseline-v1 |
| status | accepted |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | lyra |
| priority | P0 |
| deadline | 2026-04-28 |
| depends_on | docs/PRODUCT_TRUTH.md, docs/prd-v0.5.md, docs/interaction-spec-v1.1.md, docs/ux-spec-v1.1.md, docs/coordination/DOCUMENT_TEMPLATES.md |
| supersedes | none |
| tags | artifact, template-subtype, prototype, contract-closure |

## Objective

Upgrade the prototype from raw markdown-path display to typed Artifact objects that can be identified, filtered, opened, and reviewed inside the product surfaces.

This task closes the product-design and prototype baseline for `template+subtype` objectization. It is not a database delivery. It is a contract-to-prototype closure pass.

## Scope

1. Align the prototype Artifact schema to `template+subtype` with allow-list validation from `docs/coordination/DOCUMENT_TEMPLATES.md` Section 11.1.
2. Map representative coordination documents across T1-T7 into seeded Artifact objects.
3. Replace raw document-path evidence rendering with typed Artifact references in Timeline, Inbox, and relevant Detail contexts.
4. Add minimum viable Detail rendering by artifact family/subtype for the baseline set.
5. Add at least one filterable prototype surface by `template` and `subtype`.
6. Update the active PRD / Interaction / UX contract docs so the prototype behavior is explicitly covered.
7. Publish an acceptance artifact with a feature-to-contract-to-prototype coverage matrix.

## Non-goals

- No PostgreSQL or real database integration in this round.
- No full comment backend or persisted review threads.
- No automatic parsing of all repository markdown files.
- No expansion beyond the active `template+subtype` taxonomy.

## Prototype Surfaces

- `ui/src/stores/useDataStore.ts`
- `ui/src/mockData.ts`
- `ui/src/types/index.ts`
- `ui/src/App.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/components/InboxItem.tsx`
- `ui/src/components/EventRow.tsx`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/HandoffDetail.tsx`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/layouts/DetailPane.tsx`

## Done Definition

- [x] `Artifact` includes `template`, `subtype`, and structured metadata fields aligned to the allow-list.
- [x] Coordination docs in the seed data are represented as Artifact objects rather than path-only evidence.
- [x] A document evidence click opens Artifact detail with visible family badge, subtype chip, title, and path.
- [x] At least one list surface filters by `template` and `subtype` with correct results.
- [x] `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, and `docs/ux-spec-v1.1.md` explicitly cover the baseline objectization behavior.
- [x] An acceptance artifact declares `PASS` or `HOLD` with a coverage matrix and remaining gaps.
- [x] Memory writeback is completed in both `docs/coordination/MEMORY.md` and `docs/coordination/memory/2026-04-28.md` if the milestone is accepted.

## Constraints

- Follow `docs/PRODUCT_TRUTH.md` as the source entrypoint.
- Keep persistent artifacts in English; terminal summary in Chinese.
- Do not invent artifact subtypes beyond the allow-list.
- Preserve existing narrative demo density and do not revert unrelated work.

## Acceptance Reference

- `docs/acceptance-spec-v1.1.md` Section 3.1 (`US-P0-07`, `US-P0-10`)
- `docs/prd-v0.5.md` Section 6.5
- `docs/interaction-spec-v1.1.md` `INT-07`, `INT-13`
- `docs/ux-spec-v1.1.md` `UX-07`, `UX-10`

## Delivery Path

- Task/change record: `docs/coordination/tasks/lyra/LYRA-2026-04-28-artifact-objectization-baseline-v1.md`
- Acceptance artifact: `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`
- Memory writeback: `docs/coordination/MEMORY.md`, `docs/coordination/memory/2026-04-28.md`

## Outcome

Baseline accepted on 2026-04-28.

- Acceptance artifact: `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`
- Validation evidence: `cd ui && npx tsc --noEmit`, `cd ui && pnpm build`
