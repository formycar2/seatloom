# Delivery: PostgreSQL Authority Alignment + Typed Document Persistence

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-postgres-authority-and-typed-documents-v1 |
| from | Nimbus |
| to | Lyra |
| tags | nimbus, infrastructure, postgres, authority, artifacts, documents, retrieval, schema |

## 1. Scope Completed

- Realigned `docs/architecture-decisions.md` (AD-011) and `docs/architecture-design.md` (§6, §12.3): PostgreSQL is now the documented canonical structured truth store; SQLite FTS5 direction superseded; file stores demoted to cache/export/compat artifacts.
- Added `infra/postgres/schema/002_document_authority.sql`: `documents`, `document_sections`, `document_associations` tables with GIN full-text indexes.
- Added `infra/postgres/seed/002_document_seed.sql`: 11 real repo documents seeded with headers, sections, and associations (T1–T7 coverage + full active contract set).
- Added `scripts/ingest-documents.sh`: reads real repo markdown files and upserts full body text into PostgreSQL.
- Added `crates/seatloom-core/src/db/document_parser.rs`: header parser, section extractor, anchor slugger, subtype validator — 16 unit tests all passing.
- Extended `models.rs`: `DocumentRow`, `DocumentSectionRow`, `DocumentAssociationRow`.
- Extended `repositories.rs`: `list_documents`, `get_document`, `list_document_sections`, `list_document_associations`.
- Added 6 DB integration tests (all `#[ignore]`, ready for Flux postgres-capable verification).
- Updated `.seatloom/bootstrap/source-map.yaml` to v2 with authority direction statement.
- No UI work. No product behavior. No Tauri surface widening.

## 2. Files Changed

| File | Action |
|---|---|
| `docs/architecture-decisions.md` | Modified — AD-011 realigned to PostgreSQL |
| `docs/architecture-design.md` | Modified — §6 storage authority + §12.3 retrieval contract |
| `infra/postgres/schema/002_document_authority.sql` | Created |
| `infra/postgres/seed/002_document_seed.sql` | Created |
| `scripts/ingest-documents.sh` | Created |
| `crates/seatloom-core/src/db/document_parser.rs` | Created |
| `crates/seatloom-core/src/db/mod.rs` | Modified — exposed `document_parser` |
| `crates/seatloom-core/src/db/models.rs` | Modified — document row types added |
| `crates/seatloom-core/src/db/repositories.rs` | Modified — document read methods added |
| `crates/seatloom-core/tests/db_baseline_integration.rs` | Modified — 6 new document tests |
| `.seatloom/bootstrap/source-map.yaml` | Modified — v2 |

## 3. Authority Realignment Summary

**Before:** `docs/architecture-decisions.md` AD-011 declared SQLite FTS5 as "P0 存储后端（已冻结）". `docs/architecture-design.md` §12.3 specified `SQLite FTS5 virtual table` for L1/L2.

**After:** Both documents now declare PostgreSQL as the P0 retrieval authority:
- AD-011: "PostgreSQL 是 SeatLoom 结构化项目真相的唯一权威存储引擎。"
- `.seatloom/` files: "迁移输入、证据有效载荷、缓存和向后兼容层，不再是权威存储。"
- Retrieval contract updated to PostgreSQL tables + `tsvector`/GIN for L1/L2.
- L3 path: PostgreSQL + pgvector (P1 evaluation).

No competing authority claims remain in these docs.

## 4. Schema Additions

**`002_document_authority.sql`** adds three tables:

| Table | Purpose |
|---|---|
| `documents` | Typed markdown docs with all universal header fields, body text, body digest, parse status, revision counter |
| `document_sections` | Deterministic heading/anchor projection (ordinal, heading_level, anchor_slug, body_excerpt, search_text) |
| `document_associations` | Many-to-many: document ↔ workitem / session / handoff / project |

GIN indexes on `documents` and `document_sections` for PostgreSQL FTS (`tsvector`). Indexes on template, subtype, status, updated_at for structured L1 queries.

## 5. Seed Expansion Summary

**`002_document_seed.sql`** seeds 11 real repository documents:

| Family | ID | File |
|---|---|---|
| T1 | doc-product-truth | docs/PRODUCT_TRUTH.md |
| T1 | doc-prd-v05 | docs/prd-v0.5.md |
| T1 | doc-interaction-v11 | docs/interaction-spec-v1.1.md |
| T1 | doc-ux-v11 | docs/ux-spec-v1.1.md |
| T1 | doc-acceptance-v11 | docs/acceptance-spec-v1.1.md |
| T1 | doc-arch-decisions | docs/architecture-decisions.md |
| T1 | doc-arch-design | docs/architecture-design.md |
| T2 | doc-role-mira | docs/coordination/roles/MIRA.md |
| T3 | doc-task-hardening | NIMBUS-2026-04-29-foundation-hardening-v1.md |
| T3 | doc-task-db-baseline | NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md |
| T4 | doc-review-process-mapping | docs/coordination/reviews/2026-04-28-process-mapping-review.md |
| T5 | doc-acceptance-hardening | 2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md |
| T6 | doc-memory-2026-04-29 | docs/coordination/memory/2026-04-29.md |
| T7 | doc-coordination-rules | docs/coordination/COORDINATION_RULES.md |

Bodies are seeded as partial excerpts (parse_status='partial'). Full body text populated by `scripts/ingest-documents.sh` at ingest time.

10 representative document sections seeded. 13 document associations seeded.

## 6. Rust DB Model/Repository Additions

**`document_parser.rs`** (pure Rust, no DB):
- `parse_header(body)` — extracts all universal header fields from markdown tables
- `extract_sections(body)` — deterministic heading/anchor projection
- `slugify(text)` — stable anchor slug generation
- `validate_subtype(template, subtype)` — §11.1 allow-list enforcement
- 16 unit tests, all passing

**`models.rs`** additions:
- `DocumentRow` (23 fields including all universal header fields + body)
- `DocumentSectionRow`, `DocumentAssociationRow`

**`repositories.rs`** additions:
- `list_documents(project_id, template?, subtype?)` — sorted by updated_at DESC
- `get_document(id)`
- `list_document_sections(document_id)` — sorted by ordinal ASC
- `list_document_associations(document_id)`

## 7. Validation Commands and Results

| Command | Result |
|---|---|
| `$HOME/.cargo/bin/cargo check` | exit 0, 0 warnings |
| `$HOME/.cargo/bin/cargo test -p seatloom-core` | 42 passed, 15 ignored, 0 failed |
| `$HOME/.cargo/bin/cargo fmt --all --check` | exit 0 |
| `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` | exit 0, 0 errors |
| `scripts/verify-rust-foundation.sh` | exit 0, all checks passed |
| `scripts/verify-postgres-baseline.sh` | NOT RUN — docker not available on this seat |

## 8. Residual Non-Goals Kept Out

- No comment/review/thread objects (DS-07/DS-08)
- No `change_tier_record` (DS-09)
- No prompt-state persistence (DS-10/DS-11)
- No checkpoint/continuity-pack schema (DS-12/DS-13)
- No retrieval query journal (DS-16)
- No optimistic concurrency fields on existing tables (DS-17/DS-18/DS-19 — separate packet)
- No Tauri command wiring
- No UI changes
- `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused

## 9. Exact Branch and Commit Hash

| Field | Value |
|---|---|
| Branch | `track/infra-foundation` |
| Commit | `85c8a04d3c0cb66a972363ad4d23e648cb68a329` |
| Pushed to origin | yes |

## 10. Recommended Next Owner

- **Lyra / Flux**: acceptance + commit-pinned verification of the document authority layer (`scripts/ingest-documents.sh` + `scripts/verify-postgres-baseline.sh` on docker-capable seat with 002 schema applied).
- **Nimbus**: next bounded infra packet (`postgres-write-ingest-and-reconcile-contract-v1`).
