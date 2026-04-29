// Retrieval engine: fixed L1→L2→L3→explain layer order (AD-011).
// L1 structured index + L2 full-text: SQLite FTS5 (persistent, authoritative — BLOCKER-001 closed).
// L3 semantic + L4 LLM: deferred to P1.
// Full implementation deferred to data-engine packet.
pub struct RetrievalEngine;

impl RetrievalEngine {
    pub fn new() -> Self { Self }
}
