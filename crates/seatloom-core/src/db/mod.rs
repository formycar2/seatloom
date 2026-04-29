/// PostgreSQL persistence layer for structured collaboration truth.
/// Evidence payloads (markdown files) remain on disk; structured metadata lives here.
///
/// Default connection: postgresql://seatloom:seatloom@localhost:5432/seatloom
/// Schema: infra/postgres/schema/001_seatloom_core.sql
/// Seed:   infra/postgres/seed/001_real_collaboration_baseline.sql
pub mod connection;
pub mod models;
pub mod repositories;

pub use connection::{create_pool, DbPool};
pub use repositories::SeatloomDb;
