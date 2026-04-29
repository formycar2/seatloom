# Task: Architecture Baseline Alignment

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | TASK-nimbus-2026-04-28-architecture-baseline-alignment-001 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-29 12:00 Asia/Shanghai |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md` |
| supersedes | - |
| tags | architecture, alignment, product-truth, data-engine, artifacts, continuity |

## Objective

Align the architecture support docs to the active v0.5 product contract set before broad implementation starts.

You own architecture meaning inside approved product constraints. Do not rewrite product scope; resolve architecture drift, surface blockers, and patch the support docs only where the active contract already provides the answer.

## Micro-brief

1. Use `docs/PRODUCT_TRUTH.md` as the only entrypoint.
2. Treat `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, and `docs/acceptance-spec-v1.1.md` as the active product contract set.
3. Treat `docs/architecture-decisions.md` and `docs/architecture-design.md` as support docs that must align upward to the product contract, not compete with it.
4. Do not start broad coding from stale architecture assumptions.
5. Publish one delivery artifact in English and keep terminal progress updates in Chinese.

## Input Files

Read only these files unless a required fact is missing:

1. `docs/PRODUCT_TRUTH.md` - active entrypoint, precedence, archive boundaries
2. `docs/prd-v0.5.md` - active value/scope contract
3. `docs/interaction-spec-v1.1.md` - active behavior/flow contract
4. `docs/ux-spec-v1.1.md` - only sections needed to validate visible architecture implications
5. `docs/acceptance-spec-v1.1.md` - pass/fail contract for architecture-facing behavior
6. `docs/architecture-decisions.md` - decision-level support doc to align
7. `docs/architecture-design.md` - system-shape support doc to align
8. `docs/coordination/DOCUMENT_TEMPLATES.md` - canonical `template+subtype` model and subtype allow-list
9. `docs/coordination/reviews/2026-04-28-process-mapping-review.md` - recovery input for delegation, supervisor continuity, and reject/re-scope gaps

If a necessary rule is still missing after reading the files above, log a blocker instead of inventing product meaning.

## Known Drift To Resolve

At minimum, inspect and resolve or explicitly block the following drift:

1. `Artifact.kind` / `ArtifactKind` still reflects the old generic artifact model and does not express the active dual-key `template+subtype` contract or subtype validation path.
2. Continuity architecture still centers on `ContextPack` terminology and does not cleanly express the active Data Engine model, tiered continuity packs, or supervisor-vs-worker continuity separation.
3. Seat architecture does not yet clearly encode the approved three-layer model: durable seat identity, project role binding, and collaboration-mode / execution-template layer.
4. Scoped delegation overlay is not explicit enough to protect original seat identity while showing acting-seat work and audit trail.
5. WorkItem review failure, return, reissue, and re-scope semantics are behind the active review flows and process-mapping findings.
6. Retrieval architecture does not clearly freeze the required order: structured lookup -> full-text -> semantic -> explanation.
7. Artifact review/search architecture does not yet state subtype-aware rendering, filtering, routing, and degraded behavior when metadata is invalid.
8. P1 boundaries for handoff `working` and optional live activity mode are not clearly separated from P0 replay truth.

## Required Work

### P0

- Produce a drift audit between the active contract set and both architecture support docs.
- Update `docs/architecture-design.md` where the product contract already gives a clear answer.
- Update `docs/architecture-decisions.md` where a decision record is now stale or missing.
- Freeze the canonical architecture contract for:
  - dual-key artifact typing with `template+subtype`
  - subtype allow-list validation and degraded fallback behavior
  - Data Engine retrieval order and layer responsibilities
  - structured checkpoints, tiered continuity packs, budget enforcement, and deterministic fallback order
  - worker continuity vs supervisor continuity separation
  - seat three-layer architecture and scoped delegation overlay
  - artifact review workspace implications for storage, search, and routing

### P1

- Clarify the architecture stance for:
  - Playbook vs Seat Skill separation
  - read-only Execution Template inspector
  - Handoff `working` projection and optional live activity mode
  - reject / reissue / re-scope handling where event-vs-state choice is still a design concern

## Done Definition

- [ ] Delivery artifact is published at the required path below.
- [ ] Delivery artifact separates facts, required patches, open blockers, and implementation implications.
- [ ] `docs/architecture-design.md` is either aligned to the active contract or the unresolved drift is explicitly blocked with rationale.
- [ ] `docs/architecture-decisions.md` is either aligned to the active contract or the unresolved drift is explicitly blocked with rationale.
- [ ] Delivery cites exact file targets and, if edits are made, the exact sections or models changed.
- [ ] No product-scope invention or user-facing behavior rewrite is introduced.
- [ ] Any unresolved ambiguity is framed as a blocker for Lyra/Aegis rather than silently decided in engineering.

## Acceptance Spec Reference

Use these acceptance anchors while aligning the architecture docs:

- `docs/acceptance-spec-v1.1.md` - `US-P0-04`, `US-P0-05`, `US-P0-06`, `US-P0-07`, `US-P0-08`, `US-P0-09`, `US-P0-10`
- `docs/acceptance-spec-v1.1.md` - `US-P1-04`, `US-P1-07`
- `docs/acceptance-spec-v1.1.md` - `P-08`, `P-10`, `E-02`, `E-04`, `E-08`

## Constraints

- Need-to-know default: do not broad-read unrelated UI/code files.
- Token-efficient execution: delta-only notes, compact rationale, exact file targets.
- Architecture docs may constrain implementation, but may not redefine product meaning.
- Do not start broad implementation from this packet.
- Preserve any unrelated in-progress work in the repo.
- Durable artifacts must be English; terminal progress summaries must be Chinese.

## Delivery Instructions

Publish completion as:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md`

Required delivery sections:

1. Micro-brief
2. Facts
3. Patches made
4. Remaining blockers / disputed points
5. Exact file targets for follow-on engineering work
6. Commands run + results
7. Recommendation to Lyra: architecture baseline ready / conditionally ready / not ready

## Handoff Rule

After publishing the delivery artifact, send Lyra only a short Chinese terminal summary with:

- verdict,
- exact changed files,
- blockers if any,
- artifact path.
