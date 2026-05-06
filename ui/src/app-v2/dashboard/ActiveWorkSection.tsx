/**
 * ActiveWorkSection.tsx
 * 正在发生区块：显示当前 DAG 中状态为 active 的工作节点及其上下游依赖关系
 */

import React from 'react';
import { Activity } from 'lucide-react';

const Chip: React.FC<{ label: string; title?: string; color?: string }> = ({ label, title, color = 'var(--sl-brand)' }) => (
  <span
    title={title || label}
    style={{
      display: 'inline-block', fontSize: 10, fontWeight: 600, padding: '2px 6px',
      borderRadius: 'var(--sl-radius-full)', background: `${color}15`, color, marginRight: 4, marginBottom: 2,
    }}
  >{label}</span>
);

interface ActiveWorkSectionProps {
  items: any[];
  onItemMouseEnter?: (e: React.MouseEvent, item: any) => void;
  onItemMouseLeave?: () => void;
}

export const ActiveWorkSection: React.FC<ActiveWorkSectionProps> = ({ items, onItemMouseEnter, onItemMouseLeave }) => {
  if (items.length === 0) return null;

  return (
    <div style={{ padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 10, textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: 6 }}>
        <Activity size={14} /> 正在发生 (ACTIVE)
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {items.map((item: any) => {
          const color = item.ownerColor || (item.statusLabel === '受阻塞' ? 'var(--sl-red)' : 'var(--sl-brand)');
          return (
            <div key={item.sourceId}
              onMouseEnter={e => onItemMouseEnter?.(e, item)}
              onMouseLeave={() => onItemMouseLeave?.()}
              style={{
                padding: '10px 14px', borderRadius: 'var(--sl-radius-md)',
                border: `1px solid ${color}40`, background: 'var(--sl-bg)',
                cursor: 'pointer'
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                {item.ownerAvatar && (
                  <div style={{
                    width: 20, height: 20, borderRadius: '50%',
                    background: `${color}18`, color: color,
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    fontSize: 10, fontWeight: 700, border: `1px solid ${color}40`,
                  }}>{item.ownerAvatar}</div>
                )}
                <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', flex: 1 }}>
                  {item.title}
                </span>
                <span style={{
                  fontSize: 10, fontWeight: 600, padding: '2px 8px', borderRadius: 'var(--sl-radius-full)',
                  background: `${color}15`, color: color,
                }}>
                  {item.statusLabel}
                </span>
              </div>
              <div style={{ fontSize: 11, paddingLeft: 28, display: 'flex', alignItems: 'center', gap: 12 }}>
                {item.sourceKind === 'WorkItem' && (
                  <>
                    <div style={{ color: 'var(--sl-text-tertiary)' }}>
                      评审阶层: {item.reviewTier || 'N/A'}
                    </div>
                    <div style={{ color: 'var(--sl-brand)', fontWeight: 600 }}>{item.refLabel}</div>
                  </>
                )}
                {item.sourceKind === 'Session' && (
                  <>
                    <div style={{ color: 'var(--sl-text-tertiary)' }}>
                      会话ID: {item.refLabel}
                    </div>
                    {item.promptBadge && (
                      <Chip label={item.promptBadge} color="var(--sl-amber)" />
                    )}
                  </>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
