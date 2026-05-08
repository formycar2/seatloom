// SessionsWorkspace — main-window workspace listing active PTY sessions.
//
// Replaces the old "打开 Supervisor" empty state. Shows:
//   - A left rail of seat launchers (aegis / lyra / mira / nimbus / flux / custom shell)
//   - A main area with one tab per active session, each hosting a SessionTerminal
//   - A "+" new-session launcher prompting for runtime + command
//
// This is the primary surface that replaces tmux: side-by-side wrapped agent
// processes, always visible, streamable, inject-able.

import React, { useEffect, useState } from 'react';
import { api, isTauri } from '../../lib/api';
import type { LaunchRequest, LiveSessionDto, SeatDto } from '../../lib/types-dto';
import { SessionTerminal } from './SessionTerminal';

interface LiveSessionState extends LiveSessionDto {
  exited?: boolean;
  exitCode?: number | null;
  label: string;
}

interface QuickLauncher {
  key: string;
  label: string;
  runtime: string;
  command: string;
  args: string[];
}

const QUICK_LAUNCHERS: QuickLauncher[] = [
  { key: 'claude', label: 'Claude Code', runtime: 'ClaudeCode', command: 'claude', args: [] },
  { key: 'gemini', label: 'Gemini CLI', runtime: 'GeminiCli', command: 'gemini', args: [] },
  { key: 'codex',  label: 'Codex CLI',  runtime: 'Codex',      command: 'codex',  args: [] },
  { key: 'zsh',    label: 'Shell (zsh)', runtime: 'Custom',    command: 'zsh',    args: ['-l'] },
];

const SEAT_SUGGESTIONS: Record<string, string> = {
  aegis:  'Claude Code',
  lyra:   'Gemini CLI',
  mira:   'Gemini CLI',
  nimbus: 'Claude Code',
  flux:   'Shell (zsh)',
};

export const SessionsWorkspace: React.FC = () => {
  const [seats, setSeats] = useState<SeatDto[]>([]);
  const [sessions, setSessions] = useState<LiveSessionState[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Hydrate seat list from backend on mount. Silently tolerates browser-dev.
  useEffect(() => {
    if (!isTauri()) return;
    api.listSeats()
      .then((rows) => setSeats(rows))
      .catch((e) => setError(`list seats: ${String(e)}`));
  }, []);

  const launch = async (seat: SeatDto | null, launcher: QuickLauncher) => {
    if (!isTauri()) {
      setError('Tauri backend not available — run `pnpm tauri dev`.');
      return;
    }
    setLaunching(true);
    setError(null);
    try {
      const request: LaunchRequest = {
        seatId: seat?.id,
        runtime: launcher.runtime,
        command: launcher.command,
        args: launcher.args,
      };
      const live = await api.launchSession(request);
      const label = seat
        ? `${seat.name} · ${launcher.label}`
        : `${launcher.label}`;
      setSessions((prev) => [...prev, { ...live, label }]);
      setActiveId(live.id);
    } catch (e) {
      setError(`launch failed: ${String(e)}`);
    } finally {
      setLaunching(false);
    }
  };

  const kill = async (id: string) => {
    try {
      await api.killSession(id);
    } catch (e) {
      console.error(e);
    }
  };

  const closeTab = (id: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (activeId === id) {
      const remaining = sessions.filter((s) => s.id !== id);
      setActiveId(remaining.length ? remaining[remaining.length - 1].id : null);
    }
  };

  const markExited = (id: string, code: number | null) => {
    setSessions((prev) =>
      prev.map((s) => (s.id === id ? { ...s, exited: true, exitCode: code } : s)),
    );
  };

  return (
    <div style={{ display: 'flex', flex: 1, overflow: 'hidden', background: 'var(--sl-bg)' }}>
      {/* ═══ Launchers rail ═══ */}
      <aside style={{
        width: 240, borderRight: '1px solid var(--sl-border)', padding: 16,
        display: 'flex', flexDirection: 'column', gap: 16,
        background: 'var(--sl-surface)', overflow: 'auto',
      }}>
        <div>
          <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Quick launch
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {QUICK_LAUNCHERS.map((l) => (
              <button
                key={l.key}
                disabled={launching}
                onClick={() => launch(null, l)}
                style={{
                  textAlign: 'left', padding: '8px 10px',
                  background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
                  borderRadius: 'var(--sl-radius-sm)', fontSize: 13, color: 'var(--sl-text-primary)',
                  cursor: launching ? 'wait' : 'pointer',
                }}
              >
                {l.label}
              </button>
            ))}
          </div>
        </div>

        <div>
          <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Seats
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {seats.length === 0 && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)' }}>
                {isTauri() ? 'No seats loaded.' : 'Backend offline — run `pnpm tauri dev`.'}
              </div>
            )}
            {seats.map((seat) => {
              const suggestedLabel = SEAT_SUGGESTIONS[seat.name] ?? 'Claude Code';
              const launcher =
                QUICK_LAUNCHERS.find((l) => l.label === suggestedLabel) ?? QUICK_LAUNCHERS[0];
              return (
                <button
                  key={seat.id}
                  disabled={launching}
                  onClick={() => launch(seat, launcher)}
                  title={`Launch ${launcher.label} as ${seat.name}`}
                  style={{
                    textAlign: 'left', padding: '8px 10px',
                    background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
                    borderRadius: 'var(--sl-radius-sm)', fontSize: 13, color: 'var(--sl-text-primary)',
                    cursor: launching ? 'wait' : 'pointer',
                    display: 'flex', flexDirection: 'column', gap: 2,
                  }}
                >
                  <span style={{ fontWeight: 600 }}>{seat.name}</span>
                  <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                    {seat.defaultRuntime ?? 'Generic'} → {launcher.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {error && (
          <div style={{ padding: 8, background: 'rgba(255,123,114,0.1)', border: '1px solid var(--sl-red)', borderRadius: 4, fontSize: 12, color: 'var(--sl-red)' }}>
            {error}
          </div>
        )}
      </aside>

      {/* ═══ Terminal area ═══ */}
      <main style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* Tab strip */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 2,
          padding: '0 8px', background: 'var(--sl-surface)',
          borderBottom: '1px solid var(--sl-border)', minHeight: 36,
          overflow: 'auto', flexShrink: 0,
        }}>
          {sessions.length === 0 ? (
            <div style={{ fontSize: 13, color: 'var(--sl-text-tertiary)', padding: '0 12px' }}>
              No active sessions. Launch one from the left rail to replace your tmux pane.
            </div>
          ) : sessions.map((s) => (
            <div
              key={s.id}
              onClick={() => setActiveId(s.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                padding: '6px 10px', fontSize: 12,
                background: s.id === activeId ? 'var(--sl-bg)' : 'transparent',
                borderRight: '1px solid var(--sl-border-light)',
                borderTop: s.id === activeId ? '2px solid var(--sl-brand)' : '2px solid transparent',
                cursor: 'pointer', color: s.exited ? 'var(--sl-text-tertiary)' : 'var(--sl-text-primary)',
                whiteSpace: 'nowrap',
              }}
            >
              <span style={{
                width: 6, height: 6, borderRadius: '50%',
                background: s.exited ? 'var(--sl-text-tertiary)' : 'var(--sl-green)',
              }} />
              <span>{s.label}</span>
              {!s.exited && (
                <button
                  onClick={(e) => { e.stopPropagation(); kill(s.id); }}
                  title="Kill session"
                  style={{ padding: 0, width: 16, height: 16, background: 'transparent', border: 'none', color: 'inherit', cursor: 'pointer', fontSize: 14, lineHeight: 1 }}
                >×</button>
              )}
              {s.exited && (
                <button
                  onClick={(e) => { e.stopPropagation(); closeTab(s.id); }}
                  title="Close tab"
                  style={{ padding: 0, width: 16, height: 16, background: 'transparent', border: 'none', color: 'inherit', cursor: 'pointer', fontSize: 14, lineHeight: 1 }}
                >×</button>
              )}
            </div>
          ))}
        </div>

        {/* Terminal mount — only the active session renders; others unmount to
            free xterm canvas, but the backend PTY keeps running and we'll
            resubscribe to scrollback on re-activation (xterm scrollback is
            in-process, so switching tabs clears it — acceptable for v0.1). */}
        <div style={{ flex: 1, position: 'relative' }}>
          {sessions.map((s) => (
            <div
              key={s.id}
              style={{
                position: 'absolute', inset: 0,
                display: s.id === activeId ? 'block' : 'none',
              }}
            >
              <SessionTerminal
                sessionId={s.id}
                onExit={(code) => markExited(s.id, code)}
              />
            </div>
          ))}
          {sessions.length === 0 && (
            <div style={{
              display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
              height: '100%', color: 'var(--sl-text-tertiary)', textAlign: 'center',
              padding: 40, gap: 12,
            }}>
              <div style={{ fontSize: 48, opacity: 0.15 }}>⌨</div>
              <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--sl-text-secondary)' }}>
                SeatLoom Sessions
              </div>
              <div style={{ fontSize: 13, lineHeight: 1.6, maxWidth: 440 }}>
                Launch a wrapped agent from the left rail. Each session is a real PTY with a
                live terminal, its output streamed to this window and persisted to
                <code style={{ padding: '0 4px', background: 'var(--sl-surface-hover)', borderRadius: 3 }}>
                  .seatloom/transcripts/&lt;id&gt;.raw.log
                </code>.
              </div>
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)', marginTop: 12 }}>
                Press <kbd style={{ padding: '1px 6px', fontSize: 11, background: 'var(--sl-surface-hover)', border: '1px solid var(--sl-border-light)', borderRadius: 3 }}>⌘K</kbd> for Supervisor.
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
