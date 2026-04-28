import React, { useMemo, useState } from 'react';
import {
  AlertTriangle,
  Clock3,
  Cpu,
  FileText,
  FastForward,
  FolderTree,
  GitBranch,
  RefreshCcw,
  ShieldAlert,
  Terminal,
  User,
} from 'lucide-react';
import { Session } from '../types';
import SwitchRuntimeDialog from './SwitchRuntimeDialog';
import { useDataStore } from '../stores/useDataStore';
import {
  formatDateTimeZh,
  getRuntimeLabel,
  getSessionStatusLabel,
  getWorkItemStatusLabel,
} from '../utils/display';

interface SessionDetailProps {
  session: Session;
}

const SessionDetail: React.FC<SessionDetailProps> = ({ session }) => {
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

  const runtimeName = getRuntimeLabel(session.runtime);
  const isInterrupted = session.status === 'Interrupted';
  const isRunning = session.status === 'Running' || session.status === 'InputRequired';

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-start gap-3">
        <div className={`p-3 rounded-lg shadow-sm ${isInterrupted ? 'bg-status-warning/10 text-status-warning' : 'bg-primary/10 text-primary'}`}>
          <Terminal size={24} />
        </div>
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <h2 className="text-xl font-bold monospace leading-tight">{session.id}</h2>
            <div className="flex items-center gap-2 text-xs text-text-muted mt-1 flex-wrap">
              <span className={`font-semibold ${
                session.status === 'Running'
                  ? 'text-status-active'
                  : isInterrupted
                    ? 'text-status-warning'
                    : 'text-text-muted'
              }`}>
                {getSessionStatusLabel(session.status)}
              </span>
              <span>·</span>
              <span>{runtimeName}</span>
              <span>·</span>
              <span>{seat?.name || session.seat_id}</span>
            </div>
          </div>
          <p className="text-sm leading-6 text-text-secondary">
            该会话承接的是今日真实协调链路中的上下文，下面展示当前运行时、工作项挂载关系和最近几条可回放事件，方便判断是继续推进、恢复还是切换运行时。
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
          <div className="meta-label flex items-center gap-1.5"><User size={11} /> 所属席位</div>
          <div className="font-semibold text-text-primary">{seat?.name || session.seat_id}</div>
          <div className="text-xs text-text-secondary">该席位负责承接当前运行时中的工作上下文。</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
          <div className="meta-label flex items-center gap-1.5"><Cpu size={11} /> 运行时</div>
          <div className="font-semibold text-text-primary">{runtimeName}</div>
          <div className="text-xs text-text-secondary">状态：{getSessionStatusLabel(session.status)}</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
          <div className="meta-label flex items-center gap-1.5"><GitBranch size={11} /> 分支与目录</div>
          <div className="font-semibold text-text-primary">{session.branch || '未记录分支'}</div>
          <div className="text-xs text-text-secondary break-all">{session.workspace_path}</div>
        </div>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
          <div className="meta-label flex items-center gap-1.5"><Clock3 size={11} /> 时间信息</div>
          <div className="font-semibold text-text-primary">启动于 {formatDateTimeZh(session.created_at)}</div>
          <div className="text-xs text-text-secondary">{session.ended_at ? `结束于 ${formatDateTimeZh(session.ended_at)}` : '仍处于可继续接力的活动窗口。'}</div>
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">当前会话挂载的工作项</h3>
        {relatedWorkItems.length > 0 ? (
          <div className="space-y-3">
            {relatedWorkItems.map((workItem) => (
              <div key={workItem.id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="monospace text-[11px] font-semibold text-text-muted bg-bg-secondary px-2 py-0.5 rounded">{workItem.id.toUpperCase()}</span>
                  <span className="text-[11px] text-text-secondary">{getWorkItemStatusLabel(workItem.status)}</span>
                </div>
                <div className="text-sm font-semibold text-text-primary">{workItem.title}</div>
                <p className="text-[12px] leading-6 text-text-secondary whitespace-pre-wrap">{workItem.goal || '暂未填写目标说明。'}</p>
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            当前没有直接命中的工作项引用，后续可通过恢复启动包或最近事件来恢复焦点。
          </div>
        )}
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider">最近会话事件</h3>
        {relatedEvents.length > 0 ? (
          <div className="space-y-3">
            {relatedEvents.map((event) => (
              <div key={event.event_id} className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5">
                <div className="text-xs text-text-muted">{formatDateTimeZh(event.occurred_at)}</div>
                <div className="text-sm font-semibold text-text-primary leading-6">{event.payload?.title || event.payload?.summary || '已记录一条会话状态更新。'}</div>
                {event.evidence_refs.length > 0 && (
                  <div className="text-[11px] text-primary break-all">证据：{event.evidence_refs[0]}</div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">
            当前会话暂未写入独立事件摘要。
          </div>
        )}
      </div>

      {isInterrupted && (
        <div className="border border-status-warning/30 bg-status-warning/5 rounded-xl p-4 space-y-4 shadow-sm">
          <div className="flex items-start gap-3">
            <ShieldAlert size={18} className="text-status-warning mt-0.5" />
            <div>
              <h3 className="text-sm font-bold text-status-warning">会话已中断，需要明确恢复路径</h3>
              <p className="text-xs text-text-secondary mt-1 leading-6">按照当前产品合同，可选择原生恢复、检查点重建或最小启动包接力；先恢复上下文，再决定是否继续推进。</p>
            </div>
          </div>
          <div className="space-y-2">
            <button className="w-full flex items-center justify-between p-3 rounded-lg border border-status-warning/20 bg-background hover:border-status-warning transition-all group">
              <div className="flex items-center gap-3">
                <div className="p-1.5 bg-status-warning/10 rounded"><RefreshCcw size={14} className="text-status-warning" /></div>
                <div className="text-left">
                  <div className="text-xs font-bold text-foreground group-hover:text-status-warning">原生恢复</div>
                  <div className="text-[10px] text-text-secondary">优先尝试恢复原始 PID 或运行时连接，保留当前上下文。</div>
                </div>
              </div>
              <span className="text-[9px] font-black uppercase text-status-warning bg-status-warning/10 px-1.5 py-0.5 rounded">方案 1</span>
            </button>
            <button className="w-full flex items-center justify-between p-3 rounded-lg border border-border bg-background hover:border-primary transition-all group">
              <div className="flex items-center gap-3">
                <div className="p-1.5 bg-secondary rounded"><FileText size={14} className="text-text-muted" /></div>
                <div className="text-left">
                  <div className="text-xs font-bold text-foreground group-hover:text-primary">检查点重建</div>
                  <div className="text-[10px] text-text-secondary">从最近一次摘要或持久化产物重建会话记忆，适合运行时已失联的场景。</div>
                </div>
              </div>
              <span className="text-[9px] font-black uppercase text-text-muted bg-secondary px-1.5 py-0.5 rounded">方案 2</span>
            </button>
            <button className="w-full flex items-center justify-between p-3 rounded-lg border border-border bg-background hover:border-primary transition-all group">
              <div className="flex items-center gap-3">
                <div className="p-1.5 bg-secondary rounded"><FastForward size={14} className="text-text-muted" /></div>
                <div className="text-left">
                  <div className="text-xs font-bold text-foreground group-hover:text-primary">最小启动包接力</div>
                  <div className="text-[10px] text-text-secondary">仅携带工作项、分支、最近异常和回写路径，最快切到新的运行时继续推进。</div>
                </div>
              </div>
              <span className="text-[9px] font-black uppercase text-text-muted bg-secondary px-1.5 py-0.5 rounded">方案 3</span>
            </button>
          </div>
        </div>
      )}

      {isRunning && (
        <div className="pt-4 border-t border-border space-y-3">
          <div className="rounded-xl border border-border bg-bg-elevated p-4 text-sm leading-6 text-text-secondary">
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

      <SwitchRuntimeDialog isOpen={showSwitch} onClose={() => setShowSwitch(false)} session={session} />
    </div>
  );
};

export default SessionDetail;
