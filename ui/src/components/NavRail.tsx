import React from 'react';
import {
  BarChart3,
  Inbox,
  ClipboardList,
  Calendar,
  Users,
  Package,
  BookOpen,
  PanelLeftClose,
  PanelLeftOpen,
} from 'lucide-react';
import { useAppStore } from '../stores/useAppStore';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useResponsive } from '../hooks/useResponsive';
import ProjectSwitcher from './ProjectSwitcher';
import { SeatLoomLogo } from '../app-v2/components/Avatar';

export type NavTab =
  | 'dashboard'
  | 'inbox'
  | 'workitems'
  | 'timeline'
  | 'seats'
  | 'artifacts'
  | 'playbook';

interface NavItem {
  id: NavTab;
  icon: React.ElementType;
  label: string;
  badge?: number;
}

interface NavRailProps {
  activeTab: NavTab;
  onTabChange: (tab: NavTab) => void;
  onSelectProject: (id: string) => void;
}

const NavRail: React.FC<NavRailProps> = ({ activeTab, onTabChange, onSelectProject }) => {
  const { t } = useLocaleStore();
  const { navCollapsed, toggleNav } = useAppStore();
  const { activeProjectId, projectData } = useDataStore();
  const { isMobile, isCompact, isTablet } = useResponsive();

  const effectiveCollapsed = navCollapsed || isCompact || isTablet;

  if (isMobile) return null;

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const inboxCount = currentData?.inboxItems.length || 0;

  const navItems: NavItem[] = [
    { id: 'dashboard', icon: BarChart3, label: t.sidebar.dashboard },
    { id: 'inbox', icon: Inbox, label: t.sidebar.inbox, badge: inboxCount > 0 ? inboxCount : undefined },
    { id: 'workitems', icon: ClipboardList, label: t.sidebar.workitems },
    { id: 'timeline', icon: Calendar, label: t.sidebar.timeline },
    { id: 'seats', icon: Users, label: t.sidebar.seats },
    { id: 'artifacts', icon: Package, label: t.sidebar.artifacts },
    { id: 'playbook', icon: BookOpen, label: t.sidebar.playbook },
  ];

  return (
    <nav
      className="flex flex-col h-full border-r bg-canvas transition-all duration-layout shrink-0 z-[300]"
      style={{ width: effectiveCollapsed ? 52 : 200, overflow: 'visible' }}
    >
      {/* ─── 1. Logo Slot (Absolute Top-Left) ─── */}
      <div style={{ 
        height: 52, display: 'flex', alignItems: 'center', 
        padding: effectiveCollapsed ? '0' : '0 16px',
        justifyContent: effectiveCollapsed ? 'center' : 'flex-start',
        gap: 12, borderBottom: '1px solid var(--sl-border-subtle)',
        background: 'var(--sl-surface)'
      }}>
        <SeatLoomLogo size={24} />
        {!effectiveCollapsed && (
          <span style={{ fontSize: 16, fontWeight: 900, color: 'var(--sl-text-primary)', letterSpacing: '-0.03em' }}>SeatLoom</span>
        )}
      </div>

      {/* ─── 2. Project Switcher (Follows Logo) ─── */}
      <div style={{ 
        padding: '12px 8px', borderBottom: '1px solid var(--sl-border-subtle)',
        background: 'var(--sl-bg)'
      }}>
        <ProjectSwitcher onSelect={onSelectProject} />
      </div>

      {/* ─── 3. Nav Items ─── */}
      <div className="flex-1 py-6 px-2 space-y-1 overflow-y-auto">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              title={effectiveCollapsed ? item.label : undefined}
              className={`
                w-full flex items-center gap-3 rounded-lg transition-all duration-150 relative
                ${effectiveCollapsed ? 'justify-center px-0 py-3' : 'px-3 py-2.5'}
                ${isActive
                  ? 'bg-primary/10 text-primary font-bold'
                  : 'text-ink-secondary hover:bg-panel hover:text-ink'
                }
              `}
            >
              {isActive && (
                <div className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-6 bg-primary rounded-r-full" />
              )}

              <item.icon size={18} className="shrink-0" />

              {!effectiveCollapsed && (
                <>
                  <span className="text-[13px] font-bold truncate tracking-tight">{item.label}</span>
                  {item.badge !== undefined && (
                    <span className="ml-auto text-[10px] font-black bg-primary text-white rounded-full min-w-[18px] h-[18px] flex items-center justify-center px-1 shadow-sm">
                      {item.badge > 99 ? '99+' : item.badge}
                    </span>
                  )}
                </>
              )}

              {effectiveCollapsed && item.badge !== undefined && (
                <span className="absolute top-2 right-2 text-[9px] font-black bg-primary text-white rounded-full min-w-[14px] h-[14px] flex items-center justify-center px-0.5 border-2 border-canvas">
                  {item.badge > 99 ? '!' : item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* ─── 4. Collapse Toggle ─── */}
      <div className="p-3 border-t border-border-subtle bg-surface">
        <button
          onClick={toggleNav}
          className={`
            w-full flex items-center gap-2 rounded-md py-2 text-ink-muted hover:text-ink hover:bg-panel transition-all duration-150
            ${effectiveCollapsed ? 'justify-center px-0' : 'px-3'}
          `}
        >
          {effectiveCollapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          {!effectiveCollapsed && <span className="text-[11px] font-black uppercase tracking-widest">收起</span>}
        </button>
      </div>
    </nav>
  );
};

export default NavRail;
