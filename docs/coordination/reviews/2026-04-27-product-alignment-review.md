# Product Alignment Review - 2026-04-27

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active |
| Scope | Product contract alignment for SeatLoom baseline work |
| Evidence | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md`, `docs/archive/product-history/mvp-scenarios.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/MEMORY.md` |

## 1. Active product contract

### Decision made

The active working product contract for this cycle is:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`

These are draft documents, but they are the current execution contract for Mira, Nimbus, and Flux until replaced by a newer accepted baseline.

### Higher-authority constraints

- `docs/archive/product-history/mvp-scenarios.md` is `Approved` and remains the user-behavior guardrail.
- `docs/architecture-decisions.md` is `Approved` and remains the architecture guardrail.
- `docs/architecture-design.md` is `Draft` implementation reference only; it does not override product truth.

## 2. Contract status map

| Document | Doc status | Lyra usage | Current stance |
|---|---|---|---|
| `docs/archive/product-history/prd-v0.4.md` | Draft | Primary product contract | Active draft |
| `docs/archive/product-history/interaction-spec-v1.0.md` | Draft | Primary interaction contract | Active draft |
| `docs/archive/product-history/acceptance-spec-v1.0.md` | Draft | Primary acceptance contract | Active draft |
| `docs/archive/product-history/mvp-scenarios.md` | Approved | User-flow constraint and tie-breaker | Approved |
| `docs/architecture-decisions.md` | Approved | Architecture constraint and tie-breaker | Approved |
| `docs/architecture-design.md` | Draft | Implementation reference | Draft reference only |
| `docs/coordination/MEMORY.md` | Active log | Governance memory and milestone state | Active |

## 3. Team precedence rule

### Decision made

Use this order whenever two materials disagree:

1. Approved product and architecture docs under `docs/`
2. Latest accepted coordination artifacts under `docs/coordination/`
3. Draft docs and draft artifacts
4. Terminal or chat text, which is never source-of-truth by itself

## 4. Remaining conflicts and gaps

| Priority | Topic | Conflict | Source refs | Lyra direction |
|---|---|---|---|---|
| P0 | LaunchPack fallback | PRD removes clipboard full-pack fallback, but scenarios still define L2/L3 clipboard paths | `docs/archive/product-history/prd-v0.4.md:223`, `docs/archive/product-history/mvp-scenarios.md:468` | Freeze one canonical fallback policy before Nimbus implementation |
| P0 | Handoff status enum | PRD contract is `drafted -> sent -> accepted -> completed`; architecture adds `Received` | `docs/archive/product-history/prd-v0.4.md:140`, `docs/architecture-design.md:366` | Choose one canonical enum and update UI/types/docs together |
| P0 | Review event semantics | Inbox rules and scenarios use `review.requested`, but architecture event enum has no explicit review event | `docs/archive/product-history/prd-v0.4.md:163`, `docs/archive/product-history/mvp-scenarios.md:40`, `docs/architecture-design.md:427` | Add or explicitly map a review event before ledger/view wiring |
| P0 | Terminal and focus contract | Scenarios define Terminal as a bottom toggle panel; interaction spec includes Terminal in focus loop; current UI moved Terminal into main tabs | `docs/archive/product-history/mvp-scenarios.md:53`, `docs/archive/product-history/mvp-scenarios.md:73`, `docs/archive/product-history/interaction-spec-v1.0.md:22` | Freeze the shell model around a one-screen work surface with bottom terminal panel |
| P1 | Header dependency drift | Scenarios and architecture design headers still point to PRD v0.3 even though v0.4 is the active contract | `docs/archive/product-history/mvp-scenarios.md:8`, `docs/architecture-design.md:9` | Clean document headers after P0 conflicts are resolved |

## 5. Prioritized decision list

| Priority | Decision needed | Owner | Deadline | Exit condition |
|---|---|---|---|---|
| P0 | Canonical LaunchPack fallback policy | Lyra + Aegis | 2026-04-28 | One path documented across PRD, scenarios, UI copy, and implementation notes |
| P0 | Canonical Handoff status model | Lyra + Nimbus | 2026-04-28 | PRD, architecture, TS types, and mock data all match |
| P0 | Canonical review-request event mapping | Lyra + Nimbus | 2026-04-28 | Inbox and timeline can be driven from one agreed event model |
| P0 | Canonical shell and keyboard contract | Lyra + Mira | 2026-04-28 | Terminal placement and shortcut behavior are frozen for SG-01 |
| P1 | Document header dependency cleanup | Lyra | 2026-04-29 | Headers point to active contract set |
| P2 | Copy polish and naming cleanup | Mira | 2026-04-30 | Off-contract labels removed without changing behavior |

## 6. Blockers

- `SG-01 UI Contract Baseline` cannot move to Go while the four P0 conflicts above remain unresolved or only implied in chat.
- Nimbus should not start contract-bound implementation from the current Mira redesign because the UI contract is not yet acceptance-safe.

## 7. Stage-gate status

- Current gate stance: `SG-01 UI Contract Baseline = HOLD`
- Release condition to exit Hold: P0 conflicts resolved in files and Mira redesign accepted against the active contract
