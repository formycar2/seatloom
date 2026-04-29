import React, { useMemo } from 'react';
import { ArrowRight, CheckCircle, Circle, ReceiptText } from 'lucide-react';
import { Handoff } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import ArtifactChip from './ArtifactChip';
import ArtifactReferenceList from './ArtifactReferenceList';
import {
  formatDateTimeZh,
  getHandoffStatusLabel,
  getWorkItemStatusLabel,
} from '../utils/display';
import {
  dedupeArtifacts,
  getArtifactsFromIds,
  getArtifactsForEvent,
  getUnmappedArtifactIds,
  getUnmappedEvidencePaths,
} from '../utils/artifacts';

interface HandoffDetailProps {
  handoff: Handoff;
  onOpenArtifact: (artifactId: string) => void;
}

const LIFECYCLE_STAGES = [
  { key: 'sent', label: '已发出', status: ['Sent'] },
  { key: 'accepted', label: '已接收', status: ['Received', 'Accepted'] },
  { key: 'working', label: '处理中', status: ['Working'] },
  { key: 'completed', label: '已完成', status: ['Completed'] },
  { key: 'returned', label: '已退回', status: ['Returned'] },
] as const;

const stageDescriptions: Record<string, string> = {
  sent: '交接单已由发起方发出，正在等待接收方确认。',
  accepted: '接收方已确认交接内容，准备开始执行。',
  working: '接收方正在根据交接要求执行相关工作。',
  completed: '交接工作已顺利完成并达成预期交付。',
  returned: '交接内容存在疑问或交付不达标，已被退回修正。',
};

const HandoffDetail: React.FC<HandoffDetailProps> = ({ handoff, onOpenArtifact }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const workItem = currentData?.workItems.find((item) => item.id === handoff.workitem_id) || null;
  const relatedEvents = (currentData?.events || [])
    .filter((event) => event.object_refs.some((ref) => ('Handoff' in ref && ref.Handoff === handoff.id) || ('WorkItem' in ref && ref.WorkItem === handoff.workitem_id)))
    .sort((left, right) => new Date(right.occurred_at).getTime() - new Date(left.occurred_at).getTime())
    .slice(0, 4);

  const relatedArtifacts = useMemo(() => {
    if (!currentData) return [];
    const fromHandoff = getArtifactsFromIds(currentData.artifacts, handoff.artifact_ids);
    const fromEvents = relatedEvents.flatMap((event) => getArtifactsForEvent(currentData.artifacts, event));
    return dedupeArtifacts([...fromHandoff, ...fromEvents]);
  }, [currentData, handoff.artifact_ids, relatedEvents]);

  const unmappedRefs = useMemo(() => {
    if (!currentData) return [];
    const eventRefs = relatedEvents.flatMap((event) => event.evidence_refs);
    return Array.from(
      new Set([
        ...getUnmappedArtifactIds(currentData.artifacts, handoff.artifact_ids),
        ...getUnmappedEvidencePaths(currentData.artifacts, eventRefs),
      ]),
    );
  }, [currentData, handoff.artifact_ids, relatedEvents]);

  const getSeatName = (seatId: string) => currentData?.seats.find((seat) => seat.id === seatId)?.name || seatId;
  const fromLabel = typeof handoff.from_ref === 'object' && 'Seat' in handoff.from_ref ? getSeatName(handoff.from_ref.Seat) : '人工介入';
  const toLabel = typeof handoff.to_ref === 'object' && 'Seat' in handoff.to_ref ? getSeatName(handoff.to_ref.Seat) : '人工介入';

  const currentStage = LIFECYCLE_STAGES.find((s) => (s.status as readonly string[]).includes(handoff.status));
  const currentStageIndex = currentStage ? LIFECYCLE_STAGES.indexOf(currentStage) : -1;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-center gap-4 py-2 border-b border-border/60">
        <div className="flex flex-col items-center min-w-[72px]">
          <div className="w-10 h-10 bg-secondary rounded-full border border-border flex items-center justify-center text-[11px] font-bold uppercase text-text-secondary">
            {fromLabel.slice(0, 2)}
          </div>
          <span className="text-[10px] text-text-muted mt-1 font-bold tracking-wider">发起方</span>
          <span className="text-[11px] text-text-secondary mt-1 text-center break-all">{fromLabel}</span>
        </div>
        <ArrowRight size={18} className="text-border" />
        <div className="flex flex-col items-center min-w-[72px]">
          <div className="w-10 h-10 bg-accent rounded-full border border-primary/20 flex items-center justify-center text-[11px] font-bold uppercase text-primary">
            {toLabel.slice(0, 2)}
          </div>
          <span className="text-[10px] text-primary/80 mt-1 font-bold tracking-wider">接收方</span>
          <span className="text-[11px] text-text-secondary mt-1 text-center break-all">{toLabel}</span>
        </div>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between px-2">
          {LIFECYCLE_STAGES.map((stage, index) => {
            const isCompleted = index < currentStageIndex;
            const isActive = index === currentStageIndex;
            const isReturned = stage.key === 'returned' && (isActive || isCompleted);
            
            let colorClass = 'text-text-muted';
            let circleClass = 'border-border bg-background';
            
            if (isActive) {
              colorClass = isReturned ? 'text-status-error font-bold' : 'text-primary font-bold';
              circleClass = isReturned ? 'border-status-error bg-status-error/10 text-status-error' : 'border-primary bg-primary/10 text-primary';
            } else if (isCompleted) {
              colorClass = 'text-status-active font-semibold';
              circleClass = 'border-status-active/30 bg-status-active/10 text-status-active';
            }

            return (
              <React.Fragment key={stage.key}>
                <div className="flex flex-col items-center gap-2 relative">
                  <div className={`w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all ${circleClass}`}>
                    {isCompleted ? <CheckCircle size={14} /> : <div className={`w-1.5 h-1.5 rounded-full ${isActive ? 'bg-current' : 'bg-transparent'}`} />}
                  </div>
                  <span className={`text-[10px] tracking-tighter whitespace-nowrap transition-all ${colorClass}`}>{stage.label}</span>
                </div>
                {index < LIFECYCLE_STAGES.length - 1 && (
                  <div className={`flex-1 h-[2px] mt-3 -mx-2 transition-all ${index < currentStageIndex ? 'bg-status-done' : 'bg-border'}`} />
                )}
              </React.Fragment>
            );
          })}
        </div>
        {currentStage && (
          <div className="text-[11px] text-text-secondary bg-bg-secondary/50 p-2 rounded-lg text-center italic">
            {stageDescriptions[currentStage.key]}
          </div>
        )}
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
        <ArtifactReferenceList
          artifacts={relatedArtifacts}
          onOpenArtifact={onOpenArtifact}
          emptyText="当前交接没有单独附加产物；请结合关联工作项与最近事件理解上下文。"
          unmappedRefs={unmappedRefs}
        />
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近状态变化</h3>
        {relatedEvents.length > 0 ? (
          <div className="space-y-2">
            {relatedEvents.map((event) => (
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
                <div className="text-xs text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                <div className="text-sm font-semibold text-text-primary leading-6">{event.payload?.title || event.payload?.summary || '已记录一条交接状态变化。'}</div>
                {getArtifactsForEvent(currentData?.artifacts || [], event).length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5">
                    {getArtifactsForEvent(currentData?.artifacts || [], event).map((artifact) => (
                      <ArtifactChip key={`${event.event_id}-${artifact.id}`} artifact={artifact} onClick={() => onOpenArtifact(artifact.id)} />
                    ))}
                  </div>
                )}
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

