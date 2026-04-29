/// Trigger a reconciliation pass: Git/FS ↔ Ledger comparison.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn reconcile() -> bool {
    // TODO: invoke ReconcileEngine, emit ReconcileCompleted/DriftDetected events
    false
}
