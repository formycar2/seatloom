/// Postgres-backed read repositories for the SeatLoom collaboration truth baseline.
/// Uses tokio-postgres (via deadpool-postgres) for async Postgres access.
/// Sort orders are deterministic and documented inline.
use deadpool_postgres::Pool;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use thiserror::Error;
use tokio_postgres::Row;

use crate::db::event_domain::{EventType, RefRole, RefType};
use crate::db::models::{
    AcceptanceRow, AcceptanceWithDispatchRow, ArchivedSessionRow, ArtifactRow, CanonicalEventRow,
    ChannelActionReceiptRow, CheckpointRow, DispatchDependencyRow, DispatchDependencyStatus,
    DispatchReadModel, DispatchRow, DocumentAssociationRow, DocumentRow, DocumentSectionRow,
    DocumentVersionRow, HandoffReceiptRow, HandoffRow, HistoryProjectRow, KnowledgeL2Pointer,
    KnowledgeMemoryCard, KnowledgeTopicNode, LedgerEventWithRefs, MemoryRelationRow, MemoryRow,
    ObjectRefRow, PipelineRunRow, ProjectRoleBindingRow, ProjectRow, PromptActionRow,
    PromptInstanceRow, ReconcileItemRow, ReconcileRunRow, ReviewCommentRow, ReviewThreadRow,
    SeatDelegationRow, SeatRow, SessionRow, TopicRow, VerdictFacts, WorkItemRow,
};
use crate::db::write_api::{append_event_in_tx, EventDraft, ObjectRefDraft};
use crate::objects::memory::MemoryLayer;
use crate::objects::seat::SeatRole;
use crate::objects::session::{Agent, Harness, ParseValueError};
use chrono::Utc;

pub struct SeatloomDb {
    pool: Pool,
}

impl SeatloomDb {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Checked-out pooled client. `pub(crate)` because the write path
    /// (`db::write_api`) needs `&mut` access to open a transaction, and the pool
    /// itself stays private. Not part of the public API.
    pub(crate) async fn pool_client(&self) -> Result<deadpool_postgres::Object, DbError> {
        self.pool.get().await.map_err(DbError::Pool)
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
                "SELECT id, name, default_harness, default_agent, default_model, \
                 capability_tags, status, created_at \
                 FROM seats ORDER BY name ASC",
                &[],
            )
            .await?;
        rows.iter().map(row_to_seat).collect()
    }

    pub async fn get_seat(&self, seat_id: &str) -> Result<Option<SeatRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, default_harness, default_agent, default_model, \
                 capability_tags, status, created_at \
                 FROM seats WHERE id = $1",
                &[&seat_id],
            )
            .await?;
        rows.first().map(row_to_seat).transpose()
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
        rows.iter().map(row_to_role_binding).collect()
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
                "SELECT id, seat_id, harness, agent, model, native_session_id, workspace_path, \
                 branch, status, launch_pack_ref, last_checkpoint_id, pid, created_at, \
                 ended_at, project_id \
                 FROM sessions ORDER BY created_at DESC",
                &[],
            )
            .await?;
        rows.iter().map(row_to_session).collect()
    }

    pub async fn list_sessions_for_seat(&self, seat_id: &str) -> Result<Vec<SessionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, seat_id, harness, agent, model, native_session_id, workspace_path, \
                 branch, status, launch_pack_ref, last_checkpoint_id, pid, created_at, \
                 ended_at, project_id \
                 FROM sessions WHERE seat_id = $1 ORDER BY created_at DESC",
                &[&seat_id],
            )
            .await?;
        rows.iter().map(row_to_session).collect()
    }

    /// Project-mode session list. AD-013 v2 backend invariant: project mode
    /// SHALL filter by project_id; Global mode SHALL group by project_id.
    pub async fn list_sessions_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<SessionRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, seat_id, harness, agent, model, native_session_id, workspace_path, \
                 branch, status, launch_pack_ref, last_checkpoint_id, pid, created_at, \
                 ended_at, project_id \
                 FROM sessions WHERE project_id = $1 ORDER BY created_at DESC",
                &[&project_id],
            )
            .await?;
        rows.iter().map(row_to_session).collect()
    }

    // =========================================================================
    // WorkItems — sort: updated_at descending
    // =========================================================================

    pub async fn list_workitems(&self) -> Result<Vec<WorkItemRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, title, goal, acceptance_criteria, owner_seat_id, \
                 status, priority, created_at, updated_at, project_id \
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
                 status, priority, created_at, updated_at, project_id \
                 FROM workitems WHERE id = $1",
                &[&workitem_id],
            )
            .await?;
        Ok(rows.first().map(row_to_workitem))
    }

    /// Project-mode workitem list. AD-013 v2 backend invariant.
    pub async fn list_workitems_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<WorkItemRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, title, goal, acceptance_criteria, owner_seat_id, \
                 status, priority, created_at, updated_at, project_id \
                 FROM workitems WHERE project_id = $1 ORDER BY updated_at DESC",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_workitem).collect())
    }

    // =========================================================================
    // Handoffs — sort: created_at descending
    // =========================================================================

    pub async fn list_handoffs(&self) -> Result<Vec<HandoffRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, from_ref, to_ref, workitem_id, purpose, expected_outcome, \
                 required_receipt, status, created_at, sent_at, project_id \
                 FROM handoffs ORDER BY created_at DESC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_handoff).collect())
    }

    /// Project-mode handoff list. AD-013 v2 backend invariant.
    pub async fn list_handoffs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<HandoffRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, from_ref, to_ref, workitem_id, purpose, expected_outcome, \
                 required_receipt, status, created_at, sent_at, project_id \
                 FROM handoffs WHERE project_id = $1 ORDER BY created_at DESC",
                &[&project_id],
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
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, project_id \
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
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, project_id \
                 FROM canonical_events WHERE event_type = $1 ORDER BY occurred_at DESC",
                &[&event_type],
            )
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    /// Project-mode event list. AD-013 v2 backend invariant.
    pub async fn list_events_for_project(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, project_id \
                 FROM canonical_events WHERE project_id = $1 \
                 ORDER BY occurred_at DESC LIMIT $2",
                &[&project_id, &limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    // =========================================================================
    // Governed dispatch (G19) read model — schema 012
    // =========================================================================

    /// List a project's dispatches, ordered for an inbox view: grouped by seat,
    /// then in_flight → ready → blocked → done, highest priority first, oldest
    /// first within a tier.
    pub async fn list_dispatches_for_project(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<DispatchRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, target_seat_id, issued_by, project_id, title, payload, status, \
                        priority, issued_at, started_at, completed_at \
                 FROM dispatches WHERE project_id = $1 \
                 ORDER BY target_seat_id, \
                   CASE status WHEN 'in_flight' THEN 0 WHEN 'ready' THEN 1 \
                               WHEN 'blocked' THEN 2 WHEN 'done' THEN 3 ELSE 4 END, \
                   priority DESC, issued_at ASC LIMIT $2",
                &[&project_id, &limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_dispatch).collect())
    }

    /// All dependency edges for a project's dispatches (joined via dispatch).
    pub async fn list_dispatch_dependencies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<DispatchDependencyRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT dd.dispatch_id, dd.dep_kind, dd.dep_value \
                 FROM dispatch_dependencies dd \
                 JOIN dispatches d ON d.id = dd.dispatch_id \
                 WHERE d.project_id = $1",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_dispatch_dep).collect())
    }

    // =========================================================================
    // Acceptance gate (014, D2) — read model + the operator's decide write
    // =========================================================================

    /// The acceptance read model for a project: every acceptance joined to its
    /// dispatch's title/seat and its Flux verdict's rendered facts. Pending
    /// first (the UI's 待验收区), then most-recently-decided.
    pub async fn list_acceptances_for_project(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<AcceptanceWithDispatchRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT a.id, a.dispatch_id, a.project_id, a.branch, a.base, a.how_to_verify, \
                       a.flux_verdict_ref, a.status, a.feedback, a.rework_dispatch_id, \
                       a.merge_commit, a.decided_by, a.decided_at, a.created_at, \
                       a.acceptance_plan, \
                       d.title, d.target_seat_id, d.priority, d.payload, \
                       CASE WHEN e.id IS NOT NULL THEN e.payload->>'verdict' END, \
                       CASE WHEN e.id IS NOT NULL THEN e.occurred_at END \
                  FROM acceptances a \
                  JOIN dispatches d ON d.id = a.dispatch_id \
                  LEFT JOIN canonical_events e ON e.id = a.flux_verdict_ref \
                 WHERE a.project_id = $1 \
                 ORDER BY (a.status = 'pending') DESC, a.decided_at DESC NULLS LAST, \
                          a.created_at DESC \
                 LIMIT $2",
                &[&project_id, &limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_acceptance_with_dispatch).collect())
    }

    /// One acceptance (same joined shape as the list).
    pub async fn get_acceptance(
        &self,
        acceptance_id: &str,
    ) -> Result<Option<AcceptanceWithDispatchRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(
                "SELECT a.id, a.dispatch_id, a.project_id, a.branch, a.base, a.how_to_verify, \
                       a.flux_verdict_ref, a.status, a.feedback, a.rework_dispatch_id, \
                       a.merge_commit, a.decided_by, a.decided_at, a.created_at, \
                       a.acceptance_plan, \
                       d.title, d.target_seat_id, d.priority, d.payload, \
                       CASE WHEN e.id IS NOT NULL THEN e.payload->>'verdict' END, \
                       CASE WHEN e.id IS NOT NULL THEN e.occurred_at END \
                  FROM acceptances a \
                  JOIN dispatches d ON d.id = a.dispatch_id \
                  LEFT JOIN canonical_events e ON e.id = a.flux_verdict_ref \
                 WHERE a.id = $1",
                &[&acceptance_id],
            )
            .await?;
        Ok(row.as_ref().map(row_to_acceptance_with_dispatch))
    }

    // =========================================================================
    // Lineage read model (acceptance-lineage milestone, Lane B) — pure reads.
    // The assembler lives in `db::lineage`; these are the row-level queries it
    // composes. No writes, no events, no second source of truth.
    // =========================================================================

    /// One dispatch by id (the lineage's "original task statement").
    pub async fn get_dispatch(&self, dispatch_id: &str) -> Result<Option<DispatchRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(
                "SELECT id, target_seat_id, issued_by, project_id, title, payload, status, \
                        priority, issued_at, started_at, completed_at \
                 FROM dispatches WHERE id = $1",
                &[&dispatch_id],
            )
            .await?;
        Ok(row.as_ref().map(row_to_dispatch))
    }

    /// One canonical event by id.
    pub async fn get_event(&self, event_id: &str) -> Result<Option<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, project_id \
                 FROM canonical_events WHERE id = $1",
                &[&event_id],
            )
            .await?;
        Ok(row.as_ref().map(row_to_event))
    }

    /// The events that name any of `refs` (`(ref_type, ref_id)` pairs, any role)
    /// or that ARE `extra_event_id`, in `occurred_at` order.
    ///
    /// This is the event chain of an object: the dispatch's lifecycle events, the
    /// acceptance events that name the acceptance, and (via `extra_event_id`) the
    /// Flux verdict the card points at even when it names no object ref of its
    /// own. `occurred_at` then `id` for a stable order when timestamps collide.
    pub async fn list_events_for_refs(
        &self,
        refs: &[(&str, &str)],
        extra_event_id: Option<&str>,
    ) -> Result<Vec<CanonicalEventRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        // Build the OR list from the pairs (refs are polymorphic TEXT with no FK,
        // so the join is the only way to walk the chain).
        let mut sql = String::from(
            "SELECT DISTINCT e.id, e.event_type, e.occurred_at, e.actor_ref, e.payload, \
                    e.created_at, e.project_id \
             FROM canonical_events e",
        );
        // Bound before `params` so the borrowed value outlives the parameter
        // vector (an empty default keeps the shape uniform when there is no
        // extra id — the branch below only pushes when there is one).
        let extra_ref: &str = extra_event_id.unwrap_or("");
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        if !refs.is_empty() {
            sql.push_str(
                " JOIN event_object_refs r ON r.event_id = e.id \
                 AND (",
            );
            for (i, (rt, rid)) in refs.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" OR ");
                }
                sql.push_str(&format!(
                    "(r.ref_type = ${} AND r.ref_id = ${})",
                    i * 2 + 1,
                    i * 2 + 2
                ));
                params.push(rt);
                params.push(rid);
            }
            sql.push(')');
        }
        let extra_index = params.len() + 1;
        if extra_event_id.is_some() {
            sql.push_str(if refs.is_empty() { " WHERE " } else { " OR " });
            sql.push_str(&format!("e.id = ${extra_index}"));
            params.push(&extra_ref);
        }
        sql.push_str(" ORDER BY e.occurred_at, e.id");
        let rows = client.query(&sql, &params).await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    /// All object refs of the given events (the lineage's product discovery).
    pub async fn list_object_refs_for_events(
        &self,
        event_ids: &[String],
    ) -> Result<Vec<ObjectRefRow>, DbError> {
        if event_ids.is_empty() {
            return Ok(Vec::new());
        }
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT event_id, ref_type, ref_id, role FROM event_object_refs \
                 WHERE event_id = ANY($1) ORDER BY event_id, ref_type, ref_id",
                &[&event_ids],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|r| ObjectRefRow {
                event_id: r.get(0),
                ref_type: r.get(1),
                ref_id: r.get(2),
                role: r.get(3),
            })
            .collect())
    }

    /// The title and path of a product referent, joined from `artifacts` or
    /// `documents`. `None` when the referent cannot be joined — the caller then
    /// carries the ref value verbatim and says so (`resolved: false`).
    pub async fn product_title_and_path(
        &self,
        ref_type: &str,
        ref_id: &str,
    ) -> Result<Option<(Option<String>, Option<String>)>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let sql = match ref_type {
            "artifact" => "SELECT title, storage_path FROM artifacts WHERE id = $1",
            "document" => "SELECT title, file_path FROM documents WHERE id = $1",
            _ => return Ok(None),
        };
        let row = client.query_opt(sql, &[&ref_id]).await?;
        Ok(row.map(|r| (r.get(0), r.get(1))))
    }

    /// What a Flux verdict reference actually SAYS, and whether it is about a
    /// given dispatch.
    ///
    /// # Why this exists
    ///
    /// `event_or_artifact_exists` answers one question — "is the referent there" —
    /// and the acceptance gate had been using it as if it answered two more:
    /// "is it green" and "is it about this dispatch". Measured on live, it does
    /// not: three pending cards named `ev-verdict-history-sessions`, a **fail**
    /// verdict about `disp-history-sessions-verify`, for three unrelated
    /// dispatches Flux had never verified at all. Existence is not a verdict.
    ///
    /// So the gate reads the verdict's value and its subject, and refuses when
    /// either is wrong. Both are things Flux actually writes — `payload->>'verdict'`
    /// and `payload->>'dispatch_id'`, or an `event_object_refs` row with
    /// `ref_type='dispatch'` — so neither criterion is invented. Measured across
    /// all 30 `ReviewVerdictIssued` events on live: 27 carry `dispatch_id` in the
    /// payload (the 3 that do not are pre-012 seed rows and a scenario fixture),
    /// and 5 name a dispatch through `event_object_refs`. Either channel counts.
    pub async fn flux_verdict_facts(
        &self,
        ref_id: &str,
        dispatch_id: &str,
    ) -> Result<VerdictFacts, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        // Existence, event type and the verdict value, in one round trip. The ref
        // is polymorphic TEXT with no FK, so an artifact referent is possible —
        // and an artifact carries no event_type and no payload, which is exactly
        // why those fields are Option rather than defaulted.
        let row = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1) AS is_event, \
                        EXISTS (SELECT 1 FROM artifacts WHERE id = $1) AS is_artifact, \
                        (SELECT event_type FROM canonical_events WHERE id = $1), \
                        (SELECT payload->>'verdict' FROM canonical_events WHERE id = $1), \
                        (SELECT payload->>'dispatch_id' FROM canonical_events WHERE id = $1)",
                &[&ref_id],
            )
            .await?;
        let is_event: bool = row.get(0);
        let is_artifact: bool = row.get(1);
        let event_type: Option<String> = row.get(2);
        let verdict: Option<String> = row.get(3);
        let payload_dispatch_id: Option<String> = row.get(4);

        // The second way a verdict can name a dispatch. Asked separately because
        // it is a different table and the two spellings are alternatives, not a
        // conjunction.
        let names_via_ref: bool = client
            .query_one(
                "SELECT EXISTS ( \
                     SELECT 1 FROM event_object_refs \
                      WHERE event_id = $1 AND ref_type = 'dispatch' AND ref_id = $2)",
                &[&ref_id, &dispatch_id],
            )
            .await?
            .get(0);

        let names_dispatch = payload_dispatch_id.as_deref() == Some(dispatch_id) || names_via_ref;
        Ok(VerdictFacts {
            exists: is_event || is_artifact,
            event_type,
            verdict,
            payload_dispatch_id,
            names_dispatch,
        })
    }

    /// One dispatch's dependency edges (012).
    pub async fn list_dispatch_dependencies(
        &self,
        dispatch_id: &str,
    ) -> Result<Vec<DispatchDependencyRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT dispatch_id, dep_kind, dep_value FROM dispatch_dependencies \
                 WHERE dispatch_id = $1",
                &[&dispatch_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_dispatch_dep).collect())
    }

    /// Does the referent of a dependency edge exist? Same existence query the
    /// scheduler uses (no FK on `dep_value`, by 012's design) — the lineage
    /// shows each edge with its live answer instead of asserting satisfaction.
    pub async fn referent_exists(&self, dep_kind: &str, dep_value: &str) -> Result<bool, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let sql = match dep_kind {
            "event" => "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)",
            "artifact" => "SELECT EXISTS (SELECT 1 FROM artifacts WHERE id = $1)",
            "dispatch" => "SELECT EXISTS (SELECT 1 FROM dispatches WHERE id = $1)",
            _ => "SELECT FALSE",
        };
        let row = client.query_one(sql, &[&dep_value]).await?;
        Ok(row.get(0))
    }

    // =========================================================================
    // Supervisor intent tool surface — the ONE gatherer.
    //
    // Both surfaces (the `plan_intent` MCP tool and the Tauri command that
    // shells out to it) plan over the output of this function. There is no
    // second gatherer and no second planner; the contract's "both surfaces over
    // the same repository functions" is enforced structurally, by this being
    // the only function that reads for the intent surface.
    // =========================================================================

    /// The seat registry, reduced to the four columns routing needs.
    ///
    /// **Why not [`SeatloomDb::list_seats`]?** Because `list_seats` hydrates the
    /// full runtime triple and validates `default_harness` against the closed
    /// `Harness` domain — and the live registry currently contains a row
    /// (`seat-vela-001`, `default_harness = 'Pi'`) that is out of that domain, so
    /// the whole read fails. Routing has no opinion about which harness a seat
    /// runs; making the intent surface fail because of an unrelated column's
    /// domain violation would be a defect, not strictness. The domain violation
    /// itself is reported to Aegis as data debt, not patched here.
    pub async fn list_seat_routing_facts(&self) -> Result<Vec<SeatRoutingRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, name, capability_tags, status FROM seats ORDER BY name ASC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_seat_routing).collect())
    }

    /// Gather everything an intent plan is allowed to know about one project:
    /// the seat registry joined to its role bindings, the project's real budget
    /// columns, and this project's dispatch load.
    ///
    /// Three deliberate bounds:
    ///
    /// 1. **No event/artifact scan.** The caller resolves exactly the referent
    ///    ids the intent names ([`crate::intent::facts::named_referents`]) and
    ///    writes the answers into `named_referent_exists`.
    /// 2. **A dispatch limit, stated.** Load counting needs the in-flight
    ///    dispatches of one project, not history; the ceiling is a parameter so a
    ///    caller can never silently read an unbounded table.
    /// 3. **Absent reads stay absent.** A project with no budget row yields
    ///    `None`, never a defaulted number.
    pub async fn gather_intent_facts(
        &self,
        project_id: &str,
        dispatch_limit: i64,
    ) -> Result<crate::intent::IntentFacts, DbError> {
        let seats = self.list_seat_routing_facts().await?;
        let bindings = self.list_role_bindings_for_project(project_id).await?;
        let project = self.get_project(project_id).await?;
        let dispatches = self
            .list_dispatches_for_project(project_id, dispatch_limit)
            .await?;

        let mut seat_facts: Vec<crate::intent::SeatFact> = seats
            .iter()
            .map(|seat| {
                let binding = bindings.iter().find(|b| b.seat_id == seat.id);
                crate::intent::SeatFact {
                    seat_id: seat.id.clone(),
                    name: seat.name.clone(),
                    capability_tags: seat.capability_tags.clone(),
                    status: seat.status.clone(),
                    role: binding.map(|b| b.role.clone()),
                    authority_docs: binding
                        .map(|b| b.authority_doc_refs.clone())
                        .unwrap_or_default(),
                    constraints: binding.map(|b| b.constraints.clone()).unwrap_or_default(),
                    active_delegation_id: binding.and_then(|b| b.active_delegation_id.clone()),
                }
            })
            .collect();
        // Name order is the planner's deterministic ordering for candidate lists.
        seat_facts.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.seat_id.cmp(&b.seat_id)));

        let mut in_flight_by_seat: BTreeMap<String, usize> = BTreeMap::new();
        let mut dispatch_ids: BTreeSet<String> = BTreeSet::new();
        let mut dispatch_titles: BTreeSet<String> = BTreeSet::new();
        for dispatch in &dispatches {
            dispatch_ids.insert(dispatch.id.clone());
            dispatch_titles.insert(dispatch.title.clone());
            if dispatch.status == "in_flight" {
                *in_flight_by_seat
                    .entry(dispatch.target_seat_id.clone())
                    .or_insert(0) += 1;
            }
        }

        Ok(crate::intent::IntentFacts {
            project_id: project_id.to_string(),
            worker_budget_tokens: project.as_ref().map(|p| p.worker_budget_tokens),
            supervisor_budget_tokens: project.as_ref().map(|p| p.supervisor_budget_tokens),
            seats: seat_facts,
            in_flight_by_seat,
            dispatch_ids,
            dispatch_titles,
            named_referent_exists: BTreeMap::new(),
        })
    }

    // =========================================================================
    // Historical session corpus (011 + 021) — the read path.
    //
    // ONE implementation, exposed twice over the same code: the `list_history_*`
    // / `get_history_session_turns` MCP tools and their Tauri thin wrappers both
    // come through here (the turns half adds `crate::history::read`, which is
    // the file half). There is no second reader.
    //
    // Every query is written so that a database WITHOUT 021 answers honestly
    // instead of lying: the corpus columns are probed with
    // `to_regclass`/`information_schema` and a missing one is reported as
    // `CorpusNotLoaded`, never read as zero.
    // =========================================================================

    /// Does `archived_sessions.message_count` exist? `021` adds it.
    pub async fn corpus_columns_present(&self) -> Result<bool, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
                  WHERE table_name = 'archived_sessions' AND column_name = 'message_count')",
                &[],
            )
            .await?;
        Ok(row.get(0))
    }

    /// (a) Every project that has at least one corpus session, with its
    /// session count and its total message count.
    ///
    /// A project with sessions but no corpus migration reads `message_count =
    /// None` — the count is unknown, which is not the same as zero.
    ///
    /// THE INVENTORY FILTER (021's own header states the rule; Flux's verdict
    /// `FLUX-2026-10-14-history-sessions-verdict-v1` §5 measured the cost of
    /// omitting it): the live table also holds the 2026-10-08 manifest load —
    /// 271 rows, 89 of them with no extracted conversation and therefore no
    /// `message_count` — and 78 of those 89 carry a project slug that matches
    /// one of the 38 project ids. Joining them in made `sum(sessionCount)` 261
    /// against a corpus of 183 and reported 58 sessions under `sf-code/storage`
    /// for a truth of 2. So the join filters on `message_count IS NOT NULL`
    /// and a project with only inventory rows reports no sessions at all.
    pub async fn list_history_projects(&self) -> Result<Vec<HistoryProjectRow>, DbError> {
        let corpus = self.corpus_columns_present().await?;
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let message_expr = if corpus {
            "sum(a.message_count)::bigint"
        } else {
            "NULL::bigint"
        };
        let join = if corpus {
            // The corpus rows, never the 89 inventory-only ones.
            "LEFT JOIN archived_sessions a ON a.project = p.id AND a.message_count IS NOT NULL"
        } else {
            "LEFT JOIN archived_sessions a ON a.project = p.id"
        };
        let sql = format!(
            "SELECT p.id, p.name, p.home_path, count(a.id)::bigint AS sessions, {message_expr} \
               FROM projects p \
               {join} \
              GROUP BY p.id, p.name, p.home_path \
             HAVING count(a.id) > 0 \
              ORDER BY count(a.id) DESC, p.id ASC"
        );
        let rows = client.query(&sql, &[]).await?;
        Ok(rows
            .iter()
            .map(|r| HistoryProjectRow {
                id: r.get(0),
                name: r.get(1),
                home_path: r.get(2),
                session_count: r.get(3),
                message_count: r.get(4),
            })
            .collect())
    }

    /// (b) One project's CORPUS sessions, dated first (newest first) and the
    /// undated ones last and flagged — never silently interleaved, because
    /// sorting 14 undated cursor sessions as if they were the oldest would be
    /// a lie.
    ///
    /// The inventory-only rows are excluded for the same reason the projects
    /// read filters them (see `list_history_projects`): they are manifest
    /// entries, not sessions, and 78 of the 89 carry a slug matching one of the
    /// 38 project ids. Flux measured the consequence on `seatloom`: 16 rows
    /// where the corpus defines 14, one of them a gemini marker row with
    /// `firstTs: null` that the tool nevertheless flagged `dated: true`.
    pub async fn list_history_sessions(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArchivedSessionRow>, DbError> {
        let corpus = self.corpus_columns_present().await?;
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let corpus_cols = if corpus {
            "a.message_count, a.first_ts::text, a.last_ts::text"
        } else {
            "NULL::integer, NULL::text, NULL::text"
        };
        let corpus_filter = if corpus {
            "AND a.message_count IS NOT NULL"
        } else {
            ""
        };
        let sql = format!(
            "SELECT a.id, a.harness, a.project, a.cwd, a.session_date, a.bytes, a.lines, \
                    a.sha256, a.raw_path, a.src_path, {corpus_cols} \
               FROM archived_sessions a \
              WHERE a.project = $1 {corpus_filter} \
              ORDER BY (a.first_ts IS NULL) ASC, \
                       a.first_ts DESC NULLS LAST, \
                       a.id ASC"
        );
        let rows = client.query(&sql, &[&project_id]).await?;
        Ok(rows.iter().map(row_to_archived_session).collect())
    }

    /// The single row one session's turns are read for.
    pub async fn get_archived_session(
        &self,
        harness: &str,
        id: &str,
    ) -> Result<Option<ArchivedSessionRow>, DbError> {
        let corpus = self.corpus_columns_present().await?;
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let corpus_cols = if corpus {
            "a.message_count, a.first_ts::text, a.last_ts::text"
        } else {
            "NULL::integer, NULL::text, NULL::text"
        };
        let sql = format!(
            "SELECT a.id, a.harness, a.project, a.cwd, a.session_date, a.bytes, a.lines, \
                    a.sha256, a.raw_path, a.src_path, {corpus_cols} \
               FROM archived_sessions a \
              WHERE a.harness = $1 AND a.id = $2"
        );
        let row = client.query_opt(&sql, &[&harness, &id]).await?;
        Ok(row.as_ref().map(row_to_archived_session))
    }

    // =========================================================================
    // Knowledge layer (D5, 015) — memories: reads + the three write paths.
    // Every write appends its event in the SAME transaction (the 012 dispatch
    // pattern, `append_event_in_tx`) — there is no second write path, and no
    // seat writes a `memories` row by any other means.
    // =========================================================================

    /// One memory by id.
    pub async fn get_memory(&self, memory_id: &str) -> Result<Option<MemoryRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        Ok(row.as_ref().map(row_to_memory))
    }

    /// The recall read: a project+topic's ACTIVE memories, optionally filtered by
    /// a text query over `title` + `l0_summary` (002's FTS pattern — plain
    /// `to_tsvector` matching, no embeddings), ordered for an inbox: heat DESC,
    /// then most recently updated. Scope is mandatory — there is no
    /// "all projects" recall.
    pub async fn list_memories_for_recall(
        &self,
        project_id: &str,
        topic_id: &str,
        query: Option<&str>,
        include_forgotten: bool,
        include_superseded: bool,
        limit: i64,
    ) -> Result<Vec<MemoryRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT id, project_id, topic_id, title, tags, l0_summary, l1_overview, \
                        l2_ref_kind, l2_document_id, l2_file_path, l2_body_digest, \
                        source_kind, source_ref, kind, confidence, supersedes, \
                        l2_line_start, l2_line_end, heat, recall_count, last_recalled_at, \
                        tier, tier_changed_at, status, forgotten_reason, \
                        resolved_by, resolved_at, created_at, updated_at \
                   FROM memories \
                  WHERE project_id = $1 AND topic_id = $2 \
                    AND ($3::bool OR status = 'active') \
                    AND ($5::bool OR NOT EXISTS ( \
                          SELECT 1 FROM memory_relations r \
                           WHERE r.to_id = memories.id AND r.rel = 'supersedes')) \
                    AND ($4::text IS NULL OR to_tsvector('simple', title || ' ' || l0_summary) \
                         @@ plainto_tsquery('simple', $4)) \
                  ORDER BY heat DESC, updated_at DESC \
                  LIMIT $6",
                &[
                    &project_id,
                    &topic_id,
                    &include_forgotten,
                    &query,
                    &include_superseded,
                    &limit,
                ],
            )
            .await?;
        Ok(rows.iter().map(row_to_memory).collect())
    }

    /// Who supersedes this memory (the reverse lookup 018's
    /// `idx_memory_relations_to` serves). `None` when nothing does.
    pub async fn superseded_by(&self, memory_id: &str) -> Result<Option<String>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(
                "SELECT from_id FROM memory_relations \
                  WHERE to_id = $1 AND rel = 'supersedes' ORDER BY from_id LIMIT 1",
                &[&memory_id],
            )
            .await?;
        Ok(row.map(|r| r.get(0)))
    }

    /// This memory's outgoing edges (018).
    pub async fn list_memory_relations(
        &self,
        memory_id: &str,
    ) -> Result<Vec<MemoryRelationRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT from_id, to_id, rel FROM memory_relations \
                  WHERE from_id = $1 ORDER BY rel, to_id",
                &[&memory_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_memory_relation).collect())
    }

    /// A topic and its ancestor chain (001/010 `topics.parent_topic_id`), nearest
    /// first. Recalled knowledge is returned WITH this chain so a memory is
    /// never read out of context — "断章取义" is the failure mode the contract
    /// names. The chain is bounded by the tree's depth; a cycle cannot exist
    /// because 010's self-FK plus the not-self CHECK keep it a tree.
    pub async fn topic_ancestor_chain(&self, topic_id: &str) -> Result<Vec<TopicRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "WITH RECURSIVE chain AS ( \
                   SELECT id, project_id, title, status, parent_topic_id, 0 AS depth \
                     FROM topics WHERE id = $1 \
                   UNION ALL \
                   SELECT t.id, t.project_id, t.title, t.status, t.parent_topic_id, c.depth + 1 \
                     FROM topics t JOIN chain c ON t.id = c.parent_topic_id \
                    WHERE c.depth < 32 \
                 ) \
                 SELECT id, project_id, title, status, parent_topic_id FROM chain \
                  ORDER BY depth",
                &[&topic_id],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|r| TopicRow {
                id: r.get(0),
                project_id: r.get(1),
                title: r.get(2),
                status: r.get(3),
                parent_topic_id: r.get(4),
            })
            .collect())
    }

    /// `remember`: insert the row + append `MemoryCreated` in ONE transaction.
    /// The caller (the tool) has already validated the shape; this is the write
    /// path, and the only one.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_memory(
        &self,
        memory_id: &str,
        project_id: &str,
        topic_id: &str,
        title: &str,
        tags: &[String],
        l0_summary: &str,
        l1_overview: Option<&str>,
        l2_ref_kind: &str,
        l2_document_id: Option<&str>,
        l2_file_path: Option<&str>,
        l2_body_digest: Option<&str>,
        source_kind: &str,
        source_ref: &str,
        kind: &str,
        confidence: &str,
        supersedes: Option<&str>,
        l2_line_start: Option<i32>,
        l2_line_end: Option<i32>,
        relations: &[(String, String)], // (to_id, rel) pairs
        actor_ref: &str,
    ) -> Result<(), DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        tx.execute(
            "INSERT INTO memories \
               (id, project_id, topic_id, title, tags, l0_summary, l1_overview, \
                l2_ref_kind, l2_document_id, l2_file_path, l2_body_digest, \
                source_kind, source_ref, kind, confidence, supersedes, \
                l2_line_start, l2_line_end) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
            &[
                &memory_id,
                &project_id,
                &topic_id,
                &title,
                &tags,
                &l0_summary,
                &l1_overview,
                &l2_ref_kind,
                &l2_document_id,
                &l2_file_path,
                &l2_body_digest,
                &source_kind,
                &source_ref,
                &kind,
                &confidence,
                &supersedes,
                &l2_line_start,
                &l2_line_end,
            ],
        )
        .await?;
        // The supersedes edge has TWO spellings — the column above and a
        // memory_relations row — written in the SAME transaction so they can
        // never disagree (contract §G2).
        if let Some(prev) = supersedes {
            tx.execute(
                "INSERT INTO memory_relations (from_id, to_id, rel) VALUES ($1,$2,'supersedes')",
                &[&memory_id, &prev],
            )
            .await?;
        }
        for (to_id, rel) in relations {
            tx.execute(
                "INSERT INTO memory_relations (from_id, to_id, rel) VALUES ($1,$2,$3)",
                &[&memory_id, &to_id, &rel],
            )
            .await?;
        }
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::MemoryCreated,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: actor_ref.to_string(),
            project_id: project_id.to_string(),
            turn_seq: None,
            payload: serde_json::json!({
                "memory_id": memory_id,
                "topic_id": topic_id,
                "title": title,
                "source_kind": source_kind,
                "source_ref": source_ref,
                "kind": kind,
                "confidence": confidence,
                "supersedes": supersedes,
                "relations": relations.iter().map(|(to, rel)| serde_json::json!({"to_id": to, "rel": rel})).collect::<Vec<_>>(),
                "l2_line_start": l2_line_start,
                "l2_line_end": l2_line_end,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Memory, memory_id),
                ObjectRefDraft::new(RefType::Seat, actor_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Topic, topic_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        tx.commit().await?;
        Ok(())
    }

    /// The recall audit: `heat+1, recall_count+1, last_recalled_at=now` on the
    /// row AND exactly one `MemoryRecalled` event, in ONE transaction (contract
    /// §9: every recall is recorded — the ledger grows from "what happened" to
    /// "what is known"). Returns the row as landed.
    pub async fn record_recall(
        &self,
        memory_id: &str,
        actor_ref: &str,
    ) -> Result<MemoryRow, DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        // Guarded: only an ACTIVE memory is recalled. A forgotten one is a
        // tombstone — recalling it would resurrect it through the back door.
        let row = tx
            .query_opt(
                &format!("{MEMORY_COLUMNS} WHERE id = $1 AND status = 'active'"),
                &[&memory_id],
            )
            .await?;
        let Some(row) = row else {
            return Err(DbError::MemoryNotActive {
                id: memory_id.to_string(),
            });
        };
        let project_id: String = row.get(1);
        let topic_id: String = row.get(2);
        let heat_before: i32 = row.get(18);
        tx.execute(
            "UPDATE memories \
               SET heat = heat + 1, recall_count = recall_count + 1, \
                   last_recalled_at = NOW(), updated_at = NOW() \
              WHERE id = $1 AND status = 'active'",
            &[&memory_id],
        )
        .await?;
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::MemoryRecalled,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: actor_ref.to_string(),
            project_id,
            turn_seq: None,
            payload: serde_json::json!({
                "memory_id": memory_id,
                "topic_id": topic_id,
                "heat_delta": 1,
                "heat": heat_before + 1,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Memory, memory_id),
                ObjectRefDraft::new(RefType::Seat, actor_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Topic, &topic_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        let updated = tx
            .query_one(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let updated = row_to_memory(&updated);
        tx.commit().await?;
        Ok(updated)
    }

    // =========================================================================
    // Knowledge read model (overnight step 4) — the topic tree and the
    // memories-by-topic card list, with progressive disclosure.
    //
    // PURE READS: these write nothing and append no event. They are interface
    // queries for the UI, not a recall — `recall` keeps its `MemoryRecalled`
    // audit; a panel rendering a tree must not mutate what it renders.
    // =========================================================================

    /// The knowledge topic tree for a project: every topic with its parent and
    /// its memory count. `memoryCount` counts ACTIVE and NOT-superseded
    /// memories — a stale decision served as current is worse than no memory, so
    /// the count a panel shows is the count a recall would return.
    pub async fn knowledge_topic_tree(
        &self,
        project_id: &str,
    ) -> Result<Vec<KnowledgeTopicNode>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT t.id, t.title, t.status, t.parent_topic_id, \
                        (SELECT count(*) FROM memories m \
                          WHERE m.topic_id = t.id AND m.status = 'active' \
                            AND NOT EXISTS (SELECT 1 FROM memory_relations r \
                                             WHERE r.to_id = m.id AND r.rel = 'supersedes')) \
                   FROM topics t \
                  WHERE t.project_id = $1 \
                  ORDER BY t.parent_topic_id NULLS FIRST, t.id",
                &[&project_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_knowledge_topic_node).collect())
    }

    /// The topic's ancestor chain in the KNOWLEDGE READ MODEL's order: ancestors
    /// first, the topic itself last (the context, then what it is the context
    /// OF). `recall` keeps its own nearest-first order.
    pub async fn topic_chain_ancestors_first(
        &self,
        topic_id: &str,
    ) -> Result<Vec<TopicRow>, DbError> {
        let mut chain = self.topic_ancestor_chain(topic_id).await?;
        chain.reverse();
        Ok(chain)
    }

    /// The memories of one topic, at the requested disclosure depth, WITH the
    /// topic's ancestor chain (the same chain discipline `recall` follows: a
    /// memory is never returned out of its context).
    ///
    /// `depth` is `l0` (the ~100-token summary, the default), `l1` (the ~2K
    /// overview) or `l2` (the evidence pointer, with the line range when the
    /// source is a transcript slice). The token budget belongs to the caller.
    /// Superseded rows are excluded unless `include_superseded` is set, and then
    /// each carries `superseded_by`.
    pub async fn topic_memories(
        &self,
        topic_id: &str,
        depth: MemoryLayer,
        include_superseded: bool,
        limit: i64,
    ) -> Result<Vec<KnowledgeMemoryCard>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT m.id, m.topic_id, m.title, m.kind, m.confidence, m.tier, m.status, \
                        m.l0_summary, m.l1_overview, m.l2_ref_kind, m.l2_document_id, \
                        m.l2_file_path, m.l2_body_digest, m.l2_line_start, m.l2_line_end, \
                        m.source_kind, m.source_ref, \
                        (SELECT r.from_id FROM memory_relations r \
                          WHERE r.to_id = m.id AND r.rel = 'supersedes' \
                          ORDER BY r.from_id LIMIT 1) AS superseded_by \
                   FROM memories m \
                  WHERE m.topic_id = $1 \
                    AND m.project_id = (SELECT project_id FROM topics WHERE id = $1) \
                    AND ($2::bool OR NOT EXISTS (SELECT 1 FROM memory_relations r \
                                                  WHERE r.to_id = m.id AND r.rel = 'supersedes')) \
                  ORDER BY m.heat DESC, m.updated_at DESC \
                  LIMIT $3",
                &[&topic_id, &include_superseded, &limit],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|r| row_to_knowledge_memory_card(r, depth))
            .collect())
    }

    // =========================================================================
    // Topic creation (Aegis ruling 2026-10-11) — the governed opening of a
    // recall scope. `remember` cannot create one implicitly: a typo would
    // silently open a scope nobody ever recalls from.
    // =========================================================================

    /// One topic by id.
    pub async fn get_topic(&self, topic_id: &str) -> Result<Option<TopicRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_opt(
                "SELECT id, project_id, title, status, parent_topic_id \
                  FROM topics WHERE id = $1",
                &[&topic_id],
            )
            .await?;
        Ok(row.as_ref().map(row_to_topic))
    }

    /// The ancestor chain of a topic, INCLUDING itself, nearest first. Used by
    /// the cycle check (a new topic's parent must not be its own descendant).
    pub async fn topic_chain_inclusive(&self, topic_id: &str) -> Result<Vec<TopicRow>, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "WITH RECURSIVE chain AS ( \
                   SELECT id, project_id, title, status, parent_topic_id, 0 AS depth \
                     FROM topics WHERE id = $1 \
                   UNION ALL \
                   SELECT t.id, t.project_id, t.title, t.status, t.parent_topic_id, c.depth + 1 \
                     FROM topics t JOIN chain c ON t.id = c.parent_topic_id \
                    WHERE c.depth < 64 \
                 ) \
                 SELECT id, project_id, title, status, parent_topic_id FROM chain \
                  ORDER BY depth",
                &[&topic_id],
            )
            .await?;
        Ok(rows.iter().map(row_to_topic).collect())
    }

    /// Open a topic: insert the row + append `TopicOpened` in ONE transaction
    /// (the remember discipline). Idempotent by id — a second open of an
    /// existing topic writes NOTHING and emits NO event (an existing row is not
    /// a transition), returning the row as it stands.
    ///
    /// Returns `(row, created)`; `created` is false when the topic already
    /// existed.
    pub async fn open_topic(
        &self,
        topic_id: &str,
        project_id: &str,
        title: &str,
        parent_topic_id: Option<&str>,
        opened_by: &str,
    ) -> Result<(TopicRow, bool), DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        let n = tx
            .execute(
                "INSERT INTO topics (id, project_id, title, parent_topic_id, opened_by) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (id) DO NOTHING",
                &[&topic_id, &project_id, &title, &parent_topic_id, &opened_by],
            )
            .await?;
        if n == 0 {
            // Already open: return the existing row, write nothing, emit nothing.
            let row = tx
                .query_one(
                    "SELECT id, project_id, title, status, parent_topic_id \
                       FROM topics WHERE id = $1",
                    &[&topic_id],
                )
                .await?;
            let row = row_to_topic(&row);
            tx.commit().await?;
            return Ok((row, false));
        }
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::TopicOpened,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: opened_by.to_string(),
            project_id: project_id.to_string(),
            turn_seq: None,
            payload: serde_json::json!({
                "topic_id": topic_id,
                "title": title,
                "parent_topic_id": parent_topic_id,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Topic, topic_id),
                ObjectRefDraft::new(RefType::Seat, opened_by, RefRole::Actor),
                ObjectRefDraft::new(RefType::Project, project_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        let row = tx
            .query_one(
                "SELECT id, project_id, title, status, parent_topic_id \
                   FROM topics WHERE id = $1",
                &[&topic_id],
            )
            .await?;
        let row = row_to_topic(&row);
        tx.commit().await?;
        Ok((row, true))
    }

    /// `resolve` — the G3 closure path (018): `active -> resolved` +
    /// `MemoryResolved` in ONE transaction. The ONLY closure path: there is no
    /// background sweep and no automatic anything (contract §G3).
    ///
    /// `resolved_by` comes from the caller (the seat identity the tool resolved),
    /// `resolved_at` is now. A row that is already `resolved` or `forgotten` is
    /// refused — a tombstone and a closed record are both immutable facts.
    pub async fn resolve_memory(
        &self,
        memory_id: &str,
        actor_ref: &str,
    ) -> Result<MemoryRow, DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let Some(row) = row else {
            return Err(DbError::MemoryNotFound {
                id: memory_id.to_string(),
            });
        };
        let status: String = row.get(23);
        if status != "active" {
            return Err(DbError::MemoryNotActive {
                id: memory_id.to_string(),
            });
        }
        let project_id: String = row.get(1);
        let topic_id: String = row.get(2);
        let kind: String = row.get(13);
        tx.execute(
            "UPDATE memories \
               SET status = 'resolved', resolved_by = $2, resolved_at = NOW(), \
                   updated_at = NOW() \
              WHERE id = $1 AND status = 'active'",
            &[&memory_id, &actor_ref],
        )
        .await?;
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::MemoryResolved,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: actor_ref.to_string(),
            project_id,
            turn_seq: None,
            payload: serde_json::json!({
                "memory_id": memory_id,
                "topic_id": topic_id,
                "kind": kind,
                "resolved_by": actor_ref,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Memory, memory_id),
                ObjectRefDraft::new(RefType::Seat, actor_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Topic, &topic_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        let updated = tx
            .query_one(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let updated = row_to_memory(&updated);
        tx.commit().await?;
        Ok(updated)
    }

    /// `promote`: the ONLY tier channel — an explicit target tier, up OR down.
    /// Same tier = no transition = refused (nothing is written, no event).
    pub async fn set_memory_tier(
        &self,
        memory_id: &str,
        to_tier: &str,
        actor_ref: &str,
    ) -> Result<MemoryRow, DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let Some(row) = row else {
            return Err(DbError::MemoryNotFound {
                id: memory_id.to_string(),
            });
        };
        let project_id: String = row.get(1);
        let topic_id: String = row.get(2);
        let from_tier: String = row.get(21);
        if from_tier == to_tier {
            return Err(DbError::MemoryNoTierChange {
                id: memory_id.to_string(),
                tier: from_tier,
            });
        }
        tx.execute(
            "UPDATE memories SET tier = $2, tier_changed_at = NOW(), updated_at = NOW() \
              WHERE id = $1",
            &[&memory_id, &to_tier],
        )
        .await?;
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::MemoryPromoted,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: actor_ref.to_string(),
            project_id,
            turn_seq: None,
            payload: serde_json::json!({
                "memory_id": memory_id,
                "topic_id": topic_id,
                "from_tier": from_tier,
                "to_tier": to_tier,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Memory, memory_id),
                ObjectRefDraft::new(RefType::Seat, actor_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Topic, &topic_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        let updated = tx
            .query_one(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let updated = row_to_memory(&updated);
        tx.commit().await?;
        Ok(updated)
    }

    /// `forget`: the soft tombstone. `status='forgotten'` + the reason + one
    /// `MemoryForgotten` event, in ONE transaction. The row is NEVER deleted
    /// (ledger discipline), and a second forget of a tombstone is refused.
    pub async fn forget_memory(
        &self,
        memory_id: &str,
        reason: &str,
        actor_ref: &str,
    ) -> Result<MemoryRow, DbError> {
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let Some(row) = row else {
            return Err(DbError::MemoryNotFound {
                id: memory_id.to_string(),
            });
        };
        let status: String = row.get(23);
        if status != "active" {
            return Err(DbError::MemoryNotActive {
                id: memory_id.to_string(),
            });
        }
        let project_id: String = row.get(1);
        let topic_id: String = row.get(2);
        tx.execute(
            "UPDATE memories SET status = 'forgotten', forgotten_reason = $2, \
               updated_at = NOW() WHERE id = $1 AND status = 'active'",
            &[&memory_id, &reason],
        )
        .await?;
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type: EventType::MemoryForgotten,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: actor_ref.to_string(),
            project_id,
            turn_seq: None,
            payload: serde_json::json!({
                "memory_id": memory_id,
                "topic_id": topic_id,
                "reason": reason,
            }),
            refs: vec![
                ObjectRefDraft::subject(RefType::Memory, memory_id),
                ObjectRefDraft::new(RefType::Seat, actor_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Topic, &topic_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        let updated = tx
            .query_one(&format!("{MEMORY_COLUMNS} WHERE id = $1"), &[&memory_id])
            .await?;
        let updated = row_to_memory(&updated);
        tx.commit().await?;
        Ok(updated)
    }

    // =========================================================================
    // Read tools' back ends (D5 Lane B §3.5/§3.6) — pure reads, so no seat has
    // to reach for psql. They expose NO write capability.
    // =========================================================================

    /// The dispatch read model: the row, its LIVE status (recomputed from its
    /// dependencies' existence, exactly like the pull decision), and its
    /// dependency edges with each referent's existence.
    pub async fn dispatch_read_model(
        &self,
        dispatch_id: &str,
    ) -> Result<Option<DispatchReadModel>, DbError> {
        let Some(row) = self.get_dispatch(dispatch_id).await? else {
            return Ok(None);
        };
        let deps = self.list_dispatch_dependencies(dispatch_id).await?;
        let mut edges = Vec::with_capacity(deps.len());
        let mut all_satisfied = true;
        for d in deps {
            let exists = self.referent_exists(&d.dep_kind, &d.dep_value).await?;
            all_satisfied &= exists;
            edges.push(DispatchDependencyStatus {
                dep_kind: d.dep_kind,
                dep_value: d.dep_value,
                exists,
            });
        }
        // `live_status` mirrors the scheduler: terminal and in_flight rows are
        // returned as stored (never recomputed), and a non-terminal row is ready
        // iff every dependency exists, else blocked.
        let live_status = match row.status.as_str() {
            "done" | "failed" | "cancelled" | "in_flight" => row.status.clone(),
            _ => {
                if all_satisfied {
                    "ready".to_string()
                } else {
                    "blocked".to_string()
                }
            }
        };
        Ok(Some(DispatchReadModel {
            dispatch: row,
            live_status,
            dependencies: edges,
        }))
    }

    /// The ledger query: `canonical_events` filtered by project, event types, a
    /// ref (`ref_type` + `ref_id`), and an `occurred_at` window, newest first,
    /// each with its object refs. Pure read.
    #[allow(clippy::too_many_arguments)]
    pub async fn query_ledger(
        &self,
        project_id: Option<&str>,
        event_types: &[String],
        ref_type: Option<&str>,
        ref_id: Option<&str>,
        from: Option<chrono::DateTime<Utc>>,
        to: Option<chrono::DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LedgerEventWithRefs>, DbError> {
        // An empty type list means "no type filter", not "match nothing": the
        // parameter is NULL when the caller named no types, because
        // `event_type = ANY('{}')` would exclude every row.
        let types: Option<Vec<String>> = if event_types.is_empty() {
            None
        } else {
            Some(event_types.to_vec())
        };
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let rows = client
            .query(
                "SELECT DISTINCT e.id, e.event_type, e.occurred_at, e.actor_ref, e.payload, \
                        e.created_at, e.project_id \
                   FROM canonical_events e \
                   LEFT JOIN event_object_refs r ON r.event_id = e.id \
                  WHERE ($1::text IS NULL OR e.project_id = $1) \
                    AND ($2::text[] IS NULL OR e.event_type = ANY($2)) \
                    AND ($3::text IS NULL OR (r.ref_type = $3 AND r.ref_id = $4)) \
                    AND ($5::timestamptz IS NULL OR e.occurred_at >= $5) \
                    AND ($6::timestamptz IS NULL OR e.occurred_at <= $6) \
                  ORDER BY e.occurred_at DESC, e.id DESC \
                  LIMIT $7 OFFSET $8",
                &[
                    &project_id,
                    &types,
                    &ref_type,
                    &ref_id,
                    &from,
                    &to,
                    &limit,
                    &offset,
                ],
            )
            .await?;
        // An empty type list means "no type filter", not "match nothing": pass
        // NULL rather than an empty array, or `= ANY('{}')` excludes every row.
        let events: Vec<CanonicalEventRow> = rows.iter().map(row_to_event).collect();
        let refs = self
            .list_object_refs_for_events(&events.iter().map(|e| e.id.clone()).collect::<Vec<_>>())
            .await?;
        Ok(events
            .into_iter()
            .map(|e| {
                let event_refs: Vec<ObjectRefRow> = refs
                    .iter()
                    .filter(|r| r.event_id == e.id)
                    .cloned()
                    .collect();
                LedgerEventWithRefs {
                    event: e,
                    refs: event_refs,
                }
            })
            .collect())
    }

    /// Does the referent of a Flux verdict reference exist? The ref is
    /// polymorphic TEXT with no FK (the 010/012 reasoning — `ReviewVerdictIssued`
    /// event OR artifact id), so existence in either table is the check.
    pub async fn event_or_artifact_exists(&self, ref_id: &str) -> Result<bool, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let row = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1) \
                    OR EXISTS (SELECT 1 FROM artifacts WHERE id = $1)",
                &[&ref_id],
            )
            .await?;
        Ok(row.get(0))
    }

    /// The operator's decision, landed atomically with its ledger event
    /// (contract rule 2): a GUARDED `pending -> accepted|rejected` transition
    /// plus the matching `AcceptanceGranted` / `AcceptanceRejected` event in ONE
    /// transaction. The guard (`WHERE id = $1 AND status = 'pending'`) is what
    /// makes accepted/rejected terminal in the write path — a second decision of
    /// the same acceptance updates 0 rows and errors rather than silently
    /// rewriting a recorded fact.
    ///
    /// Returns the decided row as landed, so the caller's response is populated
    /// from what the database actually holds (status, merge_commit,
    /// rework_dispatch_id) rather than from the caller's intentions.
    pub async fn decide_acceptance(
        &self,
        acceptance_id: &str,
        status: &str, // 'accepted' | 'rejected'
        feedback: Option<&str>,
        merge_commit: Option<&str>,
        rework_dispatch_id: Option<&str>,
        operator_ref: &str,
    ) -> Result<AcceptanceRow, DbError> {
        let event_type = match status {
            "accepted" => EventType::AcceptanceGranted,
            "rejected" => EventType::AcceptanceRejected,
            other => {
                return Err(DbError::AcceptanceNotPending {
                    id: format!("{acceptance_id} (bad target status {other:?})"),
                })
            }
        };
        let mut client = self.pool.get().await.map_err(DbError::Pool)?;
        let tx = client.transaction().await?;

        // The guarded fetch proves pending-ness inside the tx (and supplies the
        // event's project/dispatch refs and the granted payload's branch/base).
        let row = tx
            .query_opt(
                "SELECT project_id, dispatch_id, branch, base FROM acceptances \
                  WHERE id = $1 AND status = 'pending'",
                &[&acceptance_id],
            )
            .await?;
        let Some(row) = row else {
            return Err(DbError::AcceptanceNotPending {
                id: acceptance_id.to_string(),
            });
        };
        let project_id: String = row.get(0);
        let dispatch_id: String = row.get(1);
        let branch: String = row.get(2);
        let base: String = row.get(3);

        let n = tx
            .execute(
                "UPDATE acceptances \
                   SET status = $2, feedback = $3, merge_commit = $4, \
                       rework_dispatch_id = $5, decided_by = $6, decided_at = NOW() \
                  WHERE id = $1 AND status = 'pending'",
                &[
                    &acceptance_id,
                    &status,
                    &feedback,
                    &merge_commit,
                    &rework_dispatch_id,
                    &operator_ref,
                ],
            )
            .await?;
        if n != 1 {
            return Err(DbError::AcceptanceNotPending {
                id: acceptance_id.to_string(),
            });
        }

        // Same transaction: the decision event (the row is a projection). Actor
        // is the operator (a configured actor_ref; refs stay polymorphic TEXT,
        // no FK — Lyra 014 §3). Payload shapes are the contract's: granted
        // carries {branch, base, merge_commit}, rejected {feedback,
        // rework_dispatch_id}.
        let payload = match status {
            "accepted" => serde_json::json!({
                "branch": branch,
                "base": base,
                "merge_commit": merge_commit,
            }),
            _ => serde_json::json!({
                "feedback": feedback,
                "rework_dispatch_id": rework_dispatch_id,
            }),
        };
        let draft = EventDraft {
            event_id: format!("ev-{}", nanoid::nanoid!()),
            event_type,
            schema_version: 1,
            occurred_at: Utc::now(),
            actor_ref: operator_ref.to_string(),
            project_id,
            turn_seq: None,
            payload,
            refs: vec![
                ObjectRefDraft::subject(RefType::Acceptance, acceptance_id),
                ObjectRefDraft::new(RefType::Seat, operator_ref, RefRole::Actor),
                ObjectRefDraft::new(RefType::Dispatch, dispatch_id, RefRole::Context),
            ],
        };
        append_event_in_tx(&tx, &draft).await?;
        // Read the decided row back INSIDE the transaction: the response the UI
        // renders is populated from the landed state, not from intentions.
        let decided = tx
            .query_one(
                "SELECT id, dispatch_id, project_id, branch, base, how_to_verify, \
                       flux_verdict_ref, status, feedback, rework_dispatch_id, \
                       merge_commit, decided_by, decided_at, created_at, acceptance_plan \
                  FROM acceptances WHERE id = $1",
                &[&acceptance_id],
            )
            .await?;
        let decided = row_to_acceptance(&decided);
        tx.commit().await?;
        Ok(decided)
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
        event_type: &str,
        project_id: &str,
    ) -> Result<CanonicalEventRow, DbError> {
        let client = self.pool.get().await.map_err(DbError::Pool)?;
        let payload = serde_json::json!({
            "content": content,
            "target_seat_id": target_seat_id,
        });
        client
            .execute(
                "INSERT INTO canonical_events \
                 (id, event_type, occurred_at, actor_ref, payload, project_id) \
                 VALUES ($1, $2, NOW(), $3, $4, $5)",
                &[&event_id, &event_type, &actor_ref, &payload, &project_id],
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
        let row = client
            .query_one(
                "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, project_id \
                 FROM canonical_events WHERE id = $1",
                &[&event_id],
            )
            .await?;
        Ok(row_to_event(&row))
    }

    /// List supervisor-flavoured messages + related activity events, optionally
    /// filtered to events relevant to a given seat. A seat-targeted feed shows:
    ///
    ///   - events authored by the seat         (`actor_ref = 'seat:<seat_id>'`)
    ///   - messages addressed to the seat      (`payload->>'target_seat_id' = <seat_id>`)
    ///
    /// so both the user's outbound SupervisorMessages and the seat's own
    /// SessionStarted/HandoffSent/etc. events appear in its chat.
    ///
    /// The window is the NEWEST `limit` events, returned in ascending order (a
    /// chat reads oldest-at-top, newest-at-bottom; the UI does no client-side
    /// sorting and appends live events at the tail). The previous shape was a
    /// plain `ORDER BY occurred_at ASC LIMIT n`, which returns the OLDEST n
    /// rows — on a growing ledger that silently hides everything recent (the
    /// operator reported "no valid messages" while 115 newer events, his own
    /// included, sat past the window). So: select the newest `limit` rows, then
    /// reverse.
    ///
    /// `occurred_at` alone is not a total order (events can share a timestamp),
    /// so `id` is the tiebreaker in BOTH directions — the window is stable
    /// across calls and the reversal cannot reshuffle equal-timestamp rows.
    /// Sort: occurred_at ascending within the newest-`limit` window.
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
                    "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, \
                     project_id \
                     FROM ( \
                       SELECT id, event_type, occurred_at, actor_ref, payload, created_at, \
                              project_id \
                         FROM canonical_events \
                        WHERE actor_ref = $1 \
                           OR payload->>'target_seat_id' = $2 \
                        ORDER BY occurred_at DESC, id DESC LIMIT $3 \
                     ) recent \
                     ORDER BY occurred_at ASC, id ASC",
                    &[&actor_ref, &seat_id, &limit],
                )
                .await?
        } else {
            client
                .query(
                    "SELECT id, event_type, occurred_at, actor_ref, payload, created_at, \
                     project_id \
                     FROM ( \
                       SELECT id, event_type, occurred_at, actor_ref, payload, created_at, \
                              project_id \
                         FROM canonical_events \
                        ORDER BY occurred_at DESC, id DESC LIMIT $1 \
                     ) recent \
                     ORDER BY occurred_at ASC, id ASC",
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
//
// The runtime-triple value domains (`Harness`, `Agent`, `SeatRole`) are parsed
// in exactly ONE place: `check_domain` below, reached only from
// `row_to_seat`, `row_to_session` and `row_to_role_binding`. Nothing else in
// the codebase turns a database string into one of those enums. A value outside
// its domain fails here, naming the table, the row id, the column and the
// offending string — it is never absorbed into `Custom(...)` and never panics
// (Aegis review 2026-09-28 §5).
//
// The row structs themselves stay faithful `String` mirrors of the SQL columns;
// this boundary validates them rather than retyping them, so the wire DTOs keep
// their flat string shape.
// =============================================================================

/// Parse `value` as `T` purely to prove it is inside `T`'s declared domain,
/// discarding the parsed value. `None` is always accepted: a `NULL` column means
/// *not recorded*, which is a legitimate state (review §2).
fn check_domain<T>(
    table: &'static str,
    row_id: &str,
    column: &'static str,
    value: Option<&str>,
) -> Result<(), DbError>
where
    T: FromStr<Err = ParseValueError>,
{
    let Some(raw) = value else {
        return Ok(());
    };
    raw.parse::<T>()
        .map(|_| ())
        .map_err(|source| DbError::ValueDomain {
            table,
            row_id: row_id.to_string(),
            column,
            source,
        })
}

/// Validate the seat runtime triple. Split out from [`row_to_seat`] so the
/// failure path is unit-testable without constructing a `tokio_postgres::Row`.
fn check_seat_domains(
    id: &str,
    default_harness: Option<&str>,
    default_agent: Option<&str>,
) -> Result<(), DbError> {
    check_domain::<Harness>("seats", id, "default_harness", default_harness)?;
    check_domain::<Agent>("seats", id, "default_agent", default_agent)?;
    // default_model is deliberately unvalidated: model ids are free-form text
    // (§2.1 rule 2). There is no domain to check it against.
    Ok(())
}

/// Validate the session runtime triple. Split out from [`row_to_session`] for
/// the same reason as [`check_seat_domains`].
fn check_session_domains(id: &str, harness: Option<&str>, agent: &str) -> Result<(), DbError> {
    check_domain::<Harness>("sessions", id, "harness", harness)?;
    check_domain::<Agent>("sessions", id, "agent", Some(agent))?;
    Ok(())
}

fn row_to_seat(r: &Row) -> Result<SeatRow, DbError> {
    let row = SeatRow {
        id: r.get(0),
        name: r.get(1),
        default_harness: r.get(2),
        default_agent: r.get(3),
        default_model: r.get(4),
        capability_tags: r.get(5),
        status: r.get(6),
        created_at: r.get(7),
    };
    check_seat_domains(
        &row.id,
        row.default_harness.as_deref(),
        row.default_agent.as_deref(),
    )?;
    Ok(row)
}

/// The seat registry reduced to what routing needs — id, name, capability
/// tags, status. No runtime triple, therefore no `Harness`/`Agent` domain
/// validation, therefore no failure on a row whose harness is out of domain.
#[derive(Debug, Clone)]
pub struct SeatRoutingRow {
    pub id: String,
    pub name: String,
    pub capability_tags: Vec<String>,
    pub status: String,
}

fn row_to_seat_routing(r: &Row) -> SeatRoutingRow {
    SeatRoutingRow {
        id: r.get(0),
        name: r.get(1),
        capability_tags: r.get(2),
        status: r.get(3),
    }
}

fn row_to_archived_session(r: &Row) -> ArchivedSessionRow {
    ArchivedSessionRow {
        id: r.get(0),
        harness: r.get(1),
        project: r.get(2),
        cwd: r.get(3),
        session_date: r.get(4),
        bytes: r.get(5),
        lines: r.get(6),
        sha256: r.get(7),
        raw_path: r.get(8),
        src_path: r.get(9),
        message_count: r.get(10),
        first_ts: r.get(11),
        last_ts: r.get(12),
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

fn row_to_role_binding(r: &Row) -> Result<ProjectRoleBindingRow, DbError> {
    let row = ProjectRoleBindingRow {
        seat_id: r.get(0),
        project_id: r.get(1),
        role: r.get(2),
        authority_doc_refs: r.get(3),
        constraints: r.get(4),
        collaboration_template_ref: r.get(5),
        active_delegation_id: r.get(6),
    };
    // Composite key; the seat_id side identifies the row for a human reading
    // the error.
    check_domain::<SeatRole>(
        "project_role_bindings",
        &row.seat_id,
        "role",
        Some(&row.role),
    )?;
    Ok(row)
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

fn row_to_session(r: &Row) -> Result<SessionRow, DbError> {
    let row = SessionRow {
        id: r.get(0),
        seat_id: r.get(1),
        harness: r.get(2),
        agent: r.get(3),
        model: r.get(4),
        native_session_id: r.get(5),
        workspace_path: r.get(6),
        branch: r.get(7),
        status: r.get(8),
        launch_pack_ref: r.get(9),
        last_checkpoint_id: r.get(10),
        pid: r.get(11),
        created_at: r.get(12),
        ended_at: r.get(13),
        project_id: r.get(14),
    };
    check_session_domains(&row.id, row.harness.as_deref(), &row.agent)?;
    Ok(row)
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
        project_id: r.get(9),
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
        project_id: r.get(10),
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
        project_id: r.get(6),
    }
}

fn row_to_dispatch(r: &Row) -> DispatchRow {
    DispatchRow {
        id: r.get(0),
        target_seat_id: r.get(1),
        issued_by: r.get(2),
        project_id: r.get(3),
        title: r.get(4),
        payload: r.get(5),
        status: r.get(6),
        priority: r.get(7),
        issued_at: r.get(8),
        started_at: r.get(9),
        completed_at: r.get(10),
    }
}

fn row_to_dispatch_dep(r: &Row) -> DispatchDependencyRow {
    DispatchDependencyRow {
        dispatch_id: r.get(0),
        dep_kind: r.get(1),
        dep_value: r.get(2),
    }
}

/// The `memories` column list, in `MemoryRow`'s field order. Shared by every
/// memory read so the row hydration has ONE definition.
const MEMORY_COLUMNS: &str =
    "SELECT id, project_id, topic_id, title, tags, l0_summary, l1_overview, \
        l2_ref_kind, l2_document_id, l2_file_path, l2_body_digest, \
        source_kind, source_ref, kind, confidence, supersedes, l2_line_start, l2_line_end, \
        heat, recall_count, last_recalled_at, \
        tier, tier_changed_at, status, forgotten_reason, resolved_by, resolved_at, \
        created_at, updated_at \
          FROM memories";

fn row_to_memory(r: &Row) -> MemoryRow {
    MemoryRow {
        id: r.get(0),
        project_id: r.get(1),
        topic_id: r.get(2),
        title: r.get(3),
        tags: r.get(4),
        l0_summary: r.get(5),
        l1_overview: r.get(6),
        l2_ref_kind: r.get(7),
        l2_document_id: r.get(8),
        l2_file_path: r.get(9),
        l2_body_digest: r.get(10),
        source_kind: r.get(11),
        source_ref: r.get(12),
        kind: r.get(13),
        confidence: r.get(14),
        supersedes: r.get(15),
        l2_line_start: r.get(16),
        l2_line_end: r.get(17),
        heat: r.get(18),
        recall_count: r.get(19),
        last_recalled_at: r.get(20),
        tier: r.get(21),
        tier_changed_at: r.get(22),
        status: r.get(23),
        forgotten_reason: r.get(24),
        resolved_by: r.get(25),
        resolved_at: r.get(26),
        created_at: r.get(27),
        updated_at: r.get(28),
    }
}

fn row_to_knowledge_topic_node(r: &Row) -> KnowledgeTopicNode {
    KnowledgeTopicNode {
        topic_id: r.get(0),
        title: r.get(1),
        status: r.get(2),
        parent_topic_id: r.get(3),
        memory_count: r.get(4),
    }
}

/// One memory as the knowledge panel renders it, at the requested depth. The
/// higher layers are `None` below their depth — progressive disclosure is a
/// property of the SHAPE, not of a post-filter.
fn row_to_knowledge_memory_card(r: &Row, depth: MemoryLayer) -> KnowledgeMemoryCard {
    KnowledgeMemoryCard {
        memory_id: r.get(0),
        topic_id: r.get(1),
        title: r.get(2),
        kind: r.get(3),
        confidence: r.get(4),
        tier: r.get(5),
        status: r.get(6),
        l0_summary: r.get(7),
        l1_overview: match depth {
            MemoryLayer::L0 => None,
            _ => r.get(8),
        },
        l2: match depth {
            MemoryLayer::L2 => Some(KnowledgeL2Pointer {
                ref_kind: r.get(9),
                document_id: r.get(10),
                file_path: r.get(11),
                body_digest: r.get(12),
                line_start: r.get(13),
                line_end: r.get(14),
            }),
            _ => None,
        },
        superseded_by: r.get(17),
        source_kind: r.get(15),
        source_ref: r.get(16),
    }
}

fn row_to_topic(r: &Row) -> TopicRow {
    TopicRow {
        id: r.get(0),
        project_id: r.get(1),
        title: r.get(2),
        status: r.get(3),
        parent_topic_id: r.get(4),
    }
}

fn row_to_memory_relation(r: &Row) -> MemoryRelationRow {
    MemoryRelationRow {
        from_id: r.get(0),
        to_id: r.get(1),
        rel: r.get(2),
    }
}

fn row_to_acceptance(r: &Row) -> AcceptanceRow {
    AcceptanceRow {
        id: r.get(0),
        dispatch_id: r.get(1),
        project_id: r.get(2),
        branch: r.get(3),
        base: r.get(4),
        how_to_verify: r.get(5),
        flux_verdict_ref: r.get(6),
        status: r.get(7),
        feedback: r.get(8),
        rework_dispatch_id: r.get(9),
        merge_commit: r.get(10),
        decided_by: r.get(11),
        decided_at: r.get(12),
        created_at: r.get(13),
        // 016: the structured acceptance plan. Read as stored — '{}' means "no
        // plan yet" and the lineage assembler renders that honestly.
        acceptance_plan: r.get(14),
    }
}

fn row_to_acceptance_with_dispatch(r: &Row) -> AcceptanceWithDispatchRow {
    AcceptanceWithDispatchRow {
        acceptance: row_to_acceptance(r),
        dispatch_title: r.get(15),
        dispatch_seat_id: r.get(16),
        dispatch_priority: r.get(17),
        dispatch_payload: r.get(18),
        flux_verdict: r.get(19),
        flux_verdict_occurred_at: r.get(20),
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
    /// The write API refused a draft (bad refs / payload shape) before any write.
    /// Boxed because `WriteError` itself has a `Db(DbError)` variant — an unboxed
    /// `#[from] WriteError` would make the two enums recursively sized.
    #[error("event write rejected: {0}")]
    Write(Box<crate::db::write_api::WriteError>),
    /// The acceptance was not in `pending` state (already decided, or unknown
    /// id) — the guarded write refused, so nothing was written.
    #[error("acceptance {id:?} is not pending (already decided or unknown)")]
    AcceptanceNotPending { id: String },
    /// `015`: the memory id names no row.
    #[error("memory {id:?} not found")]
    MemoryNotFound { id: String },
    /// `015`: the memory is not `active` — a forgotten row is a tombstone, and a
    /// recall of one would resurrect it through the back door; a second forget
    /// would rewrite a recorded fact.
    #[error("memory {id:?} is not active (forgotten rows are immutable tombstones)")]
    MemoryNotActive { id: String },
    /// `015`: `promote` was called with the tier the memory already has — no
    /// transition, so nothing is written and no event is appended.
    #[error(
        "memory {id:?} is already in tier {tier:?} — promote needs a target tier that differs"
    )]
    MemoryNoTierChange { id: String, tier: String },
    /// A stored string is outside the value domain its Rust type declares.
    /// Raised only at the row-hydration parse boundary (see `check_domain`).
    #[error("{table} row {row_id:?}: column {column} is out of domain: {source}")]
    ValueDomain {
        table: &'static str,
        row_id: String,
        column: &'static str,
        #[source]
        source: ParseValueError,
    },
}

impl From<crate::db::write_api::WriteError> for DbError {
    fn from(e: crate::db::write_api::WriteError) -> Self {
        DbError::Write(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::{check_seat_domains, check_session_domains, DbError};

    /// Happy path: every value the seed writes for the six converged seats and
    /// the five historical sessions must hydrate.
    #[test]
    fn seeded_values_hydrate() {
        check_seat_domains("seat-nimbus-001", Some("Stepcode"), Some("ClaudeCode"))
            .expect("converged seat triple must hydrate");
        check_session_domains("ses-lyra-coord-001", None, "GeminiCli")
            .expect("historical session with unrecorded harness must hydrate");
        // seed/001:75 stores the bare tag `Custom` with no payload.
        check_session_domains("ses-flux-verify-001", None, "Custom")
            .expect("bare Custom tag must hydrate");
    }

    /// §2.4.5 negative path. Hydrating a session row whose `agent` is out of
    /// domain must fail with a typed error naming the table, the row id, the
    /// column and the offending string — not a `Custom(...)` fallback and not a
    /// panic.
    #[test]
    fn out_of_domain_agent_fails_loudly_naming_the_row() {
        let err = check_session_domains("ses-flux-verify-001", None, "Bogus")
            .expect_err("out-of-domain agent must be rejected at the hydration boundary");

        match &err {
            DbError::ValueDomain {
                table,
                row_id,
                column,
                source,
            } => {
                assert_eq!(*table, "sessions");
                assert_eq!(row_id, "ses-flux-verify-001");
                assert_eq!(*column, "agent");
                assert_eq!(source.domain, "Agent");
                assert_eq!(source.value, "Bogus");
            }
            other => panic!("expected DbError::ValueDomain, got {other:?}"),
        }

        let rendered = err.to_string();
        for needle in ["sessions", "ses-flux-verify-001", "agent"] {
            assert!(
                rendered.contains(needle),
                "error message must contain {needle:?}: {rendered}"
            );
        }
        assert!(
            format!("{}", std::error::Error::source(&err).expect("has source"))
                .contains("\"Bogus\""),
            "source must name the offending value: {rendered}"
        );
    }

    /// Pins the exact operator-facing text a malformed row produces, so the
    /// message cannot silently lose the table, row id, column or value.
    #[test]
    fn rendered_error_names_table_row_column_and_value() {
        let err = check_session_domains("ses-flux-verify-001", None, "Bogus")
            .expect_err("out-of-domain agent must be rejected");
        assert_eq!(
            err.to_string(),
            "sessions row \"ses-flux-verify-001\": column agent is out of domain: \
             \"Bogus\" is not a valid Agent value (expected one of: ClaudeCode, Codex, \
             CursorCli, GeminiCli, Opencode, Custom, Custom(<payload>))"
        );
    }

    #[test]
    fn out_of_domain_harness_fails_loudly() {
        let err = check_session_domains("ses-aegis-supervisor-001", Some("Podman"), "ClaudeCode")
            .expect_err("out-of-domain harness must be rejected");
        assert!(matches!(
            err,
            DbError::ValueDomain {
                column: "harness",
                ..
            }
        ));
    }

    #[test]
    fn out_of_domain_seat_default_agent_fails_loudly() {
        let err = check_seat_domains("seat-onyx-001", Some("Stepcode"), Some("Bogus"))
            .expect_err("out-of-domain default_agent must be rejected");
        match err {
            DbError::ValueDomain {
                table,
                row_id,
                column,
                ..
            } => {
                assert_eq!(table, "seats");
                assert_eq!(row_id, "seat-onyx-001");
                assert_eq!(column, "default_agent");
            }
            other => panic!("expected DbError::ValueDomain, got {other:?}"),
        }
    }

    /// `NULL` is not an error. Review §2: an unrecorded harness/model is a
    /// legitimate state, distinct from a wrong one.
    #[test]
    fn null_harness_and_agent_are_accepted() {
        check_seat_domains("seat-unrecorded", None, None).expect("NULL columns are legitimate");
        check_session_domains("ses-unrecorded", None, "ClaudeCode")
            .expect("NULL harness is legitimate");
    }
}
