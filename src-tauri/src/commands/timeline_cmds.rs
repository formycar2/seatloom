use seatloom_core::ledger::event::CanonicalEvent;

/// Query the Timeline with optional filters.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn query_timeline() -> Vec<CanonicalEvent> {
    // TODO: read from LedgerReader + apply filters
    vec![]
}
