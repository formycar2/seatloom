import React, { useMemo } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  Clock,
  Inbox,
  Loader2,
  ShieldAlert,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import MetricCard from './MetricCard';
import WorkflowPanorama from './WorkflowPanorama';
import ActionQueue from './ActionQueue';
import RecentActivity from './RecentActivity';
import { InboxItem } from '../types';

interface SupervisionDashboardProps {
  onNavigateToInbox?: () => void;
  onSelectInboxItem?: (item: InboxItem) => void;
}

const ACTION_TYPES = new Set([
  '待处理交接',
  '需要验收',
  '需确认范围',
  '验收待决策',
  '输入请求',
  '会话恢复',
  '待补产物',
]);

const SupervisionDashboard: React.FC<SupervisionDashboardProps> = ({
  onNavigateToInbox,
  onSelectInboxItem,
}) => {
  const { activeProjectId, projectData, projects } = useDataStore();
  const currentProject = projects.find((p) => p.id === activeProjectId);
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const metrics = useMemo(() => {
    if (!currentData) return null;

    const pendingDecisions = currentData.inboxItems.filter((item) =>
      ACTION_TYPES.has(item.type)
    ).length;

    const inProgress = currentData.workItems.filter(
      (wi) => wi.status === 'Active'
    ).length;

    const blocked =
      currentData.workItems.filter((wi) => wi.status === 'Blocked').length +
      currentData.sessions.filter((s) => s.status === 'InputRequired').length;

    const inReview = currentData.workItems.filter(
      (wi) => wi.status === 'InReview'
    ).length;

    // Budget placeholder — no real budget data in seed yet
    const budgetHealthPercent = 72;

    return { pendingDecisions, inProgress, blocked, inReview, budgetHealthPercent };
  }, [currentData]);

  if (!currentProject || !currentData || !metrics) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)]">
        <div className="text-center space-y-3">
          <Inbox size={40} className="mx-auto opacity-30" />
          <p className="text-sm">请选择一个项目以查看监督概览</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-[1200px] mx-auto p-6 space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-xl font-semibold text-[var(--sl-ink)]">监督概览</h1>
          <p className="text-caption text-[var(--sl-ink-secondary)] mt-1">
            {currentProject.name} — 掌握全局，解决阻塞，确保高效推进
          </p>
        </div>

        {/* Metric cards — responsive grid */}
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
          <MetricCard
            label="待决策"
            value={metrics.pendingDecisions}
            icon={AlertTriangle}
            color={metrics.pendingDecisions > 0 ? 'warning' : 'done'}
            onClick={onNavigateToInbox}
          />
          <MetricCard
            label="执行中"
            value={metrics.inProgress}
            icon={Loader2}
            color="primary"
          />
          <MetricCard
            label="阻塞"
            value={metrics.blocked}
            icon={ShieldAlert}
            color={metrics.blocked > 0 ? 'error' : 'done'}
          />
          <MetricCard
            label="审查中"
            value={metrics.inReview}
            icon={CheckCircle2}
            color="review"
          />
          <MetricCard
            label="预算余量"
            value={metrics.budgetHealthPercent}
            icon={Clock}
            color="success"
          />
        </div>

        {/* Workflow panorama */}
        <WorkflowPanorama />

        {/* Action queue + Recent activity — responsive */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <div className="lg:col-span-2">
            <ActionQueue
              onSelectItem={(item) => onSelectInboxItem?.(item)}
              onViewAll={onNavigateToInbox}
              maxItems={6}
            />
          </div>
          <div className="lg:col-span-1">
            <RecentActivity />
          </div>
        </div>
      </div>
    </div>
  );
};

export default SupervisionDashboard;
