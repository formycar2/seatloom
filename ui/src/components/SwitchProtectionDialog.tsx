import React from 'react';
import { ShieldAlert, Zap } from 'lucide-react';

interface SwitchProtectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  onCancel: () => void;
  type?: 'session' | 'form';
}

const SwitchProtectionDialog: React.FC<SwitchProtectionDialogProps> = ({ isOpen, onClose, onConfirm, onCancel, type }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-md p-8 rounded-3xl border border-status-warning/30 shadow-2xl space-y-6 animate-in zoom-in-95 duration-300">
        <div className="flex flex-col items-center text-center space-y-4">
          <div className="w-16 h-16 bg-status-warning/10 text-status-warning rounded-2xl flex items-center justify-center border border-status-warning/20 shadow-lg shadow-status-warning/5">
            <ShieldAlert size={32} />
          </div>
          <h2 className="text-2xl font-black tracking-tight text-ink uppercase">高风险操作确认</h2>
          <p className="text-sm text-ink-soft leading-relaxed font-bold">
            正在尝试在会话活跃时切换运行时。这可能导致当前的未持久化上下文丢失。
          </p>
        </div>

        <div className="p-4 bg-status-warning/5 border border-status-warning/20 rounded-2xl space-y-2">
          <p className="text-[11px] text-status-warning font-black uppercase tracking-widest">警告事项</p>
          <ul className="text-[11px] text-ink-soft space-y-1 font-bold list-disc pl-4 opacity-80">
            <li>未写回的协调事件可能失效</li>
            <li>当前 Token 槽位将被强制释放</li>
            <li>新的运行时需要重新回灌启动包</li>
          </ul>
        </div>
        
        <div className="flex flex-col gap-2 pt-2">
          <button
            onClick={() => { onConfirm(); onClose(); }}
            className="w-full py-3 bg-status-warning text-white rounded-xl text-xs font-black uppercase tracking-widest hover:opacity-90 transition-all shadow-lg shadow-status-warning/20 flex items-center justify-center gap-2"
          >
            <Zap size={14} />
            强制执行切换 (FORCE SWITCH)
          </button>
          <button
            onClick={() => { onCancel(); onClose(); }}
            className="w-full py-3 bg-secondary text-ink-soft rounded-xl text-xs font-black uppercase tracking-widest hover:bg-accent transition-all border border-border/40"
          >
            返回检查 (BACK)
          </button>
        </div>
      </div>
    </div>
  );
};

export default SwitchProtectionDialog;
