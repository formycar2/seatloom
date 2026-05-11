// tmux mirror primitive for SeatLoom v0.0.1 (SG-A §A3).
//
// SeatLoom no longer spawns CLI processes. Per the tmux-mirror architecture
// (docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md,
// commit a5998c1), the runtime is tmux; SeatLoom attaches to existing tmux
// sessions and mirrors pane output via `tmux pipe-pane` → named FIFO →
// tokio tail-read → broadcast channel. Failure isolation (R3) is a hard rule:
// SeatLoom is NEVER the parent process of tmux or the wrapped CLI.
//
// Module name is kept as `pty` to avoid a cross-cutting import rename; the
// externally-visible type is `PtySession` (re-exported) but its internals now
// hold a tmux session handle, not a portable-pty PTY pair. `LaunchOptions` and
// `PtyError` are retained for source-level continuity with callers that still
// import them, and `default_transcripts_dir` remains available for any adapter
// that wants to log raw output alongside the fifo.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nix::sys::stat::Mode;
use nix::unistd;
use thiserror::Error;
use tokio::sync::broadcast;

/// Output events broadcast from the tmux mirror reader to subscribers.
#[derive(Debug, Clone)]
pub enum PtyEvent {
    /// Raw bytes read from the tmux pipe-pane FIFO.
    Output(Vec<u8>),
    /// Pipe-pane disconnected (FIFO closed or read error). Mirror is no longer
    /// live; consumer should reattach or drop the session handle.
    Exited {
        exit_code: Option<i32>,
        signal: Option<String>,
    },
}

/// Handle to an attached tmux mirror session. Clone is cheap (Arc-based).
#[derive(Clone)]
pub struct PtySession {
    pub id: String,
    /// Name of the underlying tmux session being mirrored.
    pub tmux_session_name: String,
    /// Target pane spec in tmux terms, e.g. `"Lyra-po-seatloom:0"`.
    pub tmux_target: String,
    /// Path of the FIFO that tmux pipe-pane writes to.
    pub fifo_path: PathBuf,
    /// Broadcast channel for mirrored bytes + exit events.
    events: broadcast::Sender<PtyEvent>,
    /// Set to true once `kill()` runs, so the tail-read task can exit cleanly.
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    /// Path to a raw transcript log (written alongside the fifo tail), kept
    /// for adapter consumers that expect a file-backed record.
    pub transcript_path: PathBuf,
}

impl std::fmt::Debug for PtySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySession")
            .field("id", &self.id)
            .field("tmux_session_name", &self.tmux_session_name)
            .field("tmux_target", &self.tmux_target)
            .field("fifo_path", &self.fifo_path)
            .field("transcript_path", &self.transcript_path)
            .finish()
    }
}

/// Preserved for source-level compatibility with older call sites. Under the
/// tmux-mirror architecture, most of these fields are ignored — only
/// `transcripts_dir`, `rows`, and `cols` carry meaning (rows/cols feed
/// `tmux resize-pane` when `PtySession::resize` is invoked).
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub rows: u16,
    pub cols: u16,
    pub env: Vec<(String, String)>,
    pub transcripts_dir: PathBuf,
}

impl LaunchOptions {
    pub fn default_env() -> Vec<(String, String)> {
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
    #[error("tmux error: {0}")]
    Tmux(String),
    #[error("fifo error: {0}")]
    Fifo(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

/// Metadata for a discovered tmux session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TmuxSessionInfo {
    pub session_name: String,
    /// Unix timestamp (seconds) from `#{session_created}`.
    pub created_at: i64,
    pub attached: bool,
}

/// Directory under which pipe-pane FIFOs and transcript logs live.
pub fn mirror_dir() -> PathBuf {
    PathBuf::from("/tmp/seatloom-mirror")
}

/// Default transcripts dir (kept for adapter callers that still want a log
/// file alongside the mirror — e.g., ClaudeAdapter).
pub fn default_transcripts_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".seatloom").join("transcripts")
}

/// List all tmux sessions whose name ends in `-seatloom` (the convention used
/// by the Aegis/Lyra/Mira/Nimbus/Flux seat sessions). Returns an empty vec if
/// tmux is not installed, not running, or no matching sessions exist.
pub fn list_tmux_sessions() -> Result<Vec<TmuxSessionInfo>, PtyError> {
    let output = std::process::Command::new("tmux")
        .args([
            "list-sessions",
            "-F",
            "#{session_name}:#{session_created}:#{session_attached}",
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(PtyError::Tmux(format!("spawn tmux: {e}"))),
    };

    if !output.status.success() {
        // Exit code 1 with "no server running" stderr is the "no sessions"
        // signal on most tmux builds. Treat as empty, not error.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") || stderr.contains("no current client") {
            return Ok(Vec::new());
        }
        return Err(PtyError::Tmux(format!(
            "tmux list-sessions failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sessions = Vec::new();
    for line in stdout.lines() {
        let mut parts = line.splitn(3, ':');
        let Some(name) = parts.next() else { continue };
        let Some(created) = parts.next() else { continue };
        let Some(attached) = parts.next() else { continue };
        if !name.ends_with("-seatloom") {
            continue;
        }
        let created_at = created.parse::<i64>().unwrap_or(0);
        let attached_bool = attached.trim() != "0";
        sessions.push(TmuxSessionInfo {
            session_name: name.to_string(),
            created_at,
            attached: attached_bool,
        });
    }
    Ok(sessions)
}

impl PtySession {
    /// Attach to an existing tmux session and mirror its first pane (`:0`).
    ///
    /// Steps:
    ///   1. Create a named FIFO at `/tmp/seatloom-mirror/<session_id>.fifo`.
    ///   2. Run `tmux pipe-pane -t <name>:0 -o 'cat >> <fifo>'` so tmux
    ///      appends pane output to the FIFO (and keeps doing so even if
    ///      SeatLoom dies — R3 failure isolation).
    ///   3. Spawn a std::thread that opens the FIFO read-side, tail-reads
    ///      it, and broadcasts `PtyEvent::Output` chunks.
    ///
    /// `opts.rows` / `opts.cols` are currently informational; callers that
    /// want to resize the pane should invoke `PtySession::resize`, which maps
    /// to `tmux resize-pane`.
    pub fn attach_tmux(
        id: &str,
        tmux_session_name: &str,
        opts: LaunchOptions,
    ) -> Result<Self, PtyError> {
        // Ensure transcripts dir exists (adapter consumers may want it).
        std::fs::create_dir_all(&opts.transcripts_dir)?;
        let transcript_path = opts.transcripts_dir.join(format!("{id}.raw.log"));

        // Ensure the mirror dir exists. `/tmp/seatloom-mirror/` survives
        // SeatLoom restarts; the FIFO inside it is recreated per attach.
        let fifo_dir = mirror_dir();
        std::fs::create_dir_all(&fifo_dir)?;
        let fifo_path = fifo_dir.join(format!("{id}.fifo"));

        // If a prior attach left a stale FIFO behind, unlink it before mkfifo.
        if fifo_path.exists() {
            let _ = std::fs::remove_file(&fifo_path);
        }
        unistd::mkfifo(
            &fifo_path,
            Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
        )
        .map_err(|e| PtyError::Fifo(format!("mkfifo {}: {e}", fifo_path.display())))?;

        let tmux_target = format!("{tmux_session_name}:0");

        // Ask tmux to copy pane output to our FIFO. `-o` makes the pipe-pane
        // command invoke only once (so we don't duplicate if attach is called
        // twice), and shell-quoting the fifo path guards against unusual chars.
        let cat_cmd = format!("cat >> {}", shell_quote(&fifo_path.to_string_lossy()));
        let out = std::process::Command::new("tmux")
            .args(["pipe-pane", "-t", &tmux_target, "-o", &cat_cmd])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux pipe-pane: {e}")))?;
        if !out.status.success() {
            // Clean up the FIFO on failure so we don't leak named pipes.
            let _ = std::fs::remove_file(&fifo_path);
            return Err(PtyError::Tmux(format!(
                "tmux pipe-pane {tmux_target}: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }

        let (tx, _rx) = broadcast::channel::<PtyEvent>(1024);
        let tx_reader = tx.clone();
        let fifo_for_reader = fifo_path.clone();
        let transcript_for_reader = transcript_path.clone();
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_for_reader = shutdown.clone();

        // Tail-read the FIFO on a std::thread (blocking read — we don't use
        // tokio::fs here because FIFO semantics on Darwin prefer a blocking
        // reader, and we want the same code path on Linux).
        std::thread::spawn(move || {
            // Opening a FIFO for read blocks until a writer opens it. tmux
            // holds the writer open for the lifetime of the pipe-pane.
            let mut file = match std::fs::OpenOptions::new()
                .read(true)
                .open(&fifo_for_reader)
            {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx_reader.send(PtyEvent::Exited {
                        exit_code: None,
                        signal: Some(format!("fifo open failed: {e}")),
                    });
                    return;
                }
            };
            let mut log = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&transcript_for_reader)
                .ok();
            let mut buf = [0u8; 4096];
            loop {
                if shutdown_for_reader.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                match file.read(&mut buf) {
                    Ok(0) => {
                        // EOF on a FIFO means the writer closed. Under normal
                        // operation this fires when `kill()` runs
                        // `tmux pipe-pane` with no args (stopping the copy).
                        break;
                    }
                    Ok(n) => {
                        let chunk = buf[..n].to_vec();
                        if let Some(ref mut f) = log {
                            let _ = f.write_all(&chunk);
                        }
                        if tx_reader.send(PtyEvent::Output(chunk)).is_err() {
                            // No subscribers — keep reading (we're still
                            // logging to the transcript file).
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            let _ = tx_reader.send(PtyEvent::Exited {
                exit_code: Some(0),
                signal: None,
            });
        });

        Ok(PtySession {
            id: id.to_string(),
            tmux_session_name: tmux_session_name.to_string(),
            tmux_target,
            fifo_path,
            events: tx,
            shutdown,
            transcript_path,
        })
    }

    /// Subscribe to output + exit events. The receiver must be polled or it
    /// will drop lagged messages (fine for UI subscribers that can tolerate
    /// jitter).
    pub fn subscribe(&self) -> broadcast::Receiver<PtyEvent> {
        self.events.subscribe()
    }

    /// Forward bytes into the mirrored tmux pane via the buffer path:
    ///   tmux load-buffer -b seatloom - ; paste-buffer -b seatloom -t <target>
    ///
    /// Single subprocess invocation (`;` chaining) avoids a second fork and
    /// prevents interleave between concurrent write() calls on the tmux socket.
    /// Named buffer `-b seatloom` never touches the user's default paste-buffer.
    /// Arbitrary bytes (NUL, 0x03 Ctrl-C, ANSI sequences) are delivered via
    /// stdin to load-buffer, bypassing argv length and NUL restrictions.
    pub fn write(&self, bytes: &[u8]) -> Result<(), PtyError> {
        if self.shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        if bytes.is_empty() {
            return Ok(());
        }
        use std::io::Write as _;
        use std::process::{Command, Stdio};
        let mut child = Command::new("tmux")
            .args([
                "load-buffer", "-b", "seatloom", "-",
                ";",
                "paste-buffer", "-b", "seatloom", "-t", &self.tmux_target,
            ])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux load-buffer: {e}")))?;
        child
            .stdin
            .take()
            .expect("stdin piped")
            .write_all(bytes)
            .map_err(|e| PtyError::Tmux(format!("write to tmux stdin: {e}")))?;
        let out = child
            .wait_with_output()
            .map_err(|e| PtyError::Tmux(format!("tmux write wait: {e}")))?;
        if !out.status.success() {
            return Err(PtyError::Tmux(format!(
                "tmux write to {}: {}",
                self.tmux_target,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        Ok(())
    }

    /// Resize the underlying tmux pane (maps to `tmux resize-pane`). Safe to
    /// call whenever the mirroring xterm container resizes in the UI.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        let out = std::process::Command::new("tmux")
            .args([
                "resize-pane",
                "-t",
                &self.tmux_target,
                "-x",
                &cols.to_string(),
                "-y",
                &rows.to_string(),
            ])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux resize-pane: {e}")))?;
        if !out.status.success() {
            return Err(PtyError::Tmux(format!(
                "tmux resize-pane {}: {}",
                self.tmux_target,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        Ok(())
    }

    /// Detach the mirror: stop the tmux pipe-pane, let the tail-reader hit
    /// EOF, remove the FIFO. **Does NOT kill the tmux session itself** — R3
    /// failure isolation. Safe to call multiple times.
    pub fn kill(&self) -> Result<(), PtyError> {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Ask tmux to stop copying pane output (no `-o` means toggle off if
        // enabled; passing no shell-command disables further copies).
        let _ = std::process::Command::new("tmux")
            .args(["pipe-pane", "-t", &self.tmux_target])
            .output();

        // Unlink the FIFO so no further readers block on it.
        if self.fifo_path.exists() {
            let _ = std::fs::remove_file(&self.fifo_path);
        }
        Ok(())
    }
}

/// Minimal single-quote shell quoter for the FIFO path embedded in the
/// `tmux pipe-pane` shell snippet. tmux runs the command via `/bin/sh -c`.
fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_handles_spaces_and_quotes() {
        assert_eq!(shell_quote("/tmp/x.fifo"), "'/tmp/x.fifo'");
        assert_eq!(shell_quote("/tmp/a b/x.fifo"), "'/tmp/a b/x.fifo'");
        assert_eq!(shell_quote("it's.fifo"), "'it'\\''s.fifo'");
    }

    #[test]
    fn mirror_dir_is_tmp_seatloom_mirror() {
        assert_eq!(mirror_dir(), PathBuf::from("/tmp/seatloom-mirror"));
    }

    #[test]
    fn default_transcripts_dir_nests_under_repo_root() {
        let root = PathBuf::from("/foo/bar");
        assert_eq!(
            default_transcripts_dir(&root),
            PathBuf::from("/foo/bar/.seatloom/transcripts")
        );
    }

    #[test]
    fn list_tmux_sessions_returns_vec_without_panic() {
        let _ = list_tmux_sessions().expect("list_tmux_sessions must not error on happy path");
    }

    #[test]
    fn write_noop_on_shutdown() {
        // When shutdown is set, write() must return Ok(()) without spawning tmux.
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let (tx, _rx) = broadcast::channel::<PtyEvent>(1);
        let sess = PtySession {
            id: "test".into(),
            tmux_session_name: "test".into(),
            tmux_target: "test:0".into(),
            fifo_path: PathBuf::from("/tmp/nonexistent.fifo"),
            events: tx,
            shutdown,
            transcript_path: PathBuf::from("/tmp/nonexistent.log"),
        };
        assert!(sess.write(b"hello").is_ok());
    }

    #[test]
    fn write_noop_on_empty_bytes() {
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (tx, _rx) = broadcast::channel::<PtyEvent>(1);
        let sess = PtySession {
            id: "test".into(),
            tmux_session_name: "test".into(),
            tmux_target: "test:0".into(),
            fifo_path: PathBuf::from("/tmp/nonexistent.fifo"),
            events: tx,
            shutdown,
            transcript_path: PathBuf::from("/tmp/nonexistent.log"),
        };
        // Empty slice must return Ok(()) without spawning tmux.
        assert!(sess.write(b"").is_ok());
    }
}
