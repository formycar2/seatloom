//! The D5 knowledge-layer memory tools (Lane B): `recall` / `remember` /
//! `promote` / `forget`.
//!
//! Memory management is EXPLICIT TOOLS, never a background daemon (value-domain
//! contract §4/§8): there is no automatic heat decay and no auto-promotion
//! anywhere in this crate. `heat` moves only when a `recall` happens, and a tier
//! moves only when someone calls `promote` with a target tier.
//!
//! Every write goes through `seatloom_core`'s repository methods, which append
//! the matching event in the SAME transaction (the 012 dispatch pattern,
//! `append_event_in_tx`) — there is no second write path and no direct SQL by
//! seats. A `memories` row is a projection of the ledger, exactly like
//! `dispatches` and `acceptances`.
//!
//! Identity is resolved from `SEATLOOM_ACTOR_REF` server-side (never a
//! parameter), so the actor on every memory event is the seat that actually made
//! the call — the same non-forgeable rule the dispatch tools follow.

use serde::Deserialize;

use seatloom_core::db::repositories::DbError;
use seatloom_core::db::SeatloomDb;
use seatloom_core::objects::memory::{
    L2RefKind, MemoryConfidence, MemoryKind, MemoryLayer, MemoryRelation, MemorySourceKind,
    MemoryTier,
};

use crate::dispatch::{resolve_project, resolve_seat, DispatchRejection};

/// The hard ceiling on one recall's page size. The contract gives the caller the
/// token budget; this only stops an unbounded page.
pub const RECALL_LIMIT_MAX: i64 = 100;
/// The default page size when the caller does not say.
pub const RECALL_LIMIT_DEFAULT: i64 = 20;

// =============================================================================
// Tool argument shapes
// =============================================================================

/// `recall` — progressive-disclosure retrieval. `project_id` + `topic_id` are
/// BOTH part of the scope: project + topic is the ONLY scoping framework, so
/// there is no "all projects" recall to invent.
#[derive(Debug, Deserialize)]
pub struct RecallArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    pub topic_id: String,
    /// Text filter over `title` + `l0_summary` (002's FTS pattern). Omitted =
    /// every active memory of the topic.
    #[serde(default)]
    pub query: Option<String>,
    /// 'l0' (default) | 'l1' | 'l2'. Parsed by hand — an unknown depth is a
    /// typed rejection, never a serde fallback.
    #[serde(default)]
    pub depth: Option<String>,
    /// Opt in to superseded rows (018). They are excluded by default — a stale
    /// decision served as current is worse than no memory — and, when included,
    /// each carries `superseded_by` so the caller can judge for itself.
    #[serde(default)]
    pub include_superseded: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// `remember` — create a memory. The L2 pointer pair is validated here (typed
/// rejection) as well as by `015`'s CHECK: a half-filled pointer is a memory
/// that cannot be followed.
#[derive(Debug, Deserialize)]
pub struct RememberArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    pub topic_id: String,
    pub title: String,
    pub l0_summary: String,
    #[serde(default)]
    pub l1_overview: Option<String>,
    /// 'document' | 'file'.
    pub l2_ref_kind: String,
    #[serde(default)]
    pub l2_document_id: Option<String>,
    #[serde(default)]
    pub l2_file_path: Option<String>,
    #[serde(default)]
    pub l2_body_digest: Option<String>,
    /// 'archived_session' | 'canonical_event' | 'document' | 'manual'.
    pub source_kind: String,
    pub source_ref: String,
    /// REQUIRED (018, contract §G1): 'decision' | 'failure' | 'fact' |
    /// 'preference' | 'artifact' | 'unresolved'. The additive DB default is a
    /// backfill only — the tool requires it explicitly.
    pub kind: String,
    /// REQUIRED (018, §G4): 'quoted' | 'inferred'. Binary, never a score.
    pub confidence: String,
    /// The memory this one supersedes (018, §G2). Create-time only. The edge is
    /// written in BOTH spellings (the column and a `memory_relations` row) in the
    /// SAME transaction.
    #[serde(default)]
    pub supersedes: Option<String>,
    /// First line of the L2 evidence inside `l2_file_path` (1-based). REQUIRED
    /// when l2_ref_kind='file' AND source_kind='archived_session' (018, §G5).
    #[serde(default)]
    pub l2_line_start: Option<i32>,
    /// Last line of the L2 evidence (>= l2_line_start).
    #[serde(default)]
    pub l2_line_end: Option<i32>,
    /// Extra typed edges to other memories (018, §G2): `{to_id, rel}` with rel in
    /// 'supersedes' | 'caused_by' | 'fixed_by' | 'refines'. A non-existent or
    /// self target, or a duplicate edge, is a typed rejection.
    #[serde(default)]
    pub relations: Vec<MemoryRelationArg>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// One outgoing edge in a `remember` call.
#[derive(Debug, Deserialize)]
pub struct MemoryRelationArg {
    pub to_id: String,
    pub rel: String,
}

/// `resolve` — the G3 closure path (018, §G3).
#[derive(Debug, Deserialize)]
pub struct ResolveArgs {
    pub memory_id: String,
}

/// `promote` — the ONLY tier channel, explicit target tier, up OR down.
#[derive(Debug, Deserialize)]
pub struct PromoteArgs {
    pub memory_id: String,
    /// 'cold' | 'warm' | 'hot'.
    pub to_tier: String,
}

/// `forget` — the soft tombstone. The reason is mandatory: a tombstone with no
/// recorded why is an unexplained edit to the ledger.
#[derive(Debug, Deserialize)]
pub struct ForgetArgs {
    pub memory_id: String,
    pub reason: String,
}

// =============================================================================
// Rejections
// =============================================================================

/// Why a memory-tool call was refused. Rendered verbatim into the MCP tool
/// error, like `dispatch::DispatchRejection` — every refusal is typed, and none
/// of them is a raw database error.
#[derive(Debug)]
pub enum MemoryRejection {
    /// A string that is not in the closed domain it was parsed against.
    BadDomain {
        domain: &'static str,
        value: String,
        expected: &'static str,
    },
    /// A required field is blank.
    Empty { field: &'static str },
    /// The L2 pointer pair does not match its kind.
    L2Pairing { detail: String },
    /// The memory id names no row.
    NotFound { memory_id: String },
    /// The row is not `active` (a forgotten row is an immutable tombstone).
    NotActive { memory_id: String },
    /// `promote` named the tier the memory already has.
    NoTierChange { memory_id: String, tier: String },
    /// The recall page size exceeds the hard ceiling.
    LimitTooLarge { limit: i64 },
    /// A `relations` target names no memory.
    RelationTargetMissing { to_id: String },
    /// A `relations` edge points at the memory being created (a self-edge).
    /// Not reachable through `remember` today (the new id is server-generated
    /// and every target must already exist), but the contract names the case and
    /// the day a tool takes a caller-supplied id it must refuse it.
    #[allow(dead_code)]
    RelationSelfEdge { to_id: String },
    /// The same edge is asserted twice in one call.
    RelationDuplicate { to_id: String, rel: String },
    /// The L2 evidence line range is required but absent (018 §G5: "the evidence
    /// is somewhere in this 134MB file" is NOT evidence).
    L2LinesRequired {
        l2_ref_kind: String,
        source_kind: String,
    },
    /// The L2 line range is inconsistent (end before start).
    L2LinesInverted { start: i32, end: i32 },
    /// The memory is not `active` (already resolved, or a forgotten tombstone).
    /// The repository's `MemoryNotActive` does not carry the status, so the
    /// resolve path reports the generic wording; this variant keeps the precise
    /// message available for the caller that has it.
    MemoryNotResolvable { memory_id: String, status: String },
    /// A database failure that is not one of the typed conditions above.
    Db(String),
}

impl std::fmt::Display for MemoryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryRejection::BadDomain {
                domain,
                value,
                expected,
            } => write!(
                f,
                "{domain} {value:?} is not one of {expected} (hand-written FromStr; an unknown value is never defaulted)"
            ),
            MemoryRejection::Empty { field } => write!(f, "{field} must not be empty"),
            MemoryRejection::L2Pairing { detail } => write!(f, "l2 pointer invalid: {detail}"),
            MemoryRejection::NotFound { memory_id } => {
                write!(f, "memory {memory_id:?} not found")
            }
            MemoryRejection::NotActive { memory_id } => write!(
                f,
                "memory {memory_id:?} is not active — a forgotten row is an immutable tombstone (ledger discipline)"
            ),
            MemoryRejection::NoTierChange { memory_id, tier } => write!(
                f,
                "memory {memory_id:?} is already in tier {tier:?} — promote needs a target tier that differs (no transition, nothing written)"
            ),
            MemoryRejection::LimitTooLarge { limit } => {
                write!(f, "limit {limit} exceeds the hard ceiling {RECALL_LIMIT_MAX}")
            }
            MemoryRejection::RelationTargetMissing { to_id } => write!(
                f,
                "relation target {to_id:?} names no memory — an edge to nothing is not an edge"
            ),
            MemoryRejection::RelationSelfEdge { to_id } => write!(
                f,
                "relation target {to_id:?} is the memory being created — a self-edge asserts nothing"
            ),
            MemoryRejection::RelationDuplicate { to_id, rel } => write!(
                f,
                "relation {rel} -> {to_id:?} is asserted twice — the edge set is duplicate-free"
            ),
            MemoryRejection::L2LinesRequired {
                l2_ref_kind,
                source_kind,
            } => write!(
                f,
                "l2_line_start/l2_line_end are REQUIRED when l2_ref_kind is '{l2_ref_kind}' and \
                 source_kind is '{source_kind}' — 'the evidence is somewhere in this file' is not \
                 evidence (018 §G5)"
            ),
            MemoryRejection::L2LinesInverted { start, end } => {
                write!(f, "l2_line_end ({end}) must be >= l2_line_start ({start})")
            }
            MemoryRejection::MemoryNotResolvable { memory_id, status } => write!(
                f,
                "memory {memory_id:?} is {status:?} — only an ACTIVE record can be resolved \
                 (a closed record and a tombstone are both immutable)"
            ),
            MemoryRejection::Db(e) => write!(f, "memory tool rejected: {e}"),
        }
    }
}

impl From<DispatchRejection> for MemoryRejection {
    fn from(e: DispatchRejection) -> Self {
        match e {
            DispatchRejection::MissingActorRef => MemoryRejection::BadDomain {
                domain: "SEATLOOM_ACTOR_REF",
                value: String::new(),
                expected: "the server must be launched per seat with its own actor ref (identity is never taken from a caller parameter)",
            },
            DispatchRejection::MissingProjectId => MemoryRejection::BadDomain {
                domain: "project_id",
                value: String::new(),
                expected: "pass project_id or set SEATLOOM_PROJECT_ID; a project is never fabricated",
            },
            other => MemoryRejection::Db(other.to_string()),
        }
    }
}

impl From<DbError> for MemoryRejection {
    fn from(e: DbError) -> Self {
        match e {
            DbError::MemoryNotFound { id } => MemoryRejection::NotFound { memory_id: id },
            DbError::MemoryNotActive { id } => MemoryRejection::NotActive { memory_id: id },
            DbError::MemoryNoTierChange { id, tier } => MemoryRejection::NoTierChange {
                memory_id: id,
                tier,
            },
            other => MemoryRejection::Db(other.to_string()),
        }
    }
}

/// A required non-blank string field, trimmed.
fn non_empty(value: &str, field: &'static str) -> Result<String, MemoryRejection> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(MemoryRejection::Empty { field });
    }
    Ok(trimmed.to_string())
}

/// A hand-parsed closed domain, with the parse error's own wording.
fn parse_domain<T>(value: &str, domain: &'static str) -> Result<T, MemoryRejection>
where
    T: std::str::FromStr<Err = seatloom_core::objects::session::ParseValueError>,
{
    T::from_str(value)
        .map_err(|e| MemoryRejection::BadDomain {
            domain: e.domain,
            value: e.value,
            expected: e.expected,
        })
        .map_err(|mut e| {
            // Keep the caller's domain label when the parse error carries the same
            // one (they always do — the domains name themselves).
            if let MemoryRejection::BadDomain { domain: d, .. } = &mut e {
                *d = domain;
            }
            e
        })
}

// =============================================================================
// recall
// =============================================================================

/// One memory as a recall returns it, at the requested depth. `l1_overview` and
/// the L2 pointer fields are `None` unless the caller asked for that layer — the
/// token budget belongs to the caller, so the default is the smallest answer.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalledMemory {
    pub id: String,
    pub title: String,
    pub tier: String,
    pub heat: i32,
    pub recall_count: i32,
    /// l0 is ALWAYS returned — it is the only default layer.
    pub l0_summary: String,
    /// Set only when the caller opted into superseded rows: the memory that
    /// replaced this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l2_ref_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l2_document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l2_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l2_body_digest: Option<String>,
}

/// The recall answer: the topic's ancestor chain (so a memory is never read out
/// of context) plus the memories at the requested depth.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecallResult {
    pub project_id: String,
    pub topic_id: String,
    pub depth: String,
    /// Whether superseded rows were included (and therefore annotated).
    pub include_superseded: bool,
    /// The recalled topic and its ancestors, nearest first.
    pub topic_chain: Vec<TopicChainEntry>,
    pub memories: Vec<RecalledMemory>,
    /// How many `MemoryRecalled` events this call appended (one per returned
    /// memory; zero hits = zero events).
    pub recalled: usize,
}

/// One level of the topic chain.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicChainEntry {
    pub id: String,
    pub title: String,
    pub status: String,
    pub parent_topic_id: Option<String>,
}

/// `recall(project, topic, query?, depth='l0', limit?)`.
///
/// Side effect: exactly ONE `MemoryRecalled` event per returned memory
/// (contract §9) — the ledger grows from "what happened" to "what is known".
/// Zero hits appends zero events. The audit event and the row update are one
/// transaction per memory.
pub async fn recall(db: &SeatloomDb, args: RecallArgs) -> Result<String, MemoryRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id)?;
    let depth = match &args.depth {
        Some(d) => parse_domain::<MemoryLayer>(d, "depth")?,
        None => MemoryLayer::L0,
    };
    let limit = args.limit.unwrap_or(RECALL_LIMIT_DEFAULT);
    if !(1..=RECALL_LIMIT_MAX).contains(&limit) {
        return Err(MemoryRejection::LimitTooLarge { limit });
    }

    let include_superseded = args.include_superseded.unwrap_or(false);
    let rows = db
        .list_memories_for_recall(
            &project_id,
            &args.topic_id,
            args.query.as_deref(),
            false,
            include_superseded,
            limit,
        )
        .await?;
    let chain = db.topic_ancestor_chain(&args.topic_id).await?;

    let mut recalled = 0usize;
    let mut memories = Vec::with_capacity(rows.len());
    for row in &rows {
        // The audit event is appended in the same transaction as the row update
        // — a recall that cannot be recorded is not a recall.
        db.record_recall(&row.id, &identity.actor_ref).await?;
        recalled += 1;
        memories.push(RecalledMemory {
            id: row.id.clone(),
            title: row.title.clone(),
            tier: row.tier.clone(),
            heat: row.heat + 1,
            recall_count: row.recall_count + 1,
            l0_summary: row.l0_summary.clone(),
            // Opting into superseded rows annotates WHICH memory replaced each
            // one, so the caller can judge for itself (018 §G2).
            superseded_by: if include_superseded {
                db.superseded_by(&row.id).await?
            } else {
                None
            },
            l1_overview: match depth {
                MemoryLayer::L0 => None,
                _ => row.l1_overview.clone(),
            },
            l2_ref_kind: match depth {
                MemoryLayer::L2 => Some(row.l2_ref_kind.clone()),
                _ => None,
            },
            l2_document_id: match depth {
                MemoryLayer::L2 => row.l2_document_id.clone(),
                _ => None,
            },
            l2_file_path: match depth {
                MemoryLayer::L2 => row.l2_file_path.clone(),
                _ => None,
            },
            l2_body_digest: match depth {
                MemoryLayer::L2 => row.l2_body_digest.clone(),
                _ => None,
            },
        });
    }

    let result = RecallResult {
        project_id,
        topic_id: args.topic_id,
        depth: depth.as_str().to_string(),
        include_superseded,
        topic_chain: chain
            .into_iter()
            .map(|t| TopicChainEntry {
                id: t.id,
                title: t.title,
                status: t.status,
                parent_topic_id: t.parent_topic_id,
            })
            .collect(),
        memories,
        recalled,
    };
    serde_json::to_string_pretty(&result).map_err(|e| MemoryRejection::Db(e.to_string()))
}

// =============================================================================
// remember
// =============================================================================

/// `remember(...)` — create a memory. Tier starts `cold` and status starts
/// `active`; the caller cannot specify either (the contract fixes them).
pub async fn remember(db: &SeatloomDb, args: RememberArgs) -> Result<String, MemoryRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id)?;

    let topic_id = non_empty(&args.topic_id, "topic_id")?;
    let title = non_empty(&args.title, "title")?;
    let l0_summary = non_empty(&args.l0_summary, "l0_summary")?;
    let source_ref = non_empty(&args.source_ref, "source_ref")?;
    let l2_kind = parse_domain::<L2RefKind>(&args.l2_ref_kind, "l2_ref_kind")?;
    let source_kind = parse_domain::<MemorySourceKind>(&args.source_kind, "source_kind")?;
    // 018: kind and confidence are REQUIRED — the additive DB defaults are a
    // backfill only, never a write path (contract §G1/§G4).
    let kind = parse_domain::<MemoryKind>(&args.kind, "kind")?;
    let confidence = parse_domain::<MemoryConfidence>(&args.confidence, "confidence")?;
    let supersedes = match &args.supersedes {
        Some(id) => Some(non_empty(id, "supersedes")?),
        None => None,
    };

    // The L2 pointer pair: exactly one pointer, and it must match the kind.
    // `015`'s CHECK enforces the same rule in the database — belt and braces.
    match l2_kind {
        L2RefKind::Document => {
            if args
                .l2_document_id
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                return Err(MemoryRejection::L2Pairing {
                    detail: "l2_ref_kind 'document' requires l2_document_id".to_string(),
                });
            }
            if args.l2_file_path.is_some() {
                return Err(MemoryRejection::L2Pairing {
                    detail: "l2_ref_kind 'document' must not carry l2_file_path".to_string(),
                });
            }
        }
        L2RefKind::File => {
            if args.l2_file_path.as_deref().unwrap_or("").trim().is_empty() {
                return Err(MemoryRejection::L2Pairing {
                    detail: "l2_ref_kind 'file' requires l2_file_path".to_string(),
                });
            }
            if args.l2_document_id.is_some() {
                return Err(MemoryRejection::L2Pairing {
                    detail: "l2_ref_kind 'file' must not carry l2_document_id".to_string(),
                });
            }
        }
    }

    // 018 §G5: the L2 pointer must carry a line range when the evidence is a
    // slice of an archived session transcript — "the evidence is somewhere in
    // this 134MB file" is NOT evidence. Mirrors the DB CHECK; refused here so
    // the caller gets the reason, not a constraint name.
    let (l2_line_start, l2_line_end) = match (args.l2_line_start, args.l2_line_end) {
        (Some(s), Some(e)) => {
            if e < s {
                return Err(MemoryRejection::L2LinesInverted { start: s, end: e });
            }
            (Some(s), Some(e))
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(MemoryRejection::L2LinesRequired {
                l2_ref_kind: l2_kind.as_str().to_string(),
                source_kind: source_kind.as_str().to_string(),
            });
        }
        (None, None) => {
            if l2_kind == L2RefKind::File && source_kind == MemorySourceKind::ArchivedSession {
                return Err(MemoryRejection::L2LinesRequired {
                    l2_ref_kind: l2_kind.as_str().to_string(),
                    source_kind: source_kind.as_str().to_string(),
                });
            }
            (None, None)
        }
    };

    // 018 §G2: the relation edges. Every target must exist, and no edge may be
    // asserted twice in one call — all checked before any DB contact, so the
    // caller gets a reason rather than a constraint name.
    //
    // A SELF-edge cannot occur here: `from_id` is the new memory's id, which this
    // tool generates and the caller cannot name, and every target must already
    // exist. The refusal exists for the day a tool takes a caller-supplied id.
    let mut relations: Vec<(String, String)> = Vec::with_capacity(args.relations.len());
    for r in &args.relations {
        let to_id = non_empty(&r.to_id, "relations[].to_id")?;
        parse_domain::<MemoryRelation>(&r.rel, "relations[].rel")?;
        if relations
            .iter()
            .any(|(t, rel)| t == &to_id && rel == &r.rel)
        {
            return Err(MemoryRejection::RelationDuplicate {
                to_id,
                rel: r.rel.clone(),
            });
        }
        relations.push((to_id, r.rel.clone()));
    }
    for (to_id, _) in &relations {
        if db.get_memory(to_id).await?.is_none() {
            return Err(MemoryRejection::RelationTargetMissing {
                to_id: to_id.clone(),
            });
        }
    }
    if let Some(prev) = &supersedes {
        if db.get_memory(prev).await?.is_none() {
            return Err(MemoryRejection::RelationTargetMissing {
                to_id: prev.clone(),
            });
        }
    }

    let memory_id = format!("mem-{}", nanoid::nanoid!());
    db.insert_memory(
        &memory_id,
        &project_id,
        &topic_id,
        &title,
        &args.tags,
        &l0_summary,
        args.l1_overview.as_deref(),
        l2_kind.as_str(),
        args.l2_document_id.as_deref(),
        args.l2_file_path.as_deref(),
        args.l2_body_digest.as_deref(),
        source_kind.as_str(),
        &source_ref,
        kind.as_str(),
        confidence.as_str(),
        supersedes.as_deref(),
        l2_line_start,
        l2_line_end,
        &relations,
        &identity.actor_ref,
    )
    .await?;

    Ok(format!(
        "remembered memory {memory_id} in project {project_id} / topic {topic_id} \
         (kind {}, confidence {}, tier cold, status active, source {} {}{}{})",
        kind.as_str(),
        confidence.as_str(),
        source_kind.as_str(),
        source_ref,
        supersedes
            .as_ref()
            .map(|s| format!(", supersedes {s}"))
            .unwrap_or_default(),
        if relations.is_empty() {
            String::new()
        } else {
            format!(", {} relation(s)", relations.len())
        }
    ))
}

// =============================================================================
// promote
// =============================================================================

/// `promote(memory_id, to_tier)` — the ONLY tier channel, up OR down.
pub async fn promote(db: &SeatloomDb, args: PromoteArgs) -> Result<String, MemoryRejection> {
    let identity = resolve_seat()?;
    let to_tier = parse_domain::<MemoryTier>(&args.to_tier, "to_tier")?;
    let row = db
        .set_memory_tier(&args.memory_id, to_tier.as_str(), &identity.actor_ref)
        .await?;
    Ok(format!(
        "promoted memory {} to tier {} (one MemoryPromoted event appended)",
        row.id, row.tier
    ))
}

// =============================================================================
// resolve
// =============================================================================

/// `resolve(memory_id)` — the G3 closure path (018, §G3): `active -> resolved`
/// + `MemoryResolved` in ONE transaction.
///
/// The ONLY closure path: there is no background sweep and no automatic
// anything.
///
/// `resolved_by` comes from the server-side identity, never a parameter.
pub async fn resolve(db: &SeatloomDb, args: ResolveArgs) -> Result<String, MemoryRejection> {
    let identity = resolve_seat()?;
    // Read the row first so a refusal can name the ACTUAL status: a closed
    // record and a tombstone are different facts, and the caller deserves to
    // know which one it hit.
    if let Some(existing) = db.get_memory(&args.memory_id).await? {
        if existing.status != "active" {
            return Err(MemoryRejection::MemoryNotResolvable {
                memory_id: args.memory_id.clone(),
                status: existing.status.clone(),
            });
        }
    }
    let row = db
        .resolve_memory(&args.memory_id, &identity.actor_ref)
        .await?;
    Ok(format!(
        "resolved memory {} (kind {}, closed by {} at {} — one MemoryResolved event appended)",
        row.id,
        row.kind,
        row.resolved_by.as_deref().unwrap_or("?"),
        row.resolved_at.map(|t| t.to_rfc3339()).unwrap_or_default()
    ))
}

// =============================================================================
// forget
// =============================================================================

/// `forget(memory_id, reason)` — the soft tombstone. The row stays, forever.
pub async fn forget(db: &SeatloomDb, args: ForgetArgs) -> Result<String, MemoryRejection> {
    let identity = resolve_seat()?;
    let reason = non_empty(&args.reason, "reason")?;
    let row = db
        .forget_memory(&args.memory_id, &reason, &identity.actor_ref)
        .await?;
    Ok(format!(
        "forgot memory {} (soft tombstone, reason recorded; the row is never hard-deleted)",
        row.id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// The page-size ceiling is a hard refusal, not a silent clamp — and the
    /// default is the contract's 20.
    #[test]
    fn recall_ceiling_and_default() {
        assert_eq!(RECALL_LIMIT_DEFAULT, 20);
        assert_eq!(RECALL_LIMIT_MAX, 100);
        let err = MemoryRejection::LimitTooLarge { limit: 101 }.to_string();
        assert!(err.contains("100"), "{err}");
        assert!(MemoryRejection::Empty { field: "reason" }
            .to_string()
            .contains("reason"));
    }

    /// Progressive disclosure is visible in the serialized shape: an l0 recall
    /// omits the l1/l2 fields entirely rather than sending them empty.
    #[test]
    fn l0_recall_omits_the_higher_layers() {
        let l0 = RecalledMemory {
            id: "mem-1".into(),
            title: "t".into(),
            tier: "cold".into(),
            heat: 1,
            recall_count: 1,
            l0_summary: "s".into(),
            superseded_by: None,
            l1_overview: None,
            l2_ref_kind: None,
            l2_document_id: None,
            l2_file_path: None,
            l2_body_digest: None,
        };
        let v: Value = serde_json::to_value(&l0).unwrap();
        assert!(v.get("l0Summary").is_some(), "l0 is always returned");
        assert!(v.get("l1Overview").is_none(), "l1 is omitted at l0");
        assert!(v.get("l2RefKind").is_none(), "l2 is omitted at l0");

        let l2 = RecalledMemory {
            superseded_by: None,
            l2_ref_kind: Some("file".into()),
            l2_file_path: Some("docs/x.md".into()),
            ..l0
        };
        let v: Value = serde_json::to_value(&l2).unwrap();
        assert_eq!(v["l2RefKind"], "file");
        assert_eq!(v["l2FilePath"], "docs/x.md");
    }

    /// The typed refusals name the rule, so a caller can fix the call without
    /// guessing — and none of them is a raw database error.
    #[test]
    fn refusals_name_the_rule_broken() {
        assert!(MemoryRejection::NotFound {
            memory_id: "mem-1".into()
        }
        .to_string()
        .contains("not found"));
        assert!(MemoryRejection::NotActive {
            memory_id: "mem-1".into()
        }
        .to_string()
        .contains("tombstone"));
        assert!(MemoryRejection::NoTierChange {
            memory_id: "mem-1".into(),
            tier: "warm".into()
        }
        .to_string()
        .contains("already in tier"));
        assert!(MemoryRejection::L2Pairing {
            detail: "l2_ref_kind 'file' requires l2_file_path".into()
        }
        .to_string()
        .contains("l2_file_path"));
        assert!(MemoryRejection::BadDomain {
            domain: "depth",
            value: "l3".into(),
            expected: "l0 | l1 | l2",
        }
        .to_string()
        .contains("l0 | l1 | l2"));
    }
}
