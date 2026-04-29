use seatloom_core::objects::session::PromptState;

/// Get the current PromptState for a blocked session (AD-012).
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn get_prompt_state(_session_id: String) -> Option<PromptState> {
    // TODO: read from session runtime state
    None
}
