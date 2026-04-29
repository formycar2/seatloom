# Review: v2 Frontend Data Coverage Audit

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-30-v2-data-coverage-audit-v1 |
| status | active |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/coordination/tasks/lyra/LYRA-2026-04-30-v2-data-coverage-audit-v1.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `ui/src/types/index.ts`, `ui/src/stores/useDataStore.ts`, `ui/src/app-v2/AppV2.tsx`, `ui/src/app-v2/dag-model.ts`, `crates/seatloom-core/src/objects/seat.rs`, `crates/seatloom-core/src/objects/session.rs`, `crates/seatloom-core/src/objects/workitem.rs`, `crates/seatloom-core/src/objects/handoff.rs`, `crates/seatloom-core/src/objects/artifact.rs`, `crates/seatloom-core/src/objects/checkpoint.rs`, `crates/seatloom-core/src/objects/pipeline.rs`, `crates/seatloom-core/src/objects/receipt.rs`, `crates/seatloom-core/src/ledger/event.rs`, `crates/seatloom-core/src/db/models.rs`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/003_write_ingest_reconcile.sql`, `docs/coordination/DOCUMENT_TEMPLATES.md` |
| tags | review, audit, app-v2, data-coverage, postgres, typed-documents |

## Verdict

- `HOLD`

The current `ui/src/app-v2/` baseline is presentation-promising but data-incomplete.
It does not yet expose the real PostgreSQL-backed object model, and it does not yet provide durable entrypoints for the real coordination artifacts already being produced in the repo.

## Scope and Method

This audit compared three layers side by side:

1. **Authority layer**: architecture contract, Rust domain objects, and PostgreSQL schema/repositories.
2. **Shared frontend model**: `ui/src/types/index.ts` plus seeded truth in `ui/src/stores/useDataStore.ts`.
3. **Actual v2 UI expression**: `ui/src/app-v2/AppV2.tsx` and `ui/src/app-v2/dag-model.ts`.

Document coverage was measured by scanning only `docs/coordination/**/*.md` and reading header metadata from the first 80 lines of each file. This avoids false positives from template examples inside `DOCUMENT_TEMPLATES.md`.

## Executive Findings

1. **`app-v2` still runs on a parallel mock schema.**
   `AppV2.tsx` defines local `ChatContact`, `ChatMessage`, `ProjectChannelData`, and `TimelineEntry` types, while only one dashboard slice partially projects from `useDataStore()` truth.

2. **Typed documents are not first-class UI objects yet.**
   PostgreSQL already models documents, sections, associations, versions, and reconcile runs, but `app-v2` only shows a generic recent-artifact strip with hover text.

3. **Operational truth is flattened before it reaches the UI.**
   Seat identity, role binding, delegation, session prompt state, checkpoint continuity, review tiers, handoff receipts, and document associations are either partially typed or not surfaced at all.

4. **Real repo output is dominated by T3/T5 artifacts, but v2 has no dedicated entry for them.**
   The repo already produces many task, verification, acceptance, and gate artifacts, yet `app-v2` has no subtype-specific list, filter, reader, or review surface.

5. **Several gaps are not only UI gaps.**
   `InboxItem`, playbook/skill truth, budget truth, and review/re-scope evidence are not consistently modeled across architecture, Rust, TypeScript, and UI.

## Table 1. Core Object Coverage

| Data object | Authority layer (architecture / Rust / PG) | Shared frontend model (`ui/src/types`) | v2 UI coverage today | Gap summary | Priority |
|---|---|---|---|---|---|
| Project | Yes | Yes | Partial | `app-v2` has project-channel shell and dashboard context, but not a real project detail surface backed by authority data. | P1 |
| Seat identity / role binding / delegation | Yes | Partial | Partial | Rust/PG split Seat into `SeatIdentity`, `ProjectRoleBind`, and `SeatDelegation`; TS flattens to `Seat`; v2 only shows contacts / owners and no authority-doc, role-binding, or delegation truth. | P0 |
| Session core | Yes | Partial | Partial | TS omits `PromptBlocked`; v2 has no session object surface, only snippets and blocker projections. | P0 |
| Prompt state / assist policy | Yes | Partial | No | Rust has `PromptState`, `PromptKind`, `PromptPolicy`, `PromptAction`, `AssistBudget`; TS keeps only a reduced preview model; v2 has no first-class prompt triage surface. | P0 |
| WorkItem core | Yes | Yes | Partial | v2 maps some work into blockers, in-progress cards, DAG nodes, and next-step hints, but not as a first-class WorkItem lifecycle with AC, dependencies, owner, and state transitions. | P0 |
| Review / re-scope evidence | Partial | Partial | Partial | Rust moved review/re-scope truth into canonical events; TS keeps `change_tier_record` on `WorkItem`; v2 only hints at review data through hover/meta and does not expose the closed review loop. | P0 |
| Handoff lifecycle | Yes | Yes | Partial | v2 can imply handoff activity via blockers and messages, but has no handoff detail, receipt state, working strip, or evidence bundle surface. | P0 |
| Handoff receipt | Yes | No | No | `HandoffReceipt` exists in Rust but not in TS or v2. | P1 |
| Artifact core (`template+subtype`) | Yes | Partial | Partial | Authority layer includes `subtype_valid` and `system_kind`; TS omits both; v2 shows only a recent artifact strip with chips and hover text. | P0 |
| Document authority (`documents`, `sections`, `associations`) | Yes | No | No | PostgreSQL already stores typed documents, section anchors, and object associations; v2 has no document reader, association jump, or anchor-level open path. | P0 |
| CanonicalEvent / audit chain | Yes | Partial | Partial | TS event taxonomy is behind Rust (missing prompt, delegation, review, and handoff-working events); v2 shows a generic activity list but not a full audit explorer. | P0 |
| Inbox / route-engine output | Partial | Yes | Partial | Architecture defines inbox behavior, and TS has `InboxItem`, but Rust/PG do not yet expose it as a first-class persisted object; v2 shows only one “next step” card, not a queue. | P0 |
| Checkpoint / continuity | Yes | Partial | No | Rust has `Checkpoint`; TS only has `ContinuityPreview`; v2 does not expose checkpoint history, continuity tiers, or restore choices. | P1 |
| Pipeline / pipeline run | Yes | Partial | Partial | Rust has `PipelineRun`; TS only has IDs; v2 DAG is generic and can draw pipeline-like nodes, but no truth-backed pipeline object is routed to the UI. | P2 |
| Playbook / seat skills / collaboration template | Partial | Partial | No | Role binding stores `collaboration_template_ref`; TS exposes skill / playbook strings only; neither layer provides a first-class persisted playbook object and v2 has no visibility. | P1 |
| Budget / token ROI | Partial | Partial | No | Budgets exist as embedded config / seat / prompt fields, not as one coherent domain object; v2 does not surface enforcement or current budget state. | P1 |
| Reconcile runs / document versions | Yes | No | No | PG already models ingest runs, per-file outcomes, and document revisions; v2 has zero observability for “markdown changed -> PG updated” truth maintenance. | P2 |
| Object selection / routing envelope | Partial | Partial | Partial | `SelectedObjectType` and `ObjectRef` do not cover `Delegation`, `Document`, `Checkpoint`, or `PipelineRun`, so even with data present, v2 cannot route many real objects. | P0 |

## Table 2. Real Coordination Document Coverage by `template+subtype`

**Scope note:** counts below are for `docs/coordination/` only. Active T1 authority docs such as `docs/prd-v0.5.md` live outside this folder, so they are intentionally not counted here even though they are seeded in the frontend store.

| Template | Subtype | Typed files in `docs/coordination/` | Current v2 UI expression | Coverage note |
|---|---|---:|---|---|
| T1 | `prd` | 0 | Seed-only generic artifact card | No coordination-folder source; no document reader or authority-doc entry in v2. |
| T1 | `ux_spec` | 0 | Seed-only generic artifact card | Same gap as above. |
| T1 | `interaction_spec` | 0 | Seed-only generic artifact card | Same gap as above. |
| T1 | `acceptance_spec` | 0 | Seed-only generic artifact card | Same gap as above. |
| T1 | `architecture_design` | 0 | Seed-only generic artifact card | Same gap as above. |
| T1 | `architecture_decisions` | 0 | Seed-only generic artifact card | Same gap as above. |
| T2 | `seat_role` | 0 | Seed-only generic artifact card | Real role docs exist in `docs/coordination/roles/` as legacy/untyped files; v2 has no role-profile reader. |
| T3 | `task` | 40 | Incidental generic artifact card only | No task-packet list, no subtype filter, no open/read/review entry. |
| T3 | `fix` | 15 | Incidental generic artifact card only | Same gap. |
| T3 | `integration` | 1 | Incidental generic artifact card only | Same gap. |
| T3 | `verification` | 15 | Incidental generic artifact card only | Same gap, despite high operational importance. |
| T4 | `gap_review` | 6 | Incidental generic artifact card only | No review workspace or findings reader. |
| T4 | `benchmark` | 0 | Seed-only generic artifact card | Seed exists, but no real typed coordination file and no UI entry. |
| T4 | `process_mapping` | 1 | Incidental generic artifact card only | No dedicated mapping or relationship surface. |
| T4 | `design_proposal` | 2 | Incidental generic artifact card only | No proposal comparison or approval path. |
| T5 | `acceptance_review` | 34 | Incidental generic artifact card only | No acceptance queue, verdict summary, or gate trail surface. |
| T5 | `gate_decision` | 3 | Incidental generic artifact card only | No gate history or blocking-decision entrypoint. |
| T6 | `daily_log` | 0 | Seed-only generic artifact card | Real daily logs exist in `docs/coordination/memory/` as legacy/untyped files; v2 has no memory surface. |
| T7 | `coordination_rules` | 0 | Seed-only generic artifact card | Real governance docs exist as legacy/untyped root files; v2 has no governance reader. |
| T7 | `workflow_principles` | 0 | Seed-only generic artifact card | Same gap. |
| T7 | `collaboration_protocol` | 0 | Seed-only generic artifact card | Same gap. |
| T7 | `document_templates` | 0 | Seed-only generic artifact card | Same gap; no subtype taxonomy browser or validator status. |

### Repo migration status notes

- **Valid typed coordination files:** 117
- **Invalid typed coordination files:** 7
- **Legacy / untyped coordination files:** 86

Invalid typed files are concentrated in `docs/coordination/tasks/mira/`, where seven files use `T2/task_packet` or `T2/fix`, which are outside the allow-list in `DOCUMENT_TEMPLATES.md`.

Important nuance: `ui/src/stores/useDataStore.ts` already seeds at least one artifact for every currently allowed subtype family (`T1` through `T7`), so the main gap is **not lack of seed coverage**. The gap is that `app-v2` still treats those artifacts as generic cards instead of typed, navigable document-authority objects.

## Table 3. High-Value Data Flows That Still Lack UI Expression

| Flow | Authority support today | v2 UI expression today | Gap |
|---|---|---|---|
| WorkItem -> task packet -> delivery -> verification -> acceptance / gate | Partial to strong | Mock cards plus generic artifact strip | No closed-loop surface linking object state, documents, and verdicts. |
| Handoff drafted -> sent -> accepted -> working -> receipt -> completed | Strong | Only implied in blockers/messages | No first-class handoff screen, no receipt confirmation, no evidence bundle. |
| Prompt blocked -> approve / human takeover / supervisor assist / stop | Strong | None | No prompt-state surface, action choice, or audit replay. |
| Checkpoint created -> continuity pack preview -> resume | Strong | None | No recovery path, no checkpoint list, no tiered continuity view. |
| Delegation issued -> active overlay -> close delegation | Strong | None | No delegation object surface or timeline/object routing path. |
| Markdown changed -> reconcile -> document version -> association update | Strong | None | No visibility into parse status, reconcile runs, or document version history. |
| Pipeline run started -> stage completed -> failed/completed -> evidence | Moderate | Mock DAG metaphor only | No truth-backed pipeline run UI or evidence linkage. |

## Gap Analysis

### 1. The biggest problem is model divergence, not visual incompleteness

`app-v2` is still primarily a chat/dashboard mock model with local types. It is not yet a projection of the authority model already present in Rust and PostgreSQL. As long as that divergence remains, every new surface will need manual sync work and will drift again.

### 2. The document-authority layer is the largest missing product surface

The backend already has the right shape for:
- typed documents,
- section anchors,
- object associations,
- reconcile runs,
- document versions,
- full-text retrieval.

The frontend currently exposes none of these as first-class behaviors. This means the product can store real collaboration truth, but users still cannot browse, filter, open, compare, or review that truth in v2.

### 3. The current v2 dashboard shows symptoms of work, not the work objects themselves

Blockers, next steps, timeline rows, and artifact cards are useful summaries, but they do not replace:
- a Seat detail with role binding and delegation,
- a Session detail with prompt state and checkpoint continuity,
- a WorkItem detail with AC, status, review history, and linked artifacts,
- a Handoff detail with status strip, receipt, and expected outcome,
- an Acceptance/Gate detail with verdict and evidence.

### 4. Real repo production is already heavy in T3/T5, but v2 has no operational way to consume it

The current coordination process is generating large volumes of:
- task packets,
- fixes,
- verification artifacts,
- acceptance reviews,
- gate decisions.

Those are exactly the artifacts that should drive the operational loop in SeatLoom, yet v2 still treats them as optional side evidence instead of primary objects.

### 5. Some important gaps are upstream type-system gaps

This audit also found places where the shared model is not yet aligned with the authority model:
- `SessionStatus` in TS lacks `PromptBlocked`.
- TS `PromptState` omits available actions and assist budget.
- TS `EventType` lags Rust for prompt, delegation, review, and handoff-working events.
- TS `ObjectRef` cannot route `Delegation`.
- `InboxItem` is a TS model but not yet a first-class Rust/PG object.
- Playbook, skill, and budget truth are still embedded strings/fields instead of coherent persisted objects.

These should be corrected before v2 UI contracts are allowed to harden around the wrong schema.

## Recommended Fill Order

### P0-1. Replace the parallel `app-v2` view model with an authority-backed view model

Before more UI polishing, introduce one v2-facing projection layer derived from the shared frontend types and PG-backed truth. `AppV2.tsx` should stop owning product semantics through local mock-only interfaces.

### P0-2. Open the document-authority layer as a first-class UI surface

Add at minimum:
- Artifact/Document list,
- `template` / `subtype` filters,
- document reader,
- section anchors,
- linked object associations,
- parse / validity state.

This is the fastest path to making the already-produced repo truth visible.

### P0-3. Expose the four operational objects directly

The next object surfaces should be:
1. WorkItem,
2. Handoff,
3. Session,
4. Seat.

Each must show real status, linked artifacts, audit trail, and the missing role/delegation/prompt/review fields.

### P1-1. Close the action loop

After object entrypoints exist, add:
- Inbox queue,
- review / reissue trail,
- verification routing,
- acceptance verdict surface,
- gate-decision history.

This closes the core SeatLoom loop from issuance to validation.

### P1-2. Add continuity and prompt operations

Expose checkpoint history, continuity tiers, prompt-blocked triage, and assist budget visibility only after the core object routing is stable.

### P2. Add observability for reconcile, versions, pipelines, and retrieval explanation

These are important, but they should follow after users can already open and act on the main product objects.

## Pass / Hold Statement

- **Hold for data-coverage sign-off.**
- `ui/src/app-v2/` should currently be treated as a **visual and interaction direction prototype**, not yet as a truthful operational frontend for the SeatLoom authority model.
- The next implementation packets should prioritize **object entrypoints and authority-backed projection**, not additional mock refinement.
