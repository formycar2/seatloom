import React from 'react';
import { ArrowRight, Shield } from 'lucide-react';

interface InitDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInitialize: () => void;
}

const InitDialog: React.FC<InitDialogProps> = ({ isOpen, onClose, onInitialize }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[110] flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-700">
      <div className="bg-card w-full max-w-md p-10 rounded-3xl border border-border/60 shadow-2xl text-center space-y-8 animate-in zoom-in-95 duration-500">
        <div className="space-y-4">
          <div className="mx-auto w-20 h-20 bg-primary/10 text-primary rounded-3xl flex items-center justify-center border border-primary/20 shadow-lg shadow-primary/5">
            <Shield size={40} />
          </div>
          <h2 className="text-3xl font-black tracking-tight text-text-primary uppercase">SeatLoom</h2>
          <p className="text-sm text-text-secondary leading-relaxed font-bold opacity-80">
            欢迎使用 SeatLoom 演示环境。我们需要在您的本地目录初始化演示账本以开始协作。
          </p>
        </div>
        
        <button
          onClick={onInitialize}
          className="w-full py-4 bg-primary text-white rounded-2xl text-xs font-black uppercase tracking-widest hover:opacity-90 transition-all shadow-xl shadow-primary/20 flex items-center justify-center gap-2 group"
        >
          初始化演示仓 (INITIALIZE)
          <ArrowRight size={16} className="group-hover:translate-x-1 transition-transform" />
        </button>

        <p className="text-[10px] text-text-muted uppercase tracking-widest font-black opacity-40">
          Ready for v0.5 protocol alignment
        </p>
      </div>
    </div>
  );
};

export default InitDialog;
