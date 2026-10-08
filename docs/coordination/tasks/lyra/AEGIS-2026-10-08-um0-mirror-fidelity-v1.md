# Dispatch: UM0 mirror fidelity — render pi's full TUI cleanly in the UI

[Aegis -> Lyra] Live bug Mr. Zhang hit: in the SeatLoom UI, a worker seat's **input line / content
renders garbled**. Root cause diagnosed — route to Mira (primary, UI) + Nimbus (backend only if a
gap). Please dispatch Mira NOW, then Flux-verify, then merge (don't drop the hops).

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-10-08-um0-mirror-fidelity |
| status | dispatched |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (clean) |
| tags | p0, dispatch, ui, mirror, xterm, pi |

## Diagnosis (grounded)
- Seats now run **pi**, a full-screen TUI emitting 24-bit truecolor + styling SGR
  (`\x1b[38;2;R;G;Bm`, `[48;2;…m`, `[1m`/`[7m`/`[0m`), box-drawing, and **absolute cursor
  positioning**, plus a distinct input box. Seat tmux panes are launched at **220×50**.
- UI renders the mirror with **@xterm/xterm 5.5** — which DOES support truecolor + full TUI. So the
  renderer is capable; the fault is in **feed + dimension sync**, not missing color support.
- Most likely: the xterm.js viewport size ≠ the 220×50 tmux pane, so pi's absolute-positioned TUI
  (esp. the input box) lands misaligned/garbled; and/or the initial `capture-pane` snapshot + the
  `pipe-pane` byte stream are not fed into xterm.js cleanly. The UM0 mirror was tuned against
  Claude Code's simpler output; pi's richer TUI exposes the gap.

## Mira (PI/step-5-preview) — UI fix (primary)
1. **Dimension sync (the key fix):** on mount and on UI resize, measure the xterm.js viewport
   (cols/rows via the fit addon) and call `cmd_pty_resize(sessionId, rows, cols)` so the tmux pane
   matches the xterm viewport EXACTLY. Then pi's absolute positioning lands correctly. Do not leave
   the pane at a fixed 220×50 while the viewport differs.
2. **Feed correctly:** write the initial `capture-pane` snapshot then the live `pipe-pane` bytes
   into xterm.js `.write()` as raw UTF-8 bytes — do not strip/transform escape sequences; xterm.js
   parses truecolor natively. Avoid double-injecting the stale snapshot after pi redraws on resize.
3. Verify the **input line** specifically renders clean (it's what Mr. Zhang flagged).
   Frontend-first (ui/src); zero-regression gates; your worktree.

## Nimbus (PI/opus) — backend, only if a real gap
Confirm `cmd_pty_resize` actually resizes the tmux pane (tmux `resize-window`/`resize-pane`) and
that `pipe-pane` streaming is byte-clean (no line-buffering mangle). Minimal Rust only if needed;
your worktree. Don't expand scope.

## Acceptance (Flux)
Open the running UI → a pi seat's mirror shows its TUI (input box, truecolor, boxes) **cleanly,
matching the real tmux pane**; typing shows correctly; resizing the UI resizes the pane and stays
clean. Prove via buffer-dump evidence (screenshots denied). Zero-regression.

## After
Lyra accepts + merges to track; Aegis reconciles + restarts the shared UI. Then "see the 5 live PI
seats cleanly in the UI" is truly met, and B1 (supervisor-IM via Aegis) follows.
