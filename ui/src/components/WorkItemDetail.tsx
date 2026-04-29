import React, { useMemo, useState } from 'react';
import { AlertCircle, ArrowRight, CheckCircle2, Link2, ReceiptText, ShieldCheck, Target, UserPlus } from 'lucide-react';
import { WorkItem } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import ArtifactChip from './ArtifactChip';
import ArtifactReferenceList from './ArtifactReferenceList';
import DelegationOverlay from './DelegationOverlay';
import {
  formatDateTimeZh,
  getHandoffStatusLabel,
  getPriorityLabel,
  getWorkItemStatusLabel,
} from '../utils/display';
import { dedupeArtifacts, getArtifactsForEvent, getUnmappedEvidencePaths } from '../utils/artifacts';

interface WorkItemDetailProps {
  workItem: WorkItem;
  onOpenArtifact: (artifactId: string) => void;
}

const getTierLabel = (tier: string) => {
  switch (tier) {
    case 'L1': return '微调 (L1)';
    case 'L2': return '语义变更 (L2)';
    case 'L3': return '规约变更 (L3)';
    default: return tier;
  }
};

const getAckModeLabel = (mode: string) => {
  switch (mode) {
    case 'DirectPatch': return '直接热修';
    case 'CompactAck': return '紧凑确认';
    case 'FullGate': return '全量网关';
    default: return mode;
  }
};

const WorkItemDetail: React.FC<WorkItemDetailProps> = ({ workItem, onOpenArtifact }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData, delegateWorkItem } = useDataStore();
  const [showDelegation, setShowDelegation] = useState(false);

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

  const relatedArtifacts = useMemo(() => {
    if (!currentData) return [];
    return dedupeArtifacts(relatedEvents.flatMap((event) => getArtifactsForEvent(currentData.artifacts, event)));
  }, [currentData, relatedEvents]);

  const unmappedEvidenceRefs = useMemo(() => {
    if (!currentData) return [];
    const refs = relatedEvents.flatMap((event) => event.evidence_refs);
    return Array.from(new Set(getUnmappedEvidencePaths(currentData.artifacts, refs))).slice(0, 5);
  }, [relatedEvents]);

  const changeRecord = workItem.change_tier_record;

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

      {owner && (
        <div className="flex justify-end -mt-2">
          <button 
            onClick={() => setShowDelegation(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-primary/20 bg-primary/5 text-primary text-[10px] font-black uppercase tracking-widest hover:bg-primary/10 transition-all shadow-sm"
          >
            <UserPlus size={12} />
            临时委派 (DELEGATE)
          </button>
        </div>
      )}

      {workItem.active_delegation && (
        <div className="rounded-xl border border-status-warning/30 bg-status-warning/5 p-4 space-y-3 animate-in slide-in-from-top-2 duration-300 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-status-warning text-surface text-[9px] font-black uppercase tracking-widest">Active Delegation</span>
              <span className="text-[10px] font-bold text-ink-soft">
                由 {typeof workItem.active_delegation.issuer === 'string' ? workItem.active_delegation.issuer : workItem.active_delegation.issuer.Seat} 签发
              </span>
            </div>
            <span className="text-[10px] font-black text-status-warning uppercase">有效期至: {workItem.active_delegation.expiry}</span>
          </div>
          <div className="space-y-1">
            <div className="text-[11px] font-bold text-ink leading-relaxed flex items-center gap-2">
              <ArrowRight size={12} className="text-status-warning" />
              责任已委派至: <span className="text-primary">{currentData?.seats.find(s => s.id === workItem.active_delegation?.delegate_seat_id)?.name || workItem.active_delegation.delegate_seat_id}</span>
            </div>
            <p className="text-[11px] text-ink-soft italic leading-relaxed pl-5">{workItem.active_delegation.scope}</p>
          </div>
        </div>
      )}

      {changeRecord && (
        <div className="rounded-xl border border-primary/20 bg-accent p-4 space-y-3 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className={`px-2 py-0.5 rounded text-[10px] font-black uppercase tracking-wider border ${
                changeRecord.tier === 'L3' ? 'bg-status-error/10 text-status-error border-status-error/20' : 'bg-primary/10 text-primary border-primary/20'
              }`}>
                {getTierLabel(changeRecord.tier)}
              </span>
              <span className="px-2 py-0.5 rounded bg-card border border-border/60 text-[10px] font-bold text-text-secondary shadow-sm">
                {getAckModeLabel(changeRecord.ack_mode)}
              </span>
              <span className={`px-2 py-0.5 rounded border border-border/60 bg-card text-[10px] font-bold shadow-sm ${
                changeRecord.impact_level === 'High' ? 'text-status-error' : 'text-text-secondary'
              }`}>
                影响：{changeRecord.impact_level}
              </span>
            </div>
            {changeRecord.tier === 'L3' && (
              <span className="flex items-center gap-1 text-[10px] font-black text-status-error uppercase tracking-tighter">
                <AlertCircle size={10} /> Full Gate Required
              </span>
            )}
          </div>
          
          <div className="space-y-1">
            <div className="text-[11px] font-bold text-text-muted uppercase tracking-wider">变更原因</div>
            <div className="text-sm text-text-primary leading-relaxed">{changeRecord.reason}</div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <div className="text-[10px] font-bold text-text-muted uppercase tracking-wider">评审人</div>
              <div className="text-[12px] text-text-secondary">{typeof changeRecord.reviewer === 'string' ? changeRecord.reviewer : changeRecord.reviewer.Seat}</div>
            </div>
            <div className="space-y-1">
              <div className="text-[10px] font-bold text-text-muted uppercase tracking-wider">执行人</div>
              <div className="text-[12px] text-text-secondary">{typeof changeRecord.executor === 'string' ? changeRecord.executor : changeRecord.executor.Seat}</div>
            </div>
          </div>

          <div className="space-y-1">
            <div className="text-[10px] font-bold text-text-muted uppercase tracking-wider">变更子句 / 范围</div>
            <div className="flex flex-wrap gap-1.5">
              {changeRecord.changed_clauses.map((clause, idx) => (
                <span key={idx} className="px-1.5 py-0.5 rounded bg-bg-secondary text-[10px] monospace text-text-secondary border border-border">
                  {clause}
                </span>
              ))}
            </div>
          </div>

          {changeRecord.evidence_refs.length > 0 && (
            <div className="space-y-1">
              <div className="text-[10px] font-bold text-text-muted uppercase tracking-wider">关联证据</div>
              <div className="text-[11px] text-text-muted truncate">{changeRecord.evidence_refs.join(' · ')}</div>
            </div>
          )}

          {changeRecord.tier === 'L2' && (
            <div className="mt-2 p-2 rounded border border-border bg-background/50 border-dashed">
              <div className="text-[10px] font-black uppercase text-text-muted mb-1">Compact Acknowledgment Preview</div>
              <div className="text-[10px] monospace text-text-secondary leading-relaxed">
                TaskRef: {workItem.id}<br/>
                Clauses: {changeRecord.changed_clauses.join(', ')}<br/>
                Status: {getWorkItemStatusLabel(workItem.status)}
              </div>
            </div>
          )}
        </div>
      )}

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
                {relatedHandoffs.map((handoff) => {
                  return (
                    <div key={handoff.id} className="text-[12px] leading-6 text-text-secondary border-l-2 border-border pl-3">
                      <div className="font-semibold text-text-primary">{handoff.purpose}</div>
                      <div className="text-text-muted mt-1">{handoff.id.toUpperCase()} · {getHandoffStatusLabel(handoff.status)} · {formatDateTimeZh(handoff.sent_at || handoff.created_at)}</div>
                    </div>
                  );
                })}
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
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
                <div className="text-xs text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                <div className="text-sm font-semibold text-text-primary leading-6">{event.payload?.title || event.payload?.summary || '已记录一条工作项状态更新。'}</div>
                {getArtifactsForEvent(currentData?.artifacts || [], event).length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5">
                    {getArtifactsForEvent(currentData?.artifacts || [], event).map((artifact) => (
                      <ArtifactChip key={`${event.event_id}-${artifact.id}`} artifact={artifact} onClick={() => onOpenArtifact(artifact.id)} />
                    ))}
                  </div>
                )}
                {getUnmappedEvidencePaths(currentData?.artifacts || [], event.evidence_refs).length > 0 && (
                  <div className="text-[11px] text-status-warning break-all">
                    未映射证据：{getUnmappedEvidencePaths(currentData?.artifacts || [], event.evidence_refs).join(' · ')}
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            该工作项目前还没有独立的事件写回。
          </div>
        )}

        {(relatedArtifacts.length > 0 || unmappedEvidenceRefs.length > 0) && (
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
            <div className="text-[11px] font-semibold text-text-primary">重点证据对象</div>
            <ArtifactReferenceList
              artifacts={relatedArtifacts}
              onOpenArtifact={onOpenArtifact}
              emptyText="当前还没有建立对象化证据。"
              unmappedRefs={unmappedEvidenceRefs}
            />
          </div>
        )}
      </div>

      {owner && (
        <DelegationOverlay 
          workItem={workItem}
          sourceSeat={owner}
          isOpen={showDelegation}
          onClose={() => setShowDelegation(false)}
          onConfirm={(delegation) => {
            delegateWorkItem(workItem.id, delegation);
            setShowDelegation(false);
          }}
        />
      )}
    </div>
  );
};

export default WorkItemDetail;
