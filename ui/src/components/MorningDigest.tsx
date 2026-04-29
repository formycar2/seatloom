import React from 'react';
import { AlertTriangle, Sparkles, X } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';

const MorningDigest: React.FC = () => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const interruptedSessions = currentData?.sessions.filter((session) => session.status === 'Interrupted').length || 0;
  const blockedWorkItems = currentData?.workItems.filter((workItem) => workItem.status === 'Blocked').length || 0;
  const runningSessions = currentData?.sessions.filter((session) => session.status === 'Running' || session.status === 'InputRequired').length || 0;
  const reviewWorkItems = currentData?.workItems.filter((workItem) => workItem.status === 'InReview' || workItem.status === 'Verified').length || 0;
  const totalSessions = currentData?.sessions.length || 0;
  const totalHandoffs = currentData?.handoffs.length || 0;
  const highPriorityInbox = currentData?.inboxItems.filter((item) => item.priority === 'Critical').length || 0;

  const hasHighlights = interruptedSessions > 0 || blockedWorkItems > 0 || highPriorityInbox > 0;

  return (
    <div className="bg-card border border-primary/20 rounded-2xl p-6 shadow-[0_8px_30px_rgb(0,0,0,0.02)] relative overflow-hidden group animate-in slide-in-from-top duration-700">
      <div className="absolute top-0 left-0 w-1.5 h-full bg-primary/40" />
      <div className="flex items-start justify-between">
        <div className="flex gap-4">
          <div className="mt-1 p-2 bg-primary/10 rounded-lg text-primary shadow-sm shadow-primary/5">
            <Sparkles size={18} />
          </div>
          <div>
            <h3 className="text-base font-bold mb-1 text-text-primary">{t.inbox.morningDigest}</h3>
            <p className="text-xs text-text-muted mb-4 font-bold opacity-80">
              当前项目共记录 {totalSessions} 个会话、{currentData?.workItems.length || 0} 个工作项、{totalHandoffs} 份交接与 {currentData?.events.length || 0} 条活动事件。
            </p>

            <ul className="space-y-2">
              {!hasHighlights && (
                <li className="text-sm flex items-center gap-3 text-text-primary font-bold">
                  <div className="w-1.5 h-1.5 bg-status-active rounded-full shadow-[0_0_8px_rgba(46,139,87,0.4)]" />
                  <span>当前没有紧急阻塞，{runningSessions} 个会话持续推进中，评审面上共有 {reviewWorkItems} 项待继续收敛。</span>
                </li>
              )}
              {highPriorityInbox > 0 && (
                <li className="text-sm flex items-center gap-3 text-status-error font-bold">
                  <div className="w-1.5 h-1.5 bg-status-error rounded-full shadow-[0_0_8px_rgba(195,81,58,0.4)]" />
                  <span>收件箱内有 {highPriorityInbox} 条紧急事项，需要优先处理验收结论、交接回执或冻结决策。</span>
                </li>
              )}
              {interruptedSessions > 0 && (
                <li className="text-sm flex items-center gap-3 text-status-warning font-semibold">
                  <AlertTriangle size={14} />
                  <span>发现 {interruptedSessions} 个中断会话，建议检查恢复路径与上下文回灌是否完整。</span>
                </li>
              )}
              {blockedWorkItems > 0 && (
                <li className="text-sm flex items-center gap-3 text-status-error font-semibold">
                  <div className="w-1.5 h-1.5 bg-status-error rounded-full shadow-[0_0_8px_rgba(239,68,68,0.5)]" />
                  <span>当前仍有 {blockedWorkItems} 个工作项处于阻塞状态，需要核对依赖项和放行条件。</span>
                </li>
              )}
            </ul>
          </div>
        </div>
        <button className="text-muted-foreground hover:text-foreground opacity-0 group-hover:opacity-100 transition-opacity p-1" title="关闭摘要">
          <X size={16} />
        </button>
      </div>
    </div>
  );
};

export default MorningDigest;
