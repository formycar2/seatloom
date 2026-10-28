//! The gate/announcement criterion (Aegis ruling 2026-10-11, `disp-gate-vs-announcement-enforcement`).
//!
//! One event id used to carry two meanings, and a dependency edge pointed at
//! either without being able to tell them apart:
//!
//! - a **delivery announcement**: "my code is on branch X at commit Y". True,
//!   useful, and the seat that did the work is entitled to emit it.
//! - an **availability gate**: "this capability really runs, and here is the
//!   measurement that proves it". Only true if somebody MEASURED it.
//!
//! A 013 trigger that releases `blocked -> ready` on referent existence cannot
//! tell the two apart, so an edge hung on an announcement was satisfied from the
//! moment it was hung — and `disp-onight-writein` was released onto a capability
//! that had never once succeeded. The loss was measurable: one retry budget, one
//! blocked delivery note, one idle round.
//!
//! This module is the ONE place the criterion lives, so the producer side
//! (`emit_event`), the edge side (`enqueue_dispatch`) and the release side
//! (`dep_satisfied`) cannot drift apart. All three must agree; a criterion that
//! lives in one of them is a criterion that will be wrong in the others.
//!
//! The criterion itself is Aegis's, already proven in `scripts/land-knowledge-live.sh`
//! (`gate_landed()`), and it is a PAYLOAD test, not an id test:
//!
//! ```sql
//! SELECT event_type, payload->>'gate' FROM canonical_events WHERE id = $1
//! -- a gate iff event_type = 'SupervisorMessage' AND payload->>'gate' = $1
//! ```
//!
//! plus the fields that make the assertion auditable: `statement` (what is
//! claimed), `verified` (how it was measured — what was called, what came back,
//! on which database) and `does_not_assert` (what the gate explicitly does NOT
//! claim, so a reader cannot over-read it).

use serde_json::Value;

/// The suffix that marks an event id as a GATE rather than an announcement
/// (Aegis ruling 2). It is the discriminator the edge side uses, because it is
/// the only thing knowable at edge-attachment time — the event itself does not
/// exist yet, which is the whole design tension this ruling had to resolve.
pub const GATE_ID_SUFFIX: &str = "-live";

/// The event type a gate must carry. `SupervisorMessage` is the type that
/// already means "a statement from a seat about the state of the system";
/// `ArtifactCreated` — the announcement type — is deliberately excluded.
pub const GATE_EVENT_TYPE: &str = "SupervisorMessage";

/// The payload key that names the gate. It must equal the event's own id, so a
/// row cannot be re-pointed at a different gate by editing the payload.
pub const GATE_PAYLOAD_KEY: &str = "gate";

/// The payload key that records HOW the claim was measured.
pub const VERIFIED_PAYLOAD_KEY: &str = "verified";

/// The payload key that records WHAT is claimed.
pub const STATEMENT_PAYLOAD_KEY: &str = "statement";

/// The payload key that records what the gate explicitly does NOT claim.
pub const DOES_NOT_ASSERT_PAYLOAD_KEY: &str = "does_not_assert";

/// Is this event id shaped like a gate id? A gate id carries `live` as a
/// segment (`ev-live-019-applied`) or ends with it (`ev-knowledge-read-model-live`).
///
/// BOTH shapes are matched, and that is a finding rather than a preference:
/// ruling 2 says new gate ids are uniformly `ev-<capability>-live`, but the
/// first gate actually emitted under it — `ev-live-019-applied`, Aegis's own —
/// ends in `-applied`. A suffix-only test would have read Aegis's real, fully
/// verified gate as an announcement. So the test is deliberately the looser
/// segment test, and the plan records the naming inconsistency for Aegis to
/// settle.
///
/// This is a SHAPE test only — it says "the issuer meant this as a gate", never
/// "this event is a gate". Whether it actually is one is [`gate_payload_shape`],
/// applied to the row that eventually lands.
pub fn is_gate_id(event_id: &str) -> bool {
    event_id.ends_with(GATE_ID_SUFFIX) || event_id.contains(&format!("{GATE_ID_SUFFIX}-"))
}

/// A non-empty string payload field. Absent, null, empty and whitespace-only all
/// read as "not stated" — a gate that claims to be verified by an empty string
/// is not verified by anything.
fn stated(payload: &Value, key: &str) -> bool {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Does this `(type, event_id, payload)` triple have the shape of a gate?
///
/// This is the PRODUCER-side test: what `emit_event` enforces before it will
/// mint or accept an id that claims to be a gate. It is deliberately stricter
/// than existence — a gate must say what it claims, how that was measured, and
/// what it does not claim.
pub fn gate_payload_shape(event_type: &str, event_id: &str, payload: &Value) -> bool {
    if event_type != GATE_EVENT_TYPE {
        return false;
    }
    // payload.gate must equal the event's own id.
    if payload.get(GATE_PAYLOAD_KEY).and_then(Value::as_str) != Some(event_id) {
        return false;
    }
    stated(payload, STATEMENT_PAYLOAD_KEY)
        && stated(payload, VERIFIED_PAYLOAD_KEY)
        && stated(payload, DOES_NOT_ASSERT_PAYLOAD_KEY)
}

/// Does an EXISTING row qualify as a gate? Same test as [`gate_payload_shape`],
/// named separately because the two call sites fail differently: the producer
/// refuses to write, the release side refuses to release.
pub fn row_is_gate(event_type: &str, event_id: &str, payload: Option<&Value>) -> bool {
    match payload {
        Some(p) => gate_payload_shape(event_type, event_id, p),
        None => false,
    }
}

/// Why a gate-shaped emission was refused. Rendered verbatim into the MCP tool
/// error so the issuer can fix the call rather than guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateRejection {
    /// The id claims to be a gate (`-live`) but the type is not `SupervisorMessage`.
    WrongTypeForGateId {
        event_id: String,
        event_type: String,
    },
    /// The id claims to be a gate but the payload does not name it back.
    PayloadDoesNotNameGate { event_id: String },
    /// The id claims to be a gate but says nothing about what is claimed.
    MissingStatement { event_id: String },
    /// The id claims to be a gate but does not say how the claim was measured.
    /// This is the one that would have stopped the 10-10 loss: "branch X, commit Y"
    /// is not a measurement.
    MissingVerified { event_id: String },
    /// The id claims to be a gate but does not bound what it claims.
    MissingDoesNotAssert { event_id: String },
    /// An edge names an event id that is shaped like a gate, but the issuer did
    /// not say so. Refused rather than guessed: the two edge intents release
    /// differently, and guessing here is how an announcement ends up gating a
    /// capability.
    GateEdgeUndeclared { dep_value: String },
}

impl std::fmt::Display for GateRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateRejection::WrongTypeForGateId { event_id, event_type } => write!(
                f,
                "gate rejected: event id {event_id:?} claims to be a gate (it ends in {GATE_ID_SUFFIX:?}), \
                 so it must be emitted as {GATE_EVENT_TYPE:?}, not {event_type:?}. \
                 An ArtifactCreated is a delivery announcement; it says the code is somewhere, \
                 never that the capability runs."
            ),
            GateRejection::PayloadDoesNotNameGate { event_id } => write!(
                f,
                "gate rejected: event id {event_id:?} claims to be a gate, but its payload has no \
                 {GATE_PAYLOAD_KEY:?} equal to the event's own id. Without it the row cannot be \
                 re-checked later, and a same-id row of another kind would satisfy the edge."
            ),
            GateRejection::MissingStatement { event_id } => write!(
                f,
                "gate rejected: event id {event_id:?} claims to be a gate but states nothing. \
                 Add {STATEMENT_PAYLOAD_KEY:?}: the sentence this gate asserts."
            ),
            GateRejection::MissingVerified { event_id } => write!(
                f,
                "gate rejected: event id {event_id:?} claims to be a gate but says nothing about \
                 how it was measured. Add {VERIFIED_PAYLOAD_KEY:?}: what was called, what came back, \
                 on which database. 'branch X, commit Y' is a delivery announcement, not a measurement."
            ),
            GateRejection::MissingDoesNotAssert { event_id } => write!(
                f,
                "gate rejected: event id {event_id:?} claims to be a gate but does not bound itself. \
                 Add {DOES_NOT_ASSERT_PAYLOAD_KEY:?}: what this gate deliberately does NOT claim, \
                 so a reader cannot over-read it."
            ),
            GateRejection::GateEdgeUndeclared { dep_value } => write!(
                f,
                "dependency rejected: {dep_value:?} is shaped like a gate id (it ends in {GATE_ID_SUFFIX:?}), \
                 but this edge does not declare that it requires a verified gate. A gate edge and an \
                 existence edge release differently: set `requires_gate: true` to require the referent to be \
                 a measured gate, or name an announcement id (`-landed`) if all you need is for the code to exist."
            ),
        }
    }
}

/// Validate an emission that claims to be a gate. `Ok(())` means the shape is
/// acceptable; the error names the missing field.
///
/// An id that does NOT claim to be a gate is always fine here — announcements
/// are legitimate and are not this function's business.
pub fn check_gate_emission(
    event_type: &str,
    event_id: &str,
    payload: &Value,
) -> Result<(), GateRejection> {
    if !is_gate_id(event_id) {
        return Ok(());
    }
    if event_type != GATE_EVENT_TYPE {
        return Err(GateRejection::WrongTypeForGateId {
            event_id: event_id.to_string(),
            event_type: event_type.to_string(),
        });
    }
    if payload.get(GATE_PAYLOAD_KEY).and_then(Value::as_str) != Some(event_id) {
        return Err(GateRejection::PayloadDoesNotNameGate {
            event_id: event_id.to_string(),
        });
    }
    if !stated(payload, STATEMENT_PAYLOAD_KEY) {
        return Err(GateRejection::MissingStatement {
            event_id: event_id.to_string(),
        });
    }
    if !stated(payload, VERIFIED_PAYLOAD_KEY) {
        return Err(GateRejection::MissingVerified {
            event_id: event_id.to_string(),
        });
    }
    if !stated(payload, DOES_NOT_ASSERT_PAYLOAD_KEY) {
        return Err(GateRejection::MissingDoesNotAssert {
            event_id: event_id.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn gate_payload(id: &str) -> Value {
        json!({
            "gate": id,
            "statement": "Migration 019 is live-applied: the running database admits TopicOpened.",
            "verified": "select pg_get_constraintdef(oid) ... on the live DB; returned a list containing TopicOpened",
            "does_not_assert": "does not assert the seats have reloaded their tool vocabulary",
            "issued_by": "seat-aegis-001",
        })
    }

    #[test]
    fn an_id_ending_in_live_claims_to_be_a_gate_and_nothing_else_does() {
        assert!(is_gate_id("ev-live-019-applied"));
        assert!(is_gate_id("ev-knowledge-read-model-live"));
        assert!(is_gate_id("ev-knowledge-write-path-live"));
        // The finding: ruling 2 says gates end in -live, but Aegis's own first
        // gate under that ruling ends in -applied. A suffix-only test would
        // have read it as an announcement.
        assert!(!is_gate_id("ev-019-topic-opened-landed"));
        assert!(!is_gate_id("ev-typed-memory-tools-landed"));
        assert!(!is_gate_id("ev-writein-landed"));
        assert!(!is_gate_id("ev-anything-else"));
    }

    #[test]
    fn a_well_formed_gate_payload_passes_and_an_announcement_payload_does_not() {
        let id = "ev-live-019-applied";
        assert!(gate_payload_shape(
            "SupervisorMessage",
            id,
            &gate_payload(id)
        ));

        // The exact payload that fooled the fleet on 10-10: branch/commit only.
        let announcement =
            json!({"branch": "packet/onyx/019", "commit": "abc123", "migration": "019"});
        assert!(!gate_payload_shape("ArtifactCreated", id, &announcement));
    }

    #[test]
    fn a_gate_id_emitted_as_an_announcement_type_is_refused() {
        let id = "ev-knowledge-read-model-live";
        let err = check_gate_emission("ArtifactCreated", id, &gate_payload(id)).unwrap_err();
        assert!(matches!(err, GateRejection::WrongTypeForGateId { .. }));
        assert!(err.to_string().contains("delivery announcement"));
    }

    #[test]
    fn a_gate_whose_payload_names_a_different_id_is_refused() {
        let id = "ev-knowledge-read-model-live";
        let mut p = gate_payload(id);
        p["gate"] = json!("ev-some-other-gate-live");
        let err = check_gate_emission("SupervisorMessage", id, &p).unwrap_err();
        assert!(matches!(err, GateRejection::PayloadDoesNotNameGate { .. }));
    }

    #[test]
    fn aegis_own_first_gate_passes_the_producer_check() {
        // The real shape, emitted by scripts/land-knowledge-live.sh stage 5.
        let id = "ev-live-019-applied";
        assert!(check_gate_emission("SupervisorMessage", id, &gate_payload(id)).is_ok());
    }

    #[test]
    fn each_of_the_three_auditable_fields_is_required() {
        let id = "ev-knowledge-read-model-live";
        for key in [
            STATEMENT_PAYLOAD_KEY,
            VERIFIED_PAYLOAD_KEY,
            DOES_NOT_ASSERT_PAYLOAD_KEY,
        ] {
            let mut p = gate_payload(id);
            p[key] = json!("   ");
            let err = check_gate_emission("SupervisorMessage", id, &p).unwrap_err();
            assert!(
                !matches!(err, GateRejection::WrongTypeForGateId { .. }),
                "{key}"
            );
        }
    }

    #[test]
    fn the_verified_field_is_the_one_that_separates_a_gate_from_an_announcement() {
        let id = "ev-knowledge-read-model-live";
        let mut p = gate_payload(id);
        // "branch X, commit Y" is the announcement's whole vocabulary.
        p[VERIFIED_PAYLOAD_KEY] = json!("branch packet/onyx/019, commit abc123");
        // It is a non-empty string, so it passes the shape test...
        assert!(check_gate_emission("SupervisorMessage", id, &p).is_ok());
        // ...which is why the field is REQUIRED rather than merely present: the
        // criterion cannot tell a real measurement from a commit hash, and it
        // does not try to. It only refuses a gate that declines to say anything.
    }

    #[test]
    fn an_announcement_id_is_none_of_this_functions_business() {
        let id = "ev-019-topic-opened-landed";
        let p = json!({"branch": "packet/onyx/019", "migration": "019"});
        assert!(check_gate_emission("ArtifactCreated", id, &p).is_ok());
    }

    #[test]
    fn a_missing_payload_is_never_a_gate() {
        assert!(!row_is_gate("SupervisorMessage", "ev-x-live", None));
    }

    // ---- the one place Rust and SQL could still silently drift ----
    //
    // Onyx's answer to question 1 of the ruling was that NEITHER `coalesce(x,'')
    // <> ''` nor a bare `btrim(x)` is byte-identical to Rust's `stated()`: the
    // first does not trim at all, and the second trims SPACES only. The
    // dangerous direction is a SUBSET — the database would release a dispatch
    // the seat then refuses to pull, which is the silent hang the ruling exists
    // to prevent. Onyx therefore enumerated Rust's whole whitespace set in the
    // SQL (`infra/postgres/schema/020_gate_criterion.sql`, `fn_gate_row_is_gate`).
    //
    // That makes the two implementations agree TODAY. It does not keep them
    // agreeing: `str::trim` is defined by `char::is_whitespace`, which is the
    // Unicode White_Space property, and anybody who "simplifies" `stated()` to
    // `trim_matches(' ')` — or a future Rust that changes the property — would
    // open exactly the dangerous direction with no test failing. This test pins
    // the set, so the divergence becomes a compile-time-visible failure here
    // instead of a production release/pull disagreement.

    /// The Unicode White_Space property, enumerated — the same list 020 ships in
    /// SQL as `chr(9) || chr(10) || ... || chr(12288)`.
    const UNICODE_WHITE_SPACE: [char; 25] = [
        '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', // U+0009-U+000D
        '\u{0020}', // SPACE
        '\u{0085}', // NEL
        '\u{00A0}', // NO-BREAK SPACE
        '\u{1680}', // OGHAM SPACE MARK
        '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', // U+2000-U+2004
        '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', // U+2005-U+2009
        '\u{200A}', // U+200A
        '\u{2028}', // LINE SEPARATOR
        '\u{2029}', // PARAGRAPH SEPARATOR
        '\u{202F}', // NARROW NO-BREAK SPACE
        '\u{205F}', // MEDIUM MATHEMATICAL SPACE
        '\u{3000}', // IDEOGRAPHIC SPACE
    ];

    /// Characters that LOOK like whitespace and are NOT in the property. If
    /// `stated()` ever trims one of these, Rust has become stricter than the
    /// SQL and a gate the database accepts would be refused by the seat — the
    /// opposite direction, still a divergence, still a hung dispatch.
    const WHITESPACE_IMPOSTORS: [char; 4] = [
        '\u{200B}', // ZERO WIDTH SPACE — Cf, not White_Space
        '\u{180E}', // MONGOLIAN VOWEL SEPARATOR — Cf since Unicode 6.3
        '\u{FEFF}', // ZERO WIDTH NO-BREAK SPACE / BOM — Cf
        '\u{00B7}', // MIDDLE DOT
    ];

    #[test]
    fn stated_trims_exactly_the_unicode_white_space_set_the_sql_enumerates() {
        let id = "ev-knowledge-read-model-live";
        // Every White_Space char, alone and surrounding real text, reads as
        // "not stated" / "stated" respectively — the same answer the SQL's
        // btrim(x, ws) gives.
        for ws in UNICODE_WHITE_SPACE {
            let blank: String = std::iter::repeat_n(ws, 3).collect();
            let mut p = gate_payload(id);
            p[VERIFIED_PAYLOAD_KEY] = serde_json::json!(blank);
            assert!(
                !gate_payload_shape("SupervisorMessage", id, &p),
                "U+{:04X} must read as 'not stated' — the SQL trims it, so Rust must too",
                ws as u32
            );

            let padded = format!("{ws}measured{ws}");
            let mut p = gate_payload(id);
            p[VERIFIED_PAYLOAD_KEY] = serde_json::json!(padded);
            assert!(
                gate_payload_shape("SupervisorMessage", id, &p),
                "U+{:04X} around real text must still read as 'stated'",
                ws as u32
            );
        }
    }

    #[test]
    fn stated_does_not_trim_characters_outside_the_set() {
        // The opposite direction: if Rust ever trims more than the SQL, a gate
        // the database releases would be refused by the seat.
        let id = "ev-knowledge-read-model-live";
        for c in WHITESPACE_IMPOSTORS {
            let mut p = gate_payload(id);
            p[VERIFIED_PAYLOAD_KEY] = serde_json::json!(format!("{c}measured"));
            assert!(
                gate_payload_shape("SupervisorMessage", id, &p),
                "U+{:04X} is not White_Space; trimming it would make Rust stricter than the SQL",
                c as u32
            );
        }
    }

    #[test]
    fn the_id_shape_test_matches_the_sqls_like_segment_test() {
        // 020's fn_gate_is_gate_id is `event_id LIKE '%-live-%' OR LIKE '%-live'`.
        // Both are case-sensitive substring/suffix tests, so the encodings are
        // equivalent — but Onyx proved that over 801 live ids plus 30
        // adversarial ones, and a future change to either side deserves the
        // same proof rather than my say-so. These are the shapes that actually
        // differ between a naive suffix rule and the segment rule.
        assert!(is_gate_id("ev-live-019-applied")); // the finding: not a suffix
        assert!(is_gate_id("ev-knowledge-read-model-live")); // a suffix
        assert!(is_gate_id("ev-x-live-y")); // a middle segment
        assert!(!is_gate_id("ev-delivery")); // no segment at all
        assert!(!is_gate_id("ev-lively-thing")); // '-live' must be its own segment
        assert!(!is_gate_id("ev-LIVE-019-applied")); // LIKE is case-sensitive
    }
}
