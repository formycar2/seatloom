import React, { useMemo, useState } from 'react';
import { Terminal, RefreshCw, FileText, CheckCircle } from 'lucide-react';
import { Session, Runtime } from '../types';
import { getRuntimeLabel } from '../utils/display';

interface SwitchRuntimeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  session: Session;
}

const runtimeOptions: Runtime[] = ['ClaudeCode', 'Codex', 'CursorCli', 'GeminiCli'];

const SwitchRuntimeDialog: React.FC<SwitchRuntimeDialogProps> = ({ isOpen, onClose, session }) => {
  const [targetRuntime, setTargetRuntime] = useState<Runtime>('GeminiCli');
  const [step, setStep] = useState(1);

  const launchPackPath = `.seatloom/launchpack-${session.id}.md`;

  const previewContent = useMemo(
    () => `# ${session.id} 运行时切换接力包

## 当前会话概览
- 原运行时：${typeof session.runtime === 'string' ? getRuntimeLabel(session.runtime) : session.runtime.Custom}
- 目标运行时：${typeof targetRuntime === 'string' ? getRuntimeLabel(targetRuntime) : targetRuntime.Custom}
- 工作目录：${session.workspace_path}
- 分支：${session.branch || '未记录分支'}
- 当前状态：${session.status}

## 需要继承的上下文
1. 保留当前工作项、交接单和最近事件链路，不要重新猜测产品意图。
2. 继续沿用已冻结的项目合同与阶段结论，避免把临时终端讨论当成新真值。
3. 若发现信息缺口，优先写回文件再继续推进。

## 本次切换目标
- 让新运行时在不丢上下文的前提下继续执行当前任务。
- 把关键证据、待处理事项和恢复点放进可追溯的接力包。
- 降低换模、换席位、换工具时的重新解释成本。

## 建议启动指令
${typeof targetRuntime === 'string' ? targetRuntime.toLowerCase() : 'custom-runtime'} --resume ${session.id}
`,
    [session, targetRuntime]
  );

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-card w-[640px] rounded-xl shadow-2xl border border-border overflow-hidden flex flex-col max-h-[80vh]">
        <div className="p-6 border-b border-border bg-secondary/30 space-y-2">
          <h2 className="text-lg font-bold flex items-center gap-2 text-primary">
            <RefreshCw size={20} />
            切换运行时（上下文再挂载）
          </h2>
          <p className="text-xs text-muted-foreground leading-relaxed">
            为当前会话生成接力包，把上下文从 {typeof session.runtime === 'string' ? getRuntimeLabel(session.runtime) : session.runtime.Custom}
            切换到新的工具链，而不是让下一个运行时从零理解整个项目。
          </p>
        </div>

        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {step === 1 ? (
            <>
              <div className="space-y-1.5">
                <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">目标运行时</label>
                <select
                  value={targetRuntime as string}
                  onChange={(e) => setTargetRuntime(e.target.value as Runtime)}
                  className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
                >
                  {runtimeOptions.map((option) => (
                    <option key={option as string} value={option as string}>
                      {getRuntimeLabel(option)}
                    </option>
                  ))}
                </select>
              </div>

              <div className="p-4 bg-primary/5 border border-primary/20 rounded-lg flex items-start gap-3">
                <FileText size={18} className="text-primary mt-0.5" />
                <div className="space-y-1">
                  <h4 className="text-sm font-bold text-primary">生成接力包</h4>
                  <p className="text-xs text-muted-foreground leading-relaxed">
                    SeatLoom 会把当前工作目录、分支、最近关键状态与恢复说明打包写入
                    <span className="font-mono text-foreground"> {launchPackPath}</span>，供新运行时接棒。
                  </p>
                </div>
              </div>
            </>
          ) : (
            <div className="space-y-3">
              <div className="flex items-center justify-between text-xs font-bold text-status-active flex-wrap gap-2">
                <span className="flex items-center gap-1.5"><CheckCircle size={14} /> 接力包已生成</span>
                <span className="font-mono bg-status-active/10 px-1.5 py-0.5 rounded">{launchPackPath}</span>
              </div>
              <textarea
                className="w-full h-72 bg-black text-gray-300 font-mono text-xs p-4 rounded-lg border border-border resize-none"
                readOnly
                value={previewContent}
              />
            </div>
          )}
        </div>

        <div className="p-4 border-t border-border bg-secondary flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm font-medium hover:bg-black/5 rounded-lg transition-colors">取消</button>
          {step === 1 ? (
            <button onClick={() => setStep(2)} className="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-primary/20">
              预览接力包
            </button>
          ) : (
            <button onClick={onClose} className="px-6 py-2 text-sm font-bold bg-status-active text-white rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-status-active/20 flex items-center gap-2">
              <Terminal size={14} /> 启动 {typeof targetRuntime === 'string' ? getRuntimeLabel(targetRuntime) : targetRuntime.Custom}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default SwitchRuntimeDialog;
