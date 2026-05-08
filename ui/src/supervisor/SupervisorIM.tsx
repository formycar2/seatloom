// SupervisorIM — Supervisor chat surface (L1), backend-first.
//
// Contacts are computed from useDataStore.seats (hydrated from cmd_list_seats).
// Messages for the active contact come from cmd_list_supervisor_messages;
// sends go through cmd_append_supervisor_message which persists to
// canonical_events and (when the target seat has a live PTY session) injects
// the content into the PTY. Live updates arrive via the `canonical:appended`
// Tauri event.
//
// The component accepts an `embedded` prop:
//   - embedded=true  → rendered inside the main window as a movable floating
//     panel (draggable/resizable). Shows a "Detach" button (wired in R3).
//   - embedded=false → rendered full-viewport inside the detached supervisor
//     window (R3). Shows a "Re-embed" button.

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api, isTauri, onCanonicalAppended } from '../lib/api';
import type { CanonicalEventDto, SeatDto } from '../lib/types-dto';
import { useDataStore } from '../stores/useDataStore';
import { useLiveSessionsStore } from '../stores/useLiveSessionsStore';

type ContactKind = 'supervisor' | 'seat';

interface Contact {
  id: string;            // 'supervisor' or seat.id
  kind: ContactKind;
  name: string;
  role?: string;         // lowercase backend role (e.g. 'product_owner')
  avatar: string;        // single char
  color: string;         // CSS var
  online: boolean;
  seatId?: string;       // backend seat.id for routing
  seatName?: string;     // backend seat.name for live-session lookup
}

interface Props {
  embedded: boolean;
  onClose?: () => void;
  onDetach?: () => void;
  onReembed?: () => void;
}

const COLOR_BY_ROLE: Record<string, string> = {
  supervisor: 'var(--sl-brand)',
  product_owner: 'var(--sl-purple)',
  architect: 'var(--sl-blue)',
  designer: 'var(--sl-amber)',
  verifier: 'var(--sl-green)',
  developer: 'var(--sl-teal)',
};

function contactFromSeat(seat: SeatDto, role?: string): Contact {
  return {
    id: seat.id,
    kind: 'seat',
    name: seat.name.charAt(0).toUpperCase() + seat.name.slice(1),
    role,
    avatar: seat.name.charAt(0).toUpperCase(),
    color: COLOR_BY_ROLE[role ?? ''] ?? 'var(--sl-text-secondary)',
    online: seat.status === 'active',
    seatId: seat.id,
    seatName: seat.name,
  };
}

const SUPERVISOR_CONTACT: Contact = {
  id: 'supervisor',
  kind: 'supervisor',
  name: 'Supervisor',
  avatar: 'A',
  color: 'var(--sl-brand)',
  online: true,
};

export const SupervisorIM: React.FC<Props> = ({ embedded, onClose, onDetach, onReembed }) => {
  const seats = useDataStore((s) => s.projectData[s.activeProjectId ?? '']?.seats ?? []);
  const activeProjectId = useDataStore((s) => s.activeProjectId);
  const sessionFor = useLiveSessionsStore((s) => s.sessionFor);

  // Build contacts: supervisor + each seat, stable order.
  const contacts: Contact[] = useMemo(() => {
    const list: Contact[] = [SUPERVISOR_CONTACT];
    for (const s of seats) {
      // V1 Seat role is coerced; we want the backend lowercase string for colour.
      const rawRole = typeof s.role === 'string' ? s.role : (s.role as any).Custom ?? '';
      const normalised = rawRole.toLowerCase().replace(/\s+/g, '_');
      // Reconstruct a SeatDto-ish for the factory.
      const seatDto: SeatDto = {
        id: s.id,
        name: s.name,
        defaultRuntime: null,
        capabilityTags: s.capabilities ?? [],
        status: s.status.toLowerCase(),
        createdAt: s.created_at,
      };
      list.push(contactFromSeat(seatDto, normalised));
    }
    return list;
  }, [seats]);

  const [activeContactId, setActiveContactId] = useState<string>(() => {
    return localStorage.getItem('seatloom.supervisor.activeContactId') || 'supervisor';
  });

  useEffect(() => {
    localStorage.setItem('seatloom.supervisor.activeContactId', activeContactId);
  }, [activeContactId]);

  const activeContact = contacts.find((c) => c.id === activeContactId) ?? contacts[0];

  // --- Messages ---
  const [messages, setMessages] = useState<CanonicalEventDto[]>([]);
  const [messagesLoading, setMessagesLoading] = useState(false);
  const [sendError, setSendError] = useState<string | null>(null);

  const loadMessages = useCallback(async (contact: Contact) => {
    if (!isTauri()) { setMessages([]); return; }
    setMessagesLoading(true);
    try {
      const targetSeatId = contact.kind === 'seat' ? contact.seatId : undefined;
      const rows = await api.listSupervisorMessages({ targetSeatId, limit: 200 });
      setMessages(rows);
    } catch (e) {
      console.error('[SupervisorIM] load messages failed', e);
      setMessages([]);
    } finally {
      setMessagesLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!activeContact) return;
    loadMessages(activeContact);
  }, [activeContact, loadMessages]);

  // --- Live message appends (canonical:appended) ---
  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    onCanonicalAppended((evt) => {
      // Filter: only show messages matching the active contact.
      const targetFromPayload = (evt.payload as any)?.target_seat_id ?? null;
      if (activeContact.kind === 'seat' && activeContact.seatId !== targetFromPayload) return;
      if (activeContact.kind === 'supervisor' && targetFromPayload != null) return;
      setMessages((prev) => {
        if (prev.some((m) => m.id === evt.id)) return prev;
        return [...prev, evt];
      });
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, [activeContact]);

  // --- Input ---
  const [input, setInput] = useState('');
  const inputRef = useRef<HTMLInputElement | null>(null);

  const handleSend = async () => {
    const text = input.trim();
    if (!text) return;
    if (!isTauri()) {
      setSendError('Tauri backend not available — messages cannot be persisted.');
      return;
    }
    try {
      // Persist to canonical_events.
      await api.appendSupervisorMessage({
        targetSeatId: activeContact.kind === 'seat' ? activeContact.seatId : undefined,
        content: text,
      });
      // Also route into the PTY if the seat has a live session — the backend
      // command is a hook but currently no seat→session index, so we handle
      // that client-side for v0.1 using the in-memory store populated by
      // SessionsWorkspace (R4 will unify).
      if (activeContact.kind === 'seat' && activeContact.seatName) {
        const sid = sessionFor(activeContact.seatName);
        if (sid) { api.ptyWrite(sid, text + '\n').catch(() => {}); }
      }
      setInput('');
      setSendError(null);
    } catch (e) {
      setSendError(String(e));
    }
  };

  // --- Layout ---
  const container: React.CSSProperties = embedded
    ? { position: 'fixed', right: 20, bottom: 20, width: 720, height: 560, borderRadius: 'var(--sl-radius-lg)', boxShadow: '0 12px 40px rgba(0,0,0,0.25)' }
    : { position: 'absolute', inset: 0 };

  return (
    <div style={{
      ...container,
      display: 'flex', flexDirection: 'column',
      background: 'var(--sl-surface)',
      border: '1px solid var(--sl-border)',
      overflow: 'hidden', zIndex: 9000,
    }}>
      {/* Header */}
      <div style={{
        display: 'flex', alignItems: 'center', padding: '10px 16px',
        borderBottom: '1px solid var(--sl-border)', background: 'var(--sl-surface-hover)',
        flexShrink: 0, gap: 10,
      }}>
        <span style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)' }}>Supervisor</span>
        <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>· {activeProjectId ?? 'no project'}</span>
        <div style={{ flex: 1 }} />
        {embedded ? (
          <>
            <button
              onClick={onDetach}
              title="Open in separate window"
              style={btnStyle}
            >↗ Detach</button>
            <button onClick={onClose} title="Close" style={btnStyle}>×</button>
          </>
        ) : (
          <button onClick={onReembed} title="Re-embed in main window" style={btnStyle}>
            ↙ Re-embed
          </button>
        )}
      </div>

      {/* Body: contact list + chat */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Contact list */}
        <aside style={{
          width: 200, borderRight: '1px solid var(--sl-border)',
          overflowY: 'auto', background: 'var(--sl-bg)',
        }}>
          {contacts.map((c) => (
            <div
              key={c.id}
              onClick={() => setActiveContactId(c.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                padding: '8px 12px', cursor: 'pointer',
                background: c.id === activeContactId ? 'var(--sl-surface-hover)' : 'transparent',
                borderLeft: c.id === activeContactId ? '2px solid var(--sl-brand)' : '2px solid transparent',
              }}
            >
              <span style={{
                width: 26, height: 26, borderRadius: '50%',
                background: c.color, color: 'white',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 12, fontWeight: 600,
              }}>{c.avatar}</span>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 13, fontWeight: 500, color: 'var(--sl-text-primary)' }}>{c.name}</div>
                <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {c.role ?? (c.kind === 'supervisor' ? 'Global supervision' : '')}
                </div>
              </div>
              {c.online && <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--sl-green)' }} />}
            </div>
          ))}
        </aside>

        {/* Chat pane */}
        <main style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <div style={{
            display: 'flex', alignItems: 'center', gap: 10,
            padding: '8px 16px', borderBottom: '1px solid var(--sl-divider)',
            flexShrink: 0,
          }}>
            <span style={{
              width: 28, height: 28, borderRadius: '50%',
              background: activeContact?.color, color: 'white',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontSize: 12, fontWeight: 600,
            }}>{activeContact?.avatar}</span>
            <div>
              <div style={{ fontSize: 13, fontWeight: 600 }}>{activeContact?.name}</div>
              <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>{activeContact?.role ?? 'supervisor'}</div>
            </div>
          </div>

          <div style={{ flex: 1, overflowY: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 8 }}>
            {messagesLoading && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)' }}>Loading…</div>
            )}
            {!messagesLoading && messages.length === 0 && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)', alignSelf: 'center', marginTop: 40 }}>
                No messages yet. Start a conversation with {activeContact?.name}.
              </div>
            )}
            {messages.map((m) => {
              const isUser = (m.actorRef ?? '').startsWith('human');
              const content = (m.payload as any)?.content ?? m.eventType;
              return (
                <div key={m.id} style={{
                  alignSelf: isUser ? 'flex-end' : 'flex-start',
                  maxWidth: '72%', padding: '8px 12px',
                  background: isUser ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
                  color: isUser ? 'white' : 'var(--sl-text-primary)',
                  borderRadius: 'var(--sl-radius-md)',
                  fontSize: 13, whiteSpace: 'pre-wrap', wordBreak: 'break-word',
                }}>
                  {m.eventType !== 'SupervisorMessage' && m.eventType !== 'SeatResponse' && (
                    <div style={{ fontSize: 10, fontWeight: 600, opacity: 0.7, marginBottom: 2 }}>{m.eventType}</div>
                  )}
                  {content}
                  <div style={{ fontSize: 10, opacity: 0.6, marginTop: 2 }}>
                    {new Date(m.occurredAt).toLocaleTimeString()}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Input */}
          <div style={{ borderTop: '1px solid var(--sl-divider)', padding: '10px 14px', flexShrink: 0, display: 'flex', gap: 8, alignItems: 'center' }}>
            <input
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder={`Message ${activeContact?.name}...`}
              onKeyDown={(e) => {
                if (e.key === 'Escape') onClose?.();
                if (e.key === 'Enter' && input.trim()) handleSend();
              }}
              style={{
                flex: 1, padding: '8px 12px', fontSize: 13,
                background: 'var(--sl-bg)', border: '1px solid var(--sl-border)',
                borderRadius: 'var(--sl-radius-md)', color: 'var(--sl-text-primary)',
                outline: 'none',
              }}
            />
            <button
              onClick={handleSend}
              disabled={!input.trim()}
              style={{
                padding: '8px 16px', fontSize: 13, fontWeight: 600,
                color: input.trim() ? 'white' : 'var(--sl-text-tertiary)',
                background: input.trim() ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
                border: 'none', borderRadius: 'var(--sl-radius-md)',
                cursor: input.trim() ? 'pointer' : 'default',
              }}
            >Send</button>
          </div>
          {sendError && (
            <div style={{ padding: '4px 14px 8px', fontSize: 11, color: 'var(--sl-red)' }}>
              {sendError}
            </div>
          )}
        </main>
      </div>
    </div>
  );
};

const btnStyle: React.CSSProperties = {
  padding: '2px 8px', fontSize: 12,
  background: 'transparent', border: '1px solid var(--sl-border)',
  borderRadius: 4, color: 'var(--sl-text-secondary)', cursor: 'pointer',
};
