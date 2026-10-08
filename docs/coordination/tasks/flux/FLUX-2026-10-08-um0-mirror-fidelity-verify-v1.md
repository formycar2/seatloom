# Task: Commit-pinned verification — UM0 mirror fidelity (Mira UI + Nimbus resize, combined)

[Lyra -> Flux] Both mirror-fidelity fixes are delivered. Verify them **together**, against a real UI run. This is the gate that decides whether the garbled pi TUI is actually fixed — neither packet alone can prove it, and Mira's own evidence necessarily shimmed the Rust layer, so the real binary must be exercised here.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-08-um0-mirror-fidelity-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| to | flux |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (clean) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md`; `/tmp/lyra-nudge-mirror-merge.md` |
| delivery under test (UI) | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-mirror-fidelity-delivery-v1.md` |
| delivery under test (backend) | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-pty-resize-window-delivery-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `59c162c12853d019c683ed70284ff8d7d98268fe` |
| mira branch / commit | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` @ `da4213b8e255280de277227308e735bf89d11852` (tip incl. writeback: `a8ce2ed05af3f4bc4e7f6e1d7fb187b52f40d3b1`) |
| nimbus branch / commit | `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` @ `45d6c1518c7cfd257d5c04d8c320c0bef77daffa` (tip incl. writeback: `d8c02ed`) |
| run location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux` |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-10-08-um0-mirror-fidelity-verify-delivery-v1.md` |
| tags | flux, verification, commit-pinned, um0, mirror-fidelity, xterm, tmux, pi |

---

## 0. What this gate must answer

Mr. Zhang's live bug: a worker seat's **input line / content renders garbled** in the SeatLoom UI. Two delivered fixes, one defect:

- **UI (Mira, `da4213b`):** xterm viewport is the geometry authority; the pane follows via `cmd_pty_resize` on every real geometry change; snapshot + live bytes fed raw; stale snapshot dropped once on the first effective geometry sync.
- **Backend (Nimbus, `45d6c15`):** `PtySession::resize` now uses `tmux resize-window -x/-y` (the old `resize-pane -x/-y` was a **silent no-op** — I reproduced that before dispatching) and verifies the resulting `#{pane_width} #{pane_height}`, returning typed `PtyError::ResizeNotApplied` on mismatch.

**The question is whether, with both, a real pi seat's TUI renders cleanly in the UI and the pane tracks the viewport.** A green typecheck proves nothing about that.

## 1. How to build the verification target (combined tree)

Neither branch contains the other's fix, so there is no single commit that represents "both delivered". Build the combination yourself, deterministically, and verify **that exact tree**:

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux
git status --short                       # must be empty
git checkout --detach 59c162c12853d019c683ed70284ff8d7d98268fe
git merge --no-edit da4213b8e255280de277227308e735bf89d11852
git merge --no-edit 45d6c1518c7cfd257d5c04d8c320c0bef77daffa
git rev-parse HEAD                       # <-- THE verified commit; report it verbatim
git status --short                       # must be empty
```

Rules:

- Resolve any conflict yourself only if it is trivial (docs writeback); if a conflict touches code, **STOP and return HOLD** with the exact conflicting files.
- Report the resulting merge SHA in the delivery header. Every acceptance claim must cite it.
- If the resulting tree is not exactly `59c162c` + Mira's `ui/` change + Nimbus's Rust change, say so explicitly.

### 1.1 Scope proof (both deltas)

```bash
git diff --name-only 59c162c12853d019c683ed70284ff8d7d98268fe..<merge-sha>
git diff --name-only 59c162c12853d019c683ed70284ff8d7d98268fe..<merge-sha> | grep -vE '^(ui/|crates/|src-tauri/|docs/)' || true
```

Expected shape: `ui/src/app-v2/panel/SessionTerminal.tsx`; `crates/seatloom-core/src/pty/mod.rs`; `crates/seatloom-core/examples/nimbus_resize_proof.rs`; `src-tauri/src/commands/session_cmds.rs`; plus docs. **Zero `.sql`.** `types-dto.ts` and `dto-to-v1.ts` must be untouched.

## 2. Gates (verbatim)

Rust:

```
cargo check -p seatloom-core
cargo test -p seatloom-core --lib
cargo clippy -p seatloom-core --all-targets -- -D warnings
cargo fmt --all --check
```

UI (report counts and ordering):

```
pnpm --dir ui typecheck
pnpm --dir ui exec eslint .
pnpm --dir ui exec prettier --check .
pnpm --dir ui test
pnpm --dir ui build
bash scripts/lint-changed.sh
```

Zero-regression standard, unchanged from the last two gates: eslint 151 problems / 143 errors / 8 warnings and prettier 92 files are the accepted pre-existing baselines (Flux measured 92 at `0390003`→Mira's base; Mira reports 92 at `da4213b`). Any **new** finding in a changed file is a regression — attribute it, do not just total it. `lint-changed.sh` is expected to stay red on `App.tsx`'s 15 pre-existing `no-explicit-any`; state precisely whether anything else is red and whether it is new.

Build the Rust side in an **isolated** `CARGO_TARGET_DIR` under `.local/` (CoW copy if you want speed) so the shared checkout's `target/` is never written to.

## 3. The real-binary behavioural gate — this is the point of the packet

Screenshots are denied on this host; use the buffer-dump / DOM-dump path you established, now against the **real** Tauri binary built from the combined tree (not a shimmed `invoke()` layer — Mira's shim is exactly the gap you are closing).

The shared checkout's app is running again (pid 66269, vite pid 66235 on 1420) on **pre-fix** `track` code. Do not stop it and do not branch-switch the shared checkout. Launch your own additive instance on a free port, as you did before, and prove it is serving the combined tree (`git rev-parse HEAD` at build time + the WebKit process holding an ESTABLISHED connection to your port).

### 3.1 Required observations

1. **Attach to a real pi seat** and read the xterm buffer back. The mirror must show that seat's actual TUI: the **input line**, truecolor, and box-drawing, **cleanly and matching the real tmux pane**.
2. **Pane geometry == fitted xterm viewport geometry**, verbatim, at mount and after each resize: `#{pane_width} #{pane_height}` from tmux vs `term.cols`/`term.rows` from xterm. This is the core claim — the numbers must be equal, not merely close.
3. **Typing round-trips** and renders correctly in both the xterm buffer and `tmux capture-pane -p` on the real pane.
4. **Resizing the UI resizes the pane and stays clean**: change the window/container size, then re-read both geometries and re-dump the buffer. The input box must still be in-viewport and correctly positioned; no stale frame, no doubled borders, no shifted rows.
5. Confirm the **`ResizeNotApplied`** path is real code that can fire — you do not need to trigger it live; Nimbus's unit tests cover the branch. Just confirm the error variant exists in the combined tree and is returned by `resize` on geometry mismatch (cite the code).

### 3.2 Hazards — mandatory

- Record `pane_pipe` for **all five** seats before you start; restore every one to `0` at the end and re-read them individually.
- Kill no seat session. `tmux ls` before and after must show the same sessions.
- Do not write to the shared checkout's `target/`, do not switch its branch, do not stop its app.
- If you attach to a real seat, prefer a pane whose `pane_pipe` is already `1` (held by the running shared app) or accept that you will toggle it — record before/after either way and restore.

## 4. Verdict

PASS / HOLD / FAIL. Partial verification is HOLD with the reason, not PASS with a caveat. Specifically:

- If the combined tree renders clean and the geometry matches at mount **and** after resize → PASS.
- If the geometry matches but any visual defect remains (stale frame, shifted input box, wrong colours) → HOLD naming the seat, the geometry pair, and the buffer dump.
- If the pane does not follow the viewport with the **real** binary → HOLD naming the exact `cmd_pty_resize` call, the fitted geometry, and the `#{pane_width} #{pane_height}` read-back.

## 5. Done-definition

1. Exact combined-tree commit verified from a clean workspace, SHA reported.
2. Scope proof clean: no `.sql`, no DTO-boundary files, no unrelated files.
3. All §2 gates green or explained as pre-existing, with attribution.
4. §3's five behavioural observations covered with verbatim evidence.
5. Hazards handled: `pane_pipe` before/after for all five seats, no session killed, shared checkout untouched.
6. Delivery artifact at the header path with verdict, SHAs, and evidence paths.

---

*Dispatched by Lyra · 2026-10-08 · verify BOTH fixes together against the real binary · Mira's shimmed-Rust evidence does not close the real-binary link, and that link is the whole point of this gate · buffer-dump evidence, screenshots denied*
