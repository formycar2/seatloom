# Task: Nimbus PostgreSQL Write/Ingest/Reconcile Contract

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-postgres-write-ingest-and-reconcile-contract-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/COLLABORATION_PROTOCOL.md`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/seed/001_real_collaboration_baseline.sql`, `infra/postgres/seed/002_document_seed.sql`, `scripts/ingest-documents.sh`, `src-cli/src/main.rs` |
| tags | nimbus, infrastructure, postgres, ingest, reconcile, authority, documents, sync |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet. You are not alone in the codebase; do not revert others' work, do not widen into UI or business logic, and keep the scope on write-path authority, deterministic ingest, reconcile bookkeeping, and PostgreSQL persistence only. |

## Objective

Define and implement the steady-state infrastructure contract for how newly authored coordination data enters PostgreSQL after SeatLoom is in real use.

This packet exists because we now have a solid PostgreSQL baseline and first-class typed documents, but we still do not have a durable answer for the ongoing write path: when people keep authoring markdown in the repo, how that new truth is detected, ingested, versioned, deduplicated, conflict-checked, and made visible to future projections.

This is infrastructure-only. Do not implement UI, route automation, mobile behavior, review comments, or product/business flows.

## Why This Packet Exists

We have already moved from file-only mock state toward PostgreSQL authority, but the current state is still incomplete for real use:

1. `scripts/ingest-documents.sh` is a useful seed-upsert helper, not yet a steady-state reconcile contract.
2. `documents` now exist, but there is not yet a durable version/journal model for repeat ingest over time.
3. The product contracts already require reconcile freshness, cached-state fallback, and one canonical object history across desktop/mobile/project views.
4. `AD-007` already says MVP reconcile is triggered on app startup, pre-pipeline, and manual reconcile -- with no file watcher. That direction now needs a real PostgreSQL write-path implementation and bookkeeping layer.

The goal is to make our current writing mode compatible with SeatLoom's future runtime:

- people and seats can keep producing markdown artifacts in the repo,
- SeatLoom can deterministically ingest them into PostgreSQL,
- unchanged content does not fork duplicate truth,
- changed content creates durable revisions,
- conflicts and parse failures are visible instead of silent.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/DOCUMENT_TEMPLATES.md`
6. `docs/coordination/COLLABORATION_PROTOCOL.md`
7. `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`
8. `docs/architecture-design.md`
9. `docs/architecture-decisions.md`
10. `infra/postgres/schema/001_seatloom_core.sql`
11. `infra/postgres/schema/002_document_authority.sql`
12. `infra/postgres/seed/001_real_collaboration_baseline.sql`
13. `infra/postgres/seed/002_document_seed.sql`
14. `scripts/ingest-documents.sh`
15. `src-cli/src/main.rs`
16. this packet

## Need-to-Know Scope

In scope:

- realign architecture docs so the steady-state write path is explicitly defined,
- freeze MVP ingest/reconcile trigger policy around `startup`, `pre_pipeline`, and `manual`,
- keep MVP free of file watchers and background daemons,
- add durable PostgreSQL bookkeeping for repeated document ingest runs,
- add document revision/history persistence for repeated markdown ingestion,
- add deterministic conflict/parse-failure visibility for the document channel,
- implement a deterministic reconcile/ingest utility for repo-authored markdown into PostgreSQL,
- expose the reconcile operation through the existing CLI stub if that is the cleanest infrastructure path,
- keep validation commit-pinnable and Flux-verifiable.

Out of scope:

- UI or Tauri command behavior widening,
- comment threads, review objects, or `change_tier_record`,
- prompt-state persistence,
- checkpoints / continuity packs,
- Inbox / Timeline / dashboard projections,
- mobile pending-send queue behavior,
- background daemon or file watcher,
- semantic retrieval,
- API endpoints,
- product/business automation.

## Branch and Commit Discipline

Stay on the infrastructure branch already in use for this lane:

- `track/infra-foundation`

When done:

1. commit only the bounded infrastructure changes from this packet,
2. push the branch,
3. report the exact commit hash in the delivery artifact and tmux reply,
4. keep the packet Flux-verifiable by exact commit.

## Write Boundary

Primary targets:

- `docs/architecture-design.md`
- `docs/architecture-decisions.md`
- `infra/postgres/schema/002_document_authority.sql`
- `infra/postgres/schema/003_write_ingest_reconcile.sql`
- `infra/postgres/seed/002_document_seed.sql`
- `scripts/ingest-documents.sh`
- `scripts/verify-postgres-baseline.sh`
- `src-cli/src/main.rs`
- `crates/seatloom-core/src/db/mod.rs`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`

Allowed new files if they keep the scope cleaner:

- `crates/seatloom-core/src/db/reconcile.rs`
- `crates/seatloom-core/src/db/ingest_runner.rs`
- `scripts/verify-postgres-reconcile.sh`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-write-ingest-and-reconcile-contract-delivery-v1.md`

Disallowed in this packet:

- files under `ui/`
- widening `src-tauri/src/commands/*`
- route-engine or review-engine behavior
- runtime/PTY behavior work
- mobile behavior
- comments / review automation / Inbox projection

## Required Outcome

### 1. Freeze the steady-state authority contract

Update the architecture docs so the steady-state document write path is explicit and non-contradictory.

Minimum required position:

- PostgreSQL is the canonical structured authority.
- Repo markdown is an external-authoring input surface and optional export surface.
- Reconcile is the bounded import/update operation that moves markdown truth into PostgreSQL.
- MVP does **not** use file watchers or background daemon sync.
- MVP reconcile triggers are exactly the three already accepted in `AD-007`:
  - app startup,
  - pre-pipeline,
  - manual reconcile.
- Any stale-cache / cached-state fallback language must clearly describe transition behavior, not peer authority.

### 2. Add durable write/ingest/reconcile bookkeeping in PostgreSQL

Extend the schema so repeated ingest over time is durable and inspectable.

At minimum, introduce a bounded write-path authority family that supports:

- reconcile run identity,
- trigger type (`startup`, `pre_pipeline`, `manual`),
- run status,
- scanned/inserted/updated/unchanged/failed/conflict counts,
- start/completion timestamps,
- per-file/per-document reconcile result rows,
- document revision snapshots or an equivalent durable version history,
- enough structured fields to tell whether a document changed, was unchanged, failed parse, or hit a conflict.

It is acceptable to implement this as one or more tables, but the result must let us answer these questions deterministically:

1. what reconcile run last touched this document,
2. what changed in that run,
3. whether a file was skipped / unchanged / updated / conflicted,
4. what the current revision is,
5. what prior document body/header snapshot existed before the latest import.

### 3. Support repeat ingest without duplicate truth

The current writing mode is iterative. A document may be imported multiple times as it evolves.

Implement deterministic repeat-ingest behavior so:

- unchanged file content does not create duplicate document rows,
- unchanged file content does not create duplicate revision snapshots,
- changed file content increments revision and records history,
- parse failure does not silently drop visibility,
- a conflicting `doc_id` / `file_path` situation is recorded explicitly.

You may use `body_digest`, header digest, `revision`, and/or structured conflict markers as needed.

### 4. Implement a deterministic reconcile utility

Implement the MVP reconcile utility for the document channel.

Preferred behavior:

- deterministic scan of allowed markdown docs in the repository,
- parse universal header + sections,
- upsert current document truth into PostgreSQL,
- write reconcile run + item rows,
- write section projections,
- write document version history only when content actually changes,
- exit with clear success/failure status for verification.

If the cleanest path is to wire the existing `seatloom reconcile` CLI stub to this infrastructure-only operation, do that. If a narrower helper is cleaner, document why.

### 5. Keep the ingest surface bounded and explicit

Do not attempt a magical whole-repo crawler.

Bound the ingest surface to the coordination / contract markdown families already in use for this project, such as:

- `docs/*.md`
- `docs/coordination/**/*.md`

Only ingest documents that either:

- parse into a recognized `template+subtype` pair, or
- are explicitly allow-listed transitional artifacts with a documented reason.

Do not create a loose system that ingests arbitrary markdown without classification policy.

### 6. Add focused tests

Add focused tests for:

- repeat ingest of unchanged doc -> no duplicate revision,
- changed doc -> revision increment + version snapshot,
- conflicting `doc_id` or path relationship -> reconcile item marked conflict,
- parse failure -> reconcile item marked failed/partial and not silently lost,
- CLI or script reconcile path returns expected exit code and writes reconcile bookkeeping,
- repository reads for reconcile runs / reconcile items / document versions.

DB integration tests may stay `#[ignore]` if they need PostgreSQL, but they must be real and ready for Flux on a postgres-capable seat.

## Acceptance Criteria

1. Architecture docs explicitly define the steady-state PG authority + bounded reconcile write path.
2. MVP reconcile trigger policy is frozen to startup / pre-pipeline / manual, with no file watcher.
3. PostgreSQL now records reconcile runs and reconcile item outcomes.
4. PostgreSQL now records document revision history or equivalent durable change snapshots.
5. Repeat ingest of unchanged content does not fork duplicate truth.
6. Changed content increments revision and preserves prior snapshot history.
7. Conflict and parse-failure states are durably visible.
8. A deterministic reconcile utility exists and is documented/validated.
9. No UI, no Tauri command widening, and no product/business automation is added.
10. `$HOME/.cargo/bin/cargo check` passes.
11. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
12. The PostgreSQL verification path is documented and Flux-ready by exact commit.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
scripts/verify-postgres-baseline.sh
```

If a new helper script is introduced for reconcile verification, include that exact command too.

If local postgres/docker is unavailable on your seat, state that explicitly, but keep the verification path runnable for Flux on a postgres-capable seat.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-write-ingest-and-reconcile-contract-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Authority/write-path contract summary
4. Schema additions
5. Reconcile/ingest behavior summary
6. Versioning and conflict policy summary
7. Validation commands and results
8. Residual non-goals kept out
9. Exact branch and commit hash
10. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Architecture docs define the PG write/ingest/reconcile contract clearly.
- [ ] Reconcile bookkeeping exists in PostgreSQL.
- [ ] Document revision/history support exists for repeated ingest.
- [ ] Deterministic reconcile utility exists.
- [ ] Validation commands are run or explicitly environment-blocked with a Flux-ready fallback path.
- [ ] Branch is pushed and exact commit hash is reported.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_postgres_reconcile.txt
[Nimbus -> Lyra] PostgreSQL Write/Ingest/Reconcile Contract
branch:
- track/infra-foundation
commit:
- <exact_commit_hash>
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `scripts/verify-postgres-baseline.sh` => ...
blockers:
- none / ...
next action:
- wait for Flux commit-pinned verification and Lyra acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-write-ingest-and-reconcile-contract-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_postgres_reconcile /tmp/nimbus_to_lyra_postgres_reconcile.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_postgres_reconcile
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
