import React, { useMemo, useState } from 'react';
import {
  AlertTriangle,
  BookMarked,
  Clock3,
  Coins,
  Cpu,
  FileText,
  FastForward,
  Fingerprint,
  FolderTree,
  GitBranch,
  RefreshCcw,
  ShieldAlert,
  ShieldCheck,
  Terminal,
  User,
  Zap,
} from 'lucide-react';
import { Session } from '../types';
import SwitchRuntimeDialog from './SwitchRuntimeDialog';
import { useDataStore } from '../stores/useDataStore';
import {
  formatDateTimeZh,
  getRuntimeLabel,
  getSessionStatusLabel,
} from '../utils/display';
import { dedupeArtifacts, getArtifactsForEvent, getUnmappedEvidencePaths } from '../utils/artifacts';
import ArtifactChip from './ArtifactChip';
import ArtifactReferenceList from './ArtifactReferenceList';

interface SessionDetailProps {
  session: Session;
  onOpenArtifact: (artifactId: string) => void;
}

const SessionDetail: React.FC<SessionDetailProps> = ({ session, onOpenArtifact }) => {
  const { activeProjectId, projectData } = useDataStore();
  const [showSwitch, setShowSwitch] = useState(false);

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seat = currentData?.seats.find((item) => item.id === session.seat_id) || null;

  const relatedWorkItems = useMemo(() => {
    if (!currentData) return [];
    const linkedIds = new Set<string>();

    currentData.events.forEach((event) => {
      const hitSession = event.object_refs.some((ref) => 'Session' in ref && ref.Session === session.id);
      if (!hitSession) return;
      event.object_refs.forEach((ref) => {
        if ('WorkItem' in ref) linkedIds.add(ref.WorkItem);
      });
    });

    if (linkedIds.size === 0) {
      currentData.workItems
        .filter((workItem) => workItem.owner_seat_id === session.seat_id)
        .forEach((workItem) => linkedIds.add(workItem.id));
    }

    return currentData.workItems.filter((workItem) => linkedIds.has(workItem.id));
  }, [currentData, session.id, session.seat_id]);

  const relatedEvents = useMemo(() => {
    if (!currentData) return [];
    return currentData.events
      .filter((event) => event.object_refs.some((ref) => 'Session' in ref && ref.Session === session.id))
      .sort((left, right) => new Date(right.occurred_at).getTime() - new Date(left.occurred_at).getTime())
      .slice(0, 4);
  }, [currentData, session.id]);

  const relatedArtifacts = useMemo(() => {
    if (!currentData) return [];
    return dedupeArtifacts(relatedEvents.flatMap((event) => getArtifactsForEvent(currentData.artifacts, event)));
  }, [currentData, relatedEvents]);

  const unmappedEvidenceRefs = useMemo(() => {
    if (!currentData) return [];
    const refs = relatedEvents.flatMap((event) => event.evidence_refs);
    return Array.from(new Set(getUnmappedEvidencePaths(currentData.artifacts, refs))).slice(0, 5);
  }, [currentData, relatedEvents]);

  const runtimeName = getRuntimeLabel(session.runtime);
  const isInterrupted = session.status === 'Interrupted';
  const isRunning = session.status === 'Running' || session.status === 'InputRequired';
  const promptState = session.prompt_state;
  const isBlocked = session.status === 'InputRequired' || !!promptState;

  const assistDisabled = promptState?.classification === 'sensitive' || promptState?.policy === 'human_required';

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      {isBlocked && (
        <div className="rounded-xl border border-status-warning/30 bg-status-warning/5 overflow-hidden shadow-sm">
          <div className="bg-status-warning/10 border-b border-status-warning/20 px-4 py-2 flex items-center justify-between text-status-warning">
            <div className="flex items-center gap-2">
              <ShieldAlert size={14} />
              <span className="text-[11px] font-black uppercase tracking-widest">Prompt Blocked</span>
            </div>
            <div className="text-[10px] monospace opacity-90">{runtimeName} · {session.id}</div>
          </div>
          <div className="p-4 space-y-4">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="px-2 py-0.5 rounded bg-status-warning/20 border border-status-warning/30 text-[10px] font-bold text-status-warning">
                {promptState?.classification || 'uncertain'}
              </span>
              <span className="px-2 py-0.5 rounded bg-bg-secondary border border-border text-[10px] font-bold text-text-secondary">
                {promptState?.policy || 'needs_approval'}
              </span>
              {promptState?.step_count !== undefined && (
                <span className="text-[10px] text-text-muted">步骤: {promptState.step_count}</span>
              )}
              {promptState?.token_budget !== undefined && (
                <span className="text-[10px] text-text-muted">预算: {promptState.token_budget}</span>
              )}
            </div>

            {promptState?.preview && (
              <div className="rounded-lg bg-[var(--terminal-preview)] p-3 monospace text-[10px] leading-relaxed text-green-400/90 overflow-hidden max-h-[120px] border border-border/10">
                {promptState.preview}
              </div>
            )}

            {promptState?.expected_next && (
              <div className="text-[11px] text-text-secondary italic">
                预期下一步：{promptState.expected_next}
              </div>
            )}

            <div className="grid grid-cols-2 gap-2">
              <button className="px-3 py-2 bg-status-active text-white rounded text-[10px] font-black tracking-widest hover:brightness-110 transition-all">确认 (APPROVE)</button>
              <button className="px-3 py-2 bg-bg-secondary border border-border text-text-primary rounded text-[10px] font-black tracking-widest hover:bg-secondary/60 hover:text-primary transition-all">人工接管 (TAKEOVER)</button>
              <button 
                disabled={assistDisabled}
                title={assistDisabled ? (promptState?.classification === 'sensitive' ? '检测到敏感信息，禁止 Supervisor 辅助' : '当前策略要求人工介入') : '通过 Supervisor 辅助处理此 Prompt'}
                className="px-3 py-2 bg-primary text-primary-foreground rounded text-[10px] font-black tracking-widest hover:brightness-110 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
              >
                SUPERVISOR 辅助
              </button>
              <button className="px-3 py-2 bg-status-error text-white rounded text-[10px] font-black tracking-widest hover:brightness-110 transition-all">停止 (STOP)</button>
            </div>
            {assistDisabled && (
              <p className="text-[10px] text-status-error italic text-center font-bold">
                {promptState?.classification === 'sensitive' ? '! 检测到敏感信息，禁止 Supervisor 辅助。' : '! 当前策略要求人工介入。'}
              </p>
            )}
          </div>
        </div>
      )}

      <div className="flex items-start gap-3">
        <div className={`p-3 rounded-lg shadow-sm ${isInterrupted ? 'bg-status-warning/10 text-status-warning' : 'bg-primary/10 text-primary'}`}>
          <Terminal size={24} />
        </div>
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-bold leading-tight monospace text-text-primary">{session.id}</h2>
              <span className={`text-[10px] px-2 py-0.5 rounded-full border font-bold tracking-wider ${
                session.status === 'Running'
                  ? 'bg-status-active/10 text-status-active border-status-active/20'
                  : session.status === 'InputRequired'
                    ? 'bg-status-warning/10 text-status-warning border-status-warning/20'
                    : session.status === 'Interrupted'
                      ? 'bg-status-error/10 text-status-error border-status-error/20'
                      : 'bg-bg-secondary text-text-muted border-border'
              }`}>
                {getSessionStatusLabel(session.status).toUpperCase()}
              </span>
            </div>
            <div className="text-xs text-text-muted mt-1 flex items-center gap-2">
              <span className="font-semibold text-text-secondary">{runtimeName}</span>
              <span>·</span>
              <span>{session.branch || 'no branch'}</span>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1 shadow-sm">
          <div className="meta-label">启动时间</div>
          <div className="text-sm font-semibold text-text-primary">{formatDateTimeZh(session.created_at)}</div>
          <div className="text-[10px] text-text-muted">PID: {session.pid || 'N/A'}</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1 shadow-sm">
          <div className="meta-label">所属席位</div>
          <div className="text-sm font-semibold text-text-primary">{seat?.name || session.seat_id}</div>
          <div className="text-[10px] text-text-muted">{seat?.role ? (typeof seat.role === 'string' ? seat.role : seat.role.Custom) : 'Unknown Role'}</div>
        </div>
      </div>

      <div className="space-y-2">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <FileText size={12} /> 工作目录
        </h3>
        <div className="p-3 rounded-lg bg-bg-secondary border border-border monospace text-xs text-text-secondary break-all">
          {session.workspace_path}
        </div>
      </div>

      {relatedWorkItems.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">关联工作项</h3>
          <div className="space-y-2">
            {relatedWorkItems.map((workItem) => (
              <div key={workItem.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5 shadow-sm">
                <div className="flex items-center gap-2">
                  <span className="monospace text-[11px] font-bold text-primary bg-primary/10 px-1.5 py-0.5 rounded">{workItem.id.toUpperCase()}</span>
                  <div className="text-sm font-bold text-text-primary truncate">{workItem.title}</div>
                </div>
                <p className="text-[11px] text-text-secondary leading-relaxed line-clamp-2">{workItem.goal}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {relatedEvents.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近活动事件</h3>
          <div className="space-y-3">
            {relatedEvents.map((event) => (
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
                <div className="flex items-center justify-between">
                  <div className="text-[10px] font-bold text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                  <div className="text-[10px] font-black uppercase text-primary/60">{event.event_type}</div>
                </div>
                <div className="text-[13px] font-medium text-text-primary leading-relaxed">{event.payload?.title || event.payload?.summary || 'Session activity recorded.'}</div>
                {getArtifactsForEvent(currentData?.artifacts || [], event).length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5 pt-1">
                    {getArtifactsForEvent(currentData?.artifacts || [], event).map((artifact) => (
                      <ArtifactChip key={`${event.event_id}-${artifact.id}`} artifact={artifact} onClick={() => onOpenArtifact(artifact.id)} />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {isInterrupted && (
        <div className="rounded-xl border border-status-warning/30 bg-status-warning/5 p-4 space-y-3 shadow-sm">
          <div className="flex items-start gap-3 text-status-warning">
            <AlertTriangle size={18} className="shrink-0 mt-0.5" />
            <div className="space-y-1">
              <div className="text-sm font-bold">会话已中断 (Session Interrupted)</div>
              <p className="text-xs leading-relaxed opacity-90">
                该会话可能因为 Token 额度耗尽、超时或手动挂起而进入中断状态。你可以通过切换运行时并回灌上下文来尝试恢复。
              </p>
            </div>
          </div>
          <button
            onClick={() => setShowSwitch(true)}
            className="w-full py-2 bg-status-warning text-white rounded-lg text-[10px] font-black tracking-widest hover:brightness-110 shadow-lg shadow-status-warning/20 transition-all"
          >
            尝试执行会话恢复
          </button>
        </div>
      )}

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{session.continuity_pack ? '启动包内容' : '证据链记录'}</h3>
        <ArtifactReferenceList
          artifacts={relatedArtifacts}
          onOpenArtifact={onOpenArtifact}
          emptyText="该会话目前尚未产生持久化证据对象。"
          unmappedRefs={unmappedEvidenceRefs}
        />
      </div>

      {isRunning && (
        <div className="pt-4 border-t border-border space-y-3">
          <div className="rounded-xl border border-border bg-bg-elevated p-4 text-sm leading-6 text-text-secondary shadow-sm">
            当前会话仍可继续推进；如果需要切换工具或替换席位，可以先生成中文启动包，再把上下文交给新的运行时承接。
          </div>
          <button
            onClick={() => setShowSwitch(true)}
            className="w-full py-2.5 bg-primary/10 text-primary border border-primary/20 rounded-lg text-xs font-bold tracking-widest hover:bg-primary hover:text-primary-foreground transition-all shadow-sm"
          >
            切换运行时并生成启动包
          </button>
        </div>
      )}

      {session.continuity_pack && (
        <div className="rounded-2xl border border-primary/20 bg-primary/5 overflow-hidden shadow-sm animate-in zoom-in-95 duration-300">
          <div className="bg-primary/10 border-b border-primary/20 px-4 py-2.5 flex items-center justify-between text-primary">
            <div className="flex items-center gap-2">
              <FastForward size={16} />
              <span className="text-[11px] font-black uppercase tracking-widest">Continuity Pack Preview · 启动包预览</span>
            </div>
            <div className="text-[10px] font-bold opacity-90">V0.5 PROTOCOL</div>
          </div>
          
          <div className="p-4 space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div className="space-y-1.5">
                <div className="meta-label text-primary/80 flex items-center gap-1"><Fingerprint size={10} /> Tier 0: 身份</div>
                <div className="p-2 rounded-lg bg-card border border-primary/10 text-[11px] space-y-1 monospace shadow-sm">
                  <div className="font-bold text-text-primary">{session.continuity_pack.tier_0_identity.seat_id}</div>
                  <div className="text-text-secondary text-[10px]">{getRuntimeLabel(session.continuity_pack.tier_0_identity.runtime)}</div>
                  {session.continuity_pack.tier_0_identity.branch && <div className="text-text-muted text-[10px]">分支: {session.continuity_pack.tier_0_identity.branch}</div>}
                </div>
              </div>
              <div className="space-y-1.5">
                <div className="meta-label text-primary/80 flex items-center gap-1"><FolderTree size={10} /> Tier 1: 状态</div>
                <div className="p-2 rounded-lg bg-card border border-primary/10 text-[11px] space-y-1 shadow-sm">
                  {session.continuity_pack.tier_1_state.ac_progress.length > 0 ? (
                    <div className="text-text-primary truncate font-medium">AC 进度: {session.continuity_pack.tier_1_state.ac_progress.length} 项</div>
                  ) : <div className="text-text-muted italic text-[10px]">状态规约未记录</div>}
                  {session.continuity_pack.tier_1_state.current_blocker && (
                    <div className="text-status-error text-[10px] font-bold">Blocker: {session.continuity_pack.tier_1_state.current_blocker}</div>
                  )}
                </div>
              </div>
              <div className="space-y-1.5">
                <div className="meta-label text-primary/80 flex items-center gap-1"><ShieldCheck size={10} /> Tier 2: 决策</div>
                <div className="p-2 rounded-lg bg-card border border-primary/10 text-[11px] space-y-1 shadow-sm">
                  <div className="text-text-secondary line-clamp-2 leading-relaxed text-[10px]">
                    {session.continuity_pack.tier_2_decisions.summary || '未记录关键决策摘要'}
                  </div>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-primary/10">
              <div className="space-y-2">
                <div className="meta-label flex items-center gap-1.5 text-text-secondary font-bold uppercase"><Zap size={11} className="text-primary" /> Seat Skills · 席位技能</div>
                {session.continuity_pack.seat_skills && session.continuity_pack.seat_skills.length > 0 ? (
                  <div className="flex flex-wrap gap-1.5">
                    {session.continuity_pack.seat_skills.map(skill => (
                      <span key={skill} className="px-2 py-0.5 rounded bg-primary/10 text-primary border border-primary/10 text-[10px] font-bold">{skill}</span>
                    ))}
                  </div>
                ) : <div className="text-[10px] text-text-muted italic">未挂载专用技能包</div>}
              </div>
              <div className="space-y-2">
                <div className="meta-label flex items-center gap-1.5 text-text-secondary font-bold uppercase"><BookMarked size={11} className="text-primary" /> Playbooks · 剧本匹配</div>
                {session.continuity_pack.playbook_matches && session.continuity_pack.playbook_matches.length > 0 ? (
                  <div className="flex flex-wrap gap-1.5">
                    {session.continuity_pack.playbook_matches.map(playbook => (
                      <span key={playbook} className="px-2 py-0.5 rounded bg-bg-secondary text-text-secondary border border-border text-[10px] font-bold">{playbook}</span>
                    ))}
                  </div>
                ) : <div className="text-[10px] text-text-muted italic">未匹配到标准化剧本</div>}
              </div>
            </div>

            <div className="pt-2 border-t border-primary/10 flex items-center justify-between">
              <div className="space-y-1">
                <div className="meta-label flex items-center gap-1.5 text-text-secondary font-bold uppercase"><Coins size={11} className="text-primary" /> Budget Estimate · 预算预估</div>
                <div className="flex items-center gap-3">
                  <div className="w-32 h-1.5 bg-bg-secondary rounded-full overflow-hidden border border-border">
                    <div className="h-full bg-primary" style={{ width: '45%' }} />
                  </div>
                  <span className="text-[11px] font-black text-primary">~{session.continuity_pack.budget_estimate} tokens</span>
                </div>
              </div>
              <div className="text-right space-y-1">
                <div className="meta-label font-bold uppercase">Fallback · 回退路径</div>
                <div className="text-[11px] font-bold text-status-warning monospace">{session.continuity_pack.fallback_path}</div>
              </div>
            </div>

            <p className="text-[10px] text-text-secondary leading-relaxed p-2 bg-background/50 rounded border border-dashed border-primary/20">
              启动包预览仅展示即将注入新运行时的真值上下文。确认无误后，新席位将以此为起点接力工作。
            </p>
          </div>
        </div>
      )}

      <SwitchRuntimeDialog isOpen={showSwitch} onClose={() => setShowSwitch(false)} session={session} />
    </div>
  );
};

export default SessionDetail;
