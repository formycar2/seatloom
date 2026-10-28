// SessionsWorkspace — the `sessions` tab.
//
// The tab holds two workspaces, selected by a within-tab mode switch. This is
// not a new route, not a new window and not top-level navigation: the skeleton,
// the tab strip and the ⌘2 shortcut are untouched, and each workspace keeps its
// own layout, its own states and its own wording.
//
//   history — the historical session corpus by project (LYRA-2026-10-14 contract
//             §8.2 slice 3): project list -> session list -> the conversation,
//             each turn carrying the line number it came from in the original
//             file. This is the default because it is what the operator opens
//             this tab to see.
//   mirror  — the tmux mirror that was here first (TmuxMirrorWorkspace.tsx,
//             moved verbatim out of this file so neither workspace has to
//             re-lay-out the other).

import React, { useState } from 'react';
import { HistorySessionsPanel } from './HistorySessionsPanel';
import { TmuxMirrorWorkspace } from './TmuxMirrorWorkspace';

type SessionsTabMode = 'history' | 'mirror';

export const SessionsWorkspace: React.FC = () => {
  const [mode, setMode] = useState<SessionsTabMode>('history');

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        flex: 1,
        overflow: 'hidden',
        background: 'var(--sl-bg)',
      }}
    >
      {/* Within-tab mode switch. Same pattern the Supervisor panel's intent entry
          uses (two modes, one surface) — no new nav item, no new route. */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '0 12px',
          height: 34,
          flexShrink: 0,
          background: 'var(--sl-surface)',
          borderBottom: '1px solid var(--sl-border)',
        }}
      >
        <div
          style={{
            display: 'flex',
            gap: 2,
            padding: 2,
            background: 'var(--sl-bg)',
            borderRadius: 'var(--sl-radius-md)',
            border: '1px solid var(--sl-border-light)',
          }}
        >
          {(
            [
              ['history', '历史会话'],
              ['mirror', '实时镜像'],
            ] as const
          ).map(([m, label]) => (
            <button
              key={m}
              type="button"
              onClick={() => setMode(m)}
              style={{
                padding: '3px 12px',
                fontSize: 12,
                fontWeight: 600,
                background: mode === m ? 'var(--sl-brand-subtle)' : 'transparent',
                color: mode === m ? 'var(--sl-brand)' : 'var(--sl-text-secondary)',
                border: 'none',
                borderRadius: 'var(--sl-radius-sm)',
                cursor: 'pointer',
              }}
            >
              {label}
            </button>
          ))}
        </div>
        <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
          {mode === 'history'
            ? '按项目看你自己开过的会话 —— 一个项目就是一个启动过 agent 的目录'
            : '双向镜像一个在跑的 tmux 会话;关掉标签页不会杀掉它'}
        </span>
      </div>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
        {mode === 'history' ? <HistorySessionsPanel /> : <TmuxMirrorWorkspace />}
      </div>
    </div>
  );
};

export default SessionsWorkspace;
