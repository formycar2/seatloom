//! Lineage read model — "what is this object, and what is its context chain?"
//!
//! The operator's blocking feedback on the pending-acceptance card was that it
//! shows only a branch and a one-line `how_to_verify`: no cause-and-effect, no
//! lineage. Aegis proved with a manual `psql` dig that **everything is already in
//! the ledger** — the upstream dispatch's original task statement, the event
//! chain, the Flux verdict's evidence, the dependency edges. Nothing needed to be
//! stored; it needed to be **assembled**.
//!
//! So this module is a PURE READ assembler: zero writes, zero new tables, zero
//! new event types (value-domain contract §6 red lines). It is the FIRST INSTANCE
//! of a shape, not a one-off:
//!
//! ```text
//!   object  +  its context chain  +  progressive disclosure
//! ```
//!
//! The acceptance lineage below assembles an `acceptances` row with the dispatch
//! it gates, that dispatch's event chain, the Flux verdict named by the card, the
//! products those events produced, the dispatch's dependency edges, and the
//! rework dispatch when the card was rejected. D5's recall read model (memory +
//! topic-ancestor chain) is the SAME shape — an object plus its context chain,
//! progressively disclosed — and plugs into this module as a second assembler
//! once migration 015 lands. One design, two assemblers; no second ad-hoc query
//! per feature.
//!
//! Reads never invent: a referent that cannot be joined is carried with the ref
//! value it has (`resolved: false`), never guessed at and never backfilled.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::db::repositories::SeatloomDb;

// =============================================================================
// The assembled lineage (value-domain contract §3 — the seven blocks)
// =============================================================================

/// The full lineage of one acceptance card, assembled from the ledger.
///
/// Field order is the operator's own reading order (§4 of the contract): which
/// milestone/lane/task -> who issued it and when -> the original task
/// requirements -> who did the work -> where the products are -> the Flux
/// verdict and its evidence -> the structured acceptance plan. The UI discloses
/// it in levels (L0 enough to judge, L1 expanded, L2 ids/links) from this ONE
/// object; there is no second query per level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineage {
    /// The acceptance card itself.
    pub acceptance: AcceptanceLineageCard,
    /// The dispatch the card gates — the ORIGINAL task statement.
    pub dispatch: AcceptanceLineageDispatch,
    /// The dispatch's canonical events in `occurred_at` order.
    pub events: Vec<AcceptanceLineageEvent>,
    /// The Flux green verdict named by `flux_verdict_ref`.
    pub flux_verdict: Option<AcceptanceLineageVerdict>,
    /// Artifact / document products found in the event chain.
    pub products: Vec<AcceptanceLineageProduct>,
    /// The dispatch's dependency edges, with the referent's existence.
    pub dependencies: Vec<AcceptanceLineageDependency>,
    /// The rework dispatch, when the card was rejected.
    pub rework: Option<AcceptanceLineageDispatch>,
}

/// The card block: the acceptance row, verbatim, plus the doing seat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageCard {
    pub id: String,
    pub dispatch_id: String,
    pub project_id: String,
    pub branch: String,
    pub base: String,
    pub how_to_verify: String,
    /// The structured acceptance plan (016, shape v1). `None`/`{}` means the
    /// legacy row predates the plan and Aegis has not backfilled it yet.
    pub acceptance_plan: Option<serde_json::Value>,
    pub flux_verdict_ref: String,
    /// 'pending' | 'accepted' | 'rejected' | 'cancelled'.
    pub status: String,
    pub feedback: Option<String>,
    pub rework_dispatch_id: Option<String>,
    pub merge_commit: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// The dispatch block — the work unit's own row, including the ORIGINAL task
/// statement (`payload`) that the card alone never showed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageDispatch {
    pub id: String,
    pub title: String,
    pub issued_by: String,
    pub target_seat_id: String,
    pub priority: i32,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// The original task statement (packet path / instructions), verbatim.
    pub payload: serde_json::Value,
}

/// One event in the chain, as recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageEvent {
    pub id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub actor_ref: String,
    pub payload: Option<serde_json::Value>,
}

/// The Flux verdict block: the `ReviewVerdictIssued` event named by the card,
/// rendered from its payload. Absent fields stay `None` — never defaulted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageVerdict {
    pub ref_id: String,
    pub verdict: Option<String>,
    pub evidence: Option<String>,
    pub evidence_detail: Option<String>,
    pub target_branch: Option<String>,
    pub lane: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
}

/// A product found in the event chain: an artifact or a document. `title` /
/// `path` are `None` when the referent cannot be joined (a deleted or
/// never-materialised row) — the ref value is still carried, and `resolved`
/// says plainly which of the two this is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageProduct {
    pub ref_type: String, // 'artifact' | 'document'
    pub ref_id: String,
    pub title: Option<String>,
    pub path: Option<String>,
    pub resolved: bool,
}

/// One dependency edge of the gated dispatch, with the referent's existence
/// (012: readiness is an existence query; there is no FK on `dep_value`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageDependency {
    pub dep_kind: String, // 'event' | 'artifact' | 'dispatch'
    pub dep_value: String,
    pub exists: bool,
}

// =============================================================================
// The assembler
// =============================================================================

/// Assemble one acceptance's lineage. Pure read: no writes, no events.
///
/// An unknown acceptance id is a typed error, NOT an empty object — a caller
/// that mistyped an id must hear about it rather than render an empty card.
pub async fn get_acceptance_lineage(
    db: &SeatloomDb,
    acceptance_id: &str,
) -> Result<AcceptanceLineage, LineageError> {
    let card = db
        .get_acceptance(acceptance_id)
        .await
        .map_err(LineageError::Db)?
        .ok_or_else(|| LineageError::NotFound {
            acceptance_id: acceptance_id.to_string(),
        })?;

    let dispatch = db
        .get_dispatch(&card.acceptance.dispatch_id)
        .await
        .map_err(LineageError::Db)?
        .ok_or_else(|| LineageError::DispatchMissing {
            acceptance_id: acceptance_id.to_string(),
            dispatch_id: card.acceptance.dispatch_id.clone(),
        })?;

    // The event chain, in two hops (bounded — this is a context chain, not a
    // graph walk):
    //   hop 1 — every event that names the dispatch OR the acceptance as any
    //           kind of ref (subject/actor/context), plus the Flux verdict
    //           event the card points at (it names no object of its own);
    //   hop 2 — every event that names a PRODUCT found in hop 1, because a
    //           delivery's `ArtifactCreated` names the artifact, not the
    //           dispatch — without this hop the products would be listed but
    //           the event that produced them would be missing from the chain.
    let mut events = db
        .list_events_for_refs(
            &[
                ("dispatch", card.acceptance.dispatch_id.as_str()),
                ("acceptance", card.acceptance.id.as_str()),
            ],
            Some(card.acceptance.flux_verdict_ref.as_str()),
        )
        .await
        .map_err(LineageError::Db)?;
    let hop1_refs = db
        .list_object_refs_for_events(&events.iter().map(|e| e.id.clone()).collect::<Vec<_>>())
        .await
        .map_err(LineageError::Db)?;
    let product_pairs: Vec<(String, String)> = hop1_refs
        .iter()
        .filter(|r| r.ref_type == "artifact" || r.ref_type == "document")
        .map(|r| (r.ref_type.clone(), r.ref_id.clone()))
        .collect();
    if !product_pairs.is_empty() {
        let borrowed: Vec<(&str, &str)> = product_pairs
            .iter()
            .map(|(t, i)| (t.as_str(), i.as_str()))
            .collect();
        let hop2 = db
            .list_events_for_refs(&borrowed, None)
            .await
            .map_err(LineageError::Db)?;
        // Merge, dedupe by event id, restore `occurred_at` order.
        let mut seen: std::collections::HashSet<String> =
            events.iter().map(|e| e.id.clone()).collect();
        events.extend(hop2.into_iter().filter(|e| seen.insert(e.id.clone())));
        events.sort_by(|a, b| {
            a.occurred_at
                .cmp(&b.occurred_at)
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    // The Flux verdict: the event named by the card, rendered from its payload.
    let flux_verdict = match db
        .get_event(&card.acceptance.flux_verdict_ref)
        .await
        .map_err(LineageError::Db)?
    {
        Some(e) if e.event_type == "ReviewVerdictIssued" => {
            let p = e.payload.clone().unwrap_or(serde_json::Value::Null);
            Some(AcceptanceLineageVerdict {
                ref_id: e.id,
                verdict: str_field(&p, "verdict"),
                evidence: str_field(&p, "evidence"),
                evidence_detail: str_field(&p, "evidence_detail"),
                target_branch: str_field(&p, "target_branch"),
                lane: str_field(&p, "lane"),
                occurred_at: Some(e.occurred_at),
            })
        }
        // The referent exists but is not a verdict event (or is an artifact, so
        // it is not in canonical_events at all): the card records what it
        // recorded; the verdict block stays absent rather than invented.
        _ => None,
    };

    // Products: artifact/document refs found in the chain. Joined where
    // possible, carried verbatim where not.
    let mut products = Vec::new();
    for r in &hop1_refs {
        match r.ref_type.as_str() {
            "artifact" | "document" => {
                let (title, path) = db
                    .product_title_and_path(&r.ref_type, &r.ref_id)
                    .await
                    .map_err(LineageError::Db)?
                    .unwrap_or((None, None));
                products.push(AcceptanceLineageProduct {
                    ref_type: r.ref_type.clone(),
                    ref_id: r.ref_id.clone(),
                    resolved: title.is_some(),
                    title,
                    path,
                });
            }
            _ => {}
        }
    }

    // Dependencies: the dispatch's edges, each with its referent's existence.
    let mut dependencies = Vec::new();
    for d in db
        .list_dispatch_dependencies(&dispatch.id)
        .await
        .map_err(LineageError::Db)?
    {
        let exists = db
            .referent_exists(&d.dep_kind, &d.dep_value)
            .await
            .map_err(LineageError::Db)?;
        dependencies.push(AcceptanceLineageDependency {
            dep_kind: d.dep_kind,
            dep_value: d.dep_value,
            exists,
        });
    }

    // Rework: the auto-enqueued dispatch, when the card was rejected.
    let rework = match &card.acceptance.rework_dispatch_id {
        Some(id) => db.get_dispatch(id).await.map_err(LineageError::Db)?,
        None => None,
    }
    .map(|d| dispatch_block(&d));

    Ok(AcceptanceLineage {
        acceptance: AcceptanceLineageCard {
            id: card.acceptance.id,
            dispatch_id: card.acceptance.dispatch_id,
            project_id: card.acceptance.project_id,
            branch: card.acceptance.branch,
            base: card.acceptance.base,
            how_to_verify: card.acceptance.how_to_verify,
            acceptance_plan: plan_or_none(Some(&card.acceptance.acceptance_plan)),
            flux_verdict_ref: card.acceptance.flux_verdict_ref,
            status: card.acceptance.status,
            feedback: card.acceptance.feedback,
            rework_dispatch_id: card.acceptance.rework_dispatch_id,
            merge_commit: card.acceptance.merge_commit,
            decided_by: card.acceptance.decided_by,
            decided_at: card.acceptance.decided_at,
            created_at: card.acceptance.created_at,
        },
        dispatch: dispatch_block(&dispatch),
        events: events
            .into_iter()
            .map(|e| AcceptanceLineageEvent {
                id: e.id,
                event_type: e.event_type,
                occurred_at: e.occurred_at,
                actor_ref: e.actor_ref,
                payload: e.payload,
            })
            .collect(),
        flux_verdict,
        products,
        dependencies,
        rework,
    })
}

/// `{}` is explicitly NOT a plan (contract §1): a legacy row that has not been
/// backfilled yet reads as `None`, never as an empty plan.
fn plan_or_none(plan: Option<&serde_json::Value>) -> Option<serde_json::Value> {
    match plan {
        Some(p) if p.is_object() && p.as_object().map(|o| !o.is_empty()).unwrap_or(false) => {
            Some(p.clone())
        }
        _ => None,
    }
}

fn dispatch_block(d: &crate::db::models::DispatchRow) -> AcceptanceLineageDispatch {
    AcceptanceLineageDispatch {
        id: d.id.clone(),
        title: d.title.clone(),
        issued_by: d.issued_by.clone(),
        target_seat_id: d.target_seat_id.clone(),
        priority: d.priority,
        status: d.status.clone(),
        issued_at: d.issued_at,
        started_at: d.started_at,
        completed_at: d.completed_at,
        payload: d.payload.clone(),
    }
}

/// A string field of an event payload, or `None` when absent/not a string.
/// Never defaulted: an absent evidence field reads as absent.
fn str_field(payload: &serde_json::Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// What can go wrong assembling a lineage. Read-only failures are typed, never
/// swallowed into an empty object.
#[derive(Debug, thiserror::Error)]
pub enum LineageError {
    #[error("acceptance {acceptance_id:?} not found")]
    NotFound { acceptance_id: String },
    #[error(
        "acceptance {acceptance_id:?} gates dispatch {dispatch_id:?}, which does not exist — the ledger is inconsistent; reported, not worked around"
    )]
    DispatchMissing {
        acceptance_id: String,
        dispatch_id: String,
    },
    #[error("{0}")]
    Db(crate::db::repositories::DbError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `{}` is not a plan, and neither is a non-object — the legacy-row case
    /// reads as `None` rather than as an empty plan.
    #[test]
    fn empty_plan_object_reads_as_none() {
        let empty = serde_json::json!({});
        assert!(plan_or_none(Some(&empty)).is_none());
        assert!(plan_or_none(None).is_none());
        let not_object = serde_json::json!(["not", "a", "plan"]);
        assert!(plan_or_none(Some(&not_object)).is_none());
        let real = serde_json::json!({"plan_version": 1, "scenario": "s"});
        assert!(plan_or_none(Some(&real)).is_some());
    }

    /// Verdict fields are read from the payload as-is; absent fields stay
    /// `None` and an empty string is treated as absent (never a blank badge).
    #[test]
    fn verdict_fields_are_read_verbatim_and_never_defaulted() {
        let p = serde_json::json!({
            "verdict": "pass",
            "evidence": "gate output",
            "evidence_detail": "",
            "target_branch": "track/infra-foundation",
            "lane": "nimbus"
        });
        assert_eq!(str_field(&p, "verdict").as_deref(), Some("pass"));
        assert_eq!(str_field(&p, "evidence").as_deref(), Some("gate output"));
        assert_eq!(str_field(&p, "evidence_detail"), None, "empty = absent");
        assert_eq!(str_field(&p, "lane").as_deref(), Some("nimbus"));
        assert_eq!(str_field(&p, "missing"), None);
        assert_eq!(str_field(&serde_json::json!({"n": 1}), "n"), None);
    }

    /// The seven contract blocks are all present on the assembled shape, and the
    /// wire shape is stable JSON (the DTO the UI consumes is built from this).
    #[test]
    fn the_lineage_shape_carries_all_seven_contract_blocks() {
        let v: serde_json::Value = serde_json::to_value(AcceptanceLineage {
            acceptance: AcceptanceLineageCard {
                id: "acc-1".into(),
                dispatch_id: "disp-1".into(),
                project_id: "p".into(),
                branch: "packet/nimbus/x".into(),
                base: "track/infra-foundation".into(),
                how_to_verify: "do A see B".into(),
                acceptance_plan: None,
                flux_verdict_ref: "ev-1".into(),
                status: "pending".into(),
                feedback: None,
                rework_dispatch_id: None,
                merge_commit: None,
                decided_by: None,
                decided_at: None,
                created_at: Utc::now(),
            },
            dispatch: AcceptanceLineageDispatch {
                id: "disp-1".into(),
                title: "t".into(),
                issued_by: "seat:seat-lyra-001".into(),
                target_seat_id: "seat-nimbus-001".into(),
                priority: 5,
                status: "done".into(),
                issued_at: Utc::now(),
                started_at: None,
                completed_at: None,
                payload: serde_json::json!({"packet": "p/x.md"}),
            },
            events: vec![],
            flux_verdict: None,
            products: vec![],
            dependencies: vec![],
            rework: None,
        })
        .unwrap();
        for block in [
            "acceptance",
            "dispatch",
            "events",
            "fluxVerdict",
            "products",
            "dependencies",
            "rework",
        ] {
            assert!(v.get(block).is_some(), "missing block {block}");
        }
        assert_eq!(v["acceptance"]["acceptancePlan"], serde_json::Value::Null);
    }
}
