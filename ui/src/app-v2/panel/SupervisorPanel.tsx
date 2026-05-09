/**
 * SupervisorPanel.tsx
 * Supervisor 浮层面板：可拖拽、可调整大小的 IM 风格浮窗，包含左侧联系人列表和右侧对话区。
 *
 * AD-013 v2: Supervisor 两层上下文模型 + viewMode 正交状态（chan-09）
 *   contextMode ('global' | 'project'): dashboard 数据隔离
 *     - 'global' ⇒ activeProjectId === null
 *     - 'project' ⇒ activeProjectId 非空
 *   viewMode ('dashboard' | 'chat'): 右侧 pane 路由，与 contextMode 正交
 *     - 'dashboard' → GlobalDashboard 或 ProjectDashboard（按 contextMode）
 *     - 'chat'      → IM chat pane（按 activeContact.type）
 *   切换路径仅 enterGlobal() / enterProject(projectId)，是显式用户动作
 *   chat-with-seat 面包屑的 projectName 来源是 activeContact.projectId，与 activeProjectId 解耦
 */

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Avatar } from '../components/Avatar';
import { ContactRow } from '../components/ContactRow';
import { MessageBubble } from '../components/MessageBubble';
import { ChatInput } from '../components/ChatInput';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';
import { GlobalDashboard } from '../dashboard/GlobalDashboard';
import { SupervisorContextMode, SupervisorViewMode, ChatMessage } from '../types';
import { useDataStore } from '../../stores/useDataStore';
import { useLiveSessionsStore } from '../../stores/useLiveSessionsStore';
import { api, isTauri } from '../../lib/api';
import { useContacts, useMessages, backendSeatIdFromContact } from '../hooks/useSupervisorData';

const STORAGE_KEY_POS = 'sl-supervisor-pos';
const STORAGE_KEY_SIZE = 'sl-supervisor-size';
const STORAGE_KEY_MODE = 'seatloom.supervisor.contextMode';
const STORAGE_KEY_ACTIVE_PROJECT = 'seatloom.supervisor.activeProjectId';
const STORAGE_KEY_VIEW_MODE = 'seatloom.supervisor.viewMode';

function loadSaved() {
  try {
    return {
      pos: JSON.parse(localStorage.getItem(STORAGE_KEY_POS) || 'null'),
      size: JSON.parse(localStorage.getItem(STORAGE_KEY_SIZE) || 'null'),
    };
  } catch { return { pos: null, size: null }; }
}

// AD-013 §D + §7 §E: stale state fallback.
// contextMode: project 需要 activeProjectId 存在于 useDataStore.projects，否则降级 global。
// viewMode: 缺失/非法 → 'dashboard'；'chat' 但 activeContactId 找不到 contact → 降级 'dashboard'。
function loadInitialContext(): {
  mode: SupervisorContextMode;
  activeProjectId: string | null;
  viewMode: SupervisorViewMode;
} {
  const rawMode = localStorage.getItem(STORAGE_KEY_MODE);
  const rawProj = localStorage.getItem(STORAGE_KEY_ACTIVE_PROJECT);
  const rawView = localStorage.getItem(STORAGE_KEY_VIEW_MODE);

  // contextMode + activeProjectId stale fallback (chan-03 §D)
  const wantsProject = rawMode === 'project';
  let mode: SupervisorContextMode = 'global';
  let activeProjectId: string | null = null;
  if (wantsProject && rawProj) {
    const projects = useDataStore.getState().projects;
    if (projects.some((p) => p.id === rawProj)) {
      mode = 'project';
      activeProjectId = rawProj;
    }
  }

  // viewMode stale fallback (chan-09 §E). With backend-driven contacts the
  // strict membership check happens at render time (activeContact resolution
  // falls back to contacts[0]), so init-time we just honour the saved view.
  let viewMode: SupervisorViewMode = 'dashboard';
  if (rawView === 'chat') {
    const savedContact = localStorage.getItem('sl-supervisor-active-contact');
    if (savedContact) {
      viewMode = 'chat';
    }
  }

  return { mode, activeProjectId, viewMode };
}

// Resolve project display name: useDataStore → projectId fallback.
function resolveProjectName(projectId: string | undefined): string {
  if (!projectId) return '';
  const proj = useDataStore.getState().projects.find((p) => p.id === projectId);
  if (proj) return proj.name;
  return projectId;
}

// Stable fallback contact object — defined at module scope so its identity
// never changes across renders. Embedding it inside the component body would
// allocate a new reference each render, which propagates through
// activeContact → useMessages dependencies and triggers
// "Maximum update depth exceeded" via the canonical:appended subscription.
const FALLBACK_SUPERVISOR_CONTACT = {
  id: 'supervisor',
  name: 'Supervisor',
  type: 'supervisor' as const,
  online: true,
  unread: 0,
  avatar: 'A',
  color: 'var(--sl-brand)',
};

export const SupervisorPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  // Live contact list and message stream from real backend data.
  const contacts = useContacts();

  const [activeContactId, setActiveContactId] = useState(() => {
    const saved = localStorage.getItem('sl-supervisor-active-contact');
    return saved || 'supervisor';
  });
  const [input, setInput] = useState(() => localStorage.getItem(`sl-supervisor-draft-${localStorage.getItem('sl-supervisor-active-contact') || 'supervisor'}`) || '');
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // AD-013 v2: contextMode + activeProjectId + viewMode state (option X — local useState)
  const initialContextRef = useRef(loadInitialContext());
  const [contextMode, setContextMode] = useState<SupervisorContextMode>(initialContextRef.current.mode);
  const [activeProjectId, setActiveProjectIdState] = useState<string | null>(initialContextRef.current.activeProjectId);
  const [viewMode, setViewMode] = useState<SupervisorViewMode>(initialContextRef.current.viewMode);

  // Persist contextMode
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_MODE, contextMode);
  }, [contextMode]);

  // Persist activeProjectId
  useEffect(() => {
    if (activeProjectId === null) {
      localStorage.removeItem(STORAGE_KEY_ACTIVE_PROJECT);
    } else {
      localStorage.setItem(STORAGE_KEY_ACTIVE_PROJECT, activeProjectId);
    }
  }, [activeProjectId]);

  // Persist viewMode
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_VIEW_MODE, viewMode);
  }, [viewMode]);

  // AD-013: 仅以下两个函数能修改 contextMode / activeProjectId
  const enterGlobal = useCallback(() => {
    setContextMode('global');
    setActiveProjectIdState(null);
    setViewMode('dashboard');
  }, []);

  const enterProject = useCallback((projectId: string) => {
    setContextMode('project');
    setActiveProjectIdState(projectId);
    setViewMode('dashboard');
  }, []);

  // 面包屑中间层点击 + GlobalDashboard onSelectProject：进入 project 并同步 contact 高亮到 project channel
  const enterProjectFromBreadcrumb = useCallback((projectId: string) => {
    enterProject(projectId);
    const ch = contacts.find((c) => c.type === 'project-channel' && c.projectId === projectId);
    if (ch) setActiveContactId(ch.id);
  }, [enterProject, contacts]);

  // Position & size persistence
  const savedRef = useRef(loadSaved());
  const [pos, setPos] = useState({ x: savedRef.current.pos?.x ?? -1, y: savedRef.current.pos?.y ?? -1 });
  const [size, setSize] = useState({ w: savedRef.current.size?.w ?? 680, h: savedRef.current.size?.h ?? 520 });
  const [dragging, setDragging] = useState(false);
  const [resizing, setResizing] = useState(false);
  const dragOff = useRef({ x: 0, y: 0 });
  const resizeStart = useRef({ x: 0, y: 0, w: 0, h: 0 });
  const panelRef = useRef<HTMLDivElement>(null);

  // Center on first open
  useEffect(() => {
    if (pos.x === -1) {
      setPos({ x: Math.max(0, (window.innerWidth - size.w) / 2), y: Math.max(40, (window.innerHeight - size.h) / 2) });
    }
    inputRef.current?.focus();
  }, []);

  // Persist active contact
  useEffect(() => {
    localStorage.setItem('sl-supervisor-active-contact', activeContactId);
  }, [activeContactId]);

  // Persist draft per contact
  useEffect(() => {
    const key = `sl-supervisor-draft-${activeContactId}`;
    if (input) localStorage.setItem(key, input);
    else localStorage.removeItem(key);
  }, [input, activeContactId]);

  // AD-013 v2 §D: switchContact 按 contact.type 分支
  // - project-channel → viewMode='dashboard' + enterProject + setActiveContactId
  // - seat / supervisor → viewMode='chat' + setActiveContactId；不动 contextMode / activeProjectId
  const switchContact = (id: string) => {
    // Save current draft
    const curKey = `sl-supervisor-draft-${activeContactId}`;
    if (input) localStorage.setItem(curKey, input);
    else localStorage.removeItem(curKey);
    // Switch contact + restore draft
    setActiveContactId(id);
    setInput(localStorage.getItem(`sl-supervisor-draft-${id}`) || '');
    setSearch('');
    const contact = contacts.find((c) => c.id === id);
    if (!contact) return;
    if (contact.type === 'project-channel' && contact.projectId) {
      enterProject(contact.projectId);
      // enterProject sets viewMode='dashboard'; activeContactId already set above
    } else {
      // seat or supervisor: only switch viewMode to chat, do NOT touch contextMode/activeProjectId
      setViewMode('chat');
    }
  };

  // Drag
  const onDragStart = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('input, button, [data-no-drag]')) return;
    setDragging(true);
    const r = panelRef.current!.getBoundingClientRect();
    dragOff.current = { x: e.clientX - r.left, y: e.clientY - r.top };
    e.preventDefault();
  }, []);

  useEffect(() => {
    if (!dragging) return;
    const move = (e: MouseEvent) => setPos({ x: Math.max(0, Math.min(window.innerWidth - 100, e.clientX - dragOff.current.x)), y: Math.max(0, Math.min(window.innerHeight - 60, e.clientY - dragOff.current.y)) });
    const up = () => { setDragging(false); setPos(p => { localStorage.setItem(STORAGE_KEY_POS, JSON.stringify(p)); return p; }); };
    window.addEventListener('mousemove', move); window.addEventListener('mouseup', up);
    return () => { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
  }, [dragging]);

  // Resize
  const onResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault(); e.stopPropagation(); setResizing(true);
    const r = panelRef.current!.getBoundingClientRect();
    resizeStart.current = { x: e.clientX, y: e.clientY, w: r.width, h: r.height };
  }, []);

  useEffect(() => {
    if (!resizing) return;
    const move = (e: MouseEvent) => setSize({ w: Math.max(480, Math.min(1000, resizeStart.current.w + e.clientX - resizeStart.current.x)), h: Math.max(360, Math.min(800, resizeStart.current.h + e.clientY - resizeStart.current.y)) });
    const up = () => { setResizing(false); setSize(s => { localStorage.setItem(STORAGE_KEY_SIZE, JSON.stringify(s)); return s; }); };
    window.addEventListener('mousemove', move); window.addEventListener('mouseup', up);
    return () => { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
  }, [resizing]);

  // Escape
  useEffect(() => {
    const fn = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', fn); return () => window.removeEventListener('keydown', fn);
  }, [onClose]);

  // Data — backend-driven via hooks (see useSupervisorData.ts).
  // FALLBACK_SUPERVISOR_CONTACT lives at module scope so activeContact has
  // a stable identity when contacts is empty, preventing infinite re-render
  // loops in dependent hooks (useMessages → onCanonicalAppended).
  const activeContact = contacts.find(c => c.id === activeContactId)
    ?? contacts[0]
    ?? FALLBACK_SUPERVISOR_CONTACT;
  const messages = useMessages(activeContact);

  // Live PTY session lookup (seat → sessionId), for routing.
  const sessionFor = useLiveSessionsStore((s) => s.sessionFor);

  const handleSend = useCallback(async (text: string) => {
    if (!text || !activeContact) return;
    if (!isTauri()) return;

    // Persist to canonical_events. The backend emits canonical:appended
    // which the useMessages hook listens for, so the bubble appears live.
    try {
      const targetSeatId = activeContact.type === 'seat'
        ? backendSeatIdFromContact(activeContact)
        : undefined;
      await api.appendSupervisorMessage({ targetSeatId, content: text });
    } catch (err) {
      console.error('[SupervisorPanel] appendSupervisorMessage failed:', err);
    }

    // Forward into a live PTY session if the target seat has one.
    if (activeContact.type === 'seat') {
      const sid = sessionFor(activeContact.name);
      if (sid) {
        try { await api.ptyWrite(sid, text + '\n'); }
        catch (err) { console.error('[SupervisorPanel] ptyWrite failed:', err); }
      }
    }
  }, [activeContact, sessionFor]);

  // For ProjectDashboard: find channel contact for activeProjectId
  const projectChannelForDashboard = contacts.find(
    (c) => c.type === 'project-channel' && c.projectId === activeProjectId
  );

  // Group contacts by project (for sidebar)
  const projects = new Map<string, typeof contacts>();
  const channels: typeof contacts = [];
  const supervisorContact = contacts.find(c => c.type === 'supervisor') ?? contacts[0];

  contacts.forEach(c => {
    if (c.type === 'supervisor') return;
    if (c.type === 'project-channel') { channels.push(c); return; }
    const key = c.projectId || '_';
    if (!projects.has(key)) projects.set(key, []);
    projects.get(key)!.push(c);
  });

  const filteredContacts = search.trim()
    ? contacts.filter(c => c.name.toLowerCase().includes(search.toLowerCase()) || (c.role || '').includes(search))
    : null;

  const listWidth = 220;

  // ── AD-013 v2 §B: 面包屑 4 种渲染情况 ──
  const renderBreadcrumb = () => {
    // Case 1: dashboard + global → 仅「全局」
    if (viewMode === 'dashboard' && contextMode === 'global') {
      return <span style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>全局</span>;
    }

    // Case 2: dashboard + project → 全局 › 项目名（项目名是当前位置，不可点击）
    if (viewMode === 'dashboard' && contextMode === 'project') {
      const projName = resolveProjectName(activeProjectId ?? undefined);
      return (
        <>
          <button
            type="button"
            onClick={enterGlobal}
            style={{ background: 'transparent', border: 'none', padding: 0, cursor: 'pointer', color: 'var(--sl-text-secondary)', fontSize: 13, fontWeight: 500 }}
            onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--sl-brand)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--sl-text-secondary)'; }}
          >全局</button>
          <span aria-hidden style={{ color: 'var(--sl-text-tertiary)' }}>›</span>
          <span
            title={projName}
            style={{ color: 'var(--sl-text-primary)', fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}
          >{projName}</span>
        </>
      );
    }

    // Case 3: chat + supervisor → 仅「全局」（不伸展）
    if (viewMode === 'chat' && activeContact.type === 'supervisor') {
      return <span style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>全局</span>;
    }

    // Case 4: chat + seat → 全局 › 项目名（可点击 link）› seat 名
    if (viewMode === 'chat' && activeContact.type === 'seat') {
      const seatProjName = resolveProjectName(activeContact.projectId);
      const seatProjId = activeContact.projectId;
      return (
        <>
          <button
            type="button"
            onClick={enterGlobal}
            style={{ background: 'transparent', border: 'none', padding: 0, cursor: 'pointer', color: 'var(--sl-text-secondary)', fontSize: 13, fontWeight: 500 }}
            onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--sl-brand)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--sl-text-secondary)'; }}
          >全局</button>
          <span aria-hidden style={{ color: 'var(--sl-text-tertiary)' }}>›</span>
          {/* 中间层：可点击 link，affordance 参考 §C */}
          <button
            type="button"
            onClick={() => seatProjId && enterProjectFromBreadcrumb(seatProjId)}
            title={seatProjName}
            style={{
              background: 'transparent', border: 'none', padding: 0,
              cursor: seatProjId ? 'pointer' : 'default',
              color: 'var(--sl-text-secondary)', fontSize: 13, fontWeight: 500,
              whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
              maxWidth: 120,
            }}
            onMouseEnter={(e) => { if (seatProjId) e.currentTarget.style.color = 'var(--sl-brand)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--sl-text-secondary)'; }}
          >{seatProjName}</button>
          <span aria-hidden style={{ color: 'var(--sl-text-tertiary)' }}>›</span>
          <span
            title={activeContact.name}
            style={{ color: 'var(--sl-text-primary)', fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}
          >{activeContact.name}</span>
        </>
      );
    }

    // Fallback (project-channel in chat mode — treated as dashboard alias)
    return <span style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>全局</span>;
  };

  return (
    <>
      {/* Backdrop */}
      <div onClick={onClose} style={{ position: 'fixed', inset: 0, zIndex: 9998, background: 'rgba(0,0,0,0.08)', backdropFilter: 'blur(1px)' }} />

      {/* Panel */}
      <div ref={panelRef} style={{
        position: 'fixed',
        left: pos.x === -1 ? '50%' : pos.x, top: pos.y === -1 ? '50%' : pos.y,
        transform: pos.x === -1 ? 'translate(-50%,-50%)' : 'none',
        zIndex: 9999, width: size.w, height: size.h,
        background: 'var(--sl-surface)', border: '1px solid var(--sl-border)',
        borderRadius: 'var(--sl-radius-xl)', boxShadow: 'var(--sl-shadow-overlay)',
        display: 'flex', flexDirection: 'column',
        cursor: dragging ? 'grabbing' : 'default',
        userSelect: (dragging || resizing) ? 'none' : 'auto',
        overflow: 'hidden',
      }}>

        {/* ── Invisible drag handle (top edge) ── */}
        <div
          style={{ height: 6, cursor: 'grab', flexShrink: 0 }}
          onMouseDown={e => {
            setDragging(true);
            const r = panelRef.current!.getBoundingClientRect();
            dragOff.current = { x: e.clientX - r.left, y: e.clientY - r.top };
            e.preventDefault();
          }}
        />

        {/* ── Body: sidebar + right pane ── */}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>

          {/* ── Left: Contact list ── */}
          <div data-no-drag style={{
            width: listWidth, flexShrink: 0,
            borderRight: '1px solid var(--sl-divider)',
            display: 'flex', flexDirection: 'column',
            background: 'var(--sl-bg)',
            cursor: 'default',
          }}>
            {/* Search */}
            <div style={{ padding: '8px 10px' }}>
              <input
                type="text" value={search} onChange={e => setSearch(e.target.value)}
                placeholder="搜索席位或项目..."
                style={{
                  width: '100%', padding: '6px 10px', fontSize: 12,
                  border: '1px solid var(--sl-border-light)', borderRadius: 'var(--sl-radius-md)',
                  background: 'var(--sl-surface)', outline: 'none', color: 'var(--sl-text-primary)',
                }}
                onFocus={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; }}
                onBlur={e => { e.currentTarget.style.borderColor = 'var(--sl-border-light)'; }}
              />
            </div>

            {/* Contact list */}
            <div style={{ flex: 1, overflowY: 'auto' }}>
              {filteredContacts ? (
                filteredContacts.map(c => (
                  <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => switchContact(c.id)} />
                ))
              ) : (
                <>
                  {/* Supervisor */}
                  <ContactRow contact={supervisorContact} active={activeContactId === 'supervisor'} onClick={() => switchContact('supervisor')} />

                  {/* Project channels */}
                  {channels.length > 0 && (
                    <div style={{ padding: '10px 12px 4px', fontSize: 10, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>频道</div>
                  )}
                  {channels.map(c => (
                    <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => switchContact(c.id)} />
                  ))}

                  {/* Seats grouped by project */}
                  {Array.from(projects.entries()).map(([projId, seats]) => {
                    const projChannel = channels.find(c => c.projectId === projId);
                    const projName = projChannel?.name || projId;
                    return (
                      <React.Fragment key={projId}>
                        <div style={{ padding: '10px 12px 4px', fontSize: 10, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                          {projName}
                        </div>
                        {seats.map(c => (
                          <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => switchContact(c.id)} />
                        ))}
                      </React.Fragment>
                    );
                  })}
                </>
              )}
            </div>
          </div>

          {/* ── Right: Context-aware content ── */}
          <div data-no-drag style={{ flex: 1, display: 'flex', flexDirection: 'column', cursor: 'default', minWidth: 0 }}>

            {/* AD-013 v2 §B: Breadcrumb header (4-case) */}
            <div style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '10px 16px', borderBottom: '1px solid var(--sl-divider)', flexShrink: 0,
            }}>
              <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 6, fontSize: 13, minWidth: 0 }}>
                {renderBreadcrumb()}
              </div>
              <button onClick={onClose} style={{
                width: 24, height: 24, borderRadius: 'var(--sl-radius-sm)', border: 'none',
                background: 'transparent', color: 'var(--sl-text-tertiary)', cursor: 'pointer',
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 14, flexShrink: 0,
              }}
                onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-surface-hover)'; }}
                onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
              >✕</button>
            </div>

            {viewMode === 'dashboard' ? (
              /* ════ Dashboard pane ════ */
              contextMode === 'global' ? (
                <GlobalDashboard onSelectProject={enterProjectFromBreadcrumb} />
              ) : (
                <ProjectDashboard
                  key={activeProjectId ?? 'project'}
                  channelId={projectChannelForDashboard?.id ?? ''}
                  projectId={activeProjectId ?? undefined}
                />
              )
            ) : (
              /* ════ Chat pane ════ */
              <>
                {/* Contact sub-header */}
                <div style={{
                  display: 'flex', alignItems: 'center', gap: 10,
                  padding: '10px 16px', borderBottom: '1px solid var(--sl-divider)', flexShrink: 0,
                }}>
                  <Avatar char={activeContact.avatar} color={activeContact.color} size={32} online={activeContact.type === 'seat' ? activeContact.online : undefined} />
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{activeContact.name}</div>
                    <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                      {activeContact.type === 'supervisor' ? '全局监督' : activeContact.role || ''}
                    </div>
                  </div>
                </div>

                {/* Messages */}
                <div style={{ flex: 1, overflowY: 'auto', padding: '16px', display: 'flex', flexDirection: 'column', gap: 10 }}>
                  {messages.length === 0 && (
                    <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                      <p style={{ fontSize: 13, color: 'var(--sl-text-tertiary)' }}>开始和 {activeContact.name} 对话</p>
                    </div>
                  )}
                  {messages.map(msg => <MessageBubble key={msg.id} msg={msg} />)}
                </div>

                {/* Routing governance + Input */}
                <ChatInput
                  contact={activeContact}
                  allContacts={contacts}
                  input={input}
                  onInputChange={setInput}
                  inputRef={inputRef}
                  onEscape={onClose}
                  onRouteViaPO={(poId) => switchContact(poId)}
                  onSend={handleSend}
                />
              </>
            )}
          </div>
        </div>

        {/* Resize handle */}
        <div onMouseDown={onResizeStart} style={{
          position: 'absolute', bottom: 2, right: 2, width: 14, height: 14,
          cursor: 'nwse-resize', display: 'flex', alignItems: 'center', justifyContent: 'center', opacity: 0.3,
        }}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
            <path d="M7 1L1 7M7 4L4 7" stroke="var(--sl-text-tertiary)" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </div>
      </div>

      <style>{`
        @keyframes sv-fadeIn { from { opacity: 0; } to { opacity: 1; } }
      `}</style>
    </>
  );
};
