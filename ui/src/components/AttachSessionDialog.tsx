import React from 'react';
import { Link } from 'lucide-react';

interface AttachSessionDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

// 2026-10-20 (AEGIS-2026-10-20-ui-memory-and-desync-v1 item 3a):这里曾是
// `MOCK_PROCESSES` —— 三个写死的 PID/runtime/cwd,长得像「检测到的本地进程」,
// 用户能点、能选、还能点「接入并同步」。三个数都是编的,而那个接入按钮除了关掉
// 弹窗什么也不做。
//
// 为什么改成「暂无数据」而不是接真实列表:后端没有进程注册表这个东西。能列出的
// 是 tmux 会话(`cmd_list_tmux_sessions`),而「把任意本地 PID 绑定到席位」这条能力
// 在本产品里不存在 —— 所以这里既没有真实数据,也没有可调的命令。
//
// 不删这个组件:它标记了一个真实的需求缺口。删掉会让缺口从视野里消失,留着并把
// 「做不到什么」说清,比一个看起来能用的假列表诚实。
const ATTACH_CAPABILITY_MISSING =
  '后端没有可列出的本地进程注册表,因此这里没有真实进程可选。' +
  ' SeatLoom 能镜像的是 tmux 会话(见「实时席位」页签);把任意 PID 绑定到席位这条能力尚未实现。';

const AttachSessionDialog: React.FC<AttachSessionDialogProps> = ({ isOpen, onClose }) => {
  // `selectedPid` 已随假进程列表一起移除 —— 没有可选项,就没有选择态。
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-lg rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <Link size={20} />
            </div>
            <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">
              接入进程 (ATTACH PROCESS)
            </h2>
          </div>
        </div>

        <div className="p-8 space-y-6">
          <div className="space-y-3">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">
              检测到的本地进程
            </label>
            <div className="space-y-2 max-h-60 overflow-y-auto pr-2 custom-scrollbar">
              <div className="p-4 rounded-2xl border border-border/60 bg-background space-y-2">
                <p className="text-xs font-bold text-text-secondary leading-relaxed">
                  暂无数据 —— {ATTACH_CAPABILITY_MISSING}
                </p>
                <p className="text-[10px] font-bold text-text-secondary opacity-60 uppercase tracking-tighter">
                  此处不显示示例进程,也不用占位数据冒充
                </p>
              </div>
            </div>
          </div>

          <p className="text-[11px] text-text-secondary leading-relaxed bg-accent/30 p-4 rounded-xl border border-primary/10 italic">
            原设计:接入已有进程会将该 PID 绑定到当前席位,并从终端流恢复最近的协调语境。
            这条能力尚未实现,所以上面的"接入并同步"按钮同样不执行任何绑定 —— 它只会关掉这个弹窗。
          </p>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-text-secondary hover:text-text-primary transition-colors"
          >
            取消
          </button>
          <button
            onClick={onClose}
            disabled
            title="后端没有进程注册表,也没有对应的接入命令,所以这一按钮当前不可用"
            className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-primary text-white rounded-xl hover:bg-primary-hover transition-all shadow-lg shadow-primary/20 opacity-50 cursor-not-allowed"
          >
            接入并同步 (未实现)
          </button>
        </div>
      </div>
    </div>
  );
};

export default AttachSessionDialog;
