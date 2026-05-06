/**
 * WorkItemsView.tsx
 * 工作项全生命周期视图：展示 WorkItem 列表，支持按状态筛选
 * 接收来自 ProjectDashboard 的数据作为 props，不直接访问 store
 */

import React, { useState } from 'react';
import { WorkItem, Seat } from '../../types';

const STATUS_CONFIG: Record<string, { label: string; color: string; bg: string }> = {
  Draft:    { label: '草稿',  color: 'var(--sl-text-tertiary)', bg: 'var(--sl-surface-hover)' },
  Ready:    { label: '就绪',  color: 'var(--sl-blue)',          bg: 'var(--sl-blue-subtle)' },
  Active:   { label: '进行中', color: 'var(--sl-brand)',         bg: 'var(--sl-brand-subtle)' },
  Blocked:  { label: '阻塞',  color: 'var(--sl-red)',           bg: 'var(--sl-red-subtle)' },
  InReview: { label: '审阅中', color: 'var(--sl-amber)',         bg: 'var(--sl-amber-subtle)' },
  Verified: { label: '已验证', color: 'var(--sl-green)',         bg: 'var(--sl-green-subtle)' },
  Done:     { label: '完成',  color: 'var(--sl-green)',         bg: 'var(--sl-green-subtle)' },
  Reopened: { label: '重开',  color: 'var(--sl-amber)',         bg: 'var(--sl-amber-subtle)' },
  Drifted:  { label: '漂移',  color: 'var(--sl-red)',           bg: 'var(--sl-red-subtle)' },
};

const PRIORITY_CONFIG: Record<string, { label: string; color: string }> = {
  Critical: { label: '紧急', color: 'var(--sl-red)' },
  High:     { label: '高',   color: 'var(--sl-amber)' },
  Medium:   { label: '中',   color: 'var(--sl-blue)' },
  Low:      { label: '低',   color: 'var(--sl-text-tertiary)' },
};

type FilterId = 'all' | 'active' | 'ready' | 'blocked' | 'review' | 'done';

const FILTER_TABS: { id: FilterId; label: string; statuses: string[] }[] = [
  { id: 'all',     label: '全部',   statuses: [] },
  { id: 'active',  label: '进行中', statuses: ['Active', 'Reopened'] },
  { id: 'ready',   label: '待处理', statuses: ['Draft', 'Ready'] },
  { id: 'blocked', label: '阻塞',   statuses: ['Blocked'] },
  { id: 'review',  label: '审阅中', statuses: ['InReview'] },
  { id: 'done',    label: '已完成', statuses: ['Done', 'Verified'] },
];

const TIER_COLORS: Record<string, string> = {
  L1: 'var(--sl-green)',
  L2: 'var(--sl-amber)',
  L3: 'var(--sl-red)',
};

export const WorkItemsView: React.FC<{
  workItems: WorkItem[];
  seats: Seat[];
}> = ({ workItems, seats }) => {
  const [activeFilter, setActiveFilter] = useState<FilterId>('all');

  const seatMap = new Map(seats.map(s => [s.id, s.name]));

  const filteredItems = (() => {
    const tab = FILTER_TABS.find(t => t.id === activeFilter)!;
    const base = tab.id === 'all' ? workItems : workItems.filter(w => tab.statuses.includes(w.status));
    return [...base].sort((a, b) => b.updated_at.localeCompare(a.updated_at));
  })();

  const tabCount = (tab: typeof FILTER_TABS[number]) =>
    tab.id === 'all' ? workItems.length : workItems.filter(w => tab.statuses.includes(w.status)).length;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
      {/* Status filter tabs */}
      <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 12 }}>
        {FILTER_TABS.map(tab => {
          const count = tabCount(tab);
          const isActive = tab.id === activeFilter;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveFilter(tab.id)}
              style={{
                padding: '5px 12px', fontSize: 11, fontWeight: 600,
                borderRadius: 'var(--sl-radius-full)', border: 'none', cursor: 'pointer',
                background: isActive ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
                color: isActive ? 'white' : 'var(--sl-text-secondary)',
                transition: 'all 120ms ease',
              }}
            >
              {tab.label}
              {count > 0 && (
                <span style={{ marginLeft: 5, opacity: 0.8 }}>{count}</span>
              )}
            </button>
          );
        })}
      </div>

      {/* Work item cards */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {filteredItems.length === 0 && (
          <div style={{ padding: 32, textAlign: 'center', color: 'var(--sl-text-tertiary)', fontSize: 13 }}>
            此筛选条件下暂无工作项
          </div>
        )}
        {filteredItems.map(w => {
          const statusCfg = STATUS_CONFIG[w.status] || { label: w.status, color: 'var(--sl-text-tertiary)', bg: 'var(--sl-surface-hover)' };
          const priorityCfg = PRIORITY_CONFIG[w.priority] || { label: w.priority, color: 'var(--sl-text-tertiary)' };
          const ownerName = w.owner_seat_id ? (seatMap.get(w.owner_seat_id) || w.owner_seat_id) : '—';
          const tier = w.change_tier_record?.tier;
          const delegateName = w.active_delegation
            ? (seatMap.get(w.active_delegation.delegate_seat_id) || w.active_delegation.delegate_seat_id)
            : null;

          return (
            <div
              key={w.id}
              style={{
                padding: '14px 16px', borderRadius: 'var(--sl-radius-md)',
                background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)',
              }}
            >
              {/* Header row: status + priority + tier + id */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
                <span style={{
                  fontSize: 10, fontWeight: 700, padding: '2px 8px',
                  borderRadius: 'var(--sl-radius-full)', background: statusCfg.bg, color: statusCfg.color,
                }}>{statusCfg.label}</span>
                <span style={{ fontSize: 10, fontWeight: 700, color: priorityCfg.color }}>● {priorityCfg.label}</span>
                {tier && (
                  <span style={{
                    fontSize: 10, fontWeight: 700, padding: '1px 6px', borderRadius: 3,
                    background: `${TIER_COLORS[tier] || 'var(--sl-amber)'}18`,
                    color: TIER_COLORS[tier] || 'var(--sl-amber)',
                  }}>变更 {tier}</span>
                )}
                {delegateName && (
                  <span style={{ fontSize: 10, color: 'var(--sl-brand)', fontWeight: 600 }}>↗ 委派: {delegateName}</span>
                )}
                <div style={{ flex: 1 }} />
                <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', fontFamily: 'monospace' }}>{w.id}</span>
              </div>

              {/* Title */}
              <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 4 }}>
                {w.title}
              </div>

              {/* Goal snippet */}
              {w.goal && (
                <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', lineHeight: 1.55, marginBottom: 8 }}>
                  {w.goal.length > 120 ? w.goal.slice(0, 120) + '…' : w.goal}
                </div>
              )}

              {/* Footer: owner + AC count + deps */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 14, fontSize: 11, color: 'var(--sl-text-tertiary)', flexWrap: 'wrap' }}>
                <span>负责席位: <span style={{ color: 'var(--sl-text-secondary)', fontWeight: 500 }}>{ownerName}</span></span>
                <span>验收条件: <span style={{ fontWeight: 500 }}>{w.acceptance_criteria.length} 条</span></span>
                {w.depends_on.length > 0 && (
                  <span>依赖: <span style={{ fontWeight: 500 }}>{w.depends_on.length} 项</span></span>
                )}
                <span style={{ marginLeft: 'auto', fontSize: 10 }}>
                  更新: {w.updated_at.substring(0, 10)}
                </span>
              </div>

              {/* Acceptance criteria preview (top 3) */}
              {w.acceptance_criteria.length > 0 && (
                <div style={{ marginTop: 10, paddingTop: 10, borderTop: '1px solid var(--sl-bg)' }}>
                  <div style={{ fontSize: 10, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 6, textTransform: 'uppercase', letterSpacing: '0.04em' }}>验收条件</div>
                  <ul style={{ margin: 0, padding: '0 0 0 16px' }}>
                    {w.acceptance_criteria.slice(0, 3).map((ac, i) => (
                      <li key={i} style={{ fontSize: 11, color: 'var(--sl-text-secondary)', lineHeight: 1.5, marginBottom: 2 }}>
                        {ac.length > 100 ? ac.slice(0, 100) + '…' : ac}
                      </li>
                    ))}
                    {w.acceptance_criteria.length > 3 && (
                      <li style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', listStyle: 'none', marginTop: 2 }}>
                        +{w.acceptance_criteria.length - 3} 条更多…
                      </li>
                    )}
                  </ul>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
