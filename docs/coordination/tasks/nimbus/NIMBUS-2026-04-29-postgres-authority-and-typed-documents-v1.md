# Task: Nimbus PostgreSQL Authority Alignment + Typed Document Persistence

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-postgres-authority-and-typed-documents-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/COLLABORATION_PROTOCOL.md`, `docs/coordination/reviews/2026-04-28-process-mapping-review.md`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/seed/001_real_collaboration_baseline.sql` |
| tags | nimbus, infrastructure, postgres, authority, artifacts, documents, retrieval, schema |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet. You are not alone in the codebase; do not revert others' work, do not widen into UI or product/business behavior, and keep the scope on authority schema, ingestion, and read repositories only. |

## Objective

Close the current authority drift and make typed coordination documents first-class PostgreSQL objects so future SeatLoom UI work can read real project truth from the database instead of string paths or file-only metadata.

This packet is infrastructure-only. It must not implement comments, review UI, workflow automation, or frontend behavior. It should, however, create the durable schema and seed path that those future features depend on.

## Why This Packet Exists

Current state is good enough for a first PostgreSQL baseline, but not yet good enough for PRD v0.5:

1. the repo currently has two competing truth directions: PostgreSQL read repositories vs file-backed app reads,
2. accepted architecture docs still describe `.seatloom/` file-first authority and SQLite FTS5 as the P0 retrieval authority,
3. typed coordination artifacts are only partially objectized,
4. the current seed is credible but intentionally partial, and
5. future frontend work needs real typed document data in the repository before API or business logic is added.

This packet addresses only the infrastructure slice that must exist before artifact review, retrieval, and deeper projections can become trustworthy.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/DOCUMENT_TEMPLATES.md`
6. `docs/coordination/COLLABORATION_PROTOCOL.md`
7. `docs/coordination/reviews/2026-04-28-process-mapping-review.md`
8. `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`
9. `docs/architecture-design.md`
10. `docs/architecture-decisions.md`
11. `infra/postgres/schema/001_seatloom_core.sql`
12. `infra/postgres/seed/001_real_collaboration_baseline.sql`
13. this packet

## Need-to-Know Scope

In scope:

- realign authority/storage language in architecture docs to the accepted PostgreSQL direction,
- extend PostgreSQL schema so typed markdown coordination documents are durable first-class objects,
- persist structured universal-header metadata from `DOCUMENT_TEMPLATES.md`,
- persist document body text in PostgreSQL,
- add deterministic section/anchor extraction for future artifact review and retrieval use,
- add bounded document-to-object association tables needed for future WorkItem/Handoff/Session/Project links,
- seed the active contract set and key coordination docs from the real repository into the new tables,
- add typed Rust DB models/repositories/tests for the new schema,
- keep verification commit-pinnable and flux-verifiable.

Out of scope:

- comments, replies, resolve/dispute flows,
- `change_tier_record`,
- prompt-state persistence,
- checkpoint / continuity-pack persistence,
- Inbox / Timeline / dashboard projection logic,
- Tauri command wiring to the new DB layer,
- UI/frontend changes,
- mobile-specific behavior,
- semantic retrieval,
- API or mutation command surfaces.

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
- `infra/postgres/schema/001_seatloom_core.sql`
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `.seatloom/bootstrap/source-map.yaml`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`

Allowed new files if they keep the scope cleaner:

- `crates/seatloom-core/src/db/document_ingest.rs`
- `crates/seatloom-core/src/db/document_parser.rs`
- `crates/seatloom-core/src/db/document_models.rs`
- `scripts/verify-postgres-documents.sh`

Disallowed in this packet:

- UI files under `ui/`
- Tauri command behavior widening under `src-tauri/src/commands/`
- route-engine or runtime-engine implementation
- PTY integration
- comment/review workflow objects
- business logic / automation behavior

## Required Outcome

### 1. Realign authority direction in architecture docs

Update `docs/architecture-design.md` and `docs/architecture-decisions.md` so they no longer conflict with the accepted PostgreSQL direction.

Minimum required alignment:

- PostgreSQL is the canonical structured truth store for SeatLoom project data.
- `.seatloom/` files are documented as import/export/cache/compatibility artifacts, not peer authority.
- P0 retrieval authority is no longer described as SQLite FTS5.
- Retrieval direction is documented as PostgreSQL structured index first, PostgreSQL full-text second, later semantic layer third.
- Any migration caveat or remaining temporary file-backed read paths are called out explicitly as transition state, not final design.

Do not leave mixed authority claims behind.

### 2. Add first-class typed document persistence to PostgreSQL

Extend the schema so typed markdown coordination documents become durable database objects.

At minimum, add a bounded document authority family that captures:

- project id,
- artifact id or equivalent linkage,
- template,
- subtype,
- document id,
- title,
- status,
- author,
- date,
- version,
- depends_on,
- supersedes,
- tags,
- file path,
- markdown body,
- body digest/hash,
- parse status,
- revision or equivalent optimistic version field,
- created_at,
- updated_at.

It is acceptable to implement this as a new `documents` table plus helper tables, or as a carefully designed extension around `artifacts`, as long as:

1. universal-header fields are structurally queryable,
2. body text is persisted in PostgreSQL,
3. `template+subtype` remains the canonical classification key,
4. future routing/retrieval does not depend on reparsing raw markdown on every read.

### 3. Add deterministic document section / anchor projection

Future artifact review depends on stable anchors and section counts. Add a bounded projection layer for headings/sections now.

At minimum, persist:

- parent document id,
- ordinal,
- heading text,
- heading level,
- stable anchor slug,
- body excerpt or section text slice,
- optional search text projection.

This is infrastructure, not review logic.

### 4. Add bounded document association support

The current artifact model is too narrow for future flows. Add a normalized way to associate a typed document with:

- zero or more WorkItems,
- zero or more Sessions,
- zero or more Handoffs,
- project-global scope when no narrower object exists.

This may be one generalized association table or several narrow relation tables. Keep it deterministic and typed.

### 5. Seed real repository documents into the new authority tables

Backfill the new typed-document store with the current real project truth set.

At minimum, seed:

- active T1 contract set,
- key T3 Nimbus packets already accepted,
- key T4/T5/T6/T7 coordination docs already used as product truth or governance truth.

Use actual repository content, not synthetic placeholder bodies.

Update `.seatloom/bootstrap/source-map.yaml` so the provenance of document seeding is explicit.

### 6. Add Rust DB models and read repositories

Add typed DB models and read methods for the new document authority layer.

Minimum read capability:

- list documents by project,
- filter by `template`,
- filter by `subtype`,
- get document by id,
- list document sections,
- list object associations for a document.

Keep it read-only in this packet.

### 7. Add focused tests

Add focused tests for:

- header parsing of real template fields,
- subtype allow-list handling,
- section extraction / anchor generation,
- repository filtering by `template` and `subtype`,
- seed/integration verification for the new document rows.

DB integration tests may stay `#[ignore]` if they require PostgreSQL, but they must be real and ready for Flux to run on a postgres-capable seat.

## Acceptance Criteria

1. `docs/architecture-design.md` and `docs/architecture-decisions.md` no longer claim file-first / SQLite FTS5 as the steady-state P0 authority.
2. PostgreSQL schema now includes a first-class typed-document authority layer with structured universal-header fields plus persisted body text.
3. `template+subtype` remains enforced as the canonical classification key.
4. Document sections / anchors are persisted deterministically for future artifact review use.
5. Document-to-object association support exists for WorkItems / Sessions / Handoffs / project scope.
6. The seed includes real repository-backed typed documents, not only metadata rows.
7. Rust DB models and repositories can read the new document authority structures.
8. No UI, no route-engine, no Tauri behavior widening, and no business automation is added.
9. `$HOME/.cargo/bin/cargo check` passes.
10. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
11. If a postgres-capable seat is available, the DB verification path for this packet is documented and ready.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
scripts/verify-postgres-baseline.sh
```

If `scripts/verify-postgres-baseline.sh` needs extension or a sibling verification script for the new document tables, add that in-scope and report exact usage.

If local postgres/docker is unavailable on your seat, state that explicitly, but keep the verification path runnable for Flux on a postgres-capable seat.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Authority realignment summary
4. Schema additions
5. Seed expansion summary
6. Rust DB model/repository additions
7. Validation commands and results
8. Residual non-goals kept out
9. Exact branch and commit hash
10. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Architecture docs are realigned to PostgreSQL authority.
- [ ] New typed-document persistence exists in PostgreSQL.
- [ ] Real repository markdown content is seeded into the new authority tables.
- [ ] Rust DB read repositories for the new document layer exist.
- [ ] Validation commands are run or explicitly environment-blocked with a Flux-ready fallback path.
- [ ] Branch is pushed and exact commit hash is reported.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_postgres_documents.txt
[Nimbus -> Lyra] PostgreSQL Authority + Typed Documents
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
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_postgres_documents /tmp/nimbus_to_lyra_postgres_documents.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_postgres_documents
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
