import React, { useMemo } from 'react';
import {
  Activity,
  AlertCircle,
  ArrowRight,
  Clock,
  Coins,
  Cpu,
  History,
  Inbox,
  LayoutDashboard,
  Smartphone,
  User,
  Zap,
  ShieldAlert,
  Send,
  Gavel,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';
import { InboxItem } from '../types';

interface MobileCompanionViewProps {
  onSelectObject: (item: any) => void;
}

const MOBILE_APPROVAL_GATE_TYPES = new Set([
  '待处理交接',
  '需要验收',
  '需确认范围',
  '验收待决策',
  '输入请求',
]);

const MOBILE_URGENT_ACTION_TYPES = new Set([
  ...MOBILE_APPROVAL_GATE_TYPES,
  '会话恢复',
  '待补产物',
]);

const getMobilePriorityBand = (item: InboxItem) => {
  if (item.priority === 'Critical') return 0;
  return MOBILE_URGENT_ACTION_TYPES.has(item.type) ? 1 : 2;
};

const MobileCompanionView: React.FC<MobileCompanionViewProps> = ({ onSelectObject }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData, projects } = useDataStore();
  
  const currentProject = projects.find(p => p.id === activeProjectId);
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const stats = useMemo(() => {
    if (!currentData) return null;
    
    // Explicit monitor signals per contract
    const promptBlockedCount = currentData.sessions.filter(s => !!s.prompt_state).length;
    const pendingHandoffsCount = currentData.handoffs.filter(h => h.status !== 'Completed' && h.status !== 'Expired').length;
    
    const pendingApprovalsCount = currentData.inboxItems.filter(item =>
      MOBILE_APPROVAL_GATE_TYPES.has(item.type)
    ).length;

    const openWorkItems = currentData.workItems.filter(wi => wi.status !== 'Done' && wi.status !== 'Verified').length;
    const highPriorityAlerts = currentData.inboxItems.filter(item => (item.priority as string) === 'Critical').length;
    
    const lastEvent = currentData.events[0];
    const syncFreshness = lastEvent ? `同步于 ${new Date(lastEvent.occurred_at).toLocaleTimeString()}` : '尚未同步';

    return {
      promptBlockedCount,
      pendingHandoffsCount,
      pendingApprovalsCount,
      openWorkItems,
      highPriorityAlerts,
      syncFreshness,
    };
  }, [currentData]);

  const urgentInboxItems = useMemo(() => {
    if (!currentData) return [];
    return currentData.inboxItems
      .filter(item => item.priority === 'Critical' || MOBILE_URGENT_ACTION_TYPES.has(item.type))
      .sort((a, b) => {
        const bandDelta = getMobilePriorityBand(a) - getMobilePriorityBand(b);
        if (bandDelta !== 0) return bandDelta;

        // Seeded timestamps use a zero-padded `YYYY-MM-DD HH:mm` format, so lexical compare is stable.
        return b.timestamp.localeCompare(a.timestamp);
      });
  }, [currentData]);

  if (!currentProject || !currentData || !stats) {
    return (
      <div className="flex-1 flex items-center justify-center p-12 opacity-50 text-ink">
        <div className="text-center space-y-4">
          <Smartphone size={48} className="mx-auto" />
          <p className="text-sm font-bold uppercase tracking-widest">请选择项目以开启移动伴侣</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 bg-secondary/30 overflow-y-auto custom-scrollbar">
      <div className="max-w-[400px] mx-auto min-h-full bg-background border-x border-border shadow-2xl flex flex-col transition-colors duration-500">
        {/* Mobile Header */}
        <header className="px-6 py-4 border-b border-border bg-card sticky top-0 z-10">
          <div className="flex items-center justify-between mb-1">
            <div className="flex items-center gap-2 text-primary">
              <Smartphone size={16} />
              <span className="text-[10px] font-black uppercase tracking-widest">Companion · 伴侣终端</span>
            </div>
            <div className="text-[9px] font-bold text-ink-faint uppercase bg-secondary px-1.5 py-0.5 rounded border border-border/40">
              v0.5 Protocol
            </div>
          </div>
          <h2 className="text-xl font-black text-ink truncate">{currentProject.name}</h2>
          <div className="flex items-center gap-2 text-[10px] text-ink-soft font-bold opacity-60 mt-1">
            <Clock size={12} />
            <span>{stats.syncFreshness}</span>
          </div>
        </header>

        <main className="flex-1 p-5 space-y-6">
          {/* Explicit Monitor Signals Grid */}
          <section className="grid grid-cols-3 gap-2">
            <div className="bg-card border border-border/60 p-3 rounded-2xl shadow-sm space-y-2 flex flex-col items-center text-center">
              <div className="p-1.5 bg-status-error/10 text-status-error rounded-lg">
                <Gavel size={14} />
              </div>
              <div>
                <div className="text-lg font-black text-ink">{stats.pendingApprovalsCount}</div>
                <div className="text-[8px] font-bold text-ink-soft uppercase leading-tight">待审批 / 闸门</div>
              </div>
            </div>
            <div className="bg-card border border-border/60 p-3 rounded-2xl shadow-sm space-y-2 flex flex-col items-center text-center">
              <div className="p-1.5 bg-status-warning/10 text-status-warning rounded-lg">
                <ShieldAlert size={14} />
              </div>
              <div>
                <div className="text-lg font-black text-ink">{stats.promptBlockedCount}</div>
                <div className="text-[8px] font-bold text-ink-soft uppercase leading-tight">Prompt 阻塞</div>
              </div>
            </div>
            <div className="bg-card border border-border/60 p-3 rounded-2xl shadow-sm space-y-2 flex flex-col items-center text-center">
              <div className="p-1.5 bg-primary/10 text-primary rounded-lg">
                <Send size={14} />
              </div>
              <div>
                <div className="text-lg font-black text-ink">{stats.pendingHandoffsCount}</div>
                <div className="text-[8px] font-bold text-ink-soft uppercase leading-tight">进行中交接</div>
              </div>
            </div>
          </section>

          {/* Health List */}
          <section className="bg-secondary/40 border border-border/60 rounded-2xl p-4 divide-y divide-border/40">
            <div className="pb-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Activity size={14} className="text-ink-soft" />
                <span className="text-xs font-bold text-ink">进行中工作项</span>
              </div>
              <span className="text-xs font-black text-ink">{stats.openWorkItems}</span>
            </div>
            <div className="py-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <AlertCircle size={14} className="text-status-error" />
                <span className="text-xs font-bold text-ink">紧急告警 (Critical)</span>
              </div>
              <span className="text-xs font-black text-status-error">{stats.highPriorityAlerts}</span>
            </div>
            <div className="pt-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Coins size={14} className="text-ink-soft opacity-40" />
                <span className="text-xs font-bold text-ink opacity-40">预算状态</span>
              </div>
              <span className="text-[9px] font-black text-ink-faint uppercase bg-secondary px-1.5 py-0.5 rounded">未就绪</span>
            </div>
          </section>

          {/* Recommended Action */}
          <section className="bg-primary text-surface p-5 rounded-2xl shadow-lg shadow-primary/20 space-y-3">
            <div className="flex items-center gap-2">
              <Zap size={16} className="fill-current" />
              <h4 className="text-[10px] font-black uppercase tracking-widest">推荐后续 (NEXT ACTION)</h4>
            </div>
            <p className="text-xs font-bold opacity-90 leading-relaxed">
              {stats.promptBlockedCount > 0 
                ? '有会话等待输入，建议优先处理阻塞 Prompt。'
                : stats.pendingApprovalsCount > 0
                  ? '检测到待处理审批事项，请复核验收结论或交接请求。'
                  : '项目状态稳健，可继续关注工作项进度。'}
            </p>
          </section>

          {/* Urgent Mobile Inbox */}
          <section className="space-y-4">
            <div className="flex items-center justify-between px-1">
              <h3 className="text-[11px] font-black text-ink-soft uppercase tracking-[0.2em] flex items-center gap-2">
                <Inbox size={14} />
                紧急收件箱 (URGENT ONLY)
              </h3>
              <span className="text-[9px] font-bold text-ink-faint uppercase bg-secondary px-1.5 py-0.5 rounded">
                {urgentInboxItems.length} 项
              </span>
            </div>
            
            <div className="bg-card border border-border/60 rounded-2xl shadow-sm divide-y divide-border/40 overflow-hidden">
              {urgentInboxItems.length > 0 ? (
                urgentInboxItems.slice(0, 8).map(item => (
                  <div key={item.id} className="hover:bg-primary/5 active:bg-primary/10 transition-colors cursor-pointer" onClick={() => onSelectObject(item)}>
                    <div className="p-4 space-y-2">
                      <div className="flex items-center justify-between">
                        <span className={`text-[8px] font-black uppercase px-1.5 py-0.5 rounded border ${
                          item.priority === 'Critical' ? 'bg-status-error/10 text-status-error border-status-error/20' :
                          (item.priority as string) === 'Normal' ? 'bg-status-warning/10 text-status-warning border-status-warning/20' :
                          'bg-secondary text-ink-soft border-border/40'
                        }`}>
                          {item.priority}
                        </span>
                        <span className="text-[8px] font-black text-ink-faint uppercase">{item.timestamp}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-[10px] font-black text-primary monospace uppercase">{item.object_ref}</span>
                        <span className="text-[10px] text-ink-soft opacity-40">·</span>
                        <span className="text-[10px] font-bold text-ink-soft uppercase truncate">{item.type}</span>
                      </div>
                      <p className="text-xs font-bold text-ink leading-relaxed line-clamp-2">{item.summary}</p>
                      <div className="flex items-center justify-between pt-2">
                        <span className="text-[9px] font-bold text-ink-faint uppercase">By {item.actor}</span>
                        <div className="flex items-center gap-1 text-[9px] font-black text-primary uppercase">
                          查看详情 <ArrowRight size={10} />
                        </div>
                      </div>
                    </div>
                  </div>
                ))
              ) : (
                <div className="p-12 text-center space-y-2">
                  <div className="text-sm font-bold text-ink-soft opacity-40">暂无紧急待办</div>
                  <p className="text-[10px] text-ink-faint uppercase tracking-widest">All caught up on mobile</p>
                </div>
              )}
            </div>
          </section>
        </main>

        {/* Mobile Nav Mock */}
        <footer className="mt-auto px-6 py-4 border-t border-border bg-card grid grid-cols-4 gap-2">
          <div className="flex flex-col items-center gap-1 text-primary">
            <LayoutDashboard size={18} />
            <span className="text-[8px] font-black uppercase">首页</span>
          </div>
          <div className="flex flex-col items-center gap-1 text-ink-faint opacity-60">
            <Inbox size={18} />
            <span className="text-[8px] font-black uppercase">收件箱</span>
          </div>
          <div className="flex flex-col items-center gap-1 text-ink-faint opacity-60">
            <History size={18} />
            <span className="text-[8px] font-black uppercase">历史</span>
          </div>
          <div className="flex flex-col items-center gap-1 text-ink-faint opacity-60">
            <User size={18} />
            <span className="text-[8px] font-black uppercase">我的</span>
          </div>
        </footer>
      </div>
    </div>
  );
};

export default MobileCompanionView;
