//! P0 step 6 (Nimbus half) — the DB/MCP layer for governed dispatch (G19).
//!
//! Five tools, wired onto the merged step-4 crate (extends, never forks):
//!   - `enqueue_dispatch`   (issuer: Aegis / Lyra)
//!   - `get_next_dispatch`  (the seat, when idle)
//!   - `complete_dispatch`  (the seat, when done — or when it must report failure)
//!   - `cancel_dispatch`    (the issuer, withdrawing non-terminal work)
//!   - `request_acceptance` (the seat, handing a finished delivery to the D2 gate)
//!
//! Every status transition is an EVENT appended in the **same transaction** as
//! the `dispatches` row change (contract rule 2), via
//! [`seatloom_core::db::append_event_in_tx`] against a transaction this layer
//! opens and commits — there is no second append implementation. Readiness is
//! an EXISTENCE query over `dispatch_dependencies` (no FK on `dep_value`, by
//! Onyx's `012` design). The pull decision, cycle, and stall are computed by the
//! pure [`crate::scheduler`] core.
//!
//! `013` (terminal/failure states) and `014` (the D2 acceptance gate) extend the
//! layer: `complete_dispatch` gains an `outcome` argument (done | failed, with
//! the retry budget on the row), `cancel_dispatch` is the issuer-side
//! withdrawal, and `request_acceptance` creates the `acceptances` row that the
//! Tauri accept/reject commands later decide. Each new transition has exactly
//! one event type (Lyra sign-off 2026-10-09), so the projection stays
//! replayable.
//!
//! # Non-forgeable seat identity (packet §1.1)
//!
//! `get_next_dispatch` / `complete_dispatch` resolve the calling seat from the
//! server's own `SEATLOOM_ACTOR_REF` env (the per-seat launch), **never** from a
//! caller parameter — the tools expose no `seat_id` input, so there is nothing
//! to spoof. The identity is pinned to the process the seat talks to, not to the
//! message it sends.

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::Value;

use seatloom_core::db::event_domain::{EventType, RefRole, RefType};
use seatloom_core::db::write_api::{append_event_in_tx, EventDraft, ObjectRefDraft};

use crate::scheduler::{
    classify_stall, detect_cycle, DepKind, Dependency, DispatchStatus, DispatchView, PullDecision,
};

/// Env var naming the seat this server instance acts as. Reused from step 4.
pub const ACTOR_REF_ENV: &str = "SEATLOOM_ACTOR_REF";
/// Env var naming the default project when a call omits `project_id`.
pub const PROJECT_ID_ENV: &str = "SEATLOOM_PROJECT_ID";

// =============================================================================
// Tool argument shapes
// =============================================================================

/// `enqueue_dispatch` — called by the issuer. `target_seat_id` IS a parameter
/// here (the issuer names who must act); identity-forgery protection applies to
/// the *pull* side, not to issuance (an issuer legitimately targets other seats).
#[derive(Debug, Deserialize)]
pub struct EnqueueArgs {
    pub id: Option<String>,
    pub target_seat_id: String,
    pub title: String,
    pub payload: Value,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<DepArg>,
    /// Retry budget for this dispatch (`013`): how many FAILED attempts are
    /// re-opened to `ready` before the failure becomes terminal. Omitted / `0`
    /// means the first failure is terminal (the pre-013 behaviour).
    #[serde(default)]
    pub max_retries: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DepArg {
    pub dep_kind: String,
    pub dep_value: String,
}

/// `get_next_dispatch` and `complete_dispatch` take NO seat parameter — identity
/// comes from the server env. `complete_dispatch` names the dispatch id it is
/// finishing; the write still checks it is the resolved seat's in_flight item.
#[derive(Debug, Deserialize)]
pub struct CompleteArgs {
    pub dispatch_id: String,
    /// `"done"` (the default, fully backward compatible) or `"failed"`. On
    /// `"failed"` the retry budget on the row decides between a re-open
    /// (`SeatDispatchRetried`, in_flight -> ready) and a terminal failure
    /// (`SeatDispatchFailed`, in_flight -> failed) — `013`.
    #[serde(default)]
    pub outcome: Option<String>,
    /// Optional failure reason; carried verbatim into the retry/failure event
    /// payload so the ledger records WHY an attempt ended.
    #[serde(default)]
    pub reason: Option<String>,
}

/// `cancel_dispatch` — the issuer withdraws non-terminal work (`013`). Takes no
/// seat parameter: the issuer identity comes from the server env, and the write
/// is guarded on `issued_by = <resolved actor_ref>`.
#[derive(Debug, Deserialize)]
pub struct CancelArgs {
    pub id: String,
}

/// `request_acceptance` — the seat hands a COMPLETED delivery to the D2
/// acceptance gate (`014`). The seat never merges its own work; this creates the
/// `acceptances` row the operator later accepts or rejects.
#[derive(Debug, Deserialize)]
pub struct RequestAcceptanceArgs {
    pub dispatch_id: String,
    /// The delivery branch to be merged on accept, e.g. `packet/<seat>/<id>`.
    pub branch: String,
    /// The one-step verification the operator performs: "do A, see B".
    pub how_to_verify: String,
    /// The Flux green-verdict reference (a `ReviewVerdictIssued` event or
    /// artifact id). Must exist before a request is permitted.
    pub flux_verdict_ref: String,
}

/// A rejection in the dispatch layer, rendered verbatim into the MCP tool error.
#[derive(Debug)]
pub enum DispatchRejection {
    MissingActorRef,
    MissingProjectId,
    UnknownDepKind {
        value: String,
    },
    PriorityNotI32 {
        priority: i64,
    },
    NotYourInFlight {
        dispatch_id: String,
        seat: String,
    },
    /// `complete_dispatch`'s `outcome` is neither "done" nor "failed".
    UnknownOutcome {
        value: String,
    },
    /// `enqueue_dispatch`'s `max_retries` is negative or does not fit in i32 —
    /// the budget is a hard ceiling, and a nonsense ceiling is rejected, never
    /// silently clamped (`013`).
    InvalidMaxRetries {
        max_retries: i64,
    },
    /// `cancel_dispatch`: the dispatch is not a non-terminal row issued by the
    /// resolved actor (unknown, terminal, or another issuer's).
    NotCancellable {
        dispatch_id: String,
        actor: String,
    },
    /// `request_acceptance`: the dispatch is not a COMPLETED dispatch owned by
    /// the calling seat (`014`).
    NotCompleted {
        dispatch_id: String,
        seat: String,
    },
    /// `request_acceptance`: the Flux verdict reference names no existing event
    /// or artifact — the merge criterion's first half is unproven (`014`).
    UnknownFluxVerdict {
        flux_verdict_ref: String,
    },
    /// `request_acceptance`: the dispatch already has a pending acceptance.
    AcceptancePending {
        dispatch_id: String,
    },
    Db(String),
}

impl std::fmt::Display for DispatchRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchRejection::MissingActorRef => write!(
                f,
                "dispatch rejected: {ACTOR_REF_ENV} is not set; the server must be launched per seat with its own actor ref (identity is never taken from a caller parameter)"
            ),
            DispatchRejection::MissingProjectId => write!(
                f,
                "dispatch rejected: project_id is required (pass it or set {PROJECT_ID_ENV}); a project is never fabricated"
            ),
            DispatchRejection::UnknownDepKind { value } => write!(
                f,
                "dispatch rejected: dep_kind {value:?} is not one of event|artifact|dispatch"
            ),
            DispatchRejection::PriorityNotI32 { priority } => {
                write!(f, "dispatch rejected: priority {priority} does not fit in i32")
            }
            DispatchRejection::NotYourInFlight { dispatch_id, seat } => write!(
                f,
                "dispatch rejected: {dispatch_id:?} is not an in_flight dispatch owned by {seat:?}"
            ),
            DispatchRejection::UnknownOutcome { value } => write!(
                f,
                "complete_dispatch rejected: outcome {value:?} is not one of done|failed"
            ),
            DispatchRejection::InvalidMaxRetries { max_retries } => write!(
                f,
                "enqueue_dispatch rejected: max_retries {max_retries} is invalid; the retry budget is a hard ceiling and must be a non-negative integer that fits in i32 (013)"
            ),
            DispatchRejection::NotCancellable { dispatch_id, actor } => write!(
                f,
                "cancel rejected: {dispatch_id:?} is not a non-terminal dispatch issued by {actor:?} (only the issuer may cancel, and only ready|blocked|in_flight)"
            ),
            DispatchRejection::NotCompleted { dispatch_id, seat } => write!(
                f,
                "request_acceptance rejected: {dispatch_id:?} is not a completed dispatch owned by {seat:?} (013/014: only a done dispatch of the calling seat may be handed to the acceptance gate)"
            ),
            DispatchRejection::UnknownFluxVerdict { flux_verdict_ref } => write!(
                f,
                "request_acceptance rejected: flux_verdict_ref {flux_verdict_ref:?} names no existing event or artifact — the Flux green verdict is the merge criterion's first half and must exist"
            ),
            DispatchRejection::AcceptancePending { dispatch_id } => write!(
                f,
                "request_acceptance rejected: {dispatch_id:?} already has a pending acceptance (one at a time; await the operator's decision)"
            ),
            DispatchRejection::Db(e) => write!(f, "dispatch rejected: {e}"),
        }
    }
}

type Db = tokio_postgres::Error;

/// Render a DB error with its full source chain. `tokio_postgres::Error`'s
/// `Display` is the near-useless string "db error"; the SQLSTATE, constraint
/// name, and server message live in its source chain, so walk it — a rolled-back
/// transaction is only diagnosable if the cause is reported.
fn db<E: std::error::Error + 'static>(e: E) -> DispatchRejection {
    let mut parts = vec![e.to_string()];
    let mut cur = e.source();
    while let Some(src) = cur {
        parts.push(src.to_string());
        if let Some(pg) = src.downcast_ref::<tokio_postgres::error::DbError>() {
            parts.push(format!(
                "sqlstate={} constraint={:?} table={:?}",
                pg.code().code(),
                pg.constraint(),
                pg.table()
            ));
        }
        cur = src.source();
    }
    DispatchRejection::Db(parts.join(" | "))
}

// =============================================================================
// Identity + project resolution (never fabricated)
// =============================================================================

/// The resolved seat this server acts as, as both the `seats.id` (for
/// `target_seat_id` comparisons) and the `actor_ref` (for event actor refs).
struct SeatIdentity {
    seat_id: String,
    actor_ref: String,
}

fn resolve_seat() -> Result<SeatIdentity, DispatchRejection> {
    let raw = std::env::var(ACTOR_REF_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or(DispatchRejection::MissingActorRef)?;
    // SEATLOOM_ACTOR_REF is an actor_ref like "seat:seat-nimbus-001"; the seat id
    // is the part after the "seat:" prefix. A bare id is also accepted.
    let seat_id = raw.strip_prefix("seat:").unwrap_or(&raw).to_string();
    Ok(SeatIdentity {
        seat_id,
        actor_ref: raw,
    })
}

fn resolve_project(call: Option<String>) -> Result<String, DispatchRejection> {
    if let Some(v) = call {
        if !v.trim().is_empty() {
            return Ok(v);
        }
    }
    std::env::var(PROJECT_ID_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or(DispatchRejection::MissingProjectId)
}

// =============================================================================
// Readiness by existence query (contract: READY iff every dep referent EXISTS)
// =============================================================================

/// Does the referent of `(dep_kind, dep_value)` exist? No FK is used — Onyx's
/// `012` deliberately has none on `dep_value`; a missing referent reads as
/// unsatisfied (→ blocked), never an error.
async fn dep_satisfied(
    tx: &tokio_postgres::Transaction<'_>,
    dep: &Dependency,
) -> Result<bool, DispatchRejection> {
    let sql = match dep.dep_kind {
        DepKind::Event => "SELECT 1 FROM canonical_events WHERE id = $1",
        DepKind::Artifact => "SELECT 1 FROM artifacts WHERE id = $1",
        DepKind::Dispatch => "SELECT 1 FROM dispatches WHERE id = $1",
    };
    let rows = tx.query(sql, &[&dep.dep_value]).await.map_err(db)?;
    Ok(!rows.is_empty())
}

/// Compute the live status for a row given its stored status and its deps.
/// `done`, `failed`, `cancelled` (terminal, `013`) and `in_flight` (active) are
/// returned as stored; a non-terminal row is `ready` iff every dependency is
/// satisfied, else `blocked`.
///
/// The terminal states MUST sit in the "return as stored" arm: recomputing them
/// would flip a `failed`/`cancelled` row whose deps happen to all exist back to
/// `ready` — a silent rewrite of a recorded fact (the exact anti-pattern `013`
/// added them to prevent). Flagged by Onyx's design §2.1 as a Nimbus-lane
/// correctness fix; landed here.
async fn live_status(
    tx: &tokio_postgres::Transaction<'_>,
    stored: DispatchStatus,
    deps: &[Dependency],
) -> Result<DispatchStatus, DispatchRejection> {
    match stored {
        DispatchStatus::Done
        | DispatchStatus::InFlight
        | DispatchStatus::Failed
        | DispatchStatus::Cancelled => Ok(stored),
        DispatchStatus::Ready | DispatchStatus::Blocked => {
            for d in deps {
                if !dep_satisfied(tx, d).await? {
                    return Ok(DispatchStatus::Blocked);
                }
            }
            Ok(DispatchStatus::Ready)
        }
    }
}

/// Load one project's dispatches (all seats — the cycle check is project-wide),
/// each with its dependencies and its LIVE status computed by existence query.
async fn load_views(
    tx: &tokio_postgres::Transaction<'_>,
    project_id: &str,
) -> Result<Vec<DispatchView>, DispatchRejection> {
    let rows = tx
        .query(
            "SELECT id, target_seat_id, status, priority, issued_at \
               FROM dispatches WHERE project_id = $1",
            &[&project_id],
        )
        .await
        .map_err(db)?;

    let mut views = Vec::with_capacity(rows.len());
    for row in &rows {
        let id: String = row.get(0);
        let target_seat_id: String = row.get(1);
        let status_raw: String = row.get(2);
        let priority: i32 = row.get(3);
        let issued_at: DateTime<Utc> = row.get(4);
        let stored = DispatchStatus::parse(&status_raw)
            .ok_or_else(|| DispatchRejection::Db(format!("unknown status {status_raw:?} in DB")))?;

        let dep_rows = tx
            .query(
                "SELECT dep_kind, dep_value FROM dispatch_dependencies WHERE dispatch_id = $1",
                &[&id],
            )
            .await
            .map_err(db)?;
        let mut deps = Vec::with_capacity(dep_rows.len());
        for dr in &dep_rows {
            let kind_raw: String = dr.get(0);
            let dep_kind = DepKind::parse(&kind_raw).ok_or_else(|| {
                DispatchRejection::Db(format!("unknown dep_kind {kind_raw:?} in DB"))
            })?;
            deps.push(Dependency {
                dep_kind,
                dep_value: dr.get(1),
            });
        }

        let status = live_status(tx, stored, &deps).await?;
        views.push(DispatchView {
            id,
            target_seat_id,
            status,
            priority,
            issued_at: issued_at.to_rfc3339(),
            dependencies: deps,
        });
    }
    Ok(views)
}

// =============================================================================
// Event drafts for the three transitions
// =============================================================================

/// Build the lifecycle event for a dispatch transition. `subject = dispatch:<id>`,
/// `actor = seat:<actor_ref>`; schema_version=1 explicit; role explicit; exactly
/// one subject; turn_seq None (P0).
fn lifecycle_event(
    event_type: EventType,
    dispatch_id: &str,
    actor_ref: &str,
    project_id: &str,
) -> EventDraft {
    lifecycle_event_with_payload(
        event_type,
        dispatch_id,
        actor_ref,
        project_id,
        serde_json::json!({ "dispatch_id": dispatch_id }),
    )
}

/// [`lifecycle_event`] with an explicit payload — used where the transition
/// carries facts the projection cannot hold (a retried row is `ready`, so the
/// failed attempt's reason/budget lives only in the event; `013`).
fn lifecycle_event_with_payload(
    event_type: EventType,
    dispatch_id: &str,
    actor_ref: &str,
    project_id: &str,
    payload: serde_json::Value,
) -> EventDraft {
    EventDraft {
        event_id: format!("ev-{}", nanoid::nanoid!()),
        event_type,
        schema_version: 1,
        occurred_at: Utc::now(),
        actor_ref: actor_ref.to_string(),
        project_id: project_id.to_string(),
        turn_seq: None,
        payload,
        refs: vec![
            ObjectRefDraft::subject(RefType::Dispatch, dispatch_id),
            ObjectRefDraft::new(RefType::Seat, seat_id_of(actor_ref), RefRole::Actor),
        ],
    }
}

fn seat_id_of(actor_ref: &str) -> String {
    actor_ref
        .strip_prefix("seat:")
        .unwrap_or(actor_ref)
        .to_string()
}

// =============================================================================
// Tool 1 — enqueue_dispatch
// =============================================================================

/// Insert a `dispatches` row + its `dispatch_dependencies`, compute its initial
/// status by existence query, and append `SeatDispatchIssued` — all in one
/// transaction.
pub async fn enqueue(pool: &Pool, args: EnqueueArgs) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id)?;
    let priority: i32 = match args.priority {
        None => 0,
        Some(p) => {
            i32::try_from(p).map_err(|_| DispatchRejection::PriorityNotI32 { priority: p })?
        }
    };
    // Parse dependencies up front so an unknown dep_kind is rejected before any write.
    let mut deps = Vec::with_capacity(args.dependencies.len());
    for d in &args.dependencies {
        let dep_kind =
            DepKind::parse(&d.dep_kind).ok_or_else(|| DispatchRejection::UnknownDepKind {
                value: d.dep_kind.clone(),
            })?;
        deps.push(Dependency {
            dep_kind,
            dep_value: d.dep_value.clone(),
        });
    }
    let dispatch_id = match args.id {
        Some(s) if !s.trim().is_empty() => s,
        _ => format!("disp-{}", nanoid::nanoid!()),
    };
    // Retry budget (`013`): a hard ceiling, never NULL-means-unlimited. 0 (the
    // default) keeps the pre-013 behaviour — the first failure is terminal.
    // A negative or oversized budget is REJECTED, never silently clamped: a
    // silent fallback is exactly the defect the hand-written boundaries avoid.
    let max_retries: i32 = match args.max_retries {
        None => 0,
        Some(m) => i32::try_from(m)
            .ok()
            .filter(|m| *m >= 0)
            .ok_or(DispatchRejection::InvalidMaxRetries { max_retries: m })?,
    };

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // Initial status: ready iff every dep is already satisfied, else blocked.
    let mut status = DispatchStatus::Ready;
    for d in &deps {
        if !dep_satisfied(&tx, d).await? {
            status = DispatchStatus::Blocked;
            break;
        }
    }

    tx.execute(
        "INSERT INTO dispatches \
         (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at, max_retries) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)",
        &[
            &dispatch_id,
            &args.target_seat_id,
            &identity.actor_ref,
            &project_id,
            &args.title,
            &args.payload,
            &status.as_str(),
            &priority,
            &max_retries,
        ],
    )
    .await
    .map_err(db)?;

    for d in &deps {
        tx.execute(
            "INSERT INTO dispatch_dependencies (dispatch_id, dep_kind, dep_value) VALUES ($1, $2, $3)",
            &[&dispatch_id, &d.dep_kind.as_str(), &d.dep_value],
        )
        .await
        .map_err(db)?;
    }

    // Same transaction: the SeatDispatchIssued event (the row is a projection).
    let draft = lifecycle_event(
        EventType::SeatDispatchIssued,
        &dispatch_id,
        &identity.actor_ref,
        &project_id,
    );
    append_event_in_tx(&tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;

    // Wake the target seat (event-driven, zero-poll): the waker daemon LISTENs
    // on 'dispatch_ready' and nudges the seat iff it is idle. Fires on COMMIT, so
    // the dispatch row is already visible when the notification is delivered.
    // Only notify for a READY dispatch — a blocked one has nothing to pull yet.
    if status == DispatchStatus::Ready {
        tx.execute(
            "SELECT pg_notify('dispatch_ready', $1)",
            &[&args.target_seat_id],
        )
        .await
        .map_err(db)?;
    }

    tx.commit().await.map_err(db)?;
    Ok(format!(
        "enqueued {dispatch_id} (status={})",
        status.as_str()
    ))
}

// =============================================================================
// Tool 2 — get_next_dispatch
// =============================================================================

/// Fetch a dispatch's task CONTENT (title + payload JSON) so `get_next_dispatch`
/// returns the WORK, not just the id. Without this the seat receives a dispatch
/// id with no instructions and cannot know what to do.
async fn fetch_content(
    tx: &tokio_postgres::Transaction<'_>,
    dispatch_id: &str,
) -> Result<(String, String), DispatchRejection> {
    let row = tx
        .query_one(
            "SELECT title, payload FROM dispatches WHERE id = $1",
            &[&dispatch_id],
        )
        .await
        .map_err(db)?;
    let title: String = row.get(0);
    let payload: serde_json::Value = row.get(1);
    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Ok((title, payload_str))
}

/// Resolve the seat, evaluate readiness, pull the next ready/highest-priority
/// item (or return an existing in_flight), mark it in_flight + append
/// `SeatDispatchDelivered` in one transaction. On a cycle or a stall, escalate
/// (SupervisorMessage + daily-log line) and return "nothing ready" rather than
/// hanging.
pub async fn get_next(
    pool: &Pool,
    project_id_arg: Option<String>,
) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(project_id_arg)?;

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // One-in-flight is per seat GLOBALLY: the DB index
    // `uq_dispatches_one_in_flight_per_seat` is on (target_seat_id) with no
    // project scope (contract rule 1). So before considering this project's
    // queue, honour any in_flight the seat already holds in ANY project — hand
    // it back rather than trying to deliver a second one and hitting the index
    // with an opaque rollback. Pull is idle-time, one task at a time, period.
    if let Some(existing) = tx
        .query_opt(
            "SELECT id, project_id FROM dispatches \
               WHERE target_seat_id = $1 AND status = 'in_flight' LIMIT 1",
            &[&identity.seat_id],
        )
        .await
        .map_err(db)?
    {
        let held_id: String = existing.get(0);
        let held_project: String = existing.get(1);
        tx.commit().await.map_err(db)?;
        return Ok(format!(
            "{held_id} (already in_flight for {} in project {held_project}; one in_flight per seat, globally)",
            identity.seat_id
        ));
    }

    let views = load_views(&tx, &project_id).await?;

    // Proactive cycle surfacing (contract rule 5): a structural cycle over
    // `dispatch`->`dispatch` edges is detected independently of existence-based
    // readiness, because existence alone cannot see a ring (both rows exist, so
    // both read ready, yet they depend on each other). If a cycle involves one
    // of THIS seat's dispatches, escalate it — once, deduplicated against an
    // already-emitted SupervisorMessage for the same ring so repeated pulls do
    // not spam the ledger. This does NOT block the pull: genuinely independent
    // ready work for the seat still flows (an unrelated cycle elsewhere is not
    // this seat's concern and is not even considered here).
    let seat_cycle = detect_cycle(&views).filter(|c| {
        c.ids.iter().any(|id| {
            views
                .iter()
                .any(|v| &v.id == id && v.target_seat_id == identity.seat_id)
        })
    });
    if let Some(cycle) = &seat_cycle {
        if !cycle_already_escalated(&tx, &project_id, &cycle.ids).await? {
            let offending = cycle.ids.first().cloned().unwrap_or_default();
            escalate(
                &tx,
                &identity,
                &project_id,
                &offending,
                &format!("dependency cycle detected: {}", cycle.ids.join(" -> ")),
            )
            .await?;
        }
    }

    match crate::scheduler::select_next(&identity.seat_id, &views) {
        PullDecision::AlreadyInFlight { dispatch_id } => {
            let (title, payload) = fetch_content(&tx, &dispatch_id).await?;
            tx.commit().await.map_err(db)?;
            Ok(format!(
                "delivered {dispatch_id} to {} (already in_flight — resuming this task)\n\
                 title: {title}\npayload: {payload}\n\
                 Do the work described in payload, then call complete_dispatch with dispatch_id={dispatch_id}.",
                identity.seat_id
            ))
        }
        PullDecision::Deliver { dispatch_id } => {
            // The view's status is LIVE readiness (recomputed by existence query
            // in load_views), so a row that is stored 'blocked' but whose deps
            // are now all satisfied reads 'ready' here. Transition from either
            // stored non-terminal state; the guard excludes 'in_flight'/'done'
            // so a raced double-delivery still fails (and the partial unique
            // index is the ultimate backstop).
            let n = tx
                .execute(
                    "UPDATE dispatches SET status = 'in_flight', started_at = NOW() \
                       WHERE id = $1 AND status IN ('ready', 'blocked')",
                    &[&dispatch_id],
                )
                .await
                .map_err(db)?;
            if n != 1 {
                return Err(DispatchRejection::Db(format!(
                    "dispatch {dispatch_id} was no longer pullable at delivery (raced to in_flight/done)"
                )));
            }
            let draft = lifecycle_event(
                EventType::SeatDispatchDelivered,
                &dispatch_id,
                &identity.actor_ref,
                &project_id,
            );
            append_event_in_tx(&tx, &draft)
                .await
                .map_err(|e| DispatchRejection::Db(e.to_string()))?;
            let (title, payload) = fetch_content(&tx, &dispatch_id).await?;
            tx.commit().await.map_err(db)?;
            let note = if seat_cycle.is_some() {
                " (note: a dependency cycle involving this seat was escalated)"
            } else {
                ""
            };
            Ok(format!(
                "delivered {dispatch_id} to {}{note}\n\
                 title: {title}\npayload: {payload}\n\
                 Do the work described in payload, then call complete_dispatch with dispatch_id={dispatch_id}.",
                identity.seat_id
            ))
        }
        PullDecision::Nothing { stall } => {
            if !stall {
                tx.commit().await.map_err(db)?;
                return Ok(format!(
                    "nothing ready for {} (idle, not a stall)",
                    identity.seat_id
                ));
            }
            debug_assert!(classify_stall(&identity.seat_id, &views));
            // A stall: nothing ready, nothing in_flight, something blocked. If the
            // cause is a cycle this seat is in, it was already escalated above;
            // name it. Otherwise it is a plain blocked-dependency stall — escalate
            // that (contract rule 5: all-blocked stall is surfaced, never hung).
            match &seat_cycle {
                Some(c) => {
                    tx.commit().await.map_err(db)?;
                    Ok(format!(
                        "nothing ready for {}: dependency cycle {} escalated via SupervisorMessage",
                        identity.seat_id,
                        c.ids.join(" -> ")
                    ))
                }
                None => {
                    let offending = views
                        .iter()
                        .find(|v| {
                            v.target_seat_id == identity.seat_id
                                && v.status == DispatchStatus::Blocked
                        })
                        .map(|v| v.id.clone())
                        .unwrap_or_default();
                    escalate(
                        &tx,
                        &identity,
                        &project_id,
                        &offending,
                        &format!(
                            "stall: seat {} has only blocked dispatches and nothing ready",
                            identity.seat_id
                        ),
                    )
                    .await?;
                    tx.commit().await.map_err(db)?;
                    Ok(format!(
                        "nothing ready for {}: stall escalated via SupervisorMessage",
                        identity.seat_id
                    ))
                }
            }
        }
    }
}

/// Has a `SupervisorMessage` already been emitted naming this exact cycle ring?
/// Dedup so repeated pulls over a standing cycle do not spam the ledger. We
/// match on the ring's first id appearing as the subject of a prior escalation
/// in this project.
async fn cycle_already_escalated(
    tx: &tokio_postgres::Transaction<'_>,
    project_id: &str,
    ring: &[String],
) -> Result<bool, DispatchRejection> {
    let Some(first) = ring.first() else {
        return Ok(true);
    };
    let rows = tx
        .query(
            "SELECT 1 FROM canonical_events e \
               JOIN event_object_refs r ON r.event_id = e.id AND r.role = 'subject' \
              WHERE e.project_id = $1 AND e.event_type = 'SupervisorMessage' \
                AND r.ref_type = 'dispatch' AND r.ref_id = $2 LIMIT 1",
            &[&project_id, &first],
        )
        .await
        .map_err(db)?;
    Ok(!rows.is_empty())
}

// =============================================================================
// Tool 3 — complete_dispatch (done | failed, with the 013 retry budget)
// =============================================================================

/// Close the resolved seat's in_flight dispatch and append the matching event in
/// one transaction. The seat can only complete its OWN in_flight dispatch
/// (identity from env, status guarded in SQL).
///
/// `outcome` (default `"done"`, fully backward compatible):
///   - `"done"`   -> in_flight -> done + `SeatDispatchCompleted` (unchanged);
///   - `"failed"` -> the row's retry budget decides (`013`):
///       * retry_count < max_retries -> in_flight -> ready, retry_count+1,
///         `SeatDispatchRetried` (the attempt is durably recorded in the ledger
///         even though the row is pullable again; the one-in-flight slot is
///         released, so the same or another seat can pull it);
///       * otherwise -> in_flight -> failed (terminal) + `SeatDispatchFailed`.
///
/// Each outcome maps to EXACTLY ONE event type, so the projection stays
/// replayable (a single "sometimes-retry-sometimes-terminal" event would make
/// the ledger ambiguous — Onyx's design §2.2).
pub async fn complete(pool: &Pool, args: CompleteArgs) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;
    let outcome = args.outcome.as_deref().unwrap_or("done").trim();
    if outcome != "done" && outcome != "failed" {
        return Err(DispatchRejection::UnknownOutcome {
            value: outcome.to_string(),
        });
    }

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // Fetch the project_id + retry budget for the event, and prove ownership +
    // in_flight state atomically by the SELECT's WHERE clause.
    let row = tx
        .query_opt(
            "SELECT project_id, retry_count, max_retries FROM dispatches \
               WHERE id = $1 AND target_seat_id = $2 AND status = 'in_flight'",
            &[&args.dispatch_id, &identity.seat_id],
        )
        .await
        .map_err(db)?;
    let Some(row) = row else {
        return Err(DispatchRejection::NotYourInFlight {
            dispatch_id: args.dispatch_id,
            seat: identity.seat_id,
        });
    };
    let project_id: String = row.get(0);
    let retry_count: i32 = row.get(1);
    let max_retries: i32 = row.get(2);

    // Decide the transition BEFORE writing it. A retry re-opens to plain
    // `ready` — re-pullable by any eligible seat, no stickiness (Lyra's ruling
    // on Onyx's open question: fleet elasticity over preferred-seat notions).
    let (new_status, event_type, retrying) = decide_completion(outcome, retry_count, max_retries);

    let n = if retrying {
        tx.execute(
            "UPDATE dispatches SET status = 'ready', retry_count = retry_count + 1, \
               started_at = NULL \
               WHERE id = $1 AND target_seat_id = $2 AND status = 'in_flight'",
            &[&args.dispatch_id, &identity.seat_id],
        )
        .await
        .map_err(db)?
    } else {
        tx.execute(
            "UPDATE dispatches SET status = $3, completed_at = NOW() \
               WHERE id = $1 AND target_seat_id = $2 AND status = 'in_flight'",
            &[&args.dispatch_id, &identity.seat_id, &new_status],
        )
        .await
        .map_err(db)?
    };
    if n != 1 {
        return Err(DispatchRejection::NotYourInFlight {
            dispatch_id: args.dispatch_id,
            seat: identity.seat_id,
        });
    }

    // One event per transition; the payload carries the retry/failure facts the
    // projection cannot (a retried row is `ready` — the failed attempt lives
    // only in the ledger).
    let payload = match outcome {
        "done" => serde_json::json!({ "dispatch_id": args.dispatch_id }),
        "failed" if retrying => serde_json::json!({
            "dispatch_id": args.dispatch_id,
            "reason": args.reason,
            "retry_count": retry_count + 1,
            "max_retries": max_retries,
        }),
        _ => serde_json::json!({
            "dispatch_id": args.dispatch_id,
            "reason": args.reason,
            "retry_count": retry_count,
            "max_retries": max_retries,
            "retries_exhausted": true,
        }),
    };
    let draft = lifecycle_event_with_payload(
        event_type,
        &args.dispatch_id,
        &identity.actor_ref,
        &project_id,
        payload,
    );
    append_event_in_tx(&tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;

    tx.commit().await.map_err(db)?;
    let summary = match outcome {
        "done" => format!("completed {} for {}", args.dispatch_id, identity.seat_id),
        "failed" if retrying => format!(
            "failed {} for {} — retry budget not exhausted (retry {}/{}); re-opened to ready (any eligible seat may pull)",
            args.dispatch_id,
            identity.seat_id,
            retry_count + 1,
            max_retries
        ),
        _ => format!(
            "failed {} for {} — retries exhausted (retry_count {}, max_retries {}); terminal",
            args.dispatch_id, identity.seat_id, retry_count, max_retries
        ),
    };
    Ok(summary)
}

/// The `013` completion decision, pure so it is testable without a database
/// (the scheduler-core discipline): which transition an
/// `(outcome, retry_count, max_retries)` triple implies.
///
/// Returns `(new_status, event_type, retrying)`. Exactly one event type per
/// transition — a single "sometimes-retry-sometimes-terminal" event would make
/// the ledger ambiguous to replay (Onyx's design §2.2). `outcome` is assumed
/// pre-validated as `"done"` | `"failed"` (the tool rejects anything else).
fn decide_completion(
    outcome: &str,
    retry_count: i32,
    max_retries: i32,
) -> (&'static str, EventType, bool) {
    match outcome {
        "done" => ("done", EventType::SeatDispatchCompleted, false),
        // Budget remaining: re-open to plain ready (any eligible seat may pull).
        "failed" if retry_count < max_retries => ("ready", EventType::SeatDispatchRetried, true),
        // Budget exhausted (or none was ever granted): terminal failure.
        _ => ("failed", EventType::SeatDispatchFailed, false),
    }
}

// =============================================================================
// Tool 4 — cancel_dispatch (issuer side, 013)
// =============================================================================

/// Withdraw a non-terminal dispatch: `ready | blocked | in_flight -> cancelled`
/// (terminal) + `SeatDispatchCancelled`, in one transaction.
///
/// The actor is the ISSUER, not the seat: a seat either does the work (`done`)
/// or fails it (`failed`); withdrawal is the issuer's act — so the tool is
/// guarded on `issued_by = <resolved actor_ref>` and lives beside (not inside)
/// `complete_dispatch`. Cancelling an `in_flight` dispatch releases the
/// one-in-flight slot: the seat's in-progress work is abandoned, which is the
/// intended "withdraw the work" semantics.
pub async fn cancel(pool: &Pool, args: CancelArgs) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // Guarded fetch: the dispatch must exist, be issued by the resolved actor,
    // and be non-terminal. Terminal rows are immutable (013), so a done/failed/
    // cancelled dispatch can never be cancelled — recorded facts stand.
    let project_id: Option<String> = tx
        .query_opt(
            "SELECT project_id FROM dispatches \
               WHERE id = $1 AND issued_by = $2 \
                 AND status IN ('ready', 'blocked', 'in_flight')",
            &[&args.id, &identity.actor_ref],
        )
        .await
        .map_err(db)?
        .map(|r| r.get(0));
    let Some(project_id) = project_id else {
        return Err(DispatchRejection::NotCancellable {
            dispatch_id: args.id,
            actor: identity.actor_ref,
        });
    };

    let n = tx
        .execute(
            "UPDATE dispatches SET status = 'cancelled', completed_at = NOW() \
               WHERE id = $1 AND issued_by = $2 \
                 AND status IN ('ready', 'blocked', 'in_flight')",
            &[&args.id, &identity.actor_ref],
        )
        .await
        .map_err(db)?;
    if n != 1 {
        return Err(DispatchRejection::NotCancellable {
            dispatch_id: args.id,
            actor: identity.actor_ref,
        });
    }

    let draft = lifecycle_event(
        EventType::SeatDispatchCancelled,
        &args.id,
        &identity.actor_ref,
        &project_id,
    );
    append_event_in_tx(&tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;

    tx.commit().await.map_err(db)?;
    Ok(format!(
        "cancelled {} (issued by {})",
        args.id, identity.actor_ref
    ))
}

// =============================================================================
// Tool 5 — request_acceptance (seat side, 014 — the D2 acceptance gate)
// =============================================================================

/// Hand a COMPLETED delivery to the acceptance gate: insert the `acceptances`
/// row + append `AcceptanceRequested` in ONE transaction (contract rule 2).
///
/// The seat NEVER merges its own work (Lyra's binding merge rule): this only
/// records the request. The operator later accepts (merge into track, executed
/// by the Tauri app as the user's agent) or rejects (auto-enqueued rework).
///
/// Guards, all checked inside the transaction:
///   1. the dispatch belongs to the calling seat AND is `done`
///      (a `SeatDispatchCompleted` landed — the row is its projection);
///   2. `flux_verdict_ref` names an existing event or artifact (the merge
///      criterion's first half — Flux green);
///   3. the dispatch has no PENDING acceptance already (the partial unique
///      index is the backstop; this is the clean error).
pub async fn request_acceptance(
    pool: &Pool,
    args: RequestAcceptanceArgs,
) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // Guard 1 — the caller's own COMPLETED dispatch.
    let project_id: Option<String> = tx
        .query_opt(
            "SELECT project_id FROM dispatches \
               WHERE id = $1 AND target_seat_id = $2 AND status = 'done'",
            &[&args.dispatch_id, &identity.seat_id],
        )
        .await
        .map_err(db)?
        .map(|r| r.get(0));
    let Some(project_id) = project_id else {
        return Err(DispatchRejection::NotCompleted {
            dispatch_id: args.dispatch_id,
            seat: identity.seat_id,
        });
    };

    // Guard 2 — the Flux verdict referent exists (event OR artifact; the ref is
    // polymorphic TEXT with no FK, exactly like dispatch_dependencies.dep_value).
    let verdict_exists: bool = tx
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1) \
                OR EXISTS (SELECT 1 FROM artifacts WHERE id = $1)",
            &[&args.flux_verdict_ref],
        )
        .await
        .map_err(db)?
        .get(0);
    if !verdict_exists {
        return Err(DispatchRejection::UnknownFluxVerdict {
            flux_verdict_ref: args.flux_verdict_ref,
        });
    }

    // Guard 3 — at most one pending acceptance per dispatch.
    let pending = tx
        .query_opt(
            "SELECT id FROM acceptances WHERE dispatch_id = $1 AND status = 'pending'",
            &[&args.dispatch_id],
        )
        .await
        .map_err(db)?;
    if pending.is_some() {
        return Err(DispatchRejection::AcceptancePending {
            dispatch_id: args.dispatch_id,
        });
    }

    // Deterministic id: one acceptance per dispatch, ever (a rework gets a NEW
    // dispatch id, so the lineage stays one-to-one and self-documenting).
    let acceptance_id = format!("acc-{}", args.dispatch_id);
    tx.execute(
        "INSERT INTO acceptances \
         (id, dispatch_id, project_id, branch, base, how_to_verify, flux_verdict_ref, status) \
         VALUES ($1, $2, $3, $4, 'track', $5, $6, 'pending')",
        &[
            &acceptance_id,
            &args.dispatch_id,
            &project_id,
            &args.branch,
            &args.how_to_verify,
            &args.flux_verdict_ref,
        ],
    )
    .await
    .map_err(db)?;

    // Same transaction: the request event. Subject = the acceptance (its own
    // object, `014`); actor = the seat; context = the dispatch it gates.
    let draft = EventDraft {
        event_id: format!("ev-{}", nanoid::nanoid!()),
        event_type: EventType::AcceptanceRequested,
        schema_version: 1,
        occurred_at: Utc::now(),
        actor_ref: identity.actor_ref.clone(),
        project_id,
        turn_seq: None,
        payload: serde_json::json!({
            "dispatch_id": args.dispatch_id,
            "branch": args.branch,
            "how_to_verify": args.how_to_verify,
            "flux_verdict_ref": args.flux_verdict_ref,
        }),
        refs: vec![
            ObjectRefDraft::subject(RefType::Acceptance, &acceptance_id),
            ObjectRefDraft::new(
                RefType::Seat,
                seat_id_of(&identity.actor_ref),
                RefRole::Actor,
            ),
            ObjectRefDraft::new(RefType::Dispatch, &args.dispatch_id, RefRole::Context),
        ],
    };
    append_event_in_tx(&tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;

    tx.commit().await.map_err(db)?;
    Ok(format!(
        "requested acceptance {acceptance_id} for {} (branch {}, flux verdict {}) — pending operator decision",
        args.dispatch_id, args.branch, args.flux_verdict_ref
    ))
}

// =============================================================================
// Escalation — SupervisorMessage + daily-log line, never hang
// =============================================================================

/// Emit a `SupervisorMessage` (actor_ref = the scheduler, refs = [subject = the
/// offending dispatch, actor = the scheduler]) inside `tx`, and park a line in
/// `docs/coordination/memory/YYYY-MM-DD.md`. The daily-log write is a best-effort
/// side effect outside the transaction's atomicity (it is a file, not a row);
/// its failure never blocks or rolls back the escalation event.
async fn escalate(
    tx: &tokio_postgres::Transaction<'_>,
    identity: &SeatIdentity,
    project_id: &str,
    offending_dispatch: &str,
    reason: &str,
) -> Result<(), DispatchRejection> {
    let draft = EventDraft {
        event_id: format!("ev-{}", nanoid::nanoid!()),
        event_type: EventType::SupervisorMessage,
        schema_version: 1,
        occurred_at: Utc::now(),
        actor_ref: identity.actor_ref.clone(),
        project_id: project_id.to_string(),
        turn_seq: None,
        payload: serde_json::json!({ "reason": reason, "scheduler": identity.actor_ref }),
        refs: vec![
            ObjectRefDraft::subject(RefType::Dispatch, offending_dispatch),
            ObjectRefDraft::new(RefType::Seat, identity.seat_id.clone(), RefRole::Actor),
        ],
    };
    append_event_in_tx(tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;
    park_daily_log_line(reason, offending_dispatch);
    Ok(())
}

/// Append a line to today's daily log. Best-effort: a missing repo root or a
/// write error is logged to stderr and swallowed — escalation must never hang or
/// fail because a file could not be touched.
fn park_daily_log_line(reason: &str, offending_dispatch: &str) {
    let today = Utc::now().format("%Y-%m-%d");
    let line = format!(
        "\n- **[scheduler escalation] {}** — {reason} (offending dispatch `{offending_dispatch}`, SupervisorMessage emitted). (auto, governed dispatch G19)\n",
        today
    );
    let path = format!("docs/coordination/memory/{today}.md");
    if let Err(e) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        })
    {
        eprintln!("seatloom-mcp-emit: could not park escalation in {path}: {e} (continuing)");
    }
}

// `Db` type alias is only used by `db()`; silence the unused warning if a future
// refactor drops the last direct reference.
#[allow(dead_code)]
type _KeepDb = Db;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_id_strips_the_actor_prefix() {
        assert_eq!(seat_id_of("seat:seat-nimbus-001"), "seat-nimbus-001");
        assert_eq!(seat_id_of("seat-nimbus-001"), "seat-nimbus-001");
    }

    /// `013` backward compatibility: a caller that sends only `dispatch_id`
    /// (the pre-013 shape, still what every live seat sends) parses with
    /// `outcome = None` -> `"done"`.
    #[test]
    fn complete_args_default_to_the_done_outcome() {
        let a: CompleteArgs =
            serde_json::from_value(serde_json::json!({ "dispatch_id": "disp-1" })).unwrap();
        assert_eq!(a.outcome, None);
        assert_eq!(a.reason, None);
        let b: CompleteArgs = serde_json::from_value(serde_json::json!({
            "dispatch_id": "disp-1", "outcome": "failed", "reason": "flaky dep"
        }))
        .unwrap();
        assert_eq!(b.outcome.as_deref(), Some("failed"));
        assert_eq!(b.reason.as_deref(), Some("flaky dep"));
    }

    /// `enqueue_dispatch` without `max_retries` (the pre-013 shape) parses;
    /// the budget defaults to 0 = first failure terminal.
    #[test]
    fn enqueue_args_max_retries_defaults_to_none() {
        let a: EnqueueArgs = serde_json::from_value(serde_json::json!({
            "target_seat_id": "seat-nimbus-001", "title": "t", "payload": {}
        }))
        .unwrap();
        assert_eq!(a.max_retries, None);
    }

    /// The `013` decision table: each (outcome, budget) maps to exactly one
    /// transition, and the three transitions carry three DISTINCT event types
    /// (the replayability property Onyx's design §2.2 insists on).
    #[test]
    fn decide_completion_maps_each_outcome_to_exactly_one_transition() {
        // done is unchanged and never a retry.
        assert_eq!(
            decide_completion("done", 0, 3),
            ("done", EventType::SeatDispatchCompleted, false)
        );
        // failed WITH budget remaining re-opens to plain ready (no stickiness).
        assert_eq!(
            decide_completion("failed", 0, 3),
            ("ready", EventType::SeatDispatchRetried, true)
        );
        assert_eq!(
            decide_completion("failed", 2, 3),
            ("ready", EventType::SeatDispatchRetried, true)
        );
        // budget exhausted, or never granted (max_retries = 0) -> terminal.
        assert_eq!(
            decide_completion("failed", 3, 3),
            ("failed", EventType::SeatDispatchFailed, false)
        );
        assert_eq!(
            decide_completion("failed", 0, 0),
            ("failed", EventType::SeatDispatchFailed, false)
        );
        // Distinct event types across the three transitions.
        let types = [
            decide_completion("done", 0, 0).1,
            decide_completion("failed", 0, 1).1,
            decide_completion("failed", 1, 1).1,
        ];
        assert_eq!(
            types,
            [
                EventType::SeatDispatchCompleted,
                EventType::SeatDispatchRetried,
                EventType::SeatDispatchFailed,
            ]
        );
    }

    /// `lifecycle_event_with_payload` carries the explicit payload (the retry
    /// facts a `ready` row cannot hold) while keeping the one-subject shape.
    #[test]
    fn lifecycle_event_with_payload_keeps_one_subject_and_carries_the_payload() {
        let d = lifecycle_event_with_payload(
            EventType::SeatDispatchRetried,
            "disp-1",
            "seat:seat-nimbus-001",
            "proj",
            serde_json::json!({ "retry_count": 1, "max_retries": 3 }),
        );
        assert_eq!(d.event_type, EventType::SeatDispatchRetried);
        assert_eq!(d.payload["retry_count"], 1);
        assert_eq!(
            d.refs.iter().filter(|r| r.role == RefRole::Subject).count(),
            1
        );
    }

    #[test]
    fn resolve_project_prefers_call_then_env_then_rejects() {
        std::env::remove_var(PROJECT_ID_ENV);
        assert_eq!(resolve_project(Some("p-call".into())).unwrap(), "p-call");
        assert!(matches!(
            resolve_project(Some("   ".into())),
            Err(DispatchRejection::MissingProjectId)
        ));
        assert!(matches!(
            resolve_project(None),
            Err(DispatchRejection::MissingProjectId)
        ));
    }

    #[test]
    fn lifecycle_event_shapes_subject_and_actor_refs() {
        let d = lifecycle_event(
            EventType::SeatDispatchIssued,
            "disp-1",
            "seat:seat-nimbus-001",
            "proj",
        );
        assert_eq!(d.event_type, EventType::SeatDispatchIssued);
        assert_eq!(d.schema_version, 1);
        assert_eq!(d.turn_seq, None);
        let subj = d.refs.iter().find(|r| r.role == RefRole::Subject).unwrap();
        assert_eq!(subj.ref_type, RefType::Dispatch);
        assert_eq!(subj.ref_id, "disp-1");
        let actor = d.refs.iter().find(|r| r.role == RefRole::Actor).unwrap();
        assert_eq!(actor.ref_type, RefType::Seat);
        assert_eq!(actor.ref_id, "seat-nimbus-001");
        // exactly one subject
        assert_eq!(
            d.refs.iter().filter(|r| r.role == RefRole::Subject).count(),
            1
        );
    }
}
