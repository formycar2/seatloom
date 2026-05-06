/**
 * TimelineSection.tsx
 * 时间线区块：渲染最近发生的事件（展开）和更早的事件（折叠/单行摘要）
 */

import React from 'react';
import { Clock } from 'lucide-react';
import { TimelineEntry } from '../types';

const eventColor = (type: string): string => {
  if (['WorkItemStatusChanged', 'ReconcileCompleted', 'CheckpointCreated'].includes(type)) return 'var(--sl-green)';
  if (['ArtifactCreated', 'HandoffSent', 'HandoffAccepted', 'HandoffReturned', 'HandoffCompleted'].includes(type)) return 'var(--sl-blue)';
  return 'var(--sl-amber)';
};

interface TimelineSectionProps {
  projectedEvents: any[];
  justNow: TimelineEntry[];
  showEarlier: boolean;
  earlierToday: string;
  yesterday: string;
  onToggleEarlier: () => void;
  onItemMouseEnter?: (e: React.MouseEvent, ev: any) => void;
  onItemMouseLeave?: (e: React.MouseEvent) => void;
}

export const TimelineSection: React.FC<TimelineSectionProps> = ({
  projectedEvents,
  justNow,
  showEarlier,
  earlierToday,
  yesterday,
  onToggleEarlier,
  onItemMouseEnter,
  onItemMouseLeave,
}) => (
  <div style={{ padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
    <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 12, textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: 6 }}>
      <Clock size={14} /> 活动日志 (ACTIVITY)
    </div>
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {projectedEvents.length > 0 ? (
        projectedEvents.map((ev) => (
          <div key={ev.eventId}
            onMouseEnter={evEnt => {
              evEnt.currentTarget.style.background = 'var(--sl-surface-hover)';
              onItemMouseEnter?.(evEnt, ev);
            }}
            onMouseLeave={evEnt => {
              evEnt.currentTarget.style.background = 'var(--sl-bg)';
              onItemMouseLeave?.(evEnt);
            }}
            style={{
              display: 'flex', alignItems: 'center', gap: 12, padding: '10px 12px',
              borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
              cursor: 'pointer', transition: 'background 150ms ease'
            }}
          >
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4, width: 44, flexShrink: 0 }}>
              <span style={{ color: 'var(--sl-text-tertiary)', fontFamily: 'monospace', fontSize: 11 }}>{ev.time}</span>
              <div style={{ width: 6, height: 6, borderRadius: '50%', background: eventColor(ev.eventType) }} />
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', fontWeight: 500 }}>{ev.headline}</div>
              <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', marginTop: 2, display: 'flex', alignItems: 'center', gap: 8 }}>
                <span>{ev.actor}</span>
                {ev.objectRefs.length > 0 && <span style={{ color: 'var(--sl-text-tertiary)' }}>{ev.objectRefs.length} refs</span>}
                {ev.evidenceRefs.length > 0 && <span style={{ color: 'var(--sl-text-tertiary)' }}>{ev.evidenceRefs.length} evidence</span>}
              </div>
            </div>
          </div>
        ))
      ) : (
        justNow.map((e, i) => (
          <div key={i}
            onMouseEnter={ev => {
              ev.currentTarget.style.background = 'var(--sl-surface-hover)';
              onItemMouseEnter?.(ev, e);
            }}
            onMouseLeave={ev => {
              ev.currentTarget.style.background = 'var(--sl-bg)';
              onItemMouseLeave?.(ev);
            }}
            style={{
              display: 'flex', alignItems: 'center', gap: 12, padding: '10px 12px',
              borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
              cursor: 'pointer', transition: 'background 150ms ease'
            }}
          >
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4, width: 44, flexShrink: 0 }}>
              <span style={{ color: 'var(--sl-text-tertiary)', fontFamily: 'monospace', fontSize: 11 }}>{e.time}</span>
              <div style={{ width: 6, height: 6, borderRadius: '50%', background: e.type === 'decision' ? 'var(--sl-green)' : e.type === 'delivery' ? 'var(--sl-blue)' : 'var(--sl-amber)' }} />
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', fontWeight: 500 }}>{e.text}</div>
              <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', marginTop: 2 }}>点击查看关联证据与历史快照</div>
            </div>
          </div>
        ))
      )}
    </div>
    <div style={{ marginTop: 12, paddingTop: 12, borderTop: '1px dashed var(--sl-border)' }}>
      <button
        onClick={onToggleEarlier}
        style={{
          display: 'flex', alignItems: 'center', gap: 8, width: '100%',
          padding: '8px 12px', border: 'none', background: 'var(--sl-bg)', borderRadius: 'var(--sl-radius-md)',
          cursor: 'pointer', fontSize: 12, color: 'var(--sl-text-secondary)', fontWeight: 500
        }}
      >
        <span style={{ transform: showEarlier ? 'rotate(90deg)' : 'none', transition: 'transform 150ms ease', display: 'inline-block', fontSize: 14 }}>▸</span>
        <div style={{ flex: 1, textAlign: 'left' }}>
          <span style={{ color: 'var(--sl-text-tertiary)', marginRight: 8 }}>今天早些:</span>
          {earlierToday}
        </div>
      </button>
      {showEarlier && (
        <div className="animate-in fade-in slide-in-from-top-2 duration-200" style={{ padding: '12px 12px 4px 34px', fontSize: 12, color: 'var(--sl-text-secondary)', display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ color: 'var(--sl-text-tertiary)' }}>昨天:</span>
            {yesterday}
          </div>
          <div style={{ color: 'var(--sl-brand)', cursor: 'pointer', fontWeight: 500, marginTop: 4 }}>查看完整账本历史记录 →</div>
        </div>
      )}
    </div>
  </div>
);
