import React from 'react';
import { CheckCircle, RefreshCcw, Zap } from 'lucide-react';
import { Session } from '../types';

interface SwitchRuntimeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  session: Session;
}

const SwitchRuntimeDialog: React.FC<SwitchRuntimeDialogProps> = ({ isOpen, onClose, session }) => {
  if (!isOpen) return null;

  const launchPackPath = `.local/launch-packs/${session.id}.json`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-2xl rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <RefreshCcw size={20} />
            </div>
            <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">切换运行时 (SWITCH RUNTIME)</h2>
          </div>
        </div>

        <div className="p-8 space-y-6">
          <div className="space-y-4">
            <div className="flex items-start gap-3 p-4 bg-status-active/5 border border-status-active/20 rounded-2xl">
              <CheckCircle size={18} className="text-status-active shrink-0 mt-0.5" />
              <div className="space-y-1">
                <div className="text-sm font-bold text-text-primary">状态启动包已生成 (Continuity Pack Ready)</div>
                <div className="text-xs text-text-secondary leading-relaxed">
                  系统已自动为席位 <span className="font-bold text-primary">{session.seat_id}</span> 准备好离线启动包：
                  <span className="font-mono bg-status-active/10 text-status-active px-1.5 py-0.5 rounded ml-1">{launchPackPath}</span>
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between px-1">
                <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest">启动包源码预览 (PACK SOURCE)</label>
                <span className="text-[10px] font-bold text-text-muted">JSON · v0.5 PROTOCOL</span>
              </div>
              <textarea
                readOnly
                value={JSON.stringify(session.continuity_pack || {}, null, 2)}
                className="w-full h-72 bg-[var(--terminal-preview)] text-gray-300 font-mono text-[10px] p-4 rounded-2xl border border-white/5 resize-none shadow-inner"
              />
            </div>
          </div>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-between items-center">
          <button
            onClick={onClose}
            className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-text-secondary hover:text-text-primary transition-colors"
          >
            取消 (CANCEL)
          </button>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-primary text-white rounded-xl hover:bg-primary-hover transition-all shadow-lg shadow-primary/20 flex items-center gap-2"
            >
              <Zap size={14} />
              执行切换并同步 (EXECUTE)
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SwitchRuntimeDialog;
