use seatloom_core::objects::artifact::Artifact;
use seatloom_core::storage::artifact_store::ArtifactStore;

/// List Artifacts, optionally filtered by template and/or subtype (AD-008).
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `created_at` descending.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn list_artifacts(
    _template: Option<String>,
    _subtype: Option<String>,
    _workitem_id: Option<String>,
) -> Vec<Artifact> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let store = ArtifactStore::new(&root);
    store
        .list_artifacts(None, _subtype.as_deref(), _workitem_id.as_deref())
        .unwrap_or_default()
}
