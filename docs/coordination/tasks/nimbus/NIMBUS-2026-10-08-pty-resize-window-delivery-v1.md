# Delivery: `cmd_pty_resize` silent no-op — resize the window, verify geometry

[Nimbus -> Lyra / Flux] Delivery for `NIMBUS-2026-10-08-pty-resize-window-v1`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-08-pty-resize-window-delivery-v1 |
| author | nimbus |
| date | 2026-10-08 |
| version | v1 |
| responds to | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1.md` |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md` |
| predecessor | `NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` (accepted, merged `1a34007`) |
| milestone | UM0 — one screen for the whole team (clean) |
| base branch | `track/infra-foundation` |
| base commit | `59c162c12853d019c683ed70284ff8d7d98268fe` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` |
| delivery commit | `45d6c1518c7cfd257d5c04d8c320c0bef77daffa` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` |
| status | delivered — awaiting Flux commit-pinned verify |

---

## 1. Root cause (independently reproduced)

`crates/seatloom-core/src/pty/mod.rs` `PtySession::resize` issued
`tmux resize-pane -t <session>:0 -x <cols> -y <rows>`. On a **single-pane window** —
which every seat session is — `resize-pane -x/-y` exits `0` and silently changes
nothing. `cmd_pty_resize` then reported success for a no-op, so the tmux pane stayed
frozen at launch geometry while the xterm viewport moved: the UM0 garbled-mirror
misalignment. Same silent-success defect class as the `pipe-pane -o` bug.

Reproduced by me on a throwaway session (`tmux new-session -d -s … -x 220 -y 50 'sleep 300'`,
tmux **3.6a**), reading `#{window_width}x#{window_height} pane=#{pane_width}x#{pane_height}`:

```
created              : 220x50 pane=220x50
resize-pane 120x40   : exit 0 220x50 pane=220x50   <-- NO EFFECT
resize-window 120x40 : exit 0 120x40 pane=120x40   <-- WORKS
resize-window 200x50 : exit 0 200x50 pane=200x50   <-- WORKS
resize-window 80x24  : exit 0 80x24  pane=80x24    <-- WORKS (not clamped small)
```

Matches Lyra's pre-dispatch measurement exactly.

## 2. Change

1. **`PtySession::resize`** now issues `tmux resize-window -t <session>:<window> -x <cols> -y <rows>`
   against `self.tmux_target` (`<session>:0`, i.e. the window), not `resize-pane`.
   No `resize-pane` fallback retained — it does not work on single-pane windows on
   the measured platform, so keeping it would only risk a silent regression.
2. **Verify, don't trust the exit code.** After the resize, `resize` reads back
   `#{pane_width} #{pane_height}` via `display-message` and returns a new typed
   `PtyError::ResizeNotApplied { target, want_cols, want_rows, got_cols, got_rows }`
   when the pane geometry does not match the request. This mirrors the
   `verify_pipe_open` pattern. The geometry decision is factored into a pure helper
   `verify_resize_applied(target, want_cols, want_rows, status_ok, geo_raw)` so it is
   unit-testable without a live tmux.
3. **Doc comments corrected** at the three sites the packet named, with the measured
   semantics, so nobody "restores" `resize-pane`:
   - `pty/mod.rs` `LaunchOptions` doc ("rows/cols feed `tmux resize-window` …")
   - `pty/mod.rs` `attach_tmux` doc ("maps to `tmux resize-window` (not `resize-pane`, which is a silent no-op …)")
   - `src-tauri/src/commands/session_cmds.rs` `cmd_pty_resize` doc.
4. **Untouched:** attach path, `pipe-pane` behaviour, FIFO reader, write path,
   `capture-pane`, window-size options, and `ui/`. No new Tauri command;
   `cmd_pty_resize(session_id, rows, cols)` keeps its exact name and signature.

### Changed files (exact)

```
 crates/seatloom-core/src/pty/mod.rs            | +209 / -10  (resize rewrite, ResizeNotApplied, verify_resize_applied + 5 unit tests, 2 doc comments)
 src-tauri/src/commands/session_cmds.rs         |   +8 / -1   (cmd_pty_resize doc comment)
 crates/seatloom-core/examples/nimbus_resize_proof.rs | new   (throwaway behavioural proof)
```

Zero `.tsx`, zero `.sql`.

## 3. Verification — Layer A (verbatim)

Commands (clippy run as `-D warnings`; the packet's `--D warnings` is a typo in the
flag, corrected to the valid form):

```
cargo check -p seatloom-core
  -> Finished `dev` profile ... (ok)
cargo test -p seatloom-core --lib
  -> test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
     (incl. verify_resize_applied_accepts_exact_match,
            verify_resize_applied_rejects_a_no_op_resize,
            verify_resize_applied_rejects_partial_match,
            verify_resize_applied_rejects_unparsable_geometry,
            verify_resize_applied_rejects_a_failed_probe)
cargo clippy -p seatloom-core --all-targets -- -D warnings
  -> Finished `dev` profile ... (no warnings)
cargo fmt --all --check
  -> clean
```

Also confirmed `cargo check --manifest-path src-tauri/Cargo.toml` builds
(`seatloom-tauri` + `seatloom-core`), so the doc-comment change compiles in the app crate.

## 4. Verification — behavioural proof (throwaway session, never a live seat)

Driven through the **real** `PtySession::resize` code path via
`cargo run -p seatloom-core --example nimbus_resize_proof`. The example creates and
kills its own `nimbus-resize-proof-<pid>` session and reads back
`#{pane_width} #{pane_height}` verbatim at each step:

```
created                 : 220 50
resize(40,120)          : 120 40   (want: 120 40)
resize(24,80)           : 80 24    (want: 80 24)
resize(60,240) [grow]   : 240 60  (want: 240 60)
resize on dead session   : typed error fired -> tmux error: tmux resize-window nimbus-resize-proof-<pid>:0: can't find session: nimbus-resize-proof-<pid>
session present after kill: false
killed                   : throwaway session removed
```

- Steps 2–4: shrink, shrink, and **grow past launch size** all read back **exactly**
  what was requested (step 4 is the case that fails with `resize-pane` today).
- Step 5 (typed error): a resize against a killed session fails before geometry
  read-back and surfaces as a typed `PtyError` — success is never reported for a
  resize that did nothing. The `ResizeNotApplied` branch itself (resize-window exits
  0 but geometry mismatches — i.e. the exact `resize-pane` no-op shape) is covered
  deterministically by `pty::tests::verify_resize_applied_rejects_a_no_op_resize`,
  which feeds the live helper `220 50` after requesting `120 40`. I did not reproduce
  a natural exit-0-wrong-geometry from the binary because on this host `resize-window`
  is not clamped (verified), so the unit test is the authoritative evidence for that
  branch.
- Step 6: `tmux has-session` returns false — throwaway session removed, tmux left as
  found. `tmux ls` after the run shows all five live seats untouched and no proof
  residue.

**Exit codes alone were not used as evidence** — §1 is precisely a case where exit 0
meant nothing happened; every claim above is backed by a pane-geometry read-back.

## 5. Done-definition check

1. `resize` actually resizes the mirrored pane, via the window — proved (§4). ✅
2. Post-resize geometry verified; typed error on mismatch — `ResizeNotApplied` + unit tests. ✅
3. Doc comments corrected with measured semantics (3 sites). ✅
4. Layer A green, verbatim (§3). ✅
5. Zero `.tsx`, zero `.sql`, no attach/pipe/write path changes. ✅
6. Flux commit-pinned verify — pending, target commit `45d6c15`. ⏳
7. Delivery artifact at header path with base/delivery SHAs — this file. ✅

## 6. Notes / tradeoffs for Flux + Lyra

- **`resize-pane` fallback dropped intentionally.** It is a no-op on single-pane
  windows on the measured platform; a fallback would reintroduce the silent-success
  risk. If a real multi-pane configuration ever needs per-pane sizing, that is a
  separate, explicitly-scoped packet.
- **Clamp behaviour:** on this host `resize-window` honours both grow and shrink
  (down to 20x5) without clamping, matching the packet. If a future host clamps,
  `ResizeNotApplied` will fire rather than silently mis-size — which is the intended
  safe failure.
- **Proof example** is a `cargo run` target under `examples/`, not wired into the app
  and not a test, so it never runs in CI or against a live seat. It exists as a
  reproducible runbook for Flux.

---

*Delivered by Nimbus · 2026-10-08 · base `59c162c` → delivery `45d6c15` on
`packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` · proof on a throwaway tmux
session (tmux 3.6a), live seats untouched.*
