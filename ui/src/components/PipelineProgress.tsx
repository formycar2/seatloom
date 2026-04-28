import React from 'react';
import { Play, CheckCircle2, Circle, Loader2 } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';

const PipelineProgress: React.FC = () => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const eventCount = currentData?.events.length || 0;
  const inboxCount = currentData?.inboxItems.length || 0;
  const sessionCount = currentData?.sessions.length || 0;
  const workItemCount = currentData?.workItems.length || 0;

  const stages = [
    {
      id: 1,
      name: '项目对账',
      status: 'done',
      detail: `已收敛 ${eventCount} 条事件、${inboxCount} 项待处理与最新交接回执。`,
    },
    {
      id: 2,
      name: '上下文装配',
      status: 'running',
      detail: `正在汇总 ${sessionCount} 个会话、${workItemCount} 个工作项和目标席位所需最小上下文。`,
    },
    {
      id: 3,
      name: '执行分发',
      status: 'pending',
      detail: '下一步会把收敛后的任务包下发给目标席位，并限制在 need-to-know 范围内执行。',
    },
    {
      id: 4,
      name: '验收写回',
      status: 'pending',
      detail: '完成后需要把结论写回 MEMORY、日记忆与对应 acceptance/review 产物。',
    },
  ] as const;

  return (
    <div className="p-4 bg-bg-elevated border border-border rounded-lg space-y-4">
      <div className="flex items-center justify-between gap-3 flex-wrap">
        <div>
          <h3 className="text-sm font-bold flex items-center gap-2">
            <Play size={16} className="text-accent fill-current" />
            {t.pipeline.title}
          </h3>
          <p className="text-[11px] text-text-muted mt-1">
            当前进度基于真实项目数据生成，不再使用脱离上下文的轻量占位样例。
          </p>
        </div>
        <span className="text-[10px] monospace bg-bg-secondary px-1.5 py-0.5 rounded text-text-muted border border-border">RUN-20260428-01</span>
      </div>

      <div className="space-y-3">
        {stages.map((stage, index) => (
          <div key={stage.id} className="flex items-center gap-3">
            <div className="relative flex flex-col items-center">
              {stage.status === 'done' ? (
                <CheckCircle2 size={16} className="text-status-active z-10" />
              ) : stage.status === 'running' ? (
                <Loader2 size={16} className="text-accent animate-spin z-10" />
              ) : (
                <Circle size={16} className="text-text-muted z-10" />
              )}
              {index < stages.length - 1 && <div className="absolute top-4 w-px h-9 bg-border"></div>}
            </div>
            <div className="flex-1 space-y-1 pb-3">
              <div className={`text-xs ${stage.status === 'running' ? 'font-bold text-accent' : 'text-text-primary'}`}>
                {stage.name}
              </div>
              <div className="text-[11px] text-text-muted leading-relaxed">{stage.detail}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PipelineProgress;
