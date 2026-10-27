// SessionTerminal — embedded xterm.js terminal bound to a live SeatLoom PTY.
//
// Subscribes to Tauri `session:output` events for the given sessionId, writes
// raw bytes to xterm, forwards keystrokes back via cmd_pty_write_bytes, and
// keeps the tmux pane geometry locked to the xterm viewport via
// cmd_pty_resize.
//
// Geometry contract (UM0 mirror fidelity): xterm is the authority, tmux
// follows. The pane dimensions reported at attach time seed the xterm
// *construction* size so the first frames are not re-wrapped; the viewport is
// then fitted to its container immediately and the fitted rows/cols are pushed
// to the backend on every real geometry change (mount, container resize,
// window resize, font settle, attachment tab switch). The previous
// "authoritative size" path skipped fit() and skipped cmd_pty_resize entirely
// whenever attach-time dims were present — which the live mirror always sends
// — so the pane froze at its launch geometry (measured 219×54) while the UI
// viewport moved on. That mismatch is what garbled pi's full-screen TUI.
// Whether tmux honours cmd_pty_resize today is Nimbus's resize-window fix;
// this component upholds the contract either way.
//
// Feed contract: the attach-time snapshot and live pipe-pane bytes are written
// raw (Uint8Array via term.write) — no escape stripping, no re-encoding, no
// string round-trip. xterm 5.5 parses 24-bit truecolor, box drawing and
// absolute cursor positioning natively.
//
// Visual tuning: xterm canvas renderer with sensible dark theme aligned to
// SeatLoom brand tokens.

import React, { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';

import { api, decodeSessionOutput, onSessionExit, onSessionOutput } from '../../lib/api';

interface Props {
  sessionId: string;
  onExit?: (code: number | null, signal: string | null) => void;
  /// Authoritative tmux pane dimensions queried at attach time. Used only as
  /// the xterm *construction* seed so the first frames are not re-wrapped; the
  /// viewport is fitted immediately after and the pane follows the fit.
  paneRows?: number;
  paneCols?: number;
  /// Base64-encoded snapshot of pane content at attach time. Written to xterm
  /// once at mount before subscribing so idle panes render their current
  /// content instead of a black screen.
  initialSnapshotB64?: string;
}

const THEME = {
  background: '#0d1117',
  foreground: '#e6edf3',
  cursor: '#58a6ff',
  cursorAccent: '#0d1117',
  selectionBackground: 'rgba(88, 166, 255, 0.3)',
  black: '#484f58',
  red: '#ff7b72',
  green: '#3fb950',
  yellow: '#d29922',
  blue: '#58a6ff',
  magenta: '#bc8cff',
  cyan: '#39c5cf',
  white: '#e6edf3',
  brightBlack: '#6e7681',
  brightRed: '#ffa198',
  brightGreen: '#56d364',
  brightYellow: '#e3b341',
  brightBlue: '#79c0ff',
  brightMagenta: '#d2a8ff',
  brightCyan: '#56d4dd',
  brightWhite: '#f0f6fc',
};

export const SessionTerminal: React.FC<Props> = ({
  sessionId,
  onExit,
  paneRows,
  paneCols,
  initialSnapshotB64,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  /// Geometry last pushed to the backend, so we only call cmd_pty_resize on
  /// real changes.
  const lastPushedRef = useRef<{ rows: number; cols: number } | null>(null);
  /// Coalescing state: at most one in-flight resize; a burst keeps only the
  /// latest target so a window drag cannot spam tmux.
  const inFlightResizeRef = useRef<Promise<unknown> | null>(null);
  const pendingResizeRef = useRef<{ rows: number; cols: number } | null>(null);
  /// The stale attach-time snapshot is dropped exactly once, on the first
  /// effective geometry sync after mount.
  const clearedStaleSnapshotRef = useRef(false);

  useEffect(() => {
    if (!containerRef.current) return;

    // Attach-time pane geometry is the construction seed only — see the
    // geometry contract in the file header.
    const seedDims = paneRows && paneCols ? { rows: paneRows, cols: paneCols } : null;

    const term = new Terminal({
      fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
      fontSize: 13,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: 'bar',
      scrollback: 10000,
      allowProposedApi: true,
      theme: THEME,
      ...(seedDims ?? {}),
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(containerRef.current);

    termRef.current = term;
    fitRef.current = fit;

    // ── Geometry sync: xterm is the authority, tmux follows ────────────────
    const pushResize = (rows: number, cols: number) => {
      lastPushedRef.current = { rows, cols };
      if (inFlightResizeRef.current) {
        // Supersede the queued target; only the latest geometry matters.
        pendingResizeRef.current = { rows, cols };
        return;
      }
      inFlightResizeRef.current = api
        .ptyResize(sessionId, rows, cols)
        .catch((err) => {
          console.error('[SessionTerminal] ptyResize failed:', err);
        })
        .finally(() => {
          inFlightResizeRef.current = null;
          const next = pendingResizeRef.current;
          if (next) {
            pendingResizeRef.current = null;
            pushResize(next.rows, next.cols);
          }
        });
    };

    const containerHasArea = () => {
      const el = containerRef.current;
      return !!el && el.clientWidth > 0 && el.clientHeight > 0;
    };

    // Fit the viewport to its container and push the fitted geometry to the
    // backend when it differs from what was last pushed. Runs on mount and on
    // every real geometry change. The first effective call also drops the
    // stale attach-time snapshot (see below). Gated on `geometryArmed` so no
    // resize is pushed before the output listeners exist — see below.
    let geometryArmed = false;
    /// Whether the last sync actually fitted the viewport.
    ///
    /// `FitAddon.fit()` is a SILENT NO-OP when xterm has no valid character
    /// metrics yet — `proposeDimensions()` returns `undefined` as soon as
    /// `dims.css.cell.width === 0` (FitAddon.ts), and `fit()` then returns
    /// before touching `terminal.resize`. So a fit that did nothing is
    /// indistinguishable from a fit that worked unless we ask first.
    ///
    /// This matters because the terminal is CONSTRUCTED with the pane's real
    /// attach-time rows/cols as the seed (the geometry contract above), which
    /// is normally taller than this container. If the fit is skipped, the
    /// viewport stays at the seed height: taller than its slot, and when the
    /// pane's content is not longer than the seed row count the scroll area
    /// equals the viewport height — nothing to scroll. That is the operator's
    /// "the page is there but it will not scroll", and it is intermittent
    /// because whether the metrics are valid at fit time depends on timing.
    let geometryFitted = false;
    const syncGeometry = () => {
      if (!geometryArmed) return;
      const fitAddon = fitRef.current;
      const terminal = termRef.current;
      if (!fitAddon || !terminal) return;
      // An inactive attachment is not laid out — no box to fit. (The view now
      // hides inactive attachments with `visibility` rather than `display`
      // precisely so they keep a real box; this guard is the second line.)
      if (!containerHasArea()) return;

      // Ask before fitting: an unmeasurable cell means fit() would be a no-op,
      // and a no-op leaves the viewport at its construction seed.
      const proposed = fitAddon.proposeDimensions();
      if (!proposed) {
        geometryFitted = false;
        return;
      }
      fitAddon.fit();
      geometryFitted = true;
      const { rows, cols } = terminal;
      if (!clearedStaleSnapshotRef.current) {
        clearedStaleSnapshotRef.current = true;
        // The snapshot was painted at the pane's launch geometry. Once the
        // pane follows the viewport the TUI repaints from scratch (SIGWINCH),
        // so drop the stale absolutely-positioned frame instead of leaving it
        // underneath the repaint — that layering is what made pi's input box
        // read as garbled. Only needed when the fitted geometry actually
        // differs from the seed; an exact match needs no repaint.
        if (!seedDims || seedDims.rows !== rows || seedDims.cols !== cols) {
          terminal.clear();
          terminal.reset();
        }
      }
      const last = lastPushedRef.current;
      if (!last || last.rows !== rows || last.cols !== cols) {
        pushResize(rows, cols);
      }
    };

    // B1 (v0.0.2): forward xterm keystrokes to the tmux pane via the buffer
    // write path (cmd_pty_write_bytes → PtySession::write → tmux load-buffer
    // | paste-buffer). onData gives the already-decoded byte stream including
    // paste, IME composition, and control chords (Ctrl-C = 0x03 etc.) — we
    // round-trip every byte verbatim so the wrapped CLI sees them as if typed
    // directly into the tmux pane.
    const dataDisposable = term.onData((data: string) => {
      const bytes = new TextEncoder().encode(data);
      api.ptyWriteBytes(sessionId, Array.from(bytes)).catch((err) => {
        console.error('[SessionTerminal] ptyWriteBytes failed:', err);
      });
    });

    // Subscribe to this session's output stream BEFORE the first fit
    // pushes a resize: the pane's post-SIGWINCH redraw can start flowing
    // within milliseconds of cmd_pty_resize, and bytes emitted before the
    // listener is registered would be lost (the snapshot only covers
    // attach-time content). We filter by sessionId since the event is global.
    let unlistenOutput: (() => void) | null = null;
    let unlistenExit: (() => void) | null = null;
    const listenersReady = Promise.all([
      onSessionOutput((event) => {
        if (event.sessionId !== sessionId) return;
        const bytes = decodeSessionOutput(event.data);
        term.write(bytes);
      }).then((fn) => {
        unlistenOutput = fn;
      }),
      onSessionExit((event) => {
        if (event.sessionId !== sessionId) return;
        term.write(
          `\r\n\x1b[2m[session ${sessionId.slice(0, 12)} exited${
            event.exitCode != null ? ` code=${event.exitCode}` : ''
          }${event.signal ? ` signal=${event.signal}` : ''}]\x1b[0m\r\n`,
        );
        onExit?.(event.exitCode, event.signal);
      }).then((fn) => {
        unlistenExit = fn;
      }),
    ]);

    // Arm geometry sync only once the listeners are registered (or have
    // failed — a frozen viewport is the worse failure). Registration resolves
    // after the backend has the handler wired, so the pane's first
    // post-resize redraw cannot arrive before something is listening.
    const armGeometry = () => {
      geometryArmed = true;
      syncGeometry();
    };
    listenersReady.then(armGeometry, armGeometry);

    // Container resize (window resize, panel drag, tab switch reveal) →
    // fit → cmd_pty_resize with the fitted geometry.
    const resizeObserver = new ResizeObserver(() => syncGeometry());
    resizeObserver.observe(containerRef.current);

    // A fit that could not run yet (no valid character metrics) is retried when
    // xterm says its geometry changed or when its own element's box changes.
    // xterm re-measures on becoming visible (its IntersectionObserver) and on
    // a device-pixel-ratio change; a metrics change re-renders at the CURRENT
    // row count, which changes the element's box — so the element's box is the
    // observable signal that a previously impossible fit is now possible.
    // Without this the viewport would sit at the seed geometry until the next
    // unrelated resize, which is exactly the intermittent symptom.
    const retryIfUnfitted = () => {
      if (!geometryFitted) syncGeometry();
    };
    const elementResizeObserver = term.element ? new ResizeObserver(retryIfUnfitted) : null;
    if (term.element && elementResizeObserver) elementResizeObserver.observe(term.element);
    const resizeSubscription = term.onResize(retryIfUnfitted);

    // ── Wheel scrolling, bound by us because xterm will not ────────────────
    //
    // THE OPERATOR'S BUG (2026-10-27): "the live-seats pane cannot be scrolled
    // up freely to read history". Measured, in this order, to rule out the
    // obvious suspects:
    //
    //   * `scrollback: 10000` is already set, and 200 lines of output really do
    //     land in the buffer — so it is NOT a too-small scrollback.
    //   * there is no `scrollToBottom` anywhere in this file — so it is NOT a
    //     forced return to the bottom.
    //   * writing NEW output while the viewport is parked mid-history leaves it
    //     parked (measured: 174 before the write, 174 after) — so output does
    //     NOT yank the operator back.
    //   * `scrollLines()` moves the viewport fine — so the viewport itself is
    //     not stuck.
    //
    // What is actually broken is that THE WHEEL LISTENER IS NEVER REGISTERED.
    // xterm binds `wheel` on its element only when a mouse-reporting protocol is
    // active: the binding is gated on `16 & protocolMask` inside
    // `coreMouseService.onProtocolChange`, and the mask only becomes non-zero
    // after the program in the pane emits DECSET 1000/1006/1015. Measured: with
    // no protocol the element has no `enable-mouse-events` class; after writing
    // `ESC[?1000h` the class appears. A tmux mirror pane does not emit those
    // sequences on our behalf — tmux's own `mouse on` is consumed by tmux, and
    // this xterm is a SEPARATE display surface from the pane — so the mask stays
    // zero here and the wheel event goes nowhere.
    //
    // The fix is therefore not a scrollback number and not an option flag: bind
    // the wheel ourselves and drive xterm's own public `scrollLines`, which is
    // the same call its internal handler would have made. `preventDefault` is
    // called so the page does not also scroll underneath the panel.
    //
    // Deliberately NOT done: enabling a mouse-reporting protocol ourselves. That
    // would make the pane's program start receiving mouse events it never asked
    // for, changing what the operator's session does — a behaviour change far
    // beyond "let me read my history".
    const WHEEL_LINES_PER_NOTCH = 3;
    const onWheel = (event: WheelEvent) => {
      // A modifier wheel means something else (zoom in most apps); leave it alone
      // rather than scrolling under an intent we are guessing at.
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      // Kept even though xterm's own handler also calls preventDefault: ours is
      // bound directly to the element and must not depend on xterm's internal
      // listener being registered, which (per the note above) is exactly the
      // assumption that broke scrolling in the first place. Measured redundant
      // today, load-bearing if xterm's binding ever changes.
      event.preventDefault();
      // xterm's contract: positive scrolls DOWN toward newer output, negative
      // scrolls UP toward history. A trackpad's deltaY is negative when the
      // user scrolls up, which is the direction that reads history.
      term.scrollLines(event.deltaY < 0 ? -WHEEL_LINES_PER_NOTCH : WHEEL_LINES_PER_NOTCH);
    };
    // Registered on the terminal element (not the container) so the listener is
    // scoped to the pane the operator is actually pointing at — the container
    // also holds the padding, and a wheel over the padding is not a wheel over
    // the terminal. `passive: false` is required for preventDefault to work.
    term.element?.addEventListener('wheel', onWheel, { passive: false });

    const onWindowResize = () => syncGeometry();
    window.addEventListener('resize', onWindowResize);

    // Font settle: a webfont swap changes xterm's cell metrics without
    // resizing the container; re-fit once when fonts are ready.
    let fontsPending = true;
    document.fonts?.ready
      .then(() => {
        if (fontsPending) syncGeometry();
      })
      .catch(() => {
        /* document.fonts.ready does not reject in practice; ignore */
      });

    // Write the historical pane snapshot (listeners are already registered
    // above) so idle panes render their current content instead of a black
    // screen. Raw bytes — xterm parses the escape sequences natively.
    if (initialSnapshotB64) {
      try {
        const bytes = decodeSessionOutput(initialSnapshotB64);
        if (bytes.length > 0) term.write(bytes);
      } catch (err) {
        console.warn('[SessionTerminal] failed to decode initial snapshot:', err);
      }
    }

    return () => {
      fontsPending = false;
      window.removeEventListener('resize', onWindowResize);
      term.element?.removeEventListener('wheel', onWheel);
      resizeObserver.disconnect();
      elementResizeObserver?.disconnect();
      resizeSubscription.dispose();
      dataDisposable.dispose();
      if (unlistenOutput) unlistenOutput();
      if (unlistenExit) unlistenExit();
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
      lastPushedRef.current = null;
      inFlightResizeRef.current = null;
      pendingResizeRef.current = null;
      clearedStaleSnapshotRef.current = false;
    };
    // sessionId is expected to be stable for the lifetime of the mount;
    // callers remount with a new key when switching sessions. Geometry sync
    // reads live refs, so pane dims and the snapshot are intentionally not
    // dependencies (re-running would remount the terminal).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  return (
    <div
      ref={containerRef}
      style={{
        width: '100%',
        height: '100%',
        background: THEME.background,
        padding: 8,
        boxSizing: 'border-box',
      }}
    />
  );
};
