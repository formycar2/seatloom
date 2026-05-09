// SessionsWorkspace — main-window workspace listing active PTY sessions.
//
// Replaces the old "打开 Supervisor" empty state. Shows:
//   - A left rail of seat launchers (aegis / lyra / mira / nimbus / flux / custom shell)
//   - A main area with one tab per active session, each hosting a SessionTerminal
//   - A "+" new-session launcher prompting for runtime + command

import React, { useEffect, useState, useMemo } from 'react';
import { api, isTauri } from '../../lib/api';
import type { LaunchRequest, LiveSessionDto } from '../../lib/types-dto';
import { useLiveSessionsStore } from '../../stores/useLiveSessionsStore';
import { useDataStore } from '../../stores/useDataStore';
import { SessionTerminal } from './SessionTerminal';

interface LiveSessionState extends LiveSessionDto {
  exited?: boolean;
  exitCode?: number | null;
  label: string;
  seatName?: string;
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
  const { activeProjectId, projectData } = useDataStore();
  const [sessions, setSessions] = useState<LiveSessionState[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const setSessionForSeat = useLiveSessionsStore((s) => s.setSessionForSeat);
  const clearSession = useLiveSessionsStore((s) => s.clearSession);

  // Derived seats from useDataStore (AD-014: use source of truth)
  const seats = useMemo(() => {
    const current = activeProjectId ? projectData[activeProjectId] : null;
    return current?.seats || [];
  }, [activeProjectId, projectData]);

  const launch = async (seatName: string | null, seatId: string | null, launcher: QuickLauncher) => {
    if (!isTauri()) {
      setError('Tauri backend not available — run `pnpm tauri dev`.');
      return;
    }
    setLaunching(true);
    setError(null);
    try {
      const request: LaunchRequest = {
        seatId: seatId || undefined,
        runtime: launcher.runtime,
        command: launcher.command,
        args: launcher.args,
      };
      const live = await api.launchSession(request);
      const label = seatName
        ? `${seatName} · ${launcher.label}`
        : `${launcher.label}`;
      setSessions((prev) => [...prev, { ...live, label, seatName: seatName || undefined }]);
      setActiveId(live.id);
      if (seatName) setSessionForSeat(seatName, live.id);
    } catch (e) {
      setError(`launch failed: ${String(e)}`);
    } finally {
      setLaunching(false);
    }
  };

  const kill = async (id: string) => {
    // AD-015: Kill confirmation to prevent accidental loss of work
    if (!window.confirm('确定要强行终止该会话吗？未保存的工作将会丢失。')) {
      return;
    }
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
    clearSession(id);
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
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Quick launch
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {QUICK_LAUNCHERS.map((l) => (
              <button
                key={l.key}
                disabled={launching}
                onClick={() => launch(null, null, l)}
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
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Project Seats
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {seats.length === 0 && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)' }}>
                {activeProjectId ? '当前项目暂无席位数据。' : '请先在左侧选择一个项目频道。'}
              </div>
            )}
            {seats.map((seat) => {
              const suggestedLabel = SEAT_SUGGESTIONS[seat.name.toLowerCase()] ?? 'Claude Code';
              const launcher =
                QUICK_LAUNCHERS.find((l) => l.label === suggestedLabel) ?? QUICK_LAUNCHERS[0];
              return (
                <button
                  key={seat.id}
                  disabled={launching}
                  onClick={() => launch(seat.name, seat.id, launcher)}
                  title={`使用 ${launcher.label} 启动 ${seat.name} 席位`}
                  style={{
                    textAlign: 'left', padding: '10px',
                    background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
                    borderRadius: 'var(--sl-radius-md)', fontSize: 13, color: 'var(--sl-text-primary)',
                    cursor: launching ? 'wait' : 'pointer',
                    display: 'flex', flexDirection: 'column', gap: 2,
                  }}
                >
                  <span style={{ fontWeight: 700 }}>{seat.name}</span>
                  <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', fontWeight: 500 }}>
                    {typeof seat.role === 'string' ? seat.role : seat.role.Custom} → {launcher.label}
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
              无活跃会话。从左侧席位列表中启动一个 agent 来替代 tmux。
            </div>
          ) : sessions.map((s) => (
            <div
              key={s.id}
              onClick={() => setActiveId(s.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                padding: '6px 12px', fontSize: 12,
                background: s.id === activeId ? 'var(--sl-bg)' : 'transparent',
                borderRight: '1px solid var(--sl-border-light)',
                borderTop: s.id === activeId ? '2px solid var(--sl-brand)' : '2px solid transparent',
                cursor: 'pointer', color: s.exited ? 'var(--sl-text-tertiary)' : 'var(--sl-text-primary)',
                whiteSpace: 'nowrap', transition: 'all 120ms ease',
              }}
            >
              <span style={{
                width: 6, height: 6, borderRadius: '50%',
                background: s.exited ? 'var(--sl-text-tertiary)' : 'var(--sl-green)',
              }} />
              <span style={{ fontWeight: s.id === activeId ? 600 : 500 }}>{s.label}</span>
              
              {/* Exit Code Pill */}
              {s.exited && s.exitCode !== undefined && (
                <span style={{
                  fontSize: 10, fontWeight: 700, padding: '1px 4px', borderRadius: 4,
                  background: s.exitCode === 0 ? 'var(--sl-green-subtle)' : 'var(--sl-red-subtle)',
                  color: s.exitCode === 0 ? 'var(--sl-green)' : 'var(--sl-red)',
                  border: `1px solid ${s.exitCode === 0 ? 'var(--sl-green)' : 'var(--sl-red)'}40`
                }}>
                  exit {s.exitCode}
                </span>
              )}

              {!s.exited && (
                <button
                  onClick={(e) => { e.stopPropagation(); kill(s.id); }}
                  title="终止会话"
                  style={{ padding: 0, width: 16, height: 16, background: 'transparent', border: 'none', color: 'inherit', cursor: 'pointer', fontSize: 14, lineHeight: 1, opacity: 0.6 }}
                >×</button>
              )}
              {s.exited && (
                <button
                  onClick={(e) => { e.stopPropagation(); closeTab(s.id); }}
                  title="关闭标签页"
                  style={{ padding: 0, width: 16, height: 16, background: 'transparent', border: 'none', color: 'inherit', cursor: 'pointer', fontSize: 14, lineHeight: 1, opacity: 0.6 }}
                >×</button>
              )}
            </div>
          ))}
        </div>

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
                SeatLoom 会话工作台
              </div>
              <div style={{ fontSize: 13, lineHeight: 1.6, maxWidth: 440 }}>
                从左侧侧边栏启动一个 agent 席位。每个会话都是一个真实的 PTY 终端，
                其输出将实时串流至此并持久化存储。
              </div>
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)', marginTop: 12 }}>
                按下 <kbd style={{ padding: '1px 6px', fontSize: 11, background: 'var(--sl-surface-hover)', border: '1px solid var(--sl-border-light)', borderRadius: 3 }}>⌘K</kbd> 呼出 Supervisor 指令面板。
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
