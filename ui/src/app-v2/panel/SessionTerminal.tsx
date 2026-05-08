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

export const SessionTerminal: React.FC<Props> = ({ sessionId, onExit }) => {
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
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(containerRef.current);
    fit.fit();

    termRef.current = term;
    fitRef.current = fit;

    // Forward keystrokes to the backend PTY.
    // onData fires for both typed characters and pasted content — xterm gives
    // us utf-8 strings. We send as raw bytes to preserve escape sequences.
    const keyDisposable = term.onData((data) => {
      const bytes: number[] = [];
      // Encode as UTF-8 bytes. Most typed chars are < 128, but pasted non-ASCII
      // needs proper encoding — TextEncoder handles it.
      const encoded = new TextEncoder().encode(data);
      for (let i = 0; i < encoded.length; i++) bytes.push(encoded[i]);
      api.ptyWriteBytes(sessionId, bytes).catch((err) => {
        console.error('[SessionTerminal] ptyWriteBytes failed:', err);
      });
    });

    // Resize observer → cmd_pty_resize.
    const resizeObserver = new ResizeObserver(() => {
      if (!fitRef.current || !termRef.current) return;
      fitRef.current.fit();
      const { rows, cols } = termRef.current;
      api.ptyResize(sessionId, rows, cols).catch((err) => {
        console.error('[SessionTerminal] ptyResize failed:', err);
      });
    });
    resizeObserver.observe(containerRef.current);

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
      keyDisposable.dispose();
      resizeObserver.disconnect();
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
