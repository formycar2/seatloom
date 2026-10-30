//! A multi-line paste through the REAL `write()` path, judged by what the pane's
//! program actually received.
//!
//! WHY THIS EXISTS. On 2026-10-29 Mr. Zhang reported that pasting a multi-line
//! instruction into a seat arrives in saih line by line, queued and executed
//! separately. Aegis dispatched the fix as "add an onPaste handler so a paste
//! becomes ONE write, not N", with the mechanism given as: xterm.js 5.5.0 routes
//! a paste through its default per-character input path, so `onData` fires N
//! times and `PtySession::write` forks N `load-buffer`+`paste-buffer` pairs.
//!
//! I measured that mechanism before writing any code, and the given one is wrong.
//! Reading xterm.js 5.5.0's own bundle:
//!
//!   * the string `onPaste` occurs **0** times in the whole package, and
//!     `ITerminalOptions` has no such member — the option Aegis asked for does
//!     not exist in this version, so it is not implementable as stated;
//!   * the real paste handler is `handlePasteEvent -> paste(text, textarea,
//!     coreService, optionsService)`, which runs `prepareTextForTerminal`
//!     (every `\n` becomes `\r`), then calls
//!     `coreService.triggerDataEvent(wholeText, true)` — ONE event, ONE
//!     `onData`, the ENTIRE block.
//!
//! So the paste already arrives as one chunk and already becomes one write.
//! Measured for real: three separate `load-buffer`+`paste-buffer` round trips
//! deliver byte-for-byte the same thing as one. Splitting the write is not the
//! defect.
//!
//! THE ACTUAL DEFECT, measured with a program that declares bracketed paste the
//! way saih does (`packages/tui/src/terminal.ts:212` writes `ESC[?2004h`):
//!
//!   `paste-buffer`    -> the pane receives `aa\rbb\rcc\r`            (3 submits)
//!   `paste-buffer -p` -> the pane receives `ESC[200~aa\rbb\rcc\rESC[201~`
//!                                                                   (1 paste)
//!
//! saih's editor parses the bracketed form correctly: `packages/tui/src/
//! components/editor.ts:721` detects `ESC[200~`, buffers until `ESC[201~`, and
//! `handlePaste` inserts the block as ONE message. So the capability already
//! exists on both ends and nothing was using it. A native paste works because
//! the terminal wraps it before tmux sees it; our path bypasses that entirely.
//!
//! Aegis recorded "I also tested paste-buffer -p, which made NO difference,
//! because saih does not emit 2004". That is measured false — terminal.ts:212
//! does emit it — and it is why he ruled the fix out. A `-p` measured against a
//! non-declaring program correctly does nothing, which is what he saw.
//!
//! Both tests below drive the production `PtySession::write` against a real tmux
//! pane, so they fail if the wrap is removed from the product.

use seatloom_core::pty::{is_multiline_paste, LaunchOptions, PtySession};
use std::process::Command;

fn tmux(args: &[&str]) -> String {
    let out = Command::new("tmux").args(args).output().expect("tmux");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A program that declares bracketed-paste support, exactly as saih's TUI does,
/// and records the bytes it receives. Declaring matters: `paste-buffer -p` is a
/// no-op against a pane that has not, which is what made Aegis measure
/// "no difference".
fn declaring_logger(dir: &str, outfile: &str, tag: &str) -> String {
    // The file is written after EVERY chunk, not on exit. The logger starts
    // waiting the moment the pane is created — long before the test's writes
    // happen — so an on-exit write would be read as "" by the test.
    let py = format!(
        "import sys, os, termios, select, time, signal\n\
         signal.signal(signal.SIGINT, signal.SIG_IGN)\n\
         sys.stdout.write('\\x1b[?2004h'); sys.stdout.flush()\n\
         fd = sys.stdin.fileno(); old = termios.tcgetattr(fd)\n\
         raw = termios.tcgetattr(fd)\n\
         raw[0] &= ~(termios.ICRNL|termios.INLCR|termios.IGNCR)\n\
         raw[3] &= ~(termios.ECHO|termios.ICANON)\n\
         termios.tcsetattr(fd, termios.TCSANOW, raw)\n\
         buf = b''\n\
         deadline = time.time() + 30\n\
         try:\n\
         \x20   while time.time() < deadline:\n\
         \x20       r,_,_ = select.select([fd],[],[],1.0)\n\
         \x20       if r:\n\
         \x20           b = os.read(fd, 4096)\n\
         \x20           if not b: break\n\
         \x20           buf += b\n\
         \x20           open({outfile:?}, 'w').write(repr(buf))\n\
         finally:\n\
         \x20   termios.tcsetattr(fd, termios.TCSANOW, old)\n\
         \x20   open({outfile:?}, 'w').write(repr(buf))\n"
    );
    let path = format!("{dir}/declarer-{tag}.py");
    std::fs::write(&path, py).unwrap();
    format!("python3 -u {path}")
}

/// The logger writes Python's `repr` of the bytes; turn it back into bytes.
fn decode_repr(raw: &str) -> Vec<u8> {
    let inner = raw
        .trim()
        .trim_start_matches("b'")
        .trim_end_matches("'")
        .to_string();
    let b = inner.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            match b[i + 1] {
                b'n' => {
                    out.push(b'\n');
                    i += 2;
                    continue;
                }
                b'r' => {
                    out.push(b'\r');
                    i += 2;
                    continue;
                }
                b't' => {
                    out.push(b'\t');
                    i += 2;
                    continue;
                }
                b'x' if i + 3 < b.len() => {
                    let hex = std::str::from_utf8(&b[i + 2..i + 4]).unwrap();
                    out.push(u8::from_str_radix(hex, 16).unwrap());
                    i += 4;
                    continue;
                }
                other => {
                    out.push(other);
                    i += 2;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn read_received(session: &str) -> Vec<u8> {
    let path = format!("/tmp/seatloom-pastelogger/{session}.out");
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    decode_repr(&raw)
}

fn attach(probe_id: &str, session: &str) -> PtySession {
    // The FIFO path is derived from this id, so each test MUST pass its own.
    // Two tests sharing one collide on mkfifo and the second test then reads
    // the first test's pipe — which is exactly the false failure this guards.
    let _ = std::fs::remove_file(format!("/tmp/seatloom-mirror/{probe_id}.fifo"));
    PtySession::attach_tmux(
        probe_id,
        session,
        LaunchOptions {
            command: "tmux".to_string(),
            args: vec!["pipe-pane".to_string(), session.to_string()],
            working_dir: "/tmp".into(),
            rows: 10,
            cols: 80,
            env: LaunchOptions::default_env(),
            transcripts_dir: "/tmp".into(),
            backfill_lines: Some(0),
            history_limit: None,
        },
    )
    .expect("attach")
}

fn fresh_session(session: &str, outfile: &str) {
    let dir = "/tmp/seatloom-pastelogger";
    std::fs::create_dir_all(dir).unwrap();
    let _ = std::fs::remove_file(outfile);
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output();
    let cmd = declaring_logger(dir, outfile, session);
    tmux(&[
        "new-session",
        "-d",
        "-x",
        "80",
        "-y",
        "10",
        "-s",
        session,
        &cmd,
    ]);
    std::thread::sleep(std::time::Duration::from_millis(1_500));
}

#[test]
fn a_multiline_paste_through_write_reaches_the_pane_as_one_bracketed_paste() {
    let dir = "/tmp/seatloom-pastelogger";
    std::fs::create_dir_all(dir).unwrap();
    let session = format!("pl-fix-{}", std::process::id());
    let outfile = format!("{dir}/{session}.out");
    fresh_session(&session, &outfile);
    let handle = attach(&format!("paste-{}", std::process::id()), &session);

    // What xterm's real paste handler delivers for a three-line paste: the whole
    // block, with every \n already converted to \r by prepareTextForTerminal.
    handle
        .write(b"first line\rsecond line\rthird line")
        .expect("write");
    std::thread::sleep(std::time::Duration::from_millis(4_500));

    let got = read_received(&session);
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();

    // THE CRITERION. The pane's program must see ONE bracketed paste, not three
    // submits. saih's editor parses this into a single message; three bare \r's
    // would be three submits, which is the defect Mr. Zhang reported.
    let mut expected = b"\x1b[200~first line\rsecond line\rthird line".to_vec();
    expected.extend_from_slice(b"\x1b[201~");
    assert_eq!(
        got,
        expected,
        "a multi-line paste must arrive as ONE bracketed paste; got {:?}",
        String::from_utf8_lossy(&got)
    );
}

#[test]
fn the_keystroke_path_is_unchanged_by_the_paste_wrap() {
    // The other half, and the reason the wrap is conditional: saih's handlePaste
    // STRIPS non-printables except newlines, so wrapping Ctrl+C would swallow
    // it, and it also calls cancelAutocomplete(). Keystrokes must be untouched.
    //
    // A PRE-EXISTING DEFECT, measured, not hidden: tmux's `paste-buffer` DROPS a
    // lone 0x03. Verified directly — writing only `\x03` through
    // `load-buffer -` + `paste-buffer` delivers nothing to the pane, while
    // `A\x03B` delivers `A` then `B`. That is tmux's behaviour on the raw path
    // and it predates this change: `is_multiline_paste(&[0x03])` is false, so the
    // args built are byte-identical to what this function built before. This
    // test asserts the paste wrap does not make it WORSE, and records that it is
    // already broken rather than asserting a passing lie.
    let dir = "/tmp/seatloom-pastelogger";
    std::fs::create_dir_all(dir).unwrap();
    let session = format!("pl-keys-{}", std::process::id());
    let outfile = format!("{dir}/{session}.out");
    fresh_session(&session, &outfile);
    let handle = attach(&format!("paste-{}", std::process::id()), &session);

    // A lone Enter, then a printable char, then a two-char typed word — the
    // ordinary typing path, plus the Enter that submits.
    handle.write(b"\r").expect("enter");
    handle.write(b"a").expect("a");
    handle.write(b"bc").expect("bc");
    std::thread::sleep(std::time::Duration::from_millis(4_500));

    let got = read_received(&session);
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();

    // 1. What arrives is exactly what was written — no rewriting.
    assert_eq!(
        got,
        b"\rabc".to_vec(),
        "the keystroke path must deliver bytes verbatim; got {:?}",
        String::from_utf8_lossy(&got)
    );
    // 2. NO bracketed-paste marker anywhere on the keystroke path. If one
    //    appeared, an Enter would be swallowed by handlePaste and autocomplete
    //    would be cancelled — a real regression.
    assert!(
        !got.windows(6).any(|w| w == b"\x1b[200~"),
        "no bracketed-paste marker may appear on the keystroke path"
    );
    // 3. And a lone Enter is NOT classified as a paste, which is what keeps it
    //    on this raw path in the first place.
    assert!(
        !is_multiline_paste(b"\r"),
        "a lone Enter must stay a keystroke, not become a paste"
    );
}

#[test]
fn the_classifier_separates_a_paste_from_a_keystroke() {
    // The rule, pinned so the conditional wrap cannot silently widen: a paste
    // carries a newline AND other bytes; a lone Enter is only the newline, and a
    // control chord has no newline at all.
    assert!(is_multiline_paste(b"aa\rbb"), "two lines is a paste");
    assert!(is_multiline_paste(b"aa\nbb"), "LF also counts");
    assert!(
        is_multiline_paste(b"aa\rbb\r"),
        "trailing newline is still a paste"
    );
    assert!(
        is_multiline_paste(b"\rfoo"),
        "leading newline is still a paste"
    );

    assert!(
        !is_multiline_paste(b"\r"),
        "a lone Enter is a keystroke — wrapping it would put every submit through handlePaste"
    );
    assert!(
        !is_multiline_paste(&[0x03]),
        "Ctrl+C has no newline and must stay raw, or handlePaste would strip it"
    );
    assert!(!is_multiline_paste(b"a"), "a single char is a keystroke");
    assert!(!is_multiline_paste(b""), "empty is not a paste");
    assert!(
        !is_multiline_paste(b"hello world"),
        "one line with no newline is a keystroke — and needs no wrapping"
    );
}
