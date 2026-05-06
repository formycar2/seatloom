/**
 * ArtifactsView.tsx
 * 文档与证据链视图：展示 Artifact 列表，支持按 Template（T1-T7）筛选
 * 接收来自 ProjectDashboard 的数据作为 props，不直接访问 store
 */

import React, { useState } from 'react';
import { FileText } from 'lucide-react';
import { Artifact, ArtifactTemplate } from '../../types';

const TEMPLATE_LABELS: Record<ArtifactTemplate, string> = {
  T1: 'T1 权威文档',
  T2: 'T2 席位角色',
  T3: 'T3 任务包',
  T4: 'T4 评审',
  T5: 'T5 验收',
  T6: 'T6 日志',
  T7: 'T7 治理',
};

const TEMPLATE_COLORS: Record<ArtifactTemplate, string> = {
  T1: 'var(--sl-purple)',
  T2: 'var(--sl-teal)',
  T3: 'var(--sl-brand)',
  T4: 'var(--sl-amber)',
  T5: 'var(--sl-green)',
  T6: 'var(--sl-blue)',
  T7: 'var(--sl-red)',
};

const STATUS_COLORS: Record<string, string> = {
  draft:               'var(--sl-amber)',
  issued:              'var(--sl-blue)',
  active:              'var(--sl-green)',
  delivered:           'var(--sl-blue)',
  review:              'var(--sl-amber)',
  recovered:           'var(--sl-purple)',
  accepted:            'var(--sl-green)',
  approved:            'var(--sl-green)',
  in_progress:         'var(--sl-brand)',
  ready:               'var(--sl-green)',
  pending_review:      'var(--sl-amber)',
  conditionally_adopted: 'var(--sl-teal)',
};

const SUBTYPE_LABELS: Record<string, string> = {
  prd: 'PRD', ux_spec: 'UX规范', interaction_spec: '交互规范', acceptance_spec: '验收规范',
  architecture_design: '架构设计', architecture_decisions: '架构决策', seat_role: '席位角色',
  task: '任务', fix: '修复', integration: '集成', verification: '验证',
  gap_review: '差距评审', benchmark: '基准测试', process_mapping: '过程映射', design_proposal: '设计提案',
  acceptance_review: '验收审查', gate_decision: '门禁决策',
  daily_log: '日志', coordination_rules: '协调规则', workflow_principles: '工作流原则',
  collaboration_protocol: '协作协议', document_templates: '文档模板',
};

const basename = (path: string) => path.split('/').pop() || path;

export const ArtifactsView: React.FC<{
  artifacts: Artifact[];
}> = ({ artifacts }) => {
  const [templateFilter, setTemplateFilter] = useState<ArtifactTemplate | 'all'>('all');

  const templates: Array<ArtifactTemplate | 'all'> = [
    'all',
    ...(['T1', 'T2', 'T3', 'T4', 'T5', 'T6', 'T7'] as ArtifactTemplate[]).filter(
      t => artifacts.some(a => a.template === t)
    ),
  ];

  const filtered = [...artifacts]
    .filter(a => templateFilter === 'all' || a.template === templateFilter)
    .sort((a, b) => b.created_at.localeCompare(a.created_at));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
      {/* Template filter chips */}
      <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 12 }}>
        {templates.map(t => {
          const isActive = t === templateFilter;
          const count = t === 'all'
            ? artifacts.length
            : artifacts.filter(a => a.template === t).length;
          const color = t === 'all' ? 'var(--sl-brand)' : TEMPLATE_COLORS[t];

          return (
            <button
              key={t}
              onClick={() => setTemplateFilter(t)}
              style={{
                padding: '5px 12px', fontSize: 11, fontWeight: 600,
                borderRadius: 'var(--sl-radius-full)', border: 'none', cursor: 'pointer',
                background: isActive ? color : 'var(--sl-surface-hover)',
                color: isActive ? 'white' : 'var(--sl-text-secondary)',
                transition: 'all 120ms ease',
              }}
            >
              {t === 'all' ? '全部' : TEMPLATE_LABELS[t]}
              <span style={{ marginLeft: 5, opacity: 0.8 }}>{count}</span>
            </button>
          );
        })}
      </div>

      {/* Artifact cards */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {filtered.length === 0 && (
          <div style={{ padding: 32, textAlign: 'center', color: 'var(--sl-text-tertiary)', fontSize: 13 }}>
            此筛选条件下暂无文档
          </div>
        )}
        {filtered.map(a => {
          const tColor = TEMPLATE_COLORS[a.template] || 'var(--sl-brand)';
          const statusColor = STATUS_COLORS[a.status || ''] || 'var(--sl-text-tertiary)';
          const subtypeLabel = SUBTYPE_LABELS[a.subtype] || a.subtype;

          return (
            <div
              key={a.id}
              style={{
                padding: '12px 16px', borderRadius: 'var(--sl-radius-md)',
                background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)',
                transition: 'border-color 120ms ease',
              }}
              onMouseEnter={e => { e.currentTarget.style.borderColor = `${tColor}40`; }}
              onMouseLeave={e => { e.currentTarget.style.borderColor = 'var(--sl-border-light)'; }}
            >
              {/* Badge row */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6, flexWrap: 'wrap' }}>
                <span style={{
                  fontSize: 10, fontWeight: 800, padding: '2px 7px', borderRadius: 3,
                  background: `${tColor}18`, color: tColor,
                }}>{a.template}</span>
                <span style={{
                  fontSize: 10, fontWeight: 600, padding: '2px 6px', borderRadius: 3,
                  background: 'var(--sl-surface-hover)', color: 'var(--sl-text-secondary)',
                }}>{subtypeLabel}</span>
                {a.status && (
                  <span style={{
                    fontSize: 10, fontWeight: 600, padding: '2px 6px', borderRadius: 3,
                    background: `${statusColor}15`, color: statusColor,
                  }}>{a.status}</span>
                )}
                {a.version && (
                  <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', fontFamily: 'monospace' }}>{a.version}</span>
                )}
                <div style={{ flex: 1 }} />
                <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                  {a.author && <>{a.author} · </>}{a.date}
                </span>
              </div>

              {/* Title */}
              <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 4 }}>
                {a.title}
              </div>

              {/* Summary */}
              {a.summary && (
                <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', lineHeight: 1.55, marginBottom: 8 }}>
                  {a.summary.length > 140 ? a.summary.slice(0, 140) + '…' : a.summary}
                </div>
              )}

              {/* Summary points preview */}
              {a.summary_points && a.summary_points.length > 0 && (
                <ul style={{ margin: '0 0 8px 0', padding: '0 0 0 16px' }}>
                  {a.summary_points.slice(0, 2).map((pt, i) => (
                    <li key={i} style={{ fontSize: 11, color: 'var(--sl-text-secondary)', lineHeight: 1.5, marginBottom: 2 }}>
                      {pt.length > 80 ? pt.slice(0, 80) + '…' : pt}
                    </li>
                  ))}
                </ul>
              )}

              {/* Storage path */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 5, fontSize: 10, color: 'var(--sl-text-tertiary)', fontFamily: 'monospace' }}>
                <FileText size={10} />
                <span title={a.storage_path}>{basename(a.storage_path)}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
