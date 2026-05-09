/// Postgres-backed read repositories for the SeatLoom collaboration truth baseline.
/// Uses tokio-postgres (via deadpool-postgres) for async Postgres access.
/// Sort orders are deterministic and documented inline.
use deadpool_postgres::Pool;
use thiserror::Error;
use tokio_postgres::Row;

use crate::db::models::{
    ArtifactRow, CanonicalEventRow, ChannelActionReceiptRow, CheckpointRow, DocumentAssociationRow,
    DocumentRow, DocumentSectionRow, DocumentVersionRow, HandoffReceiptRow, HandoffRow,
    PipelineRunRow, ProjectRoleBindingRow, ProjectRow, PromptActionRow, PromptInstanceRow,
    ReconcileItemRow, ReconcileRunRow, ReviewCommentRow, ReviewThreadRow, SeatDelegationRow,
    SeatRow, SessionRow, WorkItemRow,
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
    // Projects — sort: name ascending
    // =========================================================================

    pub async fn list_projects(&self) -> Result<Vec<ProjectRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, created_at, worker_budget_tokens, supervisor_budget_tokens \
                 FROM projects ORDER BY name ASC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_project).collect())
    }

    pub async fn get_project(&self, project_id: &str) -> Result<Option<ProjectRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, created_at, worker_budget_tokens, supervisor_budget_tokens \
                 FROM projects WHERE id = $1",
                &[&project_id],
            )
            .await?;
        Ok(rows.first().map(row_to_project))
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
    // Supervisor IM support (R2)
    // =========================================================================

    /// Append a supervisor-routed message as a canonical_events row.
    /// Also writes event_object_refs to target the seat for later filtering.
    /// Returns the stored row.
    pub async fn append_supervisor_message(
        &self,
        event_id: &str,
        actor_ref: &str,
        target_seat_id: Option<&str>,
        content: &str,
        event_type: &str, // 'SupervisorMessage' | 'SeatResponse'
    ) -> Result<CanonicalEventRow, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let payload = serde_json::json!({
            "content": content,
            "target_seat_id": target_seat_id,
        });
        client
            .execute(
                "INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) \
                 VALUES ($1, $2, NOW(), $3, $4)",
                &[&event_id, &event_type, &actor_ref, &payload],
            )
            .await?;
        if let Some(target) = target_seat_id {
            let _ = client
                .execute(
                    "INSERT INTO event_object_refs (event_id, ref_type, ref_id) \
                     VALUES ($1, 'seat', $2) ON CONFLICT DO NOTHING",
                    &[&event_id, &target],
                )
                .await;
        }
        // Read back the row we just wrote (captures DB-generated created_at + occurred_at).
        let row = client
            .query_one(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at \
                 FROM canonical_events WHERE id = $1",
                &[&event_id],
            )
            .await?;
        Ok(row_to_event(&row))
    }

    /// List supervisor-flavoured messages + related activity events, optionally
    /// filtered to events relevant to a given seat. A seat-targeted feed shows:
    ///   - events authored by the seat         (`actor_ref = 'seat:<seat_id>'`)
    ///   - messages addressed to the seat      (`payload->>'target_seat_id' = <seat_id>`)
    /// so both the user's outbound SupervisorMessages and the seat's own
    /// SessionStarted/HandoffSent/etc. events appear in its chat.
    /// Sort: occurred_at ascending (chat-oriented).
    pub async fn list_supervisor_messages(
        &self,
        target_seat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = if let Some(seat_id) = target_seat_id {
            let actor_ref = format!("seat:{}", seat_id);
            client
                .query(
                    "SELECT id, event_type, occurred_at, actor_ref, payload, created_at \
                     FROM canonical_events \
                     WHERE actor_ref = $1 \
                        OR payload->>'target_seat_id' = $2 \
                     ORDER BY occurred_at ASC LIMIT $3",
                    &[&actor_ref, &seat_id, &limit],
                )
                .await?
        } else {
            // Supervisor view: all events in the ledger form the global activity feed.
            client
                .query(
                    "SELECT id, event_type, occurred_at, actor_ref, payload, created_at \
                     FROM canonical_events \
                     ORDER BY occurred_at ASC LIMIT $1",
                    &[&limit],
                )
                .await?
        };
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

    // =========================================================================
    // Reconcile bookkeeping (schema 003) — sort: started_at descending
    // =========================================================================

    /// List reconcile runs sorted by started_at descending.
    pub async fn list_reconcile_runs(&self, limit: i64) -> Result<Vec<ReconcileRunRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, trigger, status, scanned, inserted, updated, unchanged, \
                 failed, conflicted, started_at, completed_at \
                 FROM reconcile_runs ORDER BY started_at DESC LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_reconcile_run).collect())
    }

    pub async fn get_reconcile_run(
        &self,
        run_id: &str,
    ) -> Result<Option<ReconcileRunRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, trigger, status, scanned, inserted, updated, unchanged, \
                 failed, conflicted, started_at, completed_at \
                 FROM reconcile_runs WHERE id = $1",
                &[&run_id],
            )
            .await?;
        Ok(rows.first().map(row_to_reconcile_run))
    }

    /// List all items for a reconcile run sorted by file_path.
    pub async fn list_reconcile_items(
        &self,
        run_id: &str,
    ) -> Result<Vec<ReconcileItemRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, run_id, file_path, document_id, outcome, \
                 digest_before, digest_after, revision_before, revision_after, \
                 parse_status, failure_reason, created_at \
                 FROM reconcile_items WHERE run_id = $1 ORDER BY file_path ASC",
                &[&run_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_reconcile_item).collect())
    }

    /// List version history for a document sorted by revision ascending.
    pub async fn list_document_versions(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentVersionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, document_id, revision, body_digest, body_text, \
                 header_snapshot, run_id, created_at \
                 FROM document_versions WHERE document_id = $1 ORDER BY revision ASC",
                &[&document_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_document_version).collect())
    }

    // =========================================================================
    // Operational review + continuity (schema 004)
    // =========================================================================

    /// List checkpoints sorted by created_at descending.
    pub async fn list_checkpoints(&self, limit: i64) -> Result<Vec<CheckpointRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, session_id, trigger, summary_what_was_done, summary_current_state, \
                 summary_open_questions, summary_quality, artifact_ids_at_checkpoint, branch, \
                 last_commit, transcript_tail_ref, continuity_tier0, continuity_tier1, \
                 continuity_tier2, continuity_budget_tokens, delta_context, created_at \
                 FROM checkpoints ORDER BY created_at DESC, id ASC LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_checkpoint).collect())
    }

    /// List checkpoints for one session sorted by created_at descending.
    pub async fn list_checkpoints_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<CheckpointRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, session_id, trigger, summary_what_was_done, summary_current_state, \
                 summary_open_questions, summary_quality, artifact_ids_at_checkpoint, branch, \
                 last_commit, transcript_tail_ref, continuity_tier0, continuity_tier1, \
                 continuity_tier2, continuity_budget_tokens, delta_context, created_at \
                 FROM checkpoints WHERE session_id = $1 ORDER BY created_at DESC, id ASC",
                &[&session_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_checkpoint).collect())
    }

    pub async fn get_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, session_id, trigger, summary_what_was_done, summary_current_state, \
                 summary_open_questions, summary_quality, artifact_ids_at_checkpoint, branch, \
                 last_commit, transcript_tail_ref, continuity_tier0, continuity_tier1, \
                 continuity_tier2, continuity_budget_tokens, delta_context, created_at \
                 FROM checkpoints WHERE id = $1",
                &[&checkpoint_id],
            )
            .await?;
        Ok(rows.first().map(row_to_checkpoint))
    }

    /// List all handoff receipts sorted by acknowledged_at descending.
    pub async fn list_handoff_receipts(&self) -> Result<Vec<HandoffReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, handoff_id, acknowledged_by, acknowledged_at, note, \
                 source_channel, created_at \
                 FROM handoff_receipts ORDER BY acknowledged_at DESC, id ASC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_handoff_receipt).collect())
    }

    /// List all receipts for a handoff sorted by acknowledged_at descending.
    pub async fn list_handoff_receipts_for_handoff(
        &self,
        handoff_id: &str,
    ) -> Result<Vec<HandoffReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, handoff_id, acknowledged_by, acknowledged_at, note, \
                 source_channel, created_at \
                 FROM handoff_receipts WHERE handoff_id = $1 \
                 ORDER BY acknowledged_at DESC, id ASC",
                &[&handoff_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_handoff_receipt).collect())
    }

    pub async fn get_handoff_receipt(
        &self,
        receipt_id: &str,
    ) -> Result<Option<HandoffReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, handoff_id, acknowledged_by, acknowledged_at, note, \
                 source_channel, created_at \
                 FROM handoff_receipts WHERE id = $1",
                &[&receipt_id],
            )
            .await?;
        Ok(rows.first().map(row_to_handoff_receipt))
    }

    /// List pipeline runs sorted by started_at descending.
    pub async fn list_pipeline_runs(&self, limit: i64) -> Result<Vec<PipelineRunRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, pipeline_id, workitem_id, status, current_stage, stage_index, \
                 trigger, initiated_by, result_summary, artifact_ids, evidence_refs, \
                 run_metadata, started_at, finished_at, created_at \
                 FROM pipeline_runs ORDER BY started_at DESC, id ASC LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_pipeline_run).collect())
    }

    /// List pipeline runs for a workitem sorted by started_at descending.
    pub async fn list_pipeline_runs_for_workitem(
        &self,
        workitem_id: &str,
    ) -> Result<Vec<PipelineRunRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, pipeline_id, workitem_id, status, current_stage, stage_index, \
                 trigger, initiated_by, result_summary, artifact_ids, evidence_refs, \
                 run_metadata, started_at, finished_at, created_at \
                 FROM pipeline_runs WHERE workitem_id = $1 \
                 ORDER BY started_at DESC, id ASC",
                &[&workitem_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_pipeline_run).collect())
    }

    pub async fn get_pipeline_run(&self, run_id: &str) -> Result<Option<PipelineRunRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, pipeline_id, workitem_id, status, current_stage, stage_index, \
                 trigger, initiated_by, result_summary, artifact_ids, evidence_refs, \
                 run_metadata, started_at, finished_at, created_at \
                 FROM pipeline_runs WHERE id = $1",
                &[&run_id],
            )
            .await?;
        Ok(rows.first().map(row_to_pipeline_run))
    }

    /// List all review threads for a project sorted by updated_at descending.
    pub async fn list_review_threads_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE project_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    /// List review threads by canonical target tuple.
    pub async fn list_review_threads_by_target(
        &self,
        target_kind: &str,
        target_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE target_kind = $1 AND target_id = $2 \
                 ORDER BY updated_at DESC, id ASC",
                &[&target_kind, &target_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn list_review_threads_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE document_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&document_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn list_review_threads_for_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE artifact_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&artifact_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn list_review_threads_for_workitem(
        &self,
        workitem_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE workitem_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&workitem_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn list_review_threads_for_handoff(
        &self,
        handoff_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE handoff_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&handoff_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn list_review_threads_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE session_id = $1 \
                 ORDER BY updated_at DESC, id ASC",
                &[&session_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_thread).collect())
    }

    pub async fn get_review_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<ReviewThreadRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, source_channel, mode, target_kind, target_id, \
                 document_id, artifact_id, workitem_id, handoff_id, session_id, anchor_kind, \
                 anchor_ref, anchor_label, title, status, requires_followup, review_tier, \
                 change_tier_record, evidence_refs, created_by, assigned_to, created_at, \
                 updated_at, resolved_at, resolved_by \
                 FROM review_threads WHERE id = $1",
                &[&thread_id],
            )
            .await?;
        Ok(rows.first().map(row_to_review_thread))
    }

    /// List review comments for a thread sorted by created_at ascending.
    pub async fn list_review_comments(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ReviewCommentRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, thread_id, parent_comment_id, author_ref, body_text, mode, \
                 source_channel, state, evidence_refs, comment_metadata, created_at, updated_at \
                 FROM review_comments WHERE thread_id = $1 ORDER BY created_at ASC, id ASC",
                &[&thread_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_review_comment).collect())
    }

    pub async fn get_review_comment(
        &self,
        comment_id: &str,
    ) -> Result<Option<ReviewCommentRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, thread_id, parent_comment_id, author_ref, body_text, mode, \
                 source_channel, state, evidence_refs, comment_metadata, created_at, updated_at \
                 FROM review_comments WHERE id = $1",
                &[&comment_id],
            )
            .await?;
        Ok(rows.first().map(row_to_review_comment))
    }

    // =========================================================================
    // Prompt authority (schema 005) — prompt_instances, prompt_actions
    // Sort: detected_at descending for instances, created_at ascending for actions
    // =========================================================================

    pub async fn create_prompt_instance(&self, r: &PromptInstanceRow) -> Result<(), DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        client
            .execute(
                "INSERT INTO prompt_instances \
                 (id, project_id, session_id, status, prompt_kind, prompt_policy, \
                  evidence_ref, evidence_preview, available_actions, \
                  assist_max_steps, assist_max_tokens, assist_steps_used, assist_tokens_used, \
                  expected_next_pattern, detected_at, resolved_at, resolved_by, \
                  result_event_id, created_at, updated_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)",
                &[
                    &r.id,
                    &r.project_id,
                    &r.session_id,
                    &r.status,
                    &r.prompt_kind,
                    &r.prompt_policy,
                    &r.evidence_ref,
                    &r.evidence_preview,
                    &r.available_actions,
                    &r.assist_max_steps,
                    &r.assist_max_tokens,
                    &r.assist_steps_used,
                    &r.assist_tokens_used,
                    &r.expected_next_pattern,
                    &r.detected_at,
                    &r.resolved_at,
                    &r.resolved_by,
                    &r.result_event_id,
                    &r.created_at,
                    &r.updated_at,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn list_prompt_instances_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<PromptInstanceRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, session_id, status, prompt_kind, prompt_policy, \
                 evidence_ref, evidence_preview, available_actions, \
                 assist_max_steps, assist_max_tokens, assist_steps_used, assist_tokens_used, \
                 expected_next_pattern, detected_at, resolved_at, resolved_by, \
                 result_event_id, created_at, updated_at \
                 FROM prompt_instances WHERE session_id = $1 ORDER BY detected_at DESC",
                &[&session_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_prompt_instance).collect())
    }

    pub async fn list_active_prompt_instances(
        &self,
        project_id: &str,
    ) -> Result<Vec<PromptInstanceRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, session_id, status, prompt_kind, prompt_policy, \
                 evidence_ref, evidence_preview, available_actions, \
                 assist_max_steps, assist_max_tokens, assist_steps_used, assist_tokens_used, \
                 expected_next_pattern, detected_at, resolved_at, resolved_by, \
                 result_event_id, created_at, updated_at \
                 FROM prompt_instances WHERE project_id = $1 AND status = 'active' \
                 ORDER BY detected_at DESC",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_prompt_instance).collect())
    }

    pub async fn get_prompt_instance(
        &self,
        id: &str,
    ) -> Result<Option<PromptInstanceRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, session_id, status, prompt_kind, prompt_policy, \
                 evidence_ref, evidence_preview, available_actions, \
                 assist_max_steps, assist_max_tokens, assist_steps_used, assist_tokens_used, \
                 expected_next_pattern, detected_at, resolved_at, resolved_by, \
                 result_event_id, created_at, updated_at \
                 FROM prompt_instances WHERE id = $1",
                &[&id],
            )
            .await?;
        Ok(rows.first().map(row_to_prompt_instance))
    }

    pub async fn append_prompt_action(&self, r: &PromptActionRow) -> Result<(), DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        client
            .execute(
                "INSERT INTO prompt_actions \
                 (id, prompt_id, action_kind, actor_ref, source_channel, note, \
                  steps_budget_used, tokens_budget_used, result_status, result_event_id, created_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
                &[
                    &r.id, &r.prompt_id, &r.action_kind, &r.actor_ref,
                    &r.source_channel, &r.note,
                    &r.steps_budget_used, &r.tokens_budget_used,
                    &r.result_status, &r.result_event_id, &r.created_at,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn list_prompt_actions_for_prompt(
        &self,
        prompt_id: &str,
    ) -> Result<Vec<PromptActionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, prompt_id, action_kind, actor_ref, source_channel, note, \
                 steps_budget_used, tokens_budget_used, result_status, result_event_id, created_at \
                 FROM prompt_actions WHERE prompt_id = $1 ORDER BY created_at ASC",
                &[&prompt_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_prompt_action).collect())
    }

    // =========================================================================
    // Channel-action receipts (schema 005)
    // Sort: created_at descending
    // =========================================================================

    pub async fn create_channel_action_receipt(
        &self,
        r: &ChannelActionReceiptRow,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        client
            .execute(
                "INSERT INTO channel_action_receipts \
                 (id, project_id, target_kind, target_id, action_kind, actor_ref, \
                  source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                  expected_revision, applied_revision, receipt_status, result_event_id, created_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
                &[
                    &r.id, &r.project_id, &r.target_kind, &r.target_id,
                    &r.action_kind, &r.actor_ref, &r.source_channel,
                    &r.note, &r.evidence_refs, &r.policy_summary,
                    &r.idempotency_key, &r.expected_revision, &r.applied_revision,
                    &r.receipt_status, &r.result_event_id, &r.created_at,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn list_channel_action_receipts_for_target(
        &self,
        target_kind: &str,
        target_id: &str,
    ) -> Result<Vec<ChannelActionReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts \
                 WHERE target_kind = $1 AND target_id = $2 \
                 ORDER BY created_at DESC",
                &[&target_kind, &target_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_channel_action_receipt).collect())
    }

    pub async fn list_channel_action_receipts(
        &self,
        project_id: &str,
        source_channel: Option<&str>,
        receipt_status: Option<&str>,
    ) -> Result<Vec<ChannelActionReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows =
            match (source_channel, receipt_status) {
                (Some(ch), Some(st)) => client
                    .query(
                        "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts \
                 WHERE project_id = $1 AND source_channel = $2 AND receipt_status = $3 \
                 ORDER BY created_at DESC",
                        &[&project_id, &ch, &st],
                    )
                    .await?,
                (Some(ch), None) => client
                    .query(
                        "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts \
                 WHERE project_id = $1 AND source_channel = $2 \
                 ORDER BY created_at DESC",
                        &[&project_id, &ch],
                    )
                    .await?,
                (None, Some(st)) => client
                    .query(
                        "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts \
                 WHERE project_id = $1 AND receipt_status = $2 \
                 ORDER BY created_at DESC",
                        &[&project_id, &st],
                    )
                    .await?,
                (None, None) => client
                    .query(
                        "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts \
                 WHERE project_id = $1 \
                 ORDER BY created_at DESC",
                        &[&project_id],
                    )
                    .await?,
            };
        Ok(rows.iter().map(row_to_channel_action_receipt).collect())
    }

    pub async fn get_channel_action_receipt(
        &self,
        id: &str,
    ) -> Result<Option<ChannelActionReceiptRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, target_kind, target_id, action_kind, actor_ref, \
                 source_channel, note, evidence_refs, policy_summary, idempotency_key, \
                 expected_revision, applied_revision, receipt_status, result_event_id, created_at \
                 FROM channel_action_receipts WHERE id = $1",
                &[&id],
            )
            .await?;
        Ok(rows.first().map(row_to_channel_action_receipt))
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

fn row_to_project(r: &Row) -> ProjectRow {
    ProjectRow {
        id: r.get(0),
        name: r.get(1),
        created_at: r.get(2),
        worker_budget_tokens: r.get(3),
        supervisor_budget_tokens: r.get(4),
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

fn row_to_reconcile_run(r: &Row) -> ReconcileRunRow {
    ReconcileRunRow {
        id: r.get(0),
        trigger: r.get(1),
        status: r.get(2),
        scanned: r.get(3),
        inserted: r.get(4),
        updated: r.get(5),
        unchanged: r.get(6),
        failed: r.get(7),
        conflicted: r.get(8),
        started_at: r.get(9),
        completed_at: r.get(10),
    }
}

fn row_to_reconcile_item(r: &Row) -> ReconcileItemRow {
    ReconcileItemRow {
        id: r.get(0),
        run_id: r.get(1),
        file_path: r.get(2),
        document_id: r.get(3),
        outcome: r.get(4),
        digest_before: r.get(5),
        digest_after: r.get(6),
        revision_before: r.get(7),
        revision_after: r.get(8),
        parse_status: r.get(9),
        failure_reason: r.get(10),
        created_at: r.get(11),
    }
}

fn row_to_document_version(r: &Row) -> DocumentVersionRow {
    DocumentVersionRow {
        id: r.get(0),
        document_id: r.get(1),
        revision: r.get(2),
        body_digest: r.get(3),
        body_text: r.get(4),
        header_snapshot: r.get(5),
        run_id: r.get(6),
        created_at: r.get(7),
    }
}

fn row_to_checkpoint(r: &Row) -> CheckpointRow {
    CheckpointRow {
        id: r.get(0),
        session_id: r.get(1),
        trigger: r.get(2),
        summary_what_was_done: r.get(3),
        summary_current_state: r.get(4),
        summary_open_questions: r.get(5),
        summary_quality: r.get(6),
        artifact_ids_at_checkpoint: r.get(7),
        branch: r.get(8),
        last_commit: r.get(9),
        transcript_tail_ref: r.get(10),
        continuity_tier0: r.get(11),
        continuity_tier1: r.get(12),
        continuity_tier2: r.get(13),
        continuity_budget_tokens: r.get(14),
        delta_context: r.get(15),
        created_at: r.get(16),
    }
}

fn row_to_handoff_receipt(r: &Row) -> HandoffReceiptRow {
    HandoffReceiptRow {
        id: r.get(0),
        handoff_id: r.get(1),
        acknowledged_by: r.get(2),
        acknowledged_at: r.get(3),
        note: r.get(4),
        source_channel: r.get(5),
        created_at: r.get(6),
    }
}

fn row_to_pipeline_run(r: &Row) -> PipelineRunRow {
    PipelineRunRow {
        id: r.get(0),
        pipeline_id: r.get(1),
        workitem_id: r.get(2),
        status: r.get(3),
        current_stage: r.get(4),
        stage_index: r.get(5),
        trigger: r.get(6),
        initiated_by: r.get(7),
        result_summary: r.get(8),
        artifact_ids: r.get(9),
        evidence_refs: r.get(10),
        run_metadata: r.get(11),
        started_at: r.get(12),
        finished_at: r.get(13),
        created_at: r.get(14),
    }
}

fn row_to_review_thread(r: &Row) -> ReviewThreadRow {
    ReviewThreadRow {
        id: r.get(0),
        project_id: r.get(1),
        source_channel: r.get(2),
        mode: r.get(3),
        target_kind: r.get(4),
        target_id: r.get(5),
        document_id: r.get(6),
        artifact_id: r.get(7),
        workitem_id: r.get(8),
        handoff_id: r.get(9),
        session_id: r.get(10),
        anchor_kind: r.get(11),
        anchor_ref: r.get(12),
        anchor_label: r.get(13),
        title: r.get(14),
        status: r.get(15),
        requires_followup: r.get(16),
        review_tier: r.get(17),
        change_tier_record: r.get(18),
        evidence_refs: r.get(19),
        created_by: r.get(20),
        assigned_to: r.get(21),
        created_at: r.get(22),
        updated_at: r.get(23),
        resolved_at: r.get(24),
        resolved_by: r.get(25),
    }
}

fn row_to_review_comment(r: &Row) -> ReviewCommentRow {
    ReviewCommentRow {
        id: r.get(0),
        thread_id: r.get(1),
        parent_comment_id: r.get(2),
        author_ref: r.get(3),
        body_text: r.get(4),
        mode: r.get(5),
        source_channel: r.get(6),
        state: r.get(7),
        evidence_refs: r.get(8),
        comment_metadata: r.get(9),
        created_at: r.get(10),
        updated_at: r.get(11),
    }
}

fn row_to_prompt_instance(r: &Row) -> PromptInstanceRow {
    PromptInstanceRow {
        id: r.get(0),
        project_id: r.get(1),
        session_id: r.get(2),
        status: r.get(3),
        prompt_kind: r.get(4),
        prompt_policy: r.get(5),
        evidence_ref: r.get(6),
        evidence_preview: r.get(7),
        available_actions: r.get(8),
        assist_max_steps: r.get(9),
        assist_max_tokens: r.get(10),
        assist_steps_used: r.get(11),
        assist_tokens_used: r.get(12),
        expected_next_pattern: r.get(13),
        detected_at: r.get(14),
        resolved_at: r.get(15),
        resolved_by: r.get(16),
        result_event_id: r.get(17),
        created_at: r.get(18),
        updated_at: r.get(19),
    }
}

fn row_to_prompt_action(r: &Row) -> PromptActionRow {
    PromptActionRow {
        id: r.get(0),
        prompt_id: r.get(1),
        action_kind: r.get(2),
        actor_ref: r.get(3),
        source_channel: r.get(4),
        note: r.get(5),
        steps_budget_used: r.get(6),
        tokens_budget_used: r.get(7),
        result_status: r.get(8),
        result_event_id: r.get(9),
        created_at: r.get(10),
    }
}

fn row_to_channel_action_receipt(r: &Row) -> ChannelActionReceiptRow {
    ChannelActionReceiptRow {
        id: r.get(0),
        project_id: r.get(1),
        target_kind: r.get(2),
        target_id: r.get(3),
        action_kind: r.get(4),
        actor_ref: r.get(5),
        source_channel: r.get(6),
        note: r.get(7),
        evidence_refs: r.get(8),
        policy_summary: r.get(9),
        idempotency_key: r.get(10),
        expected_revision: r.get(11),
        applied_revision: r.get(12),
        receipt_status: r.get(13),
        result_event_id: r.get(14),
        created_at: r.get(15),
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("connection pool error: {0}")]
    Pool(deadpool_postgres::PoolError),
}
