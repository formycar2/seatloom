// AgentAdapter trait and supporting types.
// Defines the capability boundary between SeatLoom core and individual runtime adapters.

use std::path::Path;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::objects::session::Runtime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    pub can_launch: bool,
    pub can_attach: bool,
    pub can_resume: bool,
    pub can_capture_transcript: bool,
    pub can_capture_tool_calls: bool,
    pub can_inject_input: bool,
}

/// Opaque handle to a running agent process.
pub struct RunningSession {
    pub pid: u32,
}

/// Single structured unit from raw adapter output.
pub struct TranscriptEntry {
    pub raw: Vec<u8>,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn runtime(&self) -> Runtime;
    fn capabilities(&self) -> AdapterCapabilities;

    async fn launch(
        &self,
        working_dir: &Path,
        initial_input: Option<&str>,
    ) -> Result<RunningSession, Box<dyn std::error::Error + Send + Sync>>;

    async fn attach(
        &self,
        pid: u32,
    ) -> Result<RunningSession, Box<dyn std::error::Error + Send + Sync>>;

    async fn inject_input(
        &self,
        session: &RunningSession,
        input: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn parse_transcript(&self, raw_output: &[u8]) -> Vec<TranscriptEntry>;

    fn detect_native_session_id(&self, working_dir: &Path) -> Option<String>;
}
