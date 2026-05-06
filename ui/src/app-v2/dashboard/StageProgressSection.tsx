/**
 * StageProgressSection.tsx
 * 当前阶段进度区块：显示阶段名称、完成工作项数量和进度条
 */

import React from 'react';

interface StageProgressSectionProps {
  stageName: string;
  workItemsDone: number;
  workItemsTotal: number;
}

export const StageProgressSection: React.FC<StageProgressSectionProps> = ({ stageName, workItemsDone, workItemsTotal }) => (
  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
    <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
      {stageName || '工作流全景 (DAG)'}
    </div>
    <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
      <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
        进度 <span style={{ fontWeight: 600, color: 'var(--sl-text-primary)' }}>{workItemsDone}/{workItemsTotal}</span> 项
      </div>
      <div style={{ width: 120, height: 6, borderRadius: 3, background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)', overflow: 'hidden' }}>
        <div style={{ width: `${workItemsTotal > 0 ? (workItemsDone / workItemsTotal) * 100 : 0}%`, height: '100%', background: 'var(--sl-green)', transition: 'width 1s ease' }} />
      </div>
    </div>
  </div>
);
