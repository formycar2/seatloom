# Task: B1 — PtySession write path via tmux send-keys (v0.0.2)

[Lyra -> Nimbus] B1 opens the bidirectional write path on top of A3's read-only mirror. SessionTerminal keystrokes flow xterm → cmd_pty_write_bytes → PtySession::write → `tmux send-keys -t <target>`. No new process, no FIFO change, no R3 compromise — send-keys is observe-level control, not ownership.

| Field | Value |
|---|---|
| template | T4 |
| subtype | implementation |
| id | NIMBUS-2026-05-09-b1-pty-write-send-keys-v1 |
| status | **dispatched** (restored 2026-05-09 late-evening — Aegis correction to joint-review ruling: B1 bytes-only has no `canonical_events` write, not blocked by 008) |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | nimbus |
| priority | P0 (SG-B entrypoint) |
| milestone | v0.0.2 |
| dispatched_at | 2026-05-09 late-evening |
| gate_cleared_by | A4-β PASS @ `ab672e5` (Mira delivery); Flux Layer A verify PASS; Lyra acceptance `docs/coordination/acceptance/2026-05-09-lyra-mira-a4b-sessions-real-api-wire-acceptance.md` |
| depends_on | A3 UNCONDITIONAL PASS @ `fca4fe0` (read path); A4-β PASS @ `ab672e5` (real-API wire-up); `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (architecture review) |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-delivery-v1.md` |
| tags | nimbus, B1, SG-B, tmux, send-keys, bidirectional, v0.0.2 |

> **Dispatch note (2026-05-09 late-evening, Aegis-corrected)**: B1 bytes-only scope writes zero `canonical_events` rows — my earlier cross-point (c) claim that B1 required 008 precedence was factually wrong. Nimbus's scope read was correct: B1 touches `PtySession::write` + `cmd_pty_write_bytes` + smoke steps + SessionTerminal.tsx + SessionsWorkspace.tsx, none of which emit events. Aegis final ruling: **B1 unblocked; 008 dispatched in parallel as a standalone packet but does not gate B1 acceptance.** Nimbus cleared to resume B1 implementation. Gate-cleared-by metadata above (A4-β acceptance + Flux Layer A delivery) stands unchanged.

---

## 1. Context

A3 (`fca4fe0`) landed the read path: `tmux pipe-pane` → FIFO → Tauri event stream → xterm display. Everything on the return leg is still stubbed out per v0.0.1 read-only scope:

- `PtySession::write(_bytes: &[u8])` at `crates/seatloom-core/src/pty/mod.rs:324` returns `Ok(())`, parameters underscore-prefixed.
- `cmd_pty_write` at `src-tauri/src/commands/session_cmds.rs:270` returns `Ok(())`.
- `cmd_pty_write_bytes` at `src-tauri/src/commands/session_cmds.rs:281` returns `Ok(())`.
- `SessionTerminal.tsx:66` sets `disableStdin: true`; the keystroke forwarding is marked `TODO(B2)` at line 80 but belongs to B1 per Aegis directive (the B1/B2 split collapses into this single write-path packet).

A3's R3 failure-isolation contract — "SeatLoom observes, never owns" — stays intact in B1. `tmux send-keys -t <target> …` is a client command against the existing tmux server process; SeatLoom never becomes tmux's parent, and a crashing SeatLoom leaves the tmux session untouched exactly as before.

## 2. Scope — backend

### 2.1 `crates/seatloom-core/src/pty/mod.rs`

Replace the `PtySession::write` no-op at line 324 with a real `tmux send-keys` invocation against `self.tmux_target`.

**Wire contract**: `tmux send-keys -t <target> -l -- <literal-bytes>`.
- `-l` (literal): send the bytes verbatim without interpreting them as tmux key names (`C-c`, `Enter`, etc.). We forward raw xterm keystrokes including control sequences; the terminal inside tmux interprets them, not tmux itself. This is the correct choice for a terminal-mirror UX — user types `Ctrl-C`, xterm emits `0x03`, we deliver `0x03` into the pane as a byte, bash receives SIGINT. Without `-l`, tmux would try to parse the string as a key name and reject it.
- `--`: argument terminator, guarantees literal text starting with `-` is not parsed as a flag.

**stdin delivery**: tmux `send-keys` takes the payload as an argv string. For raw bytes including NUL or invalid UTF-8 we must avoid argv entirely. The robust path is to pipe bytes through `tmux load-buffer -` then `tmux paste-buffer -t <target> -p` (paste into pane). Recommend the two-step buffer path as the primary implementation — it handles arbitrary byte sequences and removes the argv-escaping headache. Document this choice in the delivery doc; if you choose the argv path instead, justify with test evidence that all control bytes round-trip correctly.

**Signature**: keep `pub fn write(&self, bytes: &[u8]) -> Result<(), PtyError>`. Drop the underscore prefix on the parameter name. Non-empty-bytes guard optional.

**Error surface**: add a new `PtyError::Tmux(..)` variant path parallel to `kill` / `resize` (already used). If `tmux load-buffer` or `tmux paste-buffer` exits non-zero, propagate `stderr` verbatim in the error.

**Shutdown guard**: check `self.shutdown` at function entry; return `Ok(())` silently if set (same policy as the existing read loop — no errors on a racing unmount).

**Update tests**: the existing 4 `#[test]` functions at `pty/mod.rs:394-415` are read-side. Add at minimum:
1. `write_noop_on_shutdown` — set the atomic, call `write(b"x")`, assert `Ok(())` and zero side effects.
2. `write_uses_tmux_target_buffer_path` — behavioral shape only; no tmux subprocess needed, verify the command path. (If this requires refactoring `write` to call an injectable `tmux_runner`, that's a reasonable extraction; otherwise leave as a doc-test skeleton with `#[ignore]`.)

### 2.2 `src-tauri/src/commands/session_cmds.rs`

Replace the two no-ops:

**`cmd_pty_write` at line 270** — map `data: String` → `data.as_bytes()` → `session.write(bytes)`:

```rust
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
```

**`cmd_pty_write_bytes` at line 281** — the direct byte path used by xterm binary input (paste of arbitrary data):

```rust
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
```

Remove the underscore prefixes. Remove the comments "B1 (v0.0.2) will map this to tmux send-keys" — B1 is this packet.

### 2.3 Smoke example update

`crates/seatloom-core/examples/tmux_mirror_smoke.rs` currently exercises the read path + R3 isolation (9 steps). Add B1 steps:

- **Step 10** — `session.write(b"echo SMOKE_B1_ROUND_TRIP\n")`, then poll the FIFO-backed byte stream for `SMOKE_B1_ROUND_TRIP`. Fail with `B1_WRITE_TIMEOUT` if the round-trip doesn't complete inside 2 seconds.
- **Step 11** — `session.write(b"\x03")` (Ctrl-C), confirm tmux pane receives the signal (observable via a subsequent `tmux display-message -p '#{pane_in_mode}'` probe, or a simpler: write `sleep 30 &`, then Ctrl-C, then `wait` emits `Terminated`).
- **Step 12** — After write-path exercise, re-run the existing R3 assertion: `session.kill()`, then `tmux has-session -t <name>` must still return success. B1 must not regress R3.

Print `SMOKE_B1_PASS` on success. The existing `R3 OK: tmux session still running after kill_session` line must still print — B1 does not loosen R3.

## 3. Scope — frontend

### 3.1 `ui/src/app-v2/panel/SessionTerminal.tsx`

Three edits:

1. **Line 64-66** — flip `cursorBlink: false` → `cursorBlink: true`, delete `disableStdin: true` (defaults to false, i.e., stdin enabled).
2. **Line 80** — delete the `TODO(B2)` comment.
3. **After line 80, before the ResizeObserver block** — wire keystrokes to `cmd_pty_write_bytes`. Use `term.onData` (the xterm event for "user typed something, here are the bytes"):

```tsx
// B1 (v0.0.2): forward xterm keystrokes to the tmux pane via send-keys.
const dataListener = term.onData((data: string) => {
  // xterm gives us a UTF-8 string including control sequences (ESC, 0x03, etc.).
  // Encode to bytes so control characters round-trip exactly.
  const bytes = new TextEncoder().encode(data);
  api.ptyWriteBytes(sessionId, Array.from(bytes)).catch((err) => {
    console.error('[SessionTerminal] ptyWriteBytes failed:', err);
  });
});
```

Add `dataListener.dispose()` to the cleanup returned at line 116.

Prefer `onData` over `onKey` — `onData` gives the already-decoded byte stream including paste, IME composition, and control chords; `onKey` fires per-keypress and requires manual paste handling. We want keystroke fidelity, not keypress semantics.

### 3.2 SessionsWorkspace read-only banner

`ui/src/app-v2/panel/SessionsWorkspace.tsx:195-197` currently shows "Read-only mode (v0.0.1). Typing in this terminal is disabled."

After B1, flip the banner message or remove it. Recommended: replace with a softer advisory preserving the R3 mental model, not a prohibition:

```tsx
<div style={{ background: '#DBEAFE', padding: '8px', fontSize: 12, color: '#1E40AF', flexShrink: 0 }}>
  ℹ️ Bidirectional mirror (v0.0.2). Keystrokes are forwarded to the tmux pane via send-keys.
  SeatLoom never owns the tmux session — closing this tab detaches the mirror only.
</div>
```

Note the color shift: amber (warning) → blue (info). The warning was accurate in v0.0.1 because typing silently did nothing; in v0.0.2 typing works, so the warning message becomes misleading.

### 3.3 Compliance banner — v0.0.1 pill

`SessionsWorkspace.tsx:132-134` currently reads "v0.0.1 attach-only · read-only mirror". Flip to "v0.0.2 attach-only · bidirectional mirror". Keep "attach-only" — R1 is unchanged: SeatLoom still never spawns a CLI process, it only sends keystrokes into an existing tmux session.

## 4. R-rule invariants — must-hold

Copy verbatim into delivery doc §Compliance with PASS + line evidence for each:

| Rule | B1 behavior | Where verified |
|---|---|---|
| R1 attach-only | `PtySession::write` still invokes tmux subprocesses only (`tmux load-buffer` + `tmux paste-buffer`, or `tmux send-keys`). Zero new PTY spawns, zero shell forks, zero process ownership change. | `pty/mod.rs` write impl |
| R3 failure-isolation | `kill()` path at `pty/mod.rs:356` unchanged. `write()` does not open FIFOs, does not hold tmux as a child, does not change ownership. Smoke Step 12 must print `R3 OK` unchanged. | `pty/mod.rs:356`; smoke Step 12 |
| transition shim | `cmd_launch_session` at `session_cmds.rs:241` stays `Err` on non-tmux runtimes. B1 does not re-enable the legacy PTY path. | `session_cmds.rs:241` |
| v0.0.2 scope | write path is xterm → tmux pane only. No new "launch" surfaces, no seat-spawn, no process orchestration. | SessionsWorkspace, SessionTerminal |
| observability | zero new structured log channels; existing `console.error` for IPC failures is sufficient. | SessionTerminal error branch |

## 5. Verification plan

### 5.1 Layer A (Nimbus runs, Flux verifies)

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test -p seatloom-core --lib pty::  # 4 existing + ≥1 new
cargo clippy -p seatloom-core -- -D warnings
cargo run --example tmux_mirror_smoke -- b1-smoke-seatloom
# expect: SMOKE_PASS + SMOKE_B1_PASS + R3 OK
cd ui
pnpm exec tsc --noEmit
pnpm build
```

Paste all output verbatim into delivery doc.

### 5.2 Layer B (Mr. Zhang or Flux tauri-dev seat)

1. `pnpm tauri dev`, Supervisor IM surface → Sessions workspace → pick your own tmux session → Attach.
2. Type `echo hello` + Enter → `hello` appears in xterm.
3. Press `Ctrl-C` mid-command → current command interrupts.
4. Paste a multi-line block → each line lands in the pane.
5. Close tab → pane survives, other attached tmux clients (terminal app) see no disruption.
6. Ctrl-C SeatLoom (kill main window) → tmux session still up; reopen SeatLoom → session re-listable via `cmd_list_tmux_sessions` → re-attachable.

Observation 6 is the R3 regression check.

### 5.3 Known gaps — carry-forward to future packets (not B1 blockers)

- `load-buffer` + `paste-buffer` pattern loses paste-buffer contents the user had in tmux. A ring-buffer rotation or dedicated SeatLoom paste-buffer name (`-b seatloom`) avoids this — recommend `-b seatloom` on both commands.
- `send-keys` cannot currently send a literal `0x00` NUL byte through argv; `paste-buffer` can via `load-buffer -`. This is why we prefer the buffer path.
- No per-keystroke rate limit. If it becomes a problem, add a small coalescing window client-side; not required for v0.0.2.

## 6. Delivery doc requirements

Write to `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-delivery-v1.md`. Must contain:

1. T3 frontmatter (template, subtype=implementation_delivery, status=delivered, commit=<sha>, packet_ref=this packet).
2. `git show --stat <sha>` verbatim. Expected scope: `crates/seatloom-core/src/pty/mod.rs` + `crates/seatloom-core/examples/tmux_mirror_smoke.rs` + `src-tauri/src/commands/session_cmds.rs` + `ui/src/app-v2/panel/SessionTerminal.tsx` + `ui/src/app-v2/panel/SessionsWorkspace.tsx`. Five files. Zero unrelated.
3. All §5.1 Layer A outputs verbatim (cargo check/test/clippy/build, pnpm tsc/build, smoke runs with `SMOKE_B1_PASS` + `R3 OK`).
4. §4 compliance checklist with line-pinned evidence.
5. Architectural note: argv vs buffer path decision, why chosen, round-trip evidence (screenshot or byte-dump of the Ctrl-C test).
6. Known limits list (per §5.3) acknowledged.
7. If A4-β is not yet Layer-A-PASS when you read this, state "Draft only — awaiting dispatch trigger" as the top line.

## 7. Acceptance criteria (Lyra)

1. Five files touched, zero others.
2. `PtySession::write` invokes tmux through buffer path (or argv path with round-trip evidence); parameter is `bytes: &[u8]` not `_bytes`.
3. Both `cmd_pty_write` + `cmd_pty_write_bytes` reach `session.write()` with correct byte mapping.
4. `SessionTerminal` no longer sets `disableStdin: true`; `term.onData` wires keystrokes through `api.ptyWriteBytes`.
5. SessionsWorkspace read-only banner + v0.0.1 pill flipped to v0.0.2 messaging.
6. R3 smoke Step 12 prints `R3 OK` unchanged — send-keys does not escalate ownership.
7. cargo check + test + clippy all green; pnpm tsc zero errors; pnpm build exit 0.
8. Flux Layer A commit-pinned verify issues PASS (delivery → pre-flight → verify → acceptance sequence unchanged).
9. Layer B runtime observation confirms keystroke round-trip + Ctrl-C signal delivery + close-tab-preserves-session.

## 8. Scope boundaries (do not expand)

- Do **not** reintroduce `portable-pty` or any spawn-local-CLI path. v0.0.2 is still attach-only.
- Do **not** add a "send command" textbox, macro recorder, or any UI that queues keystrokes outside the terminal emulator — the only input surface is xterm itself.
- Do **not** alter `cmd_launch_session`, `cmd_list_tmux_sessions`, `cmd_attach_tmux_session`, `cmd_pty_resize`, `cmd_kill_session`, or the read-path FIFO plumbing.
- Do **not** add seat-role handshake, prompt-injection, or supervisor-pipeline wiring. Those are post-SG-B packets.
- Do **not** change transcript capture semantics.

## 9. Timing

**Draft dispatched 2026-05-09 evening** per Aegis directive, gated on A4-β Flux Layer A PASS. Nimbus may read the packet and plan mental model / buffer-vs-argv decision now. Do not commit code until Lyra sends "B1 GO". If A4-β surfaces any unexpected write-path implications during Flux verify, this packet gets a v2 before dispatch.

---

*Drafted by Lyra · 2026-05-09 evening · Aegis-confirmed scope · gated on A4-β Layer A PASS · SG-B entrypoint*
