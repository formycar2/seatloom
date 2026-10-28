// ProjectSwitcher — the project picker in the top nav.
//
// 2026-10-27 (AEGIS-2026-10-27-project-switcher-ux-v1): five interaction defects
// the operator hit with 39 projects in the ledger. What changed and why:
//
// 1. PLACEHOLDER LIED. It read "搜索项目或快速切换(Ctrl+K)" but Ctrl+K opens the
//    Supervisor IM (App.tsx) and the Command Bar (useGlobalShortcuts) — both
//    preventDefault, neither opens this picker. A hint that invites a keystroke
//    which does something else is worse than no hint. The shortcut is not
//    implemented here, so the promise is gone. (Adding Ctrl+K would mean
//    displacing a settled L1 interaction, which is not this component's call.)
//
// 2. "最近访问" WAS THE CREATION TIME. `projectFromDto` mapped `createdAt` into
//    `last_accessed` because the backend has no such field, and the in-memory
//    update in useDataStore was overwritten by that mapper on every hydrate. So
//    the column showed when a project was BUILT while claiming when it was
//    VISITED. Not fixable here: the real field is a backend change, and
//    relabelling createdAt would be the same lie in a new font (the shape of the
//    `'~/' + name` fake path removed earlier). The column is therefore GONE.
//
// 3. THE GROUP HEADING PROMISED TWO THINGS AND DELIVERED ONE. "最近访问与置顶项目"
//    over a list sorted only by isPinned. With the visit column gone there is no
//    honest ordering to name beyond pinning, so the heading says only what the
//    sort does.
//
// 4. NO KEYBOARD NAVIGATION. The search input is autoFocus — the operator's hands
//    are already on the keyboard — yet 39 projects could only be reached with the
//    mouse. Arrow/Enter/Escape/Home/End are now handled on the list, the standard
//    pattern for a filtered picker, not a new paradigm.
//
// 5. SEARCH MATCHED ONLY THE NAME. Many of the 39 projects are hierarchical
//    (project_delivery/QL, sf-code/stepcloud), so searching a parent found
//    nothing. The path now participates in the match.
//
// ONE MORE, FOUND WHILE VERIFYING THE ABOVE (reported to Aegis): `projectFromDto`
// hardcodes `isPinned: true`, so every project rendered as pinned, the sort was a
// no-op, and the pin button toggled a value the next hydrate erased. That is a
// data defect rather than an interaction one and it is NOT fixed here — changing
// it needs a persisted pin field, which is the backend/schema lane.

import React, { useMemo, useState, useRef, useEffect } from 'react';
import ReactDOM from 'react-dom';
import { Check, ChevronDown, Clock, Folder, Pin, Search } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';

interface ProjectSwitcherProps {
  onSelect: (projectId: string) => void;
}

/// Does `text` contain `needle`, case-insensitively? Both are already strings.
const contains = (text: string | null | undefined, needle: string) =>
  !!text && text.toLowerCase().includes(needle);

export const ProjectSwitcher: React.FC<ProjectSwitcherProps> = ({ onSelect }) => {
  const { projects, projectData, activeProjectId, pinProject } = useDataStore();
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [dropdownPos, setDropdownPos] = useState({ top: 0, left: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  /// Index of the keyboard-highlighted row within `filteredProjects`. -1 = none,
  /// which is distinct from 0: opening with nothing highlighted is the state
  /// before the operator presses anything.
  const [highlighted, setHighlighted] = useState(-1);
  const listRef = useRef<HTMLDivElement | null>(null);

  const activeProject = projects.find((project) => project.id === activeProjectId);

  useEffect(() => {
    if (isOpen && triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      setDropdownPos({ top: rect.bottom + 8, left: rect.left });
    }
  }, [isOpen]);

  // Name OR path, so a hierarchical project is reachable by its parent
  // (`sf-code` finds everything under it) as well as by its own name.
  const filteredProjects = useMemo(
    () =>
      projects
        .filter((project) => {
          if (!search.trim()) return true;
          const needle = search.trim().toLowerCase();
          return contains(project.name, needle) || contains(project.path, needle);
        })
        .sort((left, right) => (left.isPinned === right.isPinned ? 0 : left.isPinned ? -1 : 1)),
    [projects, search],
  );

  /// Close and forget the highlight, so the next open starts clean.
  const close = () => {
    setIsOpen(false);
    setHighlighted(-1);
  };

  /// Move the highlight, clamping at both ends rather than wrapping: with a
  /// filtered list, wrapping from the last row to the first loses the operator's
  /// place in a way that is hard to notice and annoying to undo.
  const moveHighlight = (delta: number) => {
    setHighlighted((current) => {
      const next = current + delta;
      if (next < 0) return 0;
      if (next >= filteredProjects.length) return filteredProjects.length - 1;
      return next;
    });
  };

  // Keep the highlighted row in view. Without this, arrowing past the bottom of
  // the 420px list moves a selection the operator cannot see.
  useEffect(() => {
    if (highlighted < 0 || !listRef.current) return;
    const row = listRef.current.querySelector<HTMLElement>(`[data-project-index="${highlighted}"]`);
    row?.scrollIntoView({ block: 'nearest' });
  }, [highlighted]);

  /// Open with nothing highlighted rather than with the first row: the operator
  /// has just typed a search, and pre-selecting row 0 would make Enter pick a
  /// project they never looked at.
  useEffect(() => {
    if (isOpen) setHighlighted(-1);
  }, [isOpen]);

  // Keyboard, on the list container. The input keeps focus, so this is a
  // bubbling handler rather than per-row tabIndex management — one listener, no
  // focus trap to get wrong.
  const onListKeyDown = (event: React.KeyboardEvent) => {
    if (filteredProjects.length === 0) {
      // Escape still closes an empty result; the arrows have nothing to move.
      if (event.key === 'Escape') {
        event.preventDefault();
        close();
      }
      return;
    }
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        moveHighlight(1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        moveHighlight(-1);
        break;
      case 'Home':
        event.preventDefault();
        setHighlighted(0);
        break;
      case 'End':
        event.preventDefault();
        setHighlighted(filteredProjects.length - 1);
        break;
      case 'Enter': {
        // Only when a row is actually highlighted. Enter with nothing highlighted
        // must not pick row 0 by accident — the operator may still be typing.
        if (highlighted < 0) return;
        event.preventDefault();
        const chosen = filteredProjects[highlighted];
        onSelect(chosen.id);
        close();
        break;
      }
      case 'Escape':
        event.preventDefault();
        close();
        break;
      default:
        break;
    }
  };

  return (
    <div className="relative">
      <button
        ref={triggerRef}
        onClick={() => (isOpen ? close() : setIsOpen(true))}
        className={`flex items-center gap-2 px-3 py-1.5 border rounded-lg transition-all group focus:outline-none focus:ring-2 focus:ring-primary/20 ${
          isOpen
            ? 'bg-primary/10 border-primary text-primary shadow-sm'
            : 'bg-secondary border-border text-foreground hover:bg-muted'
        }`}
      >
        <Folder size={14} className={isOpen ? 'text-primary' : 'text-primary/70'} />
        <span className="text-xs font-bold max-w-[220px] truncate">
          {activeProject?.name || '选择项目'}
        </span>
        <ChevronDown
          size={14}
          className={`text-muted-foreground transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
        />
      </button>

      {isOpen &&
        ReactDOM.createPortal(
          <>
            <div className="fixed inset-0 z-[9998] bg-accent/20" onClick={close} />
            <div
              className="w-[360px] bg-card border border-border rounded-xl shadow-2xl overflow-hidden animate-in slide-in-from-top-2 duration-200 backdrop-blur-xl"
              style={{
                position: 'fixed',
                top: dropdownPos.top,
                left: dropdownPos.left,
                zIndex: 9999,
              }}
            >
              <div className="p-3 border-b border-border bg-secondary/80">
                <div className="relative">
                  <Search
                    size={14}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                  />
                  <input
                    autoFocus
                    type="text"
                    // No shortcut is promised here: Ctrl+K belongs to the Supervisor
                    // IM and the Command Bar, neither of which opens this picker.
                    placeholder="搜索项目"
                    value={search}
                    onChange={(event) => setSearch(event.target.value)}
                    onKeyDown={onListKeyDown}
                    className="w-full bg-background border border-border rounded-md pl-9 pr-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary text-foreground font-medium shadow-inner"
                  />
                </div>
              </div>

              <div
                ref={listRef}
                onKeyDown={onListKeyDown}
                className="max-h-[420px] overflow-y-auto py-2 bg-card"
              >
                {/* Says only what the sort actually does. There is no honest visit
                  ordering to name: the backend has no last-accessed field, so
                  claiming one would be a lie in the heading. */}
                <div className="px-4 py-2 text-[9px] font-black text-muted-foreground tracking-[0.2em] opacity-70">
                  置顶项目
                </div>

                <div className="px-2 space-y-1">
                  {filteredProjects.map((project, index) => {
                    const data = projectData[project.id];
                    const inboxCount = data?.inboxItems.length || 0;
                    const runningSessions =
                      data?.sessions.filter(
                        (session) =>
                          session.status === 'Running' || session.status === 'InputRequired',
                      ).length || 0;
                    const isHighlighted = index === highlighted;

                    return (
                      <div
                        key={project.id}
                        data-project-index={index}
                        onClick={() => {
                          onSelect(project.id);
                          close();
                        }}
                        // The highlight is a visual affordance only — the row is not
                        // focusable, so there is exactly one keyboard entry point
                        // (the search input) and no focus order to reason about.
                        className={`group px-3 py-3 flex items-center justify-between rounded-lg cursor-pointer transition-all ${
                          project.id === activeProjectId
                            ? 'bg-primary/10 border border-primary/20'
                            : 'hover:bg-primary/5 border border-transparent'
                        } ${isHighlighted ? 'ring-1 ring-primary/40' : ''}`}
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
                              <span
                                className={`text-[12px] font-bold truncate ${project.id === activeProjectId ? 'text-primary' : 'text-foreground'}`}
                              >
                                {project.name}
                              </span>
                              {project.id === activeProjectId && (
                                <Check size={12} className="text-primary" />
                              )}
                            </div>
                            <div className="text-[10px] text-muted-foreground truncate font-mono opacity-80 mt-0.5">
                              {project.path ?? '无 home_path'}
                            </div>
                            <div className="mt-1 flex items-center gap-2 text-[10px] text-muted-foreground">
                              <span>{inboxCount} 条待处理</span>
                              <span>·</span>
                              <span>{runningSessions} 个活跃会话</span>
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

                {filteredProjects.length === 0 && (
                  <div className="p-8 text-center text-muted-foreground text-xs italic">
                    没有匹配“{search}”的项目
                  </div>
                )}

                <div className="mt-2 mx-2 p-1 border-t border-border/50">
                  <div
                    onClick={() => {
                      onSelect('all-projects');
                      close();
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
