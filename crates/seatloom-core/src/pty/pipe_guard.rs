// pipe_guard — prove the FIFO this session reads is still the FIFO our own pipe
// writes, and stop the mirror the moment it is not.
//
// # Why this module exists
//
// 2026-10-27. The operator reported: "the words I said to Aegis in the live seat
// show up in Nimbus's live seat". Root cause, established at byte level (AEGIS
// packet AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1 §2): while debugging,
// an agent ran `tmux pipe-pane` with no argument on the Aegis pane — believing it
// was a query, when it is a STOP — and then re-attached that pane's output to the
// FIFO the app believed belonged to Nimbus. The app's reader kept reading that
// one FIFO, so Nimbus's slot displayed Aegis's entire output, including every
// word the operator said to it.
//
// The product had zero defence against this. Anyone who runs `tmux pipe-pane`
// — a person or another agent — could make a live seat display an arbitrary
// pane, and the app would not error, grey out, or say "this stream is not the one
// I think it is".
//
// tmux gives exactly one pipe per pane and it is not reference counted: the last
// writer wins and the previous `cat` dies. So the attach-time probe
// (`verify_pipe_open`) cannot help — by the time we notice, `pane_pipe` is back
// to 1 and looks perfectly healthy. Measured on tmux 3.x, both theft shapes:
//
//   stop-then-reattach  bare `pipe-pane` then `pipe-pane <other fifo>`
//   direct-replace      `pipe-pane <other fifo>` alone
//
//   in BOTH: pane_pipe=1, the pane still works, our FIFO stops receiving bytes
//   (or starts receiving somebody else's).
//
// # What this adds, and what it deliberately does not
//
// It adds DOWNSTREAM self-proof. The capture path itself is untouched —
// `pty/mod.rs`'s reader and the pipe-pane command are exactly as they were, so
// the `session:turn` dual-render work (NIMBUS-2026-10-27) does not need
// re-verification.
//
// The proof is not "is the pane alive" (it is) and not "is pane_pipe 1" (it is).
// It is: **which processes are writing to the FIFO I read, and is exactly one of
// them mine.** That question is answerable, and it is answered here.

use std::collections::HashSet;

/// How often a live mirror re-proves its own pipe.
///
/// Chosen against the measured cost of the probe: one `ps` snapshot per interval
/// (sub-millisecond) plus no I/O on the user's pane. 5 s is well inside the
/// packet's "≤ 10 s" recommendation and is 1/60 of the seat-poll cadence, so a
/// stolen mirror is reported within one tick of the discovery loop.
pub const HEALTH_PROBE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// Per-transcript size ceiling before rotation kicks in.
///
/// The packet's recommendation is 64 MB. Measured justification: one single
/// FIFO's transcript reached **180 986 013 bytes** during the cross-wiring
/// incident, because it silently swallowed two high-traffic panes for three
/// hours. At 64 MB a rotation keeps the file at a size where the operator can
/// still copy it, and `.seatloom/transcripts/` (6.3 GB / 422 files) stops being
/// an unbounded surprise.
pub const MAX_TRANSCRIPT_BYTES: u64 = 64 * 1024 * 1024;

/// Verdict of one probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorVerdict {
    /// Exactly one writer names this FIFO and it is the one we spawned.
    ///
    /// "Healthy" is a statement about *ownership*, not about *liveness*: a pipe
    /// whose writer is ours and whose pane is idle is still healthy.
    Owned,
    /// No process is writing to this FIFO at all.
    ///
    /// Two ways to get here, both bad: tmux stopped the pipe (someone ran the
    /// bare `pipe-pane`), or our own writer was killed by a thief replacing it.
    /// Either way this session's stream is dead while `pane_pipe` may still say 1.
    Lost,
    /// More than one writer names this FIFO. **The cross-wiring case.**
    ///
    /// Our writer may well still be alive — tmux does not kill it when a second
    /// pane is pointed at the same FIFO — so the pid check alone would call this
    /// healthy. It is not: this FIFO now carries at least one pane that is not
    /// the one this session claims to mirror, and every byte in it is suspect.
    CrossWired {
        /// How many distinct writers were found. Kept so the operator sees "2",
        /// not just "wrong".
        writers: usize,
    },
    /// The probe itself failed (no `ps`, unparsable output). Reported rather than
    /// folded into [`MirrorVerdict::Owned`]: a probe that cannot run is not a
    /// mirror that is fine. The caller decides how loudly to treat this.
    ProbeFailed,
}

impl MirrorVerdict {
    /// The short label the UI shows and the operator's log records.
    pub fn as_str(&self) -> &'static str {
        match self {
            MirrorVerdict::Owned => "owned",
            MirrorVerdict::Lost => "lost",
            MirrorVerdict::CrossWired { .. } => "cross_wired",
            MirrorVerdict::ProbeFailed => "probe_failed",
        }
    }

    /// Whether this verdict means the mirror must stop being shown.
    pub fn is_broken(&self) -> bool {
        !matches!(self, MirrorVerdict::Owned)
    }
}

/// Decide whether a `ps` command string IS a pipe-pane writer of `want`.
///
/// # Why the command must START with `cat >>`, not merely contain it
///
/// The first version matched `command.contains("cat >>") && command.contains(fifo)`.
/// It was wrong in a way that only the real machine shows: the probe that
/// validated it was itself a shell whose command line contained BOTH substrings
/// (a heredoc that mentions `cat >>` and a fifo path passed as an argument), so
/// the classifier reported `CrossWired { writers: 2 }` for a perfectly healthy
/// mirror. On this machine there are eight healthy mirrors, so that is a false
/// alarm on every probe, for every seat, forever.
///
/// Any command that merely *mentions* the fifo path can contain the substring
/// `cat >>` — a `tmux pipe-pane "cat >> /tmp/.../x.fifo"` invocation from a
/// terminal, a log grep, a script being edited. None of them is a writer.
///
/// So the test is positional: after unwrapping the shell that tmux wraps the
/// command in, the remaining command must BEGIN with `cat >>` and then name the
/// fifo. That is the shape this app creates, and it is the shape only a real
/// writer has:
///
/// ```text
///   sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'     <- a writer
///   cat >> /tmp/seatloom-mirror/ses-a.fifo             <- a writer (no wrapper)
///   /bin/bash -c cd ... && cat > f <<'EOF' ... x.fifo  <- NOT a writer
///   tmux pipe-pane -t X "cat >> '/tmp/.../x.fifo'"      <- NOT a writer
///   grep --color=auto /tmp/.../x.fifo                  <- NOT a writer
/// ```
///
/// # What this still cannot tell apart, stated plainly
///
/// A shell whose ENTIRE command is `cat >> /tmp/.../x.fifo` for reasons other
/// than tmux's pipe would match. That is not a realistic shape (nobody types a
/// bare `cat >>` as their login command), and the residual risk is bounded by
/// the fact that a false positive here costs one `mirror_lost` report on a
/// session whose operator can see the pane is fine — whereas the false negative
/// it prevents costs the operator reading another agent's output in their seat.
/// The asymmetry is the reason for the choice.
fn is_pipe_writer_command(command: &str, want: &str) -> bool {
    let after_shell = unwrap_shell_command(command);
    let Some(rest) = after_shell.strip_prefix("cat >>") else {
        return false;
    };
    normalise_fifo_path(rest).contains(want)
}

/// Strip the shell that wraps a command, if any.
///
/// tmux runs the pipe command as `sh -c <cmd>`, so every writer appears with an
/// `sh -c` prefix that has to come off before the positional test above can look
/// at the real command. Handled for both spellings `ps` may show (`sh -c` and a
/// full path such as `/bin/bash -c`).
///
/// Anything that is not a shell `-c` wrapper is returned unchanged — including
/// `cat >>` with no wrapper at all, which is the other writer shape.
fn unwrap_shell_command(command: &str) -> &str {
    let Some(idx) = command.find(" -c ") else {
        return command;
    };
    // Guard against a command that legitimately contains ` -c ` further along
    // (e.g. an argument): only unwrap when what precedes it looks like a shell.
    let head = command[..idx].trim();
    let looks_like_shell = head.ends_with("sh")
        || head.ends_with("bash")
        || head.ends_with("zsh")
        || head.ends_with("/bin/sh")
        || head.is_empty();
    if looks_like_shell {
        command[idx + 4..].trim_start()
    } else {
        command
    }
}
///
/// This is the whole detection rule, kept as a pure function over the snapshot
/// text so the three states are unit-testable without a live tmux pane — the
/// same reason `verify_pipe_open` above is a pure function. Reaching the
/// cross-wired state through `attach_tmux` would need a second agent to steal
/// the pipe mid-test.
///
/// # The rule, and why each part is there
///
/// `ps -Ao pid=,ppid=,command=` lines are matched for the fifo path appearing
/// after `cat >>`. That is the command shape this app itself creates
/// (`shell_quote`d), and matching on it means:
///
///   * a `cat` reading the FIFO does not match (`cat <path>` has no `>>`);
///   * a pane writing a *different* FIFO does not match, because the full path
///     is matched as a token, not as a substring — `ses-a.fifo` is not a
///     substring of any other session id, but the check is exact regardless;
///   * the app's own reader holds no argv entry for the fifo at all (it is a Rust
///     thread with an fd), so it cannot be mistaken for a writer.
///
/// tmux wraps the pipe command in `sh -c` and that shell spawns the real `cat`,
/// so **every writer appears as two `ps` lines** (parent `sh -c`, child `sh -c`
/// that has exec'd or forked the cat). Measured on the real thing:
///
/// ```text
/// 91866  1951 sh -c cat >> '/tmp/x1.fifo'
/// 91874 91866 sh -c cat >> '/tmp/x1.fifo'
/// ```
///
/// Counting lines would therefore report 2 writers for a perfectly healthy
/// mirror — a false "cross-wired" on every single probe, which is the worst
/// possible outcome for a check: it trains the operator to ignore it. So
/// wrapper/child pairs are collapsed to their top-level parent before counting.
/// The collapse is `ppid is also in the matched set`, which is exact here
/// because tmux is the parent of every top-level writer it spawns.
///
/// `self_pids` is the caller's own process tree, excluded so that a probe run
/// from a process that happens to name the fifo (a shell the operator opened to
/// inspect it) does not read as a thief.
pub fn classify_writers(ps_text: &str, fifo_path: &str, self_pids: &HashSet<u32>) -> MirrorVerdict {
    // Normalise the path so a `/tmp` vs `/private/tmp` mismatch (macOS symlinks
    // `/tmp`) cannot turn every writer into zero writers.
    let want = normalise_fifo_path(fifo_path);

    let mut matched: Vec<(u32, u32)> = Vec::new();
    for line in ps_text.lines() {
        let (pid, ppid, command) = match parse_ps_line(line) {
            Some(v) => v,
            None => continue,
        };
        if self_pids.contains(&pid) || self_pids.contains(&ppid) {
            continue;
        }
        if is_pipe_writer_command(&command, &want) {
            matched.push((pid, ppid));
        }
    }

    if matched.is_empty() {
        return MirrorVerdict::Lost;
    }

    // Collapse tmux's wrapper/child pairs: a writer whose parent is also a
    // matched writer is the inner half of one spawn, not a second writer.
    let pids: HashSet<u32> = matched.iter().map(|(p, _)| *p).collect();
    let top_level: Vec<(u32, u32)> = matched
        .iter()
        .copied()
        .filter(|(_, ppid)| !pids.contains(ppid))
        .collect();
    let writers = if top_level.is_empty() {
        // Defensive: if every matched process has a matched parent the set is
        // pathological (a cycle, or a `ps` snapshot taken across a fork). Count
        // the raw matches rather than silently reporting zero writers.
        matched.len()
    } else {
        top_level.len()
    };

    match writers {
        1 => MirrorVerdict::Owned,
        0 => MirrorVerdict::Lost,
        n => MirrorVerdict::CrossWired { writers: n },
    }
}

/// Take a `ps -Ao pid=,ppid=,command=` snapshot.
///
/// Returns an empty string on failure; the caller turns that into
/// [`MirrorVerdict::ProbeFailed`] rather than assuming the mirror is fine.
pub fn snapshot_processes() -> String {
    use std::process::{Command, Stdio};
    Command::new("ps")
        .args(["-Ao", "pid=,ppid=,command="])
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Classify a mirror from an already-taken `ps` snapshot.
///
/// Split from [`probe`] so the failure paths are unit-testable: `probe` runs a
/// real `ps` against the real machine, which in a test means the verdict is
/// whatever this machine happens to be running. The pure form lets a test hand
/// in the exact snapshot it wants to reason about — the same reason
/// `verify_pipe_open` in the parent module is a pure function.
pub fn probe_with_snapshot(
    fifo_path: &str,
    pane_pipe_ok: bool,
    ps_text: &str,
    self_pids: &HashSet<u32>,
) -> MirrorVerdict {
    if !pane_pipe_ok {
        // tmux itself says this pane is not piped. Nothing else to learn.
        return MirrorVerdict::Lost;
    }
    if ps_text.trim().is_empty() {
        return MirrorVerdict::ProbeFailed;
    }
    classify_writers(ps_text, fifo_path, self_pids)
}

/// One probe, production path: snapshot the process table, then classify.
pub fn probe(fifo_path: &str, pane_pipe_ok: bool, self_pids: &HashSet<u32>) -> MirrorVerdict {
    probe_with_snapshot(fifo_path, pane_pipe_ok, &snapshot_processes(), self_pids)
}

/// Make a path comparable regardless of whether `/tmp` was printed as
/// `/private/tmp`.
///
/// macOS symlinks `/tmp` → `/private/tmp`; Linux does not. The app writes
/// `/tmp/...` into the pipe command, but `ps` reports whatever the kernel holds,
/// so a mirror can look like it has zero writers purely because the two spellings
/// differ. Stripping the `/private` prefix from BOTH sides makes them equal in
/// either spelling, on either OS.
///
/// Deliberately not `std::fs::canonicalize`: the FIFO may legitimately be absent
/// (a lost mirror unlinks it), and a canonicalize that fails would turn every
/// probe of a dead mirror into a ProbeFailed.
fn normalise_fifo_path(p: &str) -> String {
    p.trim()
        .strip_prefix("/private")
        .map(|rest| rest.to_string())
        .unwrap_or_else(|| p.trim().to_string())
}

/// Parse one `pid ppid command...` line. Command is everything after the second
/// whitespace run, verbatim — quoting in it is significant and must survive.
///
/// # Why this is not `splitn(3, char::is_whitespace)`
///
/// It was, and it was wrong. `splitn` with a char pattern emits an EMPTY field
/// for each extra delimiter, so the real `ps -Ao pid=,ppid=` output — which pads
/// its columns with runs of spaces — parsed as `pid=91866, ppid="", command=""`
/// and every line failed to parse. The probe would then have reported
/// [`MirrorVerdict::Lost`] for every healthy mirror on the machine, forever. The
/// first draft only survived because the other fixtures were Rust line
/// continuations, which silently collapse `\` + newline + indentation into one
/// space; this one fixture had a real double space and caught it.
///
/// So: take the two numeric tokens by hand, then keep the remainder of the line
/// untouched. That is both correct against real padding and preserves the quoting
/// the classifier matches on.
fn parse_ps_line(line: &str) -> Option<(u32, u32, String)> {
    let (pid_s, rest) = line.trim_start().split_once(char::is_whitespace)?;
    let pid = pid_s.parse::<u32>().ok()?;
    let (ppid_s, rest) = rest.trim_start().split_once(char::is_whitespace)?;
    let ppid = ppid_s.parse::<u32>().ok()?;
    let command = rest.trim_start().to_string();
    if command.is_empty() {
        return None;
    }
    Some((pid, ppid, command))
}

/// Pids of this process and its descendants, so a probe does not mistake its own
/// shelling-out for a foreign writer.
pub fn own_pids() -> HashSet<u32> {
    use std::process::Command;
    let me = std::process::id();
    let mut set = HashSet::from([me]);
    let out = Command::new("ps").args(["-Ao", "pid=,ppid="]).output();
    let Ok(out) = out else { return set };
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        // Two columns only, so this cannot reuse `parse_ps_line` (which requires a
        // third field) — that silently returned None for every line and produced
        // an empty tree, which would have excluded nothing.
        let mut it = line.split_whitespace();
        let (Some(pid), Some(ppid)) = (it.next(), it.next()) else {
            continue;
        };
        if let (Ok(pid), Ok(ppid)) = (pid.parse::<u32>(), ppid.parse::<u32>()) {
            edges.push((pid, ppid));
        }
    }
    // Walk child→parent to closure. The tree is small (hundreds of processes);
    // a linear sweep per level is clearer than building an adjacency map for
    // something that runs every 5 s.
    let mut changed = true;
    while changed {
        changed = false;
        for (pid, ppid) in &edges {
            if set.contains(ppid) && !set.contains(pid) {
                set.insert(*pid);
                changed = true;
            }
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selfs() -> HashSet<u32> {
        // A pid range no real process occupies: 4 000 000+.
        HashSet::from([4_000_001])
    }

    /// The command shape this app actually creates (`shell_quote`d path).
    const HEALTHY: &str = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                           91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n";

    #[test]
    fn a_healthy_mirror_has_exactly_one_writer_and_the_wrapper_pair_does_not_count_twice() {
        // The regression that matters: tmux spawns `sh -c` and that shell spawns
        // the cat, so every writer is TWO ps lines. Counting lines reports 2 and
        // every healthy mirror looks cross-wired — a check that cries wolf on
        // every probe is worse than no check, because it gets ignored.
        assert_eq!(
            classify_writers(HEALTHY, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn no_writer_at_all_is_lost() {
        let ps = "1951  1 tmux new-session -d -s Aegis-Supervisor\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Lost
        );
    }

    #[test]
    fn two_panes_writing_one_fifo_is_cross_wired() {
        // The reported defect, verbatim: the app's own writer is still alive
        // (tmux does not kill it) AND a second pane now writes the same FIFO.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91902  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91904 91902 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::CrossWired { writers: 2 }
        );
    }

    #[test]
    fn a_writer_for_a_different_fifo_is_not_a_writer_for_mine() {
        // Path is matched as a token, not a substring: the fleet has sessions
        // whose ids share prefixes, and `contains` on a prefix would make every
        // seat look cross-wired.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91902  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-ab.fifo'\n\
                  91904 91902 sh -c cat >> '/tmp/seatloom-mirror/ses-ab.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-ab.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn a_reader_of_my_fifo_is_not_a_writer() {
        // The reader is `cat <path>` with no `>>`. If this regressed, the app
        // would report its own reader as a second writer and go mirror_lost the
        // instant it attached.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  89220     1 cat /tmp/seatloom-mirror/ses-a.fifo\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn my_own_process_naming_the_fifo_is_excluded() {
        // An operator's shell, or the probe itself, mentioning the path is not a
        // thief. Without this, `ps | grep <fifo>` from a terminal would be read
        // as a foreign writer.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  93000  1951 grep --color=auto /tmp/seatloom-mirror/ses-a.fifo\n";
        let mut me = selfs();
        me.insert(93000);
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &me),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn a_writer_whose_parent_is_not_a_writer_still_counts_as_one() {
        // tmux respawned without the `sh -c` wrapper (observed on some tmux
        // builds): a single `cat` line with an unrelated parent. Must not be
        // collapsed away to zero writers.
        //
        // The DOUBLE SPACE after the pid is not decoration — it is what real
        // `ps -Ao pid=,ppid=` padding looks like. The first version of
        // `parse_ps_line` used `splitn(3, char::is_whitespace)`, which turns
        // that padding into an empty field and fails the line, so every healthy
        // mirror on a real machine would have been reported `Lost`.
        let ps = "91866  1951 cat >> /tmp/seatloom-mirror/ses-a.fifo\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn real_ps_padding_and_tabs_do_not_break_the_parse() {
        // `ps` right-aligns its columns, so the amount of whitespace varies with
        // the pid width. A parser that only tolerates exactly one space fails on
        // the machine it was written for.
        let ps = "  91866   1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                91874  91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn a_command_that_merely_mentions_the_fifo_is_not_a_writer() {
        // THE FALSE POSITIVE THIS MODULE ACTUALLY HIT. The first version matched
        // `command.contains("cat >>") && command.contains(fifo)`. Validating it
        // on the real machine, the validating shell's own command line was a
        // heredoc containing both substrings — so a healthy mirror was reported
        // `CrossWired { writers: 2 }`. On a machine with eight healthy mirrors
        // that is a false alarm on every probe, for every seat, forever.
        //
        // These are the shapes that must NOT count. Each is a real thing a person
        // or agent runs while looking at the mirror.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  94649 21849 /bin/bash -c cd /w && cat > f <<'EOF' ... cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  94650 94649 sh -c cargo run --example pipe_guard_probe /tmp/seatloom-mirror/ses-a.fifo\n\
                  94651 21849 tmux pipe-pane -t X \"cat >> '/tmp/seatloom-mirror/ses-a.fifo'\"\n\
                  94652 21849 grep --color=auto /tmp/seatloom-mirror/ses-a.fifo\n\
                  94653 21849 lsof -t /tmp/seatloom-mirror/ses-a.fifo\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned,
            "mentions of the fifo are not writers"
        );
    }

    #[test]
    fn a_second_real_writer_is_still_caught_next_to_a_mention() {
        // The false-positive fix must not have bought silence by stopping
        // detection: a genuine second writer alongside any number of mentions
        // still has to be reported.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  94649 21849 tmux pipe-pane -t X \"cat >> '/tmp/seatloom-mirror/ses-a.fifo'\"\n\
                  91902  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91904 91902 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::CrossWired { writers: 2 }
        );
    }

    #[test]
    fn tmp_and_private_tmp_compare_equal() {
        // macOS: `/tmp` is a symlink to `/private/tmp`. The app writes `/tmp/...`
        // into the pipe command; without normalising, every writer fails to match
        // and every healthy mirror is reported lost.
        let ps = "91866  1951 sh -c cat >> '/private/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/private/tmp/seatloom-mirror/ses-a.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn pane_pipe_zero_is_lost_before_we_even_look_at_processes() {
        // tmux says the pane is not piped. That is authoritative and cheaper than
        // a process scan, and it is the shape the attach-time probe already
        // understands.
        let ps = "91866  1951 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n\
                  91874 91866 sh -c cat >> '/tmp/seatloom-mirror/ses-a.fifo'\n";
        assert_eq!(
            classify_writers(ps, "/tmp/seatloom-mirror/ses-a.fifo", &selfs()),
            MirrorVerdict::Owned,
            "sanity: with a healthy writer the scan agrees"
        );
        // probe_with_snapshot short-circuits on pane_pipe=false without consulting
        // the snapshot at all.
        assert_eq!(
            probe_with_snapshot("/tmp/seatloom-mirror/ses-a.fifo", false, HEALTHY, &selfs()),
            MirrorVerdict::Lost
        );
        // ...and with a healthy writer it agrees with the scan.
        assert_eq!(
            probe_with_snapshot("/tmp/seatloom-mirror/ses-a.fifo", true, HEALTHY, &selfs()),
            MirrorVerdict::Owned
        );
    }

    #[test]
    fn an_empty_snapshot_is_a_failed_probe_not_a_healthy_mirror() {
        // A probe that could not run is not a mirror that is fine. If this
        // regressed to Owned, a broken `ps` would silently green-light every
        // stolen mirror — the exact "check that cannot fail" defect.
        assert_eq!(
            probe_with_snapshot("/tmp/seatloom-mirror/ses-a.fifo", true, "", &selfs()),
            MirrorVerdict::ProbeFailed
        );
        assert_eq!(
            probe_with_snapshot("/tmp/seatloom-mirror/ses-a.fifo", true, "   \n", &selfs()),
            MirrorVerdict::ProbeFailed
        );
    }

    #[test]
    fn every_broken_verdict_says_so_and_owned_does_not() {
        assert!(!MirrorVerdict::Owned.is_broken());
        assert!(MirrorVerdict::Lost.is_broken());
        assert!(MirrorVerdict::CrossWired { writers: 2 }.is_broken());
        assert!(MirrorVerdict::ProbeFailed.is_broken());
        assert_eq!(
            MirrorVerdict::CrossWired { writers: 2 }.as_str(),
            "cross_wired"
        );
    }
}
