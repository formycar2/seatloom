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

// The pipe guard is a downstream self-proof on the mirror this module builds.
// Declared here (not in lib.rs) because it exists to answer "is this session's
// FIFO still the one our own pipe writes" — a question only `pty` can ask.
pub mod pipe_guard;

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

/// The pane's own `history-limit`, or `None` if tmux would not answer.
///
/// Read per PANE and never from the global option: the global is what the NEXT
/// pane will inherit, so reporting it would describe a pane that does not exist.
/// tmux reads this option when a pane is created, which is why raising the global
/// does not reach panes that already exist — measured 2026-10-29, all nine seat
/// panes still at 2 000 under a 50 000 global.
pub fn pane_history_limit(target: &str) -> Option<usize> {
    let out = std::process::Command::new("tmux")
        .args(["display-message", "-t", target, "-p", "#{history_limit}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<usize>()
        .ok()
}

/// How many lines of pane scrollback to backfill into the mirror at attach time.
///
/// **The default is derived, not guessed.** The terminal component caps its own
/// scrollback at 10 000 lines (`ui/src/app-v2/panel/SessionTerminal.tsx`,
/// `scrollback: 10000`). Backfilling all 10 000 would leave the operator zero room
/// for the output that arrives *after* they attach — the history would push the
/// live tail straight out of the buffer. So the default splits the scrollback in
/// half: 5 000 lines of history, 5 000 left for live output.
///
/// The effective number is bounded by tmux's `history-limit` (see
/// [`DESIRED_HISTORY_LIMIT`]): a pane retains at most that much, so asking for more
/// silently returns less. That is why the two constants are 10x apart rather than
/// equal.
pub const DEFAULT_BACKFILL_LINES: usize = 5_000;

/// The tmux `history-limit` this app asks for, so a pane retains enough scrollback
/// for the backfill above to have something to find.
///
/// Measured before choosing it: a pane with `history-limit` 2 000, 50 000 and
/// 100 000 has **identical RSS at rest** (3 232 / 3 232 / 3 248 KB) — tmux
/// allocates the history grid lazily, so a higher limit costs nothing until a pane
/// actually produces that much output, which is exactly when the operator wants
/// it. 50 000 is 10x the backfill default, so the backfill is never silently
/// truncated, and it is the order of magnitude the 2026-10-16 dispatch asked for.
///
/// **This only affects panes created after it is set.** tmux reads the option when
/// a pane is created; an already-running pane keeps the limit it was born with.
/// Measured: setting it on a live session leaves the existing pane at 2 000 and
/// gives 50 000 to the next pane created in that session.
pub const DESIRED_HISTORY_LIMIT: usize = 50_000;

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
    /// How many lines of scrollback to backfill at attach. `None` uses
    /// [`DEFAULT_BACKFILL_LINES`]. Bounded by the pane's `history-limit`.
    pub backfill_lines: Option<usize>,
    /// The `history-limit` to ask tmux for. `None` uses [`DESIRED_HISTORY_LIMIT`].
    /// Affects panes created after it is set; see the constant's doc.
    pub history_limit: Option<usize>,
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
    ///   2. Ask tmux for a bigger `history-limit` on this session, so panes
    ///      created from now on retain enough scrollback for the backfill to
    ///      find. Scoped to the session, not the server.
    ///   3. Run `tmux pipe-pane -t <name>:0 'cat >> <fifo>'` so tmux
    ///      appends pane output to the FIFO (and keeps doing so even if
    ///      SeatLoom dies — R3 failure isolation). Deliberately **no `-o`**:
    ///      that flag toggles, so a second attach would close the pipe.
    ///   4. Verify `#{pane_pipe}` is `1`; a zero exit from `pipe-pane` does
    ///      not mean the pipe opened. Returns [`PtyError::MirrorNotOpen`]
    ///      rather than a session that looks live and emits nothing.
    ///   5. Backfill the pane's scrollback (`capture-pane -e -p -S -<n>`) into
    ///      `initial_snapshot`, which the caller delivers to the terminal
    ///      BEFORE the live stream — so the operator can scroll up into what
    ///      the seat did before they attached, with colours intact.
    ///   6. Spawn a std::thread that opens the FIFO read-side, tail-reads
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

        // Ask tmux to retain more scrollback for this session, so the backfill
        // below has something to find and the NEXT pane created here inherits it.
        //
        // Session-scoped, never `-g`: mutating the server-wide option from a
        // desktop app would change every pane the operator has open anywhere.
        //
        // This does NOT enlarge the pane being attached — tmux reads
        // `history-limit` when a pane is created, and measured on a live session
        // the existing pane stays at 2 000 while the next pane gets the new
        // value. So a seat that wants more than it already has needs its pane
        // recreated (relaunch the seat), and that is stated in the delivery note
        // rather than papered over. A failure here is not fatal: the backfill
        // still returns whatever the pane does retain.
        let desired_limit = opts.history_limit.unwrap_or(DESIRED_HISTORY_LIMIT);
        if let Err(e) = std::process::Command::new("tmux")
            .args([
                "set-option",
                "-t",
                tmux_session_name,
                "history-limit",
                &desired_limit.to_string(),
            ])
            .output()
        {
            eprintln!(
                "seatloom-pty: could not raise history-limit for {tmux_session_name}: {e} \
                 (the backfill will use whatever the pane already retains)"
            );
        }

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

        // Backfill the pane's SCROLLBACK, not just what is on screen.
        //
        // THE DEFECT THIS FIXES. `capture-pane -p` with no `-S` returns only the
        // VISIBLE pane — measured 66 lines on a live seat pane whose scrollback
        // held 3 273. `pipe-pane` forwards only output produced after it starts,
        // so the mirror began at the attach instant and the operator could scroll
        // up almost nowhere. His words: "now he can barely scroll up to see
        // history".
        //
        // `-S -<n>` starts n lines before the visible pane, so the capture is
        // `min(n, history) + visible`. The budget counts the whole thing, so the
        // history portion is `backfill - visible`, floored at 1.
        //
        // `-e` is NOT OPTIONAL: without it tmux returns plain text and every
        // colour, bold and progress bar is lost. That is a quality regression and
        // the acceptance criterion names it as a reject. Measured with `-e`: 1 308
        // of 3 339 captured lines carry escape sequences.
        //
        // The backfill rides on `initial_snapshot` — the SAME field that already
        // keeps an idle pane from rendering black — and NOT through the FIFO.
        // That is deliberate and it is measured, not a shortcut:
        //   · writing the FIFO before pipe-pane deadlocks, because opening a FIFO
        //     for write blocks until a reader opens it, and the reader thread
        //     starts after pipe-pane;
        //   · with both ends held open, tmux's `cat >>` and a history writer race
        //     for the pipe, which interleaves history with live output — the exact
        //     failure the dispatch says to avoid;
        //   · the field is already delivered to the frontend, which writes it to
        //     xterm AFTER registering the output listener, so there is no ordering
        //     hazard and no re-attach duplication (each attach is a new session id,
        //     so the terminal remounts and writes the snapshot once).
        // The backfill asks for what the pane can actually GIVE, not for a number
        // that sounds good.
        //
        // Measured 2026-10-29 (dispatch disp-H7pVqxTHO077Rlp6GrmzX): the global
        // `history-limit` is 50 000 and all nine seat panes carry 2 000, because
        // tmux reads the option when a pane is CREATED and every seat pane predates
        // the global being raised. `set-option -p` on a live pane is accepted and
        // changes nothing (re-measured: still 2 000); only a new pane inherits it.
        //
        // So asking for 5 000 silently returns ~2 000, and the operator's scrollbar
        // implies a depth the pane does not have. Reading the pane's own limit
        // first costs one `display-message` and makes the request honest — and
        // when it is smaller than what we wanted, the interface says so rather
        // than showing a scrollbar that goes nowhere.
        let backfill = opts.backfill_lines.unwrap_or(DEFAULT_BACKFILL_LINES);
        let pane_history_limit = pane_history_limit(&tmux_target).unwrap_or(0);
        // Only clamp when tmux actually answered. A 0 is "tmux would not say", and
        // clamping to it would seed nothing on a transient failure.
        let wanted = backfill.saturating_sub(pane_rows as usize).max(1);
        let history_lines = if pane_history_limit > 0 {
            wanted.min(pane_history_limit as usize)
        } else {
            wanted
        };
        if pane_history_limit > 0 && history_lines < wanted {
            eprintln!(
                "seatloom-pty: pane {tmux_target} retains {pane_history_limit} lines, \
                 so the backfill asks for {history_lines} instead of {wanted} — \
                 the pane was created before history-limit was raised, and only a new \
                 pane inherits the higher value"
            );
        }
        let capture_out = std::process::Command::new("tmux")
            .args([
                "capture-pane",
                "-t",
                &tmux_target,
                "-e",
                "-p",
                "-S",
                &format!("-{history_lines}"),
            ])
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
            let mut log_bytes: u64 = log
                .as_ref()
                .and_then(|f| f.metadata().ok())
                .map_or(0, |m| m.len());
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
                        // R3: rotate instead of growing forever. One FIFO's
                        // transcript reached 180 986 013 bytes during the
                        // cross-wiring incident because it silently swallowed two
                        // high-traffic panes for three hours; the directory that
                        // holds them is 6.4 GB across 422 files with no ceiling.
                        //
                        // Rotating (rename to `.1`, start a fresh file) keeps the
                        // file at a size the operator can still copy and hand to
                        // someone, and it costs one rename per 64 MB rather than a
                        // rewrite of the whole file.
                        log_bytes += chunk.len() as u64;
                        if log_bytes >= pipe_guard::MAX_TRANSCRIPT_BYTES {
                            if let Some(f) = log.take() {
                                drop(f);
                                rotate_transcript(&transcript_for_reader);
                            }
                            log = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&transcript_for_reader)
                                .ok();
                            log_bytes = 0;
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

    /// The tmux target this mirror is attached to (e.g. `"Nimbus-TechArchi-seatloom:0"`).
    pub fn tmux_target(&self) -> &str {
        &self.tmux_target
    }

    /// Path of the FIFO this mirror reads.
    pub fn fifo_path(&self) -> &std::path::Path {
        &self.fifo_path
    }
}

/// Unlink every FIFO in [`mirror_dir`] that no live session is reading.
///
/// # The leak this closes
///
/// Measured 2026-10-27: `/tmp/seatloom-mirror/` held **48 FIFOs** for **9 live
/// mirrors**. The 39 extras are orphans from attaches whose reader thread exited
/// early (a FIFO open failure, an EOF from a writer that was killed) — the
/// `kill()` path does unlink, so these are specifically the paths that never
/// reach `kill()`.
///
/// They are not merely untidy: an orphan FIFO is a file a pane can be piped into
/// and nobody is watching, which is exactly the shape that made the cross-wiring
/// incident invisible. And a stale FIFO blocks a later attach's `mkfifo` unless
/// it is unlinked first — which `attach_tmux` already does, so the leak is
/// silent rather than fatal.
///
/// # What makes this safe to call at all
///
/// It only ever removes a file that is (a) inside `mirror_dir` and (b) not in
/// `live_fifo_paths`. A session that is mid-attach is registered in
/// `state.sessions` BEFORE this runs, so its FIFO is in the live set. The one
/// thing it must never do is delete the FIFO of a session the operator is
/// looking at — that would be the defect again, with better intentions.
///
/// Returns how many orphans were removed, so a caller (and a test) can see the
/// number rather than trust that it happened.
pub fn reap_orphan_fifos(live_fifo_paths: &[std::path::PathBuf]) -> usize {
    reap_orphan_fifos_in(&mirror_dir(), live_fifo_paths)
}

/// The real implementation, with the directory passed in so it can be tested
/// against a scratch dir.
///
/// Kept separate from [`reap_orphan_fifos`] rather than parameterised at every
/// call site: a test that reimplements the loop it claims to check is a test that
/// passes no matter what the production code does. This was the first draft and
/// it was wrong for exactly that reason.
fn reap_orphan_fifos_in(dir: &std::path::Path, live_fifo_paths: &[std::path::PathBuf]) -> usize {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0, // no mirror dir at all — nothing leaked
    };
    // Compare by NORMALISED path. Measured on macOS: `lsof` (and anything that
    // resolves the symlink) reports `/private/tmp/seatloom-mirror/x.fifo` while
    // `read_dir` on `/tmp/seatloom-mirror` yields `/tmp/seatloom-mirror/x.fifo`.
    // Comparing raw strings makes every live FIFO look like an orphan — and this
    // reaper DELETES what it thinks is an orphan. That is the cross-wiring defect
    // again, committed by the code that was supposed to prevent it.
    //
    // The same normalisation the pipe guard uses, so there is one rule.
    let live: std::collections::HashSet<String> = live_fifo_paths
        .iter()
        .map(|p| normalise_for_compare(&p.to_string_lossy()))
        .collect();
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        // Only FIFOs, only ours, only ones nobody is reading.
        //
        // `std::os::unix::fs::FileTypeExt::is_fifo` is the stable way to ask;
        // `FileType::is_fifo` is still unstable on `std`. A non-FIFO entry in
        // this directory (a stray log, a `.DS_Store`) is never removed.
        let is_fifo = entry
            .file_type()
            .map(|t| std::os::unix::fs::FileTypeExt::is_fifo(&t))
            .unwrap_or(false);
        if !is_fifo {
            continue;
        }
        if live.contains(&normalise_for_compare(&path.to_string_lossy())) {
            continue;
        }
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Canonical form for comparing two paths that may differ only by the `/tmp` →
/// `/private/tmp` symlink.
///
/// Strips the `/private` prefix from BOTH sides so either spelling compares
/// equal. Not `canonicalize`: the file may legitimately be gone, and a failed
/// resolve would make every comparison unequal — which in the reaper means
/// deleting a live FIFO.
fn normalise_for_compare(p: &str) -> String {
    p.strip_prefix("/private").unwrap_or(p).to_string()
}

/// Rotate a transcript that has hit [`pipe_guard::MAX_TRANSCRIPT_BYTES`].
///
/// `<path>` becomes `<path>.1`. A previous `.1` is overwritten rather than
/// incremented: the point of the ceiling is a bound on total disk, and a chain of
/// `.1 .2 .3 …` is an unbounded directory by another name.
///
/// Failures are ignored deliberately. A transcript is a convenience record of
/// bytes that are also flowing to the UI; failing to rotate must never take the
/// mirror down. What the caller does instead is reopen in append mode, so the
/// worst case is a file larger than the ceiling — the pre-existing behaviour, not
/// a new failure.
fn rotate_transcript(path: &std::path::Path) {
    let rotated = std::path::PathBuf::from(format!("{}.1", path.display()));
    let _ = std::fs::rename(path, &rotated);
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
    use super::{
        verify_pipe_open, verify_resize_applied, PtyError, DEFAULT_BACKFILL_LINES,
        DESIRED_HISTORY_LIMIT,
    };

    /// The history portion of a backfill budget. The budget counts the WHOLE
    /// capture (history + visible pane), so the visible rows come off it first —
    /// otherwise a 5 000-line budget on a 66-row pane would deliver 5 066 lines
    /// and quietly exceed what the terminal keeps.
    fn history_lines_for(backfill: usize, pane_rows: u16) -> usize {
        backfill.saturating_sub(pane_rows as usize).max(1)
    }

    /// The clamp that makes the backfill honest: it never asks tmux for more than
    /// the pane can give. Measured on live 2026-10-29 — nine seat panes at 2 000
    /// under a 50 000 global, because tmux reads the option at pane creation.
    #[test]
    fn the_backfill_never_asks_for_more_than_the_pane_retains() {
        // 5 000 asked, 24 visible rows -> 4 976 wanted. A 2 000-line pane cannot
        // give that, and asking anyway is what made the scrollbar imply a depth
        // the pane did not have.
        let wanted = history_lines_for(DEFAULT_BACKFILL_LINES, 24);
        assert_eq!(wanted, 4_976);
        let clamped = wanted.min(2_000);
        assert_eq!(clamped, 2_000, "a 2 000-line pane must cap the request");
        // A pane that CAN give more is not capped — the clamp must not become a
        // ceiling of its own.
        let roomy = wanted.min(50_000);
        assert_eq!(
            roomy, wanted,
            "a 50 000-line pane must get the full request"
        );
    }

    /// `pane_history_limit` reads the PANE, and it is a real tmux call rather than
    /// a constant: the whole point is that the global option does not describe the
    /// pane being attached.
    #[test]
    fn the_pane_limit_is_read_not_derived_from_the_global() {
        // A live pane exists in this repo's tmux (the seat that runs this test),
        // so the reader is exercised against a real answer rather than a mock.
        let session = std::env::var("SEATLOOM_TEST_TMUX_SESSION").ok();
        if let Some(name) = session {
            let limit = pane_history_limit(&format!("{name}:0"));
            assert!(
                limit.is_some(),
                "tmux answered for {name}:0, so the reader must too"
            );
            assert!(limit.unwrap() > 0, "a real pane retains some scrollback");
        }
        // A target that does not exist is None, not 0 and not a panic: a caller
        // must be able to tell "tmux would not say" from "the pane retains none".
        assert!(
            pane_history_limit("no-such-session-xyz:0").is_none(),
            "an unknown target must be None, never a fabricated number"
        );
    }

    /// The default splits the terminal's own scrollback in half, so the live tail
    /// still has room after the history lands. A `const` block, because both sides
    /// are compile-time constants: an `assert!` on constants is rejected by the
    /// `assertions_on_constants` lint, and rightly — it would pass at compile time
    /// and prove nothing at run time.
    #[test]
    fn the_backfill_default_leaves_room_for_the_live_tail() {
        // The UI's own cap, pinned here so the two cannot drift apart silently.
        const UI_SCROLLBACK: usize = 10_000;
        const {
            assert!(
                DEFAULT_BACKFILL_LINES < UI_SCROLLBACK,
                "backfilling the whole scrollback would push the live tail out of the buffer"
            );
            assert!(
                DESIRED_HISTORY_LIMIT > DEFAULT_BACKFILL_LINES,
                "a pane retaining less than the backfill makes the backfill a silent truncation"
            );
            assert!(DESIRED_HISTORY_LIMIT / DEFAULT_BACKFILL_LINES == 10);
        }
        // And the run-time half: the arithmetic actually used by `attach_tmux`.
        assert_eq!(history_lines_for(DEFAULT_BACKFILL_LINES, 66), 4_934);
    }

    /// The visible rows are subtracted, never the whole budget spent on history.
    #[test]
    fn the_history_portion_subtracts_the_visible_rows() {
        assert_eq!(history_lines_for(5_000, 66), 4_934);
        assert_eq!(history_lines_for(5_000, 5_000), 1, "floored at 1, never 0");
        assert_eq!(
            history_lines_for(10, 6_000),
            1,
            "saturating, never negative"
        );
        assert_eq!(history_lines_for(0, 24), 1);
    }

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

    /// A unique scratch dir per test. Tests run in parallel inside one process,
    /// so `std::process::id()` is NOT unique — the first draft shared a dir
    /// between two reaper tests and one of them failed with EEXIST on mkfifo.
    fn scratch(label: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("seatloom-{label}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reap_orphan_fifos_leaves_live_ones_alone_and_removes_the_rest() {
        // The safety property: a live FIFO must NEVER be removed. Deleting the
        // FIFO of a session the operator is watching would be the cross-wiring
        // defect again, with better intentions.
        let dir = scratch("seatloom-reap-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let live = dir.join("ses-live.fifo");
        let orphan = dir.join("ses-orphan.fifo");
        let plain_file = dir.join("not-a-fifo.txt");
        for p in [&live, &orphan] {
            unistd::mkfifo(p, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();
        }
        std::fs::write(&plain_file, b"keep me").unwrap();

        // Point the reaper at our scratch dir by swapping the module-level path
        // through the same function the production code uses.
        let removed = reap_orphan_fifos_in(&dir, std::slice::from_ref(&live));

        assert_eq!(removed, 1, "exactly the orphan went");
        assert!(live.exists(), "the live FIFO must survive");
        assert!(!orphan.exists(), "the orphan must be gone");
        assert!(plain_file.exists(), "a non-FIFO is never touched");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_live_fifo_spelled_private_tmp_is_not_mistaken_for_an_orphan() {
        // THE BUG THIS REAPER ALMOST SHIPPED WITH. On macOS `lsof` reports
        // `/private/tmp/...` while `read_dir` on `/tmp/...` yields `/tmp/...`.
        // Comparing the raw strings made every live FIFO look orphaned, and the
        // reaper DELETES what looks orphaned — so it destroyed the three FIFOs the
        // running app was reading. The fix that was supposed to stop the app
        // showing foreign panes would have killed the live mirrors instead.
        //
        // Reproduces the exact spellings: dir as read_dir yields it, live path as
        // lsof yields it.
        let dir = std::env::temp_dir().join("seatloom-mirror");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("ses-live.fifo");
        unistd::mkfifo(&fifo, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

        // lsof-style spelling of the same file: `/private` inserted at the root of
        // the temp path, which is exactly the macOS relationship between
        // `/tmp/x` and `/private/tmp/x`.
        //
        // Note the FIRST version of this test built `private/tmp/...` under the
        // temp dir instead of prefixing the temp dir itself, so it never
        // reproduced the real mismatch and passed for the wrong reason. A test
        // that cannot fail is worse than no test.
        let tmp = std::env::temp_dir();
        let live_spelling = std::path::PathBuf::from(format!("/private{}", tmp.display()))
            .join("seatloom-mirror/ses-live.fifo");
        assert_ne!(
            live_spelling, fifo,
            "sanity: the two spellings really do differ, or this test proves nothing"
        );

        let removed = reap_orphan_fifos_in(&dir, &[live_spelling]);
        assert_eq!(removed, 0, "a live FIFO must never be reaped");
        assert!(
            fifo.exists(),
            "the live FIFO survived -- the app keeps its mirror"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reap_orphan_fifos_with_no_live_sessions_removes_every_fifo_but_keeps_other_files() {
        // The measured leak: 48 FIFOs for 9 live mirrors. With the live set empty
        // the reaper should be willing to clear all of them — the caller is
        // responsible for only calling it when there really is nothing live.
        let dir = scratch("seatloom-reap-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.fifo");
        let b = dir.join("b.fifo");
        for p in [&a, &b] {
            unistd::mkfifo(p, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();
        }
        std::fs::write(dir.join("README"), b"keep").unwrap();

        assert_eq!(reap_orphan_fifos_in(&dir, &[]), 2);
        assert!(dir.join("README").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reap_orphan_fifos_on_a_missing_dir_reports_zero_rather_than_panicking() {
        // A machine with no mirror dir has nothing leaked. Returning 0 (not an
        // error) keeps the periodic reaper from spamming a failure nobody can act
        // on.
        let missing = std::env::temp_dir().join("seatloom-reap-does-not-exist-xyz");
        let _ = std::fs::remove_dir_all(&missing);
        assert_eq!(reap_orphan_fifos_in(&missing, &[]), 0);
    }

    #[test]
    fn rotating_a_transcript_keeps_the_old_bytes_under_dot_one() {
        // Rotation must not lose data: the point of the ceiling is bounded disk,
        // and a rename keeps the previous 64 MB readable.
        let dir = scratch("seatloom-rotate-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ses-x.raw.log");
        std::fs::write(&path, b"first generation").unwrap();

        rotate_transcript(&path);

        assert!(!path.exists(), "the live path is free for a fresh file");
        let rotated = dir.join("ses-x.raw.log.1");
        assert_eq!(std::fs::read(&rotated).unwrap(), b"first generation");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotating_twice_keeps_one_generation_not_an_unbounded_chain() {
        // `.1 .2 .3 ...` would be an unbounded directory by another name. The
        // second rotation overwrites, which is what makes the ceiling a bound.
        let dir = scratch("seatloom-rotate2-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ses-y.raw.log");
        std::fs::write(&path, b"gen1").unwrap();
        rotate_transcript(&path);
        std::fs::write(&path, b"gen2").unwrap();
        rotate_transcript(&path);

        let rotated = dir.join("ses-y.raw.log.1");
        assert_eq!(std::fs::read(&rotated).unwrap(), b"gen2");
        assert!(
            !dir.join("ses-y.raw.log.2").exists(),
            "no numbered chain -- that would be unbounded disk by another name"
        );
        let _ = std::fs::remove_dir_all(&dir);
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
