import React from 'react';
import { BriefcaseBusiness, Circle, Clock3, GitBranch, Send, User } from 'lucide-react';
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
  const relatedHandoffs = (currentData?.handoffs || [])
    .filter((handoff) => {
      const fromSeat = typeof handoff.from_ref === 'object' && 'Seat' in handoff.from_ref ? handoff.from_ref.Seat : null;
      const toSeat = typeof handoff.to_ref === 'object' && 'Seat' in handoff.to_ref ? handoff.to_ref.Seat : null;
      return fromSeat === seat.id || toSeat === seat.id;
    })
    .sort((left, right) => new Date(right.sent_at || right.created_at).getTime() - new Date(left.sent_at || left.created_at).getTime())
    .slice(0, 4);

  const getSeatName = (seatId: string) => currentData?.seats.find((item) => item.id === seatId)?.name || seatId;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-start gap-3">
        <div className="p-3 bg-accent/10 rounded-full text-accent shadow-sm">
          <User size={24} />
        </div>
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <h2 className="text-xl font-bold leading-tight">{seat.name}</h2>
            <div className="flex items-center gap-2 text-xs text-text-muted mt-1 flex-wrap">
              <Circle size={8} className={`fill-current ${seat.status === 'Active' ? 'text-status-active' : seat.status === 'Paused' ? 'text-status-warning' : 'text-status-done'}`} />
              <span>{getSeatStatusLabel(seat.status)}</span>
              <span>·</span>
              <span>{getSeatRoleLabel(seat.role)}</span>
            </div>
          </div>
          <p className="text-sm leading-6 text-text-secondary">
            该席位当前负责 {ownedWorkItems.length} 个工作项，最近参与了 {relatedHandoffs.length} 次可追溯交接；本视图展示它在今日协作链路中的真实位置，而不是静态占位信息。
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1">
          <div className="meta-label">席位状态</div>
          <div className="text-sm font-semibold text-text-primary">{getSeatStatusLabel(seat.status)}</div>
          <div className="text-xs text-text-secondary">当前活跃会话 {activeSessions.length} 个，总会话 {seatSessions.length} 个</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1">
          <div className="meta-label">角色职责</div>
          <div className="text-sm font-semibold text-text-primary">{getSeatRoleLabel(seat.role)}</div>
          <div className="text-xs text-text-secondary">负责范围覆盖产品、设计、验证或工程协作中的一个明确席位。</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1">
          <div className="meta-label">加入时间</div>
          <div className="text-sm font-semibold text-text-primary">{formatDateTimeZh(seat.created_at)}</div>
          <div className="text-xs text-text-secondary">已在当前项目账本中建立席位身份，可承接会话、工作项和交接记录。</div>
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">{t.detail.activeSessions}</h3>
        {seatSessions.length > 0 ? (
          <div className="space-y-3">
            {seatSessions.map((session) => (
              <div key={session.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
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
              <div key={workItem.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="monospace text-[11px] font-semibold text-text-muted bg-bg-secondary px-2 py-0.5 rounded">{workItem.id.toUpperCase()}</span>
                  <span className={`text-[11px] font-semibold ${workItem.priority === 'Critical' || workItem.priority === 'High' ? 'text-status-error' : 'text-status-warning'}`}>
                    {getPriorityLabel(workItem.priority)}
                  </span>
                  <span className="text-[11px] text-text-secondary">{getWorkItemStatusLabel(workItem.status)}</span>
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
                <div key={handoff.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
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
