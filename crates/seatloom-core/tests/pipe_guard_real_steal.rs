//! The pipe-steal, reproduced for real, judged by the production classifier.
//!
//! The unit tests in `pty::pipe_guard` hand in a fake `ps` snapshot. This test
//! does not: it makes a real tmux session, attaches a real pipe, really stops it
//! the way the bug did, and feeds the REAL `ps` output to `probe_with_snapshot`.
//!
//! Why it has to be real: on 2026-10-28 I tried to reproduce the steal by
//! running `tmux pipe-pane` with a DIFFERENT cat, and nothing happened - both
//! writers coexisted. The actual bug is a BARE `pipe-pane` (no argument), which
//! is a STOP command, not a query. That distinction is invisible in a fake
//! snapshot and is the whole reason this test exists.
//!
//! The load-bearing finding this test pins: after the STOP, the `cat` processes
//! are STILL RUNNING (state S, not Z) and still appear in `ps` as writers of our
//! FIFO. So a guard that looks only at the process table reports the mirror as
//! healthy while zero bytes are flowing. `pane_pipe == 0` is the only thing that
//! tells the truth, and the guard must check it FIRST.

use seatloom_core::pty::pipe_guard::{self, MirrorVerdict};
use std::collections::HashSet;
use std::process::Command;

fn tmux(args: &[&str]) -> String {
    let out = Command::new("tmux").args(args).output().expect("tmux");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_real_bare_pipe_pane_stops_the_pipe_and_the_guard_says_lost() {
    let session = format!("pg-test-{}", std::process::id());
    let fifo = format!("/tmp/seatloom-pg-test-{}.fifo", std::process::id());
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();

    // A real session, a real FIFO, a real pipe.
    tmux(&["new-session", "-d", "-s", &session]);
    let _ = Command::new("mkfifo").arg(&fifo).output();

    // The pane must have a pid for the pipe to mean anything.
    let pane_pid: u32 = tmux(&[
        "list-panes",
        "-t",
        &format!("{session}:0"),
        "-F",
        "#{pane_pid}",
    ])
    .trim()
    .parse()
    .expect("pane pid");

    tmux(&[
        "pipe-pane",
        "-t",
        &format!("{session}:0"),
        &format!("cat >> {fifo}"),
    ]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let pipe_ok_before = tmux(&[
        "display-message",
        "-p",
        "-t",
        &format!("{session}:0"),
        "#{pane_pipe}",
    ]);
    assert_eq!(
        pipe_ok_before.trim(),
        "1",
        "our own attach should set pane_pipe=1"
    );

    // ---- THE BUG, reproduced exactly ----
    // A bare `pipe-pane` is a STOP command, not a query. This is what an
    // external process (or a seat believing it is a query) runs.
    tmux(&["pipe-pane", "-t", &format!("{session}:0")]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let pipe_ok_after = tmux(&[
        "display-message",
        "-p",
        "-t",
        &format!("{session}:0"),
        "#{pane_pipe}",
    ]);
    assert_eq!(
        pipe_ok_after.trim(),
        "0",
        "the bare pipe-pane must have stopped the pipe - if it did not, \
         this test is not reproducing the bug"
    );

    // The real process table, exactly as production would take it.
    let ps_text = pipe_guard::snapshot_processes();

    // How many of our FIFO's writers are still in the table? They ARE - that is
    // the trap. Record it so the assertion below is about the measured world.
    let writers_still_listed = ps_text
        .lines()
        .filter(|l| {
            l.contains(&format!("cat >> {fifo}")) || l.contains(&format!("cat >> '{fifo}'"))
        })
        .count();

    let self_pids: HashSet<u32> = [pane_pid].into_iter().collect();
    let verdict = pipe_guard::probe_with_snapshot(&fifo, false, &ps_text, &self_pids);

    assert_eq!(
        verdict,
        MirrorVerdict::Lost,
        "pane_pipe=0 must be judged Lost even though {writers_still_listed} writer \
         line(s) for our FIFO are still in the process table. A guard that reads \
         only the process table would call this mirror healthy while nothing flows."
    );

    // And the guard must NOT be satisfied by the process table alone: if we lied
    // and said the pipe was still open, it would look at processes and - since
    // the cats are alive and sleeping - could well call it healthy. That is
    // precisely the false-negative this ordering prevents.
    let verdict_if_we_lied = pipe_guard::probe_with_snapshot(&fifo, true, &ps_text, &self_pids);
    assert_ne!(
        verdict_if_we_lied,
        MirrorVerdict::Lost,
        "if pane_pipe is reported as open, the guard must NOT say Lost - it is \
         reading the process table, which still lists the sleeping cats"
    );

    // ---- THE LOAD-BEARING ASSERTION ----
    // The assertion above passes even if the pane_pipe check is merely MOVED
    // after the process table (verified by mutation on 2026-10-28: that mutation
    // survived). What no reordering survives is deleting the check, so assert
    // against a snapshot that has live writers AND pane_pipe=0 - the exact state
    // the real world was in above.
    let synthetic = format!("  100     1 sh -c cat >> {fifo}\n  101   100 sh -c cat >> {fifo}\n");
    let v = pipe_guard::probe_with_snapshot(&fifo, false, &synthetic, &HashSet::new());
    assert_eq!(
        v,
        MirrorVerdict::Lost,
        "pane_pipe=0 with writers still in the table must be Lost. Without the \
         pane_pipe check this returns Owned - the false-negative that let the \
         operator's text appear in the wrong seat."
    );

    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();
    let _ = std::fs::remove_file(&fifo);
}
