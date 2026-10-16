//! Throwaway behavioural proof for NIMBUS-2026-10-08-pty-resize-window-v1.
//!
//! Drives the REAL `PtySession::resize` against a throwaway tmux session that
//! this program creates and kills. Never touches a live seat. Reads back
//! `#{pane_width} #{pane_height}` verbatim at each step; an exit code alone is
//! not evidence (that is exactly the bug being fixed).
//!
//! Run: `cargo run -p seatloom-core --example nimbus_resize_proof`

use std::path::PathBuf;
use std::process::Command;

use seatloom_core::pty::{LaunchOptions, PtySession};

fn geo(target: &str) -> String {
    let out = Command::new("tmux")
        .args([
            "display-message",
            "-t",
            target,
            "-p",
            "#{pane_width} #{pane_height}",
        ])
        .output()
        .expect("display-message");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn main() {
    let session = format!("nimbus-resize-proof-{}", std::process::id());
    let target = format!("{session}:0");

    // Clean any stale session, then create a throwaway one at a known size.
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();
    let created = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &session,
            "-x",
            "220",
            "-y",
            "50",
            "sleep 300",
        ])
        .status()
        .expect("new-session");
    assert!(created.success(), "failed to create throwaway session");
    println!("created                 : {}", geo(&target));

    let opts = LaunchOptions {
        command: "sleep".into(),
        args: vec!["300".into()],
        working_dir: PathBuf::from("/tmp"),
        rows: 50,
        cols: 220,
        env: LaunchOptions::default_env(),
        transcripts_dir: std::env::temp_dir().join(&session),
        backfill_lines: None,
        history_limit: None,
    };
    let sess = PtySession::attach_tmux(&session, &session, opts).expect("attach_tmux");

    // Step 2: shrink to 120x40.
    sess.resize(40, 120).expect("resize(40,120) should succeed");
    println!(
        "resize(40,120)          : {}   (want: 120 40)",
        geo(&target)
    );

    // Step 3: 80x24.
    sess.resize(24, 80).expect("resize(24,80) should succeed");
    println!(
        "resize(24,80)           : {}    (want: 80 24)",
        geo(&target)
    );

    // Step 4: GROW past launch size (this is the case that fails today).
    sess.resize(60, 240).expect("resize(60,240) should succeed");
    println!("resize(60,240) [grow]   : {}  (want: 240 60)", geo(&target));

    // Step 5: force the typed error. Two cases.
    //
    // 5a: a resize-window that itself fails (dead session) surfaces as
    //     PtyError::Tmux before geometry read-back — still a typed error, still
    //     NOT a reported success. This is the branch exercised against a killed
    //     session.
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();
    match sess.resize(24, 80) {
        Err(e) => println!("resize on dead session   : typed error fired -> {e}"),
        Ok(()) => panic!("resize on a dead session must not report success"),
    }

    // 5b: the ResizeNotApplied branch itself — resize-window exits 0 but the
    //     pane geometry does not match. This is the exact decision the live
    //     `resize` runs after the exit-0 check. It is covered exhaustively and
    //     deterministically by the unit test
    //     `pty::tests::verify_resize_applied_rejects_a_no_op_resize`, which
    //     feeds the live helper a simulated no-op read-back (pane stayed at
    //     220x50 after requesting 120x40) — precisely what `resize-pane -x/-y`
    //     produced before this fix. See `cargo test -p seatloom-core --lib`.

    // Step 6: ensure clean — session gone.
    let still = Command::new("tmux")
        .args(["has-session", "-t", &session])
        .status()
        .unwrap();
    println!("session present after kill: {}", still.success());
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();
    println!("killed                   : throwaway session removed");
}
