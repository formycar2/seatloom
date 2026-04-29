# Task: Architecture Prompt + Retrieval Delta

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | TASK-nimbus-2026-04-28-architecture-prompt-retrieval-delta-001 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-29 18:00 Asia/Shanghai |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/acceptance/2026-04-28-lyra-architecture-baseline-alignment-acceptance.md` |
| supersedes | - |
| tags | architecture, prompts, retrieval, sqlite, delta, alignment |

## Objective

Close the remaining architecture-baseline gaps after Lyra's conditional acceptance review so the architecture support docs can be frozen without contradicting the active v0.5 contract.

## Micro-brief

1. Use `docs/PRODUCT_TRUTH.md` as the only entrypoint.
2. Treat the Lyra acceptance artifact as the authoritative review boundary for this delta.
3. Do not reopen already-accepted alignment work unless the new prompt/retrieval delta requires a local adjustment.
4. Do not expand product scope. Patch the support docs upward to the active contract only.
5. Persistent artifacts stay English. Terminal progress updates stay Chinese.

## Read Scope

Read only these files unless a required fact is missing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md` — explicitly re-read `INT-13`, `INT-14`, `INT-15`, and `INT-16`
4. `docs/ux-spec-v1.1.md` — focus on `UX-06`, `UX-10`, `UX-11`, and `UX-12`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/architecture-decisions.md`
7. `docs/architecture-design.md`
8. `docs/coordination/DOCUMENT_TEMPLATES.md`
9. `docs/coordination/acceptance/2026-04-28-lyra-architecture-baseline-alignment-acceptance.md`
10. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md`

If any fact is still missing after this set, log it as a blocker instead of inventing behavior.

## Required Work

### P0

1. **Freeze BLOCKER-001 into the architecture docs**
   - Adopt `SQLite FTS5` as the persisted backend for retrieval `L1` structured index and `L2` full-text search.
   - Clarify that any in-memory index is cache/projection only, not the authoritative persisted store.
   - Keep `L3` semantic retrieval and `L4` explanation as P1.

2. **Add the missing interactive-prompt architecture contract**
   - Represent wrapped-session prompt blocking as a first-class architecture state.
   - Add a deterministic prompt-classification model covering `deterministic`, `wizard/menu`, `freeform`, and `sensitive`.
   - Add a prompt policy model covering `auto allowed`, `needs approval`, and `human required`.
   - Add a bounded evidence-window representation so prompt handling never depends on rereading long terminal histories.
   - Add architecture support for the user actions `Approve`, `Human takeover`, `Supervisor assist`, and `Stop`.
   - Add assist token-budget and step-budget constraints plus the sensitive-prompt prohibition on Supervisor assist.
   - Add the required audit events `prompt.detected` and `prompt.input_injected` plus any minimal supporting event payload structure.

3. **Correct the review-failure lifecycle back to the active event-first contract**
   - Remove or replace durable `Rejected` / `Rescoped` WorkItem states unless product truth is formally changed first.
   - Keep explicit review-verdict and scope-change evidence through event payloads and existing lifecycle re-entry.

### P1

4. **Normalize delivery-governance metadata**
   - Reissue or patch the delivery artifact header to valid dual-key metadata.
   - Explicitly state that `INT-13` through `INT-16` were re-read for the updated delivery.

## Done Definition

- [ ] `docs/architecture-decisions.md` no longer leaves BLOCKER-001 open and explicitly records `SQLite FTS5` for P0 retrieval L1/L2.
- [ ] `docs/architecture-design.md` can represent `INT-16` / `UX-12` prompt handling end-to-end without depending on long-log rereads.
- [ ] WorkItem review failure / reissue modeling no longer contradicts the active event-first contract.
- [ ] The updated delivery artifact is compliant with `template + subtype` metadata.
- [ ] The delivery artifact explicitly cites the re-read `INT-13`-`INT-16` coverage.
- [ ] No new product meaning is invented beyond the active contract set.

## Constraints

- Need-to-know default: stay inside the files listed above.
- Token-efficient execution: delta-only notes, exact sections, no broad restatement of already-accepted architecture.
- Architecture docs may constrain implementation, but may not redefine user-visible behavior.
- Do not start broad code implementation from this packet.
- Preserve unrelated repo changes.

## Delivery Instructions

Publish completion as:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1.md`

Required delivery sections:

1. Micro-brief
2. Facts
3. Exact doc patches
4. How `INT-13`-`INT-16` changed the architecture text
5. Any remaining blockers
6. Commands run + results
7. Recommendation to Lyra: baseline ready / conditionally ready / not ready

## Handoff Rule

After publishing the delivery artifact, send Lyra only a short Chinese terminal summary with:

- verdict,
- exact files changed,
- blockers if any,
- artifact path.
