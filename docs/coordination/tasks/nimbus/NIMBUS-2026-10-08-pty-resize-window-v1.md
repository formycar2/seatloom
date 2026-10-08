# Task: `cmd_pty_resize` is a silent no-op — `resize-pane` does not size a single-pane window

[Lyra -> Nimbus] Aegis dispatch `AEGIS-2026-10-08-um0-mirror-fidelity-v1`. Mira is fixing the UI half of the garbled-mirror bug (xterm viewport ↔ tmux pane geometry sync). **I reproduced the backend half before dispatching**: the resize command the UI is contractually supposed to call does nothing. That is the gap that makes your fix mandatory.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-10-08-pty-resize-window-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (clean) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md` |
| predecessor | `NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` (accepted, merged at `1a34007`) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-pty-resize-window-delivery-v1.md` |
| tags | nimbus, pty, tmux, resize, um0, fidelity, pi |

---

## 1. The bug, reproduced by me on a throwaway tmux session

`crates/seatloom-core/src/pty/mod.rs:500` — `PtySession::resize`:

```rust
.args(["resize-pane", "-t", &self.tmux_target, "-x", &cols.to_string(), "-y", &rows.to_string()])
```

Measured on `tmux new-session -d -s lyra-resize-probe -x 220 -y 50 'sleep 300'` (tmux **3.6a**), reading `#{window_width}x#{window_height} pane=#{pane_width}x#{pane_height}` after each call:

```
created                     : 220x50 pane=220x50
resize-pane  -x 120 -y 40   : exit 0   220x50 pane=220x50   <-- NO EFFECT
resize-pane  -x 219 -y 54   : exit 0   220x50 pane=220x50   <-- NO EFFECT
resize-pane  -t :0.0 -x 150 : exit 0   200x50 pane=200x50   <-- NO EFFECT
resize-window -x 120 -y 40  : exit 0   120x40 pane=120x40   <-- WORKS
resize-window -x 200 -y 50  : exit 0   200x50 pane=200x50   <-- WORKS
resize-window -t :0 -x 150  : exit 0   150x40 pane=150x40   <-- WORKS
resize-window -x 20  -y 5   : exit 0   20x5   pane=20x5     <-- not clamped at small sizes
```

So on a single-pane window — which is what every seat session is — **`resize-pane -x/-y` exits 0 and silently changes nothing.** `resize-window -x/-y` is the command that actually sizes the pane, and it is not clamped at small sizes.

Consequence for the product: Mira's UI fix will call `cmd_pty_resize` on every geometry change (that is the contract she is building to). With `resize-pane` in place, every one of those calls returns success and does nothing, so the tmux pane stays frozen at its launch geometry while the xterm viewport changes — which is exactly the garbled/absolute-positioning misalignment Mr. Zhang hit with pi's TUI. `cmd_pty_resize` currently reports success for a no-op, which is the same class of silent-success defect as the `pipe-pane -o` bug you just fixed.

## 2. Scope

1. **`pty/mod.rs` `PtySession::resize`** — resize the **window**, not the pane. Use `tmux resize-window -t <session>:<window> -x <cols> -y <rows>`; keep a `resize-pane` fallback only if you can show it works in some real configuration (on my measurements it does not, for single-pane windows). Target the **window** index of `self.tmux_target` (`<session>:0`), not the pane, so multi-pane windows do not break the contract silently.
2. **Verify the result, don't trust the exit code.** `resize-pane` exits 0 while doing nothing, so an exit-code check is not evidence. After the resize, read `#{pane_width} #{pane_height}` for the target and return a typed `PtyError` when the pane geometry does not match what was requested. This mirrors the `verify_pipe_open` pattern you established in the `pipe-pane` fix — same principle: a silent dead mirror is the user harm.
3. **Correct the doc comments.** `pty/mod.rs:83` ("`tmux resize-pane` when `PtySession::resize` is invoked"), `:225-226`, and the `cmd_pty_resize` doc comment in `src-tauri/src/commands/session_cmds.rs:327` all describe the old behaviour. Update them with the measured semantics so nobody "restores" `resize-pane` later — same treatment you gave the `-o` comment.
4. **Do not change** the attach path, `pipe-pane` behaviour, the FIFO reader, the write path, or anything in `ui/`. Mira owns `ui/src` and is live in it right now.
5. **No `.sql`.** No new Tauri command. `cmd_pty_resize` keeps its exact name and signature `(session_id, rows, cols)`.

## 3. Out of scope

- **No UI.** Zero `.tsx`. Mira's packet covers the xterm viewport side.
- **No snapshot-on-resize change.** I checked whether the attach-time `capture-pane -p -e` is wrong for full-screen TUIs: it is not. I probed a throwaway session with a program on the alternate screen (`#{alternate_on}=1`) — the plain invocation returns the alternate-screen content correctly, and adding `-a` returns blank lines. Do not "fix" capture-pane.
- **No multi-pane layout redesign**, no window-size option changes, no `set-option -t … window-size manual` games (I tested it: it does not make `resize-pane` work either).
- **Do not touch any live seat session.** Five seats are live and working.

## 4. Verification

```
cargo check -p seatloom-core && cargo test -p seatloom-core --lib
cargo clippy -p seatloom-core -- --D warnings && cargo fmt --all --check
```

Plus the behavioural proof, **on a throwaway tmux session you create and kill — never on a seat**:

1. attach to the throwaway session;
2. `resize(40, 120)` → read `#{pane_width} #{pane_height}` back and paste it verbatim — must be `120 40`;
3. `resize(24, 80)` → must read back `80 24`;
4. resize to something larger than the launch size → must read back correctly (this is the case that fails today);
5. the typed error fires when the geometry does not match (force it however you like — a killed session is fine);
6. kill the session cleanly; tmux left exactly as found.

Paste the `#{pane_width} #{pane_height}` readings at each step verbatim. **An exit code alone is not evidence** — §1 is precisely a case where exit 0 meant nothing happened.

## 5. Done-definition

1. `resize` actually resizes the mirrored pane, via the window, with readings proving it.
2. Post-resize geometry is verified and a typed error is returned on mismatch.
3. Doc comments corrected with the measured semantics.
4. Layer A green, verbatim.
5. Zero `.tsx`, zero `.sql`, no attach/pipe/write path changes.
6. Flux commit-pinned verify PASSes against your delivery commit.
7. Delivery artifact at the header path with base/delivery SHAs.

---

*Dispatched by Lyra · 2026-10-08 · measured before dispatch on a throwaway tmux session (tmux 3.6a) · `resize-pane -x/-y` is a silent no-op on single-pane windows, `resize-window -x/-y` works · parallel-safe with Mira's UI packet*
