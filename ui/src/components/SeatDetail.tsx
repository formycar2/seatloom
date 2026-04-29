import React from 'react';
import {
  Activity,
  AlertCircle,
  ArrowRight,
  BookMarked,
  BriefcaseBusiness,
  CheckCircle2,
  Circle,
  Clock3,
  Coins,
  Cpu,
  Fingerprint,
  GitBranch,
  Lock,
  Puzzle,
  Send,
  User,
  UserPlus,
  Zap,
} from 'lucide-react';
import { Seat } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import {
  formatDateTimeZh,
  getHandoffStatusLabel,
  getPriorityLabel,
  getRuntimeLabel,
  getSeatRoleLabel,
  getSeatStatusLabel,
  getSessionStatusLabel,
  getWorkItemStatusLabel,
} from '../utils/display';

interface SeatDetailProps {
  seat: Seat;
}

const SeatDetail: React.FC<SeatDetailProps> = ({ seat }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seatSessions = (currentData?.sessions || [])
    .filter((session) => session.seat_id === seat.id)
    .sort((left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime());
  
  const activeSessions = seatSessions.filter((session) => session.status === 'Running' || session.status === 'InputRequired');
  
  const ownedWorkItems = (currentData?.workItems || [])
    .filter((workItem) => workItem.owner_seat_id === seat.id)
    .sort((left, right) => new Date(right.updated_at).getTime() - new Date(left.updated_at).getTime());
  
  const delegatedWorkItems = (currentData?.workItems || [])
    .filter((workItem) => workItem.active_delegation?.delegate_seat_id === seat.id);

  const relatedHandoffs = (currentData?.handoffs || [])
    .filter((handoff) => {
      const fromSeat = typeof handoff.from_ref === 'object' && 'Seat' in handoff.from_ref ? handoff.from_ref.Seat : null;
      const toSeat = typeof handoff.to_ref === 'object' && 'Seat' in handoff.to_ref ? handoff.to_ref.Seat : null;
      return fromSeat === seat.id || toSeat === seat.id;
    })
    .sort((left, right) => new Date(right.sent_at || right.created_at).getTime() - new Date(left.sent_at || left.created_at).getTime())
    .slice(0, 4);

  const getSeatName = (seatId: string) => currentData?.seats.find((item) => item.id === seatId)?.name || seatId;

  const hasCapabilities = seat.capabilities && seat.capabilities.length > 0;
  const hasInputs = seat.accepted_input_types && seat.accepted_input_types.length > 0;
  const hasOutputs = seat.output_types && seat.output_types.length > 0;
  const hasBudgets = seat.input_budget !== undefined || seat.output_budget !== undefined;

  const isAssignable = hasCapabilities && hasInputs && hasOutputs && hasBudgets;
  const isUnderSpecified = !hasCapabilities || !hasInputs || !hasOutputs;
  const isBudgetConstrained = hasBudgets && ((seat.input_budget || 0) <= 0 || (seat.output_budget || 0) <= 0);

  let statusBadge = { label: '可分配 (Assignable)', color: 'bg-status-active/10 text-status-active border-status-active/20' };
  if (isUnderSpecified) statusBadge = { label: '定义不全 (Under-specified)', color: 'bg-status-warning/10 text-status-warning border-status-warning/20' };
  if (isBudgetConstrained) statusBadge = { label: '预算受限 (Budget-constrained)', color: 'bg-status-error/10 text-status-error border-status-error/20' };

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="rounded-2xl border border-border bg-card overflow-hidden shadow-sm">
        <div className="bg-primary/5 px-4 py-4 border-b border-border flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-primary/10 text-primary rounded-xl flex items-center justify-center border border-primary/20">
              <Fingerprint size={28} />
            </div>
            <div>
              <h2 className="text-xl font-bold text-text-primary leading-tight">{seat.name}</h2>
              <div className="text-xs text-text-muted mt-1 font-mono uppercase tracking-widest">{seat.id}</div>
            </div>
          </div>
          <span className={`px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border ${statusBadge.color}`}>
            {statusBadge.label}
          </span>
        </div>

        <div className="p-4 space-y-5">
          {delegatedWorkItems.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-[11px] font-bold text-status-warning uppercase tracking-wider flex items-center gap-1.5">
                <UserPlus size={12} /> 活动中的受托任务 (ACTIVE DELEGATIONS)
              </h3>
              <div className="space-y-2">
                {delegatedWorkItems.map(wi => (
                  <div key={`delegated-${wi.id}`} className="p-4 rounded-xl border border-status-warning/30 bg-status-warning/5 space-y-2 shadow-sm">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="monospace text-[10px] font-black bg-status-warning text-surface px-1.5 py-0.5 rounded uppercase">{wi.id}</span>
                        <span className="text-[11px] font-bold text-ink">代 {currentData?.seats.find(s => s.id === wi.active_delegation?.source_seat_id)?.name || wi.active_delegation?.source_seat_id} 执行</span>
                      </div>
                      <span className="text-[10px] font-black text-status-warning uppercase">有效期至: {wi.active_delegation?.expiry}</span>
                    </div>
                    <p className="text-[11px] text-ink-soft leading-relaxed italic">{wi.active_delegation?.scope}</p>
                    <div className="text-[9px] font-bold text-ink-faint uppercase">
                      签发人: {typeof wi.active_delegation?.issuer === 'string' ? wi.active_delegation.issuer : wi.active_delegation?.issuer.Seat}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-xl border border-border bg-background p-3 space-y-1">
              <div className="meta-label flex items-center gap-1.5"><Puzzle size={11} /> 职责角色</div>
              <div className="font-semibold text-text-primary">{getSeatRoleLabel(seat.role)}</div>
              <div className="text-[10px] text-text-secondary">项目绑定角色，决定权限范围。</div>
            </div>
            <div className="rounded-xl border border-border bg-background p-3 space-y-1">
              <div className="meta-label flex items-center gap-1.5"><Activity size={11} /> 当前状态</div>
              <div className="font-semibold text-text-primary">{getSeatStatusLabel(seat.status)}</div>
              <div className="text-[10px] text-text-secondary">创建于 {formatDateTimeZh(seat.created_at)}</div>
            </div>
          </div>

          <div className="space-y-3">
            <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
              <Zap size={12} /> Seat Card · 能力与真值
            </h3>
            
            <div className="space-y-3 bg-background rounded-xl border border-border p-4">
              <div className="space-y-2">
                <div className="meta-label">能力集 (Capabilities)</div>
                {hasCapabilities ? (
                  <div className="flex flex-wrap gap-1.5">
                    {seat.capabilities?.map((cap) => (
                      <span key={cap} className="px-2 py-0.5 rounded-full bg-primary/5 text-primary border border-primary/10 text-[10px] font-bold">
                        {cap}
                      </span>
                    ))}
                  </div>
                ) : (
                  <div className="flex items-center gap-1.5 text-[11px] text-status-warning italic">
                    <AlertCircle size={10} /> 尚未定义能力标签，路由引擎无法进行精准匹配。
                  </div>
                )}
              </div>

              <div className="grid grid-cols-2 gap-4 pt-2 border-t border-border/50">
                <div className="space-y-2">
                  <div className="meta-label">输入规约</div>
                  {hasInputs ? (
                    <div className="text-[11px] text-text-primary font-medium">{seat.accepted_input_types?.join(', ')}</div>
                  ) : (
                    <div className="text-[10px] text-text-muted italic">未记录接受的输入类型。</div>
                  )}
                </div>
                <div className="space-y-2">
                  <div className="meta-label">输出规约</div>
                  {hasOutputs ? (
                    <div className="text-[11px] text-text-primary font-medium">{seat.output_types?.join(', ')}</div>
                  ) : (
                    <div className="text-[10px] text-text-muted italic">未记录产出的输出类型。</div>
                  )}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 pt-2 border-t border-border/50">
                <div className="space-y-2">
                  <div className="meta-label flex items-center gap-1.5"><Coins size={10} /> 输入预算</div>
                  <div className="text-[11px] font-bold text-text-primary">{seat.input_budget !== undefined ? `${seat.input_budget} tokens` : '无限制'}</div>
                </div>
                <div className="space-y-2">
                  <div className="meta-label flex items-center gap-1.5"><Coins size={10} /> 输出预算</div>
                  <div className="text-[11px] font-bold text-text-primary">{seat.output_budget !== undefined ? `${seat.output_budget} tokens` : '无限制'}</div>
                </div>
              </div>

              <div className="space-y-2 pt-2 border-t border-border/50">
                <div className="meta-label flex items-center gap-1.5"><Lock size={10} /> 显式约束</div>
                <div className="text-[11px] text-text-secondary leading-relaxed">
                  {seat.constraints && seat.constraints.length > 0 
                    ? seat.constraints.join('；') 
                    : '当前席位没有设置硬性的执行约束。'}
                </div>
              </div>

              <div className="space-y-2 pt-2 border-t border-border/50">
                <div className="meta-label flex items-center gap-1.5"><BookMarked size={10} /> 挂载的 Seat Skill</div>
                {seat.attached_skills && seat.attached_skills.length > 0 ? (
                  <div className="flex flex-wrap gap-1.5">
                    {seat.attached_skills.map((skill) => (
                      <span key={skill} className="px-2 py-0.5 rounded bg-bg-secondary text-[10px] font-semibold text-text-secondary border border-border">
                        {skill}
                      </span>
                    ))}
                  </div>
                ) : (
                  <div className="text-[10px] text-text-muted italic">当前席位没有关联的人工编写的技能指南。</div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.activeSessions}</h3>
        {seatSessions.length > 0 ? (
          <div className="space-y-3">
            {seatSessions.map((session) => (
              <div key={session.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
                <div className="flex items-start justify-between gap-3 flex-wrap">
                  <div>
                    <div className="text-sm font-semibold monospace text-text-primary">{session.id}</div>
                    <div className="text-xs text-text-secondary mt-1 flex items-center gap-2 flex-wrap">
                      <span>{getRuntimeLabel(session.runtime)}</span>
                      <span>·</span>
                      <span>{session.branch || '未记录分支'}</span>
                    </div>
                  </div>
                  <span className={`text-[10px] px-2 py-1 rounded-full border font-semibold ${
                    session.status === 'Running'
                      ? 'bg-status-active/10 text-status-active border-status-active/20'
                      : session.status === 'Interrupted'
                        ? 'bg-status-warning/10 text-status-warning border-status-warning/20'
                        : 'bg-secondary text-text-muted border-border'
                  }`}>
                    {getSessionStatusLabel(session.status)}
                  </span>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-[12px] text-text-secondary">
                  <div className="flex items-center gap-2"><Clock3 size={12} /> 启动于 {formatDateTimeZh(session.created_at)}</div>
                  <div className="flex items-center gap-2"><GitBranch size={12} /> 工作目录：{session.workspace_path}</div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            当前还没有挂到这个席位名下的会话记录。
          </div>
        )}
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.assignedWI}</h3>
        {ownedWorkItems.length > 0 ? (
          <div className="space-y-3">
            {ownedWorkItems.map((workItem) => (
              <div key={workItem.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="monospace text-[11px] font-semibold text-text-muted bg-bg-secondary px-2 py-0.5 rounded">{workItem.id.toUpperCase()}</span>
                  <span className={`text-[11px] font-semibold ${workItem.priority === 'Critical' || workItem.priority === 'High' ? 'text-status-error' : 'text-status-warning'}`}>
                    {getPriorityLabel(workItem.priority)}
                  </span>
                  <span className="text-[11px] text-text-secondary">{getWorkItemStatusLabel(workItem.status)}</span>
                  {workItem.active_delegation && (
                    <span className="text-[10px] font-black text-status-warning uppercase bg-status-warning/10 border border-status-warning/20 px-1.5 py-0.5 rounded flex items-center gap-1">
                      <UserPlus size={10} /> Delegated
                    </span>
                  )}
                </div>
                <div className="text-sm font-semibold text-text-primary leading-6">{workItem.title}</div>
                <p className="text-[12px] leading-6 text-text-secondary whitespace-pre-wrap">{workItem.goal || '该工作项暂未填写详细目标。'}</p>
                <div className="flex items-center gap-3 flex-wrap text-[11px] text-text-muted">
                  <span>验收标准 {workItem.acceptance_criteria.length} 条</span>
                  <span>依赖 {workItem.depends_on.length} 项</span>
                  <span>最近更新 {formatDateTimeZh(workItem.updated_at)}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            该席位目前没有直接负责的工作项，可能仅承担阶段复核或临时支援角色。
          </div>
        )}
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近协作交接</h3>
        {relatedHandoffs.length > 0 ? (
          <div className="space-y-3">
            {relatedHandoffs.map((handoff) => {
              const fromSeat = typeof handoff.from_ref === 'object' && 'Seat' in handoff.from_ref ? getSeatName(handoff.from_ref.Seat) : '人工介入';
              const toSeat = typeof handoff.to_ref === 'object' && 'Seat' in handoff.to_ref ? getSeatName(handoff.to_ref.Seat) : '人工介入';
              const direction = fromSeat === seat.name ? `发往 ${toSeat}` : `来自 ${fromSeat}`;

              return (
                <div key={handoff.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2 shadow-sm">
                  <div className="flex items-center justify-between gap-3 flex-wrap">
                    <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
                      <Send size={14} className="text-accent" />
                      <span>{direction}</span>
                    </div>
                    <span className="text-[11px] text-text-secondary">{getHandoffStatusLabel(handoff.status)}</span>
                  </div>
                  <p className="text-[12px] leading-6 text-text-secondary">{handoff.purpose}</p>
                  <div className="flex items-center gap-3 flex-wrap text-[11px] text-text-muted">
                    <span className="monospace">{handoff.id.toUpperCase()}</span>
                    <span>工作项 {handoff.workitem_id.toUpperCase()}</span>
                    <span>发送于 {formatDateTimeZh(handoff.sent_at || handoff.created_at)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            该席位今天尚未形成可追溯交接记录。
          </div>
        )}
      </div>
    </div>
  );
};

export default SeatDetail;
