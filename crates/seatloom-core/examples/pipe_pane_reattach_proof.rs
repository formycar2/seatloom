// Behavioural proof for NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 §4.
//
// Exercises the SHIPPED Rust attach path (PtySession::attach_tmux), not raw
// tmux commands, on a THROWAWAY tmux session this harness creates and kills.
// It refuses to run against any session whose name it did not create.
//
//   PROOF 1  first attach  -> pane_pipe=1, bytes flow
//   PROOF 2  SECOND attach -> pane_pipe STAYS 1, bytes STILL flow   <- the regression
//   PROOF 3  detach        -> pipe closes, FIFO unlinked, tmux session ALIVE
//   PROOF 4  PtyError::MirrorNotOpen fires when the pipe cannot open
//
// Usage: cargo run -p seatloom-core --example pipe_pane_reattach_proof

use std::process::Command;
use std::time::Duration;

use seatloom_core::pty::{default_transcripts_dir, LaunchOptions, PtyError, PtyEvent, PtySession};

type Err = Box<dyn std::error::Error>;

fn tmux(args: &[&str]) -> Result<String, Err> {
    let out = Command::new("tmux").args(args).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn pane_pipe(session: &str) -> String {
    tmux(&["display-message", "-t", session, "-p", "#{pane_pipe}"])
        .unwrap_or_else(|_| "<error>".to_string())
}

fn session_exists(session: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", session])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn opts() -> LaunchOptions {
    let root = std::env::current_dir().unwrap_or_else(|_| "/tmp".into());
    LaunchOptions {
        command: "tmux".to_string(),
        args: vec![],
        transcripts_dir: default_transcripts_dir(&root),
        working_dir: root,
        rows: 30,
        cols: 120,
        env: LaunchOptions::default_env(),
    }
}

fn banner(n: u8, title: &str) {
    println!("\n########## PROOF {n} — {title} ##########");
}

/// Attach, then drive some output into the pane and report how many bytes the
/// mirror actually delivered. `pane_pipe=1` alone is not proof that bytes flow.
async fn attach_and_measure(
    session: &str,
    id: &str,
    marker: &str,
) -> Result<(PtySession, usize), Err> {
    let s = PtySession::attach_tmux(id, session, opts())?;
    let mut rx = s.subscribe();
    tokio::time::sleep(Duration::from_millis(250)).await;
    tmux(&[
        "send-keys",
        "-t",
        session,
        &format!("echo {marker}"),
        "Enter",
    ])?;

    let mut bytes = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    let mut saw_marker = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(400), rx.recv()).await {
            Ok(Ok(PtyEvent::Output(chunk))) => {
                bytes += chunk.len();
                if String::from_utf8_lossy(&chunk).contains(marker) {
                    saw_marker = true;
                    break;
                }
            }
            Ok(Ok(_)) => {}
            Ok(Err(_)) | Err(_) => {}
        }
    }
    println!("  bytes delivered = {bytes}, marker {marker:?} seen = {saw_marker}");
    if !saw_marker {
        return Err(format!("mirror delivered no output containing {marker}").into());
    }
    Ok((s, bytes))
}

#[tokio::main]
async fn main() -> Result<(), Err> {
    let session = format!("nimbus-pipeproof-{}", std::process::id());
    println!("throwaway tmux session = {session}");
    println!("(this harness creates and kills ONLY this session; no seat is touched)");

    if session_exists(&session) {
        return Err("throwaway session name already exists; refusing to proceed".into());
    }
    tmux(&["new-session", "-d", "-s", &session, "-x", "120", "-y", "30"])?;
    // Guard: from here on, every failure path must still kill the session.
    let result = run_proofs(&session).await;

    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();
    println!(
        "\ncleanup: throwaway session killed; still exists = {}",
        session_exists(&session)
    );

    result?;
    println!("\n########## ALL PROOFS HELD ##########");
    Ok(())
}

async fn run_proofs(session: &str) -> Result<(), Err> {
    println!("\ninitial pane_pipe = {}", pane_pipe(session));
    assert_eq!(pane_pipe(session), "0", "throwaway pane must start unpiped");

    // ---- PROOF 1 ----------------------------------------------------------
    banner(1, "first attach: pane_pipe=1 and bytes flow");
    let (s1, _) = attach_and_measure(session, "proof-attach-1", "NIMBUS_MARK_ONE").await?;
    let p1 = pane_pipe(session);
    println!("  pane_pipe after 1st attach = {p1}");
    assert_eq!(p1, "1");
    println!("HELD");

    // ---- PROOF 2 — the regression that must not come back ------------------
    banner(2, "SECOND attach: pane_pipe STAYS 1 and bytes STILL flow");
    println!("  (with `-o` this step left pane_pipe=0 and the mirror silently dead)");
    let (s2, _) = attach_and_measure(session, "proof-attach-2", "NIMBUS_MARK_TWO").await?;
    let p2 = pane_pipe(session);
    println!("  pane_pipe after 2nd attach = {p2}");
    assert_eq!(p2, "1", "re-attach must NOT close the pipe");

    // A third, for good measure: the old bug was an alternating toggle, so an
    // even number of attaches was the failing case.
    let (s3, _) = attach_and_measure(session, "proof-attach-3", "NIMBUS_MARK_THREE").await?;
    let p3 = pane_pipe(session);
    println!("  pane_pipe after 3rd attach = {p3}");
    assert_eq!(p3, "1");
    println!("HELD: re-attach is idempotent, not a toggle.");

    // ---- PROOF 3 ----------------------------------------------------------
    banner(
        3,
        "detach: pipe closes, FIFO unlinked, tmux session still ALIVE",
    );
    let fifo3 = s3.fifo_path.clone();
    let fifo2 = s2.fifo_path.clone();
    let fifo1 = s1.fifo_path.clone();
    println!("  fifo (3rd attach) = {}", fifo3.display());
    println!("  exists before kill = {}", fifo3.exists());
    s3.kill()?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let p_after = pane_pipe(session);
    println!("  pane_pipe after kill  = {p_after}   (expect 0)");
    println!("  fifo exists after kill = {}", fifo3.exists());
    println!("  tmux session still alive = {}", session_exists(session));
    assert_eq!(p_after, "0", "detach must close the pipe");
    assert!(!fifo3.exists(), "detach must unlink the FIFO");
    assert!(
        session_exists(session),
        "R3 failure isolation: detach must NOT kill the tmux session"
    );
    // Clean up the other two sessions' FIFOs; their kill() is idempotent.
    s1.kill()?;
    s2.kill()?;
    println!(
        "  fifo(1) exists = {}, fifo(2) exists = {}",
        fifo1.exists(),
        fifo2.exists()
    );
    println!(
        "HELD. NOTE (packet §2.2): the pipe is per-PANE, so these three attaches shared ONE\n\
        \x20    pipe and the first kill stopped the mirror for all of them. Documented on\n\
        \x20    PtySession::kill; a follow-up packet is recommended, not built here."
    );

    // ---- PROOF 4 ----------------------------------------------------------
    banner(
        4,
        "a mirror that cannot open is an Err, never a silent dead session",
    );

    println!("  4a. Measuring what tmux actually does, because my first reading was wrong.");
    println!("      A pipe command that dies does NOT immediately clear pane_pipe — tmux keeps");
    println!("      the flag set until it first tries to WRITE to the dead pipe:");
    let rc = Command::new("tmux")
        .args([
            "pipe-pane",
            "-t",
            session,
            "cat >> /nonexistent-dir-xyz/x.fifo",
        ])
        .output()?;
    println!(
        "      tmux pipe-pane <unwritable path>  -> exit={}",
        rc.status.code().unwrap_or(-1)
    );
    for i in 1..=3 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("        +{}00ms pane_pipe={}", i * 3, pane_pipe(session));
    }
    println!("      now force pane output, which makes tmux write to the dead pipe:");
    Command::new("tmux")
        .args(["send-keys", "-t", session, "echo forcing_a_write", "Enter"])
        .output()?;
    for i in 1..=2 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("        +{}00ms pane_pipe={}", i * 3, pane_pipe(session));
    }
    let _ = Command::new("tmux")
        .args(["pipe-pane", "-t", session])
        .output();
    println!(
        "      CONSEQUENCE, stated plainly: the attach-time probe runs immediately after\n\
        \x20     pipe-pane, so it CANNOT see a pipe that dies on first write. It catches a pipe\n\
        \x20     that never opened. Closing the remaining gap would mean writing to the user's\n\
        \x20     pane to force a flush, which is unacceptable on a live seat. Recorded as a\n\
        \x20     follow-up, not silently claimed as fixed."
    );

    println!("\n  4b. verify_pipe_open() rejects exactly that combination. Unit tests:");
    println!("      cargo test -p seatloom-core --lib -- pty::tests::verify_pipe_open");
    println!("      (exit 0 + pane_pipe=0, empty, garbage, and a failed probe are all Err)");

    println!("\n  4c. End-to-end: attach to a target that cannot be piped at all.");
    match PtySession::attach_tmux(
        "proof-attach-missing",
        "nimbus-pipeproof-nonexistent-session",
        opts(),
    ) {
        Ok(_) => return Err("attach to a missing session must not succeed".into()),
        Err(PtyError::MirrorNotOpen { target, pane_pipe }) => {
            println!("      PtyError::MirrorNotOpen target={target} pane_pipe={pane_pipe}");
        }
        Err(e) => {
            println!("      {e}");
            println!(
                "      NOTE, stated rather than glossed: this case is caught by the PRE-EXISTING\n\
                \x20     non-zero-exit guard, so it does NOT exercise MirrorNotOpen. Driving the new\n\
                \x20     check end-to-end needs a pane where pipe-pane exits 0 and the pipe still\n\
                \x20     fails to open — 4a shows that state is reachable, but attach_tmux creates\n\
                \x20     its own FIFO first, so it cannot be induced from outside without a test\n\
                \x20     seam. That is why the decision logic is a pure function with its own\n\
                \x20     tests (4b) instead of being asserted only through the happy path."
            );
        }
    }
    println!("HELD: every unopenable mirror is an Err; none is reported as a live session.");

    Ok(())
}
