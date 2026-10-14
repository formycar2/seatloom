//! `plan_intent` — the MCP tool half of the Supervisor intent tool surface.
//!
//! Product contract: `LYRA-2026-10-13-supervisor-intent-entry-contract-v1`,
//! slice 1. This module turns **one sentence** into a typed, reviewable result:
//! a proposal, a clarification, or an exact retrieval plan. It performs **no
//! write of any kind** — the write happens only when the operator confirms, and
//! then through `enqueue_dispatch`, which already exists and already works.
//!
//! # Why there is no second implementation
//!
//! The Tauri command `cmd_plan_intent` shells out to *this* tool over stdio, the
//! same way `cmd_enqueue_dispatch` shells out to `enqueue_dispatch`. The pure
//! logic lives in `seatloom_core::intent` and the single database gatherer is
//! `SeatloomDb::gather_intent_facts`. There is exactly one of each.
//!
//! # Identity
//!
//! The acting actor is resolved server-side from `SEATLOOM_ACTOR_REF`, exactly
//! like every other tool here. For the UI that is `human:supervisor`, which is
//! what makes the eventual write attributable to the person who asked — the
//! audit-trail property the whole feature exists for.

use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{json, Value};

use seatloom_core::db::SeatloomDb;
use seatloom_core::intent::{self, IntentFacts, IntentPlan};

use crate::dispatch::{resolve_project, resolve_seat};
use crate::emit::ACTOR_REF_ENV;

/// The governing product contract, named on every answer so a reader of the
/// ledger can trace a proposal back to the rules it was built under.
pub const INTENT_CONTRACT: &str = "LYRA-2026-10-13-supervisor-intent-entry-contract-v1";

/// How many of a project's dispatches the gatherer reads. Load counting needs
/// the live in-flight set, not history; the ceiling is stated here so a caller
/// can never silently read an unbounded table.
pub const DISPATCH_FACTS_LIMIT: i64 = 5_000;

#[derive(Debug, Deserialize)]
pub struct PlanIntentArgs {
    /// The one sentence the operator typed.
    pub intent: String,
    #[serde(default)]
    pub project_id: Option<String>,
}

/// Why a `plan_intent` call was refused. Every variant names what is missing
/// rather than defaulting it.
#[derive(Debug)]
pub enum IntentRejection {
    EmptyIntent,
    MissingActorRef,
    Db(String),
}

impl std::fmt::Display for IntentRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentRejection::EmptyIntent => {
                write!(
                    f,
                    "plan_intent rejected: the intent is empty — there is nothing to plan"
                )
            }
            IntentRejection::MissingActorRef => write!(
                f,
                "plan_intent rejected: {ACTOR_REF_ENV} is not set. The eventual write is \
                 attributed to whoever confirms, so the actor is never invented here."
            ),
            IntentRejection::Db(e) => write!(f, "plan_intent rejected: {e}"),
        }
    }
}

/// Plan one intent. Reads only; appends nothing.
pub async fn plan_intent(
    db: &SeatloomDb,
    _pool: &Pool,
    args: PlanIntentArgs,
) -> Result<String, IntentRejection> {
    if args.intent.trim().is_empty() {
        return Err(IntentRejection::EmptyIntent);
    }
    // Identity is server-side, never a parameter — the same rule as every other
    // tool on this server.
    let actor = std::env::var(ACTOR_REF_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or(IntentRejection::MissingActorRef)?;
    let _ = resolve_seat().map_err(|_| IntentRejection::MissingActorRef)?;
    let project_id =
        resolve_project(args.project_id).map_err(|e| IntentRejection::Db(e.to_string()))?;

    // The project must exist: planning against a project that is not there would
    // produce a confident "no eligible seat" for the wrong reason.
    if db.get_project(&project_id).await.map_err(db_err)?.is_none() {
        return Err(IntentRejection::Db(format!(
            "project {project_id:?} does not exist (projects)"
        )));
    }

    let mut facts: IntentFacts = db
        .gather_intent_facts(&project_id, DISPATCH_FACTS_LIMIT)
        .await
        .map_err(db_err)?;

    // Resolve exactly the referents the sentence names — and nothing else.
    for (kind, value) in intent::facts::named_referents(&args.intent) {
        let exists = db
            .referent_exists(kind.as_str(), &value)
            .await
            .map_err(db_err)?;
        facts.named_referent_exists.insert((kind, value), exists);
    }

    let outcome = intent::plan(&facts, &args.intent, &actor);
    let value = match outcome {
        IntentPlan::Proposal(proposal) => json!({
            "contract": INTENT_CONTRACT,
            "outcome": "proposal",
            "wrote": false,
            "proposal": proposal,
        }),
        IntentPlan::Clarification(clarification) => json!({
            "contract": INTENT_CONTRACT,
            "outcome": "clarification",
            "wrote": false,
            "clarification": clarification,
        }),
        IntentPlan::Retrieval(retrieval) => json!({
            "contract": INTENT_CONTRACT,
            "outcome": "retrieval",
            "wrote": false,
            "retrieval": retrieval,
        }),
    };
    serde_json::to_string_pretty(&value).map_err(|e| IntentRejection::Db(e.to_string()))
}

fn db_err(e: seatloom_core::db::repositories::DbError) -> IntentRejection {
    IntentRejection::Db(e.to_string())
}

/// The `plan_intent` tool's `tools/list` entry. Kept next to the implementation
/// so the advertised schema and the accepted arguments cannot drift.
pub fn plan_intent_tool() -> Value {
    json!({
        "name": "plan_intent",
        "description": "Turn ONE sentence into a structured, reviewable result — and never a \
            write. This is the tool half of the Supervisor intent entry (product contract \
            LYRA-2026-10-13-supervisor-intent-entry-contract-v1, slice 1). It returns exactly one \
            of three typed outcomes: `proposal` (a reviewable card whose every field is filled \
            from real reads — seat registry, role bindings, dispatch load, project budgets — with \
            a per-seat routing rationale), `clarification` (the flow stopped at a question, with \
            the real choices and the reason each is offered), or `retrieval` (for a why/evidence \
            verb: the exact reads to make, never a composed answer).\n\n\
            THE RULES IT ENFORCES:\n\
            * Deterministic parse, no LLM. An unparsable intent is a typed result, not something \
            repaired by inference.\n\
            * Resolve or STOP. A seat is selected only when the sentence named one and that name \
            matches exactly one seat. Capability tags order a candidate list; they never select. \
            When the intent is unclear the system asks.\n\
            * A precondition becomes a dependency edge only when it names a referent that exists; \
            otherwise confirmation is blocked and the missing capability is named. It is never \
            silently dropped.\n\
            * No hardcoded field survives: the budget is projects.worker_budget_tokens / \
            projects.supervisor_budget_tokens plus the seat's real in-flight count, never a \
            constant string.\n\
            * It writes nothing. On the operator's Confirm the caller passes the proposal's \
            `suggested_dependencies` straight into `enqueue_dispatch`, whose spelling this tool \
            reuses verbatim. Identity is resolved server-side from SEATLOOM_ACTOR_REF (for the UI \
            that is human:supervisor), never a parameter.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["intent"],
            "properties": {
                "intent": {
                    "type": "string",
                    "minLength": 1,
                    "description": "The one sentence the operator typed, verbatim."
                },
                "project_id": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Optional only because SEATLOOM_PROJECT_ID can supply it; a project is never fabricated."
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_advertises_exactly_one_required_argument() {
        let tool = plan_intent_tool();
        assert_eq!(tool["name"], "plan_intent");
        let required = tool["inputSchema"]["required"].as_array().unwrap();
        assert_eq!(required, &vec![Value::String("intent".to_string())]);
        // No identity parameter: the actor is server-side.
        let props = tool["inputSchema"]["properties"].as_object().unwrap();
        assert!(!props.contains_key("actor_ref"));
        assert!(!props.contains_key("seat_id"));
        assert!(!props.contains_key("target_seat_id"));
    }

    #[test]
    fn the_description_states_the_no_write_rule() {
        let tool = plan_intent_tool();
        let desc = tool["description"].as_str().unwrap();
        assert!(desc.contains("never a write"), "{desc}");
        assert!(desc.contains("It writes nothing"), "{desc}");
        assert!(desc.contains("enqueue_dispatch"), "{desc}");
    }

    #[test]
    fn an_empty_intent_is_refused_before_any_read() {
        let err = IntentRejection::EmptyIntent.to_string();
        assert!(err.contains("nothing to plan"), "{err}");
    }

    #[test]
    fn a_missing_actor_is_refused_rather_than_invented() {
        let err = IntentRejection::MissingActorRef.to_string();
        assert!(err.contains(ACTOR_REF_ENV), "{err}");
        assert!(err.contains("never invented"), "{err}");
    }

    #[test]
    fn dep_kind_spellings_match_the_dispatch_domain() {
        // The proposal's edges are passed straight into enqueue_dispatch, so the
        // spellings must be the dispatch domain's spellings.
        assert_eq!(seatloom_core::intent::DepKind::Event.as_str(), "event");
        assert_eq!(
            seatloom_core::intent::DepKind::Artifact.as_str(),
            "artifact"
        );
        assert_eq!(
            seatloom_core::intent::DepKind::Dispatch.as_str(),
            "dispatch"
        );
        assert_eq!(
            seatloom_core::intent::DepKind::Event.referent_table(),
            "canonical_events"
        );
    }
}
