import React, { useMemo, useState } from 'react';
import { ChevronDown, Filter, Search } from 'lucide-react';
import EventRow from '../components/EventRow';
import { useDataStore } from '../stores/useDataStore';
import { CanonicalEvent } from '../types';
import { getEventCategoryLabel, getTimeRangeLabel } from '../utils/display';

interface TimelineFilters {
  seatId: string | null;
  workItemId: string | null;
  eventType: string | null;
  timeRange: string;
}

interface TimelineViewProps {
  onSelectObject: (type: string, data: any) => void;
  filters: TimelineFilters;
  onFiltersChange: (f: TimelineFilters) => void;
}

const EVENT_TYPES = [null, 'session', 'handoff', 'artifact', 'pipeline'];
const TIME_RANGES = ['1h', '6h', '24h', '7d'];
const RANGE_MAP: Record<string, number> = { '1h': 3600000, '6h': 21600000, '24h': 86400000, '7d': 604800000 };

const TimelineView: React.FC<TimelineViewProps> = ({ onSelectObject, filters, onFiltersChange }) => {
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const events: CanonicalEvent[] = currentData?.events || [];

  const filteredEvents = useMemo(() => {
    let filtered = [...events];

    if (filters.seatId) {
      filtered = filtered.filter((event) => {
        const actorSeat = typeof event.actor_ref === 'object' && 'Seat' in event.actor_ref ? event.actor_ref.Seat : null;
        const hasMatchingObject = event.object_refs.some((ref) => {
          if ('Session' in ref && currentData?.sessions.find((session) => session.id === ref.Session && session.seat_id === filters.seatId)) return true;
          if ('WorkItem' in ref && currentData?.workItems.find((workItem) => workItem.id === ref.WorkItem && workItem.owner_seat_id === filters.seatId)) return true;
          return false;
        });
        return actorSeat === filters.seatId || hasMatchingObject;
      });
    }

    if (filters.workItemId) {
      filtered = filtered.filter((event) => event.object_refs.some((ref) => 'WorkItem' in ref && ref.WorkItem === filters.workItemId));
    }

    if (filters.eventType) {
      filtered = filtered.filter((event) => {
        if (filters.eventType === 'session') return event.event_type.startsWith('Session');
        if (filters.eventType === 'handoff') return event.event_type.startsWith('Handoff');
        if (filters.eventType === 'artifact') return event.event_type.startsWith('Artifact');
        if (filters.eventType === 'pipeline') return event.event_type.startsWith('Pipeline');
        return true;
      });
    }

    if (filters.timeRange && RANGE_MAP[filters.timeRange]) {
      const since = Date.now() - RANGE_MAP[filters.timeRange];
      filtered = filtered.filter((event) => new Date(event.occurred_at).getTime() >= since);
    }

    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(
        (event) =>
          event.event_id.toLowerCase().includes(term) ||
          event.event_type.toLowerCase().includes(term) ||
          (event.payload?.title || '').toLowerCase().includes(term) ||
          (event.payload?.summary || '').toLowerCase().includes(term),
      );
    }

    return filtered.sort((left, right) => new Date(right.occurred_at).getTime() - new Date(left.occurred_at).getTime());
  }, [events, filters, searchTerm, currentData]);

  const clearFilters = () => {
    onFiltersChange({ seatId: null, workItemId: null, eventType: null, timeRange: '24h' });
    setSearchTerm('');
  };

  const cycle = <T,>(items: T[], current: T): T => items[(items.indexOf(current) + 1) % items.length];
  const hasActiveFilters = filters.seatId || filters.workItemId || filters.eventType;
  const activeSeatName = filters.seatId ? currentData?.seats.find((seat) => seat.id === filters.seatId)?.name : null;
  const activeWorkItemTitle = filters.workItemId ? currentData?.workItems.find((workItem) => workItem.id === filters.workItemId)?.title : null;

  return (
    <div className="flex-1 flex flex-col overflow-hidden bg-background animate-in fade-in duration-500">
      <div className="px-6 py-3 border-b border-border bg-card/50 flex items-center justify-between flex-wrap gap-2">
        <div>
          <h2 className="text-base font-bold text-foreground">活动时间线</h2>
          <p className="text-xs text-muted-foreground mt-1">按席位、工作项、事件类型和时间范围回看今天的协作轨迹。</p>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          <button
            onClick={() => onFiltersChange({ ...filters, timeRange: cycle(TIME_RANGES, filters.timeRange) })}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium hover:bg-secondary rounded border border-transparent hover:border-border transition-all"
          >
            <span className="text-muted-foreground">时间范围：</span>
            <span className="font-semibold">{getTimeRangeLabel(filters.timeRange)}</span>
            <ChevronDown size={12} className="text-muted-foreground" />
          </button>
          <button
            onClick={() => onFiltersChange({ ...filters, eventType: cycle(EVENT_TYPES, filters.eventType || null) || null })}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium hover:bg-secondary rounded border border-transparent hover:border-border transition-all"
          >
            <span className="text-muted-foreground">事件类型：</span>
            <span className="font-semibold">{getEventCategoryLabel(filters.eventType || null)}</span>
            <ChevronDown size={12} className="text-muted-foreground" />
          </button>
          {hasActiveFilters && (
            <button onClick={clearFilters} className="text-xs font-semibold text-primary hover:underline ml-1">
              清空筛选
            </button>
          )}
        </div>
      </div>

      <div className="px-6 py-2 border-b border-border bg-secondary/30 flex items-center gap-4 flex-wrap">
        <div className="relative flex-1 min-w-[240px] max-w-md">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="按事件 ID、摘要或标题搜索活动记录"
            value={searchTerm}
            onChange={(event) => setSearchTerm(event.target.value)}
            className="w-full bg-background border border-border rounded-lg pl-9 pr-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary shadow-sm"
          />
        </div>
        <div className="flex items-center gap-2 flex-wrap text-[11px] text-muted-foreground">
          {activeSeatName && <span className="px-2 py-1 rounded-full bg-background border border-border">席位：{activeSeatName}</span>}
          {activeWorkItemTitle && <span className="px-2 py-1 rounded-full bg-background border border-border">工作项：{activeWorkItemTitle}</span>}
          <span className="px-2 py-1 rounded-full bg-background border border-border">结果：{filteredEvents.length} 条</span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredEvents.length > 0 ? (
          <div className="divide-y divide-border">
            {filteredEvents.map((event) => (
              <EventRow
                key={event.event_id}
                event={event}
                onClick={() => {
                  const ref = event.object_refs[0];
                  if (!ref) return;
                  if ('Session' in ref) onSelectObject('Session', ref.Session);
                  else if ('WorkItem' in ref) onSelectObject('WorkItem', ref.WorkItem);
                  else if ('Handoff' in ref) onSelectObject('Handoff', ref.Handoff);
                }}
              />
            ))}
          </div>
        ) : (
          <div className="p-12 text-center text-muted-foreground flex flex-col items-center">
            <Filter size={32} className="mb-4 opacity-40" />
            <p className="text-sm font-medium">当前筛选条件下没有匹配的活动记录</p>
            <button onClick={clearFilters} className="text-xs text-primary hover:underline mt-2">
              重置筛选条件
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default TimelineView;
