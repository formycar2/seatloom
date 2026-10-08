//! P0 step 2 — the single write primitive: `append(event) -> projections`.
//!
//! **One rule.** Every domain mutation is an event append **plus** its projection
//! update, in **one transaction**. Nothing outside this module writes a
//! projection table. Reads stay on the projections (CQRS-lite), unchanged.
//!
//! Before this module the ledger was decorative: `canonical_events` rows were
//! written *alongside* mutations (see `repositories::append_supervisor_message`)
//! rather than *causing* them, so a projection could not be reconstructed from
//! the ledger. This inverts that for the `workitems` projection.
//!
//! ## What `010` left to this module
//!
//! `infra/postgres/schema/010_p0_schema_foundation.sql` carries two transitional
//! defaults that exist only to keep pre-step-2 writers alive. This module must
//! not benefit from either:
//!
//! - `canonical_events.schema_version DEFAULT 1` (`010:87`) — every append here
//!   sets it **explicitly** from [`EventDraft::schema_version`]. The INSERT names
//!   the column, so dropping the default (a tracked P1 cleanup) cannot affect it.
//! - `event_object_refs.role DEFAULT 'context'` (`010:220`) — every ref carries an
//!   explicit [`RefRole`]. `010:220` states the rule: *"MUST set role explicitly;
//!   a defaulted role in new code is a bug."*
//!
//! `010:332`'s partial unique index gives **at most one** subject per event.
//! [`EventDraft::validate`] supplies the other half — **presence** — so together
//! they mean *exactly one*. Zero subjects is rejected in Rust, before the database
//! is touched.
//!
//! ## Two event vocabularies exist, deliberately not unified here
//!
//! [`crate::db::event_domain::EventType`] is the **PostgreSQL** vocabulary: 12
//! values, pinned to `010`'s CHECK. [`crate::ledger::event::EventType`] is the
//! **JSONL file ledger's** vocabulary (27 variants, serde-derived, AD-005) and is
//! strictly wider — it carries `SessionCompleted`, `PromptDetected`,
//! `PipelineStarted` and others that PostgreSQL would reject. Unifying them is
//! out of scope for this packet and is reported as a finding in the delivery doc.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_postgres::Transaction;

use crate::db::event_domain::{EventType, RefRole, RefType};
use crate::db::repositories::{DbError, SeatloomDb};

// =============================================================================
// Drafts — what a caller submits
// =============================================================================

/// One object reference attached to an event. `role` is mandatory by type: there
/// is no way to construct this without stating it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRefDraft {
    pub ref_type: RefType,
    pub ref_id: String,
    pub role: RefRole,
}

impl ObjectRefDraft {
    pub fn new(ref_type: RefType, ref_id: impl Into<String>, role: RefRole) -> Self {
        Self {
            ref_type,
            ref_id: ref_id.into(),
            role,
        }
    }

    /// Convenience for the mandatory subject ref.
    pub fn subject(ref_type: RefType, ref_id: impl Into<String>) -> Self {
        Self::new(ref_type, ref_id, RefRole::Subject)
    }
}

/// A new event awaiting append.
///
/// `payload` is **required**, not `Option`. §3 of the packet leaves the choice to
/// this API and asks it to be stated: the projection is rebuilt *from the
/// payload*, so an event without one is unreplayable by construction, and
/// admitting it would reintroduce exactly the decorative ledger step 2 removes.
/// The SQL column stays nullable (`010` deliberately left it so, to keep NULL
/// "not recorded" distinguishable from `{}` "recorded, empty") — this is an API
/// tightening, not a schema change.
#[derive(Debug, Clone)]
pub struct EventDraft {
    pub event_id: String,
    pub event_type: EventType,
    /// Payload shape version, per `event_type`, starting at 1. Set explicitly —
    /// never rely on `010:87`'s `DEFAULT 1`.
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub actor_ref: String,
    pub project_id: String,
    /// Ordinal within the session's turn stream (T2 join key, `010:57`).
    /// `None` means *not recorded* — never `0`. T2 capture is P1, so P0 appends
    /// pass `None`.
    pub turn_seq: Option<i32>,
    pub payload: serde_json::Value,
    pub refs: Vec<ObjectRefDraft>,
}

impl EventDraft {
    /// All validation that can be done without the database. Called by
    /// [`SeatloomDb::append_event`] before a connection is taken, so a malformed
    /// draft never opens a transaction.
    pub fn validate(&self) -> Result<&ObjectRefDraft, WriteError> {
        if self.event_id.trim().is_empty() {
            return Err(WriteError::EmptyField { field: "event_id" });
        }
        if self.actor_ref.trim().is_empty() {
            return Err(WriteError::EmptyField { field: "actor_ref" });
        }
        if self.project_id.trim().is_empty() {
            return Err(WriteError::EmptyField {
                field: "project_id",
            });
        }
        if self.schema_version < 1 {
            return Err(WriteError::SchemaVersionOutOfRange {
                event_id: self.event_id.clone(),
                schema_version: self.schema_version,
            });
        }
        if let Some(seq) = self.turn_seq {
            if seq < 0 {
                return Err(WriteError::TurnSeqNegative {
                    event_id: self.event_id.clone(),
                    turn_seq: seq,
                });
            }
        }
        if !self.payload.is_object() {
            return Err(WriteError::PayloadNotAnObject {
                event_id: self.event_id.clone(),
                found: payload_kind(&self.payload),
            });
        }
        for r in &self.refs {
            if r.ref_id.trim().is_empty() {
                return Err(WriteError::EmptyField { field: "ref_id" });
            }
        }

        // §2.3 — the write API's half of "exactly one subject". `010:332` gives
        // at-most-one; presence is enforced here. Rejecting >1 too means the
        // caller gets a clear typed error instead of an opaque unique-index
        // violation from the driver.
        let subjects: Vec<&ObjectRefDraft> = self
            .refs
            .iter()
            .filter(|r| r.role == RefRole::Subject)
            .collect();
        match subjects.len() {
            0 => Err(WriteError::NoSubject {
                event_id: self.event_id.clone(),
                event_type: self.event_type,
                ref_count: self.refs.len(),
            }),
            1 => Ok(subjects[0]),
            n => Err(WriteError::MultipleSubjects {
                event_id: self.event_id.clone(),
                count: n,
            }),
        }
    }
}

fn payload_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// =============================================================================
// Typed payloads, per (event_type, schema_version)
// =============================================================================

/// `(WorkItemCreated, 1)`. Carries **everything the `workitems` projection needs**
/// — that is the property that makes the projection replayable. Contrast seed
/// 001's historical payloads, which carry only `{workitem_id, title}` and are
/// therefore unreplayable (packet §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItemCreatedV1 {
    pub title: String,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub owner_seat_id: Option<String>,
    pub status: String,
    pub priority: String,
}

/// `(WorkItemStatusChanged, 1)`. `from_status` is optional and advisory — the
/// replay uses `to_status` only, so a missing `from_status` cannot desynchronise
/// a rebuild.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItemStatusChangedV1 {
    #[serde(default)]
    pub from_status: Option<String>,
    pub to_status: String,
}

// =============================================================================
// Outcomes and errors
// =============================================================================

/// What an append did to the projection layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionEffect {
    /// A `workitems` row was inserted.
    WorkItemInserted { workitem_id: String },
    /// A `workitems` row's status was advanced.
    WorkItemStatusUpdated {
        workitem_id: String,
        to_status: String,
    },
    /// This `event_type` has no projection in the step-2 slice. The event is
    /// still recorded in the ledger; it simply drives no projection table yet.
    /// Named rather than silent so "nothing happened" is auditable.
    LedgerOnly { event_type: EventType },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOutcome {
    pub event_id: String,
    pub subject: ObjectRefDraft,
    pub projection: ProjectionEffect,
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },

    /// §2.3, the reason this error exists at all.
    #[error(
        "event {event_id:?} ({event_type}) carries no subject ref: exactly one ref with \
         role='subject' is required, found 0 of {ref_count} ref(s)"
    )]
    NoSubject {
        event_id: String,
        event_type: EventType,
        ref_count: usize,
    },

    #[error(
        "event {event_id:?} carries {count} subject refs: exactly one ref with role='subject' \
         is required (010:332 enforces at most one; this API enforces presence)"
    )]
    MultipleSubjects { event_id: String, count: usize },

    #[error("event {event_id:?} has schema_version {schema_version}; must be >= 1")]
    SchemaVersionOutOfRange {
        event_id: String,
        schema_version: i32,
    },

    #[error("event {event_id:?} has turn_seq {turn_seq}; must be >= 0 when recorded (NULL means not recorded)")]
    TurnSeqNegative { event_id: String, turn_seq: i32 },

    #[error("event {event_id:?} payload must be a JSON object, found {found}")]
    PayloadNotAnObject {
        event_id: String,
        found: &'static str,
    },

    #[error(
        "event {event_id:?}: subject ref must be ref_type={expected} for event_type={event_type}, \
         found {actual}"
    )]
    WrongSubjectType {
        event_id: String,
        event_type: EventType,
        expected: RefType,
        actual: RefType,
    },

    #[error("event {event_id:?} ({event_type} v{schema_version}) payload does not match its declared shape: {source}")]
    PayloadShape {
        event_id: String,
        event_type: EventType,
        schema_version: i32,
        #[source]
        source: serde_json::Error,
    },

    #[error("event {event_id:?}: no (event_type, schema_version) payload contract for ({event_type}, v{schema_version})")]
    UnsupportedPayloadVersion {
        event_id: String,
        event_type: EventType,
        schema_version: i32,
    },

    /// A projection update found no row to update — the ledger references a
    /// workitem the projection does not have. Surfaced rather than ignored: a
    /// silent zero-row UPDATE is how a projection drifts from its ledger.
    #[error("event {event_id:?}: projection update matched no row for workitem {workitem_id:?}")]
    ProjectionRowMissing {
        event_id: String,
        workitem_id: String,
    },

    /// Raised by the rebuild when a historical event cannot reconstruct its
    /// projection row. This is the measured limitation in packet §4, made
    /// demonstrable instead of merely described.
    #[error(
        "rebuild cannot replay event {event_id:?} ({event_type} v{schema_version}): {reason}. \
         Pre-step-2 history is lossy by construction; see \
         NIMBUS-2026-09-30-p0-write-api-delivery-v1 §Replayability."
    )]
    UnreplayableHistory {
        event_id: String,
        event_type: EventType,
        schema_version: i32,
        reason: String,
    },

    #[error(transparent)]
    Db(#[from] DbError),

    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
}

// =============================================================================
// The append primitive
// =============================================================================

/// Column list for `canonical_events` inserts. `schema_version` and `turn_seq`
/// are named explicitly — that is what makes `010`'s defaults irrelevant here.
const INSERT_EVENT_SQL: &str = "INSERT INTO canonical_events \
     (id, event_type, occurred_at, actor_ref, payload, project_id, schema_version, turn_seq) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

/// `role` is named explicitly. Deliberately **no** `ON CONFLICT DO NOTHING`: the
/// pre-step-2 path used one (`repositories.rs`, the Supervisor IM ref) and that is
/// precisely how a colliding ref was silently dropped, losing the role
/// distinction `010` G3 exists to record. A duplicate must fail loudly.
const INSERT_REF_SQL: &str = "INSERT INTO event_object_refs (event_id, ref_type, ref_id, role) \
     VALUES ($1, $2, $3, $4)";

impl SeatloomDb {
    /// **The** write primitive. Appends the event and applies its projection in
    /// one transaction; on any failure both roll back together.
    ///
    /// Validation happens before a connection is taken, so a rejected draft never
    /// starts a transaction.
    ///
    /// This is a thin wrapper over [`append_event_in_tx`]: it validates, opens a
    /// transaction, delegates the actual event + refs + projection write to the
    /// in-tx variant, and commits. There is exactly **one** implementation of the
    /// append (in `append_event_in_tx`); this method only owns the
    /// validate-then-open-then-commit lifecycle so every existing caller keeps
    /// its unchanged `(&EventDraft) -> AppendOutcome` contract.
    /// (LYRA-2026-10-08-p0-step6-unblock-v1 §7-C, option b.)
    pub async fn append_event(&self, draft: &EventDraft) -> Result<AppendOutcome, WriteError> {
        // Validate BEFORE taking a connection, so a rejected draft never opens a
        // transaction — the property step 4 verified and A1's doc comment states.
        draft.validate()?;

        let mut client = self.pool_client().await?;
        let tx = client.transaction().await?;
        let outcome = append_event_in_tx(&tx, draft).await?;
        tx.commit().await?;
        Ok(outcome)
    }

    /// Rebuild the `workitems` projection for one project **from the ledger
    /// alone** — `canonical_events` + `event_object_refs`, no other input. The
    /// A1 acceptance test (packet §4).
    ///
    /// Project-scoped on purpose: `handoffs.workitem_id` is `NOT NULL REFERENCES
    /// workitems(id)` (`001:99`), so a global truncate would fail against seeded
    /// handoffs. Scoping also keeps a rebuild from touching another project's
    /// rows.
    ///
    /// Runs in one transaction: delete then replay, so a failure mid-replay
    /// leaves the projection as it was.
    pub async fn rebuild_workitems_projection(
        &self,
        project_id: &str,
    ) -> Result<RebuildReport, WriteError> {
        let mut client = self.pool_client().await?;
        let tx = client.transaction().await?;

        let deleted = tx
            .execute(
                "DELETE FROM workitems WHERE project_id = $1",
                &[&project_id],
            )
            .await?;

        // Deterministic replay order. `occurred_at` is the domain ordering;
        // `id` breaks ties so a rebuild is reproducible.
        let rows = tx
            .query(
                "SELECT e.id, e.event_type, e.schema_version, e.occurred_at, e.payload, \
                        r.ref_type, r.ref_id \
                   FROM canonical_events e \
                   JOIN event_object_refs r \
                     ON r.event_id = e.id AND r.role = 'subject' \
                  WHERE e.project_id = $1 \
                    AND e.event_type IN ('WorkItemCreated', 'WorkItemStatusChanged') \
                  ORDER BY e.occurred_at ASC, e.id ASC",
                &[&project_id],
            )
            .await?;

        let mut replayed = 0usize;
        for row in &rows {
            let event_id: String = row.get(0);
            let event_type_raw: String = row.get(1);
            let schema_version: i32 = row.get(2);
            let occurred_at: DateTime<Utc> = row.get(3);
            let payload: Option<serde_json::Value> = row.get(4);
            let ref_type_raw: String = row.get(5);
            let ref_id: String = row.get(6);

            let event_type = event_type_raw.parse::<EventType>().map_err(|e| {
                WriteError::UnreplayableHistory {
                    event_id: event_id.clone(),
                    event_type: EventType::WorkItemCreated,
                    schema_version,
                    reason: format!("event_type is outside the Rust domain: {e}"),
                }
            })?;
            let ref_type =
                ref_type_raw
                    .parse::<RefType>()
                    .map_err(|e| WriteError::UnreplayableHistory {
                        event_id: event_id.clone(),
                        event_type,
                        schema_version,
                        reason: format!("subject ref_type is outside the Rust domain: {e}"),
                    })?;
            let payload = payload.ok_or_else(|| WriteError::UnreplayableHistory {
                event_id: event_id.clone(),
                event_type,
                schema_version,
                reason: "payload is NULL, so the projection row cannot be reconstructed"
                    .to_string(),
            })?;

            replay_workitem_event(
                &tx,
                &event_id,
                event_type,
                schema_version,
                occurred_at,
                &payload,
                ref_type,
                &ref_id,
                project_id,
            )
            .await?;
            replayed += 1;
        }

        tx.commit().await?;

        Ok(RebuildReport {
            project_id: project_id.to_string(),
            rows_deleted: deleted,
            events_replayed: replayed,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuildReport {
    pub project_id: String,
    pub rows_deleted: u64,
    pub events_replayed: usize,
}

// =============================================================================
// The append primitive, against a caller-supplied transaction
// =============================================================================
//
// This is the SINGLE implementation of "append event + refs + projection". Both
// `append_event` (which opens and commits its own transaction) and the governed
// dispatch scheduler (which composes the append with a `dispatches` row change
// in ONE transaction, step-6 rule 2) call through here, so there is no second
// copy of the append logic to drift — the exact property A1 exists to guarantee.
// (LYRA-2026-10-08-p0-step6-unblock-v1 §7-C, option b.)
//
// It is `pub(crate)`: the scheduler lives in a separate crate
// (`seatloom-mcp-emit`) and reaches it through a thin `SeatloomDb` method rather
// than touching the pool or the projection functions directly. The ref INSERT
// keeps NO `ON CONFLICT DO NOTHING` — a duplicate must fail loudly, the whole
// reason the clause was removed.
//
// Validation is the CALLER's responsibility before opening the transaction;
// `append_event` does exactly that, preserving "a rejected draft never starts a
// transaction". A caller that composes this into a larger transaction likewise
// validates first. We re-run `validate()` here only to obtain the subject ref
// the projection step needs; it is pure and cheap, and it cannot start a
// transaction, so it does not weaken the before-tx property.

/// Append `draft` (event + every ref + its projection) inside `tx`. Does **not**
/// commit — the caller owns the transaction lifecycle, which is what lets the
/// scheduler put a status transition and its event in the same transaction.
///
/// `pub` so the governed-dispatch scheduler in the separate `seatloom-mcp-emit`
/// crate can compose it with a `dispatches` row change in one transaction
/// (step-6 rule 2). It is the ONLY public way to append against a borrowed
/// transaction; every caller still goes through the same one implementation.
pub async fn append_event_in_tx(
    tx: &Transaction<'_>,
    draft: &EventDraft,
) -> Result<AppendOutcome, WriteError> {
    let subject = draft.validate()?.clone();

    tx.execute(
        INSERT_EVENT_SQL,
        &[
            &draft.event_id,
            &draft.event_type.as_str(),
            &draft.occurred_at,
            &draft.actor_ref,
            &draft.payload,
            &draft.project_id,
            &draft.schema_version,
            &draft.turn_seq,
        ],
    )
    .await?;

    for r in &draft.refs {
        tx.execute(
            INSERT_REF_SQL,
            &[
                &draft.event_id,
                &r.ref_type.as_str(),
                &r.ref_id,
                &r.role.as_str(),
            ],
        )
        .await?;
    }

    let projection = apply_projection(tx, draft, &subject).await?;

    Ok(AppendOutcome {
        event_id: draft.event_id.clone(),
        subject,
        projection,
    })
}

// =============================================================================
// Projection writes — PRIVATE to this module
// =============================================================================
//
// Done-definition item 1: "no repository method mutates a projection table
// directly". These functions are the ONLY place `workitems` is written, and they
// are not `pub` — nothing outside `db::write_api` can reach them, so the only way
// to change the projection is to append an event. Enforced by the module system,
// not by convention.

async fn apply_projection(
    tx: &Transaction<'_>,
    draft: &EventDraft,
    subject: &ObjectRefDraft,
) -> Result<ProjectionEffect, WriteError> {
    match draft.event_type {
        EventType::WorkItemCreated | EventType::WorkItemStatusChanged => {
            if subject.ref_type != RefType::WorkItem {
                return Err(WriteError::WrongSubjectType {
                    event_id: draft.event_id.clone(),
                    event_type: draft.event_type,
                    expected: RefType::WorkItem,
                    actual: subject.ref_type,
                });
            }
            replay_workitem_event(
                tx,
                &draft.event_id,
                draft.event_type,
                draft.schema_version,
                draft.occurred_at,
                &draft.payload,
                subject.ref_type,
                &subject.ref_id,
                &draft.project_id,
            )
            .await
        }
        // No projection in the step-2 slice. The event is still appended; it
        // simply drives no projection table yet. Adding one is a later packet.
        other => Ok(ProjectionEffect::LedgerOnly { event_type: other }),
    }
}

/// The shared apply/replay step. Used by both [`apply_projection`] (live append)
/// and `rebuild_workitems_projection` (replay), so a rebuild cannot drift from a
/// live write: there is exactly one implementation of "what this event does to the
/// projection".
#[allow(clippy::too_many_arguments)]
async fn replay_workitem_event(
    tx: &Transaction<'_>,
    event_id: &str,
    event_type: EventType,
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    payload: &serde_json::Value,
    subject_ref_type: RefType,
    workitem_id: &str,
    project_id: &str,
) -> Result<ProjectionEffect, WriteError> {
    if subject_ref_type != RefType::WorkItem {
        return Err(WriteError::WrongSubjectType {
            event_id: event_id.to_string(),
            event_type,
            expected: RefType::WorkItem,
            actual: subject_ref_type,
        });
    }

    match (event_type, schema_version) {
        (EventType::WorkItemCreated, 1) => {
            let p: WorkItemCreatedV1 =
                serde_json::from_value(payload.clone()).map_err(|source| {
                    WriteError::PayloadShape {
                        event_id: event_id.to_string(),
                        event_type,
                        schema_version,
                        source,
                    }
                })?;
            tx.execute(
                "INSERT INTO workitems \
                 (id, title, goal, acceptance_criteria, owner_seat_id, status, priority, \
                  created_at, updated_at, project_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $9)",
                &[
                    &workitem_id,
                    &p.title,
                    &p.goal,
                    &p.acceptance_criteria,
                    &p.owner_seat_id,
                    &p.status,
                    &p.priority,
                    &occurred_at,
                    &project_id,
                ],
            )
            .await?;
            Ok(ProjectionEffect::WorkItemInserted {
                workitem_id: workitem_id.to_string(),
            })
        }
        (EventType::WorkItemStatusChanged, 1) => {
            let p: WorkItemStatusChangedV1 =
                serde_json::from_value(payload.clone()).map_err(|source| {
                    WriteError::PayloadShape {
                        event_id: event_id.to_string(),
                        event_type,
                        schema_version,
                        source,
                    }
                })?;
            let n = tx
                .execute(
                    "UPDATE workitems SET status = $1, updated_at = $2 WHERE id = $3",
                    &[&p.to_status, &occurred_at, &workitem_id],
                )
                .await?;
            if n == 0 {
                return Err(WriteError::ProjectionRowMissing {
                    event_id: event_id.to_string(),
                    workitem_id: workitem_id.to_string(),
                });
            }
            Ok(ProjectionEffect::WorkItemStatusUpdated {
                workitem_id: workitem_id.to_string(),
                to_status: p.to_status,
            })
        }
        _ => Err(WriteError::UnsupportedPayloadVersion {
            event_id: event_id.to_string(),
            event_type,
            schema_version,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 30, 12, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn draft_with_refs(refs: Vec<ObjectRefDraft>) -> EventDraft {
        EventDraft {
            event_id: "ev-p0-test-001".to_string(),
            event_type: EventType::WorkItemCreated,
            schema_version: 1,
            occurred_at: ts(),
            actor_ref: "seat:seat-nimbus-001".to_string(),
            project_id: "seatloom".to_string(),
            turn_seq: None,
            payload: serde_json::json!({
                "title": "t", "status": "draft", "priority": "medium"
            }),
            refs,
        }
    }

    /// §2.3 negative path — the obligation `010` left to this API. An append with
    /// zero subjects is rejected by a typed error, before any database contact.
    #[test]
    fn zero_subject_append_is_rejected() {
        let d = draft_with_refs(vec![
            ObjectRefDraft::new(RefType::Seat, "seat-nimbus-001", RefRole::Actor),
            ObjectRefDraft::new(RefType::Project, "seatloom", RefRole::Context),
        ]);
        let err = d.validate().expect_err("zero subjects must be rejected");
        match &err {
            WriteError::NoSubject {
                event_id,
                event_type,
                ref_count,
            } => {
                assert_eq!(event_id, "ev-p0-test-001");
                assert_eq!(*event_type, EventType::WorkItemCreated);
                assert_eq!(*ref_count, 2);
            }
            other => panic!("expected NoSubject, got {other:?}"),
        }
        assert_eq!(
            err.to_string(),
            "event \"ev-p0-test-001\" (WorkItemCreated) carries no subject ref: exactly one ref \
             with role='subject' is required, found 0 of 2 ref(s)"
        );
    }

    /// An event with no refs at all is the same defect, and must not slip through
    /// a length check that only looks at the subject filter.
    #[test]
    fn append_with_no_refs_at_all_is_rejected() {
        let err = draft_with_refs(vec![])
            .validate()
            .expect_err("no refs must be rejected");
        assert!(matches!(err, WriteError::NoSubject { ref_count: 0, .. }));
    }

    #[test]
    fn exactly_one_subject_is_accepted_and_returned() {
        let d = draft_with_refs(vec![
            ObjectRefDraft::new(RefType::Seat, "seat-nimbus-001", RefRole::Actor),
            ObjectRefDraft::subject(RefType::WorkItem, "wi-p0-001"),
        ]);
        let subject = d.validate().expect("one subject is valid");
        assert_eq!(subject.role, RefRole::Subject);
        assert_eq!(subject.ref_type, RefType::WorkItem);
        assert_eq!(subject.ref_id, "wi-p0-001");
    }

    /// Belt and braces with `010:332`. The database would also reject this, but a
    /// typed error names the problem instead of surfacing a unique-index violation.
    #[test]
    fn two_subjects_are_rejected_in_rust_before_the_index_fires() {
        let err = draft_with_refs(vec![
            ObjectRefDraft::subject(RefType::WorkItem, "wi-a"),
            ObjectRefDraft::subject(RefType::WorkItem, "wi-b"),
        ])
        .validate()
        .expect_err("two subjects must be rejected");
        assert!(matches!(err, WriteError::MultipleSubjects { count: 2, .. }));
    }

    /// `schema_version` must be supplied and sane — never inherited from
    /// `010:87`'s transitional `DEFAULT 1`.
    #[test]
    fn schema_version_below_one_is_rejected() {
        let mut d = draft_with_refs(vec![ObjectRefDraft::subject(RefType::WorkItem, "wi-a")]);
        d.schema_version = 0;
        assert!(matches!(
            d.validate().expect_err("0 must be rejected"),
            WriteError::SchemaVersionOutOfRange {
                schema_version: 0,
                ..
            }
        ));
        d.schema_version = -1;
        assert!(matches!(
            d.validate().expect_err("negative must be rejected"),
            WriteError::SchemaVersionOutOfRange {
                schema_version: -1,
                ..
            }
        ));
    }

    /// `turn_seq` NULL means *not recorded*; a negative ordinal is a defect, and
    /// `0` is a legitimate recorded value that must NOT be rejected.
    #[test]
    fn turn_seq_domain_is_enforced_without_conflating_none_and_zero() {
        let mut d = draft_with_refs(vec![ObjectRefDraft::subject(RefType::WorkItem, "wi-a")]);
        d.turn_seq = Some(-1);
        assert!(matches!(
            d.validate().expect_err("negative must be rejected"),
            WriteError::TurnSeqNegative { turn_seq: -1, .. }
        ));
        d.turn_seq = Some(0);
        assert!(d.validate().is_ok(), "0 is a recorded ordinal, not absence");
        d.turn_seq = None;
        assert!(d.validate().is_ok(), "None is 'not recorded'");
    }

    /// The API requires a payload and requires it to be an object — a scalar or
    /// array cannot carry a projection's fields.
    #[test]
    fn payload_must_be_a_json_object() {
        for (bad, kind) in [
            (serde_json::Value::Null, "null"),
            (serde_json::json!([1, 2]), "array"),
            (serde_json::json!("nope"), "string"),
            (serde_json::json!(7), "number"),
            (serde_json::json!(true), "boolean"),
        ] {
            let mut d = draft_with_refs(vec![ObjectRefDraft::subject(RefType::WorkItem, "wi-a")]);
            d.payload = bad;
            match d.validate().expect_err("must be rejected") {
                WriteError::PayloadNotAnObject { found, .. } => assert_eq!(found, kind),
                other => panic!("expected PayloadNotAnObject, got {other:?}"),
            }
        }
    }

    #[test]
    fn blank_required_fields_are_rejected() {
        for (mutate, field) in [
            (
                (|d: &mut EventDraft| d.event_id = "  ".into()) as fn(&mut EventDraft),
                "event_id",
            ),
            (|d: &mut EventDraft| d.actor_ref = "".into(), "actor_ref"),
            (|d: &mut EventDraft| d.project_id = "".into(), "project_id"),
        ] {
            let mut d = draft_with_refs(vec![ObjectRefDraft::subject(RefType::WorkItem, "wi-a")]);
            mutate(&mut d);
            match d.validate().expect_err("must be rejected") {
                WriteError::EmptyField { field: f } => assert_eq!(f, field),
                other => panic!("expected EmptyField({field}), got {other:?}"),
            }
        }
    }

    #[test]
    fn blank_ref_id_is_rejected() {
        let err = draft_with_refs(vec![ObjectRefDraft::subject(RefType::WorkItem, "")])
            .validate()
            .expect_err("blank ref_id must be rejected");
        assert!(matches!(err, WriteError::EmptyField { field: "ref_id" }));
    }

    /// The typed payloads must accept exactly what the projection needs, and the
    /// optional fields must be genuinely optional so a caller can record
    /// "not set" without inventing a value.
    #[test]
    fn workitem_created_v1_payload_round_trips() {
        let p = WorkItemCreatedV1 {
            title: "Write the write API".to_string(),
            goal: Some("append(event) -> projections".to_string()),
            acceptance_criteria: vec!["atomic".to_string(), "replayable".to_string()],
            owner_seat_id: Some("seat-nimbus-001".to_string()),
            status: "in_progress".to_string(),
            priority: "high".to_string(),
        };
        let v = serde_json::to_value(&p).expect("serialize");
        assert_eq!(
            serde_json::from_value::<WorkItemCreatedV1>(v).expect("deserialize"),
            p
        );

        let minimal: WorkItemCreatedV1 = serde_json::from_value(serde_json::json!({
            "title": "t", "status": "draft", "priority": "low"
        }))
        .expect("optional fields may be omitted");
        assert_eq!(minimal.goal, None);
        assert_eq!(minimal.owner_seat_id, None);
        assert!(minimal.acceptance_criteria.is_empty());
    }

    /// Seed 001's historical `WorkItemCreated` payload shape — `{workitem_id,
    /// title}` — must NOT deserialize, because it lacks `status` and `priority`
    /// which the projection needs `NOT NULL`. This is packet §4's measurement,
    /// pinned as a test so the limitation is provable and cannot regress into a
    /// silent default.
    #[test]
    fn seeded_historical_payload_is_provably_unreplayable() {
        let seeded = serde_json::json!({"workitem_id": "wi-001", "title": "Define product form"});
        let err = serde_json::from_value::<WorkItemCreatedV1>(seeded)
            .expect_err("seeded payload lacks status/priority");
        let msg = err.to_string();
        assert!(
            msg.contains("status"),
            "error should name the first missing required field: {msg}"
        );
    }

    #[test]
    fn workitem_status_changed_v1_payload_round_trips() {
        let p = WorkItemStatusChangedV1 {
            from_status: Some("draft".to_string()),
            to_status: "in_progress".to_string(),
        };
        let v = serde_json::to_value(&p).expect("serialize");
        assert_eq!(
            serde_json::from_value::<WorkItemStatusChangedV1>(v).expect("deserialize"),
            p
        );
        let without_from: WorkItemStatusChangedV1 =
            serde_json::from_value(serde_json::json!({"to_status": "done"}))
                .expect("from_status is advisory");
        assert_eq!(without_from.from_status, None);
    }

    /// The INSERT statements must name `schema_version` and `role` explicitly, so
    /// dropping `010`'s transitional defaults (a tracked P1 cleanup) cannot
    /// change this module's behaviour. Asserted against the SQL text itself.
    #[test]
    fn inserts_name_the_columns_010_defaults_would_otherwise_fill() {
        assert!(
            INSERT_EVENT_SQL.contains("schema_version"),
            "append must set schema_version explicitly (010:87 DEFAULT 1 is a legacy crutch)"
        );
        assert!(
            INSERT_EVENT_SQL.contains("turn_seq"),
            "append must set turn_seq explicitly so NULL is a recorded decision"
        );
        assert!(
            INSERT_REF_SQL.contains("role"),
            "every ref must set role explicitly (010:220)"
        );
        assert!(
            !INSERT_REF_SQL.contains("ON CONFLICT"),
            "a duplicate ref must fail loudly, not be silently dropped"
        );
    }

    // =========================================================================
    // append_event_in_tx (7-C) — DB-backed, #[ignore] like the integration tests
    // =========================================================================
    //
    // These prove the in-tx variant honours the CALLER's transaction lifecycle:
    // it commits only when the caller commits, and rolls back when the caller
    // rolls back. Marked #[ignore] so `cargo test` stays green without a DB; run
    // with `--include-ignored` against a SCRATCH database (never live seatloom).
    use crate::db::{create_pool, SeatloomDb};

    async fn scratch_db() -> Option<(SeatloomDb, deadpool_postgres::Pool)> {
        // Refuse the live database by name, same guard as the proof harnesses.
        let url = std::env::var("DATABASE_URL").ok()?;
        if url.rsplit('/').next() == Some("seatloom") {
            panic!("refusing to run against the live `seatloom` database");
        }
        let pool = create_pool().ok()?;
        let db = SeatloomDb::new(pool.clone());
        db.ping().await.ok()?;
        Some((db, pool))
    }

    fn in_tx_draft(event_id: &str, wi: &str) -> EventDraft {
        EventDraft {
            event_id: event_id.to_string(),
            event_type: EventType::WorkItemCreated,
            schema_version: 1,
            occurred_at: ts(),
            actor_ref: "seat:seat-nimbus-001".to_string(),
            project_id: "p0-intx-proof".to_string(),
            turn_seq: None,
            payload: serde_json::json!({"title": "in-tx", "status": "draft", "priority": "low"}),
            refs: vec![
                ObjectRefDraft::subject(RefType::WorkItem, wi),
                ObjectRefDraft::new(RefType::Seat, "seat-nimbus-001", RefRole::Actor),
            ],
        }
    }

    #[tokio::test]
    #[ignore = "requires a scratch PostgreSQL (DATABASE_URL to a non-live DB)"]
    async fn append_event_in_tx_commits_with_the_callers_transaction() {
        let Some((_db, pool)) = scratch_db().await else {
            panic!("DATABASE_URL must point at a reachable scratch DB");
        };
        let mut client = pool.get().await.expect("client");
        // Fixtures the FK/projection need.
        client.execute("INSERT INTO projects (id, name, created_at, worker_budget_tokens, supervisor_budget_tokens) VALUES ('p0-intx-proof','in-tx', NOW(), 1024, 1024) ON CONFLICT (id) DO NOTHING", &[]).await.unwrap();
        client.execute("INSERT INTO seats (id, name, capability_tags, status, created_at) VALUES ('seat-nimbus-001','Nimbus','{}','active', NOW()) ON CONFLICT (id) DO NOTHING", &[]).await.unwrap();
        client
            .execute(
                "DELETE FROM event_object_refs WHERE event_id = 'ev-intx-commit'",
                &[],
            )
            .await
            .unwrap();
        client
            .execute(
                "DELETE FROM canonical_events WHERE id = 'ev-intx-commit'",
                &[],
            )
            .await
            .unwrap();
        client
            .execute("DELETE FROM workitems WHERE id = 'wi-intx-commit'", &[])
            .await
            .unwrap();

        let tx = client.transaction().await.expect("tx");
        let outcome = append_event_in_tx(&tx, &in_tx_draft("ev-intx-commit", "wi-intx-commit"))
            .await
            .expect("in-tx append should succeed");
        assert_eq!(outcome.event_id, "ev-intx-commit");
        tx.commit().await.expect("caller commits");

        let n: i64 = client
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE id = 'ev-intx-commit'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(n, 1, "committed event must be present after caller commits");
        let w: i64 = client
            .query_one(
                "SELECT count(*) FROM workitems WHERE id = 'wi-intx-commit'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(w, 1, "projection written in the same tx must be present");
    }

    #[tokio::test]
    #[ignore = "requires a scratch PostgreSQL (DATABASE_URL to a non-live DB)"]
    async fn append_event_in_tx_rolls_back_with_the_callers_transaction() {
        let Some((_db, pool)) = scratch_db().await else {
            panic!("DATABASE_URL must point at a reachable scratch DB");
        };
        let mut client = pool.get().await.expect("client");
        client.execute("INSERT INTO projects (id, name, created_at, worker_budget_tokens, supervisor_budget_tokens) VALUES ('p0-intx-proof','in-tx', NOW(), 1024, 1024) ON CONFLICT (id) DO NOTHING", &[]).await.unwrap();
        client.execute("INSERT INTO seats (id, name, capability_tags, status, created_at) VALUES ('seat-nimbus-001','Nimbus','{}','active', NOW()) ON CONFLICT (id) DO NOTHING", &[]).await.unwrap();
        client
            .execute(
                "DELETE FROM event_object_refs WHERE event_id = 'ev-intx-rollback'",
                &[],
            )
            .await
            .unwrap();
        client
            .execute(
                "DELETE FROM canonical_events WHERE id = 'ev-intx-rollback'",
                &[],
            )
            .await
            .unwrap();
        client
            .execute("DELETE FROM workitems WHERE id = 'wi-intx-rollback'", &[])
            .await
            .unwrap();

        let tx = client.transaction().await.expect("tx");
        append_event_in_tx(&tx, &in_tx_draft("ev-intx-rollback", "wi-intx-rollback"))
            .await
            .expect("in-tx append should succeed inside the tx");
        // Caller decides to roll back — e.g. a later step in the composed
        // transaction failed. The event + refs + projection must vanish together.
        tx.rollback().await.expect("caller rolls back");

        let n: i64 = client
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE id = 'ev-intx-rollback'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(n, 0, "rolled-back event must be absent");
        let w: i64 = client
            .query_one(
                "SELECT count(*) FROM workitems WHERE id = 'wi-intx-rollback'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            w, 0,
            "projection written in the rolled-back tx must be absent"
        );
    }
}
