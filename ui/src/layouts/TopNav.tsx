import React from 'react';
import { Clock, Grid, Inbox, Shield, Target } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import ProjectSwitcher from '../components/ProjectSwitcher';

interface TopNavProps {
  activeTab: string;
  onTabChange: (tab: any) => void;
  onSelectProject: (id: string) => void;
}

const TopNav: React.FC<TopNavProps> = ({ activeTab, onTabChange, onSelectProject }) => {
  const { t } = useLocaleStore();

  const tabs = [
    { id: 'inbox', icon: Inbox, label: t.sidebar.inbox },
    { id: 'timeline', icon: Clock, label: t.sidebar.timeline },
    { id: 'workitems', icon: Target, label: t.sidebar.workitems },
  ];

  return (
    <div className="h-12 bg-card border-b border-border flex items-center px-4 justify-between shadow-sm z-30">
      <div className="flex items-center gap-4 flex-1 min-w-0">
        <div className="flex items-center gap-2 mr-4 group cursor-pointer" onClick={() => onSelectProject('all-projects')}>
          <div className="w-6 h-6 bg-primary rounded flex items-center justify-center text-white shadow-lg shadow-primary/20 group-hover:scale-110 transition-transform">
            <Shield size={14} />
          </div>
          <span className="text-sm font-bold tracking-wide text-foreground">SeatLoom</span>
        </div>

        <div className="h-6 w-px bg-border mx-2" />

        <ProjectSwitcher onSelect={onSelectProject} />

        <div className="h-6 w-px bg-border mx-2 hidden md:block" />

        <div className="hidden md:flex items-center h-full">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`h-12 px-5 flex items-center gap-2 text-sm font-medium transition-all relative ${
                activeTab === tab.id ? 'text-primary' : 'text-muted-foreground hover:text-foreground'
              }`}
            >
              <tab.icon size={14} />
              <span>{tab.label}</span>
              {activeTab === tab.id && (
                <div className="absolute bottom-0 left-4 right-4 h-0.5 bg-primary rounded-t-full shadow-[0_-2px_10px_rgba(59,130,246,0.5)]" />
              )}
            </button>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-4">
        <button
          onClick={() => onSelectProject('all-projects')}
          className={`p-2 rounded-lg transition-all ${
            activeTab === 'all-projects' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-secondary'
          }`}
          title="项目总览"
        >
          <Grid size={16} />
        </button>
      </div>
    </div>
  );
};

export default TopNav;
