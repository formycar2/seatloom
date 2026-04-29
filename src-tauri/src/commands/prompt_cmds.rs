use seatloom_core::objects::session::PromptState;

/// Get the current PromptState for a blocked session (AD-012).
pub fn get_prompt_state(_session_id: String) -> Option<PromptState> {
    // TODO: read from session runtime state
    None
}
