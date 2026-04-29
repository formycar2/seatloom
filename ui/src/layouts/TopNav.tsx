import React from 'react';
import { Clock, Grid, Inbox, Shield, Smartphone, Target } from 'lucide-react';
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
    <div className="h-12 bg-card border-b border-border flex items-center px-4 justify-between shadow-[0_1px_3px_rgba(0,0,0,0.02)] z-30">
      <div className="flex items-center gap-4 flex-1 min-w-0">
        <div className="flex items-center gap-2 mr-4 group cursor-pointer" onClick={() => onSelectProject('all-projects')}>
          <div className="w-6 h-6 bg-primary rounded flex items-center justify-center text-primary-foreground shadow-md shadow-primary/10">
            <Shield size={14} />
          </div>
          <span className="text-sm font-black tracking-tight text-foreground">SeatLoom</span>
        </div>

        <div className="h-6 w-px bg-border/60 mx-2" />

        <ProjectSwitcher onSelect={onSelectProject} />

        <div className="h-6 w-px bg-border/60 mx-2 hidden md:block" />

        <div className="hidden md:flex items-center h-full">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`h-12 px-5 flex items-center gap-2 text-sm font-semibold transition-all relative ${
                activeTab === tab.id ? 'text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent/40'
              }`}
            >
              <tab.icon size={14} />
              <span>{tab.label}</span>
              {activeTab === tab.id && (
                <div className="absolute bottom-0 left-4 right-4 h-0.5 bg-primary rounded-t-full" />
              )}
            </button>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => onTabChange('mobile')}
          className={`p-2 rounded-lg transition-all ${
            activeTab === 'mobile' ? 'bg-primary text-primary-foreground shadow-md' : 'text-muted-foreground hover:bg-secondary'
          }`}
          title="移动伴侣 (Mobile Companion)"
        >
          <Smartphone size={16} />
        </button>
        <button
          onClick={() => onSelectProject('all-projects')}
          className={`p-2 rounded-lg transition-all ${
            activeTab === 'all-projects' ? 'bg-primary text-primary-foreground shadow-md' : 'text-muted-foreground hover:bg-secondary'
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
