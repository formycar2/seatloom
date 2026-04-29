import React, { useMemo } from 'react';
import { AlertCircle, CheckCircle2, Clock, ArrowRight } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { InboxItem } from '../types';

const ACTION_TYPES = new Set([
  '待处理交接',
  '需要验收',
  '需确认范围',
  '验收待决策',
  '输入请求',
  '会话恢复',
  '待补产物',
]);

const priorityColors: Record<string, string> = {
  Critical: 'bg-[var(--sl-error)] text-white',
  Normal: 'bg-primary text-white',
  Low: 'bg-[var(--sl-done)] text-white',
};

const typeIcons: Record<string, { color: string; label: string }> = {
  '待处理交接': { color: 'text-primary', label: 'Handoff' },
  '需要验收': { color: 'text-[var(--sl-warning)]', label: 'Gate 审批' },
  '需确认范围': { color: 'text-[var(--sl-warning)]', label: '范围确认' },
  '验收待决策': { color: 'text-[var(--sl-error)]', label: 'Gate 审批' },
  '输入请求': { color: 'text-[var(--sl-error)]', label: 'Prompt 阻塞' },
  '会话恢复': { color: 'text-[var(--sl-review)]', label: '会话恢复' },
  '待补产物': { color: 'text-[var(--sl-drift)]', label: '待补产物' },
};

interface ActionQueueProps {
  onSelectItem: (item: InboxItem) => void;
  onViewAll?: () => void;
  maxItems?: number;
}

const ActionQueue: React.FC<ActionQueueProps> = ({ onSelectItem, onViewAll, maxItems = 5 }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const actionItems = useMemo(() => {
    if (!currentData) return [];
    return currentData.inboxItems
      .filter((item) => ACTION_TYPES.has(item.type))
      .sort((a, b) => {
        const pOrder: Record<string, number> = { Critical: 0, High: 1, Medium: 2, Low: 3 };
        return (pOrder[a.priority] ?? 3) - (pOrder[b.priority] ?? 3);
      })
      .slice(0, maxItems);
  }, [currentData, maxItems]);

  if (!currentData) return null;

  return (
    <div className="sl-card overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--sl-border-subtle)]">
        <h3 className="text-sm font-semibold text-[var(--sl-ink)]">需要你行动</h3>
        {onViewAll && (
          <button
            onClick={onViewAll}
            className="text-xs text-primary hover:text-[var(--sl-primary-hover)] font-medium flex items-center gap-1 transition-colors"
          >
            全部 <ArrowRight size={12} />
          </button>
        )}
      </div>

      {actionItems.length === 0 ? (
        <div className="px-4 py-8 text-center">
          <CheckCircle2 size={24} className="mx-auto mb-2 text-[var(--sl-success)] opacity-60" />
          <p className="text-sm text-[var(--sl-ink-secondary)]">当前没有需要你处理的事项</p>
        </div>
      ) : (
        <div className="divide-y divide-[var(--sl-border-subtle)]">
          {actionItems.map((item) => {
            const typeInfo = typeIcons[item.type] || { color: 'text-[var(--sl-ink-secondary)]', label: item.type };
            const createdAt = new Date(item.timestamp);
            const waitMinutes = Math.round((Date.now() - createdAt.getTime()) / 60000);
            const waitLabel = waitMinutes > 60 ? `${Math.round(waitMinutes / 60)}h` : `${waitMinutes}分`;

            return (
              <button
                key={item.id}
                onClick={() => onSelectItem(item)}
                className="w-full flex items-center gap-3 px-4 py-3 hover:bg-[var(--sl-panel)] transition-colors text-left group"
              >
                {/* Priority indicator */}
                <div className="w-1 h-8 rounded-full shrink-0" style={{
                  backgroundColor: item.priority === 'Critical' ? 'var(--sl-error)'
                    : item.priority === 'Normal' ? 'var(--sl-primary)'
                    : 'var(--sl-done)'
                }} />

                {/* Type */}
                <span className={`text-xs font-semibold shrink-0 ${typeInfo.color}`}>
                  {typeInfo.label}
                </span>

                {/* Summary */}
                <span className="text-sm text-[var(--sl-ink)] truncate flex-1 group-hover:text-primary transition-colors">
                  {item.summary}
                </span>

                {/* Actor */}
                <span className="text-xs text-[var(--sl-ink-muted)] shrink-0">
                  {item.actor}
                </span>

                {/* Wait time */}
                <span className="text-xs text-[var(--sl-ink-muted)] shrink-0 flex items-center gap-1">
                  <Clock size={10} />
                  {waitLabel}
                </span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default ActionQueue;
