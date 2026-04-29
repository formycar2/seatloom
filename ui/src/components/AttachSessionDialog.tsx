import React, { useState } from 'react';
import { Link } from 'lucide-react';

interface AttachSessionDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const MOCK_PROCESSES = [
  { pid: 14210, runtime: 'Codex', cwd: '~/Documents/GitHub/seatloom' },
  { pid: 28442, runtime: 'ClaudeCode', cwd: '~/Documents/GitHub/seatloom' },
  { pid: 9912, runtime: 'CursorCli', cwd: '~/Documents/GitHub/deepspeed' },
];

const AttachSessionDialog: React.FC<AttachSessionDialogProps> = ({ isOpen, onClose }) => {
  const [selectedPid, setSelectedPid] = useState<number | null>(null);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-lg rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <Link size={20} />
            </div>
            <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">接入进程 (ATTACH PROCESS)</h2>
          </div>
        </div>

        <div className="p-8 space-y-6">
          <div className="space-y-3">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">检测到的本地进程</label>
            <div className="space-y-2 max-h-60 overflow-y-auto pr-2 custom-scrollbar">
              {MOCK_PROCESSES.map((process) => (
                <div
                  key={process.pid}
                  onClick={() => setSelectedPid(process.pid)}
                  className={`p-4 rounded-2xl border transition-all cursor-pointer flex items-center justify-between group ${
                    selectedPid === process.pid ? 'border-primary bg-accent shadow-sm' : 'border-border/60 bg-background hover:bg-secondary'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full bg-status-active ${selectedPid === process.pid ? 'animate-pulse' : 'opacity-40'}`} />
                    <div className="space-y-0.5">
                      <div className="text-sm font-bold text-text-primary monospace">PID: {process.pid}</div>
                      <div className="text-[10px] text-text-secondary font-bold opacity-60 uppercase tracking-tighter">{process.runtime} · {process.cwd}</div>
                    </div>
                  </div>
                  {selectedPid === process.pid && <div className="text-primary font-black text-[10px]">SELECTED</div>}
                </div>
              ))}
            </div>
          </div>

          <p className="text-[11px] text-text-secondary leading-relaxed bg-accent/30 p-4 rounded-xl border border-primary/10 italic">
            接入已有进程会将该 PID 绑定到当前席位。系统将尝试从终端流中恢复最近的协调语境。
          </p>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-text-secondary hover:text-text-primary transition-colors">取消</button>
          <button
            onClick={onClose}
            disabled={!selectedPid}
            className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-primary text-white rounded-xl hover:bg-primary-hover transition-all shadow-lg shadow-primary/20 disabled:opacity-50 disabled:grayscale disabled:shadow-none"
          >
            接入并同步 (ATTACH)
          </button>
        </div>
      </div>
    </div>
  );
};

export default AttachSessionDialog;
