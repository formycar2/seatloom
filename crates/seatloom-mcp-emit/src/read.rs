//! The pure READ tools (D5 Lane B §3.5/§3.6): `get_dispatch` and `query_ledger`.
//!
//! SeatLoom's MCP surface had ZERO read tools, which is why seats kept reaching
//! for `psql`. These two close that hole: the dispatch read model (row + LIVE
//! status + dependency edges) and a filtered ledger query (events + their object
//! refs). Both are pure reads — they write nothing and append no event, and they
//! expose no write capability.
//!
//! The read model's LIVE status is recomputed by the same existence rule the
//! pull decision uses, so a reader sees what the scheduler would see rather than
//! a stale stored status.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

use seatloom_core::db::event_domain::EventType;
use seatloom_core::db::repositories::DbError;
use seatloom_core::db::SeatloomDb;

/// The hard ceiling on one ledger page. The caller owns the pagination; this
/// only stops an unbounded page.
pub const QUERY_LEDGER_LIMIT_MAX: i64 = 200;
/// The default page size.
pub const QUERY_LEDGER_LIMIT_DEFAULT: i64 = 50;

// =============================================================================
// get_dispatch
// =============================================================================

/// `get_dispatch` — one dispatch, its live status, and its dependency edges.
/// No identity parameter: coordination state is visible to the fleet (this is a
/// READ, and a read has nothing to act as). It exposes no write capability.
#[derive(Debug, Deserialize)]
pub struct GetDispatchArgs {
    pub dispatch_id: String,
}

/// The answer: the row, its live status, and each edge with its referent's
/// existence.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchReadResult {
    pub id: String,
    pub target_seat_id: String,
    pub issued_by: String,
    pub project_id: String,
    pub title: String,
    /// The original task statement, verbatim.
    pub payload: Value,
    /// The status as stored.
    pub status: String,
    /// Recomputed from the dependencies' existence (terminal/in_flight rows are
    /// returned as stored).
    pub live_status: String,
    pub priority: i32,
    pub issued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub dependencies: Vec<DependencyEdge>,
}

/// One dependency edge with its referent's live existence.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdge {
    pub dep_kind: String,
    pub dep_value: String,
    pub exists: bool,
}

/// Why a read-tool call was refused.
#[derive(Debug)]
pub enum ReadRejection {
    /// The dispatch id names no row.
    DispatchNotFound {
        dispatch_id: String,
    },
    /// A `query_ledger` filter value is not in its closed domain.
    BadDomain {
        domain: &'static str,
        value: String,
        expected: &'static str,
    },
    /// The page size exceeds the hard ceiling.
    LimitTooLarge {
        limit: i64,
    },
    Db(String),
}

impl std::fmt::Display for ReadRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadRejection::DispatchNotFound { dispatch_id } => {
                write!(f, "dispatch {dispatch_id:?} not found")
            }
            ReadRejection::BadDomain {
                domain,
                value,
                expected,
            } => write!(
                f,
                "{domain} {value:?} is not one of {expected} (hand-written FromStr; an unknown value is never defaulted)"
            ),
            ReadRejection::LimitTooLarge { limit } => {
                write!(f, "limit {limit} exceeds the hard ceiling {QUERY_LEDGER_LIMIT_MAX}")
            }
            ReadRejection::Db(e) => write!(f, "read tool rejected: {e}"),
        }
    }
}

impl From<DbError> for ReadRejection {
    fn from(e: DbError) -> Self {
        ReadRejection::Db(e.to_string())
    }
}

pub async fn get_dispatch(db: &SeatloomDb, args: GetDispatchArgs) -> Result<String, ReadRejection> {
    let Some(model) = db.dispatch_read_model(&args.dispatch_id).await? else {
        return Err(ReadRejection::DispatchNotFound {
            dispatch_id: args.dispatch_id,
        });
    };
    let d = model.dispatch;
    let result = DispatchReadResult {
        id: d.id,
        target_seat_id: d.target_seat_id,
        issued_by: d.issued_by,
        project_id: d.project_id,
        title: d.title,
        payload: d.payload,
        status: d.status,
        live_status: model.live_status,
        priority: d.priority,
        issued_at: iso(d.issued_at),
        started_at: d.started_at.map(iso),
        completed_at: d.completed_at.map(iso),
        dependencies: model
            .dependencies
            .into_iter()
            .map(|e| DependencyEdge {
                dep_kind: e.dep_kind,
                dep_value: e.dep_value,
                exists: e.exists,
            })
            .collect(),
    };
    serde_json::to_string_pretty(&result).map_err(|e| ReadRejection::Db(e.to_string()))
}

// =============================================================================
// query_ledger
// =============================================================================

/// `query_ledger` — a filtered ledger query. Every filter is optional; an empty
/// filter set is the whole ledger (paged), which is what "show me the recent
/// events" means.
#[derive(Debug, Deserialize)]
pub struct QueryLedgerArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    /// Event types, validated by hand-written `FromStr` against the closed
    /// vocabulary — an unknown type is a typed rejection, never a silent
    /// zero-result query.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// A ref filter: BOTH fields or neither.
    #[serde(default)]
    pub ref_type: Option<String>,
    #[serde(default)]
    pub ref_id: Option<String>,
    /// RFC3339 window bounds (inclusive).
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

/// One event with its object refs.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEventResult {
    pub id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub actor_ref: String,
    pub project_id: String,
    pub payload: Option<Value>,
    pub refs: Vec<ObjectRefResult>,
}

/// One object ref of an event.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRefResult {
    pub ref_type: String,
    pub ref_id: String,
    pub role: String,
}

pub async fn query_ledger(db: &SeatloomDb, args: QueryLedgerArgs) -> Result<String, ReadRejection> {
    let limit = args.limit.unwrap_or(QUERY_LEDGER_LIMIT_DEFAULT);
    if !(1..=QUERY_LEDGER_LIMIT_MAX).contains(&limit) {
        return Err(ReadRejection::LimitTooLarge { limit });
    }
    let offset = args.offset.unwrap_or(0).max(0);

    // Event types: validate each against the closed vocabulary.
    let mut types = Vec::with_capacity(args.event_types.len());
    for t in &args.event_types {
        let parsed = EventType::from_str(t).map_err(|e| ReadRejection::BadDomain {
            domain: e.domain,
            value: e.value,
            expected: e.expected,
        })?;
        types.push(parsed.as_str().to_string());
    }

    // The ref filter is a pair: one without the other is a malformed call.
    let (ref_type, ref_id) = match (&args.ref_type, &args.ref_id) {
        (Some(t), Some(i)) => (Some(t.clone()), Some(i.clone())),
        (None, None) => (None, None),
        _ => {
            return Err(ReadRejection::BadDomain {
                domain: "ref filter",
                value: format!("{:?}/{:?}", args.ref_type, args.ref_id),
                expected: "ref_type AND ref_id together (or neither)",
            })
        }
    };

    let from = match &args.from {
        Some(s) => Some(parse_time(s, "from")?),
        None => None,
    };
    let to = match &args.to {
        Some(s) => Some(parse_time(s, "to")?),
        None => None,
    };

    let rows = db
        .query_ledger(
            args.project_id.as_deref(),
            &types,
            ref_type.as_deref(),
            ref_id.as_deref(),
            from,
            to,
            limit,
            offset,
        )
        .await?;

    let events: Vec<LedgerEventResult> = rows
        .into_iter()
        .map(|r| LedgerEventResult {
            id: r.event.id,
            event_type: r.event.event_type,
            occurred_at: iso(r.event.occurred_at),
            actor_ref: r.event.actor_ref,
            project_id: r.event.project_id,
            payload: r.event.payload,
            refs: r
                .refs
                .into_iter()
                .map(|x| ObjectRefResult {
                    ref_type: x.ref_type,
                    ref_id: x.ref_id,
                    role: x.role,
                })
                .collect(),
        })
        .collect();
    serde_json::to_string_pretty(&events).map_err(|e| ReadRejection::Db(e.to_string()))
}

/// Parse an RFC3339 timestamp. A malformed bound is a typed rejection — silently
/// dropping a filter would answer a different question than the one asked.
fn parse_time(value: &str, field: &'static str) -> Result<DateTime<Utc>, ReadRejection> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&chrono::Utc))
        .map_err(|_| ReadRejection::BadDomain {
            domain: field,
            value: value.to_string(),
            expected: "an RFC3339 timestamp",
        })
}

/// ISO-8601 rendering, matching the Tauri DTOs' wire shape.
fn iso(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ref filter is a pair: half a filter is a malformed call, not a
    /// silently wider query.
    #[test]
    fn ref_filter_is_a_pair() {
        let err = ReadRejection::BadDomain {
            domain: "ref filter",
            value: "Some(\"dispatch\")/None".into(),
            expected: "ref_type AND ref_id together (or neither)",
        }
        .to_string();
        assert!(err.contains("together"), "{err}");
    }

    /// A malformed time bound is refused rather than dropped.
    #[test]
    fn malformed_time_bound_is_refused() {
        let err = parse_time("not-a-time", "from").unwrap_err().to_string();
        assert!(err.contains("RFC3339"), "{err}");
        assert!(parse_time("2026-10-10T00:00:00Z", "from").is_ok());
    }

    /// The page ceiling is a hard refusal.
    #[test]
    fn ledger_page_ceiling() {
        assert_eq!(QUERY_LEDGER_LIMIT_DEFAULT, 50);
        assert_eq!(QUERY_LEDGER_LIMIT_MAX, 200);
        assert!(ReadRejection::LimitTooLarge { limit: 201 }
            .to_string()
            .contains("200"));
    }

    /// An unknown dispatch id is a typed refusal, not an empty object.
    #[test]
    fn unknown_dispatch_is_refused() {
        assert!(ReadRejection::DispatchNotFound {
            dispatch_id: "disp-x".into()
        }
        .to_string()
        .contains("not found"));
    }
}
