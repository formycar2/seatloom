# Review: app-v2 UI Coverage vs PostgreSQL Collaboration Truth

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-30-app-v2-postgres-data-coverage-review-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `ui/src/app-v2/AppV2.tsx`, `ui/src/app-v2/DagWorkflow.tsx`, `ui/src/app-v2/dag-model.ts`, `ui/src/App.tsx`, `ui/src/types/index.ts`, `ui/src/mockData.ts`, `ui/src/stores/useDataStore.ts`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/003_write_ingest_reconcile.sql`, `infra/postgres/seed/001_real_collaboration_baseline.sql`, `infra/postgres/seed/002_document_seed.sql`, `crates/seatloom-core/src/db/models.rs`, `crates/seatloom-core/src/db/repositories.rs`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md` |
| tags | lyra, review, ui, ui-v2, app-v2, postgres, coverage, data-truth, artifacts, documents |

## Verdict

**Current `app-v2` is not yet a projection of Nimbus's persisted collaboration truth. It is still a narrative/mock shell.**

The latest UI/UX/UED direction in `ui/src/app-v2/` is useful as a presentation prototype, but it does **not yet represent most of the structured data families already persisted into PostgreSQL**. In its current form, it can communicate design intent, but it cannot serve as a trustworthy operating surface for SeatLoom's real collaboration record.

The core reason is simple:

- `app-v2` renders from in-file mock constants such as `MOCK_CONTACTS`, `MOCK_MESSAGES`, and `MOCK_CHANNEL_DATA` in `ui/src/app-v2/AppV2.tsx:86`, `ui/src/app-v2/AppV2.tsx:111`, and `ui/src/app-v2/AppV2.tsx:246`.
- The real app/store path exists elsewhere through `useDataStore` in `ui/src/App.tsx:27` and `ui/src/App.tsx:37`.
- Therefore the new UI direction has **not yet consumed the persisted truth families** that Nimbus has already modeled and seeded.

## Scope

This review compares:

1. The visible design/system behavior in `ui/src/app-v2/`.
2. The structured collaboration truth already modeled by Nimbus in PostgreSQL.
3. The richer typed UI-side object shapes that already exist outside `app-v2`.

The question answered here is:

**Which persisted data families are not yet reflected in the current UI/UX/UED design, and where are the major coverage holes?**

## Findings

### F1. `app-v2` is still powered by narrative mock data, not persisted truth

- **Severity**: Critical
- **Current state**: `app-v2` uses local constants for contacts, messages, and channel data in `ui/src/app-v2/AppV2.tsx:86`, `ui/src/app-v2/AppV2.tsx:111`, and `ui/src/app-v2/AppV2.tsx:246`.
- **Gap**: None of the PostgreSQL-backed object families are actually projected into the screen from a real read model.
- **Why this matters**: A screen can look coherent while still being disconnected from the real operating truth. That makes the prototype visually persuasive but operationally unsafe.
- **Recommendation**: Before more UI polishing, define a projection contract from persisted truth into `app-v2` surfaces. The immediate goal is not API completion; it is truthful object consumption.

### F2. The document authority layer is almost entirely absent from the UI

- **Severity**: Critical
- **Current state**: Nimbus now persists typed documents, sections, and object associations in `infra/postgres/schema/002_document_authority.sql:17`, `infra/postgres/schema/002_document_authority.sql:50`, and `infra/postgres/schema/002_document_authority.sql:67`.
- **Gap**: `app-v2` does not surface typed documents as first-class objects. There is no visible document list, no template/subtype-based reading surface, no section anchors, and no association drill-through.
- **Why this matters**: SeatLoom's collaboration is document-heavy. If documents cannot be opened, filtered, linked, and reviewed as typed objects, the product loses one of its core differentiators.
- **Recommendation**: Add a typed Artifact/Document reading path that shows at minimum `title`, `template`, `subtype`, `path`, `associated object`, and `section anchors`.

### F3. Session truth is not represented as a first-class operating object

- **Severity**: Critical
- **Current state**: PostgreSQL already models sessions in `infra/postgres/schema/001_seatloom_core.sql:62`, and the richer UI-side types already define `PromptState` and `ContinuityPreview` in `ui/src/types/index.ts:80` and `ui/src/types/index.ts:89`.
- **Gap**: `app-v2` does not present session runtime, branch, checkpoint, launch-pack/continuity state, prompt-blocked state, or budget state as structured session truth.
- **Why this matters**: Session continuity, prompt handling, and supervisor takeover are P0/P1 product promises. If the UI cannot show them, the product value is not visible.
- **Recommendation**: Sessions need a dedicated detail model in `app-v2`, not just indirect narrative mentions in channel cards or activity rows.

### F4. Seat identity, project role binding, and delegation overlay are not visible as separate layers

- **Severity**: High
- **Current state**: Nimbus models `seats`, `project_role_bindings`, and `seat_delegations` separately in `infra/postgres/schema/001_seatloom_core.sql:21`, `infra/postgres/schema/001_seatloom_core.sql:33`, and `infra/postgres/schema/001_seatloom_core.sql:47`.
- **Gap**: `app-v2` does not make the distinction between global seat identity, per-project role assignment, and temporary delegation overlay visible to the user.
- **Why this matters**: This three-layer model is a core SeatLoom concept. Without it, delegation and role truth collapse back into generic avatars and labels.
- **Recommendation**: Seat cards and owner chips should explicitly carry layer meaning: identity, role binding, and active delegation.

### F5. Work review, reissue, and handoff state are underrepresented

- **Severity**: High
- **Current state**: The database models `workitems` and `handoffs` in `infra/postgres/schema/001_seatloom_core.sql:80` and `infra/postgres/schema/001_seatloom_core.sql:95`. The richer UI types already include `ReviewChangeRecord` in `ui/src/types/index.ts:55`.
- **Gap**: `app-v2` shows work largely as dashboard narrative and DAG nodes, but not as structured reviewable objects with tiered change records, handoff states, or evidence-backed return/reissue loops.
- **Why this matters**: SeatLoom's collaboration value depends on visible review loops and handoff accountability, not only progress storytelling.
- **Recommendation**: Work item and handoff details in `app-v2` should expose review tier, ack mode, reviewer/executor, evidence refs, and handoff lifecycle state.

### F6. Timeline is presentation-first, not canonical-event-first

- **Severity**: High
- **Current state**: PostgreSQL already persists append-only `canonical_events` and `event_object_refs` in `infra/postgres/schema/001_seatloom_core.sql:129` and `infra/postgres/schema/001_seatloom_core.sql:138`.
- **Gap**: The `app-v2` activity/timeline region is still narrative content inside `AppV2.tsx`, not a projection over canonical events, referenced objects, and evidence refs.
- **Why this matters**: Without event/object/evidence linking, the timeline cannot support replay, audit, or trustworthy drill-through.
- **Recommendation**: Treat timeline rows as canonical events with linked object pills and evidence pills, not as freeform summaries only.

### F7. Reconcile freshness and revision tracking are fully absent

- **Severity**: High
- **Current state**: `reconcile_runs`, `reconcile_items`, and `document_versions` are already modeled in `infra/postgres/schema/003_write_ingest_reconcile.sql:14`, `infra/postgres/schema/003_write_ingest_reconcile.sql:33`, and `infra/postgres/schema/003_write_ingest_reconcile.sql:53`.
- **Gap**: `app-v2` provides no indication of sync freshness, failed ingest, stale document body, or revision history.
- **Why this matters**: Once SeatLoom starts ingesting live markdown and runtime outputs, users need to know whether what they see is fresh, stale, conflicted, or partially ingested.
- **Recommendation**: Introduce a lightweight ingest/reconcile status surface before scaling read complexity further.

## Current `app-v2` Surface Inventory

The main visible surfaces in `app-v2` today are concentrated in the project dashboard and are still channel-story driven:

- blockers section in `ui/src/app-v2/AppV2.tsx:604`
- active work section in `ui/src/app-v2/AppV2.tsx:659`
- DAG workflow section in `ui/src/app-v2/AppV2.tsx:727`
- next-step section in `ui/src/app-v2/AppV2.tsx:745`
- activity/timeline section in `ui/src/app-v2/AppV2.tsx:777`
- goals/stages section in `ui/src/app-v2/AppV2.tsx:839`

These surfaces are useful for storytelling and overview design, but they are **not yet keyed to the persisted PostgreSQL families below**.

## Coverage Matrix

| Data family / table | Nimbus persisted | `app-v2` coverage | Current UI location | Missing data / problem | Conclusion |
|---|---|---|---|---|---|
| `projects` | Yes | Partial | channel/dashboard framing in `ui/src/app-v2/AppV2.tsx` | project identity exists narratively, but not as structured project truth with persisted metadata/freshness | Partial only |
| `seats` | Yes | Partial | chat contacts / owner labels in `ui/src/app-v2/AppV2.tsx:86` and dashboard cards | seat names/avatars appear, but no typed seat object with capability truth, budgets, constraints, or status | Partial only |
| `project_role_bindings` | Yes | Missing | none | no visible distinction between global seat and project-local role binding | Missing |
| `seat_delegations` | Yes | Missing | none | no delegation overlay object, no issuer/scope/expiry truth, no delegated ownership visualization | Missing |
| `sessions` | Yes | Missing | none | no first-class session list/detail from persisted truth; runtime, branch, checkpoint, budget, prompt, continuity absent | Missing |
| `workitems` | Yes | Partial | blockers / active items / DAG / next-step in `ui/src/app-v2/AppV2.tsx:604`, `:659`, `:727`, `:745` | work is present as narrative cards and nodes, but not as typed reviewable objects with full state, AC, review tier, evidence, or audit | Partial only |
| `handoffs` | Yes | Missing | none | no handoff object surface, no lifecycle strip, no return/rework chain, no artifact/package visibility | Missing |
| `artifacts` | Yes | Missing | none | no typed artifact object reader; path/title/template/subtype/evidence relationships are not surfaced | Missing |
| `canonical_events` | Yes | Partial | activity/timeline area in `ui/src/app-v2/AppV2.tsx:777` | activity exists, but it is not a canonical event projection with stable IDs and structured payloads | Partial only |
| `event_object_refs` | Yes | Missing | none | timeline rows cannot drill into linked workitem/session/handoff/artifact refs | Missing |
| `documents` | Yes | Missing | none | no typed document object surface despite seeded T1-T7 coverage and stored body text | Missing |
| `document_sections` | Yes | Missing | none | no heading/anchor view, no section drill-through, no review anchor targets | Missing |
| `document_associations` | Yes | Missing | none | no document-to-workitem/session/handoff/project association visibility | Missing |
| `reconcile_runs` | Yes | Missing | none | no ingest freshness, last sync, or reconcile health status | Missing |
| `reconcile_items` | Yes | Missing | none | no per-file success/failure/conflict view | Missing |
| `document_versions` | Yes | Missing | none | no revision history or version-aware document reading surface | Missing |

## Additional Structural Observation

`app-v2` does have a more future-friendly workflow model than the screen currently uses. For example, `ui/src/app-v2/dag-model.ts` already allows `workItemRef` and `artifactRefs` on nodes. But that potential has not yet been realized in the actual dashboard projection.

Likewise, richer types already exist outside `app-v2`:

- `ReviewChangeRecord` in `ui/src/types/index.ts:55`
- `DelegationRecord` in `ui/src/types/index.ts:66`
- `PromptState` in `ui/src/types/index.ts:80`
- `ContinuityPreview` in `ui/src/types/index.ts:89`
- `Artifact` in `ui/src/types/index.ts:197`
- `CanonicalEvent` in `ui/src/types/index.ts:233`
- `InboxItem` in `ui/src/types/index.ts:243`

And the seeded store/mock layer already contains examples of data that `app-v2` still ignores:

- `prompt_state` in `ui/src/mockData.ts:75`
- `continuity_pack` in `ui/src/mockData.ts:92`
- `change_tier_record` in `ui/src/mockData.ts:196`
- artifact `template/subtype` examples in `ui/src/mockData.ts:213`, `ui/src/mockData.ts:227`, `ui/src/mockData.ts:241`
- canonical `object_refs` / `evidence_refs` in `ui/src/mockData.ts:302`, `ui/src/mockData.ts:303`

This means the main issue is **not lack of conceptual data modeling**. The issue is that the new UI direction has not yet been wired to consume that truth.

## Priority Gaps To Fix First

### P0

1. Replace `app-v2` mock-channel truth with a deterministic projection contract from persisted collaboration data.
2. Introduce typed first-class object opening for `WorkItem`, `Session`, `Handoff`, `Artifact`, and `Document`.
3. Make timeline rows canonical-event-based with linked object refs and evidence refs.

### P1

1. Surface the document authority layer: `template`, `subtype`, `sections`, `associations`, and document body freshness.
2. Surface session truth: prompt-blocked state, continuity tiers, runtime/branch/checkpoint, and budget status.
3. Surface seat three-layer truth: identity, project role binding, delegation overlay.

### P2

1. Add reconcile freshness and document revision visibility.
2. Add richer review thread / annotation / comment anchor behavior on top of document sections.
3. Let the DAG and dashboard cards open the underlying typed objects instead of remaining narrative-only.

## Bottom Line

The latest `app-v2` design is a usable **presentation direction**, but it is **not yet a truthful data operating surface** for the collaboration record Nimbus has already persisted.

If the question is, "Does the current UI/UX/UED reflect the real data we already have in PostgreSQL?" the answer is:

**Not yet. Only a small subset is present, and even that subset is mostly narrative rather than object-true.**
