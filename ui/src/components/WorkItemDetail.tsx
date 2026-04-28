import React, { useMemo } from 'react';
import { CheckCircle2, Link2, ReceiptText, ShieldCheck, Target } from 'lucide-react';
import { WorkItem } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import {
  formatDateTimeZh,
  getHandoffStatusLabel,
  getPriorityLabel,
  getWorkItemStatusLabel,
} from '../utils/display';

interface WorkItemDetailProps {
  workItem: WorkItem;
}

const WorkItemDetail: React.FC<WorkItemDetailProps> = ({ workItem }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const owner = currentData?.seats.find((seat) => seat.id === workItem.owner_seat_id) || null;
  const dependencies = (currentData?.workItems || []).filter((item) => workItem.depends_on.includes(item.id));
  const relatedHandoffs = (currentData?.handoffs || [])
    .filter((handoff) => handoff.workitem_id === workItem.id)
    .sort((left, right) => new Date(right.sent_at || right.created_at).getTime() - new Date(left.sent_at || left.created_at).getTime())
    .slice(0, 4);

  const relatedEvents = useMemo(() => {
    if (!currentData) return [];
    return currentData.events
      .filter((event) => event.object_refs.some((ref) => 'WorkItem' in ref && ref.WorkItem === workItem.id))
      .sort((left, right) => new Date(right.occurred_at).getTime() - new Date(left.occurred_at).getTime())
      .slice(0, 5);
  }, [currentData, workItem.id]);

  const evidenceRefs = useMemo(() => {
    const refs = new Set<string>();
    relatedEvents.forEach((event) => event.evidence_refs.forEach((ref) => refs.add(ref)));
    return Array.from(refs).slice(0, 5);
  }, [relatedEvents]);

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="meta-header">
        <div className="flex flex-col">
          <span className="meta-label">工作项编号</span>
          <span className="meta-value font-bold">{workItem.id.toUpperCase()}</span>
        </div>
        <div className="flex flex-col">
          <span className="meta-label">权威来源</span>
          <span className="meta-value flex items-center gap-1">
            <ShieldCheck size={10} className="text-status-active" />
            协调账本
          </span>
        </div>
        <div className="flex flex-col">
          <span className={`meta-value ${workItem.priority === 'High' || workItem.priority === 'Critical' ? 'text-status-error' : 'text-status-warning'}`}>
            {getPriorityLabel(workItem.priority)}
          </span>
          <span className="meta-label">优先级</span>
        </div>
        <div className="flex flex-col">
          <span className="meta-value">{getWorkItemStatusLabel(workItem.status)}</span>
          <span className="meta-label">当前状态</span>
        </div>
      </div>

      <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
        <div className="text-[11px] font-bold text-text-muted uppercase tracking-wider">责任归属</div>
        <div className="text-sm font-semibold text-text-primary">{owner ? `${owner.name} · 负责人` : '暂未指定负责人'}</div>
        <div className="text-[12px] leading-6 text-text-secondary">创建于 {formatDateTimeZh(workItem.created_at)}，最近更新于 {formatDateTimeZh(workItem.updated_at)}。这说明当前卡片展示的是今日账本中的真实状态，而不是静态示例。</div>
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <Target size={12} /> {t.forms.goal}
        </h3>
        <div className="text-sm text-text-secondary leading-7 border-l-2 border-accent/30 pl-4 whitespace-pre-wrap">
          {workItem.goal || '该工作项尚未补充目标说明。'}
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <CheckCircle2 size={12} /> {t.forms.acceptanceCriteria}
        </h3>
        {workItem.acceptance_criteria.length > 0 ? (
          <ul className="space-y-2">
            {workItem.acceptance_criteria.map((criterion, index) => (
              <li key={`${workItem.id}-ac-${index}`} className="p-3 rounded-xl border border-border bg-bg-elevated">
                <div className="flex items-start gap-3">
                  <div className="w-6 h-6 rounded-full bg-accent/10 text-accent flex items-center justify-center text-[11px] font-bold shrink-0 mt-0.5">
                    {index + 1}
                  </div>
                  <div>
                    <div className="text-sm leading-7 text-text-primary whitespace-pre-wrap">{criterion}</div>
                    <div className="text-[11px] text-text-muted mt-1">当前状态为“{getWorkItemStatusLabel(workItem.status)}”，这条标准应成为后续验收和交接的共同判断依据。</div>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            该工作项还没有填写验收标准，因此目前只能保持在草稿或待补充状态。
          </div>
        )}
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">依赖与交接关系</h3>
        <div className="grid grid-cols-1 gap-3">
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
            <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
              <Link2 size={14} className="text-accent" /> 依赖项
            </div>
            {dependencies.length > 0 ? (
              <div className="space-y-2">
                {dependencies.map((dependency) => (
                  <div key={dependency.id} className="text-[12px] leading-6 text-text-secondary">
                    <span className="monospace text-text-muted mr-2">{dependency.id.toUpperCase()}</span>
                    {dependency.title}
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-[12px] leading-6 text-text-secondary">当前没有前置依赖，说明该工作项可以独立推进或已经完成依赖收敛。</div>
            )}
          </div>

          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
            <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
              <ReceiptText size={14} className="text-accent" /> 最近交接
            </div>
            {relatedHandoffs.length > 0 ? (
              <div className="space-y-2">
                {relatedHandoffs.map((handoff) => (
                  <div key={handoff.id} className="text-[12px] leading-6 text-text-secondary border-l-2 border-border pl-3">
                    <div className="font-semibold text-text-primary">{handoff.purpose}</div>
                    <div className="text-text-muted mt-1">{handoff.id.toUpperCase()} · {getHandoffStatusLabel(handoff.status)} · {formatDateTimeZh(handoff.sent_at || handoff.created_at)}</div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-[12px] leading-6 text-text-secondary">该工作项暂未形成交接单，说明仍在单席位推进或等待下一步分派。</div>
            )}
          </div>
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近写回与证据</h3>
        {relatedEvents.length > 0 ? (
          <div className="space-y-3">
            {relatedEvents.map((event) => (
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
                <div className="text-xs text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                <div className="text-sm font-semibold text-text-primary leading-6">{event.payload?.title || event.payload?.summary || '已记录一条工作项状态更新。'}</div>
                {event.evidence_refs.length > 0 && (
                  <div className="text-[11px] text-primary break-all">证据：{event.evidence_refs.join(' · ')}</div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            该工作项目前还没有独立的事件写回。
          </div>
        )}

        {evidenceRefs.length > 0 && (
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
            <div className="text-[11px] font-semibold text-text-primary">重点证据路径</div>
            {evidenceRefs.map((ref) => (
              <div key={ref} className="text-[11px] text-primary break-all">{ref}</div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkItemDetail;
