# Acceptance: Architecture Prompt + Retrieval Delta

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-architecture-prompt-retrieval-delta-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, architecture, prompts, retrieval, sqlite, baseline-freeze |

## Verdict

**PASS**

Nimbus closed the four required delta items from Lyra's prior conditional acceptance:

1. `AD-012` now models wrapped-session interactive prompts as a first-class architecture concern.
2. `AD-010` is aligned back to the active event-first review/reissue contract.
3. `AD-011` freezes `SQLite FTS5` as the P0 persisted backend for retrieval `L1` and `L2`.
4. The delivery and prior baseline packet now expose the required dual-key `template + subtype` metadata.

The architecture support docs are now sufficient to close the **Architecture Design Baseline Freeze** milestone.

This verdict does **not** start broad implementation. `Product Baseline Freeze` remains open, and `SG-01 UI Contract Baseline` remains on hold.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`
- `docs/coordination/acceptance/2026-04-28-lyra-architecture-baseline-alignment-acceptance.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-architecture-prompt-retrieval-delta-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md`

## Coverage Matrix

| # | Requirement slice | Contract refs | Evidence paths | Result | Notes |
|---|---|---|---|---|---|
| 1 | Wrapped-session interactive prompts are modeled as a first-class architecture concern | `US-P0-11`; `INT-16`; `UX-12` | `docs/architecture-decisions.md` `AD-012`; `docs/architecture-design.md` §3.2, §3.3, §5.1, prompt flow after §12.3 | PASS | Prompt state, classification, policy, assist budget, bounded evidence window, and audit events are now explicit. |
| 2 | Review failure / reissue remains event-first, not status-expanded | `US-P0-05`; `INT-05`; `UX-05` | `docs/architecture-decisions.md` `AD-010`; `docs/architecture-design.md` §3.2 `workitem.rs`; §3.3 `event.rs` | PASS | Durable `Rejected` / `Rescoped` states are removed; verdict + rescope evidence now lives in events only. |
| 3 | Retrieval order and persisted backend are frozen for P0 | `US-P0-10`; `INT-13`; `E-02` | `docs/architecture-decisions.md` `AD-011`; `docs/architecture-design.md` §12.3 | PASS | L1 exact fields and L2 full-text are persisted in `SQLite FTS5`; in-memory index is cache-only. |
| 4 | `INT-13` through `INT-16` were re-read and reflected in the delivery | `INT-13`; `INT-14`; `INT-15`; `INT-16` | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md` §4 | PASS | The delivery explains the downstream architecture effect for each required flow. |
| 5 | Delivery-governance metadata is typed under the current artifact taxonomy | `docs/coordination/DOCUMENT_TEMPLATES.md`; delta packet done definition | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md`; `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` | PASS | Both delivery artifacts expose `Template: T3` and `Subtype: fix`, satisfying the scoped metadata requirement. |
| 6 | Architecture baseline can be frozen without changing product meaning | `docs/PRODUCT_TRUTH.md`; prior Lyra conditional acceptance | `docs/architecture-decisions.md`; `docs/architecture-design.md`; target delivery artifact | PASS | The delta patches raise architecture support docs to the active contract; no further P0 product drift remains in this review scope. |

## Findings

| ID | Severity | Finding | Why it matters | Follow-up owner |
|---|---|---|---|---|
| APD-01 | Low | `docs/architecture-design.md` still lacks an explicit `### 12.4 Prompt Engine Architecture (AD-012)` heading even though the prompt-flow content is present, and the footer text appears twice. | This weakens section navigation and future citations, but it does not change the architecture meaning or the acceptance boundary. | Nimbus, next architecture-doc maintenance pass |
| APD-02 | Low | `docs/architecture-decisions.md` header summary still says `AD-008–AD-011 added` although `AD-012` now exists. | The header summary is slightly stale and may confuse readers scanning the decision set. | Nimbus, next architecture-doc maintenance pass |

## Post-acceptance Closure Note

On `2026-04-28`, Nimbus completed both low-severity hygiene follow-ups:

- `docs/architecture-design.md` now carries the explicit `### 12.4 Prompt Engine Architecture (AD-012)` heading, and the duplicated footer was removed.
- `docs/architecture-decisions.md` header summary now reflects `AD-008–AD-012`.

These edits close `APD-01` and `APD-02`. There are no remaining architecture-baseline follow-up items under this acceptance thread.

## Go / No-Go Recommendation

- **Architecture Design Baseline Freeze:** **GO**
- **Broad implementation handoff:** **NO-GO**
- **Reason broad implementation stays blocked:** `Product Baseline Freeze` is still open, and `SG-01 UI Contract Baseline` remains hold-level unresolved.
- **Next Nimbus action:** no blocking delta required; only low-priority editorial cleanup remains.

## Milestone Decision

| Milestone | Decision | Notes |
|---|---|---|
| `Architecture Design Baseline Freeze` | CLOSED / PASS | Architecture support docs now align to the active v0.5 contract set for the reviewed scope. |
| `Product Baseline Freeze` | OPEN | Still waiting on non-architecture product/interaction/UX closure. |
| `Nimbus Implementation Handoff` | HOLD | Remains gated by product baseline and UI contract status, not by the architecture delta. |

## Evidence Paths

- Prior Lyra conditional acceptance: `docs/coordination/acceptance/2026-04-28-lyra-architecture-baseline-alignment-acceptance.md`
- Nimbus delta task packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-architecture-prompt-retrieval-delta-v1.md`
- Nimbus delta delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md`
- Architecture decisions: `docs/architecture-decisions.md`
- Architecture design: `docs/architecture-design.md`
- Active contract entrypoint: `docs/PRODUCT_TRUTH.md`

## Follow-up

Lyra records this acceptance as the close of the architecture-baseline review thread. Future Nimbus architecture work may treat this baseline as fully clean and frozen unless product truth changes.
