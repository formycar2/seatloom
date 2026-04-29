import React from 'react';
import { FolderSync, Github, Languages, RefreshCw, Terminal, HelpCircle } from 'lucide-react';

interface StatusBarProps {
  onInitProject: () => void;
  isTerminalOpen: boolean;
  onTerminalToggle: () => void;
  onShowHelp: () => void;
}

const StatusBar: React.FC<StatusBarProps> = ({ onInitProject, isTerminalOpen, onTerminalToggle, onShowHelp }) => {
  return (
    <div className="h-6 bg-background border-t border-border flex items-center justify-between px-3 text-[10px] text-muted-foreground font-bold uppercase tracking-wider shadow-[0_-1px_3px_rgba(0,0,0,0.01)]">
      <div className="flex items-center gap-4 min-w-0">
        <div className="flex items-center gap-1.5 hover:text-primary cursor-pointer transition-colors min-w-0" tabIndex={0} onKeyDown={(event) => event.key === 'Enter' && window.open('https://github.com/seatloom')}>
          <Github size={10} />
          <span className="truncate opacity-80">~/Documents/GitHub/seatloom</span>
        </div>
        <div className="flex items-center gap-1.5">
          <RefreshCw size={10} className="text-status-active" />
          <span className="tracking-tight opacity-80">对账完成</span>
        </div>
        <button onClick={onInitProject} className="flex items-center gap-1 hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded" tabIndex={0}>
          <FolderSync size={10} />
          <span className="tracking-tight opacity-80">初始化演示仓</span>
        </button>
        <button
          onClick={onTerminalToggle}
          className={`flex items-center gap-1 hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded ${isTerminalOpen ? 'text-primary' : ''}`}
          tabIndex={0}
          title="切换终端面板（Ctrl+`）"
        >
          <Terminal size={10} />
          <span className="tracking-tight opacity-80">终端面板</span>
        </button>
      </div>
      <div className="flex items-center gap-4">
        <button 
          onClick={onShowHelp}
          className="flex items-center gap-1 hover:text-primary transition-colors opacity-60 hover:opacity-100"
          title="显示快捷键帮助 (? / F1)"
        >
          <HelpCircle size={10} />
          <span className="tracking-tight">帮助 ? / F1</span>
        </button>
        <div className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-card border border-border shadow-sm text-[9px]" title="当前演示内容固定为中文，避免评审时出现中英混杂">
          <Languages size={10} />
          <span className="opacity-80">中文模式</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="text-primary font-black">LYRA</span>
          <div className="w-1.5 h-1.5 rounded-full bg-status-active shadow-[0_0_5px_rgba(46,139,87,0.4)]" />
        </div>
      </div>
    </div>
  );
};

export default StatusBar;
