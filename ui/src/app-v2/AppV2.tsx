import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Zap, AlertTriangle, Clock, ArrowRight, Activity, Target, Info, ChevronRight, LayoutDashboard, Terminal, FileText } from 'lucide-react';
import './styles/tokens.css';
import { SupervisorPanel } from './panel/SupervisorPanel';
import { SessionsWorkspace } from './panel/SessionsWorkspace';
import { DocumentsWorkspace } from './views/DocumentsWorkspace';
import SupervisionDashboard from '../components/SupervisionDashboard';
import { MOCK_CONTACTS } from './mock-data';

import { useDataStore } from '../stores/useDataStore';

type MainView = 'dashboard' | 'sessions' | 'documents';

const AppV2: React.FC = () => {
  const { activeProjectId, projectData } = useDataStore();
  const workItems = activeProjectId ? projectData[activeProjectId]?.workItems ?? [] : [];
  const blockedCount = workItems.filter(wi => wi.status === 'Blocked').length;
  const activeCount = workItems.filter(wi => wi.status === 'Active').length;

  const [showSupervisor, setShowSupervisor] = useState(() => localStorage.getItem('sl-supervisor-open') === 'true');
  const [mainView, setMainView] = useState<MainView>(() => {
    const saved = localStorage.getItem('sl-main-view');
    if (saved === 'sessions') return 'sessions';
    if (saved === 'documents') return 'documents';
    return 'dashboard';
  });

  useEffect(() => {
    localStorage.setItem('sl-supervisor-open', showSupervisor ? 'true' : 'false');
  }, [showSupervisor]);
  useEffect(() => {
    localStorage.setItem('sl-main-view', mainView);
  }, [mainView]);

  useEffect(() => {
    const fn = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); setShowSupervisor(p => !p); }
      if ((e.metaKey || e.ctrlKey) && e.key === '1') { e.preventDefault(); setMainView('dashboard'); }
      if ((e.metaKey || e.ctrlKey) && e.key === '2') { e.preventDefault(); setMainView('sessions'); }
      if ((e.metaKey || e.ctrlKey) && e.key === '3') { e.preventDefault(); setMainView('documents'); }
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
        zIndex: 100
      }}>
        {/* Left: brand + view switcher */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}>
            <div style={{ width: 24, height: 24, borderRadius: 6, background: 'var(--sl-brand)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white', fontWeight: 900, fontSize: 14 }}>S</div>
            <span style={{ fontSize: 15, fontWeight: 700, color: 'var(--sl-text-primary)', letterSpacing: '-0.02em' }}>SeatLoom</span>
          </div>
          <div style={{ width: 1, height: 20, background: 'var(--sl-border)' }} />
          <div style={{ display: 'flex', gap: 2 }}>
            <ViewTab icon={<LayoutDashboard size={14} />} label="Dashboard" kbd="⌘1" active={mainView === 'dashboard'} onClick={() => setMainView('dashboard')} />
            <ViewTab icon={<Terminal size={14} />} label="Sessions" kbd="⌘2" active={mainView === 'sessions'} onClick={() => setMainView('sessions')} />
            <ViewTab icon={<FileText size={14} />} label="Docs" kbd="⌘3" active={mainView === 'documents'} onClick={() => setMainView('documents')} />
          </div>
        </div>

        {/* Center: Global Status (Compact) */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 24 }}>
          {blockedCount > 0 && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <div style={{ width: 8, height: 8, borderRadius: '50%', background: 'var(--sl-red)' }} />
              <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--sl-text-secondary)' }}>{blockedCount} 阻塞</span>
            </div>
          )}
          {activeCount > 0 && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <div style={{ width: 8, height: 8, borderRadius: '50%', background: 'var(--sl-green)' }} />
              <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--sl-text-secondary)' }}>{activeCount} 进行中</span>
            </div>
          )}
        </div>

        {/* Right: Seat Avatars */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
           <button onClick={() => setShowSupervisor(true)} style={{
             padding: '4px 12px', fontSize: 12, fontWeight: 600, background: 'var(--sl-brand-subtle)', color: 'var(--sl-brand)',
             border: '1px solid var(--sl-brand)30', borderRadius: 6, cursor: 'pointer'
           }}>Supervisor IM</button>
        </div>
      </header>

      {/* ═══ Main Background Workspace ═══ */}
      <div style={{ flex: 1, position: 'relative', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {mainView === 'dashboard' && <SupervisionDashboard />}
        {mainView === 'sessions' && <SessionsWorkspace />}
        {mainView === 'documents' && <DocumentsWorkspace onClose={() => setMainView('dashboard')} />}
      </div>

      {/* ═══ Floating Supervisor Layer ═══ */}
      {showSupervisor && <SupervisorPanel onClose={() => setShowSupervisor(false)} />}
    </div>
  );
};

const ViewTab: React.FC<{
  icon: React.ReactNode;
  label: string;
  kbd: string;
  active: boolean;
  onClick: () => void;
}> = ({ icon, label, kbd, active, onClick }) => (
  <button
    onClick={onClick}
    style={{
      display: 'flex', alignItems: 'center', gap: 8,
      padding: '5px 12px', fontSize: 13, fontWeight: 600,
      background: active ? 'var(--sl-brand-subtle)' : 'transparent',
      color: active ? 'var(--sl-brand)' : 'var(--sl-text-secondary)',
      border: '1px solid ' + (active ? 'var(--sl-brand)30' : 'transparent'),
      borderRadius: 'var(--sl-radius-md)', cursor: 'pointer',
      transition: 'all 120ms ease',
    }}
  >
    {icon}
    <span>{label}</span>
    <kbd style={{
      marginLeft: 4, padding: '1px 4px', fontSize: 10, fontWeight: 500,
      background: 'rgba(0,0,0,0.05)', borderRadius: 3, fontFamily: 'inherit',
      color: 'inherit', opacity: 0.6
    }}>{kbd}</kbd>
  </button>
);

export default AppV2;
