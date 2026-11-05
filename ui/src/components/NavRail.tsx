import React, { useEffect, useState } from 'react';
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
  Radio,
  Send,
} from 'lucide-react';
import { useAppStore } from '../stores/useAppStore';
import { useDataStore } from '../stores/useDataStore';
import { useLiveSessionsStore } from '../stores/useLiveSessionsStore';
import { useResponsive } from '../hooks/useResponsive';
import ProjectSwitcher from './ProjectSwitcher';
import { SeatLoomLogo } from '../app-v2/components/Avatar';
import { ProvenanceDot } from './ProvenanceBanner';
import { provenanceForTab } from '../lib/provenance';
import { api } from '../lib/api';

export type NavTab =
  | 'live-seats'
  | 'dashboard'
  | 'inbox'
  | 'workitems'
  | 'timeline'
  | 'dispatches'
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
  const { activeProjectId, projectData, backendHydrated } = useDataStore();

  // How many acceptance cards are waiting for the operator, across ALL projects.
  //
  // This is deliberately NOT scoped to `activeProjectId`. The panel that lists
  // the cards does scope its reads that way, and measured against the live
  // database that hides them: every acceptance belongs to seatloom while the
  // operator's active project was project_delivery, which has none. Opening the
  // panel showed an empty "no pending cards" — worse than not showing him at all.
  //
  // A count is also the only part of this that works with the panel CLOSED, which
  // is its default state (and his persisted state). The cards were unreachable
  // from the product for two independent reasons; this closes both.
  const [pendingAcceptanceCount, setPendingAcceptanceCount] = useState(0);
  useEffect(() => {
    let cancelled = false;
    const read = async () => {
      try {
        // `listAcceptances` takes a projectId and the backend filters by it — so
        // one call scoped to the active project is exactly the bug this badge
        // exists to fix: measured, every acceptance belongs to seatloom while the
        // operator's active project had none, so the scoped read returned an
        // empty "no pending cards" while three waited elsewhere.
        //
        // Read EVERY project and take the union. That is the honest count of
        // cards waiting for him, and it is the only thing a badge can show
        // without lying. The per-card scoping bug in the panel itself is
        // reported, not patched here: the panel's read is Nimbus's fence.
        const projects = useDataStore.getState().projects;
        const ids = projects.map((p) => p.id);
        const perProject = await Promise.all(
          ids.map((id) => api.listAcceptances(id).catch(() => [])),
        );
        if (cancelled) return;
        const seen = new Set<string>();
        let pending = 0;
        for (const rows of perProject) {
          for (const r of rows) {
            if (seen.has(r.id)) continue;
            seen.add(r.id);
            if (r.status === 'pending' || r.status === 'awaiting_acceptance') pending += 1;
          }
        }
        setPendingAcceptanceCount(pending);
      } catch {
        // A failed read must not invent a badge. Zero is the honest answer to
        // "how many do I know about", and it is what the operator already had.
        if (!cancelled) setPendingAcceptanceCount(0);
      }
    };
    void read();
    const timer = setInterval(read, 15000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
    // Depends on `backendHydrated`, not on nothing. Measured: with an empty dep
    // array the effect ran before hydration and read an empty project list, so
    // the badge sat at 0 — which is indistinguishable from "nothing is waiting".
    // The poll alone would have caught it 15 seconds later, but a badge that is
    // wrong for the first quarter minute is a badge that lies during the exact
    // window the operator opens the app in.
  }, [backendHydrated]);
  // Mirrors the banner in App.tsx: the live-seats dot must not read as LIVE
  // while tmux discovery is empty or has failed (UM0 HOLD-3).
  const seatDiscovery = useLiveSessionsStore((s) => s.seatDiscovery);
  const { isMobile, isCompact, isTablet } = useResponsive();

  // Force collapsed at compact/tablet breakpoints
  const effectiveCollapsed = navCollapsed || isCompact || isTablet;

  // Hide entirely on mobile (bottom tab bar takes over)
  if (isMobile) return null;

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const inboxCount = currentData?.inboxItems.length || 0;

  const navItems: NavItem[] = [
    // Live mirror first: opening the app should land on verifiable truth.
    { id: 'live-seats', icon: Radio, label: '实时席位' },
    { id: 'dashboard', icon: BarChart3, label: '监督概览' },
    { id: 'inbox', icon: Inbox, label: '收件箱', badge: inboxCount > 0 ? inboxCount : undefined },
    { id: 'workitems', icon: ClipboardList, label: '工作项' },
    { id: 'timeline', icon: Calendar, label: '时间线' },
    // The badge belongs on 派单: that is the surface a card is about, and it is
    // where the operator already looks for work waiting on him.
    {
      id: 'dispatches',
      icon: Send,
      label: '派单',
      badge: pendingAcceptanceCount > 0 ? pendingAcceptanceCount : undefined,
    },
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
      <div
        className="p-2 border-b border-border-subtle flex items-center gap-2"
        style={{ minHeight: 48 }}
      >
        <SeatLoomLogo size={effectiveCollapsed ? 32 : 40} />
        {!effectiveCollapsed && (
          <span
            style={{
              fontSize: 14,
              fontWeight: 700,
              color: 'var(--sl-text-primary)',
              letterSpacing: '-0.01em',
            }}
          >
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
              // The tab id, on the element. The label is translated and the rail is
              // collapsed at several breakpoints, so neither is a stable handle for
              // "which tab is this". This is what lets a test walk the real nav and
              // check each entry declares its data backing — the mutation "add a
              // tab and forget the second place" has to fail somewhere, and without
              // a stable id it fails nowhere.
              data-tab-id={item.id}
              title={effectiveCollapsed ? item.label : undefined}
              className={`
                w-full flex items-center gap-2.5 rounded-md transition-all duration-150 relative
                ${effectiveCollapsed ? 'justify-center px-0 py-2' : 'px-3 py-2'}
                ${
                  isActive
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

              {effectiveCollapsed && (
                <span className="absolute bottom-0.5 right-0.5">
                  <ProvenanceDot
                    provenance={provenanceForTab(item.id, backendHydrated, seatDiscovery)}
                  />
                </span>
              )}

              {!effectiveCollapsed && (
                <>
                  <span className="text-[13px] truncate">{item.label}</span>
                  <ProvenanceDot
                    provenance={provenanceForTab(item.id, backendHydrated, seatDiscovery)}
                  />
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
