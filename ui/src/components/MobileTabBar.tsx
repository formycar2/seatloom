import React from 'react';
import { BarChart3, Inbox, Activity, Package } from 'lucide-react';

export type MobileTab = 'overview' | 'actions' | 'activity' | 'artifacts';

interface MobileTabBarProps {
  activeTab: MobileTab;
  onTabChange: (tab: MobileTab) => void;
}

const tabs: { id: MobileTab; icon: React.ElementType; label: string }[] = [
  { id: 'overview', icon: BarChart3, label: '概览' },
  { id: 'actions', icon: Inbox, label: '行动' },
  { id: 'activity', icon: Activity, label: '活动' },
  { id: 'artifacts', icon: Package, label: '成果' },
];

const MobileTabBar: React.FC<MobileTabBarProps> = ({ activeTab, onTabChange }) => {
  return (
    <nav className="flex items-center justify-around bg-canvas border-t border-[var(--sl-border)] px-2 py-1 shrink-0">
      {tabs.map((tab) => {
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`flex flex-col items-center gap-0.5 px-3 py-1.5 rounded-lg transition-colors min-w-[56px]
              ${isActive ? 'text-primary' : 'text-[var(--sl-ink-muted)]'}
            `}
          >
            <tab.icon size={20} />
            <span className="text-[10px] font-medium">{tab.label}</span>
          </button>
        );
      })}
    </nav>
  );
};

export default MobileTabBar;
