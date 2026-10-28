//! The Supervisor intent tool surface — **one sentence in, a structured
//! proposal out, and never a write.**
//!
//! Product contract: `LYRA-2026-10-13-supervisor-intent-entry-contract-v1`
//! (§3 interaction flow, §4 screen definition, §7.2 slice-1 acceptance
//! criteria). This module is the *tool* half of slice 1; the Tauri command in
//! `src-tauri/src/commands/dispatch_cmds.rs` is the thin wrapper that calls the
//! same MCP tool, so there is exactly one implementation.
//!
//! # The three rules the whole module is built around
//!
//! 1. **Deterministic parse, no LLM.** [`crate::intent::parser`] is a pure
//!    function over the sentence. A parse it cannot make is a typed result, not
//!    something to repair by inference.
//! 2. **Resolve or STOP.** [`crate::intent::plan`] selects a seat only when the
//!    sentence named one and that name matches exactly one seat. Anything else
//!    stops at a clarification card with the real choices and the reason each is
//!    offered.
//! 3. **No write without a confirm.** This module has no write path at all. The
//!    proposal carries `suggested_dependencies` in `enqueue_dispatch`'s own
//!    spelling so the caller can pass them straight through on confirm.
//!
//! # What is deliberately absent
//!
//! No composed answer, no semantic matching, no default owner, no constant
//! budget, no "the seat is probably busy" heuristic. Each absence is a §3.3 row
//! that fires as a typed result instead.

pub mod facts;
pub mod parser;
pub mod plan;
pub mod schema;

/// Shared fixtures for the intent unit tests. One fixture, used by the parser
/// and the planner tests, so the two never drift into testing different worlds.
#[cfg(test)]
pub(crate) mod tests_support;

pub use facts::{IntentFacts, SeatFact};
pub use plan::{plan, IntentPlan};
pub use schema::{
    Clarification, ClarificationChoice, DepKind, Intent, IntentObjectType, IntentPlanKind,
    IntentQualifiers, IntentVerb, Proposal, SuggestedDependency, UnknownDomain,
};
