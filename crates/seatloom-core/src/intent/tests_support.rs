//! Shared test fixtures for the intent modules. One world, used by the parser
//! and the planner tests, so the two suites can never drift into asserting
//! against different data.

use crate::intent::facts::{IntentFacts, SeatFact};
use crate::intent::schema::DepKind;

/// A three-seat project: two routable, one retired on purpose (so the
/// not-routable path is exercised rather than assumed).
pub fn facts() -> IntentFacts {
    let mut f = IntentFacts {
        project_id: "seatloom".to_string(),
        worker_budget_tokens: Some(8192),
        supervisor_budget_tokens: Some(32768),
        ..IntentFacts::default()
    };
    f.seats = vec![
        SeatFact {
            seat_id: "seat-nimbus-001".to_string(),
            name: "nimbus".to_string(),
            capability_tags: vec!["rust".to_string(), "tauri".to_string()],
            status: "active".to_string(),
            role: Some("architect".to_string()),
            authority_docs: vec!["docs/architecture-design.md".to_string()],
            constraints: vec!["no-runtime-widening".to_string()],
            active_delegation_id: None,
        },
        SeatFact {
            seat_id: "seat-flux-001".to_string(),
            name: "flux".to_string(),
            capability_tags: vec!["qa".to_string(), "verification".to_string()],
            status: "active".to_string(),
            role: Some("verifier".to_string()),
            authority_docs: vec![],
            constraints: vec![],
            active_delegation_id: None,
        },
        SeatFact {
            seat_id: "seat-onyx-001".to_string(),
            name: "onyx".to_string(),
            capability_tags: vec!["postgres".to_string()],
            status: "retired".to_string(),
            role: Some("data_engineer".to_string()),
            authority_docs: vec![],
            constraints: vec![],
            active_delegation_id: None,
        },
    ];
    f.in_flight_by_seat.insert("seat-nimbus-001".to_string(), 2);
    f.dispatch_ids.insert("disp-a".to_string());
    f.dispatch_titles.insert("Verify the gate".to_string());
    f.named_referent_exists
        .insert((DepKind::Event, "ev-x".to_string()), true);
    f
}
