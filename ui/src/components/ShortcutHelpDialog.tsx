import React from 'react';
import { HelpCircle, X } from 'lucide-react';

interface ShortcutHelpDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const SHORTCUTS = [
  { key: 'Esc', desc: '收起详情 / 关闭表单' },
  { key: 'Cmd/Ctrl + K', desc: '打开 Supervisor 指令栏 (Command Bar)' },
  { key: 'Cmd/Ctrl + `', desc: '显示/隐藏终端面板 (Operator Console)' },
  { key: '?', desc: '显示此快捷键帮助' },
  { key: 'F1', desc: '显示帮助 (同 ?)' },
];

const ShortcutHelpDialog: React.FC<ShortcutHelpDialogProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[120] flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-md rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-6 py-4 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <HelpCircle size={18} className="text-primary" />
            <h2 className="text-sm font-black tracking-tight text-ink uppercase">快捷键说明 (KEYBOARD HELP)</h2>
          </div>
          <button onClick={onClose} className="p-1 hover:bg-accent text-text-secondary rounded-lg transition-colors">
            <X size={18} />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div className="grid grid-cols-1 gap-2">
            {SHORTCUTS.map((s, idx) => (
              <div key={idx} className="flex items-center justify-between py-2 px-3 hover:bg-secondary/40 rounded-xl transition-all border border-transparent hover:border-border/40">
                <span className="text-xs font-bold text-ink-soft">{s.desc}</span>
                <span className="monospace text-[10px] font-black bg-accent text-primary px-2 py-0.5 rounded border border-primary/20 shadow-sm">{s.key}</span>
              </div>
            ))}
          </div>
          
          <div className="bg-primary/5 p-4 rounded-2xl border border-primary/10">
            <p className="text-[11px] text-ink-soft leading-relaxed font-bold opacity-80">
              SeatLoom 优先支持键盘导航。在大规模协调场景下，配合快捷键与 Command Bar 能显著提升操作效率。
            </p>
          </div>
        </div>

        <div className="bg-secondary/30 px-6 py-4 border-t border-border/40 flex justify-center">
          <p className="text-[10px] text-ink-faint uppercase tracking-widest font-black">Operator Efficiency Layer</p>
        </div>
      </div>
    </div>
  );
};

export default ShortcutHelpDialog;
