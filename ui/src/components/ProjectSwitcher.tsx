import React, { useMemo, useState, useRef, useEffect } from 'react';
import ReactDOM from 'react-dom';
import { Check, ChevronDown, Clock, Folder, Pin, Search } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { formatShortDateTimeZh } from '../utils/display';

interface ProjectSwitcherProps {
  onSelect: (projectId: string) => void;
}

const ProjectSwitcher: React.FC<ProjectSwitcherProps> = ({ onSelect }) => {
  const { projects, projectData, activeProjectId, pinProject } = useDataStore();
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [dropdownPos, setDropdownPos] = useState({ top: 0, left: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);

  const activeProject = projects.find((project) => project.id === activeProjectId);

  useEffect(() => {
    if (isOpen && triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      setDropdownPos({ top: rect.bottom + 8, left: rect.left });
    }
  }, [isOpen]);

  const filteredProjects = useMemo(
    () =>
      projects
        .filter((project) => project.name.toLowerCase().includes(search.toLowerCase()))
        .sort((left, right) => (left.isPinned === right.isPinned ? 0 : left.isPinned ? -1 : 1)),
    [projects, search],
  );

  return (
    <div className="relative">
      <button
        ref={triggerRef}
        onClick={() => setIsOpen(!isOpen)}
        className={`flex items-center gap-2 px-3 py-1.5 border rounded-lg transition-all group focus:outline-none focus:ring-2 focus:ring-primary/20 ${
          isOpen ? 'bg-primary/10 border-primary text-primary shadow-sm' : 'bg-secondary border-border text-foreground hover:bg-muted'
        }`}
      >
        <Folder size={14} className={isOpen ? 'text-primary' : 'text-primary/70'} />
        <span className="text-xs font-bold max-w-[220px] truncate">{activeProject?.name || '选择项目'}</span>
        <ChevronDown size={14} className={`text-muted-foreground transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && ReactDOM.createPortal(
        <>
          <div className="fixed inset-0 z-[9998] bg-accent/20" onClick={() => setIsOpen(false)} />
          <div
            className="w-[360px] bg-card border border-border rounded-xl shadow-2xl overflow-hidden animate-in slide-in-from-top-2 duration-200 backdrop-blur-xl"
            style={{ position: 'fixed', top: dropdownPos.top, left: dropdownPos.left, zIndex: 9999 }}
          >
            <div className="p-3 border-b border-border bg-secondary/80">
              <div className="relative">
                <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                <input
                  autoFocus
                  type="text"
                  placeholder="搜索项目或快速切换（Ctrl+K）"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  className="w-full bg-background border border-border rounded-md pl-9 pr-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary text-foreground font-medium shadow-inner"
                />
              </div>
            </div>

            <div className="max-h-[420px] overflow-y-auto py-2 bg-card">
              <div className="px-4 py-2 text-[9px] font-black text-muted-foreground tracking-[0.2em] opacity-70">最近访问与置顶项目</div>

              <div className="px-2 space-y-1">
                {filteredProjects.map((project) => {
                  const data = projectData[project.id];
                  const inboxCount = data?.inboxItems.length || 0;
                  const runningSessions = data?.sessions.filter((session) => session.status === 'Running' || session.status === 'InputRequired').length || 0;

                  return (
                    <div
                      key={project.id}
                      onClick={() => {
                        onSelect(project.id);
                        setIsOpen(false);
                      }}
                      className={`group px-3 py-3 flex items-center justify-between rounded-lg cursor-pointer transition-all ${
                        project.id === activeProjectId ? 'bg-primary/10 border border-primary/20' : 'hover:bg-primary/5 border border-transparent'
                      }`}
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <div
                          className={`p-2 rounded-md shadow-sm border ${
                            project.id === activeProjectId
                              ? 'bg-primary text-primary-foreground border-primary/20'
                              : 'bg-background border-border text-muted-foreground group-hover:text-primary transition-colors'
                          }`}
                        >
                          <Folder size={14} />
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className={`text-[12px] font-bold truncate ${project.id === activeProjectId ? 'text-primary' : 'text-foreground'}`}>
                              {project.name}
                            </span>
                            {project.id === activeProjectId && <Check size={12} className="text-primary" />}
                          </div>
                          <div className="text-[10px] text-muted-foreground truncate font-mono opacity-80 mt-0.5">{project.path}</div>
                          <div className="mt-1 flex items-center gap-2 text-[10px] text-muted-foreground">
                            <span>{inboxCount} 条待处理</span>
                            <span>·</span>
                            <span>{runningSessions} 个活跃会话</span>
                            <span>·</span>
                            <span>{formatShortDateTimeZh(project.last_accessed)}</span>
                          </div>
                        </div>
                      </div>
                      <button
                        onClick={(event) => {
                          event.stopPropagation();
                          pinProject(project.id, !project.isPinned);
                        }}
                        className={`p-1.5 rounded-md transition-all ${
                          project.isPinned
                            ? 'text-primary bg-primary/10'
                            : 'text-muted-foreground hover:bg-accent opacity-0 group-hover:opacity-100'
                        }`}
                        title={project.isPinned ? '取消置顶' : '置顶项目'}
                      >
                        <Pin size={12} fill={project.isPinned ? 'currentColor' : 'none'} />
                      </button>
                    </div>
                  );
                })}
              </div>

              {filteredProjects.length === 0 && <div className="p-8 text-center text-muted-foreground text-xs italic">没有匹配“{search}”的项目</div>}

              <div className="mt-2 mx-2 p-1 border-t border-border/50">
                <div
                  onClick={() => {
                    onSelect('all-projects');
                    setIsOpen(false);
                  }}
                  className="w-full flex items-center justify-center gap-2 p-3 text-[11px] font-black tracking-widest text-primary hover:bg-primary/5 rounded-lg cursor-pointer transition-all group"
                >
                  <Clock size={14} className="group-hover:scale-110 transition-transform" />
                  查看全部项目总览
                </div>
              </div>
            </div>
          </div>
        </>,
        document.body,
      )}
    </div>
  );
};

export default ProjectSwitcher;
