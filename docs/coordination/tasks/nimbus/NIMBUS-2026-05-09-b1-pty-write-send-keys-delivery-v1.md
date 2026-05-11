# Delivery: B1 — PtySession write path via tmux buffer path (v0.0.2)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | NIMBUS-2026-05-09-b1-pty-write-send-keys-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-05-11 |
| version | v1 |
| to | flux (Layer A verify), lyra (acceptance) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` |
| commit | `0c0f425` |
| milestone | v0.0.2 SG-B entry |
| depends_on | A3 `fca4fe0` (read path); A4-β `ab672e5` (real tmux IPC); Aegis 2026-05-11 ruling (bytes-only, B1 ⊥ 008) |
| tags | nimbus, B1, SG-B, tmux, bidirectional, v0.0.2, delivery |

---

## 1. Summary

B1 commit `0c0f425` replaces the v0.0.1 `PtySession::write` no-op with a real tmux write path via `tmux load-buffer -b seatloom - ; paste-buffer -b seatloom -t <target>` (single invocation, `;` command chaining). `cmd_pty_write` and `cmd_pty_write_bytes` are wired to `session.write()`. `SessionTerminal` forwards xterm `onData` through `api.ptyWriteBytes`. `SessionsWorkspace` chrome flipped amber/v0.0.1/"Read-only" → blue/v0.0.2/"Bidirectional mode".

Behaviour contract: arbitrary bytes (NUL, Ctrl-C 0x03, ANSI control, UTF-8) round-trip into the tmux pane. R1 attach-only preserved (SeatLoom never becomes tmux's parent). R3 failure isolation re-asserted after the write path is exercised (smoke Step 12).

## 2. Scope — 5 files, zero unrelated

```
 crates/seatloom-core/examples/tmux_mirror_smoke.rs | 89 +++++++++++++++++++++-
 crates/seatloom-core/src/pty/mod.rs                | 83 ++++++++++++++++++--
 src-tauri/src/commands/session_cmds.rs             | 33 +++++---
 ui/src/app-v2/panel/SessionTerminal.tsx            | 17 ++++-
 ui/src/app-v2/panel/SessionsWorkspace.tsx          | 17 +++--
 5 files changed, 209 insertions(+), 30 deletions(-)
```

Matches packet §2 scope exactly. No other files touched.

## 3. Architectural decision — buffer path chosen over argv send-keys

### Decision: single tmux invocation with `;` chaining

```rust
Command::new("tmux")
    .args([
        "load-buffer", "-b", "seatloom", "-",
        ";",
        "paste-buffer", "-b", "seatloom", "-t", &self.tmux_target,
    ])
    .stdin(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
```

Bytes are written to the child's stdin (closed on drop = EOF = load-buffer terminates), then `wait_with_output()` collects stderr on non-zero exit.

### Rationale over `tmux send-keys -l -- <string>`

| Axis | `send-keys -l` (argv) | `load-buffer|paste-buffer` (stdin) |
|---|---|---|
| NUL `0x00` | ✗ Rust `OsStr`/`CStr` rejects interior NUL | ✓ Delivered through stdin pipe |
| Large pastes (>`ARG_MAX`) | ✗ Hits OS argv limit (~1 MB on macOS) | ✓ Unbounded stdin |
| Concurrent calls | Needs `Mutex<PtySession>` to serialize tmux socket ops | ✓ Single invocation = atomic within tmux |
| User paste-buffer preserved | N/A | ✓ `-b seatloom` is a named slot; default paste-buffer untouched |
| Fork-exec cost | 1× | 1× (via `;` chaining — not 2×) |

The `;` chaining trick is the key win over a naive two-step `load-buffer; paste-buffer` sequence: tmux serializes commands within a single invocation on its own socket, so two concurrent `write()` calls cannot interleave into `loadA → loadB → pasteA → pasteB`. Without this, a `Mutex` would be required on `PtySession`.

### Round-trip evidence

Smoke Step 10 writes `printf 'SEATLOOM_B1_ROUND_TRIP_OK\n'\n` (37 bytes) and observes the marker string echo back from the mirror within 2 s:

```
[smoke] B1 write(37 bytes) issued
[smoke] B1 round-trip OK: probe 'SEATLOOM_B1_ROUND_TRIP_OK' echoed back
```

Smoke Step 11 validates that `0x03` (Ctrl-C) is delivered as a real SIGINT to the foreground shell process, not interpreted by tmux as a key name:

```
[smoke] B1 Ctrl-C (0x03) sent to foreground `sleep 30`
[smoke] B1 Ctrl-C OK: shell resumed after SIGINT, post-interrupt marker echoed
```

The test is structured so that a broken SIGINT path would cause `sleep 30` to block the shell for 30 s, and the post-interrupt marker would not appear inside the 1.5 s budget — the test fails loudly rather than silently passing.

## 4. Acceptance criteria (packet §7) — line-pinned evidence

| # | Criterion | Evidence |
|---|---|---|
| 1 | Five files touched, zero others | `git show 0c0f425 --stat` (§2 above) |
| 2 | `PtySession::write` invokes tmux through buffer path; parameter is `bytes: &[u8]` not `_bytes` | `crates/seatloom-core/src/pty/mod.rs:324` — `pub fn write(&self, bytes: &[u8]) -> Result<(), PtyError>` |
| 3 | `cmd_pty_write` + `cmd_pty_write_bytes` reach `session.write()` | `src-tauri/src/commands/session_cmds.rs:270` (`sess.write(data.as_bytes())`) and `src-tauri/src/commands/session_cmds.rs:285` (`sess.write(&bytes)`) |
| 4 | `SessionTerminal` no longer sets `disableStdin: true`; `term.onData` wires through `api.ptyWriteBytes` | `ui/src/app-v2/panel/SessionTerminal.tsx:60-70` (Terminal options, no disableStdin), `:80-88` (`term.onData((data) => api.ptyWriteBytes(sessionId, Array.from(new TextEncoder().encode(data))))`) |
| 5 | SessionsWorkspace banner + pill flipped | `ui/src/app-v2/panel/SessionsWorkspace.tsx:123` (pill "v0.0.2 attach-only · bidirectional"), `:186` (banner blue/bidirectional) |
| 6 | R3 smoke Step 12 prints `R3 OK` unchanged | Smoke output below: `[smoke] R3 OK: tmux session b1-smoke-seatloom still running after kill` |
| 7 | cargo check + test + clippy + pnpm tsc + pnpm build | §5 Layer A outputs |
| 8 | Flux Layer A PASS | Pending Flux verify — this delivery triggers it |
| 9 | Layer B runtime observation | Pending Mr. Zhang live-run after Layer A PASS |

## 5. Layer A self-check outputs

### 5.1 `cargo check -p seatloom-core`

```
    Checking seatloom-core v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/crates/seatloom-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.69s
```

### 5.2 `cargo check -p seatloom-tauri`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.75s
```

3 pre-existing `dead_code` warnings on `src-tauri/src/dto.rs` (`PipelineRunDto`, `ReviewThreadDto`, `ReviewCommentDto`) — unrelated to B1, present on parent commit `b916912`.

### 5.3 `cargo test -p seatloom-core --lib pty::`

```
running 6 tests
test pty::tests::mirror_dir_is_tmp_seatloom_mirror ... ok
test pty::tests::shell_quote_handles_spaces_and_quotes ... ok
test pty::tests::default_transcripts_dir_nests_under_repo_root ... ok
test pty::tests::write_noop_on_empty_bytes ... ok
test pty::tests::write_noop_on_shutdown ... ok
test pty::tests::list_tmux_sessions_returns_vec_without_panic ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.01s
```

Two new tests added by B1: `write_noop_on_shutdown`, `write_noop_on_empty_bytes`.

### 5.4 `cargo clippy -p seatloom-core -- -D warnings`

**Fails on pre-existing `doc_lazy_continuation` lints in `crates/seatloom-core/src/db/repositories.rs` (3 errors)**. Confirmed present on parent commit `b916912` without this change — NOT a B1 regression. Recommend a standalone doc-lint cleanup packet; out of B1 scope.

Output tail:

```
   --> crates/seatloom-core/src/db/repositories.rs:373:9
    |
373 |     /// Sort: occurred_at ascending (chat-oriented).
    |         ^
    |
    = help: if this is supposed to be its own paragraph, add a blank line
```

### 5.5 `pnpm tsc --noEmit`

```
exit=0
```

Zero errors.

### 5.6 `pnpm build`

```
vite v6.4.2 building for production...
✓ 1594 modules transformed.
dist/index.html                             0.57 kB │ gzip:   0.34 kB
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/assets/SupervisorPanel-BYlfx6MH.css   47.96 kB │ gzip:   9.13 kB
dist/assets/supervisor-CXH-R9FB.js          0.86 kB │ gzip:   0.56 kB
dist/assets/main-DPQ-58Bb.js              214.73 kB │ gzip:  48.97 kB
dist/assets/SupervisorPanel-Y1qH2-MH.js   352.78 kB │ gzip: 102.94 kB
✓ built in 1.46s
```

### 5.7 `cargo run --example tmux_mirror_smoke -- b1-smoke-seatloom`

Verbatim output (headless tmux session `b1-smoke-seatloom` created + torn down per run):

```
[smoke] target tmux session = b1-smoke-seatloom
[smoke] list_tmux_sessions() returned 6 sessions (filtered by -seatloom suffix)
  - Flux-Quality&Ops-seatloom ...
  - Lyra-po-seatloom ...
  - Mira-UX/UED-seatloom ...
  - Nimbus-TechArchi-seatloom ...
  - Onyx-data-seatloom ...
  - b1-smoke-seatloom ...
[smoke] attached: id=smoke-001, fifo=/tmp/seatloom-mirror/smoke-001.fifo, tmux_target=b1-smoke-seatloom:0
[smoke] received 2284 bytes from mirror
[smoke] probe_marker_seen=true ls_output_seen=true
[smoke] resize(24,120) OK
[smoke] B1 write(37 bytes) issued
[smoke] B1 round-trip OK: probe 'SEATLOOM_B1_ROUND_TRIP_OK' echoed back
[smoke] B1 Ctrl-C (0x03) sent to foreground `sleep 30`
[smoke] B1 Ctrl-C OK: shell resumed after SIGINT, post-interrupt marker echoed
[smoke] kill OK: fifo unlinked at /tmp/seatloom-mirror/smoke-001.fifo
[smoke] R3 OK: tmux session b1-smoke-seatloom still running after kill
SMOKE_PASS probe=true ls=true bytes=2284
SMOKE_B1_PASS round_trip=true ctrl_c=true
```

All 12 steps PASS. Both `SMOKE_PASS` and `SMOKE_B1_PASS` markers emitted.

## 6. Known limits (carry-forward, not B1 blockers)

Per packet §5.3:

- **Buffer-path paste vs bracketed paste** — raw bytes delivered as if typed. Bracketed-paste-aware apps (`vim` with `:set paste`, `bash` with `bracketed-paste` readline) won't see `\e[200~` / `\e[201~` delimiters on large pastes, so auto-indent-on-paste degrades to per-char processing. Refinable later via xterm `onPaste` hook + explicit bracketed-paste wrapping on write. Not in B1 scope.
- **`-b seatloom` buffer shared across sessions** — all B1 writes use the same named buffer. For a single user on a single machine this is a non-issue (serialized through tmux's own command queue), but if two `PtySession`s write concurrently to different panes, the `;`-chained atomicity inside one process is lost across two processes. Today `seatloom-tauri` has exactly one process, so no issue; flagged for multi-process future.
- **No per-keystroke rate limit** — if it becomes a problem, add client-side coalescing in `SessionTerminal`. Not required for v0.0.2.
- **A4-β closeTab FIFO / pipe-pane residue** — carry-forward from A4-β acceptance, **not touched in B1** per Lyra's dispatch note ("不在 B1 scope"). Needs a sibling packet.

## 7. Observations and flag-ups

### 7.1 Pre-existing clippy failures in `db/repositories.rs`

`cargo clippy -p seatloom-core -- -D warnings` fails with 3 `doc_lazy_continuation` errors. Validated same failures on parent commit `b916912` via `git stash; clippy; stash pop`. Not a B1 regression. Recommend a 1-file cleanup packet.

### 7.2 Stray file in my working tree — `NIMBUS-2026-05-09-schema-008-project-isolation-v1.md`

An unpushed Lyra-authored 008 dispatch packet exists in my working tree (staged by git-stash/pop side-effect during the hold window). I caught this before pushing and **unstaged it** from the B1 commit — the final `0c0f425` is 5 files only as required. The 008 packet itself is a separate artifact whose status is now ambiguous given Aegis's 2026-05-11 "B1 ⊥ 008, no dependency" ruling superseded its "blocks B1" premise. Flagging for Lyra/Aegis to decide whether the packet is:

- (a) superseded and should be deleted,
- (b) revised and re-dispatched for 008-as-independent-migration,
- (c) left as-is for historical record.

I will not touch it without guidance.

### 7.3 Framing reminder from reciprocal review

Per Aegis's binding ratification of Lyra §1.3 ("FS = artifact truth, PG = runtime-object projection"), the B1 write path sits cleanly under that contract: bytes go FS-style (tmux pane = pseudo-fs artifact) and no `canonical_events` row is emitted (PG projection authority untouched). Audit-event (`SeatInputInjected`-style) emission is deferred to v0.0.3+ observability work per the same ruling.

## 8. Next steps

1. **Flux**: Layer A commit-pinned verify against `0c0f425` (13/13 static invariants + R-rules + acceptance §7 criteria + smoke output reproduction).
2. **Lyra**: acceptance on Layer A PASS.
3. **Mr. Zhang (via Aegis)**: Layer B live-run — attach to a real seat tmux session from the SessionsWorkspace tab, type into the xterm, verify keystroke round-trip + Ctrl-C + close-tab-preserves-session. Same observation pattern as A4-β.

---

*Delivery by Nimbus · 2026-05-11 · Commit `0c0f425` · 5 files, 209 insertions, 30 deletions · All Layer A self-checks pass (3 pre-existing doc-lint errors on `db/repositories.rs` flagged, not a B1 regression). Awaits Flux Layer A verify → Lyra acceptance → Mr. Zhang Layer B.*
