use crate::objects::id::{CheckpointId, SeatId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub seat_id: SeatId,
    /// Process wrapper the agent runs under. `None` means *not recorded* —
    /// never "direct". See `Harness`.
    pub harness: Option<Harness>,
    pub agent: Agent,
    /// Model the agent was pinned to. `None` means *not recorded*.
    pub model: Option<ModelId>,
    pub native_session_id: Option<String>,
    pub workspace_path: PathBuf,
    pub branch: Option<String>,
    pub status: SessionStatus,
    /// Path relative to .seatloom/ for the Worker continuity pack.
    pub launch_pack_ref: Option<String>,
    pub last_checkpoint_id: Option<CheckpointId>,
    pub pid: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Runtime triple: harness / agent / model
//
// These three replace the former single `Runtime` enum, which collapsed three
// orthogonal dimensions into one field (Aegis review 2026-09-28 §4).
//
// Dimension placement is load-bearing and must not be re-flattened:
//   - `stepcode` / `stepgo` is a HARNESS, not an agent.
//   - `opencode` is an AGENT, not a harness.
//
// Variant names serialize verbatim in PascalCase. Do NOT add
// `#[serde(rename_all = ...)]` here: the existing `sessions.runtime` values in
// PostgreSQL are already PascalCase, and PostgreSQL is canonical (AD-011).
// Note the deliberate asymmetry with `SeatRole` (snake_case) in
// `objects/seat.rs` — both follow "match what is already in PostgreSQL"; the
// existing data is inconsistent. Do not harmonize them.
// =============================================================================

/// Process wrapper / credential / proxy layer the agent runs under.
///
/// `Option<Harness>` is used at every call site: `None` means *the harness was
/// not recorded at the time*, never *there was no harness*. Do not default it
/// to `Direct` — a fabricated value is indistinguishable from a real one.
///
/// # `Pi` is here on the operator's ruling, and the ruling has a cost
///
/// Mr Zhang ruled on 2026-10-17 that every seat's `default_harness` becomes `Pi`.
/// He knew the cost and chose anyway: read strictly, `pi` is an *agent* (the CLI
/// being driven) rather than a harness (the wrapper it runs under), and the eight
/// seats actually run under the `stepcode` proxy — which is what six of the rows
/// said. After this change the `seats` table no longer records *which wrapper
/// layer a seat runs under*; the only proxy clue left is in `default_model`
/// (`modelproxy-anthropic/…`, `stepplan-anthropic/…`). His reading also holds —
/// `pi` is itself a wrapper, and the proxy detail is carried elsewhere — and the
/// vocabulary boundary is his to draw. The variant is here because he ruled it,
/// and the cost is recorded rather than written up as a pure improvement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Harness {
    Direct,
    Stepcode,
    /// `pi` — the operator's 2026-10-17 ruling. See the note above for the cost.
    Pi,
    Custom(String),
}

/// The CLI or protocol being driven.
///
/// `Pi` is here because the whole SeatLoom fleet runs it — every one of the eight
/// seats is a `pi` process. An enum describing "the CLI or protocol being driven"
/// that omitted the one the fleet actually runs was not describing the fleet; it
/// was describing a fleet that did not exist. `Custom(String)` is an escape hatch
/// for a one-off, not a reason to keep the primary agent out of the domain.
///
/// Serialized verbatim in PascalCase, like every other variant here — PostgreSQL
/// is canonical (AD-011), and the live `seats` rows already spell it `Pi`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    ClaudeCode,
    Codex,
    CursorCli,
    GeminiCli,
    Opencode,
    /// `pi` — the agent CLI the SeatLoom seats run.
    Pi,
    Custom(String),
}

/// Opaque model identifier, e.g. `claude-opus-5[1m]`.
///
/// Intentionally a `String` newtype and not an enum: model ids churn faster
/// than releases, so an enum would need a migration per model. This type
/// exists for type-safety at call sites only — it carries no model-specific
/// behavior and no capability table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ModelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A string crossing the PostgreSQL ⇄ Rust boundary was not inside its declared
/// value domain.
///
/// Review §5 rule: fail loudly on an unknown value rather than silently
/// absorbing it into `Custom(...)`. `Custom` is reachable only via its explicit
/// string form, never as a fallback for garbage.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{value:?} is not a valid {domain} value (expected one of: {expected})")]
pub struct ParseValueError {
    pub domain: &'static str,
    pub value: String,
    pub expected: &'static str,
}

/// String form of the `Custom(String)` escape hatch.
///
/// A bare tag (`"Custom"`) parses to an empty payload — that is the shape
/// already present in `sessions.runtime` for `ses-flux-verify-001` at
/// `infra/postgres/seed/001_real_collaboration_baseline.sql:67`. A payload is
/// carried as `"Custom(<payload>)"`, matching how the §2.1 contract table
/// writes the domain.
pub(crate) fn parse_custom_tagged(s: &str, tag: &str) -> Option<String> {
    if s == tag {
        return Some(String::new());
    }
    s.strip_prefix(tag)?
        .strip_prefix('(')?
        .strip_suffix(')')
        .map(str::to_string)
}

/// Inverse of [`parse_custom_tagged`].
pub(crate) fn render_custom_tagged(tag: &str, payload: &str) -> String {
    if payload.is_empty() {
        tag.to_string()
    } else {
        format!("{tag}({payload})")
    }
}

const HARNESS_EXPECTED: &str = "Direct, Stepcode, Pi, Custom, Custom(<payload>)";

impl FromStr for Harness {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Direct" => Ok(Self::Direct),
            "Stepcode" => Ok(Self::Stepcode),
            "Pi" => Ok(Self::Pi),
            _ => parse_custom_tagged(s, "Custom")
                .map(Self::Custom)
                .ok_or_else(|| ParseValueError {
                    domain: "Harness",
                    value: s.to_string(),
                    expected: HARNESS_EXPECTED,
                }),
        }
    }
}

impl TryFrom<&str> for Harness {
    type Error = ParseValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for Harness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct => f.write_str("Direct"),
            Self::Stepcode => f.write_str("Stepcode"),
            Self::Pi => f.write_str("Pi"),
            Self::Custom(payload) => f.write_str(&render_custom_tagged("Custom", payload)),
        }
    }
}

const AGENT_EXPECTED: &str =
    "ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Pi, Custom, Custom(<payload>)";

impl FromStr for Agent {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ClaudeCode" => Ok(Self::ClaudeCode),
            "Codex" => Ok(Self::Codex),
            "CursorCli" => Ok(Self::CursorCli),
            "GeminiCli" => Ok(Self::GeminiCli),
            "Opencode" => Ok(Self::Opencode),
            "Pi" => Ok(Self::Pi),
            _ => parse_custom_tagged(s, "Custom")
                .map(Self::Custom)
                .ok_or_else(|| ParseValueError {
                    domain: "Agent",
                    value: s.to_string(),
                    expected: AGENT_EXPECTED,
                }),
        }
    }
}

impl TryFrom<&str> for Agent {
    type Error = ParseValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClaudeCode => f.write_str("ClaudeCode"),
            Self::Codex => f.write_str("Codex"),
            Self::CursorCli => f.write_str("CursorCli"),
            Self::GeminiCli => f.write_str("GeminiCli"),
            Self::Opencode => f.write_str("Opencode"),
            Self::Pi => f.write_str("Pi"),
            Self::Custom(payload) => f.write_str(&render_custom_tagged("Custom", payload)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Launching,
    Running,
    /// Wrapped runtime is blocked on interactive stdin (AD-012).
    /// Distinct from InputRequired (SeatLoom-level request).
    PromptBlocked,
    InputRequired,
    Suspended,
    Completed,
    Failed,
    Interrupted,
}

/// Active prompt state when SessionStatus == PromptBlocked (AD-012).
/// bounded_window is bounded to last 10-20 terminal lines — never full session history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptState {
    pub kind: PromptKind,
    pub policy: PromptPolicy,
    pub bounded_window: Vec<String>,
    pub available_actions: Vec<PromptAction>,
    pub assist_budget: Option<AssistBudget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptKind {
    Deterministic,
    WizardMenu,
    Freeform,
    /// Password / OTP / sudo / secret — SupervisorAssist is disabled at architecture level.
    Sensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptPolicy {
    /// SeatLoom may inject without asking (Deterministic + config permit).
    AutoAllowed,
    /// Requires explicit user confirmation before inject.
    NeedsApproval,
    /// Human must type directly; no automation permitted.
    HumanRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptAction {
    Approve,
    HumanTakeover,
    /// Disabled when PromptKind == Sensitive.
    SupervisorAssist,
    Stop,
}

/// Hard caps on Supervisor assist loops (AD-012). Exceed either limit → force HumanTakeover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistBudget {
    pub max_steps: u32,
    pub max_tokens: u32,
    pub steps_used: u32,
    pub tokens_used: u32,
}

#[cfg(test)]
mod tests {
    use super::{Agent, Harness, ModelId, ParseValueError};

    #[test]
    fn harness_domain_round_trips() {
        for (s, v) in [
            ("Direct", Harness::Direct),
            ("Stepcode", Harness::Stepcode),
            // The operator's 2026-10-17 ruling: every seat's default_harness is
            // `Pi`. The live rows spell it that way, so the domain must admit it.
            ("Pi", Harness::Pi),
            ("Custom", Harness::Custom(String::new())),
            ("Custom(podman-exec)", Harness::Custom("podman-exec".into())),
        ] {
            assert_eq!(Harness::try_from(s).expect("in-domain"), v);
            assert_eq!(v.to_string(), s, "Display must invert FromStr");
        }
    }

    #[test]
    fn agent_domain_round_trips() {
        for (s, v) in [
            ("ClaudeCode", Agent::ClaudeCode),
            ("Codex", Agent::Codex),
            ("CursorCli", Agent::CursorCli),
            ("GeminiCli", Agent::GeminiCli),
            ("Opencode", Agent::Opencode),
            // The fleet's own agent: every seat is a pi process, and the live
            // seats rows for seat-ql-001 / seat-vela-001 already spell it `Pi`.
            ("Pi", Agent::Pi),
            // The bare tag is the shape already in sessions.runtime for
            // ses-flux-verify-001 (seed/001:67 at base commit 9599136).
            ("Custom", Agent::Custom(String::new())),
            (
                "Custom(OpenCode-DSV4Pro)",
                Agent::Custom("OpenCode-DSV4Pro".into()),
            ),
        ] {
            assert_eq!(Agent::try_from(s).expect("in-domain"), v);
            assert_eq!(v.to_string(), s, "Display must invert FromStr");
        }
    }

    /// Review §5: an unknown value fails loudly. It must NOT be absorbed into
    /// `Custom(...)` and must not panic.
    #[test]
    fn unknown_agent_is_a_typed_error_not_a_custom_fallback() {
        let err = Agent::try_from("Bogus").expect_err("must reject out-of-domain value");
        assert_eq!(
            err,
            ParseValueError {
                domain: "Agent",
                value: "Bogus".to_string(),
                expected: super::AGENT_EXPECTED,
            }
        );
        assert!(
            err.to_string().contains("\"Bogus\""),
            "error must name the offending value: {err}"
        );
    }

    #[test]
    fn unknown_harness_is_a_typed_error() {
        let err = Harness::try_from("stepcode").expect_err("casing is significant");
        assert_eq!(err.domain, "Harness");
        assert_eq!(err.value, "stepcode");
    }

    /// The Harness error names the whole domain including `Pi`, so a reader who
    /// hits an unknown value sees the complete set rather than a partial one.
    #[test]
    fn the_harness_error_names_the_whole_domain_including_pi() {
        let err = Harness::try_from("Bogus").expect_err("must reject");
        assert!(err.expected.contains("Pi"), "{}", err.expected);
        assert_eq!(
            err.expected,
            "Direct, Stepcode, Pi, Custom, Custom(<payload>)"
        );
        // And an unknown value is still refused, never absorbed into Custom.
        assert!(Harness::try_from("pi").is_err(), "casing is significant");
    }

    /// `Stepcode` is a harness and `Opencode` is an agent. If this test is ever
    /// changed to move either dimension, re-read Aegis review 2026-09-28 §4.
    #[test]
    fn dimensions_are_not_interchangeable() {
        assert!(Agent::try_from("Stepcode").is_err());
        assert!(Harness::try_from("Opencode").is_err());
    }

    #[test]
    fn model_id_is_opaque_free_form_text() {
        let m = ModelId::from("claude-opus-5[1m]");
        assert_eq!(m.as_str(), "claude-opus-5[1m]");
        assert_eq!(
            serde_json::to_string(&m).expect("newtype serializes transparently"),
            "\"claude-opus-5[1m]\""
        );
    }

    /// `Pi` serializes verbatim as `"Pi"` on BOTH dimensions — the spelling
    /// already in the live `seats` rows. No `rename_all`, no invented spelling:
    /// PostgreSQL is canonical (AD-011), so the Rust side matches what is already
    /// stored rather than the other way round. This is the operator's 2026-10-17
    /// ruling applied to `Harness` as well as `Agent`.
    #[test]
    fn pi_serializes_verbatim_on_both_dimensions() {
        assert_eq!(Harness::Pi.to_string(), "Pi");
        assert_eq!(
            serde_json::to_string(&Harness::Pi).expect("serialize"),
            "\"Pi\""
        );
        assert_eq!(Agent::Pi.to_string(), "Pi");
        // A distinct value on each dimension, not an alias of anything else.
        assert_ne!(Harness::Pi, Harness::Custom(String::new()));
        assert_ne!(Harness::Pi, Harness::Stepcode);
        assert_ne!(Harness::Pi, Harness::Direct);
    }

    /// `Pi` serializes verbatim as `"Pi"` — the spelling already in the live
    /// `seats` rows for `seat-ql-001` and `seat-vela-001`. No `rename_all`, no
    /// invented spelling: PostgreSQL is canonical (AD-011), so the Rust side
    /// matches what is already stored rather than the other way round.
    #[test]
    fn pi_serializes_verbatim_as_the_database_spells_it() {
        assert_eq!(Agent::Pi.to_string(), "Pi");
        assert_eq!(
            serde_json::to_string(&Agent::Pi).expect("serialize"),
            "\"Pi\""
        );
        assert_eq!(Agent::try_from("Pi").expect("in-domain"), Agent::Pi);
        // And it is a distinct value, not an alias of anything else.
        assert_ne!(Agent::Pi, Agent::Custom(String::new()));
        assert_ne!(Agent::Pi, Agent::Opencode);
    }

    /// The error message names the whole domain, so a reader who hits an unknown
    /// value sees the complete set rather than a partial one.
    #[test]
    fn the_agent_error_names_the_whole_domain_including_pi() {
        let err = Agent::try_from("Bogus").expect_err("must reject");
        assert!(err.expected.contains("Pi"), "{}", err.expected);
        assert_eq!(
            err.expected,
            "ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Pi, Custom, Custom(<payload>)"
        );
    }

    /// PascalCase, externally tagged — unchanged from the former `Runtime`
    /// serde shape. AD-011: PostgreSQL values are canonical.
    #[test]
    fn serde_shape_is_pascal_case_and_externally_tagged() {
        assert_eq!(
            serde_json::to_string(&Agent::ClaudeCode).expect("serialize"),
            "\"ClaudeCode\""
        );
        assert_eq!(
            serde_json::to_string(&Harness::Stepcode).expect("serialize"),
            "\"Stepcode\""
        );
        assert_eq!(
            serde_json::to_string(&Agent::Custom("x".into())).expect("serialize"),
            "{\"Custom\":\"x\"}"
        );
    }
}
