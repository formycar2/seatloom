/**
 * BlockersSection.tsx
 * 阻塞项区块：显示当前阶段的阻塞工作项（红色高亮），没有阻塞时不渲染
 */

import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface BlockersSectionProps {
  blockers: { text: string; owner: string; since: string; [key: string]: any }[];
  onItemMouseEnter?: (e: React.MouseEvent, blocker: any) => void;
  onItemMouseLeave?: () => void;
}

export const BlockersSection: React.FC<BlockersSectionProps> = ({ blockers, onItemMouseEnter, onItemMouseLeave }) => {
  if (blockers.length === 0) return null;

  return (
    <div style={{
      padding: '12px 14px', borderRadius: 'var(--sl-radius-md)',
      background: 'var(--sl-surface)', border: '1px solid var(--sl-red)'
    }}>
      <div style={{
        fontSize: 11, fontWeight: 700, color: 'var(--sl-red)',
        marginBottom: 10, textTransform: 'uppercase', letterSpacing: '0.05em',
        display: 'flex', alignItems: 'center', gap: 6
      }}>
        <AlertTriangle size={14} /> 目前阻塞 (BLOCKERS)
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {blockers.map((b: any, i: number) => (
          <div key={i}
            onMouseEnter={e => {
              const btn = e.currentTarget.querySelector('button') as HTMLButtonElement | null;
              if (btn) btn.style.opacity = '1';
              onItemMouseEnter?.(e, b);
            }}
            onMouseLeave={e => {
              const btn = e.currentTarget.querySelector('button') as HTMLButtonElement | null;
              if (btn) btn.style.opacity = '0';
              onItemMouseLeave?.();
            }}
            style={{
              display: 'flex', alignItems: 'center', gap: 12,
              padding: '10px 14px', borderRadius: 'var(--sl-radius-md)',
              background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
              transition: 'all 150ms ease',
            }}
          >
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', fontWeight: 600 }}>{b.text}</div>
              <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', marginTop: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
                <span style={{ fontWeight: 600 }}>{b.owner}</span>
                <span style={{ color: 'var(--sl-text-tertiary)' }}>· 阻塞已持续 {b.since}</span>
              </div>
            </div>
            <button
              style={{
                opacity: 0, transition: 'opacity 200ms ease',
                fontSize: 11, fontWeight: 600, padding: '6px 12px', cursor: 'pointer',
                borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-red)', color: 'white', border: 'none',
              }}
            >立即处理</button>
          </div>
        ))}
      </div>
    </div>
  );
};
