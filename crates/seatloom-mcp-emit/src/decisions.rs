//! The operator-decision tools (020) — the structured object for "what only the
//! operator can decide".
//!
//! WHY THIS EXISTS. Until 020, a product-direction ruling or an operator-only
//! precondition existed ONLY as a `SupervisorMessage`. The ledger had the record;
//! nothing structured could answer "how many things are waiting on the human
//! right now, and about what". `fleet-status.sh` read `acceptances` alone. The
//! value design (V3) names the failure mode exactly: 静默遗漏 — silent omission.
//! This object makes the obligation countable.
//!
//! THIS IS NOT A MESSAGE TABLE. The judgement rule (Aegis, from 张先生's 2026-10-20
//! authorisation): **a message does not constitute a decision obligation.** A
//! blocked dispatch, a failed read, a rework request — those belong to a seat or
//! to Aegis, and pushing them here would be exactly the "everything defaults to
//! him" failure the packet warns about. Only what meets the four criteria
//! (money/shipping/the outside world; changing the instructions given to others;
//! changing what the product shows other people by default) belongs here, and
//! Aegis owns the judgement of WHICH item that is. This module only supplies the
//! object and enforces the entry rules.
//!
//! # Who may create (the packet's third rule)
//!
//! Only the operator (`human:*`) and Aegis (`seat:seat-aegis-001`). A seat must
//! not be able to manufacture a "waiting on the operator" to push its own work —
//! that is a refusal, not a discouraged pattern.
//!
//! # Who may resolve
//!
//! The DECIDER. A row Aegis created for itself (`decider = 'seat:seat-aegis-001'`)
//! is resolved by Aegis; a row for the operator is resolved by the operator. The
//! operator's resolution path is the desktop app acting as the user's agent
//! (`cmd_resolve_operator_decision`), exactly the 014 split where
//! `request_acceptance` is the seat's MCP call and `cmd_accept_delivery` is the
//! operator's click. There is no operator MCP server, so a `human:*` actor never
//! reaches this module's resolve path.

use serde::Deserialize;

use seatloom_core::db::models::OperatorDecisionRow;
use seatloom_core::db::repositories::{DbError, NewOperatorDecision};
use seatloom_core::db::SeatloomDb;

use crate::dispatch::{resolve_project, resolve_seat, DispatchRejection};

/// The one seat allowed to create an obligation that is not its own to decide.
/// Everything else that is a seat is refused — see the module docs.
const AEGIS_ACTOR_REF: &str = "seat:seat-aegis-001";

/// May THIS actor create an operator obligation?
///
/// The operator (`human:*`) and Aegis. Nothing else — not another seat, not a
/// bare id, not an empty string. A pure function so the rule can be tested
/// without a live server: the packet's third rule is the one most likely to be
/// quietly relaxed later ("seats can propose decisions too" sounds helpful and
/// is exactly the work-pushing failure).
///
/// The `human:` prefix requires a NAME after it. `"human:"` alone is not an
/// identity — it is a prefix with nobody behind it — and a creator check that
/// accepts it would let a malformed ref put an obligation on the operator's
/// plate. The test for this exists because the naive `starts_with` version
/// passed it.
fn may_create_obligation(actor_ref: &str) -> bool {
    if actor_ref == AEGIS_ACTOR_REF {
        return true;
    }
    match actor_ref.strip_prefix("human:") {
        Some(name) => !name.trim().is_empty(),
        None => false,
    }
}

/// `request_operator_decision` — put a decision the operator must rule on into
/// the one structured place that answers "what is waiting on the human?".
#[derive(Debug, Deserialize)]
pub struct RequestOperatorDecisionArgs {
    /// The obligation id, `dec-<slug>`. Stable: it is referenced forever.
    pub decision_id: String,
    /// The CONCRETE thing to decide, not a paragraph.
    pub target: String,
    /// The legal options. A single option is not a choice; an empty array is
    /// not a decision.
    pub choices: Vec<String>,
    /// Who is entitled to rule. Omit for the operator — the default, per the
    /// packet's "默认是操作者,但不默认成所有事都推给他".
    #[serde(default)]
    pub decider: Option<String>,
    /// What happens once it is decided. REQUIRED: a choice offered without its
    /// consequence is the operator clicking in the dark.
    pub consequence: String,
    /// The support chain: dispatch ids, verdict refs, ruling refs. Optional —
    /// not every obligation has evidence, but one that claims some must name it.
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

/// `resolve_operator_decision` — land the ruling.
#[derive(Debug, Deserialize)]
pub struct ResolveOperatorDecisionArgs {
    pub decision_id: String,
    /// Which of the row's OWN `choices` was taken.
    pub chosen: String,
}

/// `list_operator_decisions` — the pure read that answers "how many things are
/// waiting on the human right now, and about what?"
///
/// PURE READ: no write, no event (a count question, not a transition).
#[derive(Debug, Deserialize)]
pub struct ListOperatorDecisionsArgs {
    /// `pending` (default) or `resolved`. The pending count is the question the
    /// object exists for; `resolved` is the audit of what has been ruled.
    #[serde(default)]
    pub status: Option<String>,
    /// Whose obligations. Defaults to the caller's OWN actor ref, so a seat
    /// asking is answered about itself and cannot enumerate the operator's
    /// plate. The operator's own server resolves to `human:*`.
    #[serde(default)]
    pub decider: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

/// Why an operator-decision call was refused. Every refusal is typed, writes
/// nothing, and names the rule broken — a caller can fix the call without
/// guessing, and no refusal is a raw database error.
#[derive(Debug)]
pub enum DecisionRejection {
    /// The caller is a seat other than Aegis. NOT "allowed but discouraged":
    /// a seat that could create an obligation could push its own work onto the
    /// operator's plate, which is the failure the packet rules out.
    NotOperatorOrAegis {
        actor_ref: String,
    },
    /// A required text field is blank.
    Empty {
        field: &'static str,
    },
    /// `choices` is empty. One option is not a choice, and zero is not a
    /// decision.
    NoChoices,
    /// `choices` names the same option twice. `chosen` would then be ambiguous
    /// in the event payload — two identical strings, one decision.
    DuplicateChoice {
        choice: String,
    },
    /// An evidence entry is blank. A blank reference is not evidence.
    BlankEvidence,
    /// `status` is neither `pending` nor `resolved`.
    BadStatus {
        status: String,
    },
    /// `decider` is neither an operator ref nor a seat ref. A fabricated decider
    /// is an obligation nobody owes.
    BadDeciderShape {
        decider: String,
    },
    /// An obligation with that id already exists. A second one would either
    /// duplicate the question or silently rewrite the first.
    DecisionIdTaken {
        decision_id: String,
    },
    /// The decision names no row.
    NotFound {
        decision_id: String,
    },
    /// The caller is not the decider. A ruling from anyone else is not a ruling.
    NotTheDecider {
        decision_id: String,
        decider: String,
    },
    /// The decision is already resolved — terminal and immutable.
    AlreadyResolved {
        decision_id: String,
    },
    /// `chosen` is not one of the row's own choices.
    ChoiceNotOffered {
        decision_id: String,
        chosen: String,
        choices: Vec<String>,
    },
    /// The project names no project.
    ProjectMissing {
        project_id: String,
    },
    Db(String),
}

impl std::fmt::Display for DecisionRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionRejection::NotOperatorOrAegis { actor_ref } => write!(
                f,
                "operator decision rejected: {actor_ref:?} may not create an obligation — only the \
                 operator (human:*) and {AEGIS_ACTOR_REF} can. A seat creating a \
                 'waiting on the operator' could push its own work onto his plate, which is a \
                 refusal, not a discouraged pattern"
            ),
            DecisionRejection::Empty { field } => {
                write!(f, "operator decision rejected: {field} must not be empty")
            }
            DecisionRejection::NoChoices => write!(
                f,
                "operator decision rejected: choices is empty — one option is not a choice, and \
                 zero options is not a decision. State the legal options"
            ),
            DecisionRejection::DuplicateChoice { choice } => write!(
                f,
                "operator decision rejected: choice {choice:?} appears twice — the ruling records \
                 ONE string, so two identical options would make it ambiguous"
            ),
            DecisionRejection::BlankEvidence => write!(
                f,
                "operator decision rejected: an evidence entry is blank — a blank reference is \
                 not evidence"
            ),
            DecisionRejection::BadStatus { status } => write!(
                f,
                "operator decision rejected: status {status:?} is neither 'pending' nor \
                 'resolved'"
            ),
            DecisionRejection::BadDeciderShape { decider } => write!(
                f,
                "operator decision rejected: decider {decider:?} is neither an operator ref \
                 (human:*) nor a seat ref (seat:*) — a fabricated decider is an obligation \
                 nobody owes"
            ),
            DecisionRejection::DecisionIdTaken { decision_id } => write!(
                f,
                "operator decision rejected: {decision_id:?} already exists — a second request \
                 would either duplicate the question or silently rewrite the first. A new \
                 question is a new id"
            ),
            DecisionRejection::NotFound { decision_id } => write!(
                f,
                "operator decision rejected: {decision_id:?} names no obligation"
            ),
            DecisionRejection::NotTheDecider {
                decision_id,
                decider,
            } => write!(
                f,
                "operator decision rejected: {decision_id:?} is for {decider:?} to rule on — a \
                 ruling from anyone else is not a ruling"
            ),
            DecisionRejection::AlreadyResolved { decision_id } => write!(
                f,
                "operator decision rejected: {decision_id:?} is already resolved — a recorded \
                 ruling is immutable, exactly like a terminal acceptance. A new question is a \
                 new decision, never a rewrite of an answered one"
            ),
            DecisionRejection::ChoiceNotOffered {
                decision_id,
                chosen,
                choices,
            } => write!(
                f,
                "operator decision rejected: {chosen:?} is not one of the choices {decision_id:?} \
                 offered ({}) — the ruling must be one of the stated options",
                choices.join(", ")
            ),
            DecisionRejection::ProjectMissing { project_id } => {
                write!(
                    f,
                    "operator decision rejected: project {project_id:?} names no project — an \
                     obligation is never fabricated"
                )
            }
            DecisionRejection::Db(e) => write!(f, "operator decision rejected: {e}"),
        }
    }
}

impl From<DbError> for DecisionRejection {
    fn from(e: DbError) -> Self {
        match e {
            DbError::OperatorDecisionNotFound { id } => {
                DecisionRejection::NotFound { decision_id: id }
            }
            DbError::OperatorDecisionAlreadyResolved { id } => {
                DecisionRejection::AlreadyResolved { decision_id: id }
            }
            DbError::OperatorDecisionChoiceNotOffered {
                id,
                chosen,
                choices,
            } => DecisionRejection::ChoiceNotOffered {
                decision_id: id,
                chosen,
                choices,
            },
            other => DecisionRejection::Db(other.to_string()),
        }
    }
}

impl From<DispatchRejection> for DecisionRejection {
    fn from(e: DispatchRejection) -> Self {
        match e {
            DispatchRejection::MissingActorRef => DecisionRejection::Empty {
                field: "SEATLOOM_ACTOR_REF",
            },
            DispatchRejection::MissingProjectId => DecisionRejection::Empty {
                field: "project_id",
            },
            other => DecisionRejection::Db(other.to_string()),
        }
    }
}

/// The normalised, validated form of a create request — the result of every
/// rule that can be checked without touching the database.
struct ValidatedRequest {
    decision_id: String,
    target: String,
    choices: Vec<String>,
    decider: String,
    consequence: String,
    evidence: Vec<String>,
}

/// Every rule that does not need the database, as ONE function so the tests can
/// exercise them without a live server.
///
/// Splitting these out is not convenience: the two rules the packet makes
/// unconditional (choices present, consequence present) are exactly the ones a
/// caller would most like to skip, and a refusal that names which one is
/// missing is the difference between a fixed call and a guessing loop.
fn validate_request(
    actor_ref: &str,
    args: &RequestOperatorDecisionArgs,
) -> Result<ValidatedRequest, DecisionRejection> {
    if !may_create_obligation(actor_ref) {
        return Err(DecisionRejection::NotOperatorOrAegis {
            actor_ref: actor_ref.to_string(),
        });
    }
    let decision_id = non_empty(&args.decision_id, "decision_id")?;
    let target = non_empty(&args.target, "target")?;
    let consequence = non_empty(&args.consequence, "consequence")?;

    // The choices: trimmed, non-blank, and DISTINCT.
    let mut choices: Vec<String> = Vec::with_capacity(args.choices.len());
    for raw in &args.choices {
        let choice = non_empty(raw, "choices")?;
        if choices.contains(&choice) {
            return Err(DecisionRejection::DuplicateChoice { choice });
        }
        choices.push(choice);
    }
    if choices.is_empty() {
        return Err(DecisionRejection::NoChoices);
    }

    // The evidence chain: each entry non-blank. A blank reference is not
    // evidence — and naming it as such is more useful than the generic
    // "evidence must not be empty", which sounds like the array itself.
    let mut evidence: Vec<String> = Vec::with_capacity(args.evidence.len());
    for raw in &args.evidence {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DecisionRejection::BlankEvidence);
        }
        evidence.push(trimmed.to_string());
    }

    // The decider: omitted means the operator; given, it must be a real actor.
    let decider = match args.decider.as_deref() {
        Some(d) => {
            let d = non_empty(d, "decider")?;
            if !(d.starts_with("human:") || d.starts_with("seat:")) {
                return Err(DecisionRejection::BadDeciderShape { decider: d });
            }
            d
        }
        None => "human:supervisor".to_string(),
    };

    Ok(ValidatedRequest {
        decision_id,
        target,
        choices,
        decider,
        consequence,
        evidence,
    })
}

/// `request_operator_decision(decision_id, target, choices, consequence,
/// evidence?, decider?, project_id?)`.
///
/// The entry rules, in the order they are checked — each one is a refusal that
/// writes nothing:
///
/// 1. the caller is the operator or Aegis (a seat is refused outright);
/// 2. `target` and `consequence` are non-blank — an obligation that does not say
///    what it is about, or what follows from deciding it, is worse than none;
/// 3. `choices` is a non-empty array of distinct non-blank strings;
/// 4. `decider`, when given, is an operator or seat ref;
/// 5. the id is free, and the project exists.
///
/// On success the row and one `OperatorDecisionRequested` land in ONE
/// transaction.
pub async fn request_operator_decision(
    db: &SeatloomDb,
    args: RequestOperatorDecisionArgs,
) -> Result<String, DecisionRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id.clone())?;

    let valid = validate_request(&identity.actor_ref, &args)?;

    // Rule 5 — the id is free and the project is real. Both are facts about the
    // database, checked before any write.
    if db
        .get_operator_decision(&valid.decision_id)
        .await?
        .is_some()
    {
        return Err(DecisionRejection::DecisionIdTaken {
            decision_id: valid.decision_id,
        });
    }
    if db.get_project(&project_id).await?.is_none() {
        return Err(DecisionRejection::ProjectMissing { project_id });
    }

    let row = db
        .request_operator_decision(NewOperatorDecision {
            decision_id: &valid.decision_id,
            project_id: &project_id,
            target: &valid.target,
            choices: &valid.choices,
            decider: &valid.decider,
            consequence: &valid.consequence,
            evidence: &valid.evidence,
            created_by: &identity.actor_ref,
        })
        .await?;

    Ok(format!(
        "requested operator decision {} for {} (created by {}, status {}) — \
         one OperatorDecisionRequested event appended; it now counts in the pending obligations \
         of project {}",
        valid.decision_id, valid.decider, identity.actor_ref, row.status, project_id
    ))
}

/// `resolve_operator_decision(decision_id, chosen)` — the ruling.
///
/// The only rule: the caller IS the decider. A ruling from anyone else is not a
/// ruling, and a second ruling on a resolved obligation is refused (terminal,
/// like a decided acceptance).
pub async fn resolve_operator_decision(
    db: &SeatloomDb,
    args: ResolveOperatorDecisionArgs,
) -> Result<String, DecisionRejection> {
    let identity = resolve_seat()?;
    let decision_id = non_empty(&args.decision_id, "decision_id")?;
    let chosen = non_empty(&args.chosen, "chosen")?;

    let Some(row) = db.get_operator_decision(&decision_id).await? else {
        return Err(DecisionRejection::NotFound { decision_id });
    };
    if row.decider != identity.actor_ref {
        return Err(DecisionRejection::NotTheDecider {
            decision_id,
            decider: row.decider,
        });
    }

    let resolved = db
        .resolve_operator_decision(&decision_id, &chosen, &identity.actor_ref)
        .await?;
    // Read the row back rather than trusting the write: a silently broken write
    // must fail loudly, not produce a plausible-looking message.
    assert_eq!(resolved.status, "resolved");
    assert_eq!(resolved.chosen.as_deref(), Some(chosen.as_str()));

    Ok(format!(
        "resolved operator decision {decision_id}: chose {chosen:?} (ruled by {}) — one \
         OperatorDecisionResolved event appended; the obligation no longer counts as pending",
        identity.actor_ref
    ))
}

/// A required non-blank string field, trimmed.
fn non_empty(value: &str, field: &'static str) -> Result<String, DecisionRejection> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DecisionRejection::Empty { field });
    }
    Ok(trimmed.to_string())
}

/// `list_operator_decisions(status?, decider?, project_id?)` — the obligations,
/// rendered for a human.
///
/// The decider defaults to the CALLER, not to the operator: a seat asking this
/// question is answered about itself, and cannot enumerate what is waiting on
/// the operator. That is the same scoping discipline the rest of the toolset
/// follows (a read never widens its own scope to be helpful).
pub async fn list_operator_decisions(
    db: &SeatloomDb,
    args: ListOperatorDecisionsArgs,
) -> Result<String, DecisionRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id)?;
    let status = match args.status.as_deref() {
        None => "pending",
        Some(s) => {
            let s = s.trim();
            if s.is_empty() || (s != "pending" && s != "resolved") {
                return Err(DecisionRejection::BadStatus {
                    status: s.to_string(),
                });
            }
            s
        }
    };
    let decider = match args.decider.as_deref() {
        Some(d) => {
            let d = non_empty(d, "decider")?;
            if !(d.starts_with("human:") || d.starts_with("seat:")) {
                return Err(DecisionRejection::BadDeciderShape { decider: d });
            }
            d
        }
        None => identity.actor_ref.clone(),
    };
    if db.get_project(&project_id).await?.is_none() {
        return Err(DecisionRejection::ProjectMissing { project_id });
    }

    let rows = if status == "pending" {
        db.list_pending_operator_decisions(&project_id, Some(&decider), 200)
            .await?
    } else {
        // `resolved` is the audit side: every ruling that has landed, newest
        // first, so the most recent decision is the first line.
        db.list_resolved_operator_decisions(&project_id, Some(&decider), 200)
            .await?
    };

    if rows.is_empty() {
        return Ok(format!(
            "no {status} operator decisions for {decider} in project {project_id}"
        ));
    }
    let mut out = format!(
        "{} {status} operator decision(s) for {decider} in project {project_id}:",
        rows.len()
    );
    for row in &rows {
        out.push_str("\n  ");
        out.push_str(&render_one(row, status));
    }
    Ok(out)
}

/// One obligation as two lines: what is being decided and what follows, plus
/// the ruling when there is one.
fn render_one(row: &OperatorDecisionRow, status: &str) -> String {
    let mut s = format!(
        "{} — {}\n      选项: {}\n      定了之后: {}",
        row.id,
        row.target,
        row.choices.join(" / "),
        row.consequence
    );
    if !row.evidence.is_empty() {
        s.push_str(&format!("\n      依据: {}", row.evidence.join(", ")));
    }
    if status == "resolved" {
        s.push_str(&format!(
            "\n      裁定: {} (by {} at {})",
            row.chosen.as_deref().unwrap_or("(未记录)"),
            row.resolved_by.as_deref().unwrap_or("(未记录)"),
            row.resolved_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "(未记录)".to_string())
        ));
    }
    s
}

/// Render the pending obligations as lines a human can read — the shape
/// `fleet-status.sh` prints and the UI panel show.
#[cfg(test)]
fn render_pending(rows: &[OperatorDecisionRow]) -> Vec<String> {
    rows.iter().map(|r| render_one(r, "pending")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The entry rule that matters most: a seat that is not Aegis is REFUSED,
    /// not tolerated. A "waiting on the operator" a seat can create is a seat
    /// that can push its own work onto his plate — which is the failure the
    /// packet rules out by name.
    #[test]
    fn only_the_operator_and_aegis_may_create_an_obligation() {
        // The operator, in both spellings the config module can produce.
        assert!(may_create_obligation("human:supervisor"));
        assert!(may_create_obligation("human:user"));
        // Aegis, exactly.
        assert!(may_create_obligation(AEGIS_ACTOR_REF));
        // Every other seat. This list is the whole fleet on purpose: the rule
        // is "seats may not", not "most seats may not".
        for seat in [
            "seat:seat-nimbus-001",
            "seat:seat-lyra-001",
            "seat:seat-flux-001",
            "seat:seat-mira-001",
            "seat:seat-vela-001",
            "seat:seat-onyx-001",
        ] {
            assert!(
                !may_create_obligation(seat),
                "{seat} must not be able to create an obligation"
            );
        }
        // A near-miss is not a pass: a renamed Aegis, a bare id, a prefix with
        // nobody behind it, and an empty string are all refused.
        //
        // NOTE ON WHAT IS *NOT* HERE. `"human:seat-aegis-001"` — a human ref
        // whose name collides with a seat id — IS allowed, and deliberately so:
        // the operator identity is whatever `~/.seatloom/operator` states, and
        // reinterpreting a person's own configured name would be inventing a
        // rule about what they may call themselves. It is not Aegis (that is
        // `seat:seat-aegis-001`), it is recorded verbatim in the event, and
        // `resolve` still checks decider == caller.
        for near in [
            "seat:seat-aegis",
            "seat-aegis-001",
            "aegis",
            "human",
            "human:",
            "",
            "system",
        ] {
            assert!(
                !may_create_obligation(near),
                "{near:?} must not be treated as an allowed creator"
            );
        }
    }

    /// Every refusal names the rule, so a caller can fix the call without
    /// guessing — and none is a raw database error.
    #[test]
    fn refusals_name_the_rule_broken() {
        let cases: Vec<(DecisionRejection, &[&str])> = vec![
            (
                DecisionRejection::NotOperatorOrAegis {
                    actor_ref: "seat:seat-nimbus-001".into(),
                },
                &["may not create", "seat-aegis-001"],
            ),
            (
                DecisionRejection::NoChoices,
                &["one option is not a choice"],
            ),
            (
                DecisionRejection::DuplicateChoice { choice: "A".into() },
                &["appears twice"],
            ),
            (DecisionRejection::BlankEvidence, &["not evidence"]),
            (
                DecisionRejection::BadDeciderShape {
                    decider: "system".into(),
                },
                &["nobody owes"],
            ),
            (
                DecisionRejection::DecisionIdTaken {
                    decision_id: "dec-x".into(),
                },
                &["already exists"],
            ),
            (
                DecisionRejection::AlreadyResolved {
                    decision_id: "dec-x".into(),
                },
                &["already resolved", "immutable"],
            ),
            (
                DecisionRejection::NotTheDecider {
                    decision_id: "dec-x".into(),
                    decider: "human:supervisor".into(),
                },
                &["not a ruling"],
            ),
            (
                DecisionRejection::ChoiceNotOffered {
                    decision_id: "dec-x".into(),
                    chosen: "C".into(),
                    choices: vec!["A".into(), "B".into()],
                },
                &["A, B"],
            ),
            (DecisionRejection::Empty { field: "target" }, &["target"]),
            (
                DecisionRejection::ProjectMissing {
                    project_id: "nope".into(),
                },
                &["never fabricated"],
            ),
        ];
        for (rejection, needles) in cases {
            let msg = rejection.to_string();
            for needle in needles {
                assert!(
                    msg.contains(needle),
                    "refusal must contain {needle:?}: {msg}"
                );
            }
            // No refusal may be a bare database error.
            assert!(
                !msg.starts_with("operator decision rejected: postgres"),
                "{msg}"
            );
        }
    }

    /// The DbError mapping keeps its typed refusals typed instead of collapsing
    /// every failure into one string — a caller must be able to tell "already
    /// resolved" from "connection died".
    #[test]
    fn db_errors_map_to_typed_refusals_not_strings() {
        assert!(matches!(
            DecisionRejection::from(DbError::OperatorDecisionNotFound { id: "d".into() }),
            DecisionRejection::NotFound { .. }
        ));
        assert!(matches!(
            DecisionRejection::from(DbError::OperatorDecisionAlreadyResolved { id: "d".into() }),
            DecisionRejection::AlreadyResolved { .. }
        ));
        assert!(matches!(
            DecisionRejection::from(DbError::OperatorDecisionChoiceNotOffered {
                id: "d".into(),
                chosen: "C".into(),
                choices: vec!["A".into()],
            }),
            DecisionRejection::ChoiceNotOffered { .. }
        ));
    }

    /// The rendered pending list is what the operator reads. It must carry the
    /// three things that make a decision possible: what is being decided, what
    /// the options are, and what follows from each.
    #[test]
    fn pending_render_carries_target_choices_and_consequence() {
        let rows = vec![OperatorDecisionRow {
            id: "dec-exemption".into(),
            project_id: "seatloom".into(),
            target: "要不要给合并防护加删除豁免".into(),
            choices: vec!["加".into(), "不加".into()],
            decider: "human:supervisor".into(),
            consequence: "加了才能合掉那个死代码删除;不加就得改派单".into(),
            evidence: vec!["disp-DFeH6Ut".into()],
            status: "pending".into(),
            created_by: "seat:seat-aegis-001".into(),
            created_at: chrono::Utc::now(),
            chosen: None,
            resolved_by: None,
            resolved_at: None,
        }];
        let rendered = render_pending(&rows);
        assert_eq!(rendered.len(), 1);
        let line = &rendered[0];
        for needle in [
            "dec-exemption",
            "要不要给合并防护加删除豁免",
            "加 / 不加",
            "定了之后",
            "disp-DFeH6Ut",
        ] {
            assert!(
                line.contains(needle),
                "rendered line must contain {needle:?}: {line}"
            );
        }
        // An empty list renders nothing — not a placeholder that looks like data.
        assert!(render_pending(&[]).is_empty());
    }

    /// The two fields the packet makes unconditional, checked WITHOUT a database:
    /// a request missing `choices` or `consequence` is refused, and the refusal
    /// says which one. This is the test that would fail if someone later decided
    /// an obligation could be created "and the consequence filled in later".
    #[test]
    fn missing_choices_or_consequence_is_refused_by_name() {
        let base = || RequestOperatorDecisionArgs {
            decision_id: "dec-x".into(),
            target: "要不要给防护加豁免".into(),
            choices: vec!["加".into(), "不加".into()],
            decider: None,
            consequence: "加了才能合掉那个删除".into(),
            evidence: vec![],
            project_id: None,
        };

        // A well-formed request passes every pre-database rule.
        let ok = validate_request("seat:seat-aegis-001", &base()).expect("a well-formed request");
        assert_eq!(
            ok.decider, "human:supervisor",
            "the decider defaults to the operator"
        );
        assert_eq!(ok.choices, vec!["加", "不加"]);
        assert!(ok.evidence.is_empty());

        // No choices at all.
        let mut no_choices = base();
        no_choices.choices = vec![];
        assert!(
            matches!(
                validate_request("seat:seat-aegis-001", &no_choices),
                Err(DecisionRejection::NoChoices)
            ),
            "an empty choices array is not a decision"
        );

        // A blank option is as bad as no options.
        let mut blank_choice = base();
        blank_choice.choices = vec!["加".into(), "  ".into()];
        assert!(matches!(
            validate_request("seat:seat-aegis-001", &blank_choice),
            Err(DecisionRejection::Empty { field: "choices" })
        ));

        // A duplicate option: the ruling records ONE string, so two identical
        // options would make it ambiguous.
        let mut dup = base();
        dup.choices = vec!["加".into(), "加".into()];
        assert!(matches!(
            validate_request("seat:seat-aegis-001", &dup),
            Err(DecisionRejection::DuplicateChoice { .. })
        ));

        // No consequence — the one the packet singles out as "worse than none".
        let mut no_consequence = base();
        no_consequence.consequence = "   ".into();
        assert!(matches!(
            validate_request("seat:seat-aegis-001", &no_consequence),
            Err(DecisionRejection::Empty {
                field: "consequence"
            })
        ));

        // A blank evidence entry.
        let mut blank_evidence = base();
        blank_evidence.evidence = vec!["disp-1".into(), String::new()];
        assert!(matches!(
            validate_request("seat:seat-aegis-001", &blank_evidence),
            Err(DecisionRejection::BlankEvidence)
        ));

        // A fabricated decider.
        let mut bad_decider = base();
        bad_decider.decider = Some("system".into());
        assert!(matches!(
            validate_request("seat:seat-aegis-001", &bad_decider),
            Err(DecisionRejection::BadDeciderShape { .. })
        ));

        // And the creator rule, through the same entry point a seat would use.
        assert!(matches!(
            validate_request("seat:seat-nimbus-001", &base()),
            Err(DecisionRejection::NotOperatorOrAegis { .. })
        ));
    }

    /// A resolved row renders its ruling; a pending one does not pretend to have
    /// one. The two shapes are what the audit read and the pending read print.
    #[test]
    fn render_distinguishes_pending_from_resolved() {
        let mut row = OperatorDecisionRow {
            id: "dec-x".into(),
            project_id: "seatloom".into(),
            target: "删不删那个文件".into(),
            choices: vec!["删".into(), "留".into()],
            decider: "human:supervisor".into(),
            consequence: "删了主线就干净了".into(),
            evidence: vec![],
            status: "pending".into(),
            created_by: "seat:seat-aegis-001".into(),
            created_at: chrono::Utc::now(),
            chosen: None,
            resolved_by: None,
            resolved_at: None,
        };
        let pending = render_one(&row, "pending");
        assert!(
            !pending.contains("裁定"),
            "a pending obligation has no ruling: {pending}"
        );

        row.status = "resolved".into();
        row.chosen = Some("删".into());
        row.resolved_by = Some("human:supervisor".into());
        row.resolved_at = Some(chrono::Utc::now());
        let resolved = render_one(&row, "resolved");
        assert!(
            resolved.contains("裁定"),
            "a resolved obligation shows its ruling: {resolved}"
        );
        assert!(resolved.contains('删'));
        assert!(resolved.contains("human:supervisor"));
    }
}
