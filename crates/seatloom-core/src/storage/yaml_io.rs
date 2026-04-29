use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YamlIoError {
    #[error("failed to read YAML file {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse YAML file {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("failed to create parent directory for {}: {source}", path.display())]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize YAML for {}: {source}", path.display())]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("failed to write temp YAML file {}: {source}", path.display())]
    WriteTemp {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to sync temp YAML file {}: {source}", path.display())]
    SyncTemp {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "failed to persist YAML file {} from {}: {source}",
        final_path.display(),
        temp_path.display()
    )]
    Persist {
        temp_path: PathBuf,
        final_path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, YamlIoError> {
    let raw = fs::read_to_string(path).map_err(|source| YamlIoError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str(&raw).map_err(|source| YamlIoError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_yaml<T: Serialize>(path: &Path, value: &T) -> Result<(), YamlIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| YamlIoError::CreateParent {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let serialized = serde_yaml::to_string(value).map_err(|source| YamlIoError::Serialize {
        path: path.to_path_buf(),
        source,
    })?;

    let temp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path).map_err(|source| YamlIoError::WriteTemp {
        path: temp_path.clone(),
        source,
    })?;
    file.write_all(serialized.as_bytes())
        .and_then(|_| file.flush())
        .map_err(|source| YamlIoError::WriteTemp {
            path: temp_path.clone(),
            source,
        })?;
    file.sync_all().map_err(|source| YamlIoError::SyncTemp {
        path: temp_path.clone(),
        source,
    })?;

    fs::rename(&temp_path, path).map_err(|source| YamlIoError::Persist {
        temp_path,
        final_path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{TimeZone, Utc};

    use crate::storage::project::ProjectConfig;

    use super::{read_yaml, write_yaml};

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
    fn project_config_round_trips_through_yaml() {
        let dir = temp_dir("yaml-roundtrip");
        let path = dir.join("config/project.yaml");
        let config = ProjectConfig::new(
            "seatloom",
            Utc.with_ymd_and_hms(2026, 4, 29, 0, 0, 0)
                .single()
                .expect("valid timestamp"),
        );

        write_yaml(&path, &config).expect("yaml should write");
        let loaded: ProjectConfig = read_yaml(&path).expect("yaml should read");

        assert_eq!(loaded.project_name, "seatloom");
        assert_eq!(loaded.version, "0.1");
        assert_eq!(loaded.created_at, config.created_at);
        assert_eq!(loaded.pack_engine.worker_budget_tokens, 8_192);
        assert_eq!(loaded.pipeline.max_retry, 2);

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }
}
