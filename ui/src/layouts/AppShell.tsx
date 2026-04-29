import React from 'react';
import NavRail from '../components/NavRail';
import MobileTabBar from '../components/MobileTabBar';
import StatusBar from './StatusBar';
import { useResponsive } from '../hooks/useResponsive';
import type { NavTab } from '../components/NavRail';
import type { MobileTab } from '../components/MobileTabBar';

// Maps mobile tabs to desktop nav tabs
const mobileToDesktop: Record<MobileTab, NavTab> = {
  overview: 'dashboard',
  actions: 'inbox',
  activity: 'timeline',
  artifacts: 'artifacts',
};

const desktopToMobile: Partial<Record<NavTab, MobileTab>> = {
  dashboard: 'overview',
  inbox: 'actions',
  timeline: 'activity',
  artifacts: 'artifacts',
};

interface AppShellProps {
  children: React.ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
  onSelectProject: (id: string) => void;
  onInitProject: () => void;
  onShowHelp: () => void;
  isTerminalOpen: boolean;
  onTerminalToggle: () => void;
}

const AppShell: React.FC<AppShellProps> = ({
  children,
  activeTab,
  onTabChange,
  onSelectProject,
  onInitProject,
  onShowHelp,
  isTerminalOpen,
  onTerminalToggle,
}) => {
  const { isMobile } = useResponsive();

  const activeMobileTab: MobileTab = desktopToMobile[activeTab as NavTab] || 'overview';

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-panel select-none">
      <div className="flex flex-1 overflow-hidden">
        {/* NavRail (hidden on mobile via component logic) */}
        <NavRail
          activeTab={activeTab as NavTab}
          onTabChange={(tab) => onTabChange(tab)}
          onSelectProject={onSelectProject}
        />

        {/* Main content area */}
        <main className="flex-1 flex flex-col min-w-0 overflow-hidden bg-panel">
          {children}
        </main>
      </div>

      {/* Mobile bottom tab bar */}
      {isMobile && (
        <MobileTabBar
          activeTab={activeMobileTab}
          onTabChange={(mTab) => onTabChange(mobileToDesktop[mTab])}
        />
      )}

      {/* Status bar (hidden on mobile) */}
      {!isMobile && (
        <StatusBar
          onInitProject={onInitProject}
          isTerminalOpen={isTerminalOpen}
          onTerminalToggle={onTerminalToggle}
          onShowHelp={onShowHelp}
        />
      )}
    </div>
  );
};

export default AppShell;
