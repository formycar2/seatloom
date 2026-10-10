//! Closed value domains that cross the Rust ⇄ PostgreSQL TEXT boundary for the
//! P0 event ledger (`canonical_events`, `event_object_refs`).
//!
//! Every enum here hand-writes `FromStr` and `Display` and derives **no** serde
//! representation. ENGINEERING_STANDARDS, "Serialization boundary rule": serde's
//! tagged representation does not match a flat Postgres TEXT column. The
//! reference pattern is `objects/session.rs::parse_custom_tagged`.
//!
//! Unlike `Harness`/`Agent` these three domains are **closed** — there is no
//! `Custom` escape hatch, because each is backed by a `CHECK` constraint in
//! `infra/postgres/schema/010_p0_schema_foundation.sql`. A value the database
//! would reject must fail in Rust first, with a typed error.
//!
//! Casing is not uniform, and that is deliberate: `event_type` is PascalCase and
//! `role`/`ref_type` are snake_case, because that is what is already committed in
//! PostgreSQL (AD-011). Do not harmonize them.

use crate::objects::session::ParseValueError;
use std::fmt;
use std::str::FromStr;

/// Closed `canonical_events.event_type` vocabulary — the 12 values admitted by
/// `010`'s `canonical_events_event_type_vocab` CHECK (`010:141-159`), widened
/// twice since: `012` (+3 dispatch lifecycle, G19) and `013`/`014` (+3 dispatch
/// failure/retry/cancel, +3 acceptance gate). Each widening carried its Lyra
/// value-domain sign-off; see the variant comments.
///
/// **`ArtifactProduced` is deliberately absent.** An earlier draft of `010` added
/// it; it was dropped on Lyra's sign-off (`AEGIS-2026-09-30-p0-step1-signoff-v1`,
/// ratified by Mr. Zhang) because the first capture slice reuses the existing
/// `ArtifactCreated` rather than introducing a near-synonym into an append-only
/// table, where two words for one meaning can never be withdrawn once rows carry
/// both. Do not re-add it without the same sign-off.
///
/// This is **not** `ledger::event::EventType`. That enum is the JSONL file
/// ledger's vocabulary (27 variants, serde-derived, AD-005) and is wider than
/// what PostgreSQL admits. See the note in `write_api`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    SessionStarted,
    WorkItemCreated,
    WorkItemStatusChanged,
    ArtifactCreated,
    HandoffSent,
    HandoffAccepted,
    HandoffCompleted,
    ReviewVerdictIssued,
    SeatDelegationIssued,
    SeatDelegationClosed,
    SupervisorMessage,
    SeatResponse,
    // (012, G19) dispatch lifecycle — the three transitions that make the
    // `dispatches` projection replayable from the ledger. Lyra sign-off in the
    // value-domain contract 2026-10-08; added to A1 under
    // LYRA-2026-10-08-p0-step6-unblock-v1 (7-A). Pinned to `012:73-75`.
    SeatDispatchIssued,
    SeatDispatchDelivered,
    SeatDispatchCompleted,
    // (013, G19 follow-on) dispatch terminal / retry / cancel transitions —
    // each exactly one (from -> to) transition, so the projection stays
    // replayable (a single "sometimes-retry-sometimes-terminal" event would
    // make the ledger ambiguous to replay). Lyra sign-off 2026-10-09
    // (VALUE-DOMAIN sign-off A; ONYX-2026-10-09-dispatch-states-design §2.5).
    SeatDispatchFailed,
    SeatDispatchRetried,
    SeatDispatchCancelled,
    // (014, D2 acceptance gate) the acceptance object's three transitions —
    // acceptance is its own first-class object (actor = operator, not the
    // seat). Lyra sign-off 2026-10-09 (VALUE-DOMAIN sign-off B/C;
    // LYRA-2026-10-09-d2-acceptance-loop-design-v1 §3).
    AcceptanceRequested,
    AcceptanceGranted,
    AcceptanceRejected,
    // (D2 follow-on, Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J) the withdrawal
    // half of the same object: a seat pulls back its OWN pending request —
    // "this card was satisfied by other means, do not merge it". Terminal and
    // immutable like accepted/rejected. The acceptance-side mirror of 013's
    // SeatDispatchCancelled (issuer-side withdrawal), so the ledger can express
    // a void instead of forcing a reject-with-rework or a hand-edited row.
    // Value-domain sign-off: REQUESTED from Lyra (the 016 scratch migration
    // carries the reasoning) — the Rust mirror lands with the dispatch, the
    // migration with Onyx.
    AcceptanceCancelled,
    // (015, D5 knowledge layer) the memory object's four transitions — each maps
    // to EXACTLY ONE, so the memories projection replays unambiguously
    // (contract §5). Lyra sign-off 2026-10-10
    // (VALUE-DOMAIN-2026-10-10-knowledge-layer-v1 §5).
    MemoryCreated,   // (none) -> active/cold; actor = the remembering seat
    MemoryRecalled,  // active: heat+1, recall_count+1, last_recalled_at=now
    MemoryPromoted,  // tier change (up OR down, explicit only)
    MemoryForgotten, // active -> forgotten (soft tombstone, reason required)
}

/// How an object relates to an event (`event_object_refs.role`) — `010:232-236`.
///
/// `Subject` is what the event is *about*, `Actor` is who did it, `Context` is
/// related without asserting agency or aboutness. `Context` is the weakest claim
/// and is what `010` backfilled legacy rows to; new code must state a role
/// explicitly (`010:220`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefRole {
    Subject,
    Actor,
    Context,
}

/// Object vocabulary for `event_object_refs.ref_type` — `010:270-276`.
///
/// **Contains `Delegation`, which the packet §3 contract table omits.** `010`
/// includes it in its CHECK because `delegation` exists in committed seed data
/// (2 rows, `ev-020`/`ev-021` → `del-flux-acting-mira-001`); a domain without it
/// could not read existing history. `010` flagged the gap to Lyra and landed with
/// it, accepted. Recorded here, and reported in the delivery doc, rather than
/// silently reconciled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefType {
    WorkItem,
    Session,
    Handoff,
    Artifact,
    Document,
    Seat,
    Topic,
    Project,
    Delegation,
    // (012, G19) `dispatches` is a table with its own object, exactly the
    // reasoning 010 used to admit `delegation`. Without it an event cannot name
    // the dispatch it is about. Lyra sign-off 2026-10-08; added under
    // LYRA-2026-10-08-p0-step6-unblock-v1 (7-A).
    Dispatch,
    // (014, D2 acceptance gate) `acceptances` is its own object — the subject
    // type of the acceptance events. Lyra sign-off 2026-10-09
    // (LYRA-2026-10-09-d2-acceptance-loop-design-v1 §3).
    Acceptance,
    // (015, D5 knowledge layer) `memories` is its own object: the four memory
    // events must be able to name the memory they are about. Same reasoning 010
    // used for 'delegation', 012 for 'dispatch', 014 for 'acceptance'. Lyra
    // sign-off 2026-10-10 (VALUE-DOMAIN-2026-10-10-knowledge-layer-v1 §6).
    Memory,
}

const EVENT_TYPE_EXPECTED: &str = "SessionStarted, WorkItemCreated, WorkItemStatusChanged, \
                                   ArtifactCreated, HandoffSent, HandoffAccepted, \
                                   HandoffCompleted, ReviewVerdictIssued, SeatDelegationIssued, \
                                   SeatDelegationClosed, SupervisorMessage, SeatResponse, \
                                   SeatDispatchIssued, SeatDispatchDelivered, SeatDispatchCompleted, \
                                   SeatDispatchFailed, SeatDispatchRetried, SeatDispatchCancelled, \
                                   AcceptanceRequested, AcceptanceGranted, AcceptanceRejected, AcceptanceCancelled, \
                                   MemoryCreated, MemoryRecalled, MemoryPromoted, MemoryForgotten";

const REF_ROLE_EXPECTED: &str = "subject, actor, context";

const REF_TYPE_EXPECTED: &str = "workitem, session, handoff, artifact, document, seat, topic, \
                                 project, delegation, dispatch, acceptance, memory";

impl EventType {
    /// Every admitted value, in `010` CHECK order. Used by the exhaustiveness
    /// test that pins this enum against the migration.
    pub const ALL: [EventType; 26] = [
        EventType::SessionStarted,
        EventType::WorkItemCreated,
        EventType::WorkItemStatusChanged,
        EventType::ArtifactCreated,
        EventType::HandoffSent,
        EventType::HandoffAccepted,
        EventType::HandoffCompleted,
        EventType::ReviewVerdictIssued,
        EventType::SeatDelegationIssued,
        EventType::SeatDelegationClosed,
        EventType::SupervisorMessage,
        EventType::SeatResponse,
        EventType::SeatDispatchIssued,
        EventType::SeatDispatchDelivered,
        EventType::SeatDispatchCompleted,
        EventType::SeatDispatchFailed,
        EventType::SeatDispatchRetried,
        EventType::SeatDispatchCancelled,
        EventType::AcceptanceRequested,
        EventType::AcceptanceGranted,
        EventType::AcceptanceRejected,
        EventType::AcceptanceCancelled,
        EventType::MemoryCreated,
        EventType::MemoryRecalled,
        EventType::MemoryPromoted,
        EventType::MemoryForgotten,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::SessionStarted => "SessionStarted",
            EventType::WorkItemCreated => "WorkItemCreated",
            EventType::WorkItemStatusChanged => "WorkItemStatusChanged",
            EventType::ArtifactCreated => "ArtifactCreated",
            EventType::HandoffSent => "HandoffSent",
            EventType::HandoffAccepted => "HandoffAccepted",
            EventType::HandoffCompleted => "HandoffCompleted",
            EventType::ReviewVerdictIssued => "ReviewVerdictIssued",
            EventType::SeatDelegationIssued => "SeatDelegationIssued",
            EventType::SeatDelegationClosed => "SeatDelegationClosed",
            EventType::SupervisorMessage => "SupervisorMessage",
            EventType::SeatResponse => "SeatResponse",
            EventType::SeatDispatchIssued => "SeatDispatchIssued",
            EventType::SeatDispatchDelivered => "SeatDispatchDelivered",
            EventType::SeatDispatchCompleted => "SeatDispatchCompleted",
            EventType::SeatDispatchFailed => "SeatDispatchFailed",
            EventType::SeatDispatchRetried => "SeatDispatchRetried",
            EventType::SeatDispatchCancelled => "SeatDispatchCancelled",
            EventType::AcceptanceRequested => "AcceptanceRequested",
            EventType::AcceptanceGranted => "AcceptanceGranted",
            EventType::AcceptanceRejected => "AcceptanceRejected",
            EventType::AcceptanceCancelled => "AcceptanceCancelled",
            EventType::MemoryCreated => "MemoryCreated",
            EventType::MemoryRecalled => "MemoryRecalled",
            EventType::MemoryPromoted => "MemoryPromoted",
            EventType::MemoryForgotten => "MemoryForgotten",
        }
    }
}

impl FromStr for EventType {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SessionStarted" => Ok(EventType::SessionStarted),
            "WorkItemCreated" => Ok(EventType::WorkItemCreated),
            "WorkItemStatusChanged" => Ok(EventType::WorkItemStatusChanged),
            "ArtifactCreated" => Ok(EventType::ArtifactCreated),
            "HandoffSent" => Ok(EventType::HandoffSent),
            "HandoffAccepted" => Ok(EventType::HandoffAccepted),
            "HandoffCompleted" => Ok(EventType::HandoffCompleted),
            "ReviewVerdictIssued" => Ok(EventType::ReviewVerdictIssued),
            "SeatDelegationIssued" => Ok(EventType::SeatDelegationIssued),
            "SeatDelegationClosed" => Ok(EventType::SeatDelegationClosed),
            "SupervisorMessage" => Ok(EventType::SupervisorMessage),
            "SeatResponse" => Ok(EventType::SeatResponse),
            "SeatDispatchIssued" => Ok(EventType::SeatDispatchIssued),
            "SeatDispatchDelivered" => Ok(EventType::SeatDispatchDelivered),
            "SeatDispatchCompleted" => Ok(EventType::SeatDispatchCompleted),
            "SeatDispatchFailed" => Ok(EventType::SeatDispatchFailed),
            "SeatDispatchRetried" => Ok(EventType::SeatDispatchRetried),
            "SeatDispatchCancelled" => Ok(EventType::SeatDispatchCancelled),
            "AcceptanceRequested" => Ok(EventType::AcceptanceRequested),
            "AcceptanceGranted" => Ok(EventType::AcceptanceGranted),
            "AcceptanceRejected" => Ok(EventType::AcceptanceRejected),
            "AcceptanceCancelled" => Ok(EventType::AcceptanceCancelled),
            "MemoryCreated" => Ok(EventType::MemoryCreated),
            "MemoryRecalled" => Ok(EventType::MemoryRecalled),
            "MemoryPromoted" => Ok(EventType::MemoryPromoted),
            "MemoryForgotten" => Ok(EventType::MemoryForgotten),
            _ => Err(ParseValueError {
                domain: "EventType",
                value: s.to_string(),
                expected: EVENT_TYPE_EXPECTED,
            }),
        }
    }
}

impl TryFrom<&str> for EventType {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl RefRole {
    pub const ALL: [RefRole; 3] = [RefRole::Subject, RefRole::Actor, RefRole::Context];

    pub fn as_str(&self) -> &'static str {
        match self {
            RefRole::Subject => "subject",
            RefRole::Actor => "actor",
            RefRole::Context => "context",
        }
    }
}

impl FromStr for RefRole {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "subject" => Ok(RefRole::Subject),
            "actor" => Ok(RefRole::Actor),
            "context" => Ok(RefRole::Context),
            _ => Err(ParseValueError {
                domain: "RefRole",
                value: s.to_string(),
                expected: REF_ROLE_EXPECTED,
            }),
        }
    }
}

impl TryFrom<&str> for RefRole {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for RefRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl RefType {
    pub const ALL: [RefType; 12] = [
        RefType::WorkItem,
        RefType::Session,
        RefType::Handoff,
        RefType::Artifact,
        RefType::Document,
        RefType::Seat,
        RefType::Topic,
        RefType::Project,
        RefType::Delegation,
        RefType::Dispatch,
        RefType::Acceptance,
        RefType::Memory,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            RefType::WorkItem => "workitem",
            RefType::Session => "session",
            RefType::Handoff => "handoff",
            RefType::Artifact => "artifact",
            RefType::Document => "document",
            RefType::Seat => "seat",
            RefType::Topic => "topic",
            RefType::Project => "project",
            RefType::Delegation => "delegation",
            RefType::Dispatch => "dispatch",
            RefType::Acceptance => "acceptance",
            RefType::Memory => "memory",
        }
    }
}

impl FromStr for RefType {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "workitem" => Ok(RefType::WorkItem),
            "session" => Ok(RefType::Session),
            "handoff" => Ok(RefType::Handoff),
            "artifact" => Ok(RefType::Artifact),
            "document" => Ok(RefType::Document),
            "seat" => Ok(RefType::Seat),
            "topic" => Ok(RefType::Topic),
            "project" => Ok(RefType::Project),
            "delegation" => Ok(RefType::Delegation),
            "dispatch" => Ok(RefType::Dispatch),
            "acceptance" => Ok(RefType::Acceptance),
            "memory" => Ok(RefType::Memory),
            _ => Err(ParseValueError {
                domain: "RefType",
                value: s.to_string(),
                expected: REF_TYPE_EXPECTED,
            }),
        }
    }
}

impl TryFrom<&str> for RefType {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{EventType, RefRole, RefType};
    use std::str::FromStr;

    /// Every variant must round-trip `Display` -> `FromStr` -> itself. This is
    /// the property a serde derive would silently break at the TEXT boundary.
    #[test]
    fn all_domains_round_trip() {
        for v in EventType::ALL {
            assert_eq!(EventType::from_str(v.as_str()).expect("round-trip"), v);
        }
        for v in RefRole::ALL {
            assert_eq!(RefRole::from_str(v.as_str()).expect("round-trip"), v);
        }
        for v in RefType::ALL {
            assert_eq!(RefType::from_str(v.as_str()).expect("round-trip"), v);
        }
    }

    /// Pins the serialized set against the migrations' CHECK constraints
    /// verbatim. `010` created the 12-value event_type and 9-value ref_type
    /// vocabularies; `012` widened them to 15 and 10 (governed dispatch, G19);
    /// `013`/`014` widen them to 21 and 11 (dispatch failure states + the D2
    /// acceptance gate); the D2 withdrawal follow-on (scratch 017, pending Onyx)
    /// adds `AcceptanceCancelled` -> 22; the D5 knowledge layer (`015`) adds the
    /// four memory transitions -> 26 and `memory` -> 12 ref types.
    ///
    /// NOTE ON THE COUNT: the D5 value-domain contract says "25 event values"
    /// because it counts from the LIVE database (21 values + 4). This branch
    /// also carries `AcceptanceCancelled`, whose migration (017) has not landed
    /// yet — so the honest Rust pin here is 26, and the database reaches 26 when
    /// 017 lands alongside 015. Declared, not silently reconciled.
    ///
    /// If a migration widens a domain, this test is where the Rust side must
    /// follow — kept verbatim, never loosened.
    #[test]
    fn serialized_sets_match_schema_010_012_013_014_015_017_checks() {
        let events: Vec<&str> = EventType::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            events,
            vec![
                "SessionStarted",
                "WorkItemCreated",
                "WorkItemStatusChanged",
                "ArtifactCreated",
                "HandoffSent",
                "HandoffAccepted",
                "HandoffCompleted",
                "ReviewVerdictIssued",
                "SeatDelegationIssued",
                "SeatDelegationClosed",
                "SupervisorMessage",
                "SeatResponse",
                // 012:73-75 (G19) canonical_events_event_type_vocab widening
                "SeatDispatchIssued",
                "SeatDispatchDelivered",
                "SeatDispatchCompleted",
                // 013 (G19 follow-on): terminal / retry / cancel transitions
                "SeatDispatchFailed",
                "SeatDispatchRetried",
                "SeatDispatchCancelled",
                // 014 (D2 acceptance gate): the acceptance object's transitions
                "AcceptanceRequested",
                "AcceptanceGranted",
                "AcceptanceRejected",
                // D2 withdrawal follow-on (017, scratch): the seat pulls back
                // its own pending request. Terminal, like accepted/rejected.
                "AcceptanceCancelled",
                // 015 (D5 knowledge layer): the memory object's transitions
                "MemoryCreated",
                "MemoryRecalled",
                "MemoryPromoted",
                "MemoryForgotten",
            ],
            "must equal 010:141-159 + 012 + 013 + 014 + 017 + 015's canonical_events_event_type_vocab"
        );
        let roles: Vec<&str> = RefRole::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(roles, vec!["subject", "actor", "context"]);
        let refs: Vec<&str> = RefType::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            refs,
            vec![
                "workitem",
                "session",
                "handoff",
                "artifact",
                "document",
                "seat",
                "topic",
                "project",
                "delegation",
                // 012 (G19) event_object_refs_ref_type_vocab widening
                "dispatch",
                // 014 (D2 acceptance gate)
                "acceptance",
                // 015 (D5 knowledge layer)
                "memory",
            ],
            "must equal 010:270-276 + 012 + 014 + 015's event_object_refs_ref_type_vocab"
        );
    }

    /// `ArtifactProduced` was dropped from `010` on Lyra's sign-off; the first
    /// slice reuses `ArtifactCreated`. Re-adding it needs the same sign-off, so
    /// this test fails loudly if someone does it silently.
    #[test]
    fn artifact_produced_is_not_in_the_vocabulary() {
        let err = EventType::from_str("ArtifactProduced")
            .expect_err("ArtifactProduced must not be admitted");
        assert_eq!(err.domain, "EventType");
        assert_eq!(err.value, "ArtifactProduced");
        assert!(EventType::from_str("ArtifactCreated").is_ok());
    }

    /// `013`/`014` widened the event vocabulary by six, the D2 withdrawal
    /// follow-on adds a seventh, and the D5 knowledge layer adds four more; this
    /// pins that all of them are admitted, round-trip, and are distinct (each
    /// names one transition), while an unknown type is still rejected.
    #[test]
    fn the_eleven_new_event_types_are_admitted_and_distinct() {
        use std::collections::HashSet;
        let new = [
            EventType::SeatDispatchFailed,
            EventType::SeatDispatchRetried,
            EventType::SeatDispatchCancelled,
            EventType::AcceptanceRequested,
            EventType::AcceptanceGranted,
            EventType::AcceptanceRejected,
            EventType::AcceptanceCancelled,
            EventType::MemoryCreated,
            EventType::MemoryRecalled,
            EventType::MemoryPromoted,
            EventType::MemoryForgotten,
        ];
        for v in new {
            assert_eq!(EventType::from_str(v.as_str()).expect("round-trip"), v);
            assert!(EventType::ALL.contains(&v), "{} must be in ALL", v.as_str());
        }
        // Distinct serialized forms (no two transitions share an event type).
        let names: HashSet<&str> = new.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            names.len(),
            11,
            "the eleven new event types must be distinct"
        );
        assert!(EventType::from_str("SeatDispatchReopened").is_err());
        assert!(EventType::from_str("MemoryUpdated").is_err());
        // A void is NOT a rejection: the two transitions stay separate words in
        // an append-only ledger, where one word for two meanings can never be
        // split once rows carry both.
        assert_ne!(
            EventType::AcceptanceRejected.to_string(),
            EventType::AcceptanceCancelled.to_string()
        );
        assert!(EventType::from_str("AcceptanceVoided").is_err());
        // ref_type widened: `acceptance` (014) and `memory` (015) are admitted.
        assert_eq!(
            RefType::from_str("acceptance").unwrap(),
            RefType::Acceptance
        );
        assert_eq!(RefType::from_str("memory").unwrap(), RefType::Memory);
    }

    /// Casing is significant in both directions, and deliberately inconsistent
    /// between the domains because the committed data is.
    #[test]
    fn casing_is_significant_and_domain_specific() {
        assert!(EventType::from_str("sessionstarted").is_err());
        assert!(EventType::from_str("session_started").is_err());
        assert!(RefRole::from_str("Subject").is_err());
        assert!(RefType::from_str("WorkItem").is_err());
    }

    /// Unknown values fail with a typed error naming the value, never a silent
    /// fallback — the same rule the runtime triple established.
    #[test]
    fn unknown_values_are_typed_errors() {
        let err = RefType::from_str("bogus").expect_err("must reject");
        assert_eq!(err.domain, "RefType");
        assert_eq!(err.value, "bogus");
        assert!(
            err.to_string().contains("\"bogus\""),
            "error must name the offending value: {err}"
        );
        assert_eq!(RefRole::from_str("").expect_err("must reject").value, "");
    }
}
