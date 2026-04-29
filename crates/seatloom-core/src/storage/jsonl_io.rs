use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonlIoError {
    #[error("failed to create parent directory for {}: {source}", path.display())]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to open JSONL file {}: {source}", path.display())]
    Open {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize JSONL record for {}: {source}", path.display())]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write JSONL record to {}: {source}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to flush JSONL file {}: {source}", path.display())]
    Flush {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to sync JSONL file {}: {source}", path.display())]
    Sync {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read JSONL file {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse JSONL line {line} in {}: {source}", path.display())]
    ParseLine {
        path: PathBuf,
        line: usize,
        #[source]
        source: serde_json::Error,
    },
}

pub fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<(), JsonlIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| JsonlIoError::CreateParent {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| JsonlIoError::Open {
            path: path.to_path_buf(),
            source,
        })?;

    serde_json::to_writer(&mut file, value).map_err(|source| JsonlIoError::Serialize {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(b"\n")
        .map_err(|source| JsonlIoError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    file.flush().map_err(|source| JsonlIoError::Flush {
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_data().map_err(|source| JsonlIoError::Sync {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, JsonlIoError> {
    let file = fs::File::open(path).map_err(|source| JsonlIoError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();

    for (index, line_result) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line_result.map_err(|source| JsonlIoError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str(&line).map_err(|source| JsonlIoError::ParseLine {
            path: path.to_path_buf(),
            line: line_number,
            source,
        })?;
        values.push(value);
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};

    use super::{append_jsonl, read_jsonl};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct SampleRecord {
        id: String,
        count: u32,
    }

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn jsonl_append_and_read_round_trip() {
        let dir = temp_dir("jsonl-roundtrip");
        let path = dir.join("ledger/events.jsonl");

        append_jsonl(
            &path,
            &SampleRecord {
                id: "one".to_string(),
                count: 1,
            },
        )
        .expect("first record should append");
        append_jsonl(
            &path,
            &SampleRecord {
                id: "two".to_string(),
                count: 2,
            },
        )
        .expect("second record should append");

        let loaded: Vec<SampleRecord> = read_jsonl(&path).expect("jsonl should read");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "one");
        assert_eq!(loaded[1].count, 2);

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }
}
