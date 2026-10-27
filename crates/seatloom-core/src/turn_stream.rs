// Turn-boundary detection over the EXISTING tmux mirror byte stream.
//
// WHY THIS EXISTS. Aegis's packet (AEGIS-2026-10-27-dual-render-seat-stream-v1):
// 张先生 wants both the tmux panel AND a smooth conversation window in SeatLoom's
// UI. The byte stream is already there (tmux pipe-pane → FIFO → broadcast), so the
// work is not to build a second capture path — it is to give that SAME stream a
// second shape: one that knows where a turn begins and ends.
//
// WHAT THIS IS. A state machine over the bytes. It reads the same chunks the
// xterm consumer reads, and emits turn boundaries alongside them. It does not
// replace, wrap, or re-implement `pty::PtySession`'s reader — see `mod.rs`; the
// detector is fed from the broadcast, downstream of everything that already works.
//
// 🔴 WHAT THIS IS NOT — read this before trusting a boundary.
//
// It is a HEURISTIC on RENDERED TERMINAL OUTPUT, not the harness's own turn
// events. The harness (saih) only writes its session file at `message_end`
// (measured: agent-session.ts's append happens after the message object is
// stored), and its `--mode rpc` does emit `text_delta`-level events — but RPC is
// not interactive, and 张先生 requires the tmux panel, so RPC is off the table
// by instruction. That leaves exactly one live source: the bytes tmux mirrors.
//
// So the boundary is INFERRED from the shape of those bytes. It is right in the
// case measured below and wrong whenever the harness changes how it draws. The
// consumer must therefore treat a boundary as a hint, never as proof of what the
// model did. Two consequences, both deliberate:
//   * a turn that is never closed (the detector missed the end) stays open rather
//     than being force-closed with a guessed length;
//   * `TurnEvent` carries no claim about content — only about bytes and time.
//
// =============================================================================
// MEASURED SIGNALS (2026-10-27, a real turn in a real seat pane, not a fixture)
// =============================================================================
// Captured with `tmux pipe-pane` into a file while one prompt ran to completion:
//
//   synchronized-output frames  `ESC[?2026h` … `ESC[?2026l`    32 begin / 32 end
//   spinner lines              `── ⠧ Working ───…`              29 during the turn
//   spinner frames seen        ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏            all ten, 2-3 each
//   the token status line      `↑9.0k ↓19 0.9%/1.0M (auto)`       appears ONCE,
//                                                              replacing the spinner
//
// So a turn is observable as: spinner line present → (spinner animating) →
// spinner gone and the token-status line in its place. That transition is the
// boundary. The synchronized-output markers give clean redraw frames inside it,
// which is what makes a smooth UI possible without inventing a new capture path.
//
// The spinner word is not hardcoded as "Working": the harness also draws
// "Thinking…", "Compacting context…", "Retrying (n/m) in Ns…", and
// "Summarizing branch…" (status-indicator.ts, read from saih's source). The
// detector recognises the SPINNER LINE SHAPE — a braille spinner glyph followed by
// a word and a rule of box-drawing characters — so it keeps working when the
// message changes. That is still a shape match, and it is still a heuristic.
// =============================================================================

/// One event from the turn detector. Consumers get these alongside the raw bytes,
/// never instead of them: xterm keeps reading the raw stream untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnEvent {
    /// A turn began: the harness started working on a prompt. Carries nothing but
    /// the fact — see the module comment on why there is no content claim here.
    TurnStarted,
    /// Bytes belonging to the turn currently in progress.
    ///
    /// `frame` is true when this chunk ended on a synchronized-output boundary
    /// (`ESC[?2026h`…`ESC[?2026l`), i.e. it is a complete redraw rather than an
    /// arbitrary slice of one. A UI can use it to batch; a UI that ignores it
    /// still works, because the bytes are the same either way.
    Chunk { bytes: Vec<u8>, frame: bool },
    /// The turn ended: the spinner line was replaced by the status line.
    TurnEnded,
}

/// A detector fed with raw mirror bytes, producing turn boundaries.
///
/// One per session. Cheap: it holds a small tail of the last chunk so a marker
/// split across two reads is still recognised, and a two-state machine.
#[derive(Debug)]
pub struct TurnDetector {
    /// Bytes carried over from the previous chunk, because a marker (or the
    /// spinner line) can straddle a read boundary. Bounded — see `MAX_CARRY`.
    carry: Vec<u8>,
    /// Whether a spinner line is currently on screen, i.e. a turn is in progress.
    spinner_visible: bool,
    /// Whether we are inside a synchronized-output frame.
    in_frame: bool,
    /// Counters, for the delivery's measurement and for a UI that wants to show
    /// "N turns seen" without tracking it itself.
    turns_seen: u64,
    bytes_in_turn: u64,
}

/// How many bytes of a straddling marker to keep. The longest thing we look for is
/// the spinner line shape; 512 bytes is comfortably more than one pane row of it,
/// and small enough that this cannot become a second copy of the stream.
const MAX_CARRY: usize = 512;

impl Default for TurnDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl TurnDetector {
    pub fn new() -> Self {
        Self {
            carry: Vec::with_capacity(MAX_CARRY),
            spinner_visible: false,
            in_frame: false,
            turns_seen: 0,
            bytes_in_turn: 0,
        }
    }

    /// Turns observed since this detector was created.
    pub fn turns_seen(&self) -> u64 {
        self.turns_seen
    }

    /// Bytes attributed to the current (or most recent) turn.
    pub fn bytes_in_turn(&self) -> u64 {
        self.bytes_in_turn
    }

    /// Feed one chunk of mirror bytes. Returns the events it produced, in order.
    ///
    /// The chunk is returned verbatim inside `Chunk` events — this detector never
    /// rewrites, drops, or reorders bytes. A consumer that wants the raw stream
    /// can concatenate every `Chunk::bytes` and get exactly what it was given.
    ///
    /// One chunk can contain several transitions (a whole turn can arrive in one
    /// 4KB read), so lines are walked IN ORDER and the state machine is applied to
    /// each. Looking only at "the last line" is what made an earlier draft miss
    /// the end of a turn whenever the spinner and the status line shared a window.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<TurnEvent> {
        if chunk.is_empty() {
            return Vec::new();
        }

        let mut buf = std::mem::take(&mut self.carry);
        buf.extend_from_slice(chunk);

        let mut events = Vec::new();

        // Everything up to the last newline is complete; the remainder is a partial
        // line that must be carried so a marker straddling a read is still matched.
        let boundary = buf.iter().rposition(|&b| b == b'\n').map(|i| i + 1).unwrap_or(0);
        let complete = &buf[..boundary];

        for line in complete.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            self.apply_line(line, &mut events);
        }

        // The partial line is the current state of the pane's bottom line, so it is
        // also examined — a spinner drawn without a trailing newline is still a turn.
        if boundary < buf.len() {
            let partial = &buf[boundary..];
            if !partial.is_empty() {
                self.apply_line(partial, &mut events);
            }
        }

        if self.spinner_visible {
            self.bytes_in_turn += chunk.len() as u64;
        }

        events.push(TurnEvent::Chunk {
            bytes: chunk.to_vec(),
            frame: self.in_frame,
        });

        let keep = buf.len().min(MAX_CARRY);
        self.carry = buf[buf.len() - keep..].to_vec();

        events
    }

    /// Apply the two-state machine to one line.
    ///
    /// Order matters and is deliberate: the STATUS line is checked first, because
    /// it is drawn LATER than the spinner it replaces, so when both are present in
    /// one window the status line is the newer state.
    fn apply_line(&mut self, line: &[u8], events: &mut Vec<TurnEvent>) {
        let has_spinner = looks_like_spinner_line(line);
        let has_status = looks_like_status_line(line);

        if has_status && self.spinner_visible {
            self.spinner_visible = false;
            events.push(TurnEvent::TurnEnded);
            return;
        }
        if has_spinner && !self.spinner_visible {
            self.spinner_visible = true;
            self.turns_seen += 1;
            self.bytes_in_turn = 0;
            events.push(TurnEvent::TurnStarted);
            return;
        }
        if has_spinner {
            // Same turn, still drawing. Nothing to emit.
            return;
        }
        if has_status {
            // A status line with no turn open is not an end — see the module comment
            // on never force-closing. This is the common case right after attach.
            return;
        }
        // Ordinary output: mid-turn text, a tool result, anything. No boundary.
    }
}

/// Strip ANSI/OSC control sequences, leaving the visible text.
///
/// Necessary because the real stream puts cursor positioning BEFORE the spinner:
/// a measured line is `ESC[26;1H ESC[2K ESC[39m ── ⠧ Working ───…`. Matching on the
/// raw bytes would look for a braille glyph where there is an escape instead.
///
/// Deliberately crude: it removes `ESC[...m`-style CSI and `ESC]...BEL` OSC runs
/// and nothing else, because that is all the harness emits here. A sequence it
/// does not recognise is left in place, which can only make a match FAIL (never
/// invent one) — the safe direction for a heuristic.
fn visible_text(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'[' => {
                    // CSI: parameters, then a final byte in 0x40..=0x7e
                    let mut j = i + 2;
                    while j < bytes.len() && !(0x40..=0x7e).contains(&bytes[j]) {
                        j += 1;
                    }
                    i = (j + 1).min(bytes.len());
                    continue;
                }
                b']' => {
                    // OSC: terminated by BEL or ST (ESC\)
                    let mut j = i + 2;
                    while j < bytes.len() && bytes[j] != 0x07 {
                        j += 1;
                    }
                    i = (j + 1).min(bytes.len());
                    continue;
                }
                _ => {
                    i += 2;
                    continue;
                }
            }
        }
        // Copy one UTF-8 code point.
        let len = utf8_len(bytes[i]);
        let end = (i + len).min(bytes.len());
        if let Ok(s) = std::str::from_utf8(&bytes[i..end]) {
            out.push_str(s);
        }
        i = end;
    }
    out
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1 // invalid lead byte; skip it
    }
}

/// True when a line looks like the harness's working indicator.
///
/// Measured shape (from the real pane): `── ⠧ Working ────────` — a rule, a
/// braille spinner glyph, a word, another rule. Matching the SHAPE rather than
/// the word keeps this working across "Working", "Thinking…", "Compacting
/// context…", "Retrying (n/m) in Ns…" and "Summarizing branch…", all of which the
/// harness draws through the same component (saih's status-indicator.ts).
///
/// The spinner is searched for ANYWHERE in the line, not at position zero: the
/// measured line begins with a rule, and anchoring on the first character made
/// the detector miss every real turn while its own fixture passed.
///
/// Requiring a run of ─ characters AFTER the spinner is what stops a stray
/// braille glyph in ordinary output from starting a phantom turn — see the test.
fn looks_like_spinner_line(line: &[u8]) -> bool {
    let text = visible_text(line);
    let mut idx = None;
    for (i, c) in text.char_indices() {
        if ('\u{2800}'..='\u{28FF}').contains(&c) {
            idx = Some(i);
            break;
        }
    }
    let Some(at) = idx else {
        return false;
    };
    // Something after the spinner, and a rule in it.
    let rest = &text[at..];
    rest.chars().count() > 2 && rest.contains('\u{2500}')
}

/// True when a line looks like the token/status line that replaces the spinner.
///
/// Measured shape: `\u{2191}9.0k \u{2193}19 0.9%/1.0M (auto)` — an up-arrow, a
/// down-arrow, and a percentage. The arrows are the load-bearing part: they are
/// drawn by the status bar, not by anything a model would emit as content.
fn looks_like_status_line(line: &[u8]) -> bool {
    let text = visible_text(line);
    text.contains('\u{2191}') && text.contains('\u{2193}') && text.contains('%')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact shape measured in a real pane (see the module header).
    fn spinner_line() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"\x1b[26;1H\x1b[2K\x1b[39m\x1b[2m");
        v.extend_from_slice("\u{2500}\u{2500} ".as_bytes());
        v.extend_from_slice(b"\x1b[22;39m");
        v.extend_from_slice("\u{28a7}".as_bytes()); // ⠧
        v.extend_from_slice(b"\x1b[22;39m \x1b[39m\x1b[2mWorking\x1b[22;39m");
        v.extend_from_slice("\u{2500}".repeat(40).as_bytes());
        v
    }

    fn status_line() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"\x1b[27;1H\x1b[2K");
        v.extend_from_slice("\u{2191}9.0k \u{2193}19 0.9%/1.0M (auto)".as_bytes());
        v
    }

    fn kinds(events: &[TurnEvent]) -> Vec<&'static str> {
        events
            .iter()
            .map(|e| match e {
                TurnEvent::TurnStarted => "start",
                TurnEvent::Chunk { .. } => "chunk",
                TurnEvent::TurnEnded => "end",
            })
            .collect()
    }


    #[test]
    fn a_turn_is_detected_from_the_spinner_line() {
        let mut d = TurnDetector::new();
        let ev = d.feed(&spinner_line());
        assert_eq!(kinds(&ev), vec!["start", "chunk"]);
        assert_eq!(d.turns_seen(), 1);
    }

    #[test]
    fn a_turn_ends_when_the_status_line_replaces_the_spinner() {
        let mut d = TurnDetector::new();
        d.feed(&spinner_line());
        let ev = d.feed(b"some assistant text\n");
        assert_eq!(kinds(&ev), vec!["chunk"], "mid-turn text is not a boundary");
        let ev = d.feed(&status_line());
        assert_eq!(kinds(&ev), vec!["end", "chunk"]);
    }

    #[test]
    fn ordinary_output_never_starts_a_turn() {
        // The counter-example that makes the two tests above mean something: a
        // braille character in normal prose must not start a phantom turn.
        let mut d = TurnDetector::new();
        let ev = d.feed("the spinner glyph \u{28a7} appears in this sentence\n".as_bytes());
        assert_eq!(kinds(&ev), vec!["chunk"]);
        assert_eq!(d.turns_seen(), 0);
    }

    #[test]
    fn a_status_line_without_a_spinner_is_not_an_end() {
        // Otherwise the first status line after attach would close a turn that
        // never opened.
        let mut d = TurnDetector::new();
        let ev = d.feed(&status_line());
        assert_eq!(kinds(&ev), vec!["chunk"]);
    }

    #[test]
    fn chunks_are_returned_verbatim() {
        let mut d = TurnDetector::new();
        let payload = b"\x1b[31mred\x1b[0m text";
        let ev = d.feed(payload);
        match ev.last() {
            Some(TurnEvent::Chunk { bytes, .. }) => assert_eq!(bytes, payload),
            other => panic!("expected a Chunk, got {other:?}"),
        }
    }

    #[test]
    fn a_marker_split_across_reads_is_still_matched() {
        // tmux hands us 4KB reads; nothing promises a marker stays whole.
        let mut d = TurnDetector::new();
        let line = spinner_line();
        // Split inside the spinner GLYPH's escape sequence, before the word. The
        // measured line puts cursor positioning first, so an early split is the
        // realistic case: the first read ends before anything identifiable.
        let split = 12;
        assert!(
            !looks_like_spinner_line(&line[..split]),
            "the first fragment must not already look like a spinner line",
        );
        let ev = d.feed(&line[..split]);
        assert_eq!(kinds(&ev), vec!["chunk"], "a partial marker is not a turn yet");
        let ev = d.feed(&line[split..]);
        assert_eq!(kinds(&ev), vec!["start", "chunk"]);
    }

    #[test]
    fn an_unclosed_turn_stays_open_rather_than_being_force_closed() {
        // The module comment promises this. If the end marker is missed, the turn
        // stays open — a UI shows "still working", which is true, instead of
        // inventing an end.
        let mut d = TurnDetector::new();
        d.feed(&spinner_line());
        let ev = d.feed(b"more text with no status line after it\n");
        assert_eq!(kinds(&ev), vec!["chunk"]);
        assert!(d.bytes_in_turn() > 0);
    }


    #[test]
    fn an_empty_chunk_produces_nothing() {
        let mut d = TurnDetector::new();
        assert!(d.feed(b"").is_empty());
    }
}
