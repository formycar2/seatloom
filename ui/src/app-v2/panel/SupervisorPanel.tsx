/**
 * SupervisorPanel.tsx
 * Supervisor 浮层面板：可拖拽、可调整大小的 IM 风格浮窗，包含左侧联系人列表和右侧对话区（支持项目频道和 seat 对话两种视图）
 * 状态通过 localStorage 持久化：面板位置、尺寸、当前联系人、各联系人草稿
 */

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { MOCK_CONTACTS, MOCK_MESSAGES } from '../mock-data';
import { Avatar } from '../components/Avatar';
import { ContactRow } from '../components/ContactRow';
import { MessageBubble } from '../components/MessageBubble';
import { ChatInput } from '../components/ChatInput';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';

const STORAGE_KEY_POS = 'sl-supervisor-pos';
const STORAGE_KEY_SIZE = 'sl-supervisor-size';

function loadSaved() {
  try {
    return {
      pos: JSON.parse(localStorage.getItem(STORAGE_KEY_POS) || 'null'),
      size: JSON.parse(localStorage.getItem(STORAGE_KEY_SIZE) || 'null'),
    };
  } catch { return { pos: null, size: null }; }
}

export const SupervisorPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [activeContactId, setActiveContactId] = useState(() => {
    const saved = localStorage.getItem('sl-supervisor-active-contact');
    // Validate saved contact still exists
    if (saved && MOCK_CONTACTS.some(c => c.id === saved)) return saved;
    return 'supervisor';
  });
  const [input, setInput] = useState(() => localStorage.getItem(`sl-supervisor-draft-${localStorage.getItem('sl-supervisor-active-contact') || 'supervisor'}`) || '');
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

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

  // When switching contact, load that contact's draft
  const switchContact = (id: string) => {
    // Save current draft
    const curKey = `sl-supervisor-draft-${activeContactId}`;
    if (input) localStorage.setItem(curKey, input);
    else localStorage.removeItem(curKey);
    // Switch
    setActiveContactId(id);
    setInput(localStorage.getItem(`sl-supervisor-draft-${id}`) || '');
    setSearch('');
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

  // Data
  const activeContact = MOCK_CONTACTS.find(c => c.id === activeContactId) || MOCK_CONTACTS[0];
  const messages = MOCK_MESSAGES[activeContactId] || [];

  // Group contacts by project
  const projects = new Map<string, typeof MOCK_CONTACTS>();
  const channels: typeof MOCK_CONTACTS = [];
  const supervisorContact = MOCK_CONTACTS.find(c => c.type === 'supervisor')!;

  MOCK_CONTACTS.forEach(c => {
    if (c.type === 'supervisor') return;
    if (c.type === 'project-channel') { channels.push(c); return; }
    const key = c.projectId || '_';
    if (!projects.has(key)) projects.set(key, []);
    projects.get(key)!.push(c);
  });

  const filteredContacts = search.trim()
    ? MOCK_CONTACTS.filter(c => c.name.toLowerCase().includes(search.toLowerCase()) || (c.role || '').includes(search))
    : null;

  const listWidth = 220;

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

        {/* ── Body: sidebar + chat ── */}
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
                // Search results
                filteredContacts.map(c => (
                  <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => { switchContact(c.id); }} />
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

            {/* Header */}
            <div style={{
              display: 'flex', alignItems: 'center', gap: 10,
              padding: '10px 16px', borderBottom: '1px solid var(--sl-divider)', flexShrink: 0,
            }}>
              <Avatar char={activeContact.avatar} color={activeContact.color} size={32} online={activeContact.type === 'seat' ? activeContact.online : undefined} />
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{activeContact.name}</div>
                <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                  {activeContact.type === 'project-channel' ? '项目工作台' : activeContact.role || '全局监督'}
                </div>
              </div>
              <button onClick={onClose} style={{
                width: 24, height: 24, borderRadius: 'var(--sl-radius-sm)', border: 'none',
                background: 'transparent', color: 'var(--sl-text-tertiary)', cursor: 'pointer',
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 14,
              }}
                onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-surface-hover)'; }}
                onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
              >✕</button>
            </div>

            {activeContact.type === 'project-channel' ? (
              /* ════ Project Channel: Workflow Dashboard ════ */
              <ProjectDashboard key={activeContactId} channelId={activeContact.id} projectId={activeContact.projectId} />
            ) : (
              /* ════ Seat / Supervisor: IM Chat ════ */
              <>
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
                  allContacts={MOCK_CONTACTS}
                  input={input}
                  onInputChange={setInput}
                  inputRef={inputRef}
                  onEscape={onClose}
                  onRouteViaPO={(poId) => switchContact(poId)}
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
