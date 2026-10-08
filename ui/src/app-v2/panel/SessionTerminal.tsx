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
    const syncGeometry = () => {
      if (!geometryArmed) return;
      const fitAddon = fitRef.current;
      const terminal = termRef.current;
      if (!fitAddon || !terminal) return;
      // An inactive attachment is display:none — no layout to fit. Fitting it
      // would propose 0×0 and resize the pane to nothing; the ResizeObserver
      // fires with real dimensions when the attachment becomes visible again.
      if (!containerHasArea()) return;
      fitAddon.fit();
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
      resizeObserver.disconnect();
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
