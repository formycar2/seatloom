// useSupervisorData — derives ChatContact[] and ChatMessage[] from real
// PostgreSQL data so SupervisorPanel renders the V2 IM UI against live data
// without changing the panel's UI/UX.
//
// - useContacts(): synthesises a stable ChatContact[] in the same shape the V2
//   IM expects (supervisor + project-channel + seat contacts) from
//   useDataStore.projects + projectData.seats. Project channels are one-per-
//   project (id = 'ch-' + project.id). Seat contacts derive seatType from the
//   coerced V1 role and project from the active project (v0.1 single-project).
//
// - useMessages(contactId): loads supervisor-flavoured canonical_events for
//   that contact via cmd_list_supervisor_messages and live-appends on the
//   backend's canonical:appended event. Maps each event row to the V2
//   ChatMessage shape so MessageBubble renders them as text bubbles.

import { useEffect, useMemo, useState } from 'react';
import { ChatContact, ChatMessage } from '../types';
import { useDataStore } from '../../stores/useDataStore';
import { api, isTauri, onCanonicalAppended } from '../../lib/api';
import type { CanonicalEventDto } from '../../lib/types-dto';
import { Seat } from '../../types';

const ROLE_TONES: Record<string, string> = {
  ProductOwner: 'var(--sl-purple)',
  Architect: 'var(--sl-blue)',
  Designer: 'var(--sl-amber)',
  Verifier: 'var(--sl-green)',
  Developer: 'var(--sl-teal)',
};

const ROLE_SEATTYPE: Record<string, ChatContact['seatType']> = {
  ProductOwner: 'po',
  Architect: 'worker',
  Designer: 'worker',
  Developer: 'worker',
  Verifier: 'verifier',
};

const ROLE_LABEL: Record<string, string> = {
  ProductOwner: '产品负责人',
  Architect: '架构师',
  Designer: 'UX 设计',
  Verifier: '质量验证',
  Developer: '工程实现',
};

function roleKey(seat: Seat): string {
  if (typeof seat.role === 'string') return seat.role;
  if ((seat.role as any).Custom) return (seat.role as any).Custom;
  return 'Developer';
}

function seatToContact(seat: Seat, projectId: string): ChatContact {
  const r = roleKey(seat);
  return {
    id: `${projectId}-${seat.name}`,
    name: seat.name.charAt(0).toUpperCase() + seat.name.slice(1),
    type: 'seat',
    role: ROLE_LABEL[r] ?? r,
    seatType: ROLE_SEATTYPE[r] ?? 'worker',
    online: seat.status === 'Active',
    projectId,
    unread: 0,
    lastMessage: undefined,
    lastTime: undefined,
    avatar: seat.name.charAt(0).toUpperCase(),
    color: ROLE_TONES[r] ?? 'var(--sl-text-secondary)',
  };
}

const PROJECT_CHANNEL_TONES = ['var(--sl-teal)', 'var(--sl-blue)', 'var(--sl-purple)', 'var(--sl-amber)'];

export function useContacts(): ChatContact[] {
  const projects = useDataStore((s) => s.projects);
  const projectData = useDataStore((s) => s.projectData);

  return useMemo(() => {
    const out: ChatContact[] = [];
    out.push({
      id: 'supervisor',
      name: 'Aegis · Supervisor',
      type: 'supervisor',
      online: true,
      unread: 0,
      lastMessage: undefined,
      lastTime: undefined,
      avatar: 'A',
      color: 'var(--sl-brand)',
    });
    projects.forEach((p, idx) => {
      out.push({
        id: 'ch-' + p.id,
        name: p.name,
        type: 'project-channel',
        online: true,
        projectId: p.id,
        unread: 0,
        avatar: '#',
        color: PROJECT_CHANNEL_TONES[idx % PROJECT_CHANNEL_TONES.length],
      });
      const data = projectData[p.id];
      if (data) {
        // Skip Aegis here because it already appears as the global supervisor.
        for (const seat of data.seats) {
          if (seat.name.toLowerCase() === 'aegis') continue;
          out.push(seatToContact(seat, p.id));
        }
      }
    });
    return out;
  }, [projects, projectData]);
}

function pad2(n: number): string {
  return String(n).padStart(2, '0');
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

function eventToMessage(evt: CanonicalEventDto, contactId: string): ChatMessage {
  const payload = (evt.payload ?? {}) as Record<string, any>;
  const fromUser = (evt.actorRef ?? '').toLowerCase().startsWith('human');

  // All ledger-derived bubbles render as plain text — the V2 MessageBubble
  // expects a populated `card` object for any non-text type, and we don't
  // have one for synthesized activity-feed lines. The emoji prefix carries
  // the category visually (📤 / ✅ / 📋 / etc.) without needing card chrome.
  let content: string;
  if (typeof payload.content === 'string') {
    content = payload.content;
  } else {
    const etype = evt.eventType;
    switch (etype) {
      case 'SessionStarted':
        content = `🟢 Session started — ${payload.session_id ?? ''}`;
        break;
      case 'SessionCompleted':
        content = `⚪ Session completed — ${payload.session_id ?? ''}`;
        break;
      case 'SessionFailed':
      case 'SessionInterrupted':
        content = `🔴 ${etype} — ${payload.session_id ?? ''}`;
        break;
      case 'ArtifactCreated':
        content = `📄 Artifact created — ${payload.artifact_id ?? payload.title ?? ''}`;
        break;
      case 'HandoffDrafted':
      case 'HandoffSent':
      case 'HandoffReceived':
      case 'HandoffAccepted':
      case 'HandoffWorking':
      case 'HandoffReturned':
      case 'HandoffCompleted':
        content = `📤 ${etype} — ${payload.handoff_id ?? ''}`;
        break;
      case 'WorkItemCreated':
        content = `📋 WorkItem created — ${payload.title ?? payload.workitem_id ?? ''}`;
        break;
      case 'WorkItemStatusChanged':
        content = `📋 WorkItem status → ${payload.new_status ?? ''} (${payload.workitem_id ?? ''})`;
        break;
      case 'ReviewVerdictIssued':
        content = `✅ Review verdict: ${payload.verdict ?? ''} — ${payload.workitem_id ?? ''}`;
        break;
      case 'SeatDelegationIssued':
        content = `🔁 Delegation issued: ${payload.from_seat ?? ''} → ${payload.to_seat ?? ''}`;
        break;
      case 'SeatDelegationClosed':
        content = `🔁 Delegation closed — ${payload.delegation_id ?? ''}`;
        break;
      case 'CheckpointCreated':
        content = `📌 Checkpoint created`;
        break;
      case 'PromptDetected':
        content = `⚠️ Prompt detected (${payload.prompt_kind ?? ''})`;
        break;
      case 'SupervisorMessage':
      case 'SeatResponse':
        content = payload.content ?? '[empty message]';
        break;
      default:
        content = `• ${etype}`;
    }
  }

  return {
    id: evt.id,
    contactId,
    from: fromUser ? 'user' : 'contact',
    content,
    time: formatTime(evt.occurredAt),
    type: 'text',
  };
}

/**
 * Live-loaded messages for a contact. Returns:
 *   - the chat history (sorted ascending by time) loaded from
 *     cmd_list_supervisor_messages on contact change
 *   - newly appended messages received via canonical:appended
 *
 * Routing rules per contact type:
 *   - seat              → events authored by the seat OR addressed to it
 *   - supervisor        → all events (global activity feed)
 *   - project-channel   → all events for the project (v0.1 single project: all)
 *
 * Effects depend only on the contact identity (id + type) so changes to other
 * fields on the ChatContact object never re-subscribe.
 */
export function useMessages(contact: ChatContact | null): ChatMessage[] {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const contactId = contact?.id ?? null;
  const contactType = contact?.type ?? null;

  useEffect(() => {
    if (!contact || !contactId) { setMessages([]); return; }
    if (!isTauri()) { setMessages([]); return; }
    const targetSeatId = contact.type === 'seat'
      ? backendSeatIdFromContact(contact)
      : undefined;
    let cancelled = false;
    api.listSupervisorMessages({ targetSeatId, limit: 400 })
      .then((rows) => {
        if (cancelled) return;
        setMessages(rows.map((r) => eventToMessage(r, contact.id)));
      })
      .catch((e) => {
        console.error('[useMessages] listSupervisorMessages failed:', e);
        if (!cancelled) setMessages([]);
      });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contactId, contactType]);

  useEffect(() => {
    if (!contact || !contactId) return;
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    onCanonicalAppended((evt) => {
      if (cancelled) return;
      const targetFromPayload = (evt.payload as any)?.target_seat_id ?? null;
      const actorRef = evt.actorRef ?? '';
      let accept = false;
      if (contact.type === 'seat') {
        const expected = backendSeatIdFromContact(contact);
        if (!expected) return;
        accept = actorRef === `seat:${expected}` || targetFromPayload === expected;
      } else if (contact.type === 'supervisor' || contact.type === 'project-channel') {
        accept = true;
      }
      if (!accept) return;
      setMessages((prev) => {
        if (prev.some((m) => m.id === evt.id)) return prev;
        return [...prev, eventToMessage(evt, contact.id)];
      });
    }).then((fn) => {
      if (cancelled) { fn(); return; }
      unlisten = fn;
    }).catch((e) => {
      console.error('[useMessages] onCanonicalAppended failed:', e);
    });
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contactId, contactType]);

  return messages;
}

/**
 * Map a V2 ChatContact id (e.g. 'p-1-lyra') to the backend seat id
 * (e.g. 'seat-lyra-001'). The backend seats have stable ids; we look them
 * up via useDataStore by matching the lowercase seat name suffix.
 */
function backendSeatIdFromContact(contact: ChatContact): string | undefined {
  if (contact.type !== 'seat') return undefined;
  const seats = useDataStore.getState().projectData[contact.projectId ?? '']?.seats ?? [];
  const lcName = contact.name.toLowerCase();
  return seats.find((s) => s.name.toLowerCase() === lcName)?.id;
}

export { backendSeatIdFromContact };
