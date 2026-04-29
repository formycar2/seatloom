use seatloom_core::objects::artifact::Artifact;

/// List Artifacts, optionally filtered by template and/or subtype (AD-008).
pub fn list_artifacts(
    _template: Option<String>,
    _subtype: Option<String>,
    _workitem_id: Option<String>,
) -> Vec<Artifact> {
    // TODO: query from retrieval engine L1 structured index
    vec![]
}
