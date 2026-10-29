// Session commands.
//
// Read commands (Phase 1) list DB-persisted session rows.
//
// Live commands (v0.0.1 A3) no longer spawn CLIs via portable-pty — per the
// tmux-mirror architecture (a5998c1), SeatLoom attaches to existing tmux
// sessions. Each live session emits two Tauri event channels:
//   - "session:output" — payload { sessionId, data (base64) } — bytes
//     mirrored from `tmux pipe-pane` into /tmp/seatloom-mirror/<id>.fifo
//   - "session:exit"   — payload { sessionId, exitCode, signal } — emitted
//     when the pipe-pane tail closes (e.g., kill_session detaches the mirror)
//
// R3 failure isolation: SeatLoom is NEVER the parent process of tmux. Even if
// SeatLoom crashes, tmux continues running and user workflow is unaffected.

use crate::dto::{CheckpointDto, SessionDto};
use crate::state::AppState;
use base64::Engine as _;
use seatloom_core::pty::{
    default_transcripts_dir, list_tmux_sessions, LaunchOptions, PtyEvent, PtySession,
    TmuxSessionInfo,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

// -----------------------------------------------------------------------------
// Read commands (Phase 1)
// -----------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_list_sessions(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<SessionDto>, String> {
    let pid = crate::commands::require_project_id("cmd_list_sessions", project_id)?;
    state
        .db
        .list_sessions_for_project(&pid)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_sessions_for_seat(
    state: State<'_, AppState>,
    seat_id: String,
) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions_for_seat(&seat_id)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

/// Project-mode session list. AD-013 v2 backend invariant — schema 008.
#[tauri::command]
pub async fn cmd_list_sessions_for_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions_for_project(&project_id)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints(limit.unwrap_or(50))
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints_for_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints_for_session(&session_id)
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}

// -----------------------------------------------------------------------------
// Live tmux-mirror commands (v0.0.1 A3)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachTmuxRequest {
    /// Seat to attribute the mirrored session to (optional).
    pub seat_id: Option<String>,
    /// tmux session name to mirror (e.g., "Lyra-po-seatloom").
    pub tmux_session_name: String,
    /// Optional explicit SeatLoom session id; generated if absent.
    pub session_id: Option<String>,
    /// Initial pane size (used by `tmux resize-pane`).
    pub rows: Option<u16>,
    pub cols: Option<u16>,
    /// How many lines of pane scrollback to backfill into the mirror at attach
    /// time. `None` uses `seatloom_core::pty::DEFAULT_BACKFILL_LINES`. Bounded by
    /// the pane's tmux `history-limit`, so asking for more silently returns less.
    #[serde(default)]
    pub backfill_lines: Option<usize>,
    /// The tmux `history-limit` to ask for. `None` uses
    /// `seatloom_core::pty::DESIRED_HISTORY_LIMIT`. Affects panes created after
    /// it is set — an already-running pane keeps the limit it was born with.
    #[serde(default)]
    pub history_limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSessionDto {
    pub id: String,
    pub seat_id: Option<String>,
    pub runtime: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub transcript_path: String,
    /// tmux session name being mirrored (empty for legacy callers).
    #[serde(default)]
    pub tmux_session_name: String,
    /// Path to the pipe-pane FIFO (empty for legacy callers).
    #[serde(default)]
    pub fifo_path: String,
    /// Actual tmux pane dimensions at attach time (queried via display-message).
    #[serde(default)]
    pub pane_rows: u16,
    #[serde(default)]
    pub pane_cols: u16,
    /// Base64-encoded snapshot of the tmux pane at attach time (capture-pane).
    /// Frontend writes this to xterm before subscribing to session:output events
    /// so idle panes don't render as a black screen.
    #[serde(default)]
    pub initial_snapshot_b64: String,
    /// tmux `#{pane_pid}` at attach time. Shown in the terminal header so a
    /// cross-wired mirror is visible by eye instead of by reading a 181 MB log.
    /// 0 when tmux did not answer (never fabricated as a plausible number).
    #[serde(default)]
    pub pane_pid: i64,
    /// tmux `#{pane_tty}` at attach time, e.g. `/dev/ttys005`.
    #[serde(default)]
    pub pane_tty: String,
    /// The pane's own `history-limit`, read from tmux at attach time.
    ///
    /// WHY THIS IS REPORTED AT ALL. tmux reads `history-limit` when a PANE IS
    /// CREATED, so raising the global option does not reach panes that already
    /// exist — measured 2026-10-29: the global is 50 000 and all nine seat panes
    /// carry 2 000, because every one of them was born before the option was
    /// raised. `set-option -p` on a live pane is ACCEPTED and changes nothing
    /// (re-measured: still 2 000 afterwards); only a new pane inherits the global.
    ///
    /// So the app's backfill asks tmux for 5 000 lines and silently gets ~2 000,
    /// and the scrollbar implies a depth the pane does not have. Reporting the
    /// real number lets the interface say what is true instead of implying more.
    ///
    /// `null` when tmux would not answer — never fabricated as a plausible number.
    ///
    /// WHY Option AND NOT 0. The Rust reader returns Option and its doc promises a
    /// None is never turned into a number. This DTO then did `.unwrap_or(0)` and the
    /// UI printed it into a sentence, so a transient `display-message` failure told
    /// the operator "this pane's history-limit is 0 lines" — an unknown stated as a
    /// measurement, which is the same lie shape this whole unit removes. Flux
    /// proved the collision: `None` and `Some(0)` both arrived at the UI as 0.
    ///
    /// Carrying the Option through lets the interface decline to state a number
    /// instead of inventing one. A limit of 0 is still expressible (Some(0)); it is
    /// just no longer confusable with "we do not know".
    #[serde(default)]
    pub pane_history_limit: Option<i64>,
    /// How many lines of scrollback the attach actually seeded, counted from the
    /// capture rather than from the request.
    ///
    /// The distinction is the whole point: `backfill_lines` is what was ASKED
    /// for, and this is what ARRIVED. A seed that reports its own size cannot
    /// pretend to be deeper than it is.
    #[serde(default)]
    pub seeded_scrollback_lines: usize,
    /// Whether the seeded scrollback is a BACKFILL rather than a recording.
    ///
    /// Always true today (the seed comes from `capture-pane`), and carried
    /// explicitly so the interface can label it. A backfilled history that looks
    /// identical to a recorded one is the same class of lie as a mockup — which
    /// is why the dispatch says a seed must be marked as a seed.
    #[serde(default = "yes")]
    pub seeded_is_backfill: bool,
}

fn yes() -> bool {
    true
}

/// The event the frontend gets when a mirror stops being provably ours.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorLostPayload {
    pub session_id: String,
    pub seat_id: Option<String>,
    pub tmux_session_name: String,
    /// `lost` | `cross_wired` | `probe_failed` — see `MirrorVerdict::as_str`.
    pub reason: String,
    /// Foreign writer count (cross_wired only).
    pub writers: usize,
    /// Operator-facing sentence. Says what is true and what to do, not what the
    /// code believes.
    pub detail: String,
}

/// Build the operator-facing sentence for a verdict.
///
/// Written out rather than formatted from a template because the two broken
/// states need different actions, and "mirror lost" for a cross-wire would send
/// the operator looking for a dead pane that is in fact very much alive.
fn mirror_lost_detail(
    verdict: seatloom_core::pty::pipe_guard::MirrorVerdict,
    session_name: &str,
) -> String {
    use seatloom_core::pty::pipe_guard::MirrorVerdict;
    match verdict {
        MirrorVerdict::Lost => format!(
            "镜像已断开：没有任何进程在写入 {session_name} 的 FIFO。可能有人执行了不带参数的 \
             `tmux pipe-pane`（那是停止，不是查询），或我们的写入端被替换掉了。请重新接入。"
        ),
        MirrorVerdict::CrossWired { writers } => format!(
            "镜像已断开：有 {writers} 个进程在写入 {session_name} 的 FIFO，这个 slot 显示的内容\
             已不可信（可能混入了别的 pane）。请重新接入。"
        ),
        MirrorVerdict::ProbeFailed => format!(
            "镜像校验失败：无法读取进程表来证明 {session_name} 的 pipe 仍归本 app 所有。\
             为避免显示陌生 pane 的内容，已停止显示。请重新接入。"
        ),
        MirrorVerdict::Owned => String::new(),
    }
}

/// `#{pane_pid}` and `#{pane_tty}` for a target, or (0, "") if tmux will not
/// say. Never guessed: a made-up pid would be indistinguishable from a real one,
/// which is exactly the "looks like data" failure the rest of this codebase
/// refuses.
fn query_pane_identity(target: &str) -> (i64, String) {
    let out = std::process::Command::new("tmux")
        .args([
            "display-message",
            "-t",
            target,
            "-p",
            "#{pane_pid} #{pane_tty}",
        ])
        .output();
    let Ok(out) = out else {
        return (0, String::new());
    };
    if !out.status.success() {
        return (0, String::new());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.split_whitespace();
    let pid = parts
        .next()
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(0);
    let tty = parts.next().unwrap_or("").to_string();
    (pid, tty)
}

/// How many lines a `capture-pane -e -p` snapshot actually contains.
///
/// Counted, not derived from the request. tmux answers `min(asked, pane history)`
/// plus the visible pane, so the number that ARRIVED is the only one that
/// describes what the operator can scroll. An empty snapshot is 0 rather than an
/// error, because a pane with nothing in it has no history to seed.
fn count_captured_lines(snapshot: &[u8]) -> usize {
    if snapshot.is_empty() {
        return 0;
    }
    // `lines()` counts a final line with no trailing newline, which is exactly
    // what capture-pane produces, so no special case is needed.
    String::from_utf8_lossy(snapshot).lines().count()
}

/// `#{pane_pipe}` for a target: "1", "0", or None if tmux would not answer.
fn tmux_pane_pipe(target: &str) -> Option<String> {
    let out = std::process::Command::new("tmux")
        .args(["display-message", "-t", target, "-p", "#{pane_pipe}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn new_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ses-{nanos:016x}")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputEventPayload {
    session_id: String,
    data: String, // base64-encoded raw bytes — preserves ANSI/UTF-8 fidelity
}

/// One event on the turn-aware channel. A SECOND VIEW of the same bytes, not a
/// second capture: `session:output` stays exactly as it is and xterm keeps reading
/// it. This is the shape a "conversation window" needs (turn boundaries), and it
/// deliberately carries no claim about what the model DID — see
/// `seatloom_core::turn_stream`'s module header on why a boundary is a heuristic.
///
/// `chunk` is the same base64 payload `session:output` carries, byte-for-byte.
/// `frame` marks a synchronized-output redraw boundary, so a UI can batch without
/// having to re-derive it from escape sequences.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum TurnStreamPayload {
    /// A turn began (the harness's working indicator appeared).
    TurnStarted { session_id: String },
    /// Bytes belonging to the turn in progress.
    Chunk {
        session_id: String,
        data: String,
        frame: bool,
    },
    /// The turn ended (the working indicator was replaced by the status line).
    TurnEnded { session_id: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExitEventPayload {
    session_id: String,
    exit_code: Option<i32>,
    signal: Option<String>,
}

/// Discover tmux sessions whose name ends in `-seatloom`. Returns an empty
/// list if tmux is not running or no matching sessions exist.
#[tauri::command]
pub async fn cmd_list_tmux_sessions() -> Result<Vec<TmuxSessionInfo>, String> {
    list_tmux_sessions().map_err(|e| e.to_string())
}

/// Attach to an existing tmux session and begin mirroring its first pane into
/// a FIFO under /tmp/seatloom-mirror/. Emits `session:output` events for each
/// chunk of bytes read from the FIFO.
///
/// R3 rule: SeatLoom never becomes the parent of tmux. If SeatLoom crashes,
/// tmux continues running and the FIFO is re-attachable on restart.
#[tauri::command]
pub async fn cmd_attach_tmux_session(
    state: State<'_, AppState>,
    app: AppHandle,
    request: AttachTmuxRequest,
) -> Result<LiveSessionDto, String> {
    let id = request.session_id.unwrap_or_else(new_session_id);
    let transcripts_dir = default_transcripts_dir(&state.repo_root);

    let opts = LaunchOptions {
        command: "tmux".to_string(),
        args: vec!["pipe-pane".to_string(), request.tmux_session_name.clone()],
        working_dir: state.repo_root.clone(),
        rows: request.rows.unwrap_or(30),
        cols: request.cols.unwrap_or(120),
        env: LaunchOptions::default_env(),
        transcripts_dir,
        // Forwarded verbatim; `None` means "use the documented default", which
        // `attach_tmux` resolves. An omitted field is never rewritten to a number
        // here, so the default has exactly one home.
        backfill_lines: request.backfill_lines,
        history_limit: request.history_limit,
    };

    let session = PtySession::attach_tmux(&id, &request.tmux_session_name, opts)
        .map_err(|e| e.to_string())?;

    // Who is actually running in this pane? Asked here, before anything can go
    // wrong, because after a theft this is the fact that lets a human SEE the
    // mismatch: two slots with two different `pane_tty` values cannot both be
    // showing the same pane.
    //
    // A failure is NOT fabricated into a plausible-looking number -- it stays
    // 0 / empty, which the UI renders as "?" rather than as data.
    let (pane_pid, pane_tty) = query_pane_identity(&session.tmux_target);

    // -- R1: the mirror re-proves its own pipe, forever ------------------
    //
    // Why this cannot be folded into attach: `verify_pipe_open` runs once, at
    // the moment we ask tmux to pipe. By then the pipe is ours. The defect
    // this closes is that SOMEONE ELSE can take it later, and tmux's own
    // `#{pane_pipe}` stays 1 throughout (measured, both theft shapes), so
    // nothing tmux reports distinguishes a healthy mirror from a stolen one.
    //
    // What IS provable is which processes write our FIFO. So we re-check that
    // every HEALTH_PROBE_INTERVAL and, the moment it stops being exactly one
    // writer that is ours, we tell the frontend and stop pretending.
    //
    // `probe_sessions` is the shared session map: if the operator detached,
    // the session is gone by choice rather than stolen, and the loop exits.
    let probe_fifo = session.fifo_path.clone();
    let probe_target = session.tmux_target.clone();
    let probe_session_name = session.tmux_session_name.clone();
    let probe_seat = request.seat_id.clone();
    let probe_id = id.clone();
    let probe_sessions = state.sessions.clone();
    let app_for_probe = app.clone();
    tokio::spawn(async move {
        use seatloom_core::pty::pipe_guard::{self, MirrorVerdict};
        // Our own pids, taken once: the app does not fork writers, and this is
        // what keeps the probe's own subprocess from counting as a foreign
        // writer. Measured on the real machine -- without it, a healthy mirror
        // was reported CrossWired because the validating shell's command line
        // contained both `cat >>` and the fifo path.
        let self_pids = pipe_guard::own_pids();
        loop {
            tokio::time::sleep(pipe_guard::HEALTH_PROBE_INTERVAL).await;
            if !probe_sessions.lock().await.contains_key(&probe_id) {
                break; // detached on purpose
            }
            let pane_pipe_ok = tmux_pane_pipe(&probe_target).as_deref() == Some("1");
            let verdict = pipe_guard::probe_with_snapshot(
                &probe_fifo.to_string_lossy(),
                pane_pipe_ok,
                &pipe_guard::snapshot_processes(),
                &self_pids,
            );
            if !verdict.is_broken() {
                continue;
            }
            let payload = MirrorLostPayload {
                session_id: probe_id.clone(),
                seat_id: probe_seat.clone(),
                tmux_session_name: probe_session_name.clone(),
                reason: verdict.as_str().to_string(),
                writers: match verdict {
                    MirrorVerdict::CrossWired { writers } => writers,
                    _ => 0,
                },
                detail: mirror_lost_detail(verdict, &probe_session_name),
            };
            let _ = app_for_probe.emit("session:mirror-lost", payload);
            // One report is enough. The frontend removes the attachment (which
            // is what actually stops the display); a repeating alarm every 5 s
            // would be a different kind of noise, and the operator cannot act
            // on the second copy of the same fact.
            break;
        }
    });

    // Forward tmux-mirror events to the Tauri app as JSON events.
    let mut rx = session.subscribe();
    let app_for_events = app.clone();
    let id_for_events = id.clone();
    let sessions_for_exit = state.sessions.clone();
    tokio::spawn(async move {
        // The turn detector is a SECOND CONSUMER of the same bytes, not a second
        // capture path. `session:output` below is unchanged and xterm keeps reading
        // it; this only adds a shape on top. If the detector is wrong about a
        // boundary, the terminal is still right — which is why it is additive and
        // why nothing here can block or drop the raw stream.
        let mut turn = seatloom_core::turn_stream::TurnDetector::new();
        loop {
            match rx.recv().await {
                Ok(PtyEvent::Output(bytes)) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let payload = OutputEventPayload {
                        session_id: id_for_events.clone(),
                        data: encoded.clone(),
                    };
                    let _ = app_for_events.emit("session:output", payload);

                    // Turn-aware view. Emitted on its own event name so a consumer
                    // that only wants the terminal is not forced to filter these.
                    for event in turn.feed(&bytes) {
                        use seatloom_core::turn_stream::TurnEvent;
                        let turn_payload = match event {
                            TurnEvent::TurnStarted => TurnStreamPayload::TurnStarted {
                                session_id: id_for_events.clone(),
                            },
                            TurnEvent::Chunk { bytes, frame } => TurnStreamPayload::Chunk {
                                session_id: id_for_events.clone(),
                                data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                                frame,
                            },
                            TurnEvent::TurnEnded => TurnStreamPayload::TurnEnded {
                                session_id: id_for_events.clone(),
                            },
                        };
                        let _ = app_for_events.emit("session:turn", turn_payload);
                    }
                }
                Ok(PtyEvent::Exited { exit_code, signal }) => {
                    let payload = ExitEventPayload {
                        session_id: id_for_events.clone(),
                        exit_code,
                        signal,
                    };
                    let _ = app_for_events.emit("session:exit", payload);
                    let mut sessions = sessions_for_exit.lock().await;
                    sessions.remove(&id_for_events);
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let dto = LiveSessionDto {
        id: id.clone(),
        seat_id: request.seat_id.clone(),
        runtime: "tmux-mirror".to_string(),
        command: "tmux pipe-pane".to_string(),
        args: vec![request.tmux_session_name.clone()],
        working_dir: state.repo_root.display().to_string(),
        transcript_path: session.transcript_path.display().to_string(),
        tmux_session_name: session.tmux_session_name.clone(),
        fifo_path: session.fifo_path.display().to_string(),
        pane_rows: session.pane_rows,
        pane_cols: session.pane_cols,
        pane_pid,
        pane_tty,
        // Passed THROUGH as an Option. Do not `unwrap_or(0)` here: that is what made
        // an unknown render as a measured zero. See the field's doc.
        pane_history_limit: seatloom_core::pty::pane_history_limit(&session.tmux_target)
            .map(|v| v as i64),
        // Counted from what ARRIVED, not from what was asked for. The seed's own
        // size is the only honest number: `backfill_lines` is a request, and a
        // request that tmux silently truncates is not a measurement.
        seeded_scrollback_lines: count_captured_lines(&session.initial_snapshot),
        seeded_is_backfill: true,
        initial_snapshot_b64: base64::engine::general_purpose::STANDARD
            .encode(&session.initial_snapshot),
    };

    state.sessions.lock().await.insert(id.clone(), session);
    Ok(dto)
}

/// v0.0.1 transitional shim: kept so existing UI callers that still invoke
/// `cmd_launch_session({runtime:'tmux', command:<tmux_session_name>, ...})`
/// can reach `cmd_attach_tmux_session` without a breaking rename. B1
/// (v0.0.2) introduces true `tmux send-keys` write support at which point the
/// command surface may be further consolidated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub seat_id: Option<String>,
    pub runtime: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub rows: Option<u16>,
    pub cols: Option<u16>,
}

#[tauri::command]
pub async fn cmd_launch_session(
    state: State<'_, AppState>,
    app: AppHandle,
    request: LaunchRequest,
) -> Result<LiveSessionDto, String> {
    // The `command` field is reinterpreted as the tmux session name under the
    // v0.0.1 mirror architecture; the legacy portable-pty spawn path is gone.
    let tmux_session_name = if request.runtime == "tmux" || request.runtime == "tmux-mirror" {
        request.command.clone()
    } else {
        // For non-tmux runtimes, fail loudly — v0.0.1 has no other launch mode.
        return Err(format!(
            "cmd_launch_session: runtime {:?} no longer supported; use cmd_attach_tmux_session",
            request.runtime
        ));
    };
    let attach = AttachTmuxRequest {
        seat_id: request.seat_id,
        tmux_session_name,
        session_id: None,
        rows: request.rows,
        cols: request.cols,
        // The legacy shim carries no backfill preference, so the documented
        // default applies — never a number invented here.
        backfill_lines: None,
        history_limit: None,
    };
    cmd_attach_tmux_session(state, app, attach).await
}

/// v0.0.2 bidirectional write path: routes UTF-8 text into the mirrored tmux
/// pane via `PtySession::write` (tmux load-buffer|paste-buffer).
#[tauri::command]
pub async fn cmd_pty_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.write(data.as_bytes()).map_err(|e| e.to_string())
}

/// v0.0.2 bidirectional byte-level write path: routes raw bytes (including
/// NUL, 0x03 Ctrl-C, ANSI control sequences) through `PtySession::write`.
/// Used by xterm.js `onData` for arbitrary keystroke / paste payloads.
#[tauri::command]
pub async fn cmd_pty_write_bytes(
    state: State<'_, AppState>,
    session_id: String,
    bytes: Vec<u8>,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.write(&bytes).map_err(|e| e.to_string())
}

/// Resize the mirrored tmux terminal to `rows` x `cols`.
///
/// Maps to `tmux resize-window` (not `resize-pane`, which exits 0 while
/// silently doing nothing on a single-pane window — every seat session is
/// single-pane). The underlying `PtySession::resize` reads the pane geometry
/// back and returns an error if the resize did not take, so a success here
/// means the pane was actually sized
/// (NIMBUS-2026-10-08-pty-resize-window-v1).
#[tauri::command]
pub async fn cmd_pty_resize(
    state: State<'_, AppState>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.resize(rows, cols).map_err(|e| e.to_string())
}

/// Detach the tmux mirror for this session.
///
/// Stops the `tmux pipe-pane` copy, removes the FIFO, and drops the session
/// handle from the live registry. The tmux session itself continues running
/// — that is the R3 failure-isolation contract: SeatLoom observes, never owns.
#[tauri::command]
pub async fn cmd_kill_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    if let Some(sess) = sessions.remove(&session_id) {
        sess.kill().map_err(|e| e.to_string())?;
    }
    // R3: this is the one moment we know exactly which FIFOs are still wanted, so
    // it is also the one moment the orphans are cheap to identify. Measured leak:
    // 48 FIFOs for 9 live mirrors — the extras come from readers that exited
    // without reaching `kill()`.
    let live: Vec<std::path::PathBuf> = sessions
        .values()
        .map(|s| s.fifo_path().to_path_buf())
        .collect();
    let removed = seatloom_core::pty::reap_orphan_fifos(&live);
    if removed > 0 {
        eprintln!(
            "seatloom-pty: reaped {removed} orphan mirror FIFO(s); {} live remain",
            live.len()
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn cmd_list_live_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let sessions = state.sessions.lock().await;
    Ok(sessions.keys().cloned().collect())
}
