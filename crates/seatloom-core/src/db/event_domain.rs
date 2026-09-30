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
/// `010`'s `canonical_events_event_type_vocab` CHECK (`010:141-159`).
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
}

const EVENT_TYPE_EXPECTED: &str = "SessionStarted, WorkItemCreated, WorkItemStatusChanged, \
                                   ArtifactCreated, HandoffSent, HandoffAccepted, \
                                   HandoffCompleted, ReviewVerdictIssued, SeatDelegationIssued, \
                                   SeatDelegationClosed, SupervisorMessage, SeatResponse";

const REF_ROLE_EXPECTED: &str = "subject, actor, context";

const REF_TYPE_EXPECTED: &str = "workitem, session, handoff, artifact, document, seat, topic, \
                                 project, delegation";

impl EventType {
    /// Every admitted value, in `010` CHECK order. Used by the exhaustiveness
    /// test that pins this enum against the migration.
    pub const ALL: [EventType; 12] = [
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
    pub const ALL: [RefType; 9] = [
        RefType::WorkItem,
        RefType::Session,
        RefType::Handoff,
        RefType::Artifact,
        RefType::Document,
        RefType::Seat,
        RefType::Topic,
        RefType::Project,
        RefType::Delegation,
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

    /// Pins the serialized set against `010`'s CHECK constraints verbatim. If a
    /// migration widens a domain, this test is where the Rust side must follow.
    #[test]
    fn serialized_sets_match_schema_010_checks() {
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
            ],
            "must equal 010:141-159 canonical_events_event_type_vocab"
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
            ],
            "must equal 010:270-276 event_object_refs_ref_type_vocab"
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
