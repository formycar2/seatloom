# NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1

| Field | Value |
|-------|-------|
| Document | Architecture Baseline Alignment — Delivery |
| Template | T3 |
| Subtype | fix |
| Status | Delivered |
| Issuer | Lyra |
| Assignee | Nimbus |
| Issued | 2026-04-28 |
| Delivered | 2026-04-29 |
| Task Ref | NIMBUS-2026-04-28-architecture-baseline-alignment-v1 |

---

## Micro-Brief

Aligned `docs/architecture-design.md` and `docs/architecture-decisions.md` to the active v0.5 contract set (PRODUCT_TRUTH.md → prd-v0.5.md, interaction-spec-v1.1.md, acceptance-spec-v1.1.md, DOCUMENT_TEMPLATES.md). All 8 known drift areas from the task packet have been resolved. One open blocker (storage backend for retrieval engine) has been surfaced for Lyra/Aegis decision.

---

## Facts

### Input Files Read

| File | Lines Read | Role |
|------|-----------|------|
| `docs/PRODUCT_TRUTH.md` | All | Single entrypoint, precedence rules |
| `docs/prd-v0.5.md` | All (453) | Primary product contract |
| `docs/interaction-spec-v1.1.md` | 1–400 (INT-01–INT-12) | Interaction flow contract |
| `docs/acceptance-spec-v1.1.md` | All | P0/P1 acceptance contract |
| `docs/architecture-decisions.md` | All (AD-001–AD-007) | Target file — pre-patch |
| `docs/architecture-design.md` | All (915 lines) | Target file — pre-patch |
| `docs/coordination/DOCUMENT_TEMPLATES.md` | §1–3, §10–13 | Subtype allow-list authority |
| `docs/coordination/reviews/2026-04-28-process-mapping-review.md` | All | Independent gap validation |
| `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-architecture-baseline-alignment-v1.md` | All | Task packet |

### Pre-Patch Drift Summary

All 8 drift areas confirmed present before patching:

| # | Drift Area | Stale Element | v0.5 Contract Violated |
|---|-----------|---------------|----------------------|
| 1 | `ArtifactKind` flat enum | `artifact.rs` `kind: ArtifactKind` | prd-v0.5 §6.5, DOCUMENT_TEMPLATES §11.1, E-08 |
| 2 | `context/` ContextPack model | AD-004 title + content, module path | prd-v0.5 §6.1 Data Engine, US-P1-04 |
| 3 | Flat `Seat` struct | `seat.rs` single struct + `SeatRole` enum | prd-v0.5 §6.2, US-P0-04 |
| 4 | No delegation overlay | No `SeatDelegation` struct | prd-v0.5 §6.2 Rule 3 |
| 5 | Missing `Rejected`/`Rescoped` WorkItem states | `WorkItemStatus` enum | process-mapping-review Gap 3, INT-05 |
| 6 | No retrieval layer order | No retrieval module | prd-v0.5 §6.1, E-02 |
| 7 | No subtype-aware artifact routing | `list_artifacts(filter?)` generic | prd-v0.5 §6.5, INT-13 |
| 8 | `HandoffStatus.Working` missing / P1 boundary undocumented | No `Working` state, no P1 annotation | US-P1-06 |

---

## Patches Made

### `docs/architecture-decisions.md`

| Change | Details |
|--------|---------|
| Header | Added `对齐版本: v0.5 contract set (AD-008–AD-011 added)` |
| AD-003 | `context: ContextPack compiler` → `data_engine: Data Engine (Pack Engine, Retrieval, Gate, Route)` |
| **AD-004** (full rewrite) | Renamed to "Pack Engine — Worker/Supervisor Pack Types, Tiered Context, Deterministic Assembly". Added Worker vs Supervisor pack table, Tier 0/1/2 content table, worker budget 8K / supervisor budget 32K (P1), updated Selector (Tier 0 from identity.yaml), added Tier 0 missing check to Verifier, added fallback order. Preserved LaunchPack file requirement. |
| **AD-008** (new) | Dual-key artifact typing: `template + subtype`, `ArtifactTemplate` T1–T7 enum, `SystemArtifactKind`, degraded fallback behavior. |
| **AD-009** (new) | Seat three-layer model: `SeatIdentity` (global), `ProjectRoleBind` (per-project), `SeatDelegation` overlay. Storage paths, delegation status machine, `SeatDelegationIssued`/`SeatDelegationClosed` Ledger events, Timeline display format. |
| **AD-010** (new) | WorkItem state machine extension: `Rejected` and `Rescoped` states, state transition table, new Ledger events `WorkItemRejected` (with `linked_evidence_artifact_id`) and `WorkItemRescoped`. |
| **AD-011** (new) | Retrieval engine fixed layer sequence: L1 structured → L2 full-text → L3 semantic (P1) → L4 LLM explanation (P1). P0 requirement for INT-13. BLOCKER-001 surfaced. |

### `docs/architecture-design.md`

| Section | Change |
|---------|--------|
| Header | `更新时间: 2026-04-28`, `对齐版本: v0.5 contract set (NIMBUS-2026-04-28)` |
| §1 System overview | `context/ ContextPack compiler` → `data_engine/ Data Engine (Pack Engine, Retrieval, Gate, Route)` |
| §2 Cargo workspace | Replaced `context/` (5 files) with `data_engine/` tree (14 files): `pack_engine/` subdir + `retrieval.rs`, `gate_engine.rs`, `route_engine.rs`, `budget_enforcer.rs`, `isolation.rs`, `audit.rs` |
| §3.1 ID system | Added `define_id!(DelegationId, "del");` |
| §3.2 `seat.rs` | Replaced flat `Seat` + `SeatRole` + `SeatStatus` with: `SeatIdentity` (Layer 1), `ProjectRoleBind` (Layer 2, with `collaboration_template_ref` + `active_delegation_id`), `SeatDelegation` (overlay), `DelegationStatus` enum. `SeatRole`/`SeatStatus` preserved. |
| §3.2 `artifact.rs` | Replaced `kind: ArtifactKind` + flat `ArtifactKind` enum with: `template: Option<ArtifactTemplate>`, `subtype: Option<String>`, `subtype_valid: Option<bool>`, `system_kind: Option<SystemArtifactKind>`. Added `ArtifactTemplate` enum (T1–T7). Added `SystemArtifactKind` enum (preserving DiffSummary/TestReport/BugReport/CheckpointSummary, adding WorkerContinuityPack/SupervisorContinuityPack). |
| §3.2 `workitem.rs` | Added `Rejected, Rescoped` to `WorkItemStatus` with inline documentation comments. |
| §3.2 `handoff.rs` | Added `Working` to `HandoffStatus` with P1 annotation comment (US-P1-06). |
| §3.3 `event.rs` | Added `HandoffWorking`, `WorkItemRejected`, `WorkItemRescoped`, `SeatDelegationIssued`, `SeatDelegationClosed`, `ReviewVerdictIssued` to `EventType`. Added `Delegation(DelegationId)` to `ObjectRef`. |
| §5.1 IPC commands | `list_artifacts(filter?)` → `list_artifacts(template?, subtype?, workitem_id?)`. Added `validate_artifact_subtype(id)`. Added `Delegation` command group: `create_delegation`, `close_delegation`, `list_delegations`. |
| §6.1 Storage layout | `seats/<name>/profile.yaml` → `seats/<name>/identity.yaml` + `seats/<name>/role-bindings/<project>.yaml`. Added `delegations/del_xxxxxxxx.yaml`. Updated `sessions/.../context/` to show `launch_pack.md` (P0) + `supervisor_pack.md` (P1). |
| §6.2 `project.yaml` | `context_pack.default_budget_tokens` → `pack_engine.worker_budget_tokens` (8192) + `pack_engine.supervisor_budget_tokens` (32768, P1). |
| §6.3 Seat profile | Replaced single `profile.yaml` example with two-file example: `identity.yaml` (Layer 1) + `role-bindings/seatloom.yaml` (Layer 2). |
| §11 Implementation | Phase 2 reference: `context/ (ContextPack compiler)` → `data_engine/pack_engine/ (Worker Continuity Pack, Tier 0/1/2)`. |
| **§12 (new)** | Data Engine Architecture section: module tree (12.1), continuity pack tier table + fallback order (12.2), retrieval layer contract with L1–L4 flow diagram + BLOCKER-001 note (12.3). |

---

## Remaining Blockers

### BLOCKER-001: Retrieval Engine Storage Backend — Decision Required

**Status**: Open. Requires Lyra/Aegis decision before retrieval engine architecture can be frozen.

**Issue**: `docs/architecture-design.md` (new §12.3) assumes in-memory index + filesystem grep as MVP L1/L2 retrieval backend. `docs/coordination/DOCUMENT_TEMPLATES.md §10` defines a PostgreSQL + pgvector schema for the structured document index. These are incompatible.

**Options**:
1. **SQLite FTS5** (embedded, no external service): Lower ops burden, no pgvector support, sufficient for L1+L2 P0 acceptance. Recommended for MVP.
2. **PostgreSQL + pgvector** (DOCUMENT_TEMPLATES schema): Full semantic capability (L3), requires external DB service, higher ops complexity.
3. **In-memory index (current MVP assumption)**: Simplest, non-persistent between restarts, insufficient for INT-13 at scale.

**Blocked work**: `data_engine/retrieval.rs` storage backend, `data_engine/audit.rs` structured index writes, L1/L2 acceptance test implementation.

---

## P1 Stance Documentation (Recorded, Not Implemented)

The following P1 concerns were noted in architecture docs but not designed in this pass:

| Area | Stance | Reference |
|------|--------|-----------|
| Supervisor Continuity Pack | `supervisor_pack.md` added to storage layout; `SupervisorContinuityPack` in `SystemArtifactKind`; budget slot in `project.yaml`. Full assembly logic is P1. | AD-004, US-P1-04 |
| `HandoffStatus::Working` | Added to enum with P1 annotation. No Tauri event or IPC changes made for live activity projection. | US-P1-06 |
| Semantic retrieval (L3) | Named in retrieval layer contract. No embedding model or vector store design in this pass. | AD-011 |
| Playbook vs Seat Skill separation | prd-v0.5 §6.3 distinguishes Playbook from Skill. No Rust type changes made; deferred to separate Playbook architecture work. | prd-v0.5 §6.3 |
| Execution Template inspector | Read-only reference in `ProjectRoleBind.collaboration_template_ref`. No `ExecutionTemplate` struct or inspector IPC in this pass. | prd-v0.5 §6.2 |

---

## Files Modified

```
docs/architecture-decisions.md
  - Header: version alignment
  - AD-003: data_engine reference
  - AD-004: full rewrite (Pack Engine)
  - AD-008–AD-011: new decision records

docs/architecture-design.md
  - Header: date + version
  - §1, §2: data_engine module reference
  - §3.1: DelegationId
  - §3.2: SeatIdentity, ProjectRoleBind, SeatDelegation, ArtifactTemplate,
           SystemArtifactKind, WorkItemStatus (+Rejected/Rescoped),
           HandoffStatus (+Working/P1)
  - §3.3: EventType (+6 new variants), ObjectRef (+Delegation)
  - §5.1: Artifact IPC filters, Delegation IPC group
  - §6.1–§6.3: storage layout, project.yaml, seat profile
  - §11: Phase 2 implementation reference
  - §12 (new): Data Engine Architecture
```

---

## Recommendation

1. **Resolve BLOCKER-001** (storage backend) before `data_engine/retrieval.rs` implementation begins. SQLite FTS5 is the recommended MVP path.

2. **Seat IPC alignment**: `list_seats()` → `Vec<Seat>` in §5.1 still returns an undifferentiated type. When `SeatIdentity` + `ProjectRoleBind` are implemented in Rust, the IPC should return a `SeatView { identity, role_binding, active_delegation? }` projection. This is not a blocker but should be addressed in Phase 1 IPC work.

3. **Frontend type mirroring**: `ui/src/types/index.ts` will need to mirror all changed structs. This is Mira's scope; the architecture-design.md changes provide the authoritative Rust definition.

4. **interaction-spec-v1.1.md lines 401–526** (INT-13–INT-15) were not read in this pass. INT-13 (exact evidence search) is the primary retrieval acceptance flow. Recommend reading before retrieval engine architecture is finalized.

---

*Delivery prepared by Nimbus. Aligned to docs/PRODUCT_TRUTH.md and v0.5 active contract set.*
