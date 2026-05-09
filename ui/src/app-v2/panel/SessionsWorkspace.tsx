// SessionsWorkspace — main-window workspace listing active tmux mirror sessions.
//
// v0.0.1: attach-only mode. Lists available tmux sessions via cmd_list_tmux_sessions
// and attaches to them. xterm is read-only (display mirror).

import React, { useEffect, useState } from 'react';
import { api, isTauri } from '../../lib/api';
import type { LiveSessionDto, TmuxSessionInfo } from '../../lib/types-dto';
import { SessionTerminal } from './SessionTerminal';

interface LiveSessionState extends LiveSessionDto {
  label: string;
}

export const SessionsWorkspace: React.FC = () => {
  const [sessions, setSessions] = useState<LiveSessionState[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tmuxSessions, setTmuxSessions] = useState<TmuxSessionInfo[]>([]);
  const [selectedTmuxSession, setSelectedTmuxSession] = useState<string>('');

  useEffect(() => {
    let cancelled = false;
    const fetchSessions = async () => {
      if (!isTauri()) {
        setTmuxSessions([]);
        return;
      }
      try {
        const list = await api.listTmuxSessions();
        if (!cancelled) setTmuxSessions(list);
      } catch (e) {
        if (!cancelled) {
          console.warn('cmd_list_tmux_sessions failed:', e);
          setTmuxSessions([]);
        }
      }
    };
    fetchSessions();
    const interval = setInterval(fetchSessions, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  const attachToTmux = async (tmuxSessionName: string) => {
    if (!isTauri()) {
      setError('Tauri backend not available — run `pnpm tauri dev`.');
      return;
    }
    setLaunching(true);
    setError(null);
    try {
      const dto = await api.attachTmuxSession({
        tmuxSessionName,
        rows: 30,
        cols: 120,
      });
      const label = `tmux: ${tmuxSessionName}`;
      setSessions((prev) => [...prev, { ...dto, label }]);
      setActiveId(dto.id);
      setSelectedTmuxSession('');
    } catch (e) {
      setError(`Failed to attach to tmux session: ${String(e)}`);
    } finally {
      setLaunching(false);
    }
  };

  const closeTab = (id: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (activeId === id) {
      const remaining = sessions.filter((s) => s.id !== id);
      setActiveId(remaining.length ? remaining[remaining.length - 1].id : null);
    }
  };

  return (
    <div style={{ display: 'flex', flex: 1, overflow: 'hidden', background: 'var(--sl-bg)' }}>
      {/* ═══ Attach rail ═══ */}
      <aside style={{
        width: 280, borderRight: '1px solid var(--sl-border)', padding: 16,
        display: 'flex', flexDirection: 'column', gap: 16,
        background: 'var(--sl-surface)', overflow: 'auto',
      }}>
        <div>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Attach to tmux
          </div>
          <select
            value={selectedTmuxSession}
            onChange={(e) => setSelectedTmuxSession(e.target.value)}
            disabled={launching}
            style={{
              width: '100%', padding: '8px 10px', marginBottom: 8,
              background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
              borderRadius: 'var(--sl-radius-sm)', fontSize: 13, color: 'var(--sl-text-primary)',
            }}
          >
            <option value="">-- Select a tmux session --</option>
            {tmuxSessions.map((s) => (
              <option key={s.session_name} value={s.session_name}>
                {s.session_name}
              </option>
            ))}
          </select>
          <button
            onClick={() => attachToTmux(selectedTmuxSession)}
            disabled={!selectedTmuxSession || launching}
            style={{
              width: '100%', padding: '8px 10px',
              background: 'var(--sl-brand)', border: '1px solid var(--sl-brand)',
              borderRadius: 'var(--sl-radius-sm)', fontSize: 13, fontWeight: 600, color: 'white',
              cursor: (!selectedTmuxSession || launching) ? 'not-allowed' : 'pointer',
              opacity: (!selectedTmuxSession || launching) ? 0.5 : 1,
            }}
          >
            Attach
          </button>
          <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', marginTop: 8, lineHeight: 1.4 }}>
            v0.0.1 attach-only · read-only mirror
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
              无附加会话。从左侧下拉框选择一个 tmux 会话并点击 Attach。
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
                cursor: 'pointer', color: 'var(--sl-text-primary)',
                whiteSpace: 'nowrap', transition: 'all 120ms ease',
              }}
            >
              <span style={{
                width: 6, height: 6, borderRadius: '50%',
                background: 'var(--sl-green)',
              }} />
              <span style={{ fontWeight: s.id === activeId ? 600 : 500 }}>{s.label}</span>
              <button
                onClick={(e) => { e.stopPropagation(); closeTab(s.id); }}
                title="关闭标签页"
                style={{ padding: 0, width: 16, height: 16, background: 'transparent', border: 'none', color: 'inherit', cursor: 'pointer', fontSize: 14, lineHeight: 1, opacity: 0.6 }}
              >×</button>
            </div>
          ))}
        </div>

        <div style={{ flex: 1, position: 'relative' }}>
          {sessions.map((s) => (
            <div
              key={s.id}
              style={{
                position: 'absolute', inset: 0,
                display: s.id === activeId ? 'flex' : 'none',
                flexDirection: 'column',
              }}
            >
              <div style={{ background: '#FEF3C7', padding: '8px', fontSize: 12, color: '#92400E', flexShrink: 0 }}>
                ⚠️ Read-only mode (v0.0.1). Typing in this terminal is disabled. Use tmux directly to send commands.
              </div>
              <div style={{ flex: 1, position: 'relative' }}>
                <SessionTerminal sessionId={s.id} />
              </div>
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
                SeatLoom tmux 镜像工作台
              </div>
              <div style={{ fontSize: 13, lineHeight: 1.6, maxWidth: 440 }}>
                从左侧选择一个现有 tmux 会话并 Attach。v0.0.1 为只读镜像模式，
                终端显示 tmux pane 的实时输出；输入请直接在 tmux 中进行。
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
