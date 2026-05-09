// Smoke harness for v0.0.1 A3 tmux mirror read path.
//
// Usage (from repo root):
//   PATH="$HOME/.cargo/bin:$PATH" cargo run -p seatloom-core --example tmux_mirror_smoke -- <tmux_session_name>
//
// Exercises:
//   - list_tmux_sessions() filtering
//   - PtySession::attach_tmux (mkfifo + tmux pipe-pane + tail-read)
//   - PtyEvent::Output broadcast delivery
//   - PtySession::resize (tmux resize-pane)
//   - PtySession::kill (stop pipe-pane, unlink fifo, do NOT kill tmux session)
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

    // Step 9: kill the mirror — must remove fifo, must NOT kill the tmux session.
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
    Ok(())
}
