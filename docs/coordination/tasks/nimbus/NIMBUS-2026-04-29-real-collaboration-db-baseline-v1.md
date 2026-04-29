# Task: Nimbus Real Collaboration PostgreSQL Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-real-collaboration-db-baseline-v1 |
| status | revised |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/reviews/2026-04-28-process-mapping-review.md`, `docs/coordination/MEMORY.md`, `docs/coordination/memory/2026-04-29.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-read-model-repositories-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-seat-registry-delegation-storage-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, infrastructure, postgres, persistence, seed-data, seatloom, artifacts, ledger, repositories |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet only. You are not alone in the codebase; do not revert others' work, do not widen into UI work, route-engine business behavior, prompt engine, or PTY/runtime implementation. |

## Objective

Build the first **real database-persisted collaboration baseline** for SeatLoom using **PostgreSQL** as the authoritative store for structured project truth.

The sponsor correction is explicit:

- structured collaboration truth must persist in a **real database**;
- file-only persistence is not sufficient as the authority layer;
- SQLite is not the chosen method for this baseline;
- the later frontend should be able to read truthful project objects from a durable PostgreSQL-backed store.

## Authority Update

This packet supersedes two earlier assumptions for this lane:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-truth-baseline-v1.md` is superseded for storage authority because it remained file-first.
- Any SQLite-first assumption for this collaboration-truth baseline is superseded by sponsor direction.

Important boundary:

- markdown documents and evidence payloads may remain on disk;
- but **structured collaboration truth** in this packet must persist into PostgreSQL as the authoritative local store.

## Method Assumption For This Packet

Use **repo-managed local PostgreSQL** for deterministic development and verification.

Default method for this packet:

- add a repo-local `docker compose` PostgreSQL service for development/bootstrap;
- keep configuration deterministic and minimal;
- do not widen into cloud hosting or multi-node deployment;
- do not attempt embedded Postgres runtime management inside Tauri in this packet.

Why this method now:

- it gives us a real DB with explicit operational truth;
- it is easier to verify and debug than an embedded-Postgres experiment at this stage;
- it stays in infrastructure scope and avoids UI/runtime coupling;
- it creates a stable truth source that later product surfaces can read from.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/DOCUMENT_TEMPLATES.md`
8. `docs/coordination/reviews/2026-04-28-process-mapping-review.md`
9. `docs/coordination/MEMORY.md`
10. `docs/coordination/memory/2026-04-29.md`
11. this packet

## Need-to-Know Scope

Implement only the **PostgreSQL-persisted truth baseline** needed for later UI consumption.

In scope:

- add a deterministic local PostgreSQL foundation for this repo;
- define schema for the current core object families;
- seed PostgreSQL with **real collaboration data** from our actual SeatLoom build process;
- keep artifact payload files on disk where appropriate, while storing artifact metadata and relationships in PostgreSQL;
- add bounded read repositories over PostgreSQL;
- add one deterministic bootstrap / verification path that proves the DB can be started, migrated, seeded, and read from this repo.

Out of scope:

- no HTTP API;
- no cloud database;
- no backend service layer;
- no route-engine / inbox projection work;
- no prompt-engine / PTY / runtime work;
- no UI / frontend work;
- no generalized sync engine from arbitrary markdown into DB;
- no semantic retrieval / embeddings / pgvector in this packet unless strictly required for schema compatibility;
- no business workflow expansion.

## Storage Authority

For this packet, storage authority is:

1. **PostgreSQL** for structured collaboration truth
2. **filesystem artifact payloads** only for document bodies / evidence payloads

That means:

- seats, role bindings, delegations, sessions, workitems, handoffs, artifact metadata, and canonical events must be queryable from PostgreSQL;
- markdown documents remain real files, but their typed metadata and relations belong in PostgreSQL;
- the later frontend should be able to read structured truth from Postgres-backed repositories rather than reconstructing object state by scanning the docs tree.

## Suggested Local Layout

Use deterministic repo-local assets such as:

- `infra/postgres/docker-compose.yml`
- `infra/postgres/init/`
- `infra/postgres/schema/`
- `infra/postgres/seed/`
- `.env.example` entries only if needed for local connection configuration
- `.seatloom/bootstrap/source-map.md` or `.seatloom/bootstrap/source-map.yaml`

You may choose a tighter path layout if it is cleaner, but keep it obvious and repo-local.

## Real Data Rules

Hard rules for this packet:

1. **No fake demo content.** No lorem ipsum, invented users, or fabricated documents.
2. **Use real repo evidence.** If an object or artifact is seeded, it must map to a real repo file, real collaboration event, or explicit evidence note.
3. **Mark approximations.** If a timestamp or exact transition cannot be proven precisely, record the approximation source in provenance.
4. **Use valid `template + subtype`.** Artifact taxonomy must align with `docs/coordination/DOCUMENT_TEMPLATES.md`.
5. **Keep the DB truthful, not exhaustively speculative.** Curated real baseline is acceptable; fiction is not.

## Write Boundary

Primary code targets:

- workspace Cargo manifests only as needed for PostgreSQL client / migration dependencies
- `crates/seatloom-core/src/` under a bounded new persistence module, for example:
  - `db/connection.rs`
  - `db/schema.rs`
  - `db/bootstrap.rs`
  - `db/repositories/*.rs`
  - `db/models/*.rs`
- a bounded migration/bootstrap path if needed
- optional read-only Tauri command wiring only if it remains tightly bounded and infrastructure-only

Primary infra/data targets:

- `infra/postgres/docker-compose.yml`
- `infra/postgres/schema/*`
- `infra/postgres/seed/*`
- `.seatloom/bootstrap/source-map.*`

Do not widen into unrelated modules.

## Required Outcome

### 1. Establish the persisted DB baseline

Create a real PostgreSQL-backed truth store for the current SeatLoom project and seed it with real collaboration truth.

At minimum, persist these families:

- project
- seat identities
- project role bindings
- seat delegations
- sessions
- workitems
- handoffs
- artifact metadata
- canonical events / timeline rows

The seeded baseline must express the real collaboration we have already done on SeatLoom.

### 2. Seed real collaboration data

The database must contain a curated but truthful baseline that covers:

- `aegis`, `lyra`, `mira`, `nimbus`, `flux`
- at least one real delegation for Flux acting on behalf of Mira
- real sessions across Apr 27-29
- real workitems and handoffs already evidenced in the repo
- artifacts covering the active contract set and coordination history
- a chronological event trail that can later drive Timeline / Activity surfaces

### 3. Finish artifact metadata persistence

Artifacts must no longer be only “string paths in the repo”.

Implement bounded Postgres-backed artifact persistence for:

- artifact id
- title
- template
- subtype
- source path / payload ref
- source session
- source workitem
- created_at

Add deterministic validation for `template + subtype` against the allow-list.

### 4. Add DB-backed read repositories

Provide bounded repository surfaces that can read from PostgreSQL for the seeded truth baseline.

At minimum, add list/load support for:

- seats
- delegations
- sessions
- workitems
- handoffs
- artifacts
- timeline events

You may keep older file-scanning repositories in place if needed for compatibility, but the new seeded baseline for this packet must be readable from PostgreSQL.

### 5. Keep evidence bodies on disk where appropriate

Do not move the actual markdown docs into opaque blobs unless necessary.

Instead:

- keep real docs in the repo as artifact payloads/evidence;
- persist their typed metadata, relationships, and retrieval keys in PostgreSQL;
- make PostgreSQL the structured truth layer and the repo files the evidence layer.

### 6. Add one deterministic bootstrap / verification path

Provide one bounded path that proves:

1. PostgreSQL can be started deterministically for local development,
2. schema can be applied,
3. the real baseline can be seeded,
4. the seeded repositories return non-empty truthful data.

This can be done through focused Rust tests, a bounded script, or both.

## Minimum Artifact Family Coverage

The seeded DB must include real artifact records spanning:

- one `T1` authority doc
- one `T2` role profile
- one `T3` task packet
- one `T4` review
- one `T5` acceptance
- one `T6` daily memory entry
- one `T7` governance doc

And it must include the active contract set:

- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`
- `docs/PRODUCT_TRUTH.md`

## Acceptance Criteria

1. A real PostgreSQL-backed store now exists for local development and contains structured collaboration truth.
2. The seeded DB expresses the current SeatLoom collaboration through connected seat / session / workitem / handoff / artifact / event records.
3. At least one DB-persisted delegation truthfully captures the `Flux acting for Mira` case.
4. Artifact metadata persists in DB with valid `template + subtype` or a clear non-template path.
5. Structured truth is queryable from Postgres-backed repositories, not only reconstructed from loose files.
6. The seeded baseline includes at least one real artifact record for each `T1` through `T7` family and includes the active contract set.
7. A deterministic provenance artifact exists and explains evidence sources plus approximations.
8. A deterministic bootstrap / verification path exists and passes on a Rust-capable seat with PostgreSQL available through the repo-managed method.
9. `$HOME/.cargo/bin/cargo check` passes.
10. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
11. Scope stays infrastructure/data-foundation only; no UI or route/prompt/runtime widening enters this packet.

## Required Validation

Run and record the exact commands you use. At minimum:

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
```

If you add a bounded bootstrap script, migration runner, or compose-based verification command, run those too and record the result.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. PostgreSQL schema summary
4. Seeded object coverage by family
5. Artifact taxonomy coverage (`template + subtype`)
6. Provenance sources and approximation notes
7. Postgres-backed repository behavior
8. Validation commands and results
9. Residual notes / explicit non-goals kept out
10. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Repo-managed PostgreSQL foundation exists and is seeded with real project truth.
- [ ] Postgres-backed repositories can read the seeded truth baseline.
- [ ] Artifact metadata is persisted in DB.
- [ ] Provenance is written.
- [ ] Deterministic validation passes on a Rust-capable seat.
- [ ] `$HOME/.cargo/bin/cargo check` passes.
- [ ] `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_real_collaboration_db_baseline.txt
[Nimbus -> Lyra] Real Collaboration PostgreSQL Baseline
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `...` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_real_collaboration_db_baseline /tmp/nimbus_to_lyra_real_collaboration_db_baseline.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_real_collaboration_db_baseline
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
