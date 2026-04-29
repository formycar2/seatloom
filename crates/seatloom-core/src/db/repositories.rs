/// Postgres-backed read repositories for the SeatLoom collaboration truth baseline.
/// Uses tokio-postgres (via deadpool-postgres) for async Postgres access.
/// Sort orders are deterministic and documented inline.
use deadpool_postgres::Pool;
use thiserror::Error;
use tokio_postgres::Row;

use crate::db::models::{
    ArtifactRow, CanonicalEventRow, DocumentAssociationRow, DocumentRow, DocumentSectionRow,
    HandoffRow, ProjectRoleBindingRow, SeatDelegationRow, SeatRow, SessionRow, WorkItemRow,
};

pub struct SeatloomDb {
    pool: Pool,
}

impl SeatloomDb {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // Health check
    // =========================================================================

    pub async fn ping(&self) -> Result<(), DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        client.query_one("SELECT 1", &[]).await?;
        Ok(())
    }

    // =========================================================================
    // Seats — sort: name ascending
    // =========================================================================

    pub async fn list_seats(&self) -> Result<Vec<SeatRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, default_runtime, capability_tags, status, created_at \
                 FROM seats ORDER BY name ASC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_seat).collect())
    }

    pub async fn get_seat(&self, seat_id: &str) -> Result<Option<SeatRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, default_runtime, capability_tags, status, created_at \
                 FROM seats WHERE id = $1",
                &[&seat_id],
            )
            .await?;
        Ok(rows.first().map(row_to_seat))
    }

    // =========================================================================
    // Project role bindings — sort: seat_id ascending
    // =========================================================================

    pub async fn list_role_bindings_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectRoleBindingRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT seat_id, project_id, role, authority_doc_refs, constraints, \
                 collaboration_template_ref, active_delegation_id \
                 FROM project_role_bindings WHERE project_id = $1 ORDER BY seat_id ASC",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_role_binding).collect())
    }

    // =========================================================================
    // Delegations — sort: issued_at descending
    // =========================================================================

    pub async fn list_delegations(&self) -> Result<Vec<SeatDelegationRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, issuer_seat_id, from_seat_id, to_seat_id, workitem_id, \
                 scope_description, issued_at, expires_at, status \
                 FROM seat_delegations ORDER BY issued_at DESC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_delegation).collect())
    }

    pub async fn get_delegation(
        &self,
        delegation_id: &str,
    ) -> Result<Option<SeatDelegationRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, issuer_seat_id, from_seat_id, to_seat_id, workitem_id, \
                 scope_description, issued_at, expires_at, status \
                 FROM seat_delegations WHERE id = $1",
                &[&delegation_id],
            )
            .await?;
        Ok(rows.first().map(row_to_delegation))
    }

    // =========================================================================
    // Sessions — sort: created_at descending
    // =========================================================================

    pub async fn list_sessions(&self) -> Result<Vec<SessionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, seat_id, runtime, native_session_id, workspace_path, branch, \
                 status, launch_pack_ref, last_checkpoint_id, pid, created_at, ended_at \
                 FROM sessions ORDER BY created_at DESC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_session).collect())
    }

    pub async fn list_sessions_for_seat(&self, seat_id: &str) -> Result<Vec<SessionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, seat_id, runtime, native_session_id, workspace_path, branch, \
                 status, launch_pack_ref, last_checkpoint_id, pid, created_at, ended_at \
                 FROM sessions WHERE seat_id = $1 ORDER BY created_at DESC",
                &[&seat_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_session).collect())
    }

    // =========================================================================
    // WorkItems — sort: updated_at descending
    // =========================================================================

    pub async fn list_workitems(&self) -> Result<Vec<WorkItemRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, title, goal, acceptance_criteria, owner_seat_id, \
                 status, priority, created_at, updated_at \
                 FROM workitems ORDER BY updated_at DESC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_workitem).collect())
    }

    pub async fn get_workitem(&self, workitem_id: &str) -> Result<Option<WorkItemRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, title, goal, acceptance_criteria, owner_seat_id, \
                 status, priority, created_at, updated_at \
                 FROM workitems WHERE id = $1",
                &[&workitem_id],
            )
            .await?;
        Ok(rows.first().map(row_to_workitem))
    }

    // =========================================================================
    // Handoffs — sort: created_at descending
    // =========================================================================

    pub async fn list_handoffs(&self) -> Result<Vec<HandoffRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, from_ref, to_ref, workitem_id, purpose, expected_outcome, \
                 required_receipt, status, created_at, sent_at \
                 FROM handoffs ORDER BY created_at DESC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_handoff).collect())
    }

    // =========================================================================
    // Artifacts — sort: created_at descending; optional template/subtype filter
    // =========================================================================

    pub async fn list_artifacts(
        &self,
        template_filter: Option<&str>,
        subtype_filter: Option<&str>,
    ) -> Result<Vec<ArtifactRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = match (template_filter, subtype_filter) {
            (Some(t), Some(s)) => {
                client
                    .query(
                        "SELECT id, template, subtype, subtype_valid, system_kind, title, summary, \
                         source_session_id, source_workitem_id, storage_path, created_at \
                         FROM artifacts WHERE template = $1 AND subtype = $2 ORDER BY created_at DESC",
                        &[&t, &s],
                    )
                    .await?
            }
            (Some(t), None) => {
                client
                    .query(
                        "SELECT id, template, subtype, subtype_valid, system_kind, title, summary, \
                         source_session_id, source_workitem_id, storage_path, created_at \
                         FROM artifacts WHERE template = $1 ORDER BY created_at DESC",
                        &[&t],
                    )
                    .await?
            }
            (None, Some(s)) => {
                client
                    .query(
                        "SELECT id, template, subtype, subtype_valid, system_kind, title, summary, \
                         source_session_id, source_workitem_id, storage_path, created_at \
                         FROM artifacts WHERE subtype = $1 ORDER BY created_at DESC",
                        &[&s],
                    )
                    .await?
            }
            (None, None) => {
                client
                    .query(
                        "SELECT id, template, subtype, subtype_valid, system_kind, title, summary, \
                         source_session_id, source_workitem_id, storage_path, created_at \
                         FROM artifacts ORDER BY created_at DESC",
                        &[],
                    )
                    .await?
            }
        };
        Ok(rows.iter().map(row_to_artifact).collect())
    }

    pub async fn get_artifact(&self, artifact_id: &str) -> Result<Option<ArtifactRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, template, subtype, subtype_valid, system_kind, title, summary, \
                 source_session_id, source_workitem_id, storage_path, created_at \
                 FROM artifacts WHERE id = $1",
                &[&artifact_id],
            )
            .await?;
        Ok(rows.first().map(row_to_artifact))
    }

    // =========================================================================
    // Canonical events — sort: occurred_at descending
    // =========================================================================

    pub async fn list_events(&self, limit: i64) -> Result<Vec<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at \
                 FROM canonical_events ORDER BY occurred_at DESC LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    pub async fn list_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at \
                 FROM canonical_events WHERE event_type = $1 ORDER BY occurred_at DESC",
                &[&event_type],
            )
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    // =========================================================================
    // Documents — schema 002 document authority layer
    // Sort: updated_at descending
    // =========================================================================

    /// List all documents for a project; optional template and subtype filters.
    pub async fn list_documents(
        &self,
        project_id: &str,
        template_filter: Option<&str>,
        subtype_filter: Option<&str>,
    ) -> Result<Vec<DocumentRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows =
            match (template_filter, subtype_filter) {
                (Some(t), Some(s)) => client
                    .query(
                        "SELECT id, project_id, artifact_id, template, subtype, subtype_valid, \
                         doc_id, title, status, author, doc_date, version, depends_on, \
                         supersedes, tags, file_path, body_text, body_digest, body_length, \
                         parse_status, revision, created_at, updated_at \
                         FROM documents WHERE project_id = $1 AND template = $2 AND subtype = $3 \
                         ORDER BY updated_at DESC",
                        &[&project_id, &t, &s],
                    )
                    .await?,
                (Some(t), None) => client
                    .query(
                        "SELECT id, project_id, artifact_id, template, subtype, subtype_valid, \
                         doc_id, title, status, author, doc_date, version, depends_on, \
                         supersedes, tags, file_path, body_text, body_digest, body_length, \
                         parse_status, revision, created_at, updated_at \
                         FROM documents WHERE project_id = $1 AND template = $2 \
                         ORDER BY updated_at DESC",
                        &[&project_id, &t],
                    )
                    .await?,
                (None, Some(s)) => client
                    .query(
                        "SELECT id, project_id, artifact_id, template, subtype, subtype_valid, \
                         doc_id, title, status, author, doc_date, version, depends_on, \
                         supersedes, tags, file_path, body_text, body_digest, body_length, \
                         parse_status, revision, created_at, updated_at \
                         FROM documents WHERE project_id = $1 AND subtype = $2 \
                         ORDER BY updated_at DESC",
                        &[&project_id, &s],
                    )
                    .await?,
                (None, None) => client
                    .query(
                        "SELECT id, project_id, artifact_id, template, subtype, subtype_valid, \
                         doc_id, title, status, author, doc_date, version, depends_on, \
                         supersedes, tags, file_path, body_text, body_digest, body_length, \
                         parse_status, revision, created_at, updated_at \
                         FROM documents WHERE project_id = $1 ORDER BY updated_at DESC",
                        &[&project_id],
                    )
                    .await?,
            };
        Ok(rows.iter().map(row_to_document).collect())
    }

    pub async fn get_document(&self, doc_id: &str) -> Result<Option<DocumentRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, artifact_id, template, subtype, subtype_valid, \
                 doc_id, title, status, author, doc_date, version, depends_on, \
                 supersedes, tags, file_path, body_text, body_digest, body_length, \
                 parse_status, revision, created_at, updated_at \
                 FROM documents WHERE id = $1",
                &[&doc_id],
            )
            .await?;
        Ok(rows.first().map(row_to_document))
    }

    /// List all sections for a document; sorted by ordinal ascending.
    pub async fn list_document_sections(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentSectionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, document_id, ordinal, heading_text, heading_level, anchor_slug, \
                 body_excerpt, search_text, created_at \
                 FROM document_sections WHERE document_id = $1 ORDER BY ordinal ASC",
                &[&document_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_document_section).collect())
    }

    /// List all object associations for a document.
    pub async fn list_document_associations(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentAssociationRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, document_id, assoc_type, assoc_id, is_primary, created_at \
                 FROM document_associations WHERE document_id = $1",
                &[&document_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_document_assoc).collect())
    }
}

// =============================================================================
// Row mapping helpers
// =============================================================================

fn row_to_seat(r: &Row) -> SeatRow {
    SeatRow {
        id: r.get(0),
        name: r.get(1),
        default_runtime: r.get(2),
        capability_tags: r.get(3),
        status: r.get(4),
        created_at: r.get(5),
    }
}

fn row_to_role_binding(r: &Row) -> ProjectRoleBindingRow {
    ProjectRoleBindingRow {
        seat_id: r.get(0),
        project_id: r.get(1),
        role: r.get(2),
        authority_doc_refs: r.get(3),
        constraints: r.get(4),
        collaboration_template_ref: r.get(5),
        active_delegation_id: r.get(6),
    }
}

fn row_to_delegation(r: &Row) -> SeatDelegationRow {
    SeatDelegationRow {
        id: r.get(0),
        issuer_seat_id: r.get(1),
        from_seat_id: r.get(2),
        to_seat_id: r.get(3),
        workitem_id: r.get(4),
        scope_description: r.get(5),
        issued_at: r.get(6),
        expires_at: r.get(7),
        status: r.get(8),
    }
}

fn row_to_session(r: &Row) -> SessionRow {
    SessionRow {
        id: r.get(0),
        seat_id: r.get(1),
        runtime: r.get(2),
        native_session_id: r.get(3),
        workspace_path: r.get(4),
        branch: r.get(5),
        status: r.get(6),
        launch_pack_ref: r.get(7),
        last_checkpoint_id: r.get(8),
        pid: r.get(9),
        created_at: r.get(10),
        ended_at: r.get(11),
    }
}

fn row_to_workitem(r: &Row) -> WorkItemRow {
    WorkItemRow {
        id: r.get(0),
        title: r.get(1),
        goal: r.get(2),
        acceptance_criteria: r.get(3),
        owner_seat_id: r.get(4),
        status: r.get(5),
        priority: r.get(6),
        created_at: r.get(7),
        updated_at: r.get(8),
    }
}

fn row_to_handoff(r: &Row) -> HandoffRow {
    HandoffRow {
        id: r.get(0),
        from_ref: r.get(1),
        to_ref: r.get(2),
        workitem_id: r.get(3),
        purpose: r.get(4),
        expected_outcome: r.get(5),
        required_receipt: r.get(6),
        status: r.get(7),
        created_at: r.get(8),
        sent_at: r.get(9),
    }
}

fn row_to_artifact(r: &Row) -> ArtifactRow {
    ArtifactRow {
        id: r.get(0),
        template: r.get(1),
        subtype: r.get(2),
        subtype_valid: r.get(3),
        system_kind: r.get(4),
        title: r.get(5),
        summary: r.get(6),
        source_session_id: r.get(7),
        source_workitem_id: r.get(8),
        storage_path: r.get(9),
        created_at: r.get(10),
    }
}

fn row_to_event(r: &Row) -> CanonicalEventRow {
    CanonicalEventRow {
        id: r.get(0),
        event_type: r.get(1),
        occurred_at: r.get(2),
        actor_ref: r.get(3),
        payload: r.get(4),
        created_at: r.get(5),
    }
}

fn row_to_document(r: &Row) -> DocumentRow {
    DocumentRow {
        id: r.get(0),
        project_id: r.get(1),
        artifact_id: r.get(2),
        template: r.get(3),
        subtype: r.get(4),
        subtype_valid: r.get(5),
        doc_id: r.get(6),
        title: r.get(7),
        status: r.get(8),
        author: r.get(9),
        doc_date: r.get(10),
        version: r.get(11),
        depends_on: r.get(12),
        supersedes: r.get(13),
        tags: r.get(14),
        file_path: r.get(15),
        body_text: r.get(16),
        body_digest: r.get(17),
        body_length: r.get(18),
        parse_status: r.get(19),
        revision: r.get(20),
        created_at: r.get(21),
        updated_at: r.get(22),
    }
}

fn row_to_document_section(r: &Row) -> DocumentSectionRow {
    DocumentSectionRow {
        id: r.get(0),
        document_id: r.get(1),
        ordinal: r.get(2),
        heading_text: r.get(3),
        heading_level: r.get(4),
        anchor_slug: r.get(5),
        body_excerpt: r.get(6),
        search_text: r.get(7),
        created_at: r.get(8),
    }
}

fn row_to_document_assoc(r: &Row) -> DocumentAssociationRow {
    DocumentAssociationRow {
        id: r.get(0),
        document_id: r.get(1),
        assoc_type: r.get(2),
        assoc_id: r.get(3),
        is_primary: r.get(4),
        created_at: r.get(5),
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("connection pool error: {0}")]
    Pool(deadpool_postgres::PoolError),
}
