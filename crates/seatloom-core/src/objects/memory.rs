// Knowledge layer (D5, migration 015) — the closed value domains of a memory.
//
// Every TEXT enum here follows the house rule the value-domain contract §7 makes
// binding: hand-written `FromStr` / `Display`, NO serde derive across the
// Rust<->PG boundary, and an unknown value is a typed `ParseValueError` — never
// a silent default. `EventType` / `RefType` in `db/event_domain.rs` are the same
// pattern; the MCP tools parse their string arguments through these `FromStr`
// impls rather than letting serde accept anything.
//
// The domains themselves (names, values, cardinality) are the contract's,
// verbatim — they are not this file's to invent.

use std::fmt;
use std::str::FromStr;

use crate::objects::session::ParseValueError;

// =============================================================================
// Progressive disclosure: the layer domain (3)
// =============================================================================

/// Which layer of a memory a recall returns. `L0` is the ONLY default (~100
/// tokens); `L1` (~2K tokens) and `L2` (the raw pointer) are fetched only on an
/// explicit request. The token budget belongs to the caller, not to us.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    L0,
    L1,
    L2,
}

impl MemoryLayer {
    /// Every admitted value, in contract order.
    pub const ALL: [MemoryLayer; 3] = [MemoryLayer::L0, MemoryLayer::L1, MemoryLayer::L2];

    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::L0 => "l0",
            MemoryLayer::L1 => "l1",
            MemoryLayer::L2 => "l2",
        }
    }
}

impl fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MemoryLayer {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "l0" => Ok(MemoryLayer::L0),
            "l1" => Ok(MemoryLayer::L1),
            "l2" => Ok(MemoryLayer::L2),
            _ => Err(ParseValueError {
                domain: "MemoryLayer",
                value: s.to_string(),
                expected: "l0 | l1 | l2",
            }),
        }
    }
}

impl TryFrom<&str> for MemoryLayer {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// =============================================================================
// The tier domain (3): cold | warm | hot
// =============================================================================

/// A memory's tier. Transitions happen ONLY through the `promote` tool, with an
/// explicit target tier, up OR down. There is no automatic decay and no
/// background daemon — `heat` moves only when a recall happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTier {
    Cold,
    Warm,
    Hot,
}

impl MemoryTier {
    /// Every admitted value, in contract order.
    pub const ALL: [MemoryTier; 3] = [MemoryTier::Cold, MemoryTier::Warm, MemoryTier::Hot];

    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryTier::Cold => "cold",
            MemoryTier::Warm => "warm",
            MemoryTier::Hot => "hot",
        }
    }
}

impl fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MemoryTier {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cold" => Ok(MemoryTier::Cold),
            "warm" => Ok(MemoryTier::Warm),
            "hot" => Ok(MemoryTier::Hot),
            _ => Err(ParseValueError {
                domain: "MemoryTier",
                value: s.to_string(),
                expected: "cold | warm | hot",
            }),
        }
    }
}

impl TryFrom<&str> for MemoryTier {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// =============================================================================
// The L2 pointer kind (2): document | file
// =============================================================================

/// Which pointer an L2 reference carries. The pairing with the actual pointer
/// (`l2_document_id` vs `l2_file_path`) is enforced twice: by the writing tool
/// (typed rejection) and by `015`'s `memories_l2_pairing` CHECK — belt and
/// braces, because a half-filled pointer is a memory that cannot be followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L2RefKind {
    Document,
    File,
}

impl L2RefKind {
    pub const ALL: [L2RefKind; 2] = [L2RefKind::Document, L2RefKind::File];

    pub fn as_str(&self) -> &'static str {
        match self {
            L2RefKind::Document => "document",
            L2RefKind::File => "file",
        }
    }
}

impl fmt::Display for L2RefKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for L2RefKind {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "document" => Ok(L2RefKind::Document),
            "file" => Ok(L2RefKind::File),
            _ => Err(ParseValueError {
                domain: "L2RefKind",
                value: s.to_string(),
                expected: "document | file",
            }),
        }
    }
}

impl TryFrom<&str> for L2RefKind {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// =============================================================================
// The source kind (4): what a memory was distilled FROM
// =============================================================================

/// Provenance of a memory. `source_ref` is polymorphic TEXT with NO FK (the
/// 010 ref-fabric reasoning): an `archived_session` carries `'<harness>:<id>'`,
/// a `canonical_event` an event id, a `document` a doc id, and `manual` an
/// actor note. Provenance is recorded, never guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemorySourceKind {
    ArchivedSession,
    CanonicalEvent,
    Document,
    Manual,
}

impl MemorySourceKind {
    pub const ALL: [MemorySourceKind; 4] = [
        MemorySourceKind::ArchivedSession,
        MemorySourceKind::CanonicalEvent,
        MemorySourceKind::Document,
        MemorySourceKind::Manual,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            MemorySourceKind::ArchivedSession => "archived_session",
            MemorySourceKind::CanonicalEvent => "canonical_event",
            MemorySourceKind::Document => "document",
            MemorySourceKind::Manual => "manual",
        }
    }
}

impl fmt::Display for MemorySourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MemorySourceKind {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "archived_session" => Ok(MemorySourceKind::ArchivedSession),
            "canonical_event" => Ok(MemorySourceKind::CanonicalEvent),
            "document" => Ok(MemorySourceKind::Document),
            "manual" => Ok(MemorySourceKind::Manual),
            _ => Err(ParseValueError {
                domain: "MemorySourceKind",
                value: s.to_string(),
                expected: "archived_session | canonical_event | document | manual",
            }),
        }
    }
}

impl TryFrom<&str> for MemorySourceKind {
    type Error = ParseValueError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every domain round-trips `Display` -> `FromStr` -> itself, and an unknown
    /// value is a typed error naming the domain — never a default. This is the
    /// property a serde derive would silently weaken at the TEXT boundary.
    #[test]
    fn every_memory_domain_round_trips_and_rejects_unknowns() {
        for v in MemoryLayer::ALL {
            assert_eq!(MemoryLayer::from_str(v.as_str()).expect("round-trip"), v);
        }
        for v in MemoryTier::ALL {
            assert_eq!(MemoryTier::from_str(v.as_str()).expect("round-trip"), v);
        }
        for v in L2RefKind::ALL {
            assert_eq!(L2RefKind::from_str(v.as_str()).expect("round-trip"), v);
        }
        for v in MemorySourceKind::ALL {
            assert_eq!(
                MemorySourceKind::from_str(v.as_str()).expect("round-trip"),
                v
            );
        }

        let err = MemoryLayer::from_str("l3").expect_err("l3 is not a layer");
        assert_eq!(err.domain, "MemoryLayer");
        assert!(err.to_string().contains("l0 | l1 | l2"), "{err}");
        assert!(MemoryTier::from_str("tepid").is_err());
        assert!(L2RefKind::from_str("url").is_err());
        assert!(MemorySourceKind::from_str("vibes").is_err());

        // Case is significant: the DB stores lowercase, and a capitalized value
        // is a different (unknown) value, not a silent normalisation.
        assert!(MemoryLayer::from_str("L0").is_err());
        assert!(MemoryTier::from_str("Cold").is_err());
    }

    /// The domains are the contract's, in the contract's order — pinned so a
    /// future "helpful" reordering cannot change the wire values.
    #[test]
    fn domain_values_match_the_contract_verbatim() {
        let layers: Vec<&str> = MemoryLayer::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(layers, vec!["l0", "l1", "l2"]);
        let tiers: Vec<&str> = MemoryTier::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(tiers, vec!["cold", "warm", "hot"]);
        let kinds: Vec<&str> = L2RefKind::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(kinds, vec!["document", "file"]);
        let sources: Vec<&str> = MemorySourceKind::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            sources,
            vec!["archived_session", "canonical_event", "document", "manual"]
        );
    }
}
