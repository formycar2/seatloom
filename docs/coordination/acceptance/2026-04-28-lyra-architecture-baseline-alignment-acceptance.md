# Acceptance: Architecture Baseline Alignment

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-architecture-baseline-alignment-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` |
| verdict | CONDITIONAL |
| tags | acceptance, architecture, data-engine, retrieval, prompts |

## Verdict

**CONDITIONAL**

Nimbus's delivery is accepted as a substantial partial alignment pass. The repaired baseline now correctly carries the Data Engine rename, worker vs supervisor continuity separation, seat three-layer modeling, delegation overlay, dual-key Artifact typing, and the fixed retrieval-layer order.

However, the architecture baseline is **not** yet frozen for implementation handoff. Two active-contract gaps remain:

1. the new P0 interactive-prompt contract (`US-P0-11`, `INT-16`, `UX-12`) is still missing from the architecture support docs, and
2. the delivery adds durable `Rejected` / `Rescoped` WorkItem states even though the active review/reissue contract still uses an event-first path through the existing lifecycle.

Because of those gaps, this review issues a conditional acceptance of the completed subset, but a **No-Go** for final architecture freeze until the delta packet closes.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`
- `docs/coordination/DOCUMENT_TEMPLATES.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md`

Note: `docs/acceptance-spec-v1.1.md` does not yet enumerate `US-P0-11`; this review therefore applies the newer PRD / Interaction / UX contract directly for the interactive-prompt delta already declared active by Lyra and Mr. Zhang.

## Coverage Matrix

| # | Requirement slice | Contract refs | Evidence paths | Result | Notes |
|---|---|---|---|---|---|
| 1 | Data Engine / Pack Engine baseline replaces the old ContextPack-centered model | `docs/prd-v0.5.md` §6.1; `docs/architecture-decisions.md` `AD-004` | `docs/architecture-decisions.md`; `docs/architecture-design.md` §2, §12.1-12.2 | PASS | Worker vs supervisor continuity separation and tiered pack assembly are now explicit. |
| 2 | Seat three-layer model and scoped delegation overlay align to the active seat contract | `US-P0-03`; `US-P0-04`; `INT-03`; `UX-04` | `docs/architecture-decisions.md` `AD-009`; `docs/architecture-design.md` §3.2 `seat.rs`; §5.1; §6.1-6.3 | PASS | The architecture now distinguishes durable identity, per-project role binding, and delegation overlay. |
| 3 | Artifact object model follows the `template+subtype` dual-key taxonomy | `docs/prd-v0.5.md` §6.5; `docs/coordination/DOCUMENT_TEMPLATES.md` §11.1; `E-08` | `docs/architecture-decisions.md` `AD-008`; `docs/architecture-design.md` §3.2 `artifact.rs`; §5.1 | PASS | Dual-key typing, allow-list validation hook, and degraded fallback are correctly represented. |
| 4 | Retrieval order is fixed as L1 structured -> L2 full-text -> L3 semantic -> L4 explanation | `US-P0-10`; `INT-13`; `E-02` | `docs/architecture-decisions.md` `AD-011`; `docs/architecture-design.md` §12.3 | PASS with Lyra decision | The order is correct. `BLOCKER-001` is resolved by this review: P0 backend = SQLite FTS5. |
| 5 | `working` remains a P1 Handoff extension, not a P0 replay replacement | `US-P1-06`; `INT-15` | `docs/architecture-design.md` §3.2 `handoff.rs`; §3.3 `event.rs` | PASS | The P1 boundary is visible and does not redefine replay truth. |
| 6 | Wrapped-session interactive prompts are modeled as a first-class architecture concern | `US-P0-11`; `INT-16`; `UX-12` | `docs/architecture-decisions.md`; `docs/architecture-design.md`; Nimbus delivery artifact | FAIL | No architecture decision or system-shape section yet covers prompt classification, bounded evidence windows, action policy, assist budgets, or required prompt audit events. |
| 7 | Review failure / reissue stays aligned to the active event-first lifecycle contract | `US-P0-05`; `INT-05`; `UX-05` | `docs/interaction-spec-v1.1.md` INT-05; `docs/architecture-decisions.md` `AD-010`; `docs/architecture-design.md` §3.2 `workitem.rs`; §3.3 `event.rs` | FAIL | The architecture currently adds durable `Rejected` / `Rescoped` states that are not defined in the active PRD / interaction / UX contract set. |
| 8 | Delivery itself is reviewable under current typed-artifact governance rules and active contract coverage | `docs/PRODUCT_TRUTH.md`; `docs/coordination/DOCUMENT_TEMPLATES.md`; task packet done definition | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` | PARTIAL | The artifact is useful, but its header uses legacy combined metadata and it explicitly did not read `INT-13`-`INT-15`, which reduces confidence for the retrieval claim. |

## Findings

| ID | Severity | Finding | Why it matters | Required target |
|---|---|---|---|---|
| ABA-01 | Critical | The architecture support docs do not yet model the new interactive-prompt baseline (`US-P0-11`, `INT-16`, `UX-12`). | SeatLoom cannot freeze P0 wrapped-session behavior without first-class prompt state, policy, bounded evidence windows, and auditability. This is now active contract, not a future nice-to-have. | `docs/architecture-decisions.md`; `docs/architecture-design.md` §3.2 `session.rs`, §3.3 `event.rs`, §5.1 IPC, §12 Data Engine |
| ABA-02 | High | `Rejected` / `Rescoped` were added as durable WorkItem states even though `INT-05` still routes failed review through verdict + scope-change events and then back into the existing lifecycle. | Architecture docs may constrain implementation, but they may not silently redefine the user-facing lifecycle. This creates contract drift right at the baseline layer. | `docs/architecture-decisions.md` `AD-010`; `docs/architecture-design.md` §3.2 `workitem.rs`; §3.3 `event.rs` |
| ABA-03 | Medium | The delivery artifact states that `INT-13`-`INT-15` were not read, despite `INT-13` being the core retrieval acceptance flow that justifies `AD-011`. | The retrieval architecture claim is directionally right, but not yet fully reviewed against the active flow contract. Freeze should happen only after that read gap is closed in writing. | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md`; `docs/architecture-design.md` §12.3 |
| ABA-04 | Low | The delivery artifact header is not compliant with the current dual-key coordination-document format (`template` + `subtype`). | Typed artifact automation should parse Nimbus delivery packets the same way it parses other coordination documents. | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` |

## BLOCKER-001 Decision

### Decision

**P0 storage backend for retrieval L1/L2 is `SQLite FTS5`.**

### Why this is the right P0 choice

| Dimension | Decision rationale |
|---|---|
| Product fit | SeatLoom is local-first in P0. SQLite keeps structured index + full-text retrieval embedded and persistent without introducing an external service dependency. |
| Acceptance fit | `US-P0-10` / `INT-13` require deterministic exact evidence retrieval before semantic or LLM help. SQLite FTS5 satisfies that requirement with restart persistence. |
| Operational burden | SQLite is materially lighter than PostgreSQL for MVP and avoids server setup becoming a hidden blocker for prototype or early product validation. |
| Upgrade path | L3 semantic retrieval remains P1. SQLite for P0 does not prevent a later PostgreSQL / pgvector or hybrid retrieval architecture if and when multi-user or cloud-backed value is proven. |
| Rejected alternatives | In-memory index is too fragile for restart persistence and scale-up reviewability. PostgreSQL FTS is too heavy for the current local-first MVP boundary. |

### Freeze implications

1. `docs/architecture-decisions.md` `AD-011` should replace the open blocker with a concrete SQLite FTS5 decision.
2. `docs/architecture-design.md` §12.3 should define SQLite as the persistent source of truth for L1 structured fields and L2 full-text search.
3. Any in-memory index may remain only as a cache or warm projection, not as the authoritative persisted store.
4. The PostgreSQL schema examples in `docs/coordination/DOCUMENT_TEMPLATES.md` should be treated as logical / future storage guidance, not as a P0 runtime requirement.

## Required Fixes for Nimbus

| # | Owner | Required fix | Exact targets | Done definition |
|---|---|---|---|---|
| 1 | Nimbus | Add an architecture-level interactive-prompt model aligned to `US-P0-11` / `INT-16` / `UX-12`. | `docs/architecture-decisions.md`; `docs/architecture-design.md` §3.2 `session.rs`, §3.3 `event.rs`, §5.1 IPC, §12 Data Engine | Prompt blocked state, prompt classification, policy gating, bounded evidence window, assist step/token budget, `Approve` / `Human takeover` / `Supervisor assist` / `Stop`, and required audit events are all represented in docs. |
| 2 | Nimbus | Replace unsupported durable `Rejected` / `Rescoped` lifecycle states with an event-first review/reissue architecture unless product truth is formally changed first. | `docs/architecture-decisions.md` `AD-010`; `docs/architecture-design.md` §3.2 `workitem.rs`; §3.3 `event.rs` | Failed review is modeled through explicit verdict and scope-change events plus existing lifecycle re-entry, without silent lifecycle expansion. |
| 3 | Nimbus | Freeze `BLOCKER-001` into the architecture docs as `SQLite FTS5` for P0 L1/L2 retrieval. | `docs/architecture-decisions.md` `AD-011`; `docs/architecture-design.md` §12.3 | The blocker is removed, the persisted backend is explicit, and the role of in-memory caches versus persisted search state is unambiguous. |
| 4 | Nimbus | Re-issue the delivery artifact with compliant dual-key metadata and explicit confirmation that `INT-13`-`INT-16` were re-read. | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` or superseding delivery file | Delivery metadata is machine-parseable and the active flow read coverage is complete. |

## Go / No-Go Recommendation

- **Architecture baseline freeze:** **NO-GO**
- **Broad implementation handoff:** **NO-GO**
- **Narrow follow-up delta for Nimbus:** **GO**

The current delivery should be treated as a strong partial alignment pass, not as the final architecture baseline freeze.

## Evidence Paths

- Delivery reviewed: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md`
- Active contract entrypoint: `docs/PRODUCT_TRUTH.md`
- Product contract: `docs/prd-v0.5.md`
- Flow contract: `docs/interaction-spec-v1.1.md`
- UX contract: `docs/ux-spec-v1.1.md`
- Acceptance contract used as baseline: `docs/acceptance-spec-v1.1.md`
- Architecture support docs: `docs/architecture-decisions.md`, `docs/architecture-design.md`

## Follow-up

A narrow Nimbus delta packet is issued in parallel to close the remaining architecture baseline gaps and encode the retrieval backend decision.
