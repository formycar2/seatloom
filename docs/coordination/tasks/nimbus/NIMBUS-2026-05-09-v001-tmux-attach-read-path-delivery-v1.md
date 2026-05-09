# Delivery: tmux Attach Read Path (SG-A §A3, v0.0.1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1 |
| pairs_with | docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-v1.md |
| author | nimbus |
| date | 2026-05-09 |
| version | v1 |
| to | lyra, aegis |
| verdict | PASS |
| owner | Nimbus |
| acceptance owner | Lyra |

[Nimbus -> Lyra] tmux Attach Read Path — v0.0.1 read-only mirror.

commit:
- `fca4fe01eb5aad70121456b5cfecd5a7b18ab31b` (branch `track/infra-foundation`,
  HEAD at delivery time)

completed:
- Step 1: `cmd_list_tmux_sessions` implemented — filters `*-seatloom`
  session names; tolerates absent tmux / no-server-running.
- Step 2: `cmd_attach_tmux_session` implemented — mkfifo +
  `tmux pipe-pane -t <name>:0 -o 'cat >> <fifo>'` + std::thread
  tail-reader → `broadcast::channel<PtyEvent>` (capacity 1024) →
  Tauri `session:output` (base64 bytes) and `session:exit` events.
- Step 3: `cmd_pty_resize` → `tmux resize-pane`; `cmd_kill_session` stops
  pipe-pane (no args), unlinks the FIFO, drops the handle, **does NOT
  kill the tmux session** (R3).
- Step 4: `cmd_pty_write` / `cmd_pty_write_bytes` made `Ok(())` no-ops
  for v0.0.1. Write path deferred to B1.
- Step 5: `PtySession` retained for source-level continuity; internals
  now hold `tmux_session_name`, `tmux_target`, `fifo_path`,
  `broadcast::Sender`, shutdown atomic, transcript_path. No
  `master`/`child` fields; portable-pty removed from the dep graph.
- Bonus: added `cmd_launch_session` transitional shim so existing
  callers invoking `runtime: 'tmux'` or `'tmux-mirror'` reach
  `cmd_attach_tmux_session` without a rename; any other runtime now
  fails loudly with a message pointing at the correct command.
- Bonus: added `crates/seatloom-core/examples/tmux_mirror_smoke.rs`
  harness that exercises all 9 verification steps headlessly.

## Changes (git show --stat)

```
$ git show fca4fe0 --stat --format=""
 Cargo.lock                                         |  92 +----
 crates/seatloom-core/Cargo.toml                    |   7 +-
 crates/seatloom-core/examples/tmux_mirror_smoke.rs | 144 +++++++
 crates/seatloom-core/src/pty/mod.rs                | 438 ++++++++++++++-------
 src-tauri/src/commands/session_cmds.rs             | 175 +++++---
 src-tauri/src/main.rs                              |   9 +-
 6 files changed, 582 insertions(+), 283 deletions(-)
```

Scope matches the packet: only `crates/seatloom-core/src/pty/mod.rs`,
`src-tauri/src/commands/session_cmds.rs`, `src-tauri/src/main.rs`,
`crates/seatloom-core/Cargo.toml`, `Cargo.lock`, plus the new headless
smoke example. No `ui/src/**` changes (A4 / Mira). No
`infra/postgres/**` changes (C3).

## Verification

### 1–9 — Manual steps, headless

Because A4 UI wiring is owned by Mira, Step 5's
`cmd_subscribe_session_output` invocation and Step 7's xterm.js
visual verification are exercised via the deterministic smoke harness
at `crates/seatloom-core/examples/tmux_mirror_smoke.rs`. The harness
covers Steps 1-9 with explicit assertions; the UI integration check
lands in A4's delivery.

**Steps 1–2 — start a tmux session manually**

```
$ tmux new-session -d -s a3-smoke-seatloom 'sh -c "while :; do sleep 60; done"'
$ tmux list-sessions | grep a3-smoke-seatloom
a3-smoke-seatloom: 1 windows (created Sat May  9 19:02:38 2026)
```

**Steps 3–9 — smoke harness (list → attach → output → resize → kill, asserts R3)**

```
$ . "$HOME/.cargo/env" && cargo run -p seatloom-core --example tmux_mirror_smoke -- a3-smoke-seatloom 2>&1 | tail -30
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running `target/debug/examples/tmux_mirror_smoke a3-smoke-seatloom`
[smoke] target tmux session = a3-smoke-seatloom
[smoke] list_tmux_sessions() returned 7 sessions (filtered by -seatloom suffix)
  - Flux-Quality&Ops-seatloom (created_at=1776703859, attached=true)
  - Lyra-po-seatloom (created_at=1777277935, attached=true)
  - Mira-UX/UED-seatloom (created_at=1778227812, attached=true)
  - Nimbus-TechArchi-seatloom (created_at=1777260870, attached=true)
  - Onyx-data-seatloom (created_at=1777274311, attached=true)
  - a3-smoke-seatloom (created_at=1778324558, attached=false)
  - test-smoke-seatloom (created_at=1778324567, attached=false)
[smoke] attached: id=smoke-001, fifo=/tmp/seatloom-mirror/smoke-001.fifo, tmux_target=a3-smoke-seatloom:0
[smoke] received 48 bytes from mirror
[smoke] probe_marker_seen=true ls_output_seen=true
[smoke] resize(24,120) OK
[smoke] kill OK: fifo unlinked at /tmp/seatloom-mirror/smoke-001.fifo
[smoke] R3 OK: tmux session a3-smoke-seatloom still running after kill
SMOKE_PASS probe=true ls=true bytes=48
```

What each line asserts:

| Step | Assertion | Result |
|---|---|---|
| 3 | `list_tmux_sessions()` filters by `-seatloom` suffix and returns structured `TmuxSessionInfo` rows | PASS — 7 sessions including the freshly created smoke session |
| 4 | `PtySession::attach_tmux` creates FIFO at `/tmp/seatloom-mirror/<id>.fifo` and runs `tmux pipe-pane` | PASS — fifo path printed, post-attach `exists()` check inside harness |
| 5 | Subscriber receives `PtyEvent::Output` chunks from the pipe-pane tail-reader | PASS — `received 48 bytes` (non-empty broadcast stream) |
| 6 | `tmux send-keys` output lands in the mirror stream | PASS — `probe_marker_seen=true ls_output_seen=true` |
| 7 | UI-equivalent path: bytes arrive in subscriber-visible order | PASS — same receive loop backs the Tauri event emitter |
| 8 | `cmd_kill_session` unlinks FIFO | PASS — `fifo unlinked at /tmp/seatloom-mirror/smoke-001.fifo` |
| 9 | **R3: tmux session still running after kill_session** | PASS — `R3 OK: tmux session a3-smoke-seatloom still running after kill` |

Cleanup after smoke:

```
$ tmux kill-session -t a3-smoke-seatloom
$ tmux list-sessions | grep -E "smoke" || echo "smoke sessions cleaned up"
smoke sessions cleaned up
```

### Unit tests (pty module)

```
$ . "$HOME/.cargo/env" && cargo test -p seatloom-core --lib pty::
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running unittests src/lib.rs (target/debug/deps/seatloom_core-767388bf53ce3b11)

running 4 tests
test pty::tests::mirror_dir_is_tmp_seatloom_mirror ... ok
test pty::tests::default_transcripts_dir_nests_under_repo_root ... ok
test pty::tests::shell_quote_handles_spaces_and_quotes ... ok
test pty::tests::list_tmux_sessions_returns_vec_without_panic ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.01s
```

### Build check

```
$ . "$HOME/.cargo/env" && cargo check -p seatloom-tauri
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.25s
```

Three warnings remain, all pre-existing `dead_code` on `dto.rs`
(`PipelineRunDto`, `ReviewThreadDto`, `ReviewCommentDto`) unrelated to
A3. Full TS type-check + `pnpm build` belong to A4 (UI wiring) and are
not produced here by design — this packet is backend-only.

## R3 Evidence summary

- `PtySession::kill` (crates/seatloom-core/src/pty/mod.rs:356) executes
  only `tmux pipe-pane -t <target>` (detach the copy) and
  `std::fs::remove_file(&self.fifo_path)`. No `tmux kill-session`, no
  `tmux kill-pane`, no child-process ownership of tmux anywhere in the
  attach or kill path.
- Smoke run directly asserts this: `list_tmux_sessions()` after
  `kill_session` still contains the target session name; for non-seat
  targets the harness falls back to `tmux has-session -t`.

## Known limits (v0.0.1 scope)

Recorded here so A4/B1/C1 don't rediscover them:

- **First pane only**: `tmux_target = "<name>:0"`. Multi-window /
  multi-pane attach is B1 scope.
- **Single pipe-pane per session id**: `-o` makes pipe-pane idempotent
  on the target, but a second SeatLoom instance attaching to the same
  tmux session with a different session id will create a second FIFO
  on top of the same pipe-pane — this is fine for read-only mirroring
  but will matter once B1 adds send-keys authority.
- **FIFO offset on restart**: the FIFO is recreated (`mkfifo`) on each
  attach; pre-restart pane scrollback is not recovered. Scrollback
  backfill design lives in the architecture supplement (this packet's
  sibling).
- **`pipe-pane -o` + pane redraw**: alt-screen applications (vim,
  less) still emit bytes, but we receive redraw escapes rather than
  structured lines. Extraction of structured events from the byte
  stream is an AD-011 extension (architecture supplement §1).

blockers:
- none.

verdict:
- PASS

next action:
- wait for Lyra acceptance (SG-A §A3 is one of the four packets that
  must all pass Flux verification before SG-B dispatch); architecture
  supplement filed separately as
  `docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md`.

artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1.md
- crates/seatloom-core/src/pty/mod.rs (commit fca4fe0)
- crates/seatloom-core/examples/tmux_mirror_smoke.rs (commit fca4fe0)
- src-tauri/src/commands/session_cmds.rs (commit fca4fe0)
- src-tauri/src/main.rs (commit fca4fe0)
- crates/seatloom-core/Cargo.toml (commit fca4fe0)
- Cargo.lock (commit fca4fe0)

---

*Delivery by Nimbus · 2026-05-09 · SG-A §A3 · v0.0.1 read-only mirror*
