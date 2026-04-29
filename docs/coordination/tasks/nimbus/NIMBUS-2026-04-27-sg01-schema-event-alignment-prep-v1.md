# NIMBUS-2026-04-27-sg01-schema-event-alignment-prep-v1

| Field | Value |
|---|---|
| Owner | Nimbus |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-30 12:00 |
| Acceptance owner | Lyra |
| Current handoff status | NO-GO for full implementation |

## 1. Objective

Prepare the engineering alignment memo needed for SG-01 recovery without starting full implementation from the rejected UI baseline.

## 2. Authority and evidence

Required sources:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`
4. `docs/archive/product-history/mvp-scenarios.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`
8. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`

## 3. Scope restriction

Allowed now:

- schema and event alignment memo
- implementation sequencing notes
- explicit blocker and risk list

Not allowed now:

- treating current Mira prototype as baseline truth for shell, Inbox, Timeline, Handoff, or recovery flows
- full implementation handoff or product-semantic changes without Lyra approval

## 4. Required work

| Priority | Requirement | Expected coverage |
|---|---|---|
| P0 | Freeze recommended canonical Handoff status enum | Map current docs/types drift and recommend one enum across PRD, architecture, TS types, and mock data |
| P0 | Freeze recommended `review.requested` event mapping | Show how Inbox and Timeline can project from one event model |
| P0 | Clarify LaunchPack fallback handling | Recommend one canonical fallback policy consistent with active product contract |
| P0 | Define Inbox projection and state mutation model | Show which source objects/events should drive each Inbox action |
| P1 | Identify implementation sequence after Mira acceptance | Note frontend/backend dependencies and any type migrations |

## 5. Done definition

All items below must be true:

- Nimbus publishes an alignment memo under `docs/coordination/tasks/nimbus/`.
- Memo covers Handoff status enum, review event mapping, LaunchPack fallback, Inbox projection model, and implementation sequence.
- Memo cites exact file targets for expected type or event updates.
- Memo clearly separates fact, recommendation, and blocker.
- Memo does not rely on rejected UI behavior as contract truth.

## 6. Delivery artifact required

Publish completion as:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-sg01-schema-event-alignment-delivery-v1.md`

Include:

- facts vs recommendations
- exact file targets
- blocker list
- commands run, if any, and results
