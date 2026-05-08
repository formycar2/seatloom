// PTY session primitive for SeatLoom v0.1.
//
// Owns a portable-pty PTY pair, spawns the child process, and exposes:
//   - a `tokio::sync::broadcast` channel of raw output chunks (for UI streaming)
//   - write / resize / kill operations
//   - transcript-log tee to `.seatloom/transcripts/<session_id>.raw.log`
//
// This module is runtime-generic. Adapter-specific behaviour (Claude Code
// transcript tailing, plan-mode capture) lives in `adapter/claude.rs` on top
// of this primitive. See
// docs/coordination/reviews/2026-05-08-aegis-cli-plan-mode-integration-design.md

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use thiserror::Error;
use tokio::sync::broadcast;

/// Output events broadcast from the PTY reader to subscribers.
#[derive(Debug, Clone)]
pub enum PtyEvent {
    /// Raw bytes read from the PTY master.
    Output(Vec<u8>),
    /// Child process exited (success / failure / signal).
    Exited {
        exit_code: Option<i32>,
        signal: Option<String>,
    },
}

/// Handle to a live PTY session. Clone is cheap (Arc-based internals).
#[derive(Clone)]
pub struct PtySession {
    pub id: String,
    /// Broadcast channel for output + exit events.
    /// Subscribers receive all events after the moment they subscribe.
    events: broadcast::Sender<PtyEvent>,
    /// Master PTY fd wrapped in a mutex so write/resize can be called from any task.
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    /// Child handle wrapped so we can call kill from another task.
    child: Arc<Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>>,
    /// Path to the raw-transcript log file (for later evidence replay).
    pub transcript_path: PathBuf,
}

impl std::fmt::Debug for PtySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySession")
            .field("id", &self.id)
            .field("transcript_path", &self.transcript_path)
            .finish()
    }
}

/// Configuration for launching a new PTY session.
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub rows: u16,
    pub cols: u16,
    pub env: Vec<(String, String)>,
    /// Directory under which `<session_id>.raw.log` is written. Created if absent.
    pub transcripts_dir: PathBuf,
}

impl LaunchOptions {
    pub fn default_env() -> Vec<(String, String)> {
        // Propagate PATH, HOME, USER, LANG, TERM from the parent. Everything else
        // is discarded on purpose — we do not want the child seeing SEATLOOM_*
        // internal env vars.
        let mut out = Vec::new();
        for key in [
            "PATH", "HOME", "USER", "LANG", "LC_ALL", "LC_CTYPE", "LOGNAME", "SHELL",
        ] {
            if let Ok(v) = std::env::var(key) {
                out.push((key.to_string(), v));
            }
        }
        out.push(("TERM".to_string(), "xterm-256color".to_string()));
        out
    }
}

#[derive(Debug, Error)]
pub enum PtyError {
    #[error("pty open error: {0}")]
    PtyOpen(String),
    #[error("spawn error: {0}")]
    Spawn(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

impl PtySession {
    /// Launch a new PTY session. Spawns:
    ///   - a blocking std::thread that reads master PTY output and fans out to subscribers
    ///   - a blocking std::thread that waits on the child for exit and emits Exited event
    pub fn launch(id: &str, opts: LaunchOptions) -> Result<Self, PtyError> {
        std::fs::create_dir_all(&opts.transcripts_dir)?;
        let transcript_path = opts.transcripts_dir.join(format!("{id}.raw.log"));

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: opts.rows,
                cols: opts.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::PtyOpen(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&opts.command);
        cmd.args(&opts.args);
        cmd.cwd(&opts.working_dir);
        // Clear the env first; portable-pty inherits parent env otherwise.
        cmd.env_clear();
        for (k, v) in &opts.env {
            cmd.env(k, v);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::Spawn(e.to_string()))?;
        drop(pair.slave); // we only keep master for I/O

        let master = pair.master;
        let reader = master
            .try_clone_reader()
            .map_err(|e| PtyError::Spawn(format!("cannot clone reader: {e}")))?;

        let (tx, _rx) = broadcast::channel::<PtyEvent>(1024);
        let tx_reader = tx.clone();
        let transcript = transcript_path.clone();

        // Output reader thread. portable-pty's Reader is sync, so we use std::thread.
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            let mut log = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&transcript)
                .ok();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let chunk = buf[..n].to_vec();
                        if let Some(ref mut f) = log {
                            let _ = f.write_all(&chunk);
                        }
                        // Drop send errors silently — no subscribers is fine.
                        let _ = tx_reader.send(PtyEvent::Output(chunk));
                    }
                    Err(_) => break,
                }
            }
        });

        // Child waiter thread.
        let tx_exit = tx.clone();
        let child_arc: Arc<Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>> =
            Arc::new(Mutex::new(Some(child)));
        let child_for_waiter = child_arc.clone();
        std::thread::spawn(move || {
            // Poll loop: portable-pty's Child exposes try_wait but not blocking wait
            // uniformly across platforms.
            loop {
                std::thread::sleep(Duration::from_millis(250));
                let mut guard = child_for_waiter.lock().unwrap();
                let Some(child) = guard.as_mut() else {
                    return;
                };
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let exit_code = status.exit_code() as i32;
                        let _ = tx_exit.send(PtyEvent::Exited {
                            exit_code: Some(exit_code),
                            signal: None,
                        });
                        *guard = None; // release the child
                        return;
                    }
                    Ok(None) => continue,
                    Err(_) => {
                        let _ = tx_exit.send(PtyEvent::Exited {
                            exit_code: None,
                            signal: Some("wait-failed".to_string()),
                        });
                        *guard = None;
                        return;
                    }
                }
            }
        });

        Ok(PtySession {
            id: id.to_string(),
            events: tx,
            master: Arc::new(Mutex::new(master)),
            child: child_arc,
            transcript_path,
        })
    }

    /// Subscribe to output + exit events. The receiver must be polled or it will
    /// drop lagged messages (fine for UI subscribers that can tolerate jitter).
    pub fn subscribe(&self) -> broadcast::Receiver<PtyEvent> {
        self.events.subscribe()
    }

    /// Write bytes to the PTY master (injected input).
    pub fn write(&self, bytes: &[u8]) -> Result<(), PtyError> {
        let master = self.master.lock().unwrap();
        let mut writer = master
            .take_writer()
            .map_err(|e| PtyError::Spawn(format!("take_writer: {e}")))?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    /// Resize the PTY (forwards WINCH to the child).
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        let master = self.master.lock().unwrap();
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::Spawn(e.to_string()))?;
        Ok(())
    }

    /// Kill the child process. Safe to call multiple times.
    pub fn kill(&self) -> Result<(), PtyError> {
        let mut guard = self.child.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        Ok(())
    }
}

/// Helper: default transcripts dir under a repo root.
pub fn default_transcripts_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".seatloom").join("transcripts")
}
