use std::path::{Path, PathBuf};

use crate::ledger::event::CanonicalEvent;
use crate::storage::jsonl_io::{append_jsonl, JsonlIoError};

/// Append-only ledger writer. Writes one CanonicalEvent per line as JSON to events.jsonl.
/// Uses O_APPEND + flush-after-write for deterministic persistence.
pub struct LedgerWriter {
    pub events_path: PathBuf,
}

impl LedgerWriter {
    pub fn new(events_path: &Path) -> Self {
        Self {
            events_path: events_path.to_path_buf(),
        }
    }

    pub fn append(&self, event: &CanonicalEvent) -> Result<(), JsonlIoError> {
        append_jsonl(&self.events_path, event)
    }
}
