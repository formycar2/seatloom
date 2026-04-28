import React, { useMemo } from 'react';
import { ArrowRight, CheckCircle, FileText, ReceiptText, Send } from 'lucide-react';
import { Handoff } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import {
  formatDateTimeZh,
  getHandoffStatusLabel,
  getWorkItemStatusLabel,
} from '../utils/display';

interface HandoffDetailProps {
  handoff: Handoff;
}

const HandoffDetail: React.FC<HandoffDetailProps> = ({ handoff }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const workItem = currentData?.workItems.find((item) => item.id === handoff.workitem_id) || null;
  const relatedEvents = (currentData?.events || [])
    .filter((event) => event.object_refs.some((ref) => ('Handoff' in ref && ref.Handoff === handoff.id) || ('WorkItem' in ref && ref.WorkItem === handoff.workitem_id)))
    .sort((left, right) => new Date(right.occurred_at).getTime() - new Date(left.occurred_at).getTime())
    .slice(0, 4);

  const evidencePaths = useMemo(() => {
    const paths = new Set<string>();
    handoff.artifact_ids.forEach((artifactId) => paths.add(artifactId));
    relatedEvents.forEach((event) => event.evidence_refs.forEach((ref) => paths.add(ref)));
    return Array.from(paths);
  }, [handoff.artifact_ids, relatedEvents]);

  const getSeatName = (seatId: string) => currentData?.seats.find((seat) => seat.id === seatId)?.name || seatId;
  const fromLabel = typeof handoff.from_ref === 'object' && 'Seat' in handoff.from_ref ? getSeatName(handoff.from_ref.Seat) : '人工介入';
  const toLabel = typeof handoff.to_ref === 'object' && 'Seat' in handoff.to_ref ? getSeatName(handoff.to_ref.Seat) : '人工介入';

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-center gap-4 py-2 border-b border-border">
        <div className="flex flex-col items-center min-w-[72px]">
          <div className="w-10 h-10 bg-bg-elevated rounded-full border border-border flex items-center justify-center text-[11px] font-bold uppercase">
            {fromLabel.slice(0, 2)}
          </div>
          <span className="text-[10px] text-text-muted mt-1 font-bold tracking-wider">发起方</span>
          <span className="text-[11px] text-text-secondary mt-1 text-center break-all">{fromLabel}</span>
        </div>
        <ArrowRight size={18} className="text-text-muted" />
        <div className="flex flex-col items-center min-w-[72px]">
          <div className="w-10 h-10 bg-accent/20 rounded-full border border-accent/30 flex items-center justify-center text-[11px] font-bold uppercase text-accent">
            {toLabel.slice(0, 2)}
          </div>
          <span className="text-[10px] text-accent mt-1 font-bold tracking-wider">接收方</span>
          <span className="text-[11px] text-text-secondary mt-1 text-center break-all">{toLabel}</span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1">
          <div className="meta-label">交接状态</div>
          <div className="text-sm font-semibold text-text-primary">{getHandoffStatusLabel(handoff.status)}</div>
          <div className="text-xs text-text-secondary">创建于 {formatDateTimeZh(handoff.created_at)}</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1">
          <div className="meta-label">关联工作项</div>
          <div className="text-sm font-semibold text-text-primary">{workItem?.title || handoff.workitem_id}</div>
          <div className="text-xs text-text-secondary">{workItem ? getWorkItemStatusLabel(workItem.status) : '待从账本中恢复工作项状态'}</div>
        </div>
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.purpose}</h3>
        <p className="text-sm font-medium leading-7 text-text-primary whitespace-pre-wrap">{handoff.purpose}</p>
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.expected}</h3>
        <div className="text-sm text-text-secondary leading-7 p-4 bg-bg-elevated rounded-xl border border-border whitespace-pre-wrap">
          {handoff.expected_outcome}
        </div>
      </div>

      {workItem && (
        <div className="space-y-2">
          <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">工作项上下文</h3>
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="monospace text-[11px] font-semibold text-text-muted bg-bg-secondary px-2 py-0.5 rounded">{workItem.id.toUpperCase()}</span>
              <span className="text-[11px] text-text-secondary">{getWorkItemStatusLabel(workItem.status)}</span>
            </div>
            <div className="text-sm font-semibold text-text-primary">{workItem.title}</div>
            <p className="text-[12px] leading-6 text-text-secondary whitespace-pre-wrap">{workItem.goal || '该工作项暂未写入更长的目标说明。'}</p>
          </div>
        </div>
      )}

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.artifacts}</h3>
        {evidencePaths.length > 0 ? (
          <ul className="space-y-2">
            {evidencePaths.map((path) => (
              <li key={path} className="flex items-start gap-3 p-3 rounded-xl border border-border bg-bg-elevated">
                <FileText size={14} className="text-accent mt-0.5" />
                <div className="min-w-0">
                  <div className="text-xs font-semibold text-text-primary break-all">{path}</div>
                  <div className="text-[11px] text-text-secondary mt-1">这是与当前交接相关的可追溯产物或证据引用，可直接作为下一席位的接力入口。</div>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            当前交接没有单独附加产物；请结合关联工作项与最近事件理解上下文。
          </div>
        )}
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近状态变化</h3>
        {relatedEvents.length > 0 ? (
          <div className="space-y-2">
            {relatedEvents.map((event) => (
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
                <div className="text-xs text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                <div className="text-sm font-semibold text-text-primary leading-6">{event.payload?.title || event.payload?.summary || '已记录一条交接状态变化。'}</div>
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            当前没有额外的交接状态事件写回。
          </div>
        )}
      </div>

      {handoff.required_receipt && (
        <div className="flex items-start gap-3 p-4 bg-status-active/5 border border-status-active/20 rounded-xl text-status-active">
          <ReceiptText size={16} className="mt-0.5" />
          <div>
            <div className="text-sm font-bold flex items-center gap-2">
              <CheckCircle size={14} /> 需要回执确认
            </div>
            <div className="text-xs mt-1 leading-6 text-status-active/90">这意味着接收方必须显式确认已读取并理解交接要求，才能让协作链路继续向下推进。</div>
          </div>
        </div>
      )}
    </div>
  );
};

export default HandoffDetail;
