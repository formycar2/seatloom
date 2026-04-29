import React from 'react';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  Clock,
  Coins,
  Cpu,
  FolderOpen,
  LayoutDashboard,
  MessageSquare,
  ShieldCheck,
  Zap,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { formatDateTimeZh } from '../utils/display';

const ProjectOverview: React.FC = () => {
  const { activeProjectId, projectData, projects } = useDataStore();
  const currentProject = projects.find(p => p.id === activeProjectId);
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentProject || !currentData) {
    return (
      <div className="p-8 flex flex-col items-center justify-center h-full text-center space-y-4 opacity-50">
        <LayoutDashboard size={48} strokeWidth={1} />
        <p className="text-sm font-bold uppercase tracking-widest text-ink">请选择一个项目以查看概览</p>
      </div>
    );
  }

  const runningSessions = currentData.sessions.filter(s => s.status === 'Running' || s.status === 'InputRequired').length;
  const blockedSessions = currentData.sessions.filter(s => s.status === 'InputRequired' || !!s.prompt_state).length;
  const openWorkItems = currentData.workItems.filter(wi => wi.status !== 'Done' && wi.status !== 'Verified').length;
  const pendingInbox = currentData.inboxItems.length;
  
  // Real calculation from seeded data
  const unresolvedThreads = currentData.workItems.filter(wi => wi.status === 'InReview').length; 
  // Budget modeling not yet supported by current seeded schema, show unavailable
  const budgetAlert = null; 
  const lastReconcile = currentData.events.find(e => e.event_type === 'ReconcileCompleted')?.occurred_at || currentData.workItems[0]?.updated_at;

  return (
    <div className="p-8 space-y-8 animate-in fade-in duration-500">
      <header className="space-y-1">
        <div className="flex items-center gap-2 text-primary">
          <ShieldCheck size={18} />
          <span className="text-[10px] font-black uppercase tracking-[0.2em]">项目权威源概览 (AUTHORITY HUB)</span>
        </div>
        <h2 className="text-3xl font-black tracking-tight text-ink">{currentProject.name}</h2>
        <div className="flex items-center gap-2 text-xs text-ink-soft font-bold opacity-60">
          <FolderOpen size={14} />
          <span className="monospace">{currentProject.path}</span>
        </div>
      </header>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-card border border-border/60 p-5 rounded-2xl shadow-sm space-y-4">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-primary/10 text-primary rounded-xl">
              <Cpu size={20} />
            </div>
            <span className="text-[10px] font-black text-ink-faint uppercase">运行时状态</span>
          </div>
          <div>
            <div className="text-2xl font-black text-ink">{runningSessions}</div>
            <div className="text-[10px] font-bold text-ink-soft opacity-70 uppercase tracking-tight">活跃会话 (Active Sessions)</div>
          </div>
          {blockedSessions > 0 && (
            <div className="flex items-center gap-1.5 text-status-warning animate-pulse">
              <AlertTriangle size={12} />
              <span className="text-[10px] font-black uppercase">{blockedSessions} 处输入阻塞</span>
            </div>
          )}
        </div>

        <div className="bg-card border border-border/60 p-5 rounded-2xl shadow-sm space-y-4">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-primary/10 text-primary rounded-xl">
              <Activity size={20} />
            </div>
            <span className="text-[10px] font-black text-ink-faint uppercase">任务负荷</span>
          </div>
          <div>
            <div className="text-2xl font-black text-ink">{openWorkItems}</div>
            <div className="text-[10px] font-bold text-ink-soft opacity-70 uppercase tracking-tight">进行中工作项 (Open WorkItems)</div>
          </div>
          <div className="text-[10px] font-bold text-ink-faint uppercase">
            收件箱待处理: {pendingInbox}
          </div>
        </div>
      </div>

      <div className="space-y-4">
        <h3 className="text-[11px] font-black text-ink-soft uppercase tracking-[0.2em] px-1 flex items-center gap-2">
          <BarChart3 size={14} />
          项目健康度监测
        </h3>
        
        <div className="bg-secondary/40 border border-border/60 rounded-2xl p-4 divide-y divide-border/40">
          <div className="py-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <ShieldCheck size={16} className="text-status-active" />
              <span className="text-sm font-bold text-ink">协作模版状态</span>
            </div>
            <span className="text-xs font-black text-primary bg-accent px-2 py-0.5 rounded border border-primary/10 shadow-sm">V0.5 标准规约</span>
          </div>

          <div className="py-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Clock size={16} className="text-ink-soft" />
              <span className="text-sm font-bold text-ink">最后对账时间</span>
            </div>
            <span className="text-xs font-bold text-ink-soft opacity-80">{lastReconcile ? formatDateTimeZh(lastReconcile) : '尚未对账'}</span>
          </div>

          <div className="py-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <MessageSquare size={16} className="text-ink-soft" />
              <span className="text-sm font-bold text-ink">未解决评审线程</span>
            </div>
            <span className={`text-xs font-black ${unresolvedThreads > 0 ? 'text-status-warning' : 'text-ink-faint'}`}>{unresolvedThreads} 条</span>
          </div>

          <div className="py-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Coins size={16} className="text-ink-faint opacity-40" />
              <span className="text-sm font-bold text-ink opacity-40">预算预警状态</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-black text-ink-faint uppercase bg-secondary px-1.5 py-0.5 rounded">未就绪 / 未索引</span>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-primary text-surface p-6 rounded-2xl shadow-xl shadow-primary/20 space-y-4">
        <div className="flex items-center gap-2">
          <Zap size={18} className="fill-current" />
          <h4 className="text-sm font-black uppercase tracking-widest">快捷操作 (QUICK ACTIONS)</h4>
        </div>
        <p className="text-[11px] font-bold opacity-80 leading-relaxed">
          当前项目基线已锁定。您可以直接在左侧导航中查看收件箱或时间线，或者通过席位卡片启动新的运行时会话。
        </p>
        <div className="flex gap-2">
          <div className="px-3 py-1.5 bg-surface/20 border border-surface/20 rounded-lg text-[10px] font-black uppercase tracking-tighter">
            Shift + / 帮助
          </div>
          <div className="px-3 py-1.5 bg-surface/20 border border-surface/20 rounded-lg text-[10px] font-black uppercase tracking-tighter">
            Cmd + K 切换项目
          </div>
        </div>
      </div>
    </div>
  );
};

export default ProjectOverview;
