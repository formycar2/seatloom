import React from 'react';
import { Circle, Clock, Target, Terminal } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';

interface AllProjectsViewProps {
  onSelectProject: (id: string) => void;
}

const AllProjectsView: React.FC<AllProjectsViewProps> = ({ onSelectProject }) => {
  const { projects, projectData, backendHydrated, backendHydrateError } = useDataStore();

  // 2026-10-16: this page used to render whatever `projects` held, and the store's
  // initial value was three hardcoded seeds — so a failed hydration showed three
  // plausible project cards and the operator concluded the product had three projects
  // (ql_delivery was in the database the whole time). The store now starts empty, so an
  // empty list here means one of two things, and they must not be confused.
  if (!backendHydrated) {
    return (
      <div className="flex-1 overflow-y-auto p-12 bg-background">
        <div className="max-w-6xl mx-auto">
          <header className="space-y-2 mb-8">
            <h1 className="text-4xl font-black tracking-tight text-text-primary">Projects</h1>
            <p className="text-text-secondary font-bold opacity-60 uppercase tracking-widest text-xs">
              Project Authority Hub · 项目权威源概览
            </p>
          </header>
          <div
            role="status"
            className="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-6 space-y-3"
          >
            <p className="text-sm font-bold text-text-primary">
              项目还没读出来 —— 上面空着是因为没连上数据库,不是因为你没有项目。
            </p>
            <p className="text-[12.5px] leading-relaxed text-text-secondary">
              这一页的数字来自 PostgreSQL。启动时那次读取没有成功,所以这里一个项目都不显示; 按 house
              规则,没有拿写死的假数据顶着。页面顶部有一条红框,写着后端原样返回的
              错误和下一步该做什么。
            </p>
            {backendHydrateError && (
              <p className="text-[11px] font-mono text-text-secondary break-all">
                {backendHydrateError}
              </p>
            )}
          </div>
        </div>
      </div>
    );
  }

  if (projects.length === 0) {
    return (
      <div className="flex-1 overflow-y-auto p-12 bg-background">
        <div className="max-w-6xl mx-auto">
          <header className="space-y-2 mb-8">
            <h1 className="text-4xl font-black tracking-tight text-text-primary">Projects</h1>
            <p className="text-text-secondary font-bold opacity-60 uppercase tracking-widest text-xs">
              Project Authority Hub · 项目权威源概览
            </p>
          </header>
          <div
            role="status"
            className="rounded-2xl border border-border/60 bg-card p-6 text-text-secondary"
          >
            <p className="text-sm font-bold text-text-primary">数据库里一个项目都没有。</p>
            <p className="text-[12.5px] leading-relaxed mt-1">
              这次读取是成功的,所以这个空是真的空 —— 不是没连上。要建项目,用顶栏的初始化入口。
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-12 bg-background">
      <div className="max-w-6xl mx-auto space-y-12">
        <header className="space-y-2">
          <h1 className="text-4xl font-black tracking-tight text-text-primary">Projects</h1>
          <p className="text-text-secondary font-bold opacity-60 uppercase tracking-widest text-xs">
            Project Authority Hub · 项目权威源概览
          </p>
        </header>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {projects.map((project) => {
            const data = projectData[project.id];
            const activeSessions =
              data?.sessions.filter((s) => s.status === 'Running' || s.status === 'InputRequired')
                .length || 0;
            const openWorkItems =
              data?.workItems.filter((wi) => wi.status !== 'Done' && wi.status !== 'Verified')
                .length || 0;

            return (
              <div
                key={project.id}
                onClick={() => onSelectProject(project.id)}
                className="group relative bg-card border border-border/60 p-8 rounded-3xl cursor-pointer hover:border-primary/40 hover:shadow-2xl hover:shadow-primary/5 transition-all animate-in fade-in slide-in-from-bottom-4 duration-500"
              >
                <div className="absolute top-6 right-8">
                  {project.isPinned && (
                    <div className="p-1.5 bg-primary/10 text-primary rounded-full shadow-sm">
                      <Target size={14} />
                    </div>
                  )}
                </div>

                <div className="space-y-6">
                  <div className="flex items-center justify-between">
                    <div
                      className={`p-1.5 rounded-full border ${activeSessions > 0 ? 'bg-status-active/10 text-status-active border-status-active/20' : 'bg-secondary border-border text-text-muted opacity-60'}`}
                    >
                      <Circle size={10} className="fill-current" />
                    </div>
                    <span className="text-[10px] font-black text-text-muted opacity-40 monospace">
                      PROJECT ID: {project.id.toUpperCase()}
                    </span>
                  </div>

                  <div className="space-y-1">
                    <h3 className="text-2xl font-bold text-text-primary group-hover:text-primary transition-colors leading-tight">
                      {project.name}
                    </h3>
                    <div className="flex items-center gap-2 text-xs text-text-muted font-bold opacity-60 truncate">
                      <span className="monospace">{project.path ?? '无 home_path'}</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4 pt-6 border-t border-border/40">
                    <div className="flex items-center gap-2">
                      <div className="p-1.5 bg-secondary rounded-lg text-text-secondary group-hover:bg-accent transition-colors">
                        <Terminal size={14} />
                      </div>
                      <div>
                        <div className="text-[10px] font-black text-text-muted uppercase">
                          Sessions
                        </div>
                        <div className="text-sm font-bold text-text-primary">
                          {activeSessions} 活跃
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="p-1.5 bg-secondary rounded-lg text-text-secondary group-hover:bg-accent transition-colors">
                        <Clock size={14} />
                      </div>
                      <div>
                        <div className="text-[10px] font-black text-text-muted uppercase">
                          WorkItems
                        </div>
                        <div className="text-sm font-bold text-text-primary">
                          {openWorkItems} 待办
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center justify-between pt-4">
                    <span className="text-[10px] font-bold text-text-muted opacity-60">
                      最近访问: 2小时前
                    </span>
                    <span className="text-[10px] font-black text-primary uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity">
                      进入项目 →
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default AllProjectsView;
