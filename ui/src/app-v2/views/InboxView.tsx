/**
 * InboxView.tsx
 * 待办决策队列视图：展示按优先级排序的 InboxItem 列表，支持归档操作
 * 接收来自 ProjectDashboard 的数据和回调，不直接访问 store
 */

import React from 'react';
import { InboxItem, Artifact } from '../../types';

const PRIORITY_CONFIG: Record<InboxItem['priority'], { label: string; color: string; bg: string }> = {
  Critical: { label: '紧急', color: 'var(--sl-red)', bg: 'var(--sl-red-subtle)' },
  Normal:   { label: '普通', color: 'var(--sl-amber)', bg: 'var(--sl-amber-subtle)' },
  Low:      { label: '低',   color: 'var(--sl-text-tertiary)', bg: 'var(--sl-surface-hover)' },
};

const PRIORITY_ORDER: Record<InboxItem['priority'], number> = { Critical: 0, Normal: 1, Low: 2 };

export const InboxView: React.FC<{
  items: InboxItem[];
  artifacts: Artifact[];
  onDismiss: (id: string) => void;
}> = ({ items, artifacts, onDismiss }) => {
  const sorted = [...items].sort((a, b) => PRIORITY_ORDER[a.priority] - PRIORITY_ORDER[b.priority]);

  if (sorted.length === 0) {
    return (
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 12, padding: 48 }}>
        <div style={{ fontSize: 36, opacity: 0.15 }}>📭</div>
        <div style={{ fontSize: 14, color: 'var(--sl-text-tertiary)' }}>待办队列已清空</div>
        <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)', opacity: 0.6 }}>所有 InboxItem 已处理或归档</div>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {sorted.map(item => {
        const cfg = PRIORITY_CONFIG[item.priority];
        const linkedArts = item.linked_artifact_ids
          ? artifacts.filter(a => item.linked_artifact_ids!.includes(a.id))
          : [];
        const isCritical = item.priority === 'Critical';

        return (
          <div
            key={item.id}
            style={{
              padding: '14px 16px',
              borderRadius: 'var(--sl-radius-md)',
              background: isCritical ? `var(--sl-red-subtle)` : 'var(--sl-surface)',
              border: `1px solid ${isCritical ? 'var(--sl-red)40' : 'var(--sl-border-light)'}`,
            }}
          >
            {/* Top row: badges + timestamp */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
              <span style={{
                fontSize: 10, fontWeight: 700, padding: '2px 8px',
                borderRadius: 'var(--sl-radius-full)', background: cfg.bg, color: cfg.color,
              }}>{cfg.label}</span>
              <span style={{
                fontSize: 11, padding: '2px 7px',
                borderRadius: 'var(--sl-radius-full)', background: 'var(--sl-surface-hover)', color: 'var(--sl-text-secondary)',
              }}>{item.type}</span>
              <span style={{ fontSize: 10, fontFamily: 'monospace', fontWeight: 700, color: 'var(--sl-brand)' }}>{item.object_ref}</span>
              <div style={{ flex: 1 }} />
              <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', flexShrink: 0 }}>{item.timestamp}</span>
            </div>

            {/* Summary */}
            <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', lineHeight: 1.65, marginBottom: 10 }}>
              {item.summary}
            </div>

            {/* Bottom row: actor + linked artifacts + archive button */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
              <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                执行者: <span style={{ color: 'var(--sl-text-secondary)', fontWeight: 500 }}>{item.actor}</span>
              </span>
              {linkedArts.map(a => (
                <span
                  key={a.id}
                  title={a.title}
                  style={{
                    fontSize: 10, padding: '2px 6px', borderRadius: 3,
                    background: 'var(--sl-brand-subtle)', color: 'var(--sl-brand)', fontWeight: 600,
                  }}
                >
                  {a.template}·{a.subtype}
                </span>
              ))}
              <div style={{ flex: 1 }} />
              <button
                onClick={() => onDismiss(item.id)}
                title="将此待办标记为已归档"
                style={{
                  padding: '4px 12px', fontSize: 11, fontWeight: 600,
                  color: 'var(--sl-text-tertiary)', background: 'var(--sl-surface-hover)',
                  border: '1px solid var(--sl-border-light)', borderRadius: 'var(--sl-radius-sm)', cursor: 'pointer',
                  transition: 'all 120ms ease',
                }}
                onMouseEnter={e => {
                  e.currentTarget.style.color = 'var(--sl-red)';
                  e.currentTarget.style.borderColor = 'var(--sl-red)40';
                  e.currentTarget.style.background = 'var(--sl-red-subtle)';
                }}
                onMouseLeave={e => {
                  e.currentTarget.style.color = 'var(--sl-text-tertiary)';
                  e.currentTarget.style.borderColor = 'var(--sl-border-light)';
                  e.currentTarget.style.background = 'var(--sl-surface-hover)';
                }}
              >归档</button>
            </div>
          </div>
        );
      })}
    </div>
  );
};
