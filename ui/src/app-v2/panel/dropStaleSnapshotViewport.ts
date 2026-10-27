// Drop the stale attach-time frame from the VIEWPORT without touching the
// scrollback, then park the viewport at the live end.
//
// This lives in its own file, not beside the component, for two reasons: the
// component file must only export components (react-refresh), and — more to the
// point — both test halves drive THIS function rather than each re-implementing
// the sequence. `SessionTerminal.scrollback.test.tsx` calls it against the real
// xterm to show what survives; `SessionTerminal.geometry.test.tsx` asserts the
// component actually reaches for it. A copy of this sequence inside a test would
// prove nothing about the product.
//
// ── Why this exists at all (P0, 2026-10-27) ─────────────────────────────────
//
// The operator: "除了 ql 这个 seat 实时席位可以上下翻,其他所有 seat 都不能上下翻".
// The component used to answer a geometry change with `terminal.clear();
// terminal.reset();`, and BOTH destroy the scrollback:
//
//   · clear() -> BufferService.clear() sets `lines.length = 1`
//   · reset() -> BufferService.reset() does `this._normal = new Buffer(...)`,
//     i.e. a brand-new buffer object
//
// Measured on a 300-line history, that pair left 40 lines (== the row count) with
// the oldest line empty. So every seat whose fitted geometry differed from its
// pane seed had NO history to scroll. The seat that worked was the active one,
// and only because its fitted geometry happened to equal the seed — the
// difference between seats was never about the seats.
//
// ── Why a viewport clear is enough ──────────────────────────────────────────
//
// The stale frame had to go because it was painted at the pane's LAUNCH geometry:
// once the pane follows the viewport (SIGWINCH) the TUI repaints, and the old
// absolutely-positioned frame would sit underneath the repaint — that layering is
// what made pi's input box read as garbled. That garbling was always in the
// VIEWPORT, never in the scrollback. Measured: a 80->120 column resize kept a
// 300-line history at 301 lines with the oldest line intact, and so did an
// 80->78 one. `resize` itself does not clear the scrollback.
//
// ── The residual, stated because it is real ─────────────────────────────────
//
// `resize` reflows a line WIDER than the new column count onto extra rows, and
// those reflowed rows stay in the scrollback, so scrolling up can show reflowed
// debris from the old geometry. Measured with 45-character lines (what a real TUI
// emits) resized 80->78: zero reflow, `lines.length` unchanged, oldest line
// intact. A tmux pane is typically 200+ columns and the UI viewport is usually
// narrower, so the affected lines are the genuinely over-wide ones. That is a far
// better trade than no history at all, and it is the honest limit of a UI-side
// fix that cannot re-capture the history at the new geometry.

import type { Terminal } from '@xterm/xterm';

export const dropStaleSnapshotViewport = (terminal: Terminal) => {
  // ESC[2J clears the screen (the viewport) and ESC[H homes the cursor. Neither
  // touches the scrollback — that is the whole point.
  terminal.write('\x1b[2J\x1b[H');
  // Park the viewport at the live end so the operator lands on the repaint rather
  // than somewhere in the middle of the history. scrollToBottom() is the public
  // API for exactly this (d.ts:1195) and does not touch the scrollback.
  terminal.scrollToBottom();
};
