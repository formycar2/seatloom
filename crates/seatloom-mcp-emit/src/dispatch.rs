//! P0 step 6 (Nimbus half) — the DB/MCP layer for governed dispatch (G19).
//!
//! Five tools, wired onto the merged step-4 crate (extends, never forks):
//!   - `enqueue_dispatch`   (issuer: Aegis / Lyra)
//!   - `get_next_dispatch`  (the seat, when idle)
//!   - `complete_dispatch`  (the seat, when done — or when it must report failure)
//!   - `cancel_dispatch`    (the issuer, withdrawing non-terminal work)
//!   - `request_acceptance` (the seat, handing a finished delivery to the D2 gate)
//!   - `cancel_acceptance` (the seat, pulling back its own pending request)
//!   - `get_acceptance_lineage` (anyone: the pure-read lineage of one card)
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
//! The D2 follow-on (Aegis dispatch `disp-E8BP2vKlOwMHLomm0eX8J`) closes two
//! holes in that gate: the merge BASE is now an explicit, validated input
//! instead of a hardcoded `'track'` that does not exist, and a pending card can
//! be withdrawn by the seat that requested it (`cancel_acceptance`) instead of
//! only ever being accepted or rejected.
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
use serde::{Deserialize, Serialize};
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
/// Env var naming the integration branch a delivery merges into on accept,
/// used when `request_acceptance` is called without an explicit `base`.
///
/// The base used to be the hardcoded literal `'track'`, which is not a branch
/// that exists in this repository (`track/infra-foundation` is) — so the merge
/// only failed at click time. It is now discovered, never invented: the caller
/// names it, or the operator configures it here, and if neither does the
/// request is REFUSED rather than guessing a branch name.
pub const INTEGRATION_BRANCH_ENV: &str = "SEATLOOM_INTEGRATION_BRANCH";

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
    /// The pre-declared id of the verdict event this dispatch is asking for
    /// (Aegis ruling 2026-10-11, part 2). Only meaningful — and only accepted —
    /// when `payload.kind == "verification"`: the tool MINTS one when the caller
    /// omits it and VALIDATES a caller-supplied one, then returns it so the
    /// requester can hang a gated self-dispatch on it. The verifier MUST emit
    /// `ReviewVerdictIssued` with exactly this id; that is what makes the
    /// blocked -> ready trigger wake the requester.
    #[serde(default)]
    pub verdict_event_id: Option<String>,
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
    /// The branch the delivery merges into on accept — the integration branch
    /// the packet branched FROM (here `track/infra-foundation`).
    ///
    /// EXPLICIT, not defaulted: the pre-fix code hardcoded the literal
    /// `'track'`, which is not a branch that exists, so the failure only
    /// surfaced when the operator clicked Accept. Omitted, the value comes from
    /// `SEATLOOM_INTEGRATION_BRANCH`; if neither names it the request is
    /// refused — a branch name is never invented at request time.
    #[serde(default)]
    pub base: Option<String>,
    /// The structured acceptance plan (016, shape v1) — the product-operation
    /// script the operator follows to verify a user-perceivable result:
    /// `scenario`, `steps`, `expected` (+ optional `preconditions`).
    ///
    /// REQUIRED, and validated BEFORE any DB contact (value-domain contract
    /// §2, tool layer): a missing, empty or malformed plan is a typed
    /// rejection, so "no plan -> no card" cannot be bypassed by a caller that
    /// skips the check. Authored by the PO; immutable after the request, same
    /// rule as `how_to_verify`.
    pub acceptance_plan: serde_json::Value,
}

/// `cancel_acceptance` — the seat pulls back its OWN pending acceptance request
/// (D2 follow-on). The delivery is not merged and no rework is created: the card
/// is closed as `cancelled` with the reason recorded, which is how "this card
/// was satisfied by other means" is expressed in the ledger instead of by hand.
#[derive(Debug, Deserialize)]
pub struct CancelAcceptanceArgs {
    /// The acceptance to withdraw (`acc-<dispatch-id>`).
    pub acceptance_id: String,
    /// Why the card is being pulled back — required, one line, recorded
    /// verbatim on the row and in the `AcceptanceCancelled` event payload.
    pub reason: String,
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
    /// `enqueue_dispatch`: `verdict_event_id` was supplied for a payload that is
    /// not a verification request — the pre-declaration belongs to the
    /// verification path only, so accepting it anywhere else would create an id
    /// nothing will ever resolve.
    VerdictIdNotAVerification {
        verdict_event_id: String,
    },
    /// `enqueue_dispatch`: the caller-supplied `verdict_event_id` is not a
    /// well-formed event id.
    VerdictIdMalformed {
        verdict_event_id: String,
    },
    /// `enqueue_dispatch`: the caller-supplied `verdict_event_id` is already in
    /// the ledger — a pre-declared id must be unique, or two verdicts would
    /// share one gate.
    VerdictIdTaken {
        verdict_event_id: String,
    },
    /// `request_acceptance`: neither the `base` argument nor
    /// `SEATLOOM_INTEGRATION_BRANCH` named the merge base. Refusing beats the
    /// old behaviour of hardcoding `'track'` — a branch that does not exist in
    /// this repository, so the merge only failed at click time.
    MissingBase,
    /// `request_acceptance`: the resolved base is not a local branch (or the
    /// repository could not be read). Symmetric with the delivery-branch check
    /// the merge path performs, moved earlier so the error is a request-time
    /// refusal instead of a click-time surprise.
    UnknownBase {
        base: String,
        detail: String,
    },
    /// `request_acceptance` (containment defence): the delivery branch does not
    /// CONTAIN the base — accepting it would merge a branch that never saw the
    /// base's later work. The fix is to sync the branch, not to weaken the gate.
    BranchNotSynced {
        branch: String,
        base: String,
    },
    /// `request_acceptance` (containment defence): the merge would DELETE files
    /// tracked on the base — the failure mode that let an Accept destroy
    /// already-merged work. The paths are listed so the caller knows exactly
    /// what would be lost.
    MergeWouldDelete {
        branch: String,
        base: String,
        paths: Vec<String>,
    },
    /// `cancel_acceptance`: the acceptance is not a PENDING acceptance of a
    /// dispatch owned by the calling seat (unknown id, already decided, or
    /// another seat's). Only the requesting seat may withdraw its own request.
    AcceptanceNotCancellable {
        acceptance_id: String,
        seat: String,
    },
    /// `cancel_acceptance`: the reason is empty — a void with no recorded why is
    /// an unexplained edit to the ledger.
    EmptyCancelReason {
        acceptance_id: String,
    },
    /// `get_acceptance_lineage`: no acceptance carries that id. A read of a
    /// mistyped id must say so, not render an empty card.
    AcceptanceNotFound {
        acceptance_id: String,
    },
    /// `request_acceptance`: no acceptance plan was supplied (absent, not an
    /// object, or the empty object — which the contract states is NOT a plan).
    MissingAcceptancePlan,
    /// `request_acceptance`: the plan is an object but does not match shape v1.
    /// `detail` names the first rule broken, so the caller can fix it without
    /// guessing (contract §1: unknown keys, empty scenario/steps/expected,
    /// plan_version != 1, blank action/output/expect, wrong types).
    InvalidAcceptancePlan {
        detail: String,
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
            DispatchRejection::MissingBase => write!(
                f,
                "request_acceptance rejected: no merge base — pass `base` explicitly or set {INTEGRATION_BRANCH_ENV} (the integration branch this delivery branched from). The old hardcoded 'track' is not a branch in this repository, so the base is never invented"
            ),
            DispatchRejection::UnknownBase { base, detail } => write!(
                f,
                "request_acceptance rejected: merge base {base:?} does not exist ({detail}) — a delivery may only be handed to a base that exists, exactly like the delivery branch"
            ),
            DispatchRejection::BranchNotSynced { branch, base } => write!(
                f,
                "request_acceptance rejected: branch {branch:?} does not contain base {base:?} — \
                 merge {base} into {branch} first; accepting an unsynced branch would merge work \
                 that never saw the base's later commits"
            ),
            DispatchRejection::MergeWouldDelete { branch, base, paths } => write!(
                f,
                "request_acceptance rejected: merging {branch:?} into {base:?} would DELETE {} \
                 file(s) tracked on {base:?}: {} — the delivery must not remove merged work",
                paths.len(),
                paths.join(", ")
            ),
            DispatchRejection::AcceptanceNotCancellable { acceptance_id, seat } => write!(
                f,
                "cancel_acceptance rejected: {acceptance_id:?} is not a pending acceptance of a dispatch owned by {seat:?} (unknown id, already decided, or another seat's — only the requesting seat may withdraw its own request)"
            ),
            DispatchRejection::EmptyCancelReason { acceptance_id } => write!(
                f,
                "cancel_acceptance rejected: {acceptance_id:?} needs a reason — a void with no recorded why is an unexplained edit to the ledger"
            ),
            DispatchRejection::AcceptanceNotFound { acceptance_id } => write!(
                f,
                "get_acceptance_lineage rejected: acceptance {acceptance_id:?} not found"
            ),
            DispatchRejection::MissingAcceptancePlan => write!(
                f,
                "request_acceptance rejected: no acceptance plan — the card must carry the product-operation script (scenario/steps/expected, shape v1); an empty object is NOT a plan (no plan -> no card)"
            ),
            DispatchRejection::InvalidAcceptancePlan { detail } => write!(
                f,
                "request_acceptance rejected: the acceptance plan does not match shape v1: {detail}"
            ),
            DispatchRejection::VerdictIdNotAVerification { verdict_event_id } => write!(
                f,
                "enqueue_dispatch rejected: verdict_event_id {verdict_event_id:?} was given for a \
                 payload whose kind is not 'verification' — the pre-declaration belongs to the \
                 verification path only"
            ),
            DispatchRejection::VerdictIdMalformed { verdict_event_id } => write!(
                f,
                "enqueue_dispatch rejected: verdict_event_id {verdict_event_id:?} is not a \
                 well-formed event id (expected 'ev-' followed by a non-empty token)"
            ),
            DispatchRejection::VerdictIdTaken { verdict_event_id } => write!(
                f,
                "enqueue_dispatch rejected: verdict_event_id {verdict_event_id:?} is already in the \
                 ledger — a pre-declared id must be unique, or two verdicts would share one gate"
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
///
/// `pub(crate)` so the memory tools (Lane B) resolve the SAME identity through
/// the SAME function — one rule, one place, no second resolution path.
pub(crate) struct SeatIdentity {
    pub(crate) seat_id: String,
    pub(crate) actor_ref: String,
}

pub(crate) fn resolve_seat() -> Result<SeatIdentity, DispatchRejection> {
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

pub(crate) fn resolve_project(call: Option<String>) -> Result<String, DispatchRejection> {
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

    // The verdict-event-id contract (Aegis ruling 2026-10-11 part 2): a
    // verification request MUST carry a pre-declared verdict event id, so the
    // requester can hang a gated self-dispatch on it and the existing
    // blocked -> ready trigger does the waking. Minted here (or validated when
    // the caller supplies one) and written INTO the payload, so the id is
    // ledger data — visible and auditable — rather than a convention.
    let is_verification = args
        .payload
        .get("kind")
        .and_then(|k| k.as_str())
        .map(|k| k == "verification")
        .unwrap_or(false);
    let mut payload = args.payload.clone();
    let verdict_event_id = if is_verification {
        match &args.verdict_event_id {
            Some(id) => {
                validate_verdict_event_id(id)?;
                Some(id.trim().to_string())
            }
            None => Some(format!("ev-{}", nanoid::nanoid!())),
        }
    } else {
        if let Some(id) = &args.verdict_event_id {
            return Err(DispatchRejection::VerdictIdNotAVerification {
                verdict_event_id: id.clone(),
            });
        }
        None
    };
    if let Some(id) = &verdict_event_id {
        payload["verdict_event_id"] = serde_json::json!(id);
    }

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // A pre-declared id must be unique: two dispatches pre-declaring the same
    // verdict id would share one gate, and the second verdict would silently
    // resolve the first requester's dependency too. "Taken" means BOTH already
    // pre-declared by another dispatch (the payload is where the pre-declaration
    // lives) and already present in the ledger as an event — an id that exists
    // can never be emitted again.
    if let Some(id) = &verdict_event_id {
        let taken = tx
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM dispatches WHERE payload->>'verdict_event_id' = $1) \
                     OR EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)",
                &[&id],
            )
            .await
            .map_err(db)?
            .get::<_, bool>(0);
        if taken {
            return Err(DispatchRejection::VerdictIdTaken {
                verdict_event_id: id.clone(),
            });
        }
    }

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
            &payload,
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
    Ok(match &verdict_event_id {
        Some(id) => format!(
            "enqueued {dispatch_id} (status={}) | verdict_event_id={id} — \
             the verifier MUST emit ReviewVerdictIssued with exactly this id; \
             hang the gated self-dispatch on dep_kind=event, dep_value={id}",
            status.as_str()
        ),
        None => format!("enqueued {dispatch_id} (status={})", status.as_str()),
    })
}

/// Validate a caller-supplied pre-declared verdict event id: `ev-` followed by a
/// non-empty, whitespace-free token. A malformed id would be unresolvable, so it
/// is refused rather than normalised.
fn validate_verdict_event_id(id: &str) -> Result<(), DispatchRejection> {
    let id = id.trim();
    let ok = id.starts_with("ev-") && id.len() > 3 && !id.chars().any(|c| c.is_whitespace());
    if ok {
        Ok(())
    } else {
        Err(DispatchRejection::VerdictIdMalformed {
            verdict_event_id: id.to_string(),
        })
    }
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
/// records the request. The operator later accepts (merge into the recorded
/// base, executed by the Tauri app as the user's agent) or rejects
/// (auto-enqueued rework).
///
/// Guards:
///   0. the merge base is NAMED (explicit `base` argument, else
///      `SEATLOOM_INTEGRATION_BRANCH`) and EXISTS as a local branch — the
///      delivery-branch check the merge path performs, moved to request time so
///      a wrong base is a refusal here instead of a surprise at click time
///      (Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J, defect 1);
///   0b. the acceptance plan is present and matches shape v1 (016) — a missing,
///      empty or malformed plan is a typed rejection before any DB contact
///      (acceptance-lineage milestone, value-domain contract §2);
///   0c. CONTAINMENT: the delivery branch exists, CONTAINS the base
///      (`merge-base --is-ancestor`), and merging it would not DELETE any file
///      tracked on the base (previewed with `merge-tree`). An unsynced branch or
///      a destructive merge is a typed refusal listing the paths — the
///      product-level guarantee that accepting a card cannot destroy merged work
///      (Aegis blocking ruling, dispatch disp-acclose-nimbus);
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

    // Guard 0 — the merge base is named (never invented) and exists. Checked
    // BEFORE the transaction: it is a fact about the repository, not about the
    // dispatch, and a caller that got the base wrong should learn that without
    // a DB round-trip.
    let base = resolve_base(args.base.as_deref())?;
    // Guard 0b — the acceptance plan (016, shape v1) is present and well-formed,
    // ALSO before the transaction: no plan -> no card, and a malformed plan is a
    // typed rejection here rather than a DB CHECK violation later. The DB (016)
    // only re-checks non-emptiness; this is the shape authority.
    let plan = validate_acceptance_plan(&args.acceptance_plan)?;
    let plan_json =
        serde_json::to_value(&plan).map_err(|e| DispatchRejection::Db(e.to_string()))?;
    let repo_root = seatloom_core::git::ops::resolve_repo_root();
    let base_exists = seatloom_core::git::ops::local_branch_exists(&repo_root, &base)
        .await
        .map_err(|detail| DispatchRejection::UnknownBase {
            base: base.clone(),
            detail,
        })?;
    if !base_exists {
        return Err(DispatchRejection::UnknownBase {
            base,
            detail: format!("no refs/heads/<base> in {}", repo_root.display()),
        });
    }
    // The delivery branch must exist too — the merge path checks it, and the
    // containment previews below need a resolvable ref.
    let branch_exists = seatloom_core::git::ops::local_branch_exists(&repo_root, &args.branch)
        .await
        .map_err(|detail| DispatchRejection::UnknownBase {
            base: args.branch.clone(),
            detail,
        })?;
    if !branch_exists {
        return Err(DispatchRejection::UnknownBase {
            base: args.branch.clone(),
            detail: format!("no refs/heads/<branch> in {}", repo_root.display()),
        });
    }
    // Containment (b): the branch must CONTAIN the base. Without this an
    // unsynced branch could be accepted, and the merge would silently drop the
    // base's later commits.
    let synced = seatloom_core::git::ops::branch_contains(&repo_root, &args.branch, &base)
        .await
        .map_err(|detail| DispatchRejection::UnknownBase {
            base: base.clone(),
            detail,
        })?;
    if !synced {
        return Err(DispatchRejection::BranchNotSynced {
            branch: args.branch.clone(),
            base,
        });
    }
    // Containment (c): the merge must not DELETE files tracked on the base.
    // Previewed with `merge-tree` (no ref moves, no working-tree change).
    let would_delete = seatloom_core::git::ops::merge_would_delete(&repo_root, &args.branch, &base)
        .await
        .map_err(|detail| DispatchRejection::UnknownBase {
            base: base.clone(),
            detail,
        })?;
    if !would_delete.is_empty() {
        return Err(DispatchRejection::MergeWouldDelete {
            branch: args.branch.clone(),
            base,
            paths: would_delete,
        });
    }

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
         (id, dispatch_id, project_id, branch, base, how_to_verify, flux_verdict_ref, \
          status, acceptance_plan) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8)",
        &[
            &acceptance_id,
            &args.dispatch_id,
            &project_id,
            &args.branch,
            &base,
            &args.how_to_verify,
            &args.flux_verdict_ref,
            &plan_json,
        ],
    )
    .await
    .map_err(db)?;

    // Same transaction: the request event. Subject = the acceptance (its own
    // object, `014`); actor = the seat; context = the dispatch it gates. The
    // base rides in the payload so the ledger records WHAT the delivery will
    // merge into, not just that a request happened.
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
            "base": base,
            "how_to_verify": args.how_to_verify,
            "flux_verdict_ref": args.flux_verdict_ref,
            "acceptance_plan": plan_json,
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
        "requested acceptance {acceptance_id} for {} (branch {}, base {}, flux verdict {}) — pending operator decision",
        args.dispatch_id, args.branch, base, args.flux_verdict_ref
    ))
}

// =============================================================================
// Tool 6 — cancel_acceptance (seat side, D2 follow-on — the withdrawal)
// =============================================================================

/// Withdraw the calling seat's OWN pending acceptance request: `pending ->
/// cancelled` (terminal) + `AcceptanceCancelled` in ONE transaction.
///
/// Why this exists: before it, the only ways out of a pending card were accept
/// (merge) and reject (auto-enqueue rework). Neither expresses "this card was
/// satisfied by other means" — the bootstrap-install card
/// (`acc-disp-me6Nm8e9D6ZRe4fKyVokY`, base recorded as the non-existent
/// `'track'`, pointing at a branch whose merge would revert merged work) could
/// only be closed by hand-editing the database. A void is now a first-class,
/// recorded transition instead.
///
/// Authority mirrors `cancel_dispatch` (`013`): only the actor that created the
/// row may withdraw it, and only while it is non-terminal. Here that is the
/// seat whose dispatch the acceptance gates — resolved from
/// `SEATLOOM_ACTOR_REF`, never from a parameter.
///
/// A cancel produces NO rework and NO merge: the delivery simply stops being
/// offered to the operator. The reason is mandatory and is recorded on the row
/// (`feedback`) and in the event payload.
pub async fn cancel_acceptance(
    pool: &Pool,
    args: CancelAcceptanceArgs,
) -> Result<String, DispatchRejection> {
    let identity = resolve_seat()?;
    let reason = args.reason.trim().to_string();
    if reason.is_empty() {
        return Err(DispatchRejection::EmptyCancelReason {
            acceptance_id: args.acceptance_id,
        });
    }

    let mut client = pool.get().await.map_err(db)?;
    let tx = client.transaction().await.map_err(db)?;

    // Guarded fetch: the acceptance must be pending AND gate a dispatch owned by
    // the calling seat. Terminal rows are immutable (014), so an
    // accepted/rejected/cancelled card can never be re-cancelled — recorded
    // facts stand.
    let row = tx
        .query_opt(
            "SELECT a.project_id, a.dispatch_id FROM acceptances a \
               JOIN dispatches d ON d.id = a.dispatch_id \
              WHERE a.id = $1 AND a.status = 'pending' AND d.target_seat_id = $2",
            &[&args.acceptance_id, &identity.seat_id],
        )
        .await
        .map_err(db)?;
    let Some(row) = row else {
        return Err(DispatchRejection::AcceptanceNotCancellable {
            acceptance_id: args.acceptance_id,
            seat: identity.seat_id,
        });
    };
    let project_id: String = row.get(0);
    let dispatch_id: String = row.get(1);

    let n = tx
        .execute(
            "UPDATE acceptances \
               SET status = 'cancelled', feedback = $2, decided_by = $3, decided_at = NOW() \
              WHERE id = $1 AND status = 'pending'",
            &[&args.acceptance_id, &reason, &identity.actor_ref],
        )
        .await
        .map_err(db)?;
    if n != 1 {
        return Err(DispatchRejection::AcceptanceNotCancellable {
            acceptance_id: args.acceptance_id,
            seat: identity.seat_id,
        });
    }

    // Same transaction: the withdrawal event. Actor is the SEAT (it is pulling
    // back its own hand-over), refs exactly like AcceptanceRequested.
    let draft = EventDraft {
        event_id: format!("ev-{}", nanoid::nanoid!()),
        event_type: EventType::AcceptanceCancelled,
        schema_version: 1,
        occurred_at: Utc::now(),
        actor_ref: identity.actor_ref.clone(),
        project_id,
        turn_seq: None,
        payload: serde_json::json!({
            "dispatch_id": dispatch_id,
            "reason": reason,
        }),
        refs: vec![
            ObjectRefDraft::subject(RefType::Acceptance, &args.acceptance_id),
            ObjectRefDraft::new(
                RefType::Seat,
                seat_id_of(&identity.actor_ref),
                RefRole::Actor,
            ),
            ObjectRefDraft::new(RefType::Dispatch, &dispatch_id, RefRole::Context),
        ],
    };
    append_event_in_tx(&tx, &draft)
        .await
        .map_err(|e| DispatchRejection::Db(e.to_string()))?;

    tx.commit().await.map_err(db)?;
    Ok(format!(
        "cancelled acceptance {} for {} (reason: {}) — the card is closed; no merge, no rework",
        args.acceptance_id, dispatch_id, reason
    ))
}

// =============================================================================
// Tool 7 — get_acceptance_lineage (read only, acceptance-lineage Lane B)
// =============================================================================

/// Arguments of the lineage read tool. `acceptance_id` only: this is a READ of
/// an already-recorded card, so no seat identity is resolved and none is
/// forgeable — the non-forgeable-identity rule governs WRITES (a seat cannot
/// act as another seat); a read has nothing to act as.
#[derive(Debug, Deserialize)]
pub struct GetAcceptanceLineageArgs {
    pub acceptance_id: String,
}

/// Assemble one acceptance's lineage: the card, the dispatch it gates (with the
/// ORIGINAL task statement), the event chain, the Flux verdict and its evidence,
/// the products, the dependency edges, and the rework dispatch.
///
/// Pure read — zero writes, zero events (value-domain contract §3/§6). It exists
/// so no seat has to `psql` the ledger to answer "what is this card and where
/// did it come from?", which is exactly what Aegis had to do by hand before.
/// The assembler is ONE shared function (`seatloom_core::db::lineage`); the
/// Tauri command `cmd_get_acceptance_lineage` calls the same one, so the two
/// surfaces cannot drift.
pub async fn get_acceptance_lineage(
    db: &seatloom_core::db::SeatloomDb,
    args: GetAcceptanceLineageArgs,
) -> Result<String, DispatchRejection> {
    let lineage = seatloom_core::db::lineage::get_acceptance_lineage(db, &args.acceptance_id)
        .await
        .map_err(|e| match e {
            seatloom_core::db::lineage::LineageError::NotFound { acceptance_id } => {
                DispatchRejection::AcceptanceNotFound { acceptance_id }
            }
            other => DispatchRejection::Db(other.to_string()),
        })?;
    // The response is the assembled object itself: the caller asked for the
    // lineage, so the lineage is the answer — no second, summarised shape that
    // could drift from what the UI renders.
    serde_json::to_string_pretty(&lineage).map_err(|e| DispatchRejection::Db(e.to_string()))
}

// =============================================================================
// Acceptance plan — shape v1 (value-domain contract §1, Lyra sign-off 2026-10-10)
// =============================================================================

/// The structured acceptance plan: the product-operation script the operator
/// follows to verify a user-perceivable result. Shape v1, VERBATIM from the
/// value-domain contract:
///
/// ```text
/// acceptance_plan JSONB, an object with EXACTLY these keys (unknown key =
/// typed rejection):
///   plan_version  INTEGER = 1
///   scenario      STRING  non-empty   -- what the user is trying to achieve
///   steps         ARRAY   non-empty   -- each: {action non-empty, input optional}
///   expected      ARRAY   non-empty   -- each: {output non-empty, expect non-empty}
///   preconditions ARRAY of STRING, optional (empty allowed)
/// ```
///
/// `{}` is NOT a plan. The plan is authored by the PO at request time and is
/// immutable afterwards — the same immutability rule as `how_to_verify`, and
/// there is deliberately no update path.
///
/// The struct exists so the validated plan has ONE typed home; the tool layer
/// validates it BEFORE any DB contact (house discipline: typed errors, never a
/// DB-error crutch), and the database layer (016) only re-checks non-emptiness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptancePlanV1 {
    pub plan_version: i64,
    pub scenario: String,
    pub steps: Vec<AcceptancePlanStep>,
    pub expected: Vec<AcceptancePlanExpectation>,
    #[serde(default)]
    pub preconditions: Vec<String>,
}

/// One product-operation step: where to click / what to fill / what to say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptancePlanStep {
    pub action: String,
    #[serde(default)]
    pub input: Option<String>,
}

/// One observable output and its pass criterion ("seeing X = pass").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptancePlanExpectation {
    pub output: String,
    pub expect: String,
}

/// Validate a plan value against shape v1.
///
/// The order of the checks is the contract's order, and each failure names the
/// rule broken — a caller that gets `plan_version: 2` back is told exactly that,
/// not "invalid plan". Blank strings are treated as absent everywhere: a step
/// whose `action` is `"   "` is an empty action, not a step.
pub fn validate_acceptance_plan(
    value: &serde_json::Value,
) -> Result<AcceptancePlanV1, DispatchRejection> {
    if value.is_null() {
        return Err(DispatchRejection::MissingAcceptancePlan);
    }
    if !value.is_object() {
        return Err(DispatchRejection::InvalidAcceptancePlan {
            detail: format!("the plan must be a JSON object, found {}", json_kind(value)),
        });
    }
    if value.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return Err(DispatchRejection::MissingAcceptancePlan);
    }
    // `deny_unknown_fields` on every struct makes an unknown key a serde error
    // here, which is exactly the contract's rule.
    let plan: AcceptancePlanV1 = serde_json::from_value(value.clone()).map_err(|e| {
        DispatchRejection::InvalidAcceptancePlan {
            detail: e.to_string(),
        }
    })?;
    if plan.plan_version != 1 {
        return Err(DispatchRejection::InvalidAcceptancePlan {
            detail: format!("plan_version must be 1, found {}", plan.plan_version),
        });
    }
    let scenario = plan.scenario.trim();
    if scenario.is_empty() {
        return Err(DispatchRejection::InvalidAcceptancePlan {
            detail: "scenario must be a non-empty string".to_string(),
        });
    }
    if plan.steps.is_empty() {
        return Err(DispatchRejection::InvalidAcceptancePlan {
            detail: "steps must be a non-empty array".to_string(),
        });
    }
    for (i, step) in plan.steps.iter().enumerate() {
        if step.action.trim().is_empty() {
            return Err(DispatchRejection::InvalidAcceptancePlan {
                detail: format!("steps[{i}].action must be a non-empty string"),
            });
        }
        if let Some(input) = &step.input {
            if input.trim().is_empty() {
                return Err(DispatchRejection::InvalidAcceptancePlan {
                    detail: format!("steps[{i}].input must be a non-empty string when present"),
                });
            }
        }
    }
    if plan.expected.is_empty() {
        return Err(DispatchRejection::InvalidAcceptancePlan {
            detail: "expected must be a non-empty array".to_string(),
        });
    }
    for (i, exp) in plan.expected.iter().enumerate() {
        if exp.output.trim().is_empty() {
            return Err(DispatchRejection::InvalidAcceptancePlan {
                detail: format!("expected[{i}].output must be a non-empty string"),
            });
        }
        if exp.expect.trim().is_empty() {
            return Err(DispatchRejection::InvalidAcceptancePlan {
                detail: format!("expected[{i}].expect must be a non-empty string"),
            });
        }
    }
    for (i, pre) in plan.preconditions.iter().enumerate() {
        if pre.trim().is_empty() {
            return Err(DispatchRejection::InvalidAcceptancePlan {
                detail: format!("preconditions[{i}] must be a non-empty string when present"),
            });
        }
    }
    Ok(plan)
}

/// The JSON kind of a value, for an error that says what it found.
fn json_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Array(_) => "an array",
        serde_json::Value::Object(_) => "an object",
    }
}

/// Resolve the merge base: the explicit argument wins, then
/// `SEATLOOM_INTEGRATION_BRANCH`. Neither naming it is a REFUSAL — the old
/// hardcoded `'track'` was a branch that does not exist in this repository, so
/// the base is discovered or the request does not happen.
fn resolve_base(explicit: Option<&str>) -> Result<String, DispatchRejection> {
    if let Some(v) = explicit.filter(|v| !v.trim().is_empty()) {
        return Ok(v.trim().to_string());
    }
    std::env::var(INTEGRATION_BRANCH_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(|v| v.trim().to_string())
        .ok_or(DispatchRejection::MissingBase)
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

/// Serialises env mutation across this file's tests. The tools read identity
/// (`SEATLOOM_ACTOR_REF`) and the merge base (`SEATLOOM_INTEGRATION_BRANCH`,
/// `SEATLOOM_REPO_ROOT`) from the process env, and cargo runs a crate's tests
/// on parallel threads inside one process — so every test that touches them
/// holds this lock.
#[cfg(test)]
pub(crate) static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A valid shape-v1 acceptance plan, for tests whose subject is another guard
/// (base, pending, ownership). The plan's own rules have their own matrix in
/// `mod tests`. Shared by both test modules, so a plan-shape change cannot
/// silently break the other guards' fixtures.
#[cfg(test)]
pub(crate) fn valid_plan() -> serde_json::Value {
    serde_json::json!({
        "plan_version": 1,
        "scenario": "operator verifies the delivered slice end to end",
        "steps": [
            { "action": "open the dispatch page and expand the acceptance card" },
            { "action": "run the one-step check", "input": "pnpm test" }
        ],
        "expected": [
            { "output": "the suite passes", "expect": "seeing a green run" }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_id_strips_the_actor_prefix() {
        assert_eq!(seat_id_of("seat:seat-nimbus-001"), "seat-nimbus-001");
        assert_eq!(seat_id_of("seat-nimbus-001"), "seat-nimbus-001");
    }

    /// A pool pointed at a closed port: it exists so a test can prove the code
    /// REACHED the database (the failure is a connection error, not a guard
    /// refusal) without needing a database at all.
    fn dead_pool() -> Pool {
        use deadpool_postgres::Runtime;
        use tokio_postgres::NoTls;
        let mut cfg = deadpool_postgres::Config::new();
        cfg.url = Some("postgresql://seatloom:seatloom@127.0.0.1:1/seatloom_unused".to_string());
        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("a pool object builds without connecting")
    }

    /// A scratch git repo whose local `refs/heads` carries `branch` — the shape
    /// the request-time base guard inspects. Worktrees share `refs/heads`, so a
    /// branch created here answers exactly like one on the main checkout.
    fn scratch_repo_with_branch(branch: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "seatloom-base-guard-{}-{}",
            std::process::id(),
            branch.replace('/', "_")
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let g = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap()
        };
        let ok = |args: &[&str]| assert!(g(args).status.success(), "git {args:?} failed");
        ok(&["init", "-b", "main"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("f.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);
        ok(&["branch", branch]);
        root
    }

    /// The containment defence (Aegis blocking ruling, dispatch
    /// disp-acclose-nimbus PART 2): a branch that does not contain the base, and
    /// a branch whose merge would DELETE base-tracked files, are both refused
    /// BEFORE any DB contact. A scratch repo supplies the three shapes; the pool
    /// is dead, so reaching the database would fail differently — proving the
    /// refusal happened first.
    #[tokio::test]
    async fn request_acceptance_refuses_unsynced_and_destructive_branches() {
        let _guard = ENV_LOCK.lock().await;
        let root =
            std::env::temp_dir().join(format!("seatloom-containment-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let g = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap()
        };
        let ok = |args: &[&str]| assert!(g(args).status.success(), "git {args:?} failed");
        ok(&["init", "-b", "track"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("keep.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);
        // The base moves on — a delivery that never sees this is unsynced.
        ok(&["checkout", "-b", "packet/seat/synced"]);
        std::fs::write(root.join("added.txt"), "delivery\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "delivery"]);
        ok(&["checkout", "track"]);
        std::fs::write(root.join("later.txt"), "later\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base moves on"]);
        // unsynced: forked from the base, never merged it back.
        ok(&["checkout", "-b", "packet/seat/unsynced", "HEAD~1"]);
        std::fs::write(root.join("other.txt"), "other\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "never synced"]);
        // destructive: synced, but deletes a base-tracked file.
        ok(&["checkout", "packet/seat/synced"]);
        ok(&["merge", "--no-edit", "track"]);
        ok(&["checkout", "-b", "packet/seat/destructive"]);
        std::fs::remove_file(root.join("keep.txt")).unwrap();
        ok(&["add", "-A"]);
        ok(&["commit", "-m", "deletes a tracked file"]);
        // missing: a name that is not a branch at all.
        ok(&["checkout", "track"]);

        let had_root = std::env::var_os(seatloom_core::git::ops::REPO_ROOT_ENV);
        let had_base = std::env::var_os(INTEGRATION_BRANCH_ENV);
        std::env::set_var(seatloom_core::git::ops::REPO_ROOT_ENV, &root);
        std::env::set_var(INTEGRATION_BRANCH_ENV, "track");
        std::env::set_var(ACTOR_REF_ENV, "seat:seat-nimbus-001");
        let pool = dead_pool();
        let args = |branch: &str| RequestAcceptanceArgs {
            dispatch_id: "disp-containment".into(),
            branch: branch.into(),
            how_to_verify: "do A see B".into(),
            flux_verdict_ref: "ev-verdict".into(),
            base: Some("track".into()),
            acceptance_plan: valid_plan(),
        };

        // 1. the branch does not exist at all.
        let err = request_acceptance(&pool, args("packet/seat/nope"))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::UnknownBase { .. }),
            "a missing branch is a typed refusal: {err}"
        );

        // 2. the branch exists but does not contain the base.
        let err = request_acceptance(&pool, args("packet/seat/unsynced"))
            .await
            .unwrap_err();
        let msg = err.to_string();
        match err {
            DispatchRejection::BranchNotSynced { branch, base } => {
                assert_eq!(branch, "packet/seat/unsynced");
                assert_eq!(base, "track");
            }
            other => panic!("expected BranchNotSynced, got {other:?}"),
        }
        assert!(
            msg.contains("does not contain base"),
            "the refusal says what to do: {msg}"
        );

        // 3. the merge would delete a base-tracked file, and the path is named.
        let err = request_acceptance(&pool, args("packet/seat/destructive"))
            .await
            .unwrap_err();
        let msg = err.to_string();
        match err {
            DispatchRejection::MergeWouldDelete { paths, .. } => {
                assert_eq!(paths, vec!["keep.txt".to_string()], "{paths:?}");
            }
            other => panic!("expected MergeWouldDelete, got {other:?}"),
        }
        assert!(
            msg.contains("keep.txt"),
            "the refusal lists what would be lost: {msg}"
        );

        // 4. the synced, non-destructive branch passes every containment check
        //    and reaches the database (the dead pool then proves it).
        let err = request_acceptance(&pool, args("packet/seat/synced"))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::Db(_)),
            "a contained delivery gets past the defence into the DB: {err}"
        );

        match had_root {
            Some(v) => std::env::set_var(seatloom_core::git::ops::REPO_ROOT_ENV, v),
            None => std::env::remove_var(seatloom_core::git::ops::REPO_ROOT_ENV),
        }
        match had_base {
            Some(v) => std::env::set_var(INTEGRATION_BRANCH_ENV, v),
            None => std::env::remove_var(INTEGRATION_BRANCH_ENV),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Defect 1 (Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J): the merge base is
    /// NAMED and EXISTS before a card is recorded. The old code hardcoded
    /// `'track'` — not a branch in this repository — so the error only appeared
    /// when the operator clicked Accept. Order pinned here: an unnamed base
    /// refuses, an absent base refuses, and a base that exists gets PAST the
    /// guard (the dead pool then proves the database was reached).
    #[tokio::test]
    async fn request_acceptance_names_and_verifies_the_base_before_the_db() {
        let _guard = ENV_LOCK.lock().await;
        let root = scratch_repo_with_branch("track/infra-foundation");
        // The delivery branch must exist (and contain the base) for the
        // containment checks to let a request through — this test's subject is
        // the BASE, so the branch is a well-formed fixture here.
        {
            let g = |args: &[&str]| {
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(&root)
                    .output()
                    .unwrap()
            };
            assert!(
                g(&["branch", "packet/nimbus/x", "track/infra-foundation"])
                    .status
                    .success(),
                "the delivery branch fixture"
            );
        }
        let had_root = std::env::var_os(seatloom_core::git::ops::REPO_ROOT_ENV);
        let had_base = std::env::var_os(INTEGRATION_BRANCH_ENV);
        std::env::set_var(seatloom_core::git::ops::REPO_ROOT_ENV, &root);
        std::env::set_var(ACTOR_REF_ENV, "seat:seat-nimbus-001");
        std::env::remove_var(INTEGRATION_BRANCH_ENV);
        let pool = dead_pool();
        let args = |base: Option<&str>| RequestAcceptanceArgs {
            dispatch_id: "disp-base-guard".into(),
            branch: "packet/nimbus/x".into(),
            how_to_verify: "do A see B".into(),
            flux_verdict_ref: "ev-verdict".into(),
            base: base.map(str::to_string),
            acceptance_plan: valid_plan(),
        };

        // 1. Nothing names the base: refuse rather than invent 'track'.
        let err = request_acceptance(&pool, args(None)).await.unwrap_err();
        assert!(
            matches!(err, DispatchRejection::MissingBase),
            "an unnamed base must refuse, never default: {err}"
        );

        // 2. The base is named but is not a branch: refuse, symmetric with the
        //    delivery-branch check the merge path performs.
        let err = request_acceptance(&pool, args(Some("track")))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::UnknownBase { .. }),
            "a non-existent base must refuse at request time: {err}"
        );

        // 3. The base exists: the guard passes and the DB is reached (the pool
        //    is dead, so the only possible failure is the connection).
        let err = request_acceptance(&pool, args(Some("track/infra-foundation")))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::Db(_)),
            "an existing base must get past the guard into the DB: {err}"
        );

        // 4. The env var supplies the base when the argument omits it — the
        //    discoverable source, so callers need not repeat the branch name.
        std::env::set_var(INTEGRATION_BRANCH_ENV, "track/infra-foundation");
        let err = request_acceptance(&pool, args(None)).await.unwrap_err();
        assert!(
            matches!(err, DispatchRejection::Db(_)),
            "the configured integration branch must satisfy the guard: {err}"
        );

        match had_root {
            Some(v) => std::env::set_var(seatloom_core::git::ops::REPO_ROOT_ENV, v),
            None => std::env::remove_var(seatloom_core::git::ops::REPO_ROOT_ENV),
        }
        match had_base {
            Some(v) => std::env::set_var(INTEGRATION_BRANCH_ENV, v),
            None => std::env::remove_var(INTEGRATION_BRANCH_ENV),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Defect 2: a void with no recorded why is an unexplained edit to the
    /// ledger, so the reason is screened BEFORE any row is touched.
    #[tokio::test]
    async fn cancel_acceptance_requires_a_reason_before_the_db() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var(ACTOR_REF_ENV, "seat:seat-nimbus-001");
        let pool = dead_pool();
        let err = cancel_acceptance(
            &pool,
            CancelAcceptanceArgs {
                acceptance_id: "acc-disp-x".into(),
                reason: "   ".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::EmptyCancelReason { .. }),
            "a blank reason must refuse: {err}"
        );
    }

    /// Contract §1/§2 + packet §5.2: every malformed-plan branch is a TYPED
    /// rejection naming the rule broken, and a valid plan passes. `{}` and
    /// `null` are MissingAcceptancePlan (not "an empty plan"); everything else
    /// is InvalidAcceptancePlan with the first broken rule.
    #[test]
    fn acceptance_plan_shape_v1_rejects_every_malformed_branch() {
        // --- the two "no plan at all" cases
        for empty in [
            serde_json::json!({}),
            serde_json::Value::Null,
            serde_json::json!("a string is not a plan"),
            serde_json::json!([1, 2, 3]),
            serde_json::json!(42),
        ] {
            let err = validate_acceptance_plan(&empty).expect_err(&format!("must refuse: {empty}"));
            assert!(
                matches!(
                    err,
                    DispatchRejection::MissingAcceptancePlan
                        | DispatchRejection::InvalidAcceptancePlan { .. }
                ),
                "must be a typed plan rejection, got {err:?}"
            );
        }
        assert!(matches!(
            validate_acceptance_plan(&serde_json::json!({})).unwrap_err(),
            DispatchRejection::MissingAcceptancePlan
        ));
        assert!(matches!(
            validate_acceptance_plan(&serde_json::Value::Null).unwrap_err(),
            DispatchRejection::MissingAcceptancePlan
        ));

        // --- plan_version
        let mut p = valid_plan();
        p["plan_version"] = serde_json::json!(2);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("plan_version must be 1"), "{err}");
        p = valid_plan();
        p["plan_version"] = serde_json::json!("1");
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(!err.is_empty(), "a string version is refused");

        // --- unknown key (the contract's own rule, enforced by deny_unknown_fields)
        let mut p = valid_plan();
        p["surprise"] = serde_json::json!("x");
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("surprise"), "the unknown key is named: {err}");

        // --- scenario
        let mut p = valid_plan();
        p["scenario"] = serde_json::json!("   ");
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("scenario"), "{err}");
        let mut p = valid_plan();
        p.as_object_mut().unwrap().remove("scenario");
        assert!(
            validate_acceptance_plan(&p).is_err(),
            "scenario is required"
        );

        // --- steps
        let mut p = valid_plan();
        p["steps"] = serde_json::json!([]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("steps must be a non-empty array"), "{err}");
        let mut p = valid_plan();
        p["steps"] = serde_json::json!([{ "action": "  " }]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("steps[0].action"), "{err}");
        let mut p = valid_plan();
        p["steps"] = serde_json::json!([{ "action": "do it", "input": "" }]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("steps[0].input"), "{err}");
        let mut p = valid_plan();
        p["steps"] = serde_json::json!([{ "action": "do it", "extra": 1 }]);
        assert!(
            validate_acceptance_plan(&p).is_err(),
            "an unknown step key is refused"
        );

        // --- expected
        let mut p = valid_plan();
        p["expected"] = serde_json::json!([]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("expected must be a non-empty array"), "{err}");
        let mut p = valid_plan();
        p["expected"] = serde_json::json!([{ "output": "", "expect": "x" }]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("expected[0].output"), "{err}");
        let mut p = valid_plan();
        p["expected"] = serde_json::json!([{ "output": "o", "expect": " " }]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("expected[0].expect"), "{err}");

        // --- preconditions: optional, but a blank entry is refused
        let mut p = valid_plan();
        p["preconditions"] = serde_json::json!([]);
        assert!(validate_acceptance_plan(&p).is_ok(), "empty is allowed");
        let mut p = valid_plan();
        p["preconditions"] = serde_json::json!(["db up", " "]);
        let err = validate_acceptance_plan(&p).unwrap_err().to_string();
        assert!(err.contains("preconditions[1]"), "{err}");

        // --- the happy path returns the typed plan
        let plan = validate_acceptance_plan(&valid_plan()).expect("a valid plan passes");
        assert_eq!(plan.plan_version, 1);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[1].input.as_deref(), Some("pnpm test"));
        assert_eq!(plan.expected.len(), 1);
        assert!(plan.preconditions.is_empty());
    }

    /// The withdrawal args are strict about the shape the tool schema advertises:
    /// `reason` is required (an absent one is a bad-arguments rejection, not a
    /// silent empty reason).
    #[test]
    fn cancel_acceptance_args_require_the_reason() {
        let ok: CancelAcceptanceArgs = serde_json::from_value(serde_json::json!({
            "acceptance_id": "acc-disp-x",
            "reason": "already landed via the bootstrap install"
        }))
        .unwrap();
        assert_eq!(ok.acceptance_id, "acc-disp-x");
        assert!(!ok.reason.is_empty());
        let missing: Result<CancelAcceptanceArgs, _> = serde_json::from_value(serde_json::json!({
            "acceptance_id": "acc-disp-x"
        }));
        assert!(missing.is_err(), "reason is required");
    }

    /// The resolution order for the merge base, pinned directly: the explicit
    /// argument wins, the env var is the discoverable fallback, and silence is a
    /// refusal. A blank/whitespace value is treated as absent, never as a
    /// branch name.
    #[test]
    fn resolve_base_prefers_the_argument_then_the_env_and_never_invents() {
        // A sync test: `blocking_lock` (an async-aware lock would deadlock a
        // plain #[test] that tried to await it).
        let _guard = ENV_LOCK.blocking_lock();
        let had = std::env::var_os(INTEGRATION_BRANCH_ENV);

        std::env::set_var(INTEGRATION_BRANCH_ENV, "track/infra-foundation");
        assert_eq!(
            resolve_base(Some("track/other")).unwrap(),
            "track/other",
            "the explicit argument wins over the env var"
        );
        assert_eq!(
            resolve_base(None).unwrap(),
            "track/infra-foundation",
            "the env var is the discoverable source"
        );
        assert_eq!(
            resolve_base(Some("   ")).unwrap(),
            "track/infra-foundation",
            "a blank argument is absent, not a branch name"
        );

        std::env::remove_var(INTEGRATION_BRANCH_ENV);
        assert!(
            matches!(resolve_base(None), Err(DispatchRejection::MissingBase)),
            "nothing naming the base is a refusal — 'track' is never invented"
        );

        match had {
            Some(v) => std::env::set_var(INTEGRATION_BRANCH_ENV, v),
            None => std::env::remove_var(INTEGRATION_BRANCH_ENV),
        }
    }

    /// `base` stays optional on the wire (the env var can supply it), while
    /// `acceptance_plan` does NOT: 016 made the plan part of the card, so a call
    /// without one is a bad-arguments rejection rather than a silently
    /// plan-less card. This is the wire contract of the two optionality rules
    /// side by side.
    #[test]
    fn request_acceptance_base_is_optional_but_the_plan_is_not() {
        let legacy: RequestAcceptanceArgs = serde_json::from_value(serde_json::json!({
            "dispatch_id": "disp-1", "branch": "packet/nimbus/x",
            "how_to_verify": "do A see B", "flux_verdict_ref": "ev-1",
            "acceptance_plan": valid_plan()
        }))
        .unwrap();
        assert_eq!(legacy.base, None, "base stays optional on the wire");
        assert!(legacy.acceptance_plan.is_object());

        let explicit: RequestAcceptanceArgs = serde_json::from_value(serde_json::json!({
            "dispatch_id": "disp-1", "branch": "packet/nimbus/x",
            "how_to_verify": "do A see B", "flux_verdict_ref": "ev-1",
            "base": "track/infra-foundation",
            "acceptance_plan": valid_plan()
        }))
        .unwrap();
        assert_eq!(explicit.base.as_deref(), Some("track/infra-foundation"));

        let no_plan: Result<RequestAcceptanceArgs, _> = serde_json::from_value(serde_json::json!({
            "dispatch_id": "disp-1", "branch": "packet/nimbus/x",
            "how_to_verify": "do A see B", "flux_verdict_ref": "ev-1"
        }));
        assert!(
            no_plan.is_err(),
            "a call without a plan must not parse into a plan-less card"
        );
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

/// D2 / 013 integration tests for the dispatch TOOLS (enqueue / get_next /
/// complete / cancel / request_acceptance) against a real Postgres. `#[ignore]`
/// so `cargo test -p seatloom-mcp-emit` passes without a DB; they REFUSE the
/// live `seatloom` database. The tools resolve identity from env, so these are
/// env-serialised — run SINGLE-THREADED against a scratch DB:
///
///   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_d2_nimbus \
///     cargo test -p seatloom-mcp-emit -- --ignored --test-threads=1
#[cfg(test)]
mod db_itests {
    use super::*;
    use crate::memory::{
        forget, promote, recall, remember, resolve, ForgetArgs, MemoryRelationArg, PromoteArgs,
        RecallArgs, RememberArgs, ResolveArgs,
    };
    use crate::read;
    use crate::topics;
    use deadpool_postgres::Pool;
    use seatloom_core::db::SeatloomDb;
    use tokio_postgres::NoTls;

    const SEAT: &str = "seat:seat-nimbus-001";
    const SEAT_ID: &str = "seat-nimbus-001";

    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!("REFUSING live `seatloom` DB — set DATABASE_URL to a scratch DB");
            return None;
        }
        let mut cfg = deadpool_postgres::Config::new();
        cfg.url = Some(url);
        let pool = cfg
            .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
            .ok()?;
        let c = pool.get().await.ok()?;
        // Require 013 (retry_count) + 014 (acceptances); else skip.
        let ready: bool = c
            .query_one(
                "SELECT to_regclass('public.acceptances') IS NOT NULL \
                    AND EXISTS (SELECT 1 FROM information_schema.columns \
                                 WHERE table_name='dispatches' AND column_name='retry_count')",
                &[],
            )
            .await
            .ok()?
            .get(0);
        ready.then_some(pool)
    }

    fn as_seat() {
        std::env::set_var(ACTOR_REF_ENV, SEAT);
        std::env::set_var(PROJECT_ID_ENV, "seatloom_itest");
    }
    fn as_actor(actor: &str) {
        std::env::set_var(ACTOR_REF_ENV, actor);
        std::env::set_var(PROJECT_ID_ENV, "seatloom_itest");
    }

    /// Point the request-time base guard at a scratch git repo that carries a
    /// local `track` branch, so the guard is satisfied by a REAL repository
    /// rather than by whatever the runner happens to have checked out. Returns
    /// the temp root (the OS reclaims /tmp).
    fn with_base_repo() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("seatloom-itest-base-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let g = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap()
        };
        let ok = |args: &[&str]| assert!(g(args).status.success(), "git {args:?} failed");
        ok(&["init", "-b", "track"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("f.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);
        // The delivery branch the tests hand to the gate: created FROM the base
        // (so it contains it) and non-destructive (so the merge preview finds
        // nothing to delete) — the shape the containment defence requires.
        ok(&["branch", "packet/nimbus/x", "track"]);
        std::env::set_var(seatloom_core::git::ops::REPO_ROOT_ENV, &root);
        root
    }

    async fn clean(pool: &Pool, project: &str) {
        let c = pool.get().await.expect("client");
        for sql in [
            format!("DELETE FROM acceptances WHERE project_id='{project}'"),
            format!("DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id='{project}')"),
            format!("DELETE FROM dispatches WHERE project_id='{project}'"),
            format!("DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='{project}')"),
            format!("DELETE FROM canonical_events WHERE project_id='{project}'"),
        ] {
            c.execute(&sql, &[]).await.expect("clean");
        }
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ($1,$1,NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[&project],
        ).await.expect("project");
    }

    /// Clean the D5 fixtures (015): memories, topics, and the memory events.
    async fn clean_memories(pool: &Pool, project: &str) {
        let c = pool.get().await.expect("client");
        // Memories are deleted by topic membership as well as by project: a
        // memory created under another project's fixture may still point at one
        // of this project's topics, and the FK would refuse the topic delete.
        // Edges first: 018's memory_relations references memories(id), so the
        // memories cannot be deleted while an edge points at them.
        for sql in [
            format!(
                "DELETE FROM memory_relations WHERE from_id IN (SELECT id FROM memories WHERE project_id = '{project}') \
                  OR to_id IN (SELECT id FROM memories WHERE project_id = '{project}')"
            ),
            format!(
                "DELETE FROM memories WHERE project_id = '{project}' \
                  OR topic_id IN (SELECT id FROM topics WHERE project_id = '{project}')"
            ),
            format!("DELETE FROM topics WHERE project_id = '{project}'"),
            format!("DELETE FROM documents WHERE project_id = '{project}'"),
        ] {
            c.execute(&sql, &[]).await.expect("clean");
        }
    }

    async fn status_of(pool: &Pool, id: &str) -> (String, i32) {
        let c = pool.get().await.expect("client");
        let r = c
            .query_one(
                "SELECT status, retry_count FROM dispatches WHERE id=$1",
                &[&id],
            )
            .await
            .expect("row");
        (r.get(0), r.get(1))
    }

    async fn dispatch_event_types(pool: &Pool, id: &str) -> Vec<String> {
        let c = pool.get().await.expect("client");
        c.query(
            "SELECT e.event_type FROM canonical_events e \
               JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' \
              WHERE r.ref_type='dispatch' AND r.ref_id=$1 ORDER BY e.occurred_at",
            &[&id],
        )
        .await
        .expect("events")
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect()
    }

    fn enqueue_args(id: &str, project: &str, max_retries: i64) -> EnqueueArgs {
        EnqueueArgs {
            id: Some(id.to_string()),
            target_seat_id: SEAT_ID.to_string(),
            title: "itest".to_string(),
            payload: serde_json::json!({"packet": "p/x.md"}),
            priority: Some(5),
            project_id: Some(project.to_string()),
            dependencies: vec![],
            max_retries: Some(max_retries),
            verdict_event_id: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-014 (scratch DB); run --test-threads=1"]
    async fn complete_failed_retries_within_budget_then_fails_terminally() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        as_seat();

        enqueue(&pool, enqueue_args("disp-it-retry", project, 1))
            .await
            .unwrap();
        get_next(&pool, Some(project.into())).await.unwrap(); // -> in_flight
                                                              // Within budget -> ready + retry_count+1 + SeatDispatchRetried.
        let msg = complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-retry".into(),
                outcome: Some("failed".into()),
                reason: Some("flaky".into()),
            },
        )
        .await
        .unwrap();
        assert!(msg.contains("re-opened to ready"), "{msg}");
        assert_eq!(status_of(&pool, "disp-it-retry").await, ("ready".into(), 1));

        // Pull again, exhaust the budget -> failed (terminal) + SeatDispatchFailed.
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-retry".into(),
                outcome: Some("failed".into()),
                reason: Some("still flaky".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            status_of(&pool, "disp-it-retry").await,
            ("failed".into(), 1)
        );
        assert_eq!(
            dispatch_event_types(&pool, "disp-it-retry").await,
            vec![
                "SeatDispatchIssued",
                "SeatDispatchDelivered",
                "SeatDispatchRetried",
                "SeatDispatchDelivered",
                "SeatDispatchFailed"
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-014 (scratch DB); run --test-threads=1"]
    async fn complete_failed_with_default_budget_fails_on_first_failure() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        as_seat();
        enqueue(&pool, enqueue_args("disp-it-nobudget", project, 0))
            .await
            .unwrap();
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-nobudget".into(),
                outcome: Some("failed".into()),
                reason: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            status_of(&pool, "disp-it-nobudget").await,
            ("failed".into(), 0)
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-014 (scratch DB); run --test-threads=1"]
    async fn cancel_is_issuer_only_and_releases_the_in_flight_slot() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;

        // Issued by the operator; pulled in_flight by the seat.
        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-cancel", project, 0))
            .await
            .unwrap();
        as_seat();
        get_next(&pool, Some(project.into())).await.unwrap(); // in_flight (holds the slot)

        // A non-issuer (the seat) cannot cancel.
        let err = cancel(
            &pool,
            CancelArgs {
                id: "disp-it-cancel".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::NotCancellable { .. }),
            "{err}"
        );

        // The issuer can; it releases the slot and records SeatDispatchCancelled.
        as_actor("human:supervisor");
        cancel(
            &pool,
            CancelArgs {
                id: "disp-it-cancel".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(status_of(&pool, "disp-it-cancel").await.0, "cancelled");
        assert!(dispatch_event_types(&pool, "disp-it-cancel")
            .await
            .contains(&"SeatDispatchCancelled".to_string()));

        // Slot released: the seat can now hold a fresh in_flight.
        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-after", project, 0))
            .await
            .unwrap();
        as_seat();
        let msg = get_next(&pool, Some(project.into())).await.unwrap();
        assert!(msg.contains("delivered disp-it-after"), "{msg}");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-014 (scratch DB); run --test-threads=1"]
    async fn request_acceptance_guards_then_happy_path_and_rework_pullable() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        with_base_repo();

        // A verdict event the request can reference.
        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ('ev-it-verdict','ReviewVerdictIssued',NOW(),'seat:seat-flux-001','{\"verdict\":\"pass\"}',$1,1)",
                &[&project],
            ).await.unwrap();
        }

        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-req", project, 0))
            .await
            .unwrap();
        as_seat();

        // Guard: not completed yet (still ready, not pulled).
        let err = request_acceptance(
            &pool,
            RequestAcceptanceArgs {
                dispatch_id: "disp-it-req".into(),
                branch: "packet/nimbus/x".into(),
                how_to_verify: "h".into(),
                flux_verdict_ref: "ev-it-verdict".into(),
                base: Some("track".into()),
                acceptance_plan: valid_plan(),
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::NotCompleted { .. }),
            "{err}"
        );

        // Complete it, then the happy path creates the acceptance + event.
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-req".into(),
                outcome: None,
                reason: None,
            },
        )
        .await
        .unwrap();
        let ok = request_acceptance(
            &pool,
            RequestAcceptanceArgs {
                dispatch_id: "disp-it-req".into(),
                branch: "packet/nimbus/x".into(),
                how_to_verify: "do A see B".into(),
                flux_verdict_ref: "ev-it-verdict".into(),
                base: Some("track".into()),
                acceptance_plan: valid_plan(),
            },
        )
        .await
        .unwrap();
        assert!(ok.contains("acc-disp-it-req"), "{ok}");

        // Guard: a second request while pending is refused.
        let err = request_acceptance(
            &pool,
            RequestAcceptanceArgs {
                dispatch_id: "disp-it-req".into(),
                branch: "packet/nimbus/x".into(),
                how_to_verify: "h".into(),
                flux_verdict_ref: "ev-it-verdict".into(),
                base: Some("track".into()),
                acceptance_plan: valid_plan(),
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, DispatchRejection::AcceptancePending { .. }),
            "{err}"
        );

        // The acceptance row + AcceptanceRequested landed.
        let c = pool.get().await.unwrap();
        let st: String = c
            .query_one(
                "SELECT status FROM acceptances WHERE id='acc-disp-it-req'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(st, "pending");

        // The recorded base is the one the caller named (NOT the old hardcoded
        // 'track' default), and the ledger event carries it too.
        let row = c
            .query_one(
                "SELECT a.base, \
                        (SELECT e.payload FROM canonical_events e \
                           JOIN event_object_refs r ON r.event_id = e.id \
                            AND r.role = 'subject' AND r.ref_type = 'acceptance' \
                          WHERE r.ref_id = a.id) \
                   FROM acceptances a WHERE a.id = 'acc-disp-it-req'",
                &[],
            )
            .await
            .unwrap();
        let recorded_base: String = row.get(0);
        let payload: serde_json::Value = row.get(1);
        assert_eq!(recorded_base, "track");
        assert_eq!(payload["base"], "track", "the event records the merge base");

        // A rework dispatch (as the reject path would enqueue) is pullable.
        as_actor("human:supervisor");
        enqueue(&pool, EnqueueArgs {
            id: Some("disp-it-rework".into()), target_seat_id: SEAT_ID.into(),
            title: "打回重做:itest".into(),
            payload: serde_json::json!({"rework_of":"disp-it-req","feedback":"redo","branch":"packet/nimbus/x"}),
            priority: Some(15), project_id: Some(project.into()), dependencies: vec![], max_retries: Some(0),
            verdict_event_id: None,
        }).await.unwrap();
        as_seat();
        let msg = get_next(&pool, Some(project.into())).await.unwrap();
        assert!(
            msg.contains("delivered disp-it-rework"),
            "rework must be pullable: {msg}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-014 (scratch DB); run --test-threads=1"]
    async fn live_status_returns_terminal_rows_as_stored_never_recomputed_to_ready() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        as_seat();

        // A failed (terminal) dispatch with NO unmet deps: a naive recompute
        // would read it ready. It must stay failed and never be delivered.
        enqueue(&pool, enqueue_args("disp-it-term", project, 0))
            .await
            .unwrap();
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-term".into(),
                outcome: Some("failed".into()),
                reason: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(status_of(&pool, "disp-it-term").await.0, "failed");

        // Nothing is pullable (the terminal row is not recomputed to ready).
        let msg = get_next(&pool, Some(project.into())).await.unwrap();
        assert!(
            msg.contains("nothing ready"),
            "terminal row must not be delivered: {msg}"
        );
        assert_eq!(status_of(&pool, "disp-it-term").await.0, "failed");
    }

    /// Contract §2 (tool layer) against a real database: a missing or malformed
    /// plan refuses with a TYPED rejection and writes NOTHING (the event count
    /// does not move), while a valid plan lands on the row AND in the
    /// `AcceptanceRequested` payload — the same transaction, one write path.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-017 (scratch DB); run --test-threads=1"]
    async fn request_acceptance_enforces_the_plan_and_lands_it() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        with_base_repo();

        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ('ev-it-verdict-plan','ReviewVerdictIssued',NOW(),'seat:seat-flux-001','{\"verdict\":\"pass\"}',$1,1)",
                &[&project],
            ).await.unwrap();
        }
        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-plan", project, 0))
            .await
            .unwrap();
        as_seat();
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-plan".into(),
                outcome: None,
                reason: None,
            },
        )
        .await
        .unwrap();

        let events_before: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);

        // --- the negative matrix, each refusing BEFORE any write
        let bad: Vec<(&str, serde_json::Value)> = vec![
            ("missing", serde_json::json!({})),
            ("null", serde_json::Value::Null),
            (
                "wrong version",
                serde_json::json!({"plan_version": 2, "scenario": "s",
                "steps": [{"action": "a"}], "expected": [{"output": "o", "expect": "e"}]}),
            ),
            (
                "unknown key",
                serde_json::json!({"plan_version": 1, "scenario": "s",
                "steps": [{"action": "a"}], "expected": [{"output": "o", "expect": "e"}],
                "surprise": 1}),
            ),
            (
                "empty steps",
                serde_json::json!({"plan_version": 1, "scenario": "s",
                "steps": [], "expected": [{"output": "o", "expect": "e"}]}),
            ),
            (
                "empty expected",
                serde_json::json!({"plan_version": 1, "scenario": "s",
                "steps": [{"action": "a"}], "expected": []}),
            ),
            (
                "blank action",
                serde_json::json!({"plan_version": 1, "scenario": "s",
                "steps": [{"action": "  "}], "expected": [{"output": "o", "expect": "e"}]}),
            ),
            (
                "blank expect",
                serde_json::json!({"plan_version": 1, "scenario": "s",
                "steps": [{"action": "a"}], "expected": [{"output": "o", "expect": " "}]}),
            ),
        ];
        for (label, plan) in bad {
            let err = request_acceptance(
                &pool,
                RequestAcceptanceArgs {
                    dispatch_id: "disp-it-plan".into(),
                    branch: "packet/nimbus/x".into(),
                    how_to_verify: "do A see B".into(),
                    flux_verdict_ref: "ev-it-verdict-plan".into(),
                    base: Some("track".into()),
                    acceptance_plan: plan,
                },
            )
            .await
            .expect_err(&format!("{label} must refuse"));
            assert!(
                matches!(
                    err,
                    DispatchRejection::MissingAcceptancePlan
                        | DispatchRejection::InvalidAcceptancePlan { .. }
                ),
                "{label}: expected a typed plan rejection, got {err:?}"
            );
        }
        let events_after_refusals: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            events_before, events_after_refusals,
            "every refusal writes nothing — the event count must not move"
        );
        let cards: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM acceptances WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(cards, 0, "no card was created by any refusal");

        // --- a valid plan lands on the row AND in the event payload
        request_acceptance(
            &pool,
            RequestAcceptanceArgs {
                dispatch_id: "disp-it-plan".into(),
                branch: "packet/nimbus/x".into(),
                how_to_verify: "do A see B".into(),
                flux_verdict_ref: "ev-it-verdict-plan".into(),
                base: Some("track".into()),
                acceptance_plan: valid_plan(),
            },
        )
        .await
        .expect("a valid plan creates the card");
        let row = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT a.acceptance_plan, e.payload \
                   FROM acceptances a \
                   JOIN canonical_events e ON e.id = ( \
                         SELECT r.event_id FROM event_object_refs r \
                          WHERE r.ref_type = 'acceptance' AND r.ref_id = a.id \
                            AND r.role = 'subject') \
                  WHERE a.id = 'acc-disp-it-plan'",
                &[],
            )
            .await
            .unwrap();
        let row_plan: serde_json::Value = row.get(0);
        let event_payload: serde_json::Value = row.get(1);
        assert_eq!(row_plan["scenario"], valid_plan()["scenario"]);
        assert_eq!(row_plan["steps"][1]["input"], "pnpm test");
        assert_eq!(
            event_payload["acceptance_plan"], row_plan,
            "the event carries the same plan as the row — one transaction"
        );
    }

    /// D5 Lane B against a real database: the whole memory lifecycle through the
    /// TOOLS — remember, recall at each depth (with the audit event and the row's
    /// heat/recall_count moving together), promote up and down, and the soft
    /// tombstone — plus every negative branch as a typed refusal that writes
    /// nothing. This is the contract's §5 acceptance matrix.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-017 (scratch DB); run --test-threads=1"]
    async fn memory_tools_lifecycle_and_refusals_against_the_db() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        clean_memories(&pool, project).await;
        as_seat();
        let db = SeatloomDb::new(pool.clone());

        // --- the scope fixture: a parent topic and a child under it, plus a
        // document the l2 pointer can name.
        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO topics (id,project_id,title,status,opened_by) \
                 VALUES ('topic-parent',$1,'parent topic','open','seat:seat-lyra-001')",
                &[&project],
            )
            .await
            .unwrap();
            c.execute(
                "INSERT INTO topics (id,project_id,title,status,opened_by,parent_topic_id) \
                 VALUES ('topic-child',$1,'child topic','open','seat:seat-lyra-001','topic-parent')",
                &[&project],
            )
            .await
            .unwrap();
            c.execute(
                "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) \
                 VALUES ('doc-mem-target',$1,'the source doc','docs/src.md','ok','{}','{}')",
                &[&project],
            )
            .await
            .unwrap();
        }

        let events = || async {
            pool.get()
                .await
                .unwrap()
                .query_one(
                    "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                    &[&project],
                )
                .await
                .unwrap()
                .get::<_, i64>(0)
        };
        let before = events().await;

        // --- remember (document pointer)
        let msg = remember(
            &db,
            RememberArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                title: "how the gate works".into(),
                l0_summary: "accept needs flux green plus the operator".into(),
                l1_overview: Some("the long version".into()),
                l2_ref_kind: "document".into(),
                l2_document_id: Some("doc-mem-target".into()),
                l2_file_path: None,
                l2_body_digest: None,
                source_kind: "document".into(),
                source_ref: "doc-mem-target".into(),
                kind: "fact".into(),
                confidence: "quoted".into(),
                supersedes: None,
                l2_line_start: None,
                l2_line_end: None,
                relations: vec![],
                tags: vec!["d2".into()],
            },
        )
        .await
        .expect("remember must succeed");
        let memory_id = msg
            .split_whitespace()
            .nth(2)
            .expect("remembered memory <id>")
            .to_string();
        assert_eq!(events().await, before + 1, "exactly one MemoryCreated");

        // The event's subject ref names the MEMORY (015's ref_type).
        // The subject ref of THIS memory's creation event (filtered by the
        // memory id — the project may hold other memories' events).
        let subject = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT r.ref_type, r.ref_id FROM event_object_refs r \
                   JOIN canonical_events e ON e.id = r.event_id \
                  WHERE e.event_type = 'MemoryCreated' AND r.role = 'subject' \
                    AND r.ref_id = $1",
                &[&memory_id],
            )
            .await
            .unwrap();
        let ref_type: String = subject.get(0);
        let ref_id: String = subject.get(1);
        assert_eq!(ref_type, "memory");
        assert_eq!(ref_id, memory_id);

        // --- recall at l0: only the l0 layer, and the audit moves the row.
        let out = recall(
            &db,
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                query: None,
                depth: None,
                include_superseded: None,
                limit: None,
            },
        )
        .await
        .expect("recall l0");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["depth"], "l0");
        assert_eq!(v["memories"].as_array().unwrap().len(), 1);
        let m = &v["memories"][0];
        assert!(m.get("l0Summary").is_some(), "l0 is always returned");
        assert!(m.get("l1Overview").is_none(), "l1 is omitted at l0");
        assert!(m.get("l2RefKind").is_none(), "l2 is omitted at l0");
        // The topic's ancestor chain rides along.
        let chain: Vec<&str> = v["topicChain"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap())
            .collect();
        assert_eq!(chain, vec!["topic-child", "topic-parent"]);
        // heat/recall_count/last_recalled_at moved, and exactly one event landed.
        let row = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT heat, recall_count, last_recalled_at FROM memories WHERE id=$1",
                &[&memory_id],
            )
            .await
            .unwrap();
        let heat: i32 = row.get(0);
        let recall_count: i32 = row.get(1);
        let last: Option<chrono::DateTime<chrono::Utc>> = row.get(2);
        assert_eq!((heat, recall_count), (1, 1));
        assert!(last.is_some(), "last_recalled_at must be set");
        assert_eq!(events().await, before + 2, "one MemoryRecalled");

        // --- l1 and l2 disclose their layers
        for (depth, field) in [("l1", "l1Overview"), ("l2", "l2RefKind")] {
            let out = recall(
                &db,
                RecallArgs {
                    project_id: Some(project.into()),
                    topic_id: "topic-child".into(),
                    query: None,
                    depth: Some(depth.into()),
                    include_superseded: None,
                    limit: None,
                },
            )
            .await
            .expect("recall at depth");
            let v: serde_json::Value = serde_json::from_str(&out).unwrap();
            assert!(
                v["memories"][0].get(field).is_some(),
                "{depth} must disclose {field}"
            );
        }
        // heat is now 3 (three recalls), each audited.
        let heat: i32 = pool
            .get()
            .await
            .unwrap()
            .query_one("SELECT heat FROM memories WHERE id=$1", &[&memory_id])
            .await
            .unwrap()
            .get(0);
        assert_eq!(heat, 3);

        // --- promote up, then down; the same tier is refused
        promote(
            &db,
            PromoteArgs {
                memory_id: memory_id.clone(),
                to_tier: "warm".into(),
            },
        )
        .await
        .expect("cold -> warm");
        let tier: String = pool
            .get()
            .await
            .unwrap()
            .query_one("SELECT tier FROM memories WHERE id=$1", &[&memory_id])
            .await
            .unwrap()
            .get(0);
        assert_eq!(tier, "warm");
        promote(
            &db,
            PromoteArgs {
                memory_id: memory_id.clone(),
                to_tier: "warm".into(),
            },
        )
        .await
        .expect_err("the same tier is no transition");
        promote(
            &db,
            PromoteArgs {
                memory_id: memory_id.clone(),
                to_tier: "cold".into(),
            },
        )
        .await
        .expect("warm -> cold (down is allowed)");
        let promoted: Vec<(String, String)> = pool
            .get()
            .await
            .unwrap()
            .query(
                "SELECT payload->>'from_tier', payload->>'to_tier' FROM canonical_events \
                  WHERE event_type='MemoryPromoted' AND project_id = $1 \
                  ORDER BY occurred_at",
                &[&project],
            )
            .await
            .unwrap()
            .iter()
            .map(|r| (r.get(0), r.get(1)))
            .collect();
        assert_eq!(
            promoted,
            vec![
                ("cold".to_string(), "warm".to_string()),
                ("warm".to_string(), "cold".to_string())
            ]
        );

        // --- forget: soft tombstone, row stays, second forget refused
        forget(
            &db,
            ForgetArgs {
                memory_id: memory_id.clone(),
                reason: "superseded by the newer note".into(),
            },
        )
        .await
        .expect("forget");
        let row = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT status, forgotten_reason FROM memories WHERE id=$1",
                &[&memory_id],
            )
            .await
            .unwrap();
        let status: String = row.get(0);
        let reason: Option<String> = row.get(1);
        assert_eq!(status, "forgotten");
        assert_eq!(reason.as_deref(), Some("superseded by the newer note"));
        forget(
            &db,
            ForgetArgs {
                memory_id: memory_id.clone(),
                reason: "again".into(),
            },
        )
        .await
        .expect_err("a tombstone is immutable");
        // A forgotten memory is excluded from recall.
        let out = recall(
            &db,
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                query: None,
                depth: None,
                include_superseded: None,
                limit: None,
            },
        )
        .await
        .expect("recall after forget");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["memories"].as_array().unwrap().len(),
            0,
            "a forgotten memory is not recalled"
        );
        assert_eq!(v["recalled"], 0, "zero hits = zero events");

        // --- the negative branches, each a typed refusal that writes nothing
        let settled = events().await;
        for bad in [
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                query: None,
                depth: Some("l3".into()),
                include_superseded: None,
                limit: None,
            },
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                query: None,
                depth: None,
                include_superseded: None,
                limit: Some(101),
            },
        ] {
            recall(&db, bad).await.expect_err("must refuse");
        }
        remember(
            &db,
            RememberArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                title: "t".into(),
                l0_summary: "s".into(),
                l1_overview: None,
                l2_ref_kind: "file".into(),
                l2_document_id: None,
                l2_file_path: None,
                l2_body_digest: None,
                source_kind: "manual".into(),
                source_ref: "note".into(),
                kind: "fact".into(),
                confidence: "quoted".into(),
                supersedes: None,
                l2_line_start: None,
                l2_line_end: None,
                relations: vec![],
                tags: vec![],
            },
        )
        .await
        .expect_err("a file pointer without a path is refused");
        remember(
            &db,
            RememberArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                title: "t".into(),
                l0_summary: "s".into(),
                l1_overview: None,
                l2_ref_kind: "document".into(),
                l2_document_id: Some("doc-mem-target".into()),
                l2_file_path: Some("docs/x.md".into()),
                l2_body_digest: None,
                source_kind: "vibes".into(),
                source_ref: "note".into(),
                kind: "fact".into(),
                confidence: "quoted".into(),
                supersedes: None,
                l2_line_start: None,
                l2_line_end: None,
                relations: vec![],
                tags: vec![],
            },
        )
        .await
        .expect_err("an unknown source_kind is refused");
        promote(
            &db,
            PromoteArgs {
                memory_id: memory_id.clone(),
                to_tier: "tepid".into(),
            },
        )
        .await
        .expect_err("an unknown tier is refused");
        assert_eq!(events().await, settled, "every refusal above wrote nothing");
    }

    /// 018 (D5 Phase 2) against a real database: `remember` writes kind /
    /// confidence / the L2 line range and the supersede edge in BOTH spellings
    /// (the column and a `memory_relations` row) in one transaction; `recall`
    /// hides superseded rows by default and annotates them when opted in;
    /// `resolve` closes an ACTIVE record with `MemoryResolved`. Every malformed
    /// input is a typed refusal that moves no ledger row.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-018 (scratch DB); run --test-threads=1"]
    async fn memory_tools_018_typed_fields_supersede_and_resolve() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        clean_memories(&pool, project).await;
        as_seat();
        let db = SeatloomDb::new(pool.clone());

        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO topics (id,project_id,title,status,opened_by) \
                 VALUES ('topic-018',$1,'018 topic','open','seat:seat-lyra-001')",
                &[&project],
            )
            .await
            .unwrap();
        }

        // --- the negative matrix, each a typed refusal with zero writes
        let events_before: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        let base = || RememberArgs {
            project_id: Some(project.into()),
            topic_id: "topic-018".into(),
            title: "t".into(),
            l0_summary: "s".into(),
            l1_overview: None,
            l2_ref_kind: "document".into(),
            l2_document_id: None,
            l2_file_path: None,
            l2_body_digest: None,
            source_kind: "manual".into(),
            source_ref: "note".into(),
            kind: "fact".into(),
            confidence: "quoted".into(),
            supersedes: None,
            l2_line_start: None,
            l2_line_end: None,
            relations: vec![],
            tags: vec![],
        };
        // a document pointer needs a document id
        remember(&db, base())
            .await
            .expect_err("document pointer without an id");
        // kind out of the 6-value vocab
        let mut a = base();
        a.kind = "note".into();
        remember(&db, a).await.expect_err("kind out of vocab");
        // confidence out of the 2-value vocab
        let mut a = base();
        a.confidence = "0.9".into();
        remember(&db, a).await.expect_err("confidence is binary");
        // file + archived_session without the line range
        let mut a = base();
        a.l2_ref_kind = "file".into();
        a.l2_file_path = Some("docs/x.md".into());
        a.source_kind = "archived_session".into();
        a.source_ref = "claude:ses-1".into();
        remember(&db, a)
            .await
            .expect_err("the l2 line range is required");
        // ... and an inverted one
        let mut a = base();
        a.l2_ref_kind = "file".into();
        a.l2_file_path = Some("docs/x.md".into());
        a.source_kind = "archived_session".into();
        a.source_ref = "claude:ses-1".into();
        a.l2_line_start = Some(20);
        a.l2_line_end = Some(10);
        remember(&db, a).await.expect_err("end before start");
        // a relation to a memory that does not exist
        let mut a = base();
        a.l2_document_id = Some("doc-018".into());
        a.relations = vec![MemoryRelationArg {
            to_id: "mem-nope".into(),
            rel: "caused_by".into(),
        }];
        remember(&db, a).await.expect_err("a relation to nothing");
        let events_after: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(events_before, events_after, "every refusal wrote nothing");

        // --- a valid record, with kind/confidence/l2 lines
        let c = pool.get().await.unwrap();
        c.execute(
            "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) \
             VALUES ('doc-018',$1,'the doc','docs/018.md','ok','{}','{}')",
            &[&project],
        )
        .await
        .unwrap();
        drop(c);
        let mut a = base();
        a.l2_document_id = Some("doc-018".into());
        a.kind = "failure".into();
        a.confidence = "quoted".into();
        let msg = remember(&db, a).await.expect("a valid record");
        let old_id = msg.split_whitespace().nth(2).unwrap().to_string();
        let row = db.get_memory(&old_id).await.unwrap().unwrap();
        assert_eq!(row.kind, "failure");
        assert_eq!(row.confidence, "quoted");
        assert_eq!(row.l2_document_id.as_deref(), Some("doc-018"));

        // --- supersede: BOTH spellings, one transaction
        let mut a = base();
        a.l2_document_id = Some("doc-018".into());
        a.kind = "decision".into();
        a.supersedes = Some(old_id.clone());
        let msg = remember(&db, a).await.expect("the superseding record");
        let new_id = msg.split_whitespace().nth(2).unwrap().to_string();
        let (col, rel_rows): (Option<String>, i64) = {
            let c = pool.get().await.unwrap();
            let r = c
                .query_one("SELECT supersedes FROM memories WHERE id = $1", &[&new_id])
                .await
                .unwrap();
            let n = c
                .query_one(
                    "SELECT count(*) FROM memory_relations \
                      WHERE from_id = $1 AND to_id = $2 AND rel = 'supersedes'",
                    &[&new_id, &old_id],
                )
                .await
                .unwrap()
                .get(0);
            (r.get(0), n)
        };
        assert_eq!(col.as_deref(), Some(old_id.as_str()), "the column spelling");
        assert_eq!(rel_rows, 1, "the relation spelling");

        // --- recall hides the superseded row by default, annotates when opted in
        let out = recall(
            &db,
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-018".into(),
                query: None,
                depth: None,
                include_superseded: None,
                limit: None,
            },
        )
        .await
        .expect("recall");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let ids: Vec<&str> = v["memories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec![new_id.as_str()], "the superseded row is hidden");
        assert_eq!(v["includeSuperseded"], false);

        let out = recall(
            &db,
            RecallArgs {
                project_id: Some(project.into()),
                topic_id: "topic-018".into(),
                query: None,
                depth: None,
                include_superseded: Some(true),
                limit: None,
            },
        )
        .await
        .expect("recall with superseded");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let superseded = v["memories"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == old_id.as_str())
            .expect("the superseded row is now visible");
        assert_eq!(
            superseded["supersededBy"].as_str(),
            Some(new_id.as_str()),
            "and annotated with its replacement"
        );

        // --- resolve: active -> resolved + MemoryResolved, and the refusals
        let out = resolve(
            &db,
            ResolveArgs {
                memory_id: new_id.clone(),
            },
        )
        .await
        .expect("resolve");
        assert!(out.contains("resolved memory"), "{out}");
        let row = db.get_memory(&new_id).await.unwrap().unwrap();
        assert_eq!(row.status, "resolved");
        assert_eq!(row.resolved_by.as_deref(), Some("seat:seat-nimbus-001"));
        assert!(row.resolved_at.is_some());
        let resolved_events: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1 \
                  AND event_type = 'MemoryResolved'",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(resolved_events, 1);

        let err = resolve(
            &db,
            ResolveArgs {
                memory_id: new_id.clone(),
            },
        )
        .await
        .expect_err("a resolved record is immutable");
        assert!(
            err.to_string().contains("resolved"),
            "the refusal names the status: {err}"
        );
        resolve(
            &db,
            ResolveArgs {
                memory_id: "mem-nope".into(),
            },
        )
        .await
        .expect_err("an unknown id");
    }

    /// The verdict-event-id contract (Aegis ruling 2026-10-11 part 2): a
    /// verification request gets a pre-declared verdict id MININTED into its
    /// payload (or a supplied one VALIDATED), the id is returned in the result,
    /// and a duplicate or malformed id is refused. A non-verification payload
    /// carrying the argument is refused too.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-019 (scratch DB); run --test-threads=1"]
    async fn enqueue_mints_and_validates_the_verdict_event_id() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        as_actor("human:supervisor");

        // 1. A verification request gets a minted id, written into the payload
        //    and returned in the result.
        let msg = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-1".into()),
                target_seat_id: SEAT_ID.into(),
                title: "verify me".into(),
                payload: serde_json::json!({"kind": "verification", "target_commit": "abc"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: None,
            },
        )
        .await
        .expect("a verification request enqueues");
        assert!(
            msg.contains("verdict_event_id=ev-"),
            "the minted id is returned: {msg}"
        );
        assert!(
            msg.contains("dep_kind=event"),
            "the gating instruction: {msg}"
        );
        let stored: serde_json::Value = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT payload FROM dispatches WHERE id = 'disp-verdict-1'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        let minted = stored["verdict_event_id"]
            .as_str()
            .expect("the id rides in the payload")
            .to_string();
        assert!(minted.starts_with("ev-"), "{minted}");

        // 2. The same id cannot be pre-declared twice (one gate, one verdict).
        let err = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-2".into()),
                target_seat_id: SEAT_ID.into(),
                title: "verify me too".into(),
                payload: serde_json::json!({"kind": "verification"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: Some(minted.clone()),
            },
        )
        .await
        .expect_err("a taken id must be refused");
        assert!(
            matches!(err, DispatchRejection::VerdictIdTaken { .. }),
            "{err}"
        );

        // 3. A malformed id is refused, not normalised.
        let err = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-3".into()),
                target_seat_id: SEAT_ID.into(),
                title: "verify me".into(),
                payload: serde_json::json!({"kind": "verification"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: Some("not-an-event-id".into()),
            },
        )
        .await
        .expect_err("a malformed id must be refused");
        assert!(
            matches!(err, DispatchRejection::VerdictIdMalformed { .. }),
            "{err}"
        );

        // 4. The argument is refused for a non-verification payload.
        let err = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-4".into()),
                target_seat_id: SEAT_ID.into(),
                title: "ordinary work".into(),
                payload: serde_json::json!({"kind": "implementation"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: Some("ev-something".into()),
            },
        )
        .await
        .expect_err("the pre-declaration belongs to the verification path");
        assert!(
            matches!(err, DispatchRejection::VerdictIdNotAVerification { .. }),
            "{err}"
        );

        // 5. An ordinary payload is untouched: no id in the payload, no mention
        //    in the result.
        let msg = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-5".into()),
                target_seat_id: SEAT_ID.into(),
                title: "ordinary work".into(),
                payload: serde_json::json!({"kind": "implementation"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: None,
            },
        )
        .await
        .expect("an ordinary enqueue");
        assert!(!msg.contains("verdict_event_id"), "{msg}");
        let stored: serde_json::Value = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT payload FROM dispatches WHERE id = 'disp-verdict-5'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert!(stored.get("verdict_event_id").is_none());

        // 6. A well-formed caller-supplied id is accepted as-is.
        let msg = enqueue(
            &pool,
            EnqueueArgs {
                id: Some("disp-verdict-6".into()),
                target_seat_id: SEAT_ID.into(),
                title: "verify me".into(),
                payload: serde_json::json!({"kind": "verification"}),
                priority: Some(5),
                project_id: Some(project.into()),
                dependencies: vec![],
                max_retries: Some(0),
                verdict_event_id: Some("ev-mine-1".into()),
            },
        )
        .await
        .expect("a well-formed supplied id");
        assert!(msg.contains("verdict_event_id=ev-mine-1"), "{msg}");
    }

    /// `open_topic` (Aegis ruling 2026-10-11): the governed creation of a recall
    /// scope. Row + `TopicOpened` in one transaction; idempotent by id; and
    /// every malformed call a typed refusal with zero writes.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-019 (scratch DB); run --test-threads=1"]
    async fn open_topic_creates_idempotently_and_refuses_malformed_calls() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        clean_memories(&pool, project).await;
        as_seat();
        let db = SeatloomDb::new(pool.clone());

        // The negative matrix, each a typed refusal with zero writes.
        let events_before: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        let base = |parent: Option<&str>| topics::OpenTopicArgs {
            topic_id: "topic-root".into(),
            title: "the root".into(),
            project_id: Some(project.into()),
            parent_topic_id: parent.map(str::to_string),
        };
        // parent that does not exist
        topics::open_topic(&db, base(Some("topic-nope")))
            .await
            .expect_err("a parent that does not exist");
        // project that does not exist
        topics::open_topic(
            &db,
            topics::OpenTopicArgs {
                topic_id: "topic-root".into(),
                title: "t".into(),
                project_id: Some("no-such-project".into()),
                parent_topic_id: None,
            },
        )
        .await
        .expect_err("a project that does not exist");
        // self-cycle
        topics::open_topic(
            &db,
            topics::OpenTopicArgs {
                topic_id: "topic-root".into(),
                title: "t".into(),
                project_id: Some(project.into()),
                parent_topic_id: Some("topic-root".into()),
            },
        )
        .await
        .expect_err("a self-cycle");
        let events_after: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(events_before, events_after, "every refusal wrote nothing");

        // The happy path: row + event, opened_by from the server-side identity.
        let msg = topics::open_topic(&db, base(None))
            .await
            .expect("open the root");
        assert!(msg.contains("opened topic topic-root"), "{msg}");
        let row = db.get_topic("topic-root").await.unwrap().unwrap();
        assert_eq!(row.project_id, project);
        assert_eq!(row.status, "open");
        let opened_by: String = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT r.ref_id FROM event_object_refs r \
                   JOIN canonical_events e ON e.id = r.event_id \
                  WHERE e.event_type = 'TopicOpened' AND e.project_id = $1 \
                    AND r.role = 'actor' AND r.ref_type = 'seat'",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(opened_by, "seat:seat-nimbus-001");
        let subject = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT r.ref_type, r.ref_id FROM event_object_refs r \
                   JOIN canonical_events e ON e.id = r.event_id \
                  WHERE e.event_type = 'TopicOpened' AND e.project_id = $1 \
                    AND r.role = 'subject'",
                &[&project],
            )
            .await
            .unwrap();
        let ref_type: String = subject.get(0);
        let ref_id: String = subject.get(1);
        assert_eq!(
            (ref_type.as_str(), ref_id.as_str()),
            ("topic", "topic-root")
        );

        // Idempotent: a second open succeeds, changes nothing, emits nothing.
        let msg = topics::open_topic(&db, base(None))
            .await
            .expect("re-open is idempotent");
        assert!(msg.contains("already open"), "{msg}");
        let events_now: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM canonical_events WHERE project_id = $1 \
                  AND event_type = 'TopicOpened'",
                &[&project],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(events_now, 1, "no second event");

        // A child, then the cross-project and cycle refusals.
        let msg = topics::open_topic(
            &db,
            topics::OpenTopicArgs {
                topic_id: "topic-child".into(),
                title: "the child".into(),
                project_id: Some(project.into()),
                parent_topic_id: Some("topic-root".into()),
            },
        )
        .await
        .expect("open the child");
        assert!(msg.contains("opened topic topic-child"), "{msg}");

        // A second project, and a topic in it: parenting across is refused.
        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
                 VALUES ('other-project','other',NOW(),1024,1024) \
                 ON CONFLICT (id) DO NOTHING",
                &[],
            )
            .await
            .unwrap();
            c.execute(
                "INSERT INTO topics (id,project_id,title,opened_by) \
                 VALUES ('topic-other','other-project','other','seat:seat-lyra-001') \
                 ON CONFLICT (id) DO NOTHING",
                &[],
            )
            .await
            .unwrap();
        }
        topics::open_topic(
            &db,
            topics::OpenTopicArgs {
                topic_id: "topic-cross".into(),
                title: "cross".into(),
                project_id: Some(project.into()),
                parent_topic_id: Some("topic-other".into()),
            },
        )
        .await
        .expect_err("a scope tree does not straddle projects");

        // With open_topic live, remember finally works (the FK is satisfiable).
        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) \
                 VALUES ('doc-open-topic',$1,'the doc','docs/open-topic.md','ok','{}','{}') \
                 ON CONFLICT (id) DO NOTHING",
                &[&project],
            )
            .await
            .unwrap();
        }
        let msg = remember(
            &db,
            RememberArgs {
                project_id: Some(project.into()),
                topic_id: "topic-child".into(),
                title: "a memory".into(),
                l0_summary: "the summary".into(),
                l1_overview: None,
                l2_ref_kind: "document".into(),
                l2_document_id: Some("doc-open-topic".into()),
                l2_file_path: None,
                l2_body_digest: None,
                source_kind: "manual".into(),
                source_ref: "note".into(),
                kind: "fact".into(),
                confidence: "quoted".into(),
                supersedes: None,
                l2_line_start: None,
                l2_line_end: None,
                relations: vec![],
                tags: vec![],
            },
        )
        .await
        .expect("remember works once a topic exists");
        assert!(msg.contains("remembered memory"), "{msg}");
    }

    /// The two pure reads against a real database: `get_dispatch` answers with
    /// the row, its LIVE status and its edges; `query_ledger` filters and pages.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-017 (scratch DB); run --test-threads=1"]
    async fn read_tools_answer_from_the_ledger() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        as_seat();
        // One in_flight dispatch per seat, GLOBALLY: drain whatever an earlier
        // test left this seat holding before pulling this test's own dispatch.
        let held = get_next(&pool, Some(project.into())).await.unwrap();
        if let Some(id) = held
            .split_whitespace()
            .nth(1)
            .filter(|_| held.contains("delivered"))
        {
            complete(
                &pool,
                CompleteArgs {
                    dispatch_id: id.to_string(),
                    outcome: None,
                    reason: None,
                },
            )
            .await
            .unwrap();
        }
        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-read", project, 0))
            .await
            .unwrap();
        as_seat();
        get_next(&pool, Some(project.into())).await.unwrap();
        let db = SeatloomDb::new(pool.clone());

        // get_dispatch: in_flight is returned as stored (never recomputed).
        let out = read::get_dispatch(
            &db,
            read::GetDispatchArgs {
                dispatch_id: "disp-it-read".into(),
            },
        )
        .await
        .expect("the dispatch read model");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["status"], "in_flight");
        assert_eq!(v["liveStatus"], "in_flight");
        assert_eq!(v["title"], "itest");

        // An unknown id is a typed refusal, not an empty object.
        read::get_dispatch(
            &db,
            read::GetDispatchArgs {
                dispatch_id: "disp-nope".into(),
            },
        )
        .await
        .expect_err("unknown dispatch");

        // query_ledger: filter by type, and the ref filter as a pair.
        let out = read::query_ledger(
            &db,
            read::QueryLedgerArgs {
                project_id: Some(project.into()),
                event_types: vec!["SeatDispatchIssued".into(), "SeatDispatchDelivered".into()],
                ref_type: None,
                ref_id: None,
                from: None,
                to: None,
                limit: None,
                offset: None,
            },
        )
        .await
        .expect("the ledger query");
        let events: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(events.len(), 2, "issued + delivered");
        assert_eq!(events[0]["eventType"], "SeatDispatchDelivered");

        // A half ref filter is a malformed call, not a wider query.
        read::query_ledger(
            &db,
            read::QueryLedgerArgs {
                project_id: None,
                event_types: vec![],
                ref_type: Some("dispatch".into()),
                ref_id: None,
                from: None,
                to: None,
                limit: None,
                offset: None,
            },
        )
        .await
        .expect_err("half a ref filter");
        // An unknown event type is refused by the hand-written FromStr.
        read::query_ledger(
            &db,
            read::QueryLedgerArgs {
                project_id: None,
                event_types: vec!["SeatDispatchReopened".into()],
                ref_type: None,
                ref_id: None,
                from: None,
                to: None,
                limit: None,
                offset: None,
            },
        )
        .await
        .expect_err("an unknown event type");
    }

    /// Defect 2 (Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J): a pending card can
    /// be withdrawn by the seat that requested it, so "this card was satisfied
    /// by other means" is a RECORDED transition instead of a hand-edited row.
    /// No merge, no rework; the row and the ledger agree, and a second
    /// withdrawal is refused because cancelled is terminal.
    #[tokio::test]
    #[ignore = "requires PostgreSQL 001-016 (scratch DB); run --test-threads=1"]
    async fn cancel_acceptance_withdraws_own_pending_card_and_is_terminal() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let project = "seatloom_itest";
        clean(&pool, project).await;
        with_base_repo();

        {
            let c = pool.get().await.unwrap();
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ('ev-it-verdict-c','ReviewVerdictIssued',NOW(),'seat:seat-flux-001','{\"verdict\":\"pass\"}',$1,1)",
                &[&project],
            ).await.unwrap();
        }
        as_actor("human:supervisor");
        enqueue(&pool, enqueue_args("disp-it-cancel", project, 0))
            .await
            .unwrap();
        as_seat();
        get_next(&pool, Some(project.into())).await.unwrap();
        complete(
            &pool,
            CompleteArgs {
                dispatch_id: "disp-it-cancel".into(),
                outcome: None,
                reason: None,
            },
        )
        .await
        .unwrap();
        request_acceptance(
            &pool,
            RequestAcceptanceArgs {
                dispatch_id: "disp-it-cancel".into(),
                branch: "packet/nimbus/x".into(),
                how_to_verify: "do A see B".into(),
                flux_verdict_ref: "ev-it-verdict-c".into(),
                base: Some("track".into()),
                acceptance_plan: valid_plan(),
            },
        )
        .await
        .unwrap();

        // The seat withdraws its own pending card.
        let msg = cancel_acceptance(
            &pool,
            CancelAcceptanceArgs {
                acceptance_id: "acc-disp-it-cancel".into(),
                reason: "already landed via the bootstrap install".into(),
            },
        )
        .await
        .expect("the requesting seat may withdraw its own card");
        assert!(
            msg.contains("cancelled acceptance acc-disp-it-cancel"),
            "{msg}"
        );

        // The row is terminal, the reason is recorded, and the event landed.
        let c = pool.get().await.unwrap();
        let row = c
            .query_one(
                "SELECT status, feedback, decided_by FROM acceptances WHERE id='acc-disp-it-cancel'",
                &[],
            )
            .await
            .unwrap();
        let status: String = row.get(0);
        let feedback: Option<String> = row.get(1);
        let decided_by: Option<String> = row.get(2);
        assert_eq!(status, "cancelled");
        assert_eq!(
            feedback.as_deref(),
            Some("already landed via the bootstrap install")
        );
        assert_eq!(decided_by.as_deref(), Some(SEAT));
        let events = c
            .query(
                "SELECT e.event_type FROM canonical_events e \
                   JOIN event_object_refs r ON r.event_id=e.id AND r.role='subject' \
                  WHERE r.ref_type='acceptance' AND r.ref_id='acc-disp-it-cancel' \
                  ORDER BY e.occurred_at",
                &[],
            )
            .await
            .unwrap()
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect::<Vec<_>>();
        assert_eq!(events, vec!["AcceptanceRequested", "AcceptanceCancelled"]);

        // Terminal is immutable: a second withdrawal is refused, and nothing is
        // rewritten. No rework dispatch was created either — a void is not a
        // reject.
        let err = cancel_acceptance(
            &pool,
            CancelAcceptanceArgs {
                acceptance_id: "acc-disp-it-cancel".into(),
                reason: "again".into(),
            },
        )
        .await
        .expect_err("a cancelled acceptance is terminal");
        assert!(
            matches!(err, DispatchRejection::AcceptanceNotCancellable { .. }),
            "{err}"
        );
        let reworks: i64 = c
            .query_one(
                "SELECT count(*) FROM dispatches WHERE payload->>'rework_of' = 'disp-it-cancel'",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(reworks, 0, "a void creates no rework");

        // Another seat cannot withdraw it either (authority is the requester).
        std::env::set_var(ACTOR_REF_ENV, "seat:seat-mira-001");
        let err = cancel_acceptance(
            &pool,
            CancelAcceptanceArgs {
                acceptance_id: "acc-disp-it-cancel".into(),
                reason: "not mine".into(),
            },
        )
        .await
        .expect_err("only the requesting seat may withdraw");
        assert!(
            matches!(err, DispatchRejection::AcceptanceNotCancellable { .. }),
            "{err}"
        );
    }
}
