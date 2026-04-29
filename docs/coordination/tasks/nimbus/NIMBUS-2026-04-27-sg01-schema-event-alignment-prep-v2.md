# NIMBUS-2026-04-27-sg01-schema-event-alignment-prep-v2

| Field | Value |
|---|---|
| Owner | Nimbus |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-30 12:00 |
| Acceptance owner | Lyra |
| Current handoff status | NO-GO for full implementation |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. Produce the engineering alignment memo needed for SG-01 recovery.
2. Stay packet-driven; do not start full implementation from the rejected UI baseline.
3. Freeze recommendations for Handoff status, `review.requested`, LaunchPack fallback, and Inbox projection.
4. Separate facts, recommendations, and blockers.
5. Persist the memo in English; keep terminal progress summaries in Chinese.

## 2. Contract pack

Mandatory reads, in this order:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`
4. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
5. `docs/coordination/COORDINATION_RULES.md`
6. `docs/archive/product-history/mvp-scenarios.md`
7. `docs/architecture-decisions.md`
8. `docs/architecture-design.md`
9. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`
10. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`

## 3. Need-to-know scope

Read only:

- this packet
- the contract pack above
- exact schema/event/type files needed to support the memo

Do not consume by default:

- unrelated UI files
- full worktree history
- unrelated seat packets or broad repo exploration

If a required fact is missing, record a blocker instead of inferring a new contract.

## 4. Token-efficiency rules

| Item | Rule |
|---|---|
| Input budget target | <= 6,000 tokens total live context |
| Output budget target | <= 1,000 words in the memo body, excluding tables |
| Sync policy | Delta-only from `NIMBUS-2026-04-27-sg01-schema-event-alignment-prep-v1.md` |
| Preferred output | facts, recommended canonical model, file targets, blockers |
| Truncation strategy | If output grows too long, keep P0 recommendations and file targets; compress rationale into short bullets |

## 5. Delta from v1 packet

New requirements in this v2 packet:

- `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md` is now mandatory authority for execution behavior.
- Need-to-know scope and token budgets are explicit.
- Terminal progress must be Chinese; durable artifacts must remain English.
- The memo must be structured for minimum verifiable output, not broad narrative.

## 6. Required work

| Priority | Requirement | Expected coverage |
|---|---|---|
| P0 | Freeze recommended canonical Handoff status enum | Map current drift and recommend one enum across PRD, architecture reference, TS types, and mock data |
| P0 | Freeze recommended `review.requested` event mapping | Show how Inbox and Timeline can project from one event model |
| P0 | Clarify LaunchPack fallback handling | Recommend one canonical fallback policy consistent with the active product contract |
| P0 | Define Inbox projection and state mutation model | Show which source objects/events should drive each Inbox action |
| P1 | Identify implementation sequence after Mira acceptance | Note frontend/backend dependencies and any type migrations |

## 7. Done definition

All items below must be true:

- Nimbus publishes an alignment memo under the required artifact path.
- Memo covers Handoff status, review-event mapping, LaunchPack fallback, Inbox projection, and implementation sequence.
- Memo cites exact file targets for expected type/event updates.
- Memo clearly separates fact, recommendation, and blocker.
- Memo does not rely on rejected UI behavior as contract truth.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-sg01-schema-event-alignment-delivery-v2.md`

Required sections:

1. Micro-brief
2. Facts
3. Recommendations
4. Exact file targets
5. Commands run + results
6. Blockers / risks / next owner

## 9. Reporting rules

- Durable artifact: English only.
- Terminal progress summary: Chinese only.
- Do not expand into full implementation unless Lyra issues a new packet after Mira acceptance.
- If blocked, report exact missing evidence and recommended owner.
