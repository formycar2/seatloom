import React, { useState } from 'react';
import { Play } from 'lucide-react';

interface WrapLaunchDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const RUNTIMES = ['Codex', 'ClaudeCode', 'CursorCli', 'GeminiCli', 'OpenCode'];

const WrapLaunchDialog: React.FC<WrapLaunchDialogProps> = ({ isOpen, onClose }) => {
  const [runtime, setRuntime] = useState('Codex');

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-lg rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <Play size={20} />
            </div>
            <h2 className="text-xl font-black tracking-tight text-ink uppercase">包裹式启动 (WRAP LAUNCH)</h2>
          </div>
        </div>

        <div className="p-8 space-y-6">
          <div className="space-y-3">
            <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">选择目标运行时</label>
            <div className="grid grid-cols-2 gap-2">
              {RUNTIMES.map((r) => (
                <button
                  key={r}
                  onClick={() => setRuntime(r)}
                  className={`px-4 py-2.5 rounded-xl border text-[10px] font-black tracking-widest transition-all ${
                    runtime === r ? 'bg-primary text-white border-primary shadow-lg shadow-primary/20' : 'bg-card border-border/60 text-ink-soft hover:bg-accent'
                  }`}
                >
                  {r}
                </button>
              ))}
            </div>
          </div>

          <p className="text-[11px] text-ink-soft leading-relaxed bg-accent/30 p-4 rounded-xl border border-primary/10 italic">
            包裹式启动将创建一个被 SeatLoom 审计追踪的子进程。所有的文件变更和终端输出都将自动关联到当前 WorkItem 证据链。
          </p>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-ink-soft hover:text-ink transition-colors">取消</button>
          <button
            onClick={onClose}
            className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-primary text-white rounded-xl hover:bg-primary-hover transition-all shadow-lg shadow-primary/20"
          >
            启动并录制 (LAUNCH)
          </button>
        </div>
      </div>
    </div>
  );
};

export default WrapLaunchDialog;
