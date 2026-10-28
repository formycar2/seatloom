/**
 * GlobalDashboard.tsx
 * AD-013 Global context view: 跨项目摘要面板。
 * 每个 project 一行卡片，点击进入 project mode。
 *
 * 2026-10-20 (AEGIS-2026-10-20-v5-honest-surfaces-v1 三):这里曾 import
 * MOCK_GLOBAL_SUMMARY —— 一个纯写死的假汇总，里面是 p-1/p-2/p-3 这种从未存在过的
 * 项目，配着"健康/受阻/阻塞项"这些数字。它在 SupervisorPanel 里完整可达，所以操作者
 * 看到的就是三个不存在的项目和三个编造的状态。
 *
 * 改成从 useDataStore 派生：行来自真实 projects，数字来自每个项目自己的
 * workItems / sessions / inboxItems / events。不猜的部分说不猜。
 */

import React, { useMemo } from 'react';
import { AlertTriangle, ChevronRight } from 'lucide-react';
import { useDataStore } from '../../stores/useDataStore';
import { GlobalProjectSummary } from '../types';

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
  const { projects, projectData, backendHydrated, backendHydrateError } = useDataStore();

  // 从真实 store 派生,一个项目一行。数字全部来自这个项目自己的数据。
  //
  // 三项说明(契约验收标准 3 —— 聚合数值必须带范围标注):
  //   - healthStatus 不是后端字段,是从这个项目自己的工作项/会话/收件箱推出来的:
  //     有阻塞 -> blocked;有待决策 -> warning;都没有 -> healthy。这不是"健康度评分",
  //     只是把同一批数字换了个说法,所以每行同时给出原始计数,不只给颜色。
  //   - activeBlockerCount = 被阻塞的工作项 + 等待输入的会话(不是任何推测值)。
  //   - lastActivityTime = 这个项目最近一条账本事件的发生时刻;没有事件就没有时间,
  //     那个项目显示"无记录"而不是一个编的日期。
  const summary: GlobalProjectSummary[] = useMemo(() => {
    return projects.map((p) => {
      const d = projectData[p.id];
      const blockedWorkItems = d ? d.workItems.filter((wi) => wi.status === 'Blocked').length : 0;
      const inputRequiredSessions = d
        ? d.sessions.filter((se) => se.status === 'InputRequired').length
        : 0;
      const pendingDecisions = d
        ? d.inboxItems.filter((item) =>
            ['WorkItemBlocked', 'HandoffPending', 'ReviewPending', 'AcceptancePending'].includes(
              item.type,
            ),
          ).length
        : 0;
      const activeBlockerCount = blockedWorkItems + inputRequiredSessions;
      const events = d?.events ?? [];
      const lastEvent = events.reduce<{ occurred_at: string } | null>(
        (latest, e) => (!latest || e.occurred_at > latest.occurred_at ? e : latest),
        null,
      );
      const healthStatus: GlobalProjectSummary['healthStatus'] =
        activeBlockerCount > 0 ? 'blocked' : pendingDecisions > 0 ? 'warning' : 'healthy';
      return {
        projectId: p.id,
        projectName: p.name,
        healthStatus,
        activeBlockerCount,
        // 没有事件时给空串;渲染层把空串显示成"无记录",不造日期。
        lastActivityTime: lastEvent ? lastEvent.occurred_at : '',
      };
    });
  }, [projects, projectData]);

  const blockedCount = summary.filter((s) => s.healthStatus === 'blocked').length;
  const warningCount = summary.filter((s) => s.healthStatus === 'warning').length;
  const totalBlockers = summary.reduce((sum, s) => sum + s.activeBlockerCount, 0);
  const scopeLabel = `统计范围:当前读取到的 ${summary.length} 个项目`;

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
            <strong style={{ fontSize: 16, fontWeight: 700, color: 'var(--sl-red)' }}>
              {blockedCount}
            </strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>受阻</span>
          </span>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700, color: 'var(--sl-amber)' }}>
              {warningCount}
            </strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>需关注</span>
          </span>
          <span>
            <strong style={{ fontSize: 16, fontWeight: 700 }}>{totalBlockers}</strong>
            <span style={{ color: 'var(--sl-text-tertiary)', marginLeft: 4 }}>阻塞项</span>
          </span>
        </div>
        <div style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 6 }}>
          {scopeLabel} · 阻塞项 = 被阻塞的工作项 + 等待输入的会话 ·{' '}
          {backendHydrated ? '数据来自当前后端读取' : '后端读取未成功'}
        </div>
      </div>

      {/* V5: 后端没读成功时明说,不渲染一片空白让人以为"没有项目" */}
      {!backendHydrated && (
        <div
          style={{
            padding: '10px 12px',
            borderRadius: 'var(--sl-radius-md)',
            background: 'var(--sl-red-subtle)',
            border: '1px solid var(--sl-red)',
            color: 'var(--sl-red)',
            fontSize: 12,
            lineHeight: 1.6,
            marginBottom: 12,
          }}
        >
          项目数据读取未成功,因此这里给不出真实汇总 —— 不用假数据顶着。
          <div style={{ fontFamily: 'var(--sl-font-mono, ui-monospace, monospace)', marginTop: 4 }}>
            {backendHydrateError ?? '(未记录错误文本)'}
          </div>
        </div>
      )}

      {backendHydrated && summary.length === 0 && (
        <div
          style={{
            padding: '10px 12px',
            borderRadius: 'var(--sl-radius-md)',
            background: 'var(--sl-surface)',
            border: '1px solid var(--sl-border-light)',
            color: 'var(--sl-text-secondary)',
            fontSize: 12,
            lineHeight: 1.6,
            marginBottom: 12,
          }}
        >
          数据库里一个项目都没有(读取是成功的,所以这个空是真的空)。
        </div>
      )}

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
                <span>
                  {p.lastActivityTime
                    ? `最近活动 ${formatRelativeTime(p.lastActivityTime)}`
                    : '无活动记录'}
                </span>
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
