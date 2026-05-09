import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Zap, AlertTriangle, Clock, ArrowRight, Activity, Target, Info, ChevronRight, LayoutDashboard, Terminal, FileText } from 'lucide-react';
import './styles/tokens.css';
import { SeatLoomLogo } from './components/Avatar';
import { SupervisorPanel } from './panel/SupervisorPanel';
import { SessionsWorkspace } from './panel/SessionsWorkspace';
import { DocumentsWorkspace } from './views/DocumentsWorkspace';
import SupervisionDashboard from '../components/SupervisionDashboard';
import NavRail from '../components/NavRail';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';

type MainView = 'dashboard' | 'sessions' | 'documents';

const AppV2: React.FC = () => {
  const { t } = useLocaleStore();
  const { setActiveProject, activeProjectId, projectData } = useDataStore();
  
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
    <div style={{ display: 'flex', height: '100vh', background: 'var(--sl-bg)', overflow: 'hidden' }}>

      {/* ─── Left: Full Height Sidebar ─── */}
      <NavRail 
        activeTab={mainView === 'dashboard' ? 'dashboard' : mainView === 'sessions' ? 'seats' : 'artifacts'}
        onTabChange={(tab) => {
           if (tab === 'dashboard') setMainView('dashboard');
           else if (tab === 'seats') setMainView('sessions');
           else if (tab === 'artifacts') setMainView('documents');
        }}
        onSelectProject={(id) => setActiveProject(id)}
      />

      {/* ─── Right: Main Content Area ─── */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, overflow: 'hidden' }}>
        
        {/* Top View Switcher Header */}
        <header style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          height: 52, padding: '0 20px',
          background: 'var(--sl-surface)', borderBottom: '1px solid var(--sl-border)', flexShrink: 0,
          zIndex: 100
        }}>
          <div style={{ display: 'flex', gap: 4 }}>
            <ViewTab icon={<LayoutDashboard size={14} />} label={t.sidebar.dashboard} kbd="⌘1" active={mainView === 'dashboard'} onClick={() => setMainView('dashboard')} />
            <ViewTab icon={<Terminal size={14} />} label={t.sidebar.sessions} kbd="⌘2" active={mainView === 'sessions'} onClick={() => setMainView('sessions')} />
            <ViewTab icon={<FileText size={14} />} label={t.sidebar.artifacts} kbd="⌘3" active={mainView === 'documents'} onClick={() => setMainView('documents')} />
          </div>

          {/* Center: Global Truth Status */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 24 }}>
            {blockedCount > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <div style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--sl-red)' }} />
                <span style={{ fontSize: 11, fontWeight: 800, color: 'var(--sl-text-secondary)', textTransform: 'uppercase' }}>{blockedCount} 阻塞</span>
              </div>
            )}
            {activeCount > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <div style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--sl-green)' }} />
                <span style={{ fontSize: 11, fontWeight: 800, color: 'var(--sl-text-secondary)', textTransform: 'uppercase' }}>{activeCount} 进行中</span>
              </div>
            )}
          </div>


          <button onClick={() => setShowSupervisor(true)} style={{
             padding: '5px 16px', fontSize: 12, fontWeight: 800, 
             background: 'var(--sl-brand)', color: 'white',
             border: 'none', borderRadius: 8, cursor: 'pointer',
             boxShadow: '0 2px 6px var(--sl-brand-subtle)',
             textTransform: 'uppercase', letterSpacing: '0.05em'
          }}>Supervisor</button>
        </header>

        {/* Dynamic View Content */}
        <main style={{ flex: 1, position: 'relative', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          {mainView === 'dashboard' && <SupervisionDashboard />}
          {mainView === 'sessions' && <SessionsWorkspace />}
          {mainView === 'documents' && <DocumentsWorkspace onClose={() => setMainView('dashboard')} />}
        </main>
      </div>

      {/* ═══ IM Layer ═══ */}
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
      padding: '5px 16px', fontSize: 13, fontWeight: 700,
      background: active ? 'var(--sl-brand-subtle)' : 'transparent',
      color: active ? 'var(--sl-brand)' : 'var(--sl-text-secondary)',
      border: '1px solid ' + (active ? 'var(--sl-brand)30' : 'transparent'),
      borderRadius: 8, cursor: 'pointer',
      transition: 'all 120ms ease',
    }}
  >
    {icon}
    <span style={{ letterSpacing: '-0.01em' }}>{label}</span>
    <kbd style={{
      marginLeft: 4, padding: '1px 4px', fontSize: 10, fontWeight: 600,
      background: 'rgba(0,0,0,0.04)', borderRadius: 3, fontFamily: 'inherit',
      color: 'inherit', opacity: 0.5
    }}>{kbd}</kbd>
  </button>
);

export default AppV2;
