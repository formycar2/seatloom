/**
 * GlobalDashboard.tsx
 * AD-013 Global context view: 跨项目摘要面板。
 * 每个 project 一行卡片，点击进入 project mode。
 * 数据源：MOCK_GLOBAL_SUMMARY（纯 mock，不从 useDataStore 派生）。
 */

import React from 'react';
import { AlertTriangle, ChevronRight } from 'lucide-react';
import { GlobalProjectSummary } from '../types';
import { MOCK_GLOBAL_SUMMARY } from '../mock-data';

interface GlobalDashboardProps {
  onSelectProject: (projectId: string) => void;
}

const HEALTH_LABEL: Record<GlobalProjectSummary['healthStatus'], string> = {
  healthy: '健康',
  warning: '需关注',
  blocked: '受阻',
};

const HEALTH_COLOR: Record<GlobalProjectSummary['healthStatus'], string> = {
  healthy: 'var(--sl-green)',
  warning: 'var(--sl-amber)',
  blocked: 'var(--sl-red)',
};

function formatRelativeTime(iso: string): string {
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return '';
  const diffMs = Date.now() - t;
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return '刚刚';
  if (diffMin < 60) return `${diffMin} 分钟前`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr} 小时前`;
  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 7) return `${diffDay} 天前`;
  return new Date(iso).toLocaleDateString('zh-CN');
}

export const GlobalDashboard: React.FC<GlobalDashboardProps> = ({ onSelectProject }) => {
  const summary = MOCK_GLOBAL_SUMMARY;
  const blockedCount = summary.filter((s) => s.healthStatus === 'blocked').length;
  const warningCount = summary.filter((s) => s.healthStatus === 'warning').length;
  const totalBlockers = summary.reduce((sum, s) => sum + s.activeBlockerCount, 0);

  return (
    <div style={{ flex: 1, overflowY: 'auto', padding: 16 }}>
      {/* Summary header */}
      <div
        style={{
          padding: '12px 14px',
          borderRadius: 'var(--sl-radius-md)',
          background: 'var(--sl-surface)',
          border: '1px solid var(--sl-divider)',
          marginBottom: 12,
        }}
      >
        <div
          style={{
            fontSize: 11,
            fontWeight: 700,
            color: 'var(--sl-text-secondary)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
            marginBottom: 8,
          }}
        >
          全局摘要
        </div>
        <div style={{ display: 'flex', gap: 16, fontSize: 12, color: 'var(--sl-text-primary)' }}>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700 }}>{summary.length}</strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>项目</span>
          </span>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700, color: 'var(--sl-red)' }}>{blockedCount}</strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>受阻</span>
          </span>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700, color: 'var(--sl-amber)' }}>{warningCount}</strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>需关注</span>
          </span>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700 }}>{totalBlockers}</strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>阻塞项</span>
          </span>
        </div>
      </div>

      {/* Project rows */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {summary.map((p) => (
          <button
            key={p.projectId}
            type="button"
            onClick={() => onSelectProject(p.projectId)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 12,
              padding: '12px 14px',
              borderRadius: 'var(--sl-radius-md)',
              background: 'var(--sl-surface)',
              border: '1px solid var(--sl-divider)',
              cursor: 'pointer',
              textAlign: 'left',
              transition: 'background 0.12s, border-color 0.12s',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'var(--sl-surface-hover)';
              e.currentTarget.style.borderColor = 'var(--sl-border)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--sl-surface)';
              e.currentTarget.style.borderColor = 'var(--sl-divider)';
            }}
          >
            {/* Health dot */}
            <span
              aria-hidden
              style={{
                width: 10,
                height: 10,
                borderRadius: '50%',
                background: HEALTH_COLOR[p.healthStatus],
                flexShrink: 0,
              }}
            />

            {/* Name + status */}
            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: 14,
                  fontWeight: 600,
                  color: 'var(--sl-text-primary)',
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                }}
              >
                {p.projectName}
              </div>
              <div
                style={{
                  fontSize: 11,
                  color: 'var(--sl-text-tertiary)',
                  marginTop: 2,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                }}
              >
                <span style={{ color: HEALTH_COLOR[p.healthStatus], fontWeight: 600 }}>
                  {HEALTH_LABEL[p.healthStatus]}
                </span>
                <span>·</span>
                <span>{formatRelativeTime(p.lastActivityTime)}</span>
              </div>
            </div>

            {/* Blocker chip */}
            {p.activeBlockerCount > 0 && (
              <span
                title={`${p.activeBlockerCount} 个活跃阻塞`}
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 4,
                  fontSize: 11,
                  fontWeight: 600,
                  padding: '2px 8px',
                  borderRadius: 'var(--sl-radius-full)',
                  background: 'var(--sl-red)15',
                  color: 'var(--sl-red)',
                  flexShrink: 0,
                }}
              >
                <AlertTriangle size={11} />
                {p.activeBlockerCount}
              </span>
            )}

            <ChevronRight size={16} style={{ color: 'var(--sl-text-tertiary)', flexShrink: 0 }} />
          </button>
        ))}
      </div>
    </div>
  );
};
