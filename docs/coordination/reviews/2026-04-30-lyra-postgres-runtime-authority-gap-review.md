# Review: PostgreSQL Runtime Authority Gap

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-30-postgres-runtime-authority-gap-review-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/003_write_ingest_reconcile.sql`, `infra/postgres/schema/004_operational_review_and_continuity.sql`, `scripts/verify-postgres-baseline.sh`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md` |
| tags | lyra, review, postgres, runtime-authority, prompt, mobile, ingest, infrastructure |
| owner | Lyra |

## Verdict

**Accept the current PostgreSQL baseline as a strong P0 infrastructure foundation. Hold for runtime-authority completeness.**

Nimbus has now established a credible database authority layer for:
- core project / seat / session / workitem / handoff / artifact truth,
- typed documents and deterministic section extraction,
- reconcile bookkeeping and document versioning,
- operational review / continuity families,
- deterministic baseline bootstrap with commit-pinned verification discipline.

That foundation is worth keeping.

However, the current PostgreSQL authority still does not fully cover the remaining P0 contract promises around interactive prompt handling, mobile state-changing actions, and the live write path that SeatLoom itself will need once the product becomes the primary operating surface.

## What Is Already Good Enough

### 1. Database authority direction

The repo now has a real PostgreSQL-managed baseline with:
- repo-managed schema and seed,
- typed read repositories,
- document-body persistence,
- deterministic baseline verification,
- commit-pinned remote proof.

This is enough to keep moving on infrastructure without falling back to file-only truth.

### 2. Historical collaboration truth

The current seeded data is honest about provenance and approximation. It gives the frontend and later projections a real object graph instead of pure mocks.

### 3. Review / continuity groundwork

`checkpoints`, `handoff_receipts`, `pipeline_runs`, `review_threads`, and `review_comments` are now first-class PostgreSQL families. This closes several important contract gaps for continuity and document critique.

## Remaining Authority Gaps Against PRD v0.5

### 1. Prompt-blocked state is not yet a first-class PostgreSQL object

Contract pressure:
- `US-P0-11`
- `INT-16`
- `E-10`

Current issue:
- Rust architecture types define `PromptState`, `PromptKind`, `PromptPolicy`, and `PromptAction`.
- Event definitions already reference prompt detection / injection.
- But PostgreSQL has no canonical table family for the active prompt instance, its bounded evidence window, policy classification, or its resolution history.

Why this matters:
- the product contract requires prompt state to be auditable without rereading raw terminal noise,
- the UI cannot safely consume prompt truth from event payload guesses alone,
- supervisor assist budgets and human takeover decisions need durable authority rows, not only transient memory.

### 2. Mobile state-changing actions do not yet have a canonical receipt family

Contract pressure:
- `US-P0-13`
- `US-P0-14`
- `US-P0-15`
- `INT-18`
- `INT-19`
- `INT-20`
- `E-11`

Current issue:
- `review_threads` / `review_comments` provide a good family for bounded feedback,
- but there is still no explicit canonical receipt for state-changing actions such as:
  - mobile approve / reject / escalate,
  - reserve desktop takeover,
  - prompt continue / stop,
  - desktop or supervisor equivalents of the same decision class.

Why this matters:
- mobile must reuse the same canonical transitions as desktop,
- channel metadata such as `source=mobile` cannot stay implicit,
- auditability and replay need a durable action object that links actor, target, policy, evidence, and resulting state/event.

### 3. The live write ingress path is still underspecified

User concern addressed here:
- once SeatLoom becomes the primary workspace, newly created data should be written directly into PostgreSQL as canonical truth,
- repo-authored Markdown remains an important input surface, but it is no longer the only source of new truth.

Current issue:
- schema `003` covers repo reconcile (`markdown -> PostgreSQL`),
- but the runtime write edge is not yet structurally defined for SeatLoom-originated actions.

What is missing:
- idempotency key / mutation key discipline,
- actor + channel metadata at the write boundary,
- expected revision / resulting revision or equivalent optimistic write guard,
- explicit success / rejected / conflict result state for inbound writes,
- linkage from a state-changing action to the canonical event it emitted.

Why this matters:
- without a write-ingress contract, future desktop/mobile/product actions have no stable persistence boundary,
- concurrent collaboration will become guesswork instead of governed mutation flow,
- the product cannot safely move from “reconstructed baseline” to “live system of record.”

### 4. Reconcile and runtime-authority coexistence still needs sharper rules

Current issue:
- documents already have a reconcile path,
- runtime objects increasingly have direct DB authority,
- but the repo does not yet make the source-of-truth split explicit enough at the object-family level.

Required clarification:
- which families are repo-reconciled inputs,
- which families are runtime-authored canonical objects,
- what happens when a runtime action references a document that is later re-reconciled,
- where conflict detection lives.

This is still an infrastructure concern, not a UI concern.

## Required Next Infrastructure Requirements

### Requirement A: Prompt authority family

Add a bounded PostgreSQL family for prompt lifecycle truth.

Minimum capability:
- one first-class prompt instance row per detected blocked prompt,
- session linkage,
- prompt classification (`deterministic`, `wizard/menu`, `freeform`, `sensitive`),
- policy classification (`auto allowed`, `needs approval`, `human required`),
- bounded evidence reference or bounded preview storage,
- allowed actions,
- assist budget state,
- active / resolved / stopped status,
- timestamps,
- resulting event linkage where applicable.

### Requirement B: Prompt action history

Persist the resolution trail for prompt decisions.

Minimum capability:
- approve / human takeover / supervisor assist / stop,
- actor identity,
- channel,
- optional bounded note,
- optional injected input preview or reference,
- step/token budget accounting when supervisor assist is used,
- result status,
- canonical event linkage.

### Requirement C: Generic channel-action receipt family

Add a canonical receipt layer for state-changing desktop/mobile/supervisor actions.

Minimum capability:
- target object kind + id,
- action kind,
- actor,
- source channel,
- note,
- evidence refs,
- policy or gate summary,
- expected revision / resulting revision or equivalent optimistic guard,
- applied / rejected / conflicted status,
- resulting canonical event linkage.

This is the key bridge between future UI actions and durable PostgreSQL truth.

### Requirement D: Read/write-edge repository contracts

Even if no API or Tauri command is added yet, the infrastructure should expose typed repository contracts for:
- creating prompt rows,
- appending prompt actions,
- appending channel-action receipts,
- listing and reading them by session / target / status.

This keeps the mutation boundary explicit and testable before any product surface is wired to it.

### Requirement E: Honest seed policy

Do not fabricate prompt/mobile runtime rows if the current collaboration record does not actually contain them.

Allowed:
- zero-row baseline with explicit source-map note,
- test fixtures created only inside Rust integration tests,
- seed rows only when directly backed by real evidence.

### Requirement F: Verification continuity

The next schema packet must remain compatible with commit-pinned remote verification:
- schema application is deterministic,
- tests are real,
- `scripts/verify-postgres-baseline.sh` remains the accepted bootstrap gate.

## Recommended Packet Order

1. **Flux** verifies commit `24c820b81de8f8a479e9d229f5c106ed440eb675` on the remote Docker-capable workspace by running the hardened PostgreSQL baseline script twice in a row.
2. **Nimbus** delivers the next infra-only packet for prompt authority plus generic channel-action receipts.
3. **Nimbus** then continues into a later packet only if needed for deeper runtime ingress / conflict-handling details that do not fit cleanly in the bounded packet above.

## Bottom Line

The current PostgreSQL baseline is no longer the blocker.

The next blocker is **runtime-authority completeness**:
- prompt truth,
- state-changing channel actions,
- and a governed direct-write boundary for the live SeatLoom product.

Those are still infrastructure tasks, and they should be completed before we treat the database layer as product-complete truth.
