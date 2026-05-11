// SessionTerminal — embedded xterm.js terminal bound to a live SeatLoom PTY.
//
// Subscribes to Tauri `session:output` events for the given sessionId, writes
// raw bytes to xterm, forwards keystrokes back via cmd_pty_write_bytes, and
// notifies the backend on ResizeObserver-driven resizes.
//
// Visual tuning: xterm canvas renderer with sensible dark theme aligned to
// SeatLoom brand tokens.

import React, { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';

import {
  api,
  decodeSessionOutput,
  onSessionExit,
  onSessionOutput,
} from '../../lib/api';

interface Props {
  sessionId: string;
  onExit?: (code: number | null, signal: string | null) => void;
  /// Authoritative tmux pane dimensions queried at attach time. When present,
  /// xterm initialises at this size so output is not re-wrapped at a smaller
  /// width. Absent for legacy callers; falls back to FitAddon defaults.
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

export const SessionTerminal: React.FC<Props> = ({ sessionId, onExit, paneRows, paneCols, initialSnapshotB64 }) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
      fontSize: 13,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: 'bar',
      scrollback: 10000,
      allowProposedApi: true,
      theme: THEME,
      ...(paneRows && paneCols ? { rows: paneRows, cols: paneCols } : {}),
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(containerRef.current);
    // If tmux authoritative dimensions came in from the backend, keep them —
    // re-fitting would re-wrap to the container which may be narrower than the
    // tmux pane. Only call fit() when we're running without explicit sizing.
    if (!(paneRows && paneCols)) {
      fit.fit();
    }

    termRef.current = term;
    fitRef.current = fit;

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

    // Resize observer → cmd_pty_resize. Skipped when tmux authoritative
    // dimensions are in use — re-fitting would re-wrap to the container width
    // (often narrower than the tmux pane) and re-introduce the double-wrap bug.
    const hasAuthoritativeSize = !!(paneRows && paneCols);
    const resizeObserver = new ResizeObserver(() => {
      if (hasAuthoritativeSize) return;
      if (!fitRef.current || !termRef.current) return;
      fitRef.current.fit();
      const { rows, cols } = termRef.current;
      api.ptyResize(sessionId, rows, cols).catch((err) => {
        console.error('[SessionTerminal] ptyResize failed:', err);
      });
    });
    resizeObserver.observe(containerRef.current);

    // Write the historical pane snapshot before subscribing to new output, so
    // idle panes render their current content instead of a black screen.
    if (initialSnapshotB64) {
      try {
        const bytes = decodeSessionOutput(initialSnapshotB64);
        if (bytes.length > 0) term.write(bytes);
      } catch (err) {
        console.warn('[SessionTerminal] failed to decode initial snapshot:', err);
      }
    }

    // Subscribe to this session's output stream. We filter by sessionId since
    // the event is global.
    let unlistenOutput: (() => void) | null = null;
    let unlistenExit: (() => void) | null = null;
    onSessionOutput((event) => {
      if (event.sessionId !== sessionId) return;
      const bytes = decodeSessionOutput(event.data);
      term.write(bytes);
    }).then((fn) => {
      unlistenOutput = fn;
    });
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
    });

    return () => {
      resizeObserver.disconnect();
      dataDisposable.dispose();
      if (unlistenOutput) unlistenOutput();
      if (unlistenExit) unlistenExit();
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
    // sessionId is expected to be stable for the lifetime of the mount;
    // callers remount with a new key when switching sessions.
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
