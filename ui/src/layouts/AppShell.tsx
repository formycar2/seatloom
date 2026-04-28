import React from 'react';
import Sidebar from './Sidebar';
import StatusBar from './StatusBar';
import TopNav from './TopNav';

interface AppShellProps {
  children: React.ReactNode;
  detailPane?: React.ReactNode;
  terminalPane?: React.ReactNode;
  isTerminalOpen: boolean;
  onTerminalToggle: () => void;
  activeTab: string;
  onTabChange: (tab: string) => void;
  onSelectProject: (id: string) => void;
  activeObjectId: string | null;
  onSelectObject: (type: string, data: any) => void;
  onInitProject: () => void;
}

const AppShell: React.FC<AppShellProps> = ({ 
  children, 
  detailPane,
  terminalPane,
  isTerminalOpen,
  onTerminalToggle, 
  activeTab, 
  onTabChange,
  onSelectProject,
  activeObjectId,
  onSelectObject,
  onInitProject
}) => {
  return (
    <div className="flex flex-col h-screen overflow-hidden bg-background text-foreground select-none">
      <TopNav 
        activeTab={activeTab} 
        onTabChange={onTabChange} 
        onSelectProject={onSelectProject}
      />

      <div className="flex flex-1 overflow-hidden">
        {activeTab !== 'all-projects' && (
          <Sidebar 
            activeObjectId={activeObjectId} 
            onSelectObject={onSelectObject} 
          />
        )}

        <main className="flex-1 flex flex-col min-w-[400px] border-r border-border bg-background relative">
          {children}
        </main>

        {detailPane && (
          <aside className="w-[380px] bg-card overflow-y-auto">
            {detailPane}
          </aside>
        )}
      </div>

      <StatusBar onInitProject={onInitProject} isTerminalOpen={isTerminalOpen} onTerminalToggle={onTerminalToggle} />

      {isTerminalOpen && terminalPane}
    </div>
  );
};

export default AppShell;
