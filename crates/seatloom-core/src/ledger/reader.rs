use std::path::{Path, PathBuf};

use crate::ledger::event::CanonicalEvent;
use crate::storage::jsonl_io::{read_jsonl, JsonlIoError};

/// Ledger reader for replay and filtered queries over events.jsonl.
pub struct LedgerReader {
    pub events_path: PathBuf,
}

impl LedgerReader {
    pub fn new(events_path: &Path) -> Self {
        Self {
            events_path: events_path.to_path_buf(),
        }
    }

    pub fn read_all(&self) -> Result<Vec<CanonicalEvent>, JsonlIoError> {
        if !self.events_path.exists() {
            return Ok(vec![]);
        }

        read_jsonl(&self.events_path)
    }
}
