# Task: UM0 mirror fidelity — make pi's full TUI render cleanly in the UI

[Lyra -> Mira] Live bug Mr. Zhang hit: in the SeatLoom UI a worker seat's input line / content renders garbled. Aegis diagnosed the root cause as **feed + dimension sync**, not missing colour support. This packet is the UI half. It continues on your existing UM0 branch.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | MIRA-2026-10-08-um0-mirror-fidelity-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | mira |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (clean) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md` |
| predecessor | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-v1.md` (delivered `0390003`) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira` |
| delivery path | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-mirror-fidelity-delivery-v1.md` |
| tags | mira, ui, um0, xterm, tmux, fidelity, pi |

---

## 1. The diagnosis, grounded — read this before deciding anything

- Seats now run **pi**, a full-screen TUI emitting 24-bit truecolor + styling SGR (`\x1b[38;2;R;G;Bm`, `[48;2;…m`, `[1m`/`[7m`/`[0m`), box-drawing, and **absolute cursor positioning**, plus a distinct input box.
- `@xterm/xterm` **5.5** is in use and **does** support truecolor + full TUI. The renderer is capable. **Do not** propose swapping the terminal library, adding a colour polyfill, or stripping/rewriting escape sequences.
- The fault is that **the xterm viewport geometry and the tmux pane geometry do not match**, and the attach-time snapshot is fed into a buffer whose geometry may already be wrong.

### 1.1 What I measured in your code (this is the actual defect)

`ui/src/app-v2/panel/SessionTerminal.tsx`:

```ts
...(paneRows && paneCols ? { rows: paneRows, cols: paneCols } : {}),   // line ~78
if (!(paneRows && paneCols)) { fit.fit(); }                            // line ~88
const hasAuthoritativeSize = !!(paneRows && paneCols);
const resizeObserver = new ResizeObserver(() => {
  if (hasAuthoritativeSize) return;                                    // line ~112  <-- never resizes
  ...
  api.ptyResize(sessionId, rows, cols)                                  // line ~116
});
```

`LiveSeatsView.tsx:553-557` passes `paneRows` / `paneCols` straight from `attach` (`a.dto.paneRows` / `a.dto.paneCols`), so in the live mirror `hasAuthoritativeSize` is **always true**. Consequences:

1. xterm is constructed at the pane's geometry (measured on the real seats today: **219×54**, not the 220×50 the dispatch assumed) regardless of how wide the UI container actually is. If the container is narrower, xterm keeps a 219-column buffer and the view clips/scrolls — pi's absolutely-positioned input box lands off-screen or visually garbled.
2. The `ResizeObserver` early-returns, so **`cmd_pty_resize` is never called from the live mirror**. The pane is frozen at 219×54 forever while the UI window changes.

### 1.2 The real pane geometry today (measured)

```
tmux display-message -p -t Lyra-po-seatloom '#{window_width}x#{window_height} pane=#{pane_width}x#{pane_height}'
219x54 pane=219x54
```

Do not hard-code 220×50 or 219×54 anywhere. The viewport must win.

### 1.3 What I ruled out, so you don't chase it

- **`capture-pane -p -e` is correct** for a full-screen TUI. I probed it on a throwaway tmux session running a program on the alternate screen (`#{alternate_on}=1`): the plain invocation returns the alternate-screen content correctly; adding `-a` returns blank lines. No change needed there.
- **`pipe-pane` streaming is byte-clean.** The backend tail-reads the FIFO in raw 4096-byte chunks with no line buffering (`crates/seatloom-core/src/pty/mod.rs` reader thread). Do not add any transform in the UI either.
- **Colour support** is not the issue (§1).

### 1.4 The backend gap — already dispatched, do not work around it

`PtySession::resize` runs `tmux resize-pane -t <target> -x <cols> -y <rows>`. I measured on a throwaway session: **`resize-pane -x/-y` exits 0 and changes nothing** on these single-pane windows; `tmux resize-window -t <window> -x <cols> -y <rows>` works (verified 220×50 → 120×40 → 200×50 → 150×40 → 20×5). Nimbus is fixing that in `NIMBUS-2026-10-08-pty-resize-window-v1`. **Your packet must still call `api.ptyResize(...)` on every real geometry change** — that is the contract Nimbus's fix makes real. Build against the wrapper, not against today's broken behaviour.

## 2. Scope — `ui/src` only

### 2.1 Dimension sync (the key fix)

1. On mount, **always** fit the xterm viewport to its container (drop the `hasAuthoritativeSize` early-return for fitting). Use the initial pane geometry only as the *construction* seed so the first frames are not re-wrapped, then immediately `fit()`.
2. On every real geometry change — container `ResizeObserver`, window resize, font/layout change, tab switch, and the first fit after mount — call `api.ptyResize(sessionId, rows, cols)` with the **fitted** `rows`/`cols` from `term.rows` / `term.cols`, so the tmux pane matches the xterm viewport exactly.
3. Debounce/coalesce resize bursts (one in-flight resize at a time is fine; drop superseded ones) so a window drag does not spam tmux.
4. Never let the two sides disagree for more than one frame of intent: xterm must be the authority on geometry, tmux must follow it.

### 2.2 Feed correctly

1. Write the attach-time snapshot (`initialSnapshotB64`) as **raw bytes** via `term.write()` — no escape stripping, no re-encoding, no string round-trip. xterm parses truecolor natively.
2. Write live `pipe-pane` bytes the same way. Keep the existing `onSessionOutput` path.
3. **Avoid double-injecting the stale snapshot.** After a resize the pane content is re-laid-out by the app, so the snapshot captured at attach time is stale. Preferred fix inside your scope: on the first geometry change after mount, clear the xterm buffer (`term.clear()` + `term.reset()`) and let the app's post-SIGWINCH redraw repaint, rather than leaving the old absolutely-positioned frame underneath. State in your delivery exactly what you chose and why, and whether any stale-pixel case remains.
4. Do not re-fit to a geometry *smaller* than the pane and then resize tmux down to it without clearing — that is how the double-wrap/garbled artefacts appear.

### 2.3 Prove the input line renders clean

Mr. Zhang flagged the **input line** specifically. Your evidence must cover it: the input box must be inside the viewport, correctly positioned, and typing must round-trip (you already have `cmd_pty_write_bytes` → `term.onData`).

## 3. Hard constraints

- **Frontend only.** Zero files under `crates/`, `src-tauri/`, `infra/`; zero `.sql`. Nimbus owns the Rust side of this same bug right now — do not touch it.
- Do not touch `ui/src/lib/types-dto.ts` or `ui/src/lib/dto-to-v1.ts`.
- Do not disturb any live seat: do not kill or restart tmux sessions; if your verification toggles `pane_pipe`, record before/after and restore all five seats to `0`.
- Work in your worktree only.

## 4. Verification

In your worktree, after first `git merge track/infra-foundation` (picks up the merged pipe-pane fix so your branch and Nimbus's agree):

```
pnpm --dir ui typecheck
pnpm --dir ui exec eslint .
pnpm --dir ui exec prettier --check .
pnpm --dir ui test
pnpm --dir ui build
git show --stat HEAD
```

Plus the part no command proves, with buffer-dump evidence (screenshots are denied on this host — use the xterm-buffer / DOM-dump approach Flux used, which is now the sanctioned path):

1. The mirror of a real pi seat shows its TUI — input box, truecolor, box-drawing — **cleanly and matching the real tmux pane**.
2. Typing shows correctly in the mirrored pane and round-trips.
3. Resizing the UI resizes the pane and the render stays clean.
4. `#{pane_width}×#{pane_height}` of the mirrored seat **equals** the fitted xterm `cols×rows` — state both numbers verbatim.

## 5. Done-definition

1. xterm viewport is fitted to its container; the pane follows the viewport via `cmd_pty_resize` on every real geometry change.
2. Snapshot + live bytes are fed raw, with no stale-snapshot double-inject.
3. The input line renders clean, in-viewport, and typing round-trips.
4. Zero backend/Rust/schema/DTO changes.
5. All five §4 commands green with verbatim output.
6. Buffer-dump evidence for §4's four behavioural items, including the pane-vs-viewport geometry match.
7. Delivery artifact at the path in the header, with exact SHAs.

## 6. Blockers

- None. If the fix turns out to require a backend change, **STOP and report** rather than editing Rust — Nimbus has that lane.
- Note for your own sequencing: Nimbus's `resize-window` fix may not be on `track` when you start. Your packet is written so it is correct either way; verify against the merged branch once Nimbus lands, and say so in your delivery.

---

*Dispatched by Lyra · 2026-10-08 · UM0 mirror fidelity · frontend-only, own worktree, parallel-safe with Nimbus's Rust fix · §1.1 is the measured defect, §1.4 is the backend gap already dispatched*
