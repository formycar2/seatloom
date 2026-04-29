# SeatLoom Product Truth Index

| Field | Value |
|---|---|
| Product | SeatLoom |
| Document | Product Truth Index |
| Status | Active canonical entrypoint during v0.5 consolidation |
| Updated | 2026-04-28 |
| Owner | Lyra |
| Purpose | Give the team one entrypoint for active product truth, while older design documents are migrated or archived |

## 0. Why this document exists

SeatLoom currently has useful design information spread across multiple PRD generations, scenarios, UX specs, interaction specs, acceptance notes, architecture docs, and role instructions.

That document sprawl creates four risks:

1. different seats read different truths,
2. the same concept is re-read multiple times in slightly different language,
3. implementation starts from stale references,
4. valuable earlier decisions are lost because no one knows whether they were superseded or merely not rewritten yet.

This document creates a single operational answer to that problem:

- one canonical entrypoint,
- one active contract set,
- one precedence rule,
- one migration path for older documents.

## 1. Canonical operating rule

From this point forward, product work should use **one entrypoint** and **one active contract set**.

### 1.1 Single entrypoint

The team starts from this file:

- `docs/PRODUCT_TRUTH.md`

No seat should begin product work by guessing which PRD generation or scenario file is still valid.

### 1.2 Active contract set

The current active product-design contract set is:

1. `docs/prd-v0.5.md`
2. `docs/interaction-spec-v1.1.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`

Reading aid:

- `docs/prd-v0.5.zhs.md` is a non-authoritative Simplified Chinese translation of the active PRD for reading convenience only. Canonical product meaning remains in `docs/prd-v0.5.md`.

Important clarification:

- `docs/prd-v0.5.md` is the active product-thesis and scope contract.
- `docs/interaction-spec-v1.1.md` is the active behavior and flow contract.
- `docs/ux-spec-v1.1.md` is the active screen and field contract.
- `docs/acceptance-spec-v1.1.md` is the active verification artifact aligned to the v0.5 contract set.
- `docs/architecture-decisions.md` and `docs/architecture-design.md` remain active only as implementation constraints and system-shape references, not as competing product-definition sources.
- superseded product-contract generations now live under `docs/archive/product-history/` and remain traceable without acting as parallel authority.

## 2. Precedence rule

Product truth is not resolved by timestamp alone. It is resolved by **topic ownership**.

| Topic | Canonical source |
|---|---|
| Product value, scope, priorities, user promise, P0/P1/P2 boundary | `docs/prd-v0.5.md` |
| User trigger, flow, success/failure behavior, state-change interaction | `docs/interaction-spec-v1.1.md` |
| Screen structure, visible fields, surface layout, interaction affordance | `docs/ux-spec-v1.1.md` |
| Pass/fail criteria for delivery and demo readiness | `docs/acceptance-spec-v1.1.md` |
| System constraints, schema implications, runtime behavior constraints | `docs/architecture-decisions.md`, `docs/architecture-design.md` |

### Conflict resolution rule

If documents disagree, resolve them in this order:

1. If the disagreement is about **why the feature exists or whether it is in scope**, PRD wins.
2. If the disagreement is about **how the user initiates and completes the flow**, Interaction Spec wins.
3. If the disagreement is about **what must be visible on screen**, UX Spec wins.
4. If the disagreement is about **whether the implementation is acceptable**, Acceptance Spec wins for pass/fail only, but must not redefine product meaning.
5. Architecture documents may constrain implementation, but must not silently redefine user-facing product behavior.

## 3. Document status map

### 3.1 Active now

| Document | Status | Role |
|---|---|---|
| `docs/PRODUCT_TRUTH.md` | Active entrypoint | Navigation and precedence |
| `docs/prd-v0.5.md` | Active | Product contract |
| `docs/interaction-spec-v1.1.md` | Active | Interaction contract |
| `docs/ux-spec-v1.1.md` | Active | UX contract |
| `docs/acceptance-spec-v1.1.md` | Active | Acceptance contract |
| `docs/architecture-decisions.md` | Active support | Product-aligned architecture constraints |
| `docs/architecture-design.md` | Active support | Engineering design reference |

### 3.2 Archived legacy reference set

These documents may still contain useful details, but they are **not allowed to act as parallel source of truth**.

| Document | Current use |
|---|---|
| `docs/archive/product-history/prd-v0.4.md` | Carry-forward source for rules not yet rewritten into the active contract set |
| `docs/archive/product-history/prd-v0.3.md` | Historical product-definition source retained for audit |
| `docs/archive/product-history/mvp-scenarios.md` | Reference scenarios and older flow detail; must not override active contract |
| `docs/archive/product-history/product-positioning.md` | Historical positioning input |
| `docs/archive/product-history/mvp-validation.md` | Historical validation input |
| `docs/archive/product-history/ux-spec.md` | Obsolete UX baseline; keep only for carry-forward audit |
| `docs/archive/product-history/interaction-spec-v1.0.md` | Obsolete interaction baseline; keep only for carry-forward audit |
| `docs/archive/product-history/acceptance-spec-v1.0.md` | Superseded acceptance baseline kept for historical review context |

### 3.3 Already archived

| Document | Status |
|---|---|
| `docs/archive/product-history/prd-v0.md` | Archived |
| `docs/archive/product-history/prd-v0.2.md` | Archived |

## 4. What must happen before archived legacy docs can become low-touch history only

The docs have now been moved into a single archive location, but some of them still contain value-bearing rules that are not fully rewritten into the active contract set.

Track the remaining carry-forward work in `docs/coordination/reviews/2026-04-28-carry-forward-matrix.md`.

| Gap | Why it matters | Current source |
|---|---|---|
| Some lifecycle/state-machine semantics remain more explicit in v0.4 than in v0.5 contract set | Old rules may be lost or reinterpreted if archived prematurely | `docs/archive/product-history/prd-v0.4.md` |
| Some multi-project and switch-protection rules remain clearer in older docs | The team may under-specify project isolation and switching behavior | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/mvp-scenarios.md` |
| Some archived docs still contain user-visible rules not yet restated in active specs | Archive alone does not remove the risk of silent rule loss | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/mvp-scenarios.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/ux-spec.md` |

## 5. Working rules during the consolidation period

Until the cleanup is complete, the team must follow these rules:

1. No new task packet may cite `docs/archive/product-history/prd-v0.3.md`, `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/mvp-scenarios.md`, `docs/archive/product-history/ux-spec.md`, or `docs/archive/product-history/interaction-spec-v1.0.md` as primary authority.
2. If a needed rule exists only in an older document, Lyra must either:
   - carry it into the active contract set, or
   - explicitly mark it rejected.
3. Older documents may be read for recovery, but they may not override the active contract set.
4. New reviews, tasks, and acceptance artifacts should cite `docs/PRODUCT_TRUTH.md` first, then only the active documents they actually need.
5. Architecture support documents may shape implementation choices, but any user-visible product rule must live in the active contract set.

## 6. Recommended reading path by seat

### Lyra

Read in this order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/architecture-decisions.md`
7. `docs/architecture-design.md`

### Mira

Read in this order:

1. `docs/PRODUCT_TRUTH.md`
2. relevant PRD sections in `docs/prd-v0.5.md`
3. relevant flows in `docs/interaction-spec-v1.1.md`
4. relevant surfaces in `docs/ux-spec-v1.1.md`
5. only the acceptance slices needed for current UI scope

### Nimbus

Read in this order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`

### Flux

Read in this order:

1. `docs/PRODUCT_TRUTH.md`
2. current task packet
3. only the active contract slices directly needed for the assigned work

## 7. Consolidation plan

### Phase A - Freeze the active contract set

Done when:

- the team explicitly agrees that product work starts from `docs/PRODUCT_TRUTH.md`,
- `docs/prd-v0.5.md` is treated as the active PRD,
- old PRDs stop being cited as primary authority in new tasks.

### Phase B - Carry forward missing value-bearing rules

Done when:

- retained rules from `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/mvp-scenarios.md`, `docs/archive/product-history/ux-spec.md`, and `docs/archive/product-history/interaction-spec-v1.0.md` are either migrated or rejected,
- no important product behavior remains “known only from an older document.”

### Phase C - Align support contracts

Done when:

- `docs/acceptance-spec-v1.1.md` is treated as the active acceptance contract,
- architecture and role docs reference the active contract set instead of older PRDs,
- UI and engineering seats no longer need to open older PRDs to do routine work.

### Phase D - Archive and label legacy docs

Done when:

- legacy product docs live under `docs/archive/product-history/`,
- active docs no longer point to archived files as primary authority,
- archived files are treated as carry-forward input only.

## 8. End-state definition

The cleanup is complete only when all of the following are true:

- there is one obvious entrypoint for product truth,
- there is one active contract set,
- older product docs are either archived or explicitly reference-only,
- no seat has to re-read multiple generations of the same product idea to do current work,
- valuable earlier rules were either carried forward or intentionally rejected,
- implementation no longer starts from ambiguity.
