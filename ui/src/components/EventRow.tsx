import React from 'react';
import { CanonicalEvent } from '../types';
import { useDataStore } from '../stores/useDataStore';
import { formatTimeZh, getActorLabel, getEventTypeLabel } from '../utils/display';

interface EventRowProps {
  event: CanonicalEvent;
  onClick: () => void;
}

const EventRow: React.FC<EventRowProps> = ({ event, onClick }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const actorSeatId = typeof event.actor_ref === 'object' && 'Seat' in event.actor_ref ? event.actor_ref.Seat : null;
  const actorSeatName = actorSeatId
    ? currentData?.seats.find((seat) => seat.id === actorSeatId)?.name
    : undefined;

  const objectLabel = event.object_refs
    .map((ref) => {
      if ('Session' in ref) return ref.Session;
      if ('Artifact' in ref) return ref.Artifact;
      if ('WorkItem' in ref) return ref.WorkItem;
      if ('Handoff' in ref) return ref.Handoff;
      return '';
    })
    .filter(Boolean)
    .join(' / ');

  const summary = event.payload?.title || event.payload?.summary || '已记录一条状态变化。';

  const getTypeColor = (type: string) => {
    if (type.startsWith('Session')) return 'text-primary bg-primary/10 border-primary/20';
    if (type.startsWith('Handoff')) return 'text-status-active bg-status-active/10 border-status-active/20';
    if (type.startsWith('Artifact')) return 'text-violet-600 bg-violet-500/10 border-violet-500/20';
    if (type.startsWith('WorkItem')) return 'text-orange-500 bg-orange-500/10 border-orange-500/20';
    return 'text-muted-foreground bg-secondary border-border';
  };

  return (
    <div
      onClick={onClick}
      onKeyDown={(eventKey) => eventKey.key === 'Enter' && onClick()}
      tabIndex={0}
      className="sl-clickable px-6 py-4 flex items-start gap-6 border-b border-border last:border-0 focus:outline-none focus:ring-1 focus:ring-inset focus:ring-primary group"
    >
      <div className="w-14 text-xs text-muted-foreground mt-0.5 font-semibold text-right">{formatTimeZh(event.occurred_at)}</div>
      <div className="w-28 text-xs font-semibold truncate mt-0.5 text-foreground group-hover:text-primary transition-colors">{getActorLabel(event.actor_ref, actorSeatName)}</div>
      <div className="flex-1 min-w-0 flex items-start gap-4">
        <span className={`text-[10px] px-2 py-0.5 rounded border font-black tracking-widest whitespace-nowrap ${getTypeColor(event.event_type)}`}>
          {getEventTypeLabel(event.event_type)}
        </span>
        <div className="flex-1 min-w-0">
          <div className="text-[12px] text-foreground font-medium group-hover:underline cursor-pointer mb-1 leading-snug whitespace-pre-wrap">{summary}</div>
          <div className="text-[10px] text-primary font-mono opacity-80 break-all">{objectLabel || '无直接对象引用'}</div>
        </div>
      </div>
    </div>
  );
};

export default EventRow;
