import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface SwitchProtectionDialogProps {
  isOpen: boolean;
  type: 'session' | 'form';
  onConfirm: () => void;
  onCancel: () => void;
}

const SwitchProtectionDialog: React.FC<SwitchProtectionDialogProps> = ({ isOpen, type, onConfirm, onCancel }) => {
  if (!isOpen) return null;

  const content = {
    session: {
      title: '仍有会话在后台运行',
      message:
        '切换项目不会停止这些运行中的会话，但它们会继续写入当前项目上下文。如果你现在离开，请确认自己不会因此错过终端输出、交接回执或待处理输入。',
      confirm: '继续运行并切换项目',
      cancel: '留在当前项目',
    },
    form: {
      title: '当前表单尚未保存',
      message:
        '你正在编辑的字段还没有写入系统。若现在切换项目，这些输入会被丢弃，后续也不会出现在工作项、交接或记忆日志中。',
      confirm: '放弃输入并切换',
      cancel: '返回继续编辑',
    },
  }[type];

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-card w-[440px] rounded-xl shadow-2xl border-2 border-primary/20 overflow-hidden animate-in fade-in zoom-in duration-200">
        <div className="p-6 space-y-4">
          <div className="flex items-center gap-3 text-primary">
            <div className="p-2 bg-primary/10 rounded-full">
              <AlertTriangle size={24} />
            </div>
            <h2 className="text-xl font-bold">{content.title}</h2>
          </div>

          <p className="text-sm text-muted-foreground leading-relaxed">
            {content.message}
          </p>
        </div>

        <div className="p-4 bg-secondary/50 border-t border-border flex flex-col gap-2">
          <button
            onClick={onConfirm}
            className="w-full py-2.5 bg-primary text-primary-foreground text-xs font-black uppercase tracking-widest rounded-lg hover:opacity-90 transition-all shadow-lg shadow-primary/20"
          >
            {content.confirm}
          </button>
          <button
            onClick={onCancel}
            className="w-full py-2.5 bg-background border border-border text-foreground text-xs font-black uppercase tracking-widest rounded-lg hover:bg-secondary transition-all"
          >
            {content.cancel}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SwitchProtectionDialog;
