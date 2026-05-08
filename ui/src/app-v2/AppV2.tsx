import React, { useState, useEffect } from 'react';
import './styles/tokens.css';
import { SeatLoomLogo, StatusDot, Avatar } from './components/Avatar';
import { SupervisorPanel } from './panel/SupervisorPanel';
import { SessionsWorkspace } from './panel/SessionsWorkspace';
import { MOCK_CONTACTS } from './mock-data';

const AppV2: React.FC = () => {
  const [showSupervisor, setShowSupervisor] = useState(() => localStorage.getItem('sl-supervisor-open') === 'true');

  // Persist open state
  useEffect(() => {
    localStorage.setItem('sl-supervisor-open', showSupervisor ? 'true' : 'false');
  }, [showSupervisor]);

  useEffect(() => {
    const fn = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); setShowSupervisor(p => !p); }
    };
    window.addEventListener('keydown', fn); return () => window.removeEventListener('keydown', fn);
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--sl-bg)', overflow: 'hidden' }}>

      {/* ═══ Top Bar ═══ */}
      <header style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        height: 52, padding: '0 20px',
        background: 'var(--sl-surface)', borderBottom: '1px solid var(--sl-border)', flexShrink: 0,
      }}>
        {/* Left */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}>
            <SeatLoomLogo />
            <span style={{ fontSize: 15, fontWeight: 700, color: 'var(--sl-text-primary)', letterSpacing: '-0.02em' }}>SeatLoom</span>
          </div>
          <div style={{ width: 1, height: 20, background: 'var(--sl-border)' }} />
          <button style={{
            display: 'flex', alignItems: 'center', gap: 6, padding: '4px 10px',
            background: 'var(--sl-surface-hover)', border: '1px solid var(--sl-border-light)',
            borderRadius: 'var(--sl-radius-md)', cursor: 'pointer', fontSize: 13, fontWeight: 500, color: 'var(--sl-text-primary)',
          }}>
            <span style={{ fontSize: 14 }}>📁</span>
            <span>SeatLoom 主项目</span>
            <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>▾</span>
          </button>
        </div>

        {/* Center: Supervisor trigger */}
        <button onClick={() => setShowSupervisor(true)} style={{
          display: 'flex', alignItems: 'center', gap: 10, padding: '6px 16px',
          background: 'var(--sl-bg)', border: '1px solid var(--sl-border)',
          borderRadius: 'var(--sl-radius-lg)', cursor: 'pointer', fontSize: 13,
          color: 'var(--sl-text-tertiary)', minWidth: 300, transition: 'all 120ms ease',
        }}
          onMouseEnter={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; e.currentTarget.style.color = 'var(--sl-text-secondary)'; }}
          onMouseLeave={e => { e.currentTarget.style.borderColor = 'var(--sl-border)'; e.currentTarget.style.color = 'var(--sl-text-tertiary)'; }}
        >
          <span style={{ fontSize: 14, opacity: 0.5 }}>💬</span>
          <span style={{ flex: 1, textAlign: 'left' }}>打开 Supervisor...</span>
          <kbd style={{
            padding: '1px 6px', fontSize: 11, fontWeight: 500,
            background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)',
            borderRadius: 'var(--sl-radius-sm)', fontFamily: 'inherit',
          }}>⌘K</kbd>
        </button>

        {/* Right */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <StatusDot color="var(--sl-red)" label="阻塞" value={1} />
            <StatusDot color="var(--sl-amber)" label="待决" value={3} />
            <StatusDot color="var(--sl-green)" label="进行" value={5} />
          </div>
          <div style={{ width: 1, height: 20, background: 'var(--sl-border)' }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            {MOCK_CONTACTS.filter(c => c.type === 'seat').map(c => (
              <Avatar key={c.id} char={c.avatar} color={c.color} size={28} online={c.online} />
            ))}
          </div>
        </div>
      </header>

      {/* ═══ Workspace ═══ */}
      <SessionsWorkspace />

      {/* ═══ Floating Supervisor ═══ */}
      {showSupervisor && <SupervisorPanel onClose={() => setShowSupervisor(false)} />}
    </div>
  );
};

export default AppV2;
