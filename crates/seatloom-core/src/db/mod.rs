/// PostgreSQL persistence layer for structured collaboration truth.
/// Evidence payloads (markdown files) remain on disk; structured metadata lives here.
///
/// Default connection: postgresql://seatloom:seatloom@localhost:5432/seatloom
/// Schema 001: infra/postgres/schema/001_seatloom_core.sql (core objects)
/// Schema 002: infra/postgres/schema/002_document_authority.sql (document layer)
/// Schema 003: infra/postgres/schema/003_write_ingest_reconcile.sql (reconcile bookkeeping)
/// Schema 004: infra/postgres/schema/004_operational_review_and_continuity.sql (checkpoints/reviews/pipeline runs)
/// Seed 001:   infra/postgres/seed/001_real_collaboration_baseline.sql
/// Seed 002:   infra/postgres/seed/002_document_seed.sql
/// Seed 003:   infra/postgres/seed/003_operational_review_and_continuity_seed.sql
/// Full ingest: scripts/ingest-documents.sh
/// Reconcile:  seatloom reconcile  OR  scripts/verify-postgres-reconcile.sh
pub mod connection;
pub mod document_parser;
/// Closed value domains for the P0 event ledger (schema 010).
pub mod event_domain;
pub mod models;
pub mod reconcile;
pub mod repositories;
/// P0 step 2 write primitive: `append(event) -> projections`.
pub mod write_api;

pub use connection::{create_pool, DbPool};
pub use event_domain::{EventType, RefRole, RefType};
pub use repositories::SeatloomDb;
pub use write_api::{AppendOutcome, EventDraft, ObjectRefDraft, ProjectionEffect, WriteError};
