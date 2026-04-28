import React from 'react';

interface InitDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInitialize: () => void;
}

const InitDialog: React.FC<InitDialogProps> = ({ isOpen, onClose, onInitialize }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-bg-elevated w-[520px] rounded-xl shadow-2xl border border-border overflow-hidden animate-in fade-in zoom-in duration-200">
        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <h2 className="text-xl font-bold">初始化 SeatLoom</h2>
            <p className="text-sm text-text-secondary leading-relaxed">
              这一步会为当前仓库建立可追溯的协作基线：事件流、会话记录、交接产物、恢复包和项目状态都将开始以文件方式落盘，而不是只存在于临时终端输出中。
            </p>
          </div>

          <div className="space-y-4 py-2">
            <div className="p-3 bg-bg-secondary rounded-lg border border-border">
              <div className="text-xs font-semibold text-muted-foreground mb-1">项目路径</div>
              <div className="text-sm monospace truncate">~/Documents/GitHub/seatloom</div>
            </div>

            <div className="flex items-center gap-3 text-xs text-text-secondary flex-wrap">
              <div className="flex items-center gap-1.5">
                <div className="w-1.5 h-1.5 rounded-full bg-status-active"></div>
                <span>已检测到 Git 仓库</span>
              </div>
              <span>·</span>
              <span>3 条本地分支</span>
              <span>·</span>
              <span>支持写回 `.seatloom/` 协调状态目录</span>
            </div>

            <div className="rounded-xl border border-border bg-secondary/30 p-4 text-sm text-text-secondary leading-relaxed space-y-2">
              <p>
                初始化后，仓库根目录会创建 <span className="monospace bg-bg-secondary px-1 rounded text-accent font-medium">.seatloom/</span>
                ，用于持久化事件、会话、交接、恢复包和状态快照。
              </p>
              <p>
                这不会改写你的业务代码，但会让 SeatLoom 能够在多席位协作、切换 runtime、恢复上下文和阶段验收时保持一致真值。
              </p>
            </div>
          </div>
        </div>

        <div className="p-4 bg-bg-secondary flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium hover:bg-bg-elevated rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            onClick={onInitialize}
            className="px-6 py-2 text-sm font-bold bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors shadow-lg shadow-accent/20"
          >
            开始初始化
          </button>
        </div>
      </div>
    </div>
  );
};

export default InitDialog;
