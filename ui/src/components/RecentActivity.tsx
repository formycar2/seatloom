import React from 'react';
import { useDataStore } from '../stores/useDataStore';

const RecentActivity: React.FC = () => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentData) return null;

  const recentEvents = currentData.events.slice(0, 8);

  return (
    <div className="sl-card overflow-hidden">
      <div className="px-4 py-3 border-b border-[var(--sl-border-subtle)]">
        <h3 className="text-sm font-semibold text-[var(--sl-ink)]">最近活动</h3>
      </div>

      {recentEvents.length === 0 ? (
        <div className="px-4 py-6 text-center text-sm text-[var(--sl-ink-secondary)]">
          暂无活动记录
        </div>
      ) : (
        <div className="divide-y divide-[var(--sl-border-subtle)]">
          {recentEvents.map((event) => {
            const time = new Date(event.occurred_at);
            const timeStr = `${time.getHours().toString().padStart(2, '0')}:${time.getMinutes().toString().padStart(2, '0')}`;
            const summary = event.payload?.summary || `${event.event_type} — ${event.actor_ref}`;

            return (
              <div key={event.event_id} className="flex items-start gap-3 px-4 py-2.5 hover:bg-[var(--sl-panel)] transition-colors">
                <span className="text-xs text-[var(--sl-ink-muted)] font-mono shrink-0 pt-0.5 w-10">
                  {timeStr}
                </span>
                <span className="text-sm text-[var(--sl-ink)] leading-5 line-clamp-2">
                  {summary}
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default RecentActivity;
