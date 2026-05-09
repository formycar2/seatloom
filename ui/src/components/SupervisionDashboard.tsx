import React, { useMemo, useState } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  Clock,
  Inbox,
  Loader2,
  ShieldAlert,
  Info,
  ChevronRight,
  Zap,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import MetricCard from './MetricCard';
import WorkflowPanorama from './WorkflowPanorama';
import ActionQueue from './ActionQueue';
import RecentActivity from './RecentActivity';
import { InboxItem } from '../types';

const ACTION_TYPES = new Set([
  '待处理交接',
  '需要验收',
  '需确认范围',
  '验收待决策',
  '输入请求',
  '会话恢复',
  '待补产物',
]);

const SupervisionDashboard: React.FC<{
  onNavigateToInbox?: () => void;
  onSelectInboxItem?: (item: InboxItem) => void;
}> = ({
  onNavigateToInbox,
  onSelectInboxItem,
}) => {
  const { activeProjectId, projectData, projects } = useDataStore();
  const currentProject = projects.find((p) => p.id === activeProjectId);
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const [hoveredItem, setHoveredItem] = useState<{ type: string; data: any; x: number; y: number } | null>(null);

  const metrics = useMemo(() => {
    if (!currentData) return null;

    const pendingDecisions = currentData.inboxItems.length;
    const inProgress = currentData.workItems.filter(wi => wi.status === 'Active').length;
    const blocked = currentData.workItems.filter(wi => wi.status === 'Blocked').length +
                    currentData.sessions.filter(s => s.status === 'InputRequired').length;
    const inReview = currentData.workItems.filter(wi => wi.status === 'InReview').length;
    const budgetHealthPercent = 72;

    return { pendingDecisions, inProgress, blocked, inReview, budgetHealthPercent };
  }, [currentData]);

  const handleMouseMove = (e: React.MouseEvent, type: string, itemData: any) => {
    setHoveredItem({
      type,
      data: itemData,
      x: e.clientX + 15,
      y: e.clientY + 15
    });
  };

  if (!currentProject || !currentData || !metrics) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)] bg-canvas">
        <div className="text-center space-y-3">
          <Inbox size={40} className="mx-auto opacity-30" />
          <p className="text-sm">请选择项目以开启监察</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto bg-canvas relative">
      <div className="max-w-[1400px] mx-auto p-8 space-y-8">
        {/* Header */}
        <div className="flex items-end justify-between border-b border-[var(--sl-border-subtle)] pb-6">
          <div>
            <h1 className="text-3xl font-black text-[var(--sl-ink)] tracking-tighter uppercase">指挥中心</h1>
            <p className="text-base font-bold text-[var(--sl-ink-secondary)] mt-1 flex items-center gap-3">
              <span className="px-2 py-0.5 rounded bg-primary text-white text-[10px] font-black">{currentProject.id}</span>
              {currentProject.name} — 基于真值账本的项目监察
            </p>
          </div>
          <div className="text-right">
             <div className="text-[10px] font-black text-[var(--sl-ink-muted)] uppercase tracking-widest mb-1">最后对账</div>
             <div className="text-lg font-black text-primary font-mono">{new Date().toLocaleTimeString()}</div>
          </div>
        </div>

        {/* Metric cards */}
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-5">
          <MetricCard
            label="待决策"
            value={metrics.pendingDecisions}
            icon={AlertTriangle}
            color={metrics.pendingDecisions > 0 ? 'warning' : 'done'}
            onClick={onNavigateToInbox}
            onMouseEnter={(e) => handleMouseMove(e, 'metric', { label: '待决策', detail: '当前账本中标记为 Pending 的决策环路。', impact: '影响下游 2 个席位的启动计划。' })}
            onMouseLeave={() => setHoveredItem(null)}
          />
          <MetricCard
            label="推进中"
            value={metrics.inProgress}
            icon={Loader2}
            color="primary"
            onMouseEnter={(e) => handleMouseMove(e, 'metric', { label: '推进中', detail: '席位正在活跃推进的工作项总数。', impact: '平均完成率 64%，处于健康区间。' })}
            onMouseLeave={() => setHoveredItem(null)}
          />
          <MetricCard
            label="已阻塞"
            value={metrics.blocked}
            icon={ShieldAlert}
            color={metrics.blocked > 0 ? 'error' : 'done'}
            onMouseEnter={(e) => handleMouseMove(e, 'metric', { label: '已阻塞', detail: '严重偏离基线或由于输入缺失导致中断的任务。', impact: '导致关键路径延期 4h。' })}
            onMouseLeave={() => setHoveredItem(null)}
          />
          <MetricCard
            label="待验收"
            value={metrics.inReview}
            icon={CheckCircle2}
            color="review"
            onMouseEnter={(e) => handleMouseMove(e, 'metric', { label: '待验收', detail: '已完成实现，等待人工或自动化复核的工作项。', impact: '待审计文档共 3 份。' })}
            onMouseLeave={() => setHoveredItem(null)}
          />
          <MetricCard
            label="额度余量"
            value={`${metrics.budgetHealthPercent}%`}
            icon={Clock}
            color="success"
            onMouseEnter={(e) => handleMouseMove(e, 'metric', { label: '额度余量', detail: '当前 Token 消耗量与预估进度的匹配程度。', impact: '额度充足，无熔断风险。' })}
            onMouseLeave={() => setHoveredItem(null)}
          />
        </div>

        {/* Workflow panorama */}
        <div className="space-y-4">
          <div className="flex items-center gap-3">
             <div style={{ width: 4, height: 16, background: 'var(--sl-brand)', borderRadius: 2 }} />
             <h3 className="text-sm font-black text-[var(--sl-ink)] uppercase tracking-widest">工作流全景</h3>
          </div>
          <WorkflowPanorama 
            onNodeMouseEnter={(e, node) => handleMouseMove(e, 'node', node)}
            onNodeMouseLeave={() => setHoveredItem(null)}
          />
        </div>

        {/* Action queue + Recent activity */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2 space-y-4">
            <div className="flex items-center gap-3">
               <div style={{ width: 4, height: 16, background: 'var(--sl-brand)', borderRadius: 2 }} />
               <h3 className="text-sm font-black text-[var(--sl-ink)] uppercase tracking-widest">行动队列</h3>
            </div>
            <ActionQueue
              onSelectItem={(item) => onSelectInboxItem?.(item)}
              onViewAll={onNavigateToInbox}
              maxItems={6}
              onItemMouseEnter={(e, item) => handleMouseMove(e, 'action', item)}
              onItemMouseLeave={() => setHoveredItem(null)}
            />
          </div>
          <div className="lg:col-span-1 space-y-4">
            <div className="flex items-center gap-3">
               <div style={{ width: 4, height: 16, background: 'var(--sl-brand)', borderRadius: 2 }} />
               <h3 className="text-sm font-black text-[var(--sl-ink)] uppercase tracking-widest">活动日志</h3>
            </div>
            <RecentActivity />
          </div>
        </div>
      </div>

      {/* ─── Global Hover Popup ─── */}
      {hoveredItem && (
        <div 
          className="fixed z-[9999] pointer-events-none animate-in fade-in zoom-in-95 duration-150 shadow-2xl"
          style={{
            left: hoveredItem.x, top: hoveredItem.y,
            width: 340, background: 'var(--sl-surface)', borderRadius: 'var(--sl-radius-lg)',
            border: '1px solid var(--sl-border)', boxShadow: 'var(--sl-shadow-overlay)',
            padding: '20px', display: 'flex', flexDirection: 'column', gap: 14
          }}
        >
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded-full bg-primary text-white flex items-center justify-center shadow-sm">
              <Info size={12} />
            </div>
            <div className="text-[10px] font-black text-primary uppercase tracking-widest">
              {hoveredItem.type === 'metric' ? '数据深度透视' : hoveredItem.type === 'action' ? '待办任务穿透' : '席位节点详情'}
            </div>
          </div>

          <div>
            <div className="text-lg font-black text-[var(--sl-ink)] leading-tight tracking-tight">
              {hoveredItem.type === 'node' ? hoveredItem.data.label : hoveredItem.data.label || hoveredItem.data.summary || hoveredItem.data.text}
            </div>
            <div className="text-sm font-bold text-[var(--sl-ink-secondary)] mt-3 leading-relaxed">
              {hoveredItem.type === 'node' ? (hoveredItem.data.description || hoveredItem.data.payload?.summary) : hoveredItem.data.detail || hoveredItem.data.payload?.summary || '暂无详细描述。'}
            </div>
          </div>

          <div className="space-y-3 pt-4 border-t border-[var(--sl-border-subtle)]">
            <div className="flex justify-between items-center text-xs font-bold">
              <span className="text-[var(--sl-ink-muted)] uppercase">状态判定</span>
              <span className={hoveredItem.type === 'node' && hoveredItem.data.status === 'blocked' ? 'text-[var(--sl-error)]' : 'text-primary'}>
                {hoveredItem.type === 'node' ? `当前: ${hoveredItem.data.status}` : hoveredItem.data.impact || '数据对齐中'}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2 text-[11px] font-black text-primary uppercase tracking-tight pt-2 border-t border-dashed border-[var(--sl-border-subtle)]">
            <Zap size={12} fill="currentColor" />
            进入详细透视图
            <ChevronRight size={12} className="ml-auto" />
          </div>
        </div>
      )}
    </div>
  );
};

export default SupervisionDashboard;
