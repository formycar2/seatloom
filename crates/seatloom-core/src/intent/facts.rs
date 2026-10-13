//! The facts a plan is grounded in — **real reads, gathered once**.
//!
//! A proposal is only as honest as the reads behind it. This struct is the
//! single place the intent tool surface touches the database: the caller (the
//! MCP tool, and through it the Tauri command) gathers these rows and hands
//! them to [`crate::intent::plan::plan`], which is pure. Both surfaces therefore
//! plan over the *same* repository functions — there is no second
//! implementation, only one gatherer and one pure planner.
//!
//! # What is deliberately NOT gathered
//!
//! The whole event and artifact tables. A precondition only ever needs to know
//! whether the *specific* referents the intent named exist, so the caller
//! resolves exactly those ids ([`precondition_candidates`]) and reports the
//! answers here. An unbounded table scan per keystroke would be the second
//! way to make this surface dishonest about cost.

use std::collections::{BTreeMap, BTreeSet};

use crate::intent::schema::DepKind;

/// One seat as the planner sees it: the `seats` row joined to its
/// `project_role_bindings` row for this project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatFact {
    pub seat_id: String,
    pub name: String,
    pub capability_tags: Vec<String>,
    /// `seats.status`. Anything other than `active` is not routable.
    pub status: String,
    /// `project_role_bindings.role`, when the seat is bound to this project.
    pub role: Option<String>,
    pub authority_docs: Vec<String>,
    pub constraints: Vec<String>,
    pub active_delegation_id: Option<String>,
}

impl SeatFact {
    /// A seat is routable when it exists, is active, and is bound to the
    /// project. These are the only two hard facts; everything else is reported,
    /// never used to exclude.
    pub fn is_routable(&self) -> bool {
        self.status == "active" && self.role.is_some()
    }

    /// Why this seat is not routable, as a fact. `None` when it is routable.
    pub fn not_routable_reason(&self, project_id: &str) -> Option<String> {
        if self.status != "active" {
            return Some(format!(
                "seats.status is {:?}, not 'active' (seats row for {})",
                self.status, self.seat_id
            ));
        }
        if self.role.is_none() {
            return Some(format!(
                "no project_role_bindings row for seat {} in project {:?} (project_role_bindings)",
                self.seat_id, project_id
            ));
        }
        None
    }
}

/// Everything the planner is allowed to know.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntentFacts {
    pub project_id: String,
    /// `projects.worker_budget_tokens` — `None` when the project row is absent,
    /// which is reported as absent rather than defaulted to a number.
    pub worker_budget_tokens: Option<i32>,
    /// `projects.supervisor_budget_tokens`.
    pub supervisor_budget_tokens: Option<i32>,
    /// Every seat, sorted by name, joined to its role binding for this project.
    pub seats: Vec<SeatFact>,
    /// `dispatches` in-flight count per seat, for this project.
    pub in_flight_by_seat: BTreeMap<String, usize>,
    /// Every dispatch id in this project.
    pub dispatch_ids: BTreeSet<String>,
    /// Every dispatch title in this project (exact-title precondition matching).
    pub dispatch_titles: BTreeSet<String>,
    /// Existence answers for the referent ids the intent actually named,
    /// resolved by the caller. A missing key means "not checked", which the
    /// planner treats as not existing.
    pub named_referent_exists: BTreeMap<(DepKind, String), bool>,
}

impl IntentFacts {
    /// The seats the planner may route to, in name order.
    pub fn routable_seats(&self) -> Vec<&SeatFact> {
        self.seats.iter().filter(|s| s.is_routable()).collect()
    }

    pub fn seat(&self, seat_id: &str) -> Option<&SeatFact> {
        self.seats.iter().find(|s| s.seat_id == seat_id)
    }

    /// In-flight dispatch count for a seat, from the gathered `dispatches` rows.
    pub fn in_flight(&self, seat_id: &str) -> usize {
        self.in_flight_by_seat.get(seat_id).copied().unwrap_or(0)
    }

    /// Does this referent exist? Dispatch ids come from the gathered rows;
    /// event and artifact ids come from the caller's targeted answers.
    pub fn referent_exists(&self, kind: DepKind, value: &str) -> bool {
        match kind {
            DepKind::Dispatch => self.dispatch_ids.contains(value),
            DepKind::Event | DepKind::Artifact => self
                .named_referent_exists
                .get(&(kind, value.to_string()))
                .copied()
                .unwrap_or(false),
        }
    }

    /// Every capability tag any seat in this project carries — the closed
    /// vocabulary the intent's tokens are matched against, exactly.
    pub fn known_capability_tags(&self) -> BTreeSet<String> {
        self.seats
            .iter()
            .flat_map(|s| s.capability_tags.iter().cloned())
            .collect()
    }
}

/// The referent ids a precondition clause names, with the kind each id's
/// spelling implies. Deterministic and total: a token that names no known
/// prefix is not a candidate at all.
///
/// The prefixes are the project's own id conventions (`ev-` canonical event,
/// `art-` artifact, `disp-` dispatch), which is why they can be used as a
/// spelling rule rather than a guess.
pub fn precondition_candidates(clause: &str) -> Vec<(DepKind, String)> {
    let mut out: Vec<(DepKind, String)> = Vec::new();
    for token in super::parser::latin_terms(clause) {
        let kind = if token.starts_with("ev-") {
            Some(DepKind::Event)
        } else if token.starts_with("art-") {
            Some(DepKind::Artifact)
        } else if token.starts_with("disp-") {
            Some(DepKind::Dispatch)
        } else {
            None
        };
        if let Some(kind) = kind {
            if !out.iter().any(|(_, v)| v == &token) {
                out.push((kind, token));
            }
        }
    }
    out
}

/// The referent ids an intent's precondition names — the caller resolves
/// **exactly these** (and nothing else) before calling
/// [`crate::intent::plan::plan`]. Bounding the existence queries to the ids the
/// sentence actually names is what keeps the gatherer honest about cost: a
/// keystroke never triggers a table scan.
pub fn named_referents(input: &str) -> Vec<(DepKind, String)> {
    match super::parser::parse(input) {
        super::parser::IntentParse::Parsed(intent) => intent
            .qualifiers
            .precondition
            .as_deref()
            .map(precondition_candidates)
            .unwrap_or_default(),
        // A sentence that does not parse names no referent, so nothing is
        // resolved on its behalf.
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat(id: &str, name: &str, tags: &[&str], status: &str, role: Option<&str>) -> SeatFact {
        SeatFact {
            seat_id: id.to_string(),
            name: name.to_string(),
            capability_tags: tags.iter().map(|t| t.to_string()).collect(),
            status: status.to_string(),
            role: role.map(|r| r.to_string()),
            authority_docs: vec![],
            constraints: vec![],
            active_delegation_id: None,
        }
    }

    #[test]
    fn routability_is_two_facts_and_nothing_else() {
        let active = seat(
            "seat-nimbus-001",
            "nimbus",
            &["rust"],
            "active",
            Some("architect"),
        );
        assert!(active.is_routable());
        assert_eq!(active.not_routable_reason("seatloom"), None);

        let retired = seat("seat-x", "x", &[], "retired", Some("architect"));
        assert!(!retired.is_routable());
        assert!(retired
            .not_routable_reason("seatloom")
            .unwrap()
            .contains("seats.status"));

        let unbound = seat("seat-y", "y", &[], "active", None);
        assert!(!unbound.is_routable());
        assert!(unbound
            .not_routable_reason("seatloom")
            .unwrap()
            .contains("project_role_bindings"));
    }

    #[test]
    fn precondition_candidates_read_the_id_spelling_only() {
        let got = precondition_candidates("等 ev-supervisor-intent-tools-landed 落地,别等 art-020");
        assert_eq!(
            got,
            vec![
                (DepKind::Artifact, "art-020".to_string()),
                (
                    DepKind::Event,
                    "ev-supervisor-intent-tools-landed".to_string()
                ),
            ],
            "latin_terms is sorted, so the order is deterministic"
        );
        // A clause with no id-shaped token yields no candidate, so nothing is
        // invented to satisfy it.
        assert!(precondition_candidates("等 Flux 验证通过").is_empty());
    }

    #[test]
    fn named_referents_are_bounded_to_what_the_sentence_names() {
        assert_eq!(
            named_referents("给 nimbus 派个活,等 ev-x 落地再动"),
            vec![(DepKind::Event, "ev-x".to_string())]
        );
        // No precondition, no referent, no query.
        assert!(named_referents("给 nimbus 派个活").is_empty());
        // A clause that names no id-shaped token yields nothing to resolve.
        assert!(named_referents("给 nimbus 派个活,等 Flux 验证通过再动").is_empty());
    }

    #[test]
    fn a_referent_is_only_known_when_it_was_checked() {
        let mut facts = IntentFacts::default();
        facts.dispatch_ids.insert("disp-a".to_string());
        assert!(facts.referent_exists(DepKind::Dispatch, "disp-a"));
        assert!(!facts.referent_exists(DepKind::Dispatch, "disp-b"));
        assert!(!facts.referent_exists(DepKind::Event, "ev-a"));
        facts
            .named_referent_exists
            .insert((DepKind::Event, "ev-a".to_string()), true);
        assert!(facts.referent_exists(DepKind::Event, "ev-a"));
    }
}
