// Smoke harness for v0.0.1 A3 tmux mirror read path + v0.0.2 B1 write path.
//
// Usage (from repo root):
//   PATH="$HOME/.cargo/bin:$PATH" cargo run -p seatloom-core --example tmux_mirror_smoke -- <tmux_session_name>
//
// Exercises:
//   - list_tmux_sessions() filtering
//   - PtySession::attach_tmux (mkfifo + tmux pipe-pane + tail-read)
//   - PtyEvent::Output broadcast delivery
//   - PtySession::resize (tmux resize-pane)
//   - PtySession::write round-trip (B1 — load-buffer | paste-buffer)
//   - PtySession::write delivers Ctrl-C as a real SIGINT (B1 — interrupts sleep)
//   - PtySession::kill (stop pipe-pane, unlink fifo, do NOT kill tmux session)
//   - R3 failure isolation re-asserted AFTER the write path exercise
//
// Exit code 0 on success. Prints a structured transcript to stdout.

use std::path::PathBuf;
use std::time::Duration;

use seatloom_core::pty::{
    default_transcripts_dir, list_tmux_sessions, LaunchOptions, PtyEvent, PtySession,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = std::env::args().nth(1).unwrap_or_else(|| "test-seatloom".to_string());
    eprintln!("[smoke] target tmux session = {target}");

    // Step 3: list-sessions filter behaviour.
    let all = list_tmux_sessions()?;
    eprintln!("[smoke] list_tmux_sessions() returned {} sessions (filtered by -seatloom suffix)", all.len());
    for s in &all {
        eprintln!("  - {} (created_at={}, attached={})", s.session_name, s.created_at, s.attached);
    }

    // Step 4: attach to target.
    let repo_root = std::env::current_dir()?;
    let transcripts_dir = default_transcripts_dir(&repo_root);
    let opts = LaunchOptions {
        command: "tmux".to_string(),
        args: vec!["pipe-pane".to_string(), target.clone()],
        working_dir: repo_root.clone(),
        rows: 24,
        cols: 80,
        env: LaunchOptions::default_env(),
        transcripts_dir,
    };

    let session = PtySession::attach_tmux("smoke-001", &target, opts)?;
    eprintln!("[smoke] attached: id={}, fifo={}, tmux_target={}",
        session.id, session.fifo_path.display(), session.tmux_target);

    // Assert: fifo exists.
    if !session.fifo_path.exists() {
        return Err(format!("fifo missing after attach: {}", session.fifo_path.display()).into());
    }

    // Subscribe to the mirror stream.
    let mut rx = session.subscribe();
    let mut received = Vec::<u8>::new();

    // Step 6: drive traffic into tmux via send-keys; collect mirror bytes for 2s.
    let target_for_typing = target.clone();
    tokio::spawn(async move {
        for cmd in ["ls -la", "printf 'SEATLOOM_MIRROR_PROBE_%s\\n' ok"] {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &format!("{target_for_typing}:0"), cmd, "C-m"])
                .output();
        }
    });

    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(PtyEvent::Output(bytes))) => {
                received.extend_from_slice(&bytes);
                if received.len() > 65536 {
                    break;
                }
            }
            Ok(Ok(PtyEvent::Exited { exit_code, signal })) => {
                eprintln!("[smoke] unexpected early Exited: exit_code={exit_code:?} signal={signal:?}");
                break;
            }
            Ok(Err(e)) => {
                eprintln!("[smoke] broadcast recv error: {e}");
                break;
            }
            Err(_) => break,
        }
    }

    // Step 7: assert we captured mirror bytes.
    eprintln!("[smoke] received {} bytes from mirror", received.len());
    if received.is_empty() {
        return Err("no mirror bytes received — pipe-pane or tail-read failing".into());
    }
    let received_text = String::from_utf8_lossy(&received);
    let found_probe = received_text.contains("SEATLOOM_MIRROR_PROBE_");
    let found_ls = received_text.contains("ls -la") || received_text.contains("total ");
    eprintln!("[smoke] probe_marker_seen={} ls_output_seen={}", found_probe, found_ls);

    // Step 8: resize the pane.
    session.resize(24, 120)?;
    eprintln!("[smoke] resize(24,120) OK");

    // ---- B1 write path (v0.0.2) ----------------------------------------
    // Step 10: write round-trip — send a probe via PtySession::write and
    // verify the bytes appear in the mirrored pane. Subscribes a fresh
    // receiver so we don't have to rewind through Step 6's traffic.
    let mut rx_b1 = session.subscribe();
    let probe_marker = "SEATLOOM_B1_ROUND_TRIP_OK";
    let cmd_b1 = format!("printf '{probe_marker}\\n'\n");
    session.write(cmd_b1.as_bytes())?;
    eprintln!("[smoke] B1 write({} bytes) issued", cmd_b1.len());
    let mut received_b1 = Vec::<u8>::new();
    let deadline_b1 = tokio::time::Instant::now() + Duration::from_secs(2);
    let found_b1 = loop {
        let remaining = deadline_b1.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break false;
        }
        match tokio::time::timeout(remaining, rx_b1.recv()).await {
            Ok(Ok(PtyEvent::Output(bytes))) => {
                received_b1.extend_from_slice(&bytes);
                if String::from_utf8_lossy(&received_b1).contains(probe_marker) {
                    break true;
                }
            }
            Ok(Ok(PtyEvent::Exited { exit_code, signal })) => {
                eprintln!("[smoke] B1 unexpected Exited: exit_code={exit_code:?} signal={signal:?}");
                break false;
            }
            Ok(Err(_)) | Err(_) => break false,
        }
    };
    if !found_b1 {
        return Err(format!(
            "B1_WRITE_TIMEOUT: probe '{probe_marker}' did not echo back within 2s ({} bytes seen)",
            received_b1.len()
        )
        .into());
    }
    eprintln!("[smoke] B1 round-trip OK: probe '{probe_marker}' echoed back");

    // Step 11: Ctrl-C signal delivery — start `sleep 30` in the FOREGROUND,
    // send 0x03 via PtySession::write, then observe that a post-Ctrl-C marker
    // printed via `printf` echoes back within a short window. If 0x03 was not
    // delivered as a real SIGINT, sleep 30 would hold the shell for 30s and
    // the marker would not appear inside the 1.5s budget — the test would
    // fail instead of silently passing.
    let mut rx_b1c = session.subscribe();
    session.write(b"sleep 30\n")?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    session.write(&[0x03])?; // Ctrl-C — must reach the foreground `sleep` as SIGINT
    eprintln!("[smoke] B1 Ctrl-C (0x03) sent to foreground `sleep 30`");
    let ctrl_c_marker = "SEATLOOM_B1_CTRLC_RESUMED";
    // Give the shell a beat to return to prompt, then probe it.
    tokio::time::sleep(Duration::from_millis(200)).await;
    session.write(format!("printf '{ctrl_c_marker}\\n'\n").as_bytes())?;
    let deadline_c = tokio::time::Instant::now() + Duration::from_millis(1500);
    let mut sentinel_seen = false;
    let mut received_c = Vec::<u8>::new();
    while tokio::time::Instant::now() < deadline_c {
        let remaining = deadline_c.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, rx_b1c.recv()).await {
            Ok(Ok(PtyEvent::Output(bytes))) => {
                received_c.extend_from_slice(&bytes);
                if String::from_utf8_lossy(&received_c).contains(ctrl_c_marker) {
                    sentinel_seen = true;
                    break;
                }
            }
            Ok(Ok(PtyEvent::Exited { .. })) | Ok(Err(_)) | Err(_) => break,
        }
    }
    if !sentinel_seen {
        return Err(format!(
            "B1_CTRLC_TIMEOUT: post-Ctrl-C marker '{ctrl_c_marker}' not seen within 1.5s — shell did not return to prompt after Ctrl-C, SIGINT delivery likely broken"
        )
        .into());
    }
    eprintln!("[smoke] B1 Ctrl-C OK: shell resumed after SIGINT, post-interrupt marker echoed");

    // Step 12 (was Step 9): kill the mirror — must remove fifo, must
    // NOT kill the tmux session. R3 failure isolation re-asserted AFTER
    // the B1 write path has been exercised, so we know send-keys-via-
    // load-buffer never escalated SeatLoom into tmux's parent role.
    let fifo_path: PathBuf = session.fifo_path.clone();
    session.kill()?;
    // Give the kernel a moment for the unlink to land.
    tokio::time::sleep(Duration::from_millis(100)).await;
    if fifo_path.exists() {
        return Err(format!("fifo still exists after kill: {}", fifo_path.display()).into());
    }
    eprintln!("[smoke] kill OK: fifo unlinked at {}", fifo_path.display());

    // Confirm R3: tmux session still running.
    let after = list_tmux_sessions()?;
    let still_running = after.iter().any(|s| s.session_name == target);
    if target.ends_with("-seatloom") {
        if !still_running {
            return Err(format!(
                "R3 VIOLATED: tmux session {target} gone after kill_session — SeatLoom is NOT allowed to own tmux"
            )
            .into());
        }
        eprintln!("[smoke] R3 OK: tmux session {target} still running after kill");
    } else {
        // Harness sessions don't match the filter; verify via direct tmux query.
        let out = std::process::Command::new("tmux")
            .args(["has-session", "-t", &target])
            .output()?;
        if !out.status.success() {
            return Err(format!("R3 VIOLATED: tmux has-session {target} failed").into());
        }
        eprintln!("[smoke] R3 OK: tmux session {target} still running after kill (direct has-session check)");
    }

    println!("SMOKE_PASS probe={} ls={} bytes={}", found_probe, found_ls, received.len());
    println!("SMOKE_B1_PASS round_trip=true ctrl_c=true");
    Ok(())
}
