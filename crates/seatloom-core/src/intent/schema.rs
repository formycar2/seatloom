//! The typed intent schema — **data, not prose**.
//!
//! This module is the first acceptance criterion of the Supervisor intent tool
//! surface: a typed intent schema whose ambiguity outcomes are *typed results*,
//! not error strings. Every type below is a closed domain with a hand-written
//! `FromStr`/`Display` pair, because each of them crosses a boundary (Rust ⇄
//! JSON ⇄ PostgreSQL TEXT) and serde's derived representation does not match a
//! flat TEXT column (ENGINEERING_STANDARDS.md, serialization boundary rule).
//!
//! # Value-domain contract
//!
//! Reproduced verbatim in the governing delivery note.
//!
//! | Domain | Rust type | Serialized spelling | Nullable |
//! |---|---|---|---|
//! | verb | [`IntentVerb`] | kebab-case: `create` `route` `explain` `find-evidence` `recover` | no |
//! | object type | [`IntentObjectType`] | kebab-case: `dispatch` `workitem` `handoff` `comment` | no |
//! | dependency kind | [`DepKind`] | snake_case: `event` `artifact` `dispatch` | no |
//! | dependency referent | `String` | the referent id, verbatim | no |
//! | plan outcome | [`IntentPlanKind`] | kebab-case: `proposal` `clarification` `retrieval` | no |
//! | seat eligibility | `bool` | `true` / `false` | no |
//!
//! [`DepKind`] deliberately reuses the governed-dispatch domain (schema `012`)
//! instead of inventing a second one: a proposal's `suggested_dependencies` are
//! passed straight into `enqueue_dispatch`'s `dependencies` array, so the two
//! spellings must be the same spelling. A second spelling here would be the
//! "second implementation" the contract forbids.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

// =============================================================================
// Verb
// =============================================================================

/// The five supported verbs (product contract §3.2). Closed domain: a verb
/// outside this set is a *result* ([`crate::intent::parser::IntentParse::UnsupportedVerb`]),
/// never something to repair by guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentVerb {
    /// Bring a new object into existence (`派个活`, `create a dispatch`).
    Create,
    /// Send an object to a seat (`派给 flux`, `assign to onyx`).
    Route,
    /// Answer a why-question from evidence (`为什么放弃 passport.js?`).
    Explain,
    /// Point at the evidence for a claim (`找证据`).
    FindEvidence,
    /// Resume something that stopped (`恢复`, `接着做`).
    Recover,
}

impl IntentVerb {
    /// The closed domain, in vocabulary order. The order is the tie-break for
    /// every deterministic choice in the parser, so it is stated once here.
    pub const ALL: [IntentVerb; 5] = [
        IntentVerb::Create,
        IntentVerb::Route,
        IntentVerb::Explain,
        IntentVerb::FindEvidence,
        IntentVerb::Recover,
    ];

    /// The wire spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentVerb::Create => "create",
            IntentVerb::Route => "route",
            IntentVerb::Explain => "explain",
            IntentVerb::FindEvidence => "find-evidence",
            IntentVerb::Recover => "recover",
        }
    }

    /// One-line example, shown verbatim when the verb is unsupported
    /// (contract §3.3 row 1: "Show the five supported verbs with one-line
    /// examples and ask which").
    pub fn one_line_example(&self) -> &'static str {
        match self {
            IntentVerb::Create => "给 nimbus 派个活,把唤醒服务的硬底兜底接上",
            IntentVerb::Route => "把这个派给 flux 验证",
            IntentVerb::Explain => "我们为什么放弃了 passport.js?",
            IntentVerb::FindEvidence => "找出 020 门判据的证据",
            IntentVerb::Recover => "恢复昨天晚上中断的派单",
        }
    }

    /// Does this verb create or route an object (and therefore need an owner
    /// and an object type before anything can be proposed)?
    pub fn needs_object(&self) -> bool {
        matches!(self, IntentVerb::Create | IntentVerb::Route)
    }
}

impl fmt::Display for IntentVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for IntentVerb {
    type Err = UnknownDomain;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        IntentVerb::ALL
            .into_iter()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| {
                UnknownDomain::new("verb", s, "create|route|explain|find-evidence|recover")
            })
    }
}

// =============================================================================
// Object type
// =============================================================================

/// The four object types an intent can name (contract §3.2). Closed domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentObjectType {
    /// A governed dispatch — the object this project already routes work with.
    Dispatch,
    Workitem,
    Handoff,
    Comment,
}

impl IntentObjectType {
    pub const ALL: [IntentObjectType; 4] = [
        IntentObjectType::Dispatch,
        IntentObjectType::Workitem,
        IntentObjectType::Handoff,
        IntentObjectType::Comment,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            IntentObjectType::Dispatch => "dispatch",
            IntentObjectType::Workitem => "workitem",
            IntentObjectType::Handoff => "handoff",
            IntentObjectType::Comment => "comment",
        }
    }

    /// One-line example, shown when the object type is unclear (§3.3 row 5).
    pub fn one_line_example(&self) -> &'static str {
        match self {
            IntentObjectType::Dispatch => "派单( governed dispatch,由 seat 自主拉取)",
            IntentObjectType::Workitem => "工作项( 需要人跟进的产物)",
            IntentObjectType::Handoff => "交接( 把一件事交给另一个 seat)",
            IntentObjectType::Comment => "评论( 对已有对象的批注)",
        }
    }
}

impl fmt::Display for IntentObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for IntentObjectType {
    type Err = UnknownDomain;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        IntentObjectType::ALL
            .into_iter()
            .find(|o| o.as_str() == s)
            .ok_or_else(|| {
                UnknownDomain::new("object type", s, "dispatch|workitem|handoff|comment")
            })
    }
}

// =============================================================================
// Dependency kind — the governed-dispatch domain, reused verbatim (012)
// =============================================================================

/// The kind of a dependency referent. **Same domain, same spelling** as
/// `enqueue_dispatch`'s `dependencies[].dep_kind` (schema `012`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepKind {
    /// `canonical_events.id`
    Event,
    /// `artifacts.id`
    Artifact,
    /// `dispatches.id`
    Dispatch,
}

impl DepKind {
    pub const ALL: [DepKind; 3] = [DepKind::Event, DepKind::Artifact, DepKind::Dispatch];

    pub fn as_str(&self) -> &'static str {
        match self {
            DepKind::Event => "event",
            DepKind::Artifact => "artifact",
            DepKind::Dispatch => "dispatch",
        }
    }

    /// The table this kind's referent must exist in — stated so a reader never
    /// has to infer it from the name.
    pub fn referent_table(&self) -> &'static str {
        match self {
            DepKind::Event => "canonical_events",
            DepKind::Artifact => "artifacts",
            DepKind::Dispatch => "dispatches",
        }
    }
}

impl fmt::Display for DepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DepKind {
    type Err = UnknownDomain;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DepKind::ALL
            .into_iter()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| UnknownDomain::new("dep_kind", s, "event|artifact|dispatch"))
    }
}

// =============================================================================
// Typed rejection for an unknown domain value
// =============================================================================

/// A value outside a closed domain. Carries the domain, the offending value and
/// the expected set, so the message is actionable instead of "invalid input".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownDomain {
    pub domain: &'static str,
    pub value: String,
    pub expected: &'static str,
}

impl UnknownDomain {
    pub fn new(domain: &'static str, value: &str, expected: &'static str) -> Self {
        Self {
            domain,
            value: value.to_string(),
            expected,
        }
    }
}

impl fmt::Display for UnknownDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:?} is not one of {} (hand-written FromStr; an unknown value is never defaulted)",
            self.domain, self.value, self.expected
        )
    }
}

impl std::error::Error for UnknownDomain {}

// =============================================================================
// The parsed intent
// =============================================================================

/// Qualifiers lifted out of the sentence, each **verbatim** — nothing is
/// normalised, expanded or interpreted at parse time. Interpretation happens
/// later, against real reads, and every interpretation is reported as a fact.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentQualifiers {
    /// The subject the intent is about, as typed.
    pub topic: Option<String>,
    /// A branch the intent names, as typed.
    pub branch: Option<String>,
    /// A precondition clause, as typed (`等 X 落地再动` → `X 落地`). It becomes
    /// a dependency edge only if it resolves to a real referent id; otherwise it
    /// is reported unresolved and the flow stops (contract §3.3 row 6).
    pub precondition: Option<String>,
    /// A budget hint, as typed. Never used as the budget itself — the budget is
    /// computed from `projects.worker_budget_tokens` and the seat's real load.
    pub budget_hint: Option<String>,
}

/// One parsed intent. `raw` is the sentence verbatim; every other field is
/// derived from it by the deterministic parser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Intent {
    pub verb: IntentVerb,
    /// `None` only for verbs that do not create an object.
    pub object_type: Option<IntentObjectType>,
    /// The owner as typed (`给 Nimbus` → `Nimbus`). Resolution against the seat
    /// registry happens later and is exact-match only.
    pub owner_hint: Option<String>,
    pub qualifiers: IntentQualifiers,
    /// Every latin token in the sentence, lowercased, deduplicated, sorted.
    /// Used for exact capability-tag and seat-name matching — never for
    /// keyword guessing.
    pub terms: Vec<String>,
    pub raw: String,
}

// =============================================================================
// The plan outcome
// =============================================================================

/// What `plan_intent` decided. A clarification is a *result*, not a failure:
/// the system stopped and asked instead of selecting a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentPlanKind {
    /// A reviewable proposal. Nothing has been written.
    Proposal,
    /// The flow stopped at a question (§3.3).
    Clarification,
    /// A why/evidence verb: the exact retrieval plan, not a composed answer.
    Retrieval,
}

impl IntentPlanKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentPlanKind::Proposal => "proposal",
            IntentPlanKind::Clarification => "clarification",
            IntentPlanKind::Retrieval => "retrieval",
        }
    }
}

impl fmt::Display for IntentPlanKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One choice offered by a clarification, with the reason it is offered.
/// "the real choices, and the reason each choice is offered" (§4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClarificationChoice {
    /// The value to send back (a seat id, a verb, an object type).
    pub value: String,
    /// What the user sees.
    pub label: String,
    /// Why this choice is on the list — a fact with a source.
    pub why: String,
}

/// The flow stopped. One question, the real choices, the reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clarification {
    /// Which §3.3 row fired, so the rule is auditable rather than implied.
    pub rule_id: String,
    /// The one question.
    pub question: String,
    /// Why the flow stopped here.
    pub reason: String,
    pub choices: Vec<ClarificationChoice>,
}

/// A dependency edge to attach on confirm, in `enqueue_dispatch`'s spelling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedDependency {
    pub dep_kind: DepKind,
    pub dep_value: String,
}

/// How a stated precondition fared. A precondition is never silently dropped:
/// it either becomes an edge or it blocks confirmation with a named reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreconditionResolution {
    /// The clause as typed.
    pub raw: Option<String>,
    /// The edge, when the clause resolved to a real referent.
    pub edge: Option<SuggestedDependency>,
    /// Which deterministic rule matched — a fact, not a claim.
    pub matched_rule: Option<String>,
    /// Why it did not resolve, when it did not.
    pub unresolved_reason: Option<String>,
}

/// One evidence line: a real read that grounds the proposal, with its source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceLink {
    pub label: String,
    /// The table or document the fact came from.
    pub source: String,
}

/// The budget, computed. Every number is a column value; no constant string
/// survives (the prototype's `budget: '150,000 tokens'` is the reference for
/// what "hardcoded" means).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetFacts {
    pub project_worker_budget_tokens: Option<i32>,
    pub project_supervisor_budget_tokens: Option<i32>,
    /// The selected seat's in-flight dispatch count, from `dispatches`.
    pub seat_in_flight_dispatches: usize,
    /// The hint as typed, when the intent carried one.
    pub stated_hint: Option<String>,
    /// What the numbers are and where they came from.
    pub note: String,
}

/// The proposed object, in `enqueue_dispatch`'s shape so the caller can pass it
/// through without translating anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedObject {
    pub kind: IntentObjectType,
    /// One-line routing summary, derived from the intent (never a constant).
    pub title: String,
    /// The intent verbatim, which is what the payload note carries.
    pub payload_note: String,
    pub suggested_dependencies: Vec<SuggestedDependency>,
}

/// Ownership of the proposed object, from the seat's real rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ownership {
    pub seat_id: String,
    pub seat_name: String,
    pub role: Option<String>,
    /// Capability tags stated in the intent AND present on the seat.
    pub capability_match: Vec<String>,
    pub authority_docs: Vec<String>,
    pub constraints: Vec<String>,
    pub active_delegation_id: Option<String>,
}

/// One seat's routing rationale. Every field is a fact read from the database;
/// `excluding_constraint` is why the seat was NOT selected, stated as a fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatRationale {
    pub seat_id: String,
    pub seat_name: String,
    pub eligible: bool,
    pub matched_capabilities: Vec<String>,
    pub excluding_constraint: Option<String>,
    pub role: Option<String>,
    pub authority_docs: Vec<String>,
    pub constraints: Vec<String>,
    pub in_flight_dispatches: usize,
    pub selected: bool,
    /// Where these facts came from.
    pub source: String,
}

/// A reviewable proposal. **Nothing has been written** — the write happens only
/// on the user's `Confirm`, through `enqueue_dispatch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    /// The intent restated in one line.
    pub intent_restated: String,
    pub proposed_object: ProposedObject,
    pub ownership: Ownership,
    pub evidence: Vec<EvidenceLink>,
    pub budget: BudgetFacts,
    /// What changes on confirm — facts about the write path, not adjectives.
    pub impact: Vec<String>,
    pub routing_rationale: Vec<SeatRationale>,
    pub precondition: PreconditionResolution,
    /// Set when a §3.3 row blocks confirmation (e.g. an unresolvable
    /// precondition). While set, `Confirm` is not offered.
    pub blocking_clarification: Option<Clarification>,
    /// The actions the card offers. `confirm` appears only when nothing blocks.
    pub actions: Vec<String>,
    /// Who the write will be attributed to, resolved server-side.
    pub actor: String,
}

/// The exact retrieval plan for a why/evidence/recover verb. The tool never
/// composes an answer: it says which reads to make, and the answer is built only
/// from what those reads return. "Insufficient evidence is a first-class
/// result" (§3.2) is enforced by never producing an answer here at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalPlan {
    pub verb: IntentVerb,
    /// The topic the intent named, as typed — the caller resolves it against
    /// `get_knowledge_tree` rather than having this tool guess an id.
    pub topic_hint: Option<String>,
    /// Terms to filter memories by (exact match over title + l0_summary).
    pub recall_query_terms: Vec<String>,
    /// The exact reads to make, named as tool calls.
    pub reads: Vec<String>,
    pub note: String,
}
