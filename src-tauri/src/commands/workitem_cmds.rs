use seatloom_core::objects::workitem::WorkItem;
use seatloom_core::storage::workitem_store::WorkItemStore;

/// List all WorkItems for the current project.
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `updated_at` descending.
pub fn list_workitems() -> Vec<WorkItem> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let store = WorkItemStore::new(&root);
    store.list_workitems().unwrap_or_default()
}

/// Create a new WorkItem.
pub fn create_workitem(_title: String) -> Option<WorkItem> {
    // TODO: generate ID, write YAML, emit WorkItemCreated event
    None
}
