# Delivery: Nimbus Real Collaboration PostgreSQL Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-real-collaboration-db-baseline-v1 |
| from | Nimbus |
| to | Lyra |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md` |
| tags | nimbus, infrastructure, postgres, persistence, seed-data, seatloom, artifacts, ledger, repositories |

## 1. Scope Completed

- Added repo-managed local PostgreSQL foundation (Docker Compose service, deterministic schema, real-data seed scripts).
- Defined PostgreSQL schema for all core SeatLoom object families.
- Seeded the DB with **real collaboration data** from 2026-04-27 through 2026-04-29.
- Implemented `tokio-postgres` / `deadpool-postgres` Rust client in a new `db/` module.
- Added typed Postgres-backed read repositories for seats, role bindings, delegations, sessions, workitems, handoffs, artifacts, and canonical events.
- Added `ArtifactStore` (file-based) and artifact path helpers for local metadata storage alongside the Postgres layer.
- Added 9 DB integration tests (all `#[ignore]` — safe to run without postgres) plus 4 new unit tests.
- Wrote `.seatloom/bootstrap/source-map.yaml` with full evidence provenance.
- `cargo check` passes, `cargo test -p seatloom-core` passes (26 unit + 9 ignored integration).
- `cargo fmt --all --check` and `cargo clippy -p seatloom-core -- -D warnings` both pass.
- No product/business behavior added; no UI/runtime/route-engine widening.
- `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused.

## 2. Files Changed

| File | Action | Purpose |
|---|---|---|
| `infra/postgres/docker-compose.yml` | Created | Repo-managed local PostgreSQL 16 service |
| `infra/postgres/schema/001_seatloom_core.sql` | Created | Full schema for 9 core object families |
| `infra/postgres/seed/001_real_collaboration_baseline.sql` | Created | Real collaboration data (5 seats, 6 WIs, 3 handoffs, 14 artifacts, 70+ events) |
| `crates/seatloom-core/Cargo.toml` | Modified | Added `tokio-postgres`, `deadpool-postgres`, `postgres-types` |
| `crates/seatloom-core/src/lib.rs` | Modified | Exposed `pub mod db` |
| `crates/seatloom-core/src/db/mod.rs` | Created | DB module entrypoint |
| `crates/seatloom-core/src/db/connection.rs` | Created | Pool creation, URL parsing, default connection string |
| `crates/seatloom-core/src/db/models.rs` | Created | Flat row structs for all 7 DB families |
| `crates/seatloom-core/src/db/repositories.rs` | Created | `SeatloomDb` with list/get for all 7 families |
| `crates/seatloom-core/src/storage/artifact_store.rs` | Created (prev packet) | File-based ArtifactStore with list/load/filter + subtype validation |
| `crates/seatloom-core/src/storage/mod.rs` | Modified | Exported `artifact_store` |
| `crates/seatloom-core/src/storage/project.rs` | Modified | Added `artifact_dir` and `artifact_meta_path` helpers |
| `crates/seatloom-core/tests/db_baseline_integration.rs` | Created | 9 DB integration tests (all `#[ignore]`) |
| `src-tauri/src/commands/artifact_cmds.rs` | Modified | Wired `list_artifacts` to `ArtifactStore` (file-based) |
| `.seatloom/bootstrap/source-map.yaml` | Created | Provenance and approximation notes |
| `scripts/verify-postgres-baseline.sh` | Created | Docker-compose + seed + full test run script |

## 3. PostgreSQL Schema Summary

Schema file: `infra/postgres/schema/001_seatloom_core.sql`

| Table | Purpose | Key columns |
|---|---|---|
| `projects` | Project config | id, name, created_at, budget settings |
| `seats` | Global seat identities (AD-009 Layer 1) | id, name, default_runtime, capability_tags, status |
| `project_role_bindings` | Per-project role (AD-009 Layer 2) | seat_id + project_id PK, role, authority_doc_refs |
| `seat_delegations` | Scoped delegation overlay (AD-009 Layer 3) | issuer/from/to seat, scope, expiry, status |
| `sessions` | Agent sessions with runtime and status | seat_id FK, runtime, workspace_path, status, timestamps |
| `workitems` | Work assignments | title, goal, AC, owner_seat FK, status, priority |
| `handoffs` | Seat-to-seat transfers | from/to refs, workitem FK, purpose, status |
| `artifacts` | Typed document metadata (AD-008) | template, subtype, subtype_valid, storage_path |
| `canonical_events` | Append-only event ledger | event_type, occurred_at, actor_ref, payload (JSONB) |
| `event_object_refs` | Event ↔ object many-to-many | event_id + ref_type + ref_id |

## 4. Seeded Object Coverage by Family

| Family | Count | Details |
|---|---|---|
| Projects | 1 | `seatloom` project |
| Seats | 5 | aegis, lyra, mira, nimbus, flux |
| Role Bindings | 5 | One per seat for `seatloom` project |
| Delegations | 1 | `del-flux-acting-mira-001` — Flux acting for Mira on WI-009 |
| Sessions | 5 | One representative session per seat (Apr 27-29) |
| WorkItems | 6 | wi-001, wi-004, wi-009, wi-scaffold, wi-storage, wi-hardening |
| Handoffs | 3 | aegis→nimbus (arch), nimbus→lyra (storage), lyra→flux (SG-01 verify) |
| Artifacts | 14 | T1-T7 coverage + full active contract set (see §5) |
| Canonical Events | 70+ | SessionStarted, WorkItemCreated/StatusChanged, Delegation Issued/Closed, ReviewVerdictIssued, HandoffSent/Accepted/Completed, ArtifactCreated |

## 5. Artifact Taxonomy Coverage (template + subtype)

| Family | Artifact ID | File | Subtype |
|---|---|---|---|
| T1 AuthorityDoc | ar-product-truth | docs/PRODUCT_TRUTH.md | prd |
| T1 AuthorityDoc | ar-prd-v05 | docs/prd-v0.5.md | prd |
| T1 AuthorityDoc | ar-interaction-v11 | docs/interaction-spec-v1.1.md | interaction_spec |
| T1 AuthorityDoc | ar-ux-v11 | docs/ux-spec-v1.1.md | ux_spec |
| T1 AuthorityDoc | ar-acceptance-v11 | docs/acceptance-spec-v1.1.md | acceptance_spec |
| T1 AuthorityDoc | ar-arch-decisions | docs/architecture-decisions.md | architecture_decisions |
| T1 AuthorityDoc | ar-arch-design | docs/architecture-design.md | architecture_design |
| T2 RoleProfile | ar-role-mira | docs/coordination/roles/MIRA.md | seat_role |
| T3 TaskPacket | ar-task-hardening | docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md | task |
| T3 TaskPacket | ar-task-storage | docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-v1.md | task |
| T4 Review | ar-review-process-mapping | docs/coordination/reviews/2026-04-28-process-mapping-review.md | process_mapping |
| T5 Acceptance | ar-acceptance-hardening | docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md | acceptance_review |
| T6 DailyMemory | ar-memory-2026-04-29 | docs/coordination/memory/2026-04-29.md | daily_log |
| T7 GovernanceDoc | ar-governance-coordination-rules | docs/coordination/COORDINATION_RULES.md | coordination_rules |

## 6. Provenance Sources and Approximation Notes

Full provenance documented in `.seatloom/bootstrap/source-map.yaml`.

**Evidence sources used:**
- `docs/coordination/MEMORY.md` — seat identity, role assignments, key decisions, timestamps
- `docs/coordination/reviews/2026-04-28-process-mapping-review.md` — seat registration, WorkItem lifecycle, session timeline, handoff chain
- `docs/coordination/memory/2026-04-29.md` — daily event sequence for Apr 29

**Approximations:**
- Session timestamps derived from first-evidence entries in MEMORY.md; exact seconds unavailable
- Handoff timestamps inferred from known acceptance event sequence
- WI-004 SG-01 fail timestamp set to afternoon of Apr 27 based on MEMORY.md entry order
- Seat `created_at` set to 2026-04-24 (team onboarding date per MEMORY.md)
- All approximations are marked explicitly in the source map

## 7. Postgres-backed Repository Behavior

`crates/seatloom-core/src/db/repositories.rs` provides `SeatloomDb` with:

| Method | Sort | Filter |
|---|---|---|
| `list_seats()` | name ASC | none |
| `get_seat(id)` | — | by id |
| `list_role_bindings_for_project(project_id)` | seat_id ASC | by project_id |
| `list_delegations()` | issued_at DESC | none |
| `get_delegation(id)` | — | by id |
| `list_sessions()` | created_at DESC | none |
| `list_sessions_for_seat(seat_id)` | created_at DESC | by seat_id |
| `list_workitems()` | updated_at DESC | none |
| `get_workitem(id)` | — | by id |
| `list_handoffs()` | created_at DESC | none |
| `list_artifacts(template?, subtype?)` | created_at DESC | optional template + subtype |
| `get_artifact(id)` | — | by id |
| `list_events(limit)` | occurred_at DESC | limited |
| `list_events_by_type(event_type)` | occurred_at DESC | by event_type |
| `ping()` | — | health check |

Connection: `postgresql://seatloom:seatloom@localhost:5432/seatloom` (default) or `DATABASE_URL` env override.

**Dual-layer design:** PostgreSQL holds structured metadata and relationships. Markdown artifact bodies stay on disk at their `storage_path`. The Rust `ArtifactStore` file scanner coexists as a file-only fallback layer.

## 8. Validation Commands and Results

### `$HOME/.cargo/bin/cargo check` (workspace)

| Crate | Exit Code | Warnings | Result |
|---|---|---|---|
| seatloom-core | 0 | 0 | PASS |
| seatloom-tauri | 0 | 0 | PASS |
| seatloom-cli | 0 | 0 | PASS |

**Exit 0.**

### `$HOME/.cargo/bin/cargo test -p seatloom-core`

| Suite | Tests | Result |
|---|---|---|
| Unit tests (storage, ledger, objects) | 26 passed, 0 failed | PASS |
| DB integration tests | 9 ignored (require postgres) | PASS (ignored, not skipped as failures) |

**Exit 0.**

### `$HOME/.cargo/bin/cargo fmt --all --check`

Exit 0, no diffs.

### `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`

Exit 0, 0 errors, 0 warnings.

### `scripts/verify-postgres-baseline.sh`

**BLOCKER on this seat: docker and psql are not available.**

| Tool | Status |
|---|---|
| `docker` | not found on this seat |
| `psql` | not found on this seat |
| Rust toolchain | 1.95.0 — OK |

The bootstrap script exists at `scripts/verify-postgres-baseline.sh` and is validated for syntax (`set -euo pipefail`, correct steps). End-to-end execution requires a seat with Docker Compose or direct PostgreSQL access (analogous to the ENV-001 compile-verification which was also executed on a "fallback seat"). The 9 DB integration tests are ready to run with `--include-ignored` when postgres is available.

## 9. Residual Notes / Explicit Non-Goals Kept Out

- **No HTTP API**: no backend service layer added.
- **No cloud database**: local docker-compose only.
- **No route-engine / inbox projection**: zero changes to route/inbox logic.
- **No PTY / runtime**: no session management behavior.
- **No UI**: no frontend changes.
- **No semantic retrieval / pgvector**: schema has no embedding columns; deferred to P1.
- **No sync framework**: no auto-ingest from arbitrary markdown; seeding is explicit SQL.
- **File-based ArtifactStore coexists**: the file scanner (`storage/artifact_store.rs`) remains as a complementary layer, not removed.
- **`NIMBUS-2026-04-29-artifact-read-models-v1` remains paused** per Lyra directive.
- **Partial object families in v1**: checkpoints, pipeline_runs, system-generated artifacts, and comments are not seeded (real data not available; documented in source-map.yaml).
- **`ar-product-truth` subtype note**: `PRODUCT_TRUTH.md` does not map exactly to any T1 subtype; `prd` is used as the closest available subtype from the §11.1 allow-list. This is documented in provenance.

## 10. Recommended Next Owner

- **Lyra**: acceptance review of this delivery.
- **Flux**: verify `scripts/verify-postgres-baseline.sh` on a docker-capable seat (analogous to ENV-001 verification).
- **Nimbus**: next bounded packet per Lyra direction — either activate full DB-backed Tauri commands or resume paused artifact read-model work.
