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
  Folder,
  Terminal,
} from 'lucide-react';
import { useAppStore } from '../stores/useAppStore';
import { useDataStore } from '../stores/useDataStore';
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
  | 'playbook'
  | 'sessions';

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
  const { navCollapsed, toggleNav } = useAppStore();
  const { activeProjectId, projectData } = useDataStore();
  const { isMobile, isCompact, isTablet } = useResponsive();

  // Force collapsed at compact/tablet breakpoints
  const effectiveCollapsed = navCollapsed || isCompact || isTablet;

  // Hide entirely on mobile (bottom tab bar takes over)
  if (isMobile) return null;

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const inboxCount = currentData?.inboxItems.length || 0;

  const navItems: NavItem[] = [
    { id: 'dashboard', icon: BarChart3, label: '监督概览' },
    { id: 'inbox', icon: Inbox, label: '收件箱', badge: inboxCount > 0 ? inboxCount : undefined },
    { id: 'workitems', icon: ClipboardList, label: '工作项' },
    { id: 'timeline', icon: Calendar, label: '时间线' },
    { id: 'seats', icon: Users, label: '席位' },
    { id: 'artifacts', icon: Package, label: '产出物' },
    { id: 'playbook', icon: BookOpen, label: 'Playbook' },
    { id: 'sessions', icon: Terminal, label: '会话' },
  ];

  return (
    <nav
      className="flex flex-col h-full border-r bg-canvas transition-all duration-layout overflow-hidden shrink-0"
      style={{ width: effectiveCollapsed ? 48 : 180 }}
    >
      {/* Brand logo */}
      <div className="p-2 border-b border-border-subtle flex items-center gap-2" style={{ minHeight: 48 }}>
        <SeatLoomLogo size={effectiveCollapsed ? 32 : 40} />
        {!effectiveCollapsed && (
          <span style={{ fontSize: 14, fontWeight: 700, color: 'var(--sl-text-primary)', letterSpacing: '-0.01em' }}>
            SeatLoom
          </span>
        )}
      </div>

      {/* Project switcher area */}
      <div className="p-2 border-b border-border-subtle">
        {effectiveCollapsed ? (
          <button
            onClick={() => onSelectProject('all-projects')}
            className="w-8 h-8 mx-auto flex items-center justify-center rounded-md bg-primary text-white"
            title="切换项目"
          >
            <Folder size={16} />
          </button>
        ) : (
          <ProjectSwitcher onSelect={onSelectProject} />
        )}
      </div>

      {/* Nav items */}
      <div className="flex-1 py-2 px-1.5 space-y-0.5 overflow-y-auto">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              title={effectiveCollapsed ? item.label : undefined}
              className={`
                w-full flex items-center gap-2.5 rounded-md transition-all duration-150 relative
                ${effectiveCollapsed ? 'justify-center px-0 py-2' : 'px-3 py-2'}
                ${isActive
                  ? 'bg-primary/10 text-primary font-semibold'
                  : 'text-ink-secondary hover:bg-panel hover:text-ink'
                }
              `}
            >
              {/* Active indicator bar */}
              {isActive && (
                <div className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 bg-primary rounded-r-full" />
              )}

              <item.icon size={18} className="shrink-0" />

              {!effectiveCollapsed && (
                <>
                  <span className="text-[13px] truncate">{item.label}</span>
                  {item.badge !== undefined && (
                    <span className="ml-auto text-[10px] font-semibold bg-primary text-white rounded-full min-w-[18px] h-[18px] flex items-center justify-center px-1">
                      {item.badge > 99 ? '99+' : item.badge}
                    </span>
                  )}
                </>
              )}

              {effectiveCollapsed && item.badge !== undefined && (
                <span className="absolute -top-0.5 -right-0.5 text-[9px] font-bold bg-primary text-white rounded-full min-w-[14px] h-[14px] flex items-center justify-center px-0.5">
                  {item.badge > 99 ? '!' : item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Collapse toggle */}
      <div className="p-2 border-t border-border-subtle">
        <button
          onClick={toggleNav}
          className={`
            w-full flex items-center gap-2 rounded-md py-2 text-ink-muted hover:text-ink hover:bg-panel transition-all duration-150
            ${effectiveCollapsed ? 'justify-center px-0' : 'px-3'}
          `}
          title={effectiveCollapsed ? '展开导航' : '收起导航'}
        >
          {effectiveCollapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          {!effectiveCollapsed && <span className="text-[12px]">收起</span>}
        </button>
      </div>
    </nav>
  );
};

export default NavRail;
