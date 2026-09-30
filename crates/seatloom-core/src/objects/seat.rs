// Three-layer seat model: AD-009.
// Layer 1 (SeatIdentity) is global and project-agnostic.
// Layer 2 (ProjectRoleBind) is per-project and stored separately.
// Layer 3 (SeatDelegation) is a scoped overlay that does not rewrite identity.

use crate::objects::id::{DelegationId, SeatId, WorkItemId};
use crate::objects::session::{
    parse_custom_tagged, render_custom_tagged, Agent, Harness, ModelId, ParseValueError,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Layer 1: Durable global seat identity. Stable across projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatIdentity {
    pub id: SeatId,
    pub name: String,
    /// Default runtime triple. Each component is independently `Option`:
    /// `None` means *not recorded*, never a default to be guessed at.
    pub default_harness: Option<Harness>,
    pub default_agent: Option<Agent>,
    pub default_model: Option<ModelId>,
    pub capability_tags: Vec<String>,
    pub status: SeatStatus,
    pub created_at: DateTime<Utc>,
}

/// Layer 2: Per-project role binding. Encodes role, authority docs, constraints,
/// and active delegation for one project. Stored separately from global identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoleBind {
    pub seat_id: SeatId,
    pub project_id: String,
    pub role: SeatRole,
    pub authority_doc_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub collaboration_template_ref: Option<String>,
    pub active_delegation_id: Option<DelegationId>,
}

/// Layer 3: Scoped delegation overlay. Does not rewrite original seat identity.
/// Timeline shows: "{to_seat} acting for {from_seat} on {workitem_id}".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatDelegation {
    pub id: DelegationId,
    pub issuer_seat_id: SeatId,
    pub from_seat_id: SeatId,
    pub to_seat_id: SeatId,
    pub workitem_id: Option<WorkItemId>,
    pub scope_description: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: DelegationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationStatus {
    Active,
    Closed,
    Expired,
}

/// Per-project role a seat is bound to.
///
/// Serializes in **snake_case**, matching the values already written by
/// `infra/postgres/seed/001_real_collaboration_baseline.sql` — `supervisor`,
/// `product_owner`, `designer`, `architect`, `verifier` at `:33-37` as of base
/// commit 9599136, plus `data_engineer` at `:45` as of 18a69fb. PostgreSQL is
/// canonical (AD-011), so the DB values win.
///
/// This is deliberately the opposite casing from `Harness`/`Agent`
/// (PascalCase, `objects/session.rs`). Both follow the same rule — *match what
/// is already in PostgreSQL* — and produce opposite casing because the existing
/// data is inconsistent. Harmonizing them would mean rewriting historical seed
/// rows, which is out of scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatRole {
    Supervisor,
    ProductOwner,
    Architect,
    Verifier,
    Designer,
    DataEngineer,
    Custom(String),
}

const SEAT_ROLE_EXPECTED: &str = "supervisor, product_owner, architect, verifier, designer, \
                                  data_engineer, custom, custom(<payload>)";

impl FromStr for SeatRole {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "supervisor" => Ok(Self::Supervisor),
            "product_owner" => Ok(Self::ProductOwner),
            "architect" => Ok(Self::Architect),
            "verifier" => Ok(Self::Verifier),
            "designer" => Ok(Self::Designer),
            "data_engineer" => Ok(Self::DataEngineer),
            _ => parse_custom_tagged(s, "custom")
                .map(Self::Custom)
                .ok_or_else(|| ParseValueError {
                    domain: "SeatRole",
                    value: s.to_string(),
                    expected: SEAT_ROLE_EXPECTED,
                }),
        }
    }
}

impl TryFrom<&str> for SeatRole {
    type Error = ParseValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for SeatRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Supervisor => f.write_str("supervisor"),
            Self::ProductOwner => f.write_str("product_owner"),
            Self::Architect => f.write_str("architect"),
            Self::Verifier => f.write_str("verifier"),
            Self::Designer => f.write_str("designer"),
            Self::DataEngineer => f.write_str("data_engineer"),
            Self::Custom(payload) => f.write_str(&render_custom_tagged("custom", payload)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatStatus {
    Active,
    Paused,
    Archived,
}

#[cfg(test)]
mod tests {
    use super::SeatRole;

    /// Every role value the seed writes must now parse. Base commit 9599136
    /// has five, at `seed/001_real_collaboration_baseline.sql:33-37`; `18a69fb`
    /// adds the onyx `data_engineer` binding at `:45`. Packet §1.3 cited the
    /// post-18a69fb shape as a base-commit fact — see delivery doc §9. Before
    /// this change `supervisor` and `data_engineer` had no variant at all.
    #[test]
    fn every_seeded_role_value_parses() {
        for (s, v) in [
            ("supervisor", SeatRole::Supervisor),
            ("product_owner", SeatRole::ProductOwner),
            ("designer", SeatRole::Designer),
            ("architect", SeatRole::Architect),
            ("verifier", SeatRole::Verifier),
            ("data_engineer", SeatRole::DataEngineer),
        ] {
            assert_eq!(SeatRole::try_from(s).expect("seeded value must parse"), v);
            assert_eq!(v.to_string(), s, "Display must invert FromStr");
        }
    }

    #[test]
    fn unknown_role_is_a_typed_error_not_a_custom_fallback() {
        let err = SeatRole::try_from("ProductOwner")
            .expect_err("PascalCase is out-of-domain for SeatRole");
        assert_eq!(err.domain, "SeatRole");
        assert_eq!(err.value, "ProductOwner");
    }

    #[test]
    fn custom_role_round_trips() {
        assert_eq!(
            SeatRole::try_from("custom(scribe)").expect("in-domain"),
            SeatRole::Custom("scribe".to_string())
        );
        assert_eq!(
            SeatRole::Custom("scribe".to_string()).to_string(),
            "custom(scribe)"
        );
    }

    /// The serde representation is snake_case, deliberately opposite to the
    /// PascalCase `Harness`/`Agent` in `objects/session.rs`.
    #[test]
    fn serde_shape_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&SeatRole::DataEngineer).expect("serialize"),
            "\"data_engineer\""
        );
        assert_eq!(
            serde_json::from_str::<SeatRole>("\"supervisor\"").expect("deserialize"),
            SeatRole::Supervisor
        );
    }
}
