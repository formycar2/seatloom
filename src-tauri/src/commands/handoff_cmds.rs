use seatloom_core::objects::handoff::Handoff;
use seatloom_core::storage::handoff_store::HandoffStore;

/// List all Handoffs for the current project.
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `created_at` descending.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn list_handoffs() -> Vec<Handoff> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let store = HandoffStore::new(&root);
    store.list_handoffs().unwrap_or_default()
}
