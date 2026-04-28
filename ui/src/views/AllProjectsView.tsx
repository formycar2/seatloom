import React from 'react';
import { Folder, Clock, Inbox, Zap, ChevronRight, Pin, ListTodo, Users } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { formatShortDateTimeZh } from '../utils/display';

interface AllProjectsViewProps {
  onSelectProject: (id: string) => void;
}

const AllProjectsView: React.FC<AllProjectsViewProps> = ({ onSelectProject }) => {
  const { projects, projectData } = useDataStore();

  return (
    <div className="flex-1 overflow-y-auto p-8 md:p-12 bg-background">
      <div className="max-w-5xl mx-auto space-y-10">
        <header className="space-y-3">
          <h1 className="text-3xl font-black tracking-tight text-foreground">项目总览与协调态势</h1>
          <p className="text-sm leading-7 text-muted-foreground max-w-3xl">
            这里汇总每个仓的当前待处理规模、会话活跃度、工作项压力和最近一次写回信号，帮助你先判断哪里最需要进入，再决定切换上下文。
          </p>
        </header>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {projects.map((project) => {
            const data = projectData[project.id];
            const inboxCount = data?.inboxItems.length || 0;
            const activeSessions = data?.sessions.filter((session) => session.status === 'Running' || session.status === 'InputRequired').length || 0;
            const workItemCount = data?.workItems.length || 0;
            const blockedCount = data?.workItems.filter((workItem) => workItem.status === 'Blocked').length || 0;
            const reviewCount = data?.workItems.filter((workItem) => workItem.status === 'InReview' || workItem.status === 'Verified').length || 0;
            const seatCount = data?.seats.length || 0;
            const lastEvent = data?.events[data.events.length - 1]?.occurred_at || project.last_accessed;
            const latestNarrative =
              data?.events[data.events.length - 1]?.payload?.summary ||
              data?.events[data.events.length - 1]?.payload?.title ||
              '暂无最近活动摘要。';

            return (
              <div
                key={project.id}
                onClick={() => onSelectProject(project.id)}
                className="sl-clickable group relative p-6 bg-card border border-border rounded-2xl shadow-sm hover:border-primary/40 hover:shadow-xl hover:shadow-primary/5 transition-all cursor-pointer"
              >
                {project.isPinned && (
                  <div className="absolute top-4 right-4 text-primary opacity-70 group-hover:opacity-100">
                    <Pin size={14} fill="currentColor" />
                  </div>
                )}

                <div className="flex items-start gap-4 mb-5">
                  <div className="p-3 bg-primary/5 rounded-xl border border-primary/20 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-colors">
                    <Folder size={24} />
                  </div>
                  <div className="min-w-0 flex-1">
                    <h2 className="text-lg font-bold truncate group-hover:text-primary transition-colors">{project.name}</h2>
                    <p className="text-xs text-muted-foreground font-mono truncate mt-1">{project.path}</p>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-3 mb-5">
                  <div className="p-3 rounded-xl bg-secondary/50 border border-border flex items-center gap-3">
                    <div className={`p-1.5 rounded-full ${inboxCount > 0 ? 'bg-primary text-primary-foreground' : 'bg-background border border-border text-muted-foreground'}`}>
                      <Inbox size={12} />
                    </div>
                    <div>
                      <div className="text-[11px] font-semibold text-muted-foreground tracking-wide">待处理</div>
                      <div className={`text-sm font-bold ${inboxCount > 0 ? 'text-primary' : 'text-foreground'}`}>{inboxCount} 条收件箱事项</div>
                    </div>
                  </div>

                  <div className="p-3 rounded-xl bg-secondary/50 border border-border flex items-center gap-3">
                    <div className={`p-1.5 rounded-full ${activeSessions > 0 ? 'bg-status-active text-white' : 'bg-background border border-border text-muted-foreground'}`}>
                      <Zap size={12} />
                    </div>
                    <div>
                      <div className="text-[11px] font-semibold text-muted-foreground tracking-wide">活跃会话</div>
                      <div className="text-sm font-bold text-foreground">{activeSessions} 个正在推进</div>
                    </div>
                  </div>

                  <div className="p-3 rounded-xl bg-secondary/50 border border-border flex items-center gap-3">
                    <div className="p-1.5 rounded-full bg-background border border-border text-muted-foreground">
                      <ListTodo size={12} />
                    </div>
                    <div>
                      <div className="text-[11px] font-semibold text-muted-foreground tracking-wide">工作面</div>
                      <div className="text-sm font-bold text-foreground">{workItemCount} 项工作项</div>
                    </div>
                  </div>

                  <div className="p-3 rounded-xl bg-secondary/50 border border-border flex items-center gap-3">
                    <div className="p-1.5 rounded-full bg-background border border-border text-muted-foreground">
                      <Users size={12} />
                    </div>
                    <div>
                      <div className="text-[11px] font-semibold text-muted-foreground tracking-wide">团队规模</div>
                      <div className="text-sm font-bold text-foreground">{seatCount} 个席位在线</div>
                    </div>
                  </div>
                </div>

                <div className="rounded-xl border border-border bg-background/70 p-4 space-y-3">
                  <div className="flex items-center gap-2 flex-wrap text-[11px] text-muted-foreground">
                    <span className="px-2 py-1 rounded-full bg-secondary border border-border">评审面 {reviewCount}</span>
                    <span className="px-2 py-1 rounded-full bg-secondary border border-border">阻塞 {blockedCount}</span>
                    <span className="px-2 py-1 rounded-full bg-secondary border border-border">事件 {data?.events.length || 0}</span>
                  </div>
                  <p className="text-sm leading-6 text-foreground/85 line-clamp-3">{latestNarrative}</p>
                </div>

                <div className="mt-5 pt-4 border-t border-border/50 flex items-center justify-between text-[11px] font-semibold text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <Clock size={12} />
                    最近写回：{formatShortDateTimeZh(lastEvent)}
                  </div>
                  <div className="opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1 text-primary font-bold">
                    进入项目 <ChevronRight size={12} />
                  </div>
                </div>
              </div>
            );
          })}

          <button className="p-6 border-2 border-dashed border-border rounded-2xl flex flex-col items-center justify-center gap-3 hover:border-primary/40 hover:bg-primary/5 transition-all text-muted-foreground hover:text-primary group">
            <div className="w-10 h-10 rounded-full border border-current flex items-center justify-center group-hover:scale-110 transition-transform">
              <Folder size={20} />
            </div>
            <span className="text-sm font-bold tracking-wide">初始化新的项目工作区</span>
            <span className="text-xs text-center leading-6 max-w-[240px]">为新的仓建立 SeatLoom 协调账本、状态目录与最近活动基线。</span>
          </button>
        </div>
      </div>
    </div>
  );
};

export default AllProjectsView;
