import React from 'react';
import { FolderSync, Github, Languages, RefreshCw, Terminal } from 'lucide-react';

interface StatusBarProps {
  onInitProject: () => void;
  isTerminalOpen: boolean;
  onTerminalToggle: () => void;
}

const StatusBar: React.FC<StatusBarProps> = ({ onInitProject, isTerminalOpen, onTerminalToggle }) => {
  return (
    <div className="h-6 bg-secondary/80 border-t border-border flex items-center justify-between px-3 text-xs text-muted-foreground font-medium">
      <div className="flex items-center gap-4 min-w-0">
        <div className="flex items-center gap-1.5 hover:text-primary cursor-pointer transition-colors min-w-0" tabIndex={0} onKeyDown={(event) => event.key === 'Enter' && window.open('https://github.com/seatloom')}>
          <Github size={12} />
          <span className="truncate">~/Documents/GitHub/seatloom</span>
        </div>
        <div className="flex items-center gap-1.5">
          <RefreshCw size={12} className="text-status-active" />
          <span className="tracking-tight">已完成对账</span>
        </div>
        <button onClick={onInitProject} className="flex items-center gap-1 hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded" tabIndex={0}>
          <FolderSync size={12} />
          <span className="tracking-tight">初始化演示仓</span>
        </button>
        <button
          onClick={onTerminalToggle}
          className={`flex items-center gap-1 hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded ${isTerminalOpen ? 'text-primary font-semibold' : ''}`}
          tabIndex={0}
          title="切换终端面板（Ctrl+`）"
        >
          <Terminal size={12} />
          <span className="tracking-tight">终端面板</span>
        </button>
      </div>
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-background border border-border shadow-sm" title="当前演示内容固定为中文，避免评审时出现中英混杂">
          <Languages size={10} />
          <span>中文演示视图</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="text-primary font-bold">lyra</span>
          <div className="w-1.5 h-1.5 rounded-full bg-status-active" />
        </div>
      </div>
    </div>
  );
};

export default StatusBar;
