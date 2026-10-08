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
    /// Actual pane dimensions queried from tmux at attach time via display-message.
    pub pane_rows: u16,
    pub pane_cols: u16,
    /// Snapshot of pane content captured via `tmux capture-pane -p` at attach
    /// time. Empty if the pane was empty or capture-pane failed. Consumers
    /// should write these bytes to their renderer before tail-reading new
    /// FIFO bytes — fixes the "black screen on idle session" bug.
    pub initial_snapshot: Vec<u8>,
}

impl std::fmt::Debug for PtySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySession")
            .field("id", &self.id)
            .field("tmux_session_name", &self.tmux_session_name)
            .field("tmux_target", &self.tmux_target)
            .field("fifo_path", &self.fifo_path)
            .field("transcript_path", &self.transcript_path)
            .field("pane_rows", &self.pane_rows)
            .field("pane_cols", &self.pane_cols)
            .finish()
    }
}

/// Preserved for source-level compatibility with older call sites. Under the
/// tmux-mirror architecture, most of these fields are ignored — only
/// `transcripts_dir`, `rows`, and `cols` carry meaning (rows/cols feed
/// `tmux resize-window` when `PtySession::resize` is invoked).
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
    /// `tmux pipe-pane` exited 0 but the pane is not actually piped. Returned
    /// rather than handing back a session that looks live and produces no bytes
    /// — the silent dead mirror is the real user harm
    /// (NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 §2.3).
    #[error(
        "tmux mirror did not open for {target}: pipe-pane exited 0 but pane_pipe={pane_pipe} \
         (expected 1). The mirror would have reported success and produced no output."
    )]
    MirrorNotOpen { target: String, pane_pipe: String },
    /// `tmux resize-window` exited 0 but the pane geometry does not match the
    /// requested size. Returned rather than reporting success for a resize that
    /// did nothing — the same silent-success defect class as [`PtyError::MirrorNotOpen`]
    /// (NIMBUS-2026-10-08-pty-resize-window-v1 §2). `resize-pane -x/-y` on a
    /// single-pane window is precisely this case: exit 0, no effect.
    #[error(
        "tmux resize did not take for {target}: requested {want_cols}x{want_rows} but pane is \
         {got_cols}x{got_rows} after resize-window. A zero exit from resize does not mean the \
         pane was sized."
    )]
    ResizeNotApplied {
        target: String,
        want_cols: u16,
        want_rows: u16,
        got_cols: String,
        got_rows: String,
    },
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
        let Some(created) = parts.next() else {
            continue;
        };
        let Some(attached) = parts.next() else {
            continue;
        };
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
    ///   2. Run `tmux pipe-pane -t <name>:0 'cat >> <fifo>'` so tmux
    ///      appends pane output to the FIFO (and keeps doing so even if
    ///      SeatLoom dies — R3 failure isolation). Deliberately **no `-o`**:
    ///      that flag toggles, so a second attach would close the pipe.
    ///   3. Verify `#{pane_pipe}` is `1`; a zero exit from `pipe-pane` does
    ///      not mean the pipe opened. Returns [`PtyError::MirrorNotOpen`]
    ///      rather than a session that looks live and emits nothing.
    ///   4. Spawn a std::thread that opens the FIFO read-side, tail-reads
    ///      it, and broadcasts `PtyEvent::Output` chunks.
    ///
    /// **Re-attach is safe and idempotent.** Calling this twice on the same
    /// pane replaces the pipe and leaves the mirror open.
    ///
    /// `opts.rows` / `opts.cols` are currently informational; callers that
    /// want to resize the pane should invoke `PtySession::resize`, which maps
    /// to `tmux resize-window` (not `resize-pane`, which is a silent no-op on
    /// single-pane windows — NIMBUS-2026-10-08-pty-resize-window-v1 §1).
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

        // Query real pane dimensions before pipe-pane (§1.1 fix).
        let dims_out = std::process::Command::new("tmux")
            .args([
                "display-message",
                "-t",
                &tmux_target,
                "-p",
                "#{pane_width} #{pane_height}",
            ])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux display-message: {e}")))?;
        if !dims_out.status.success() {
            return Err(PtyError::Tmux(format!(
                "tmux display-message {tmux_target}: {}",
                String::from_utf8_lossy(&dims_out.stderr)
            )));
        }
        let dims_str = String::from_utf8_lossy(&dims_out.stdout);
        let dims_parts: Vec<&str> = dims_str.split_whitespace().collect();
        let (pane_cols, pane_rows) = if dims_parts.len() == 2 {
            let cols = dims_parts[0].parse::<u16>().unwrap_or(120);
            let rows = dims_parts[1].parse::<u16>().unwrap_or(30);
            (cols, rows)
        } else {
            // Fallback if parse fails (shouldn't happen with valid tmux).
            (120, 30)
        };

        // Ask tmux to copy pane output to our FIFO.
        //
        // NO `-o`. An earlier comment here claimed `-o` "makes the pipe-pane
        // command invoke only once (so we don't duplicate if attach is called
        // twice)". That reading is wrong: `-o` TOGGLES. Measured on a throwaway
        // tmux session (tmux 3.x), reading `#{pane_pipe}` at each step:
        //
        //     initial pane_pipe  = 0
        //     after 1st  -o call = 1
        //     after 2nd  -o call = 0      <- the pipe CLOSED
        //     after plain (no -o)= 1
        //     after 2nd plain    = 1      <- replaces, stays open
        //
        // So with `-o` the SECOND attach silently killed the mirror: pane_pipe=0,
        // zero bytes, and attach still returned a successful session. Any page
        // reload, second viewer, or already-mirrored seat hit it. Plain
        // `pipe-pane <cmd>` replaces an existing pipe unconditionally, which is
        // exactly re-attach semantics.
        //
        // Do not "restore" `-o`. Reported by Mira from UM0, reproduced by Lyra
        // and again here; see NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 §1.
        //
        // Shell-quoting the fifo path guards against unusual chars.
        let cat_cmd = format!("cat >> {}", shell_quote(&fifo_path.to_string_lossy()));
        let out = std::process::Command::new("tmux")
            .args(["pipe-pane", "-t", &tmux_target, &cat_cmd])
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

        // A zero exit from `pipe-pane` does NOT mean the pipe is open — that is
        // precisely how the `-o` bug stayed invisible: the command "succeeded"
        // while leaving pane_pipe=0, and attach reported a live session that
        // produced no bytes forever. Verify the flag and fail loudly instead.
        let pipe_state = std::process::Command::new("tmux")
            .args(["display-message", "-t", &tmux_target, "-p", "#{pane_pipe}"])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux display-message: {e}")))?;
        let pane_pipe_raw = String::from_utf8_lossy(&pipe_state.stdout);
        if let Err(e) = verify_pipe_open(&tmux_target, pipe_state.status.success(), &pane_pipe_raw)
        {
            // Stop any half-open pipe, then unlink the FIFO, so a failed attach
            // leaves no named pipe and no dangling tmux redirection behind.
            let _ = std::process::Command::new("tmux")
                .args(["pipe-pane", "-t", &tmux_target])
                .output();
            let _ = std::fs::remove_file(&fifo_path);
            return Err(e);
        }

        // Capture historical pane content immediately after pipe-pane (§1.3 fix).
        // `-e` preserves escape sequences so colors/attributes survive; lines
        // are joined with `\n`, so we rewrite to `\r\n` for xterm.
        let capture_out = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", &tmux_target, "-e", "-p"])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux capture-pane: {e}")))?;
        let historical_content = if capture_out.status.success() {
            // Normalize LF → CRLF for xterm row advancement.
            let mut out = Vec::with_capacity(capture_out.stdout.len() + 32);
            for &b in &capture_out.stdout {
                if b == b'\n' {
                    out.push(b'\r');
                }
                out.push(b);
            }
            out
        } else {
            // If capture-pane fails, treat as empty (pane might be genuinely empty).
            Vec::new()
        };

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
            pane_rows,
            pane_cols,
            initial_snapshot: historical_content,
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
                "load-buffer",
                "-b",
                "seatloom",
                "-",
                ";",
                "paste-buffer",
                "-b",
                "seatloom",
                "-t",
                &self.tmux_target,
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

    /// Resize the mirrored terminal to `rows` x `cols`.
    ///
    /// Maps to `tmux resize-window -t <session>:<window> -x <cols> -y <rows>`,
    /// **not** `resize-pane`. Measured on tmux 3.6a: `resize-pane -x/-y` exits 0
    /// and silently changes nothing on a single-pane window — which is what
    /// every seat session is — while `resize-window -x/-y` actually sizes the
    /// pane (and is not clamped at small sizes). Using `resize-pane` here made
    /// `cmd_pty_resize` a silent no-op: the pane stayed frozen at its launch
    /// geometry while the xterm viewport moved, which is the garbled-mirror
    /// misalignment (NIMBUS-2026-10-08-pty-resize-window-v1 §1).
    ///
    /// A zero exit is **not** evidence the resize took — §1 is exactly a case
    /// where exit 0 meant nothing happened. After resizing, the pane geometry
    /// is read back via `#{pane_width}`/`#{pane_height}` and
    /// [`PtyError::ResizeNotApplied`] is returned on mismatch, mirroring the
    /// `verify_pipe_open` pattern: a silent dead resize is the user harm.
    ///
    /// Targets the **window** index of `self.tmux_target` (`<session>:0`), so a
    /// multi-pane window is sized as a whole rather than silently breaking the
    /// contract on one pane.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        let out = std::process::Command::new("tmux")
            .args([
                "resize-window",
                "-t",
                &self.tmux_target,
                "-x",
                &cols.to_string(),
                "-y",
                &rows.to_string(),
            ])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux resize-window: {e}")))?;
        if !out.status.success() {
            return Err(PtyError::Tmux(format!(
                "tmux resize-window {}: {}",
                self.tmux_target,
                String::from_utf8_lossy(&out.stderr)
            )));
        }

        // Verify the resize actually took — the exit code above is not evidence
        // (resize-pane exits 0 while doing nothing; resize-window can also be
        // bounded by the tmux server). Read back the pane geometry and compare.
        let geo = std::process::Command::new("tmux")
            .args([
                "display-message",
                "-t",
                &self.tmux_target,
                "-p",
                "#{pane_width} #{pane_height}",
            ])
            .output()
            .map_err(|e| PtyError::Tmux(format!("spawn tmux display-message: {e}")))?;
        verify_resize_applied(
            &self.tmux_target,
            cols,
            rows,
            geo.status.success(),
            &String::from_utf8_lossy(&geo.stdout),
        )
    }

    /// Detach the mirror: stop the tmux pipe-pane, let the tail-reader hit
    /// EOF, remove the FIFO. **Does NOT kill the tmux session itself** — R3
    /// failure isolation. Safe to call multiple times.
    ///
    /// # Known limitation: the pipe is per-pane, not per-client
    ///
    /// `pipe-pane` with no shell-command unconditionally disables output
    /// copying for the pane. tmux keeps **one** pipe per pane, not one per
    /// consumer, so if two SeatLoom clients mirror the same pane, whichever
    /// detaches first **stops the mirror for both**. The survivor keeps a
    /// `PtySession` whose reader simply goes quiet — the same silent-dead-mirror
    /// shape that [`PtyError::MirrorNotOpen`] now catches at attach time, but
    /// arriving later, so attach-time verification cannot detect it.
    ///
    /// This is a property of tmux, not a bug in this function: disabling is the
    /// correct behaviour for a single-client detach, and no per-client pipe
    /// exists to disable instead. Fixing it needs an ownership model
    /// (ref-counting attachers, or per-client FIFOs fanned out from one pipe),
    /// which is deliberately **out of scope** for
    /// NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 §3 and is recommended as a
    /// follow-up packet in that delivery doc.
    pub fn kill(&self) -> Result<(), PtyError> {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Ask tmux to stop copying pane output. Passing no shell-command
        // disables copying unconditionally — correct for detach, and NOT a
        // toggle (unlike `-o`, which is why the attach path must not use it).
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

/// Decide whether `tmux pipe-pane` actually opened the pipe.
///
/// Split out as a pure function so the FAILURE path is unit-testable: reaching
/// it through `attach_tmux` needs a live tmux pane whose pipe command fails
/// while `pipe-pane` itself exits 0, which cannot be synthesised reliably in a
/// test. See NIMBUS-2026-09-30-pipe-pane-reattach-fix-delivery-v1 §5 PROOF 4.
///
/// # What this catches, and what it does not
///
/// It catches a pipe that **never opened**: `pipe-pane` exits 0 but the pane is
/// not piped (empty command, or a target tmux accepted without piping).
///
/// It does **not** catch a pipe whose command starts and then dies, because
/// tmux keeps `pane_pipe=1` until it first tries to WRITE to the dead pipe.
/// Measured on tmux 3.x:
///
/// ```text
/// $ tmux pipe-pane -t <s> "cat >> /nonexistent-dir-xyz/x.fifo"
///   +300ms pane_pipe=1      <- child already dead, flag still 1
///   +900ms pane_pipe=1
/// $ tmux send-keys -t <s> "echo hello" Enter     # forces a write
///   +300ms pane_pipe=0      <- only NOW does tmux clear it
/// ```
///
/// So the attach-time probe runs too early to see that case, and closing the
/// gap would mean writing to the user's pane to force a flush — unacceptable on
/// a live seat. A dying pipe therefore still surfaces only as "no bytes
/// arriving", which is a liveness concern for a follow-up, recorded in the
/// delivery doc alongside the multi-client issue. This check is a strict
/// improvement over reporting success unconditionally, not a complete one.
fn verify_pipe_open(target: &str, status_ok: bool, pane_pipe_raw: &str) -> Result<(), PtyError> {
    let pane_pipe = pane_pipe_raw.trim();
    if status_ok && pane_pipe == "1" {
        return Ok(());
    }
    Err(PtyError::MirrorNotOpen {
        target: target.to_string(),
        pane_pipe: if !status_ok {
            "<display-message failed>".to_string()
        } else if pane_pipe.is_empty() {
            "<empty>".to_string()
        } else {
            pane_pipe.to_string()
        },
    })
}

/// Verify that a `resize-window` actually sized the pane. The exit code of the
/// resize command is not evidence — `resize-pane -x/-y` exits 0 while doing
/// nothing on a single-pane window, and even `resize-window` can be bounded by
/// the server — so we read back `#{pane_width} #{pane_height}` and compare.
///
/// `geo_raw` is the stdout of `display-message -p "#{pane_width} #{pane_height}"`:
/// two space-separated integers. Anything else (probe failed, unparsable) is a
/// mismatch, not a pass-through.
fn verify_resize_applied(
    target: &str,
    want_cols: u16,
    want_rows: u16,
    status_ok: bool,
    geo_raw: &str,
) -> Result<(), PtyError> {
    let mut parts = geo_raw.split_whitespace();
    let got_cols = parts.next().unwrap_or("");
    let got_rows = parts.next().unwrap_or("");
    if status_ok
        && got_cols.parse::<u16>() == Ok(want_cols)
        && got_rows.parse::<u16>() == Ok(want_rows)
    {
        return Ok(());
    }
    let describe = |s: &str| {
        if !status_ok {
            "<display-message failed>".to_string()
        } else if s.is_empty() {
            "<empty>".to_string()
        } else {
            s.to_string()
        }
    };
    Err(PtyError::ResizeNotApplied {
        target: target.to_string(),
        want_cols,
        want_rows,
        got_cols: describe(got_cols),
        got_rows: describe(got_rows),
    })
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
    use super::{verify_pipe_open, verify_resize_applied, PtyError};

    /// The only accepting case: display-message succeeded AND the pane reports
    /// pane_pipe=1.
    #[test]
    fn verify_pipe_open_accepts_only_an_open_pipe() {
        assert!(verify_pipe_open("s:0", true, "1").is_ok());
        assert!(
            verify_pipe_open("s:0", true, "1\n").is_ok(),
            "trailing newline is normal"
        );
        assert!(verify_pipe_open("s:0", true, " 1 ").is_ok());
    }

    /// THE case this check exists for: `pipe-pane` exits 0 while the pane is
    /// not piped. Before this check that produced a session which reported
    /// success and emitted nothing for its whole lifetime.
    #[test]
    fn verify_pipe_open_rejects_exit_zero_with_closed_pipe() {
        let err = verify_pipe_open("throwaway:0", true, "0")
            .expect_err("pane_pipe=0 must not be treated as an open mirror");
        match &err {
            PtyError::MirrorNotOpen { target, pane_pipe } => {
                assert_eq!(target, "throwaway:0");
                assert_eq!(pane_pipe, "0");
            }
            other => panic!("expected MirrorNotOpen, got {other:?}"),
        }
        let msg = err.to_string();
        assert!(msg.contains("throwaway:0"), "must name the target: {msg}");
        assert!(
            msg.contains("pane_pipe=0"),
            "must report what it saw: {msg}"
        );
    }

    /// Garbage or absent output must not be read as success either.
    #[test]
    fn verify_pipe_open_rejects_unreadable_pane_pipe() {
        for raw in ["", "   ", "\n", "banana", "2", "01"] {
            assert!(
                verify_pipe_open("s:0", true, raw).is_err(),
                "pane_pipe={raw:?} must not be accepted"
            );
        }
        match verify_pipe_open("s:0", true, "") {
            Err(PtyError::MirrorNotOpen { pane_pipe, .. }) => assert_eq!(pane_pipe, "<empty>"),
            other => panic!("expected MirrorNotOpen with <empty>, got {other:?}"),
        }
    }

    /// A failed `display-message` is also a failure, not a pass-through.
    #[test]
    fn verify_pipe_open_rejects_a_failed_probe_even_if_stdout_says_one() {
        match verify_pipe_open("s:0", false, "1") {
            Err(PtyError::MirrorNotOpen { pane_pipe, .. }) => {
                assert_eq!(pane_pipe, "<display-message failed>");
            }
            other => panic!("expected MirrorNotOpen, got {other:?}"),
        }
    }

    /// The only accepting case: the probe succeeded AND the pane geometry read
    /// back equals exactly what was requested. Whitespace/newline variants of
    /// `display-message -p "#{pane_width} #{pane_height}"` are normal.
    #[test]
    fn verify_resize_applied_accepts_exact_match() {
        assert!(verify_resize_applied("s:0", 120, 40, true, "120 40").is_ok());
        assert!(
            verify_resize_applied("s:0", 120, 40, true, "120 40\n").is_ok(),
            "trailing newline is normal"
        );
        assert!(
            verify_resize_applied("s:0", 80, 24, true, " 80 24 ").is_ok(),
            "leading/trailing spaces are normal"
        );
    }

    /// THE case this check exists for: the resize command exited 0 but the pane
    /// did not move — exactly what `resize-pane -x/-y` does on a single-pane
    /// window (measured on tmux 3.6a). A no-op must not be reported as success.
    #[test]
    fn verify_resize_applied_rejects_a_no_op_resize() {
        let err = verify_resize_applied("throwaway:0", 120, 40, true, "220 50")
            .expect_err("a pane that stayed at 220x50 must not be treated as resized to 120x40");
        match &err {
            PtyError::ResizeNotApplied {
                target,
                want_cols,
                want_rows,
                got_cols,
                got_rows,
            } => {
                assert_eq!(target, "throwaway:0");
                assert_eq!(*want_cols, 120);
                assert_eq!(*want_rows, 40);
                assert_eq!(got_cols, "220");
                assert_eq!(got_rows, "50");
            }
            other => panic!("expected ResizeNotApplied, got {other:?}"),
        }
        let msg = err.to_string();
        assert!(msg.contains("throwaway:0"), "must name the target: {msg}");
        assert!(msg.contains("120x40"), "must report what it wanted: {msg}");
        assert!(msg.contains("220x50"), "must report what it saw: {msg}");
    }

    /// A partial match (one axis right, one wrong) is still a mismatch.
    #[test]
    fn verify_resize_applied_rejects_partial_match() {
        assert!(
            verify_resize_applied("s:0", 120, 40, true, "120 50").is_err(),
            "height wrong must fail even though width is right"
        );
        assert!(
            verify_resize_applied("s:0", 120, 40, true, "100 40").is_err(),
            "width wrong must fail even though height is right"
        );
    }

    /// Unreadable or absent geometry must not be read as success.
    #[test]
    fn verify_resize_applied_rejects_unparsable_geometry() {
        for raw in ["", "   ", "\n", "banana", "120", "120 banana", "-1 40"] {
            assert!(
                verify_resize_applied("s:0", 120, 40, true, raw).is_err(),
                "geometry={raw:?} must not be accepted"
            );
        }
        match verify_resize_applied("s:0", 120, 40, true, "") {
            Err(PtyError::ResizeNotApplied {
                got_cols, got_rows, ..
            }) => {
                assert_eq!(got_cols, "<empty>");
                assert_eq!(got_rows, "<empty>");
            }
            other => panic!("expected ResizeNotApplied with <empty>, got {other:?}"),
        }
    }

    /// A failed `display-message` probe is a failure, not a pass-through —
    /// even if stdout happens to look like the requested size.
    #[test]
    fn verify_resize_applied_rejects_a_failed_probe() {
        match verify_resize_applied("s:0", 120, 40, false, "120 40") {
            Err(PtyError::ResizeNotApplied {
                got_cols, got_rows, ..
            }) => {
                assert_eq!(got_cols, "<display-message failed>");
                assert_eq!(got_rows, "<display-message failed>");
            }
            other => panic!("expected ResizeNotApplied, got {other:?}"),
        }
    }

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
            pane_rows: 30,
            pane_cols: 120,
            initial_snapshot: Vec::new(),
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
            pane_rows: 30,
            pane_cols: 120,
            initial_snapshot: Vec::new(),
        };
        // Empty slice must return Ok(()) without spawning tmux.
        assert!(sess.write(b"").is_ok());
    }
}
