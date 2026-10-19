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
import {
  eventToText,
  type DispatchFact,
  type EventTextContext,
  type MemoryFact,
} from '../lib/eventText';
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
  const custom = (seat.role as { Custom?: unknown }).Custom;
  if (typeof custom === 'string' && custom) return custom;
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

const PROJECT_CHANNEL_TONES = [
  'var(--sl-teal)',
  'var(--sl-blue)',
  'var(--sl-purple)',
  'var(--sl-amber)',
];

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

/**
 * Resolve the ids a ledger line carries into things a person recognises.
 *
 * The events themselves carry ids and titles only where the writer put them: a
 * `SeatDispatchIssued` row says `dispatch_id` and nothing else, so "which work,
 * to whom" is only answerable by reading the dispatch table. That read happens
 * here, once per mount, and is handed to the translator as an
 * [`EventTextContext`].
 *
 * Two passes, because the memory titles are only worth fetching for the topics
 * the loaded events actually mention — fetching every topic's memories on every
 * panel open would be tens of calls for a handful of rows.
 *
 * Every failure degrades to "没取到" inside the line. Nothing here may invent a
 * title, a seat name or a status.
 */
function useEventTextContext(events: CanonicalEventDto[]): EventTextContext {
  const projectData = useDataStore((s) => s.projectData);
  const [ctx, setCtx] = useState<EventTextContext>({});

  // Seat ids -> display names, from the store that already holds both.
  const seatNames = useMemo(() => {
    const m = new Map<string, string>();
    for (const data of Object.values(projectData)) {
      for (const seat of data.seats) m.set(seat.id, seat.name);
    }
    return m;
  }, [projectData]);

  // Scoped to the selected project. The unscoped call hit the backend's
  // default-project fallback, so dispatch titles were resolved against seatloom's
  // dispatches no matter which project's events were on screen.
  const activeProjectId = useDataStore((s) => s.activeProjectId);

  useEffect(() => {
    if (!isTauri()) return;
    if (!activeProjectId) return;
    let cancelled = false;
    api
      .listDispatches(activeProjectId, 500)
      .then((rows) => {
        if (cancelled) return;
        const dispatches = new Map<string, DispatchFact>();
        for (const r of rows) {
          dispatches.set(r.id, {
            title: r.title,
            targetSeatId: r.targetSeatId,
            status: r.status,
          });
        }
        setCtx((prev) => ({ ...prev, dispatches, seatNames }));
      })
      .catch((e) => {
        // A dispatch list that will not load must not blank the panel: the lines
        // still render, each saying its title was not resolved.
        console.error('[useEventTextContext] listDispatches failed:', e);
        setCtx((prev) => ({ ...prev, seatNames }));
      });
    return () => {
      cancelled = true;
    };
  }, [seatNames, activeProjectId]);

  // Second pass: memory titles for the topics these events actually mention.
  const topicIds = useMemo(() => {
    const ids = new Set<string>();
    for (const evt of events) {
      const p = (evt.payload ?? {}) as Record<string, unknown>;
      const t = typeof p.topic_id === 'string' ? p.topic_id : null;
      if (t) ids.add(t);
    }
    return [...ids].sort();
  }, [events]);

  useEffect(() => {
    if (!isTauri() || topicIds.length === 0) return;
    let cancelled = false;
    Promise.all(
      topicIds.map((topicId) =>
        api.getTopicMemories(topicId, 'l0', false, 200).then(
          (res) => [topicId, res.memories] as const,
          () => [topicId, []] as const,
        ),
      ),
    )
      .then((rows) => {
        if (cancelled) return;
        const memories = new Map<string, MemoryFact>();
        for (const [topicId, cards] of rows) {
          for (const c of cards) memories.set(c.memoryId, { title: c.title, topicId });
        }
        setCtx((prev) => ({ ...prev, memories }));
      })
      .catch(() => {
        /* leave the memory titles unresolved; the lines say so */
      });
    return () => {
      cancelled = true;
    };
  }, [topicIds]);

  return ctx;
}

/**
 * The raw ledger rows for a contact, loaded once per contact identity and
 * live-appended from the backend's `canonical:appended` event.
 *
 * Split out from `useMessages` so the rows can feed the id resolver as well as
 * the message list — a `MemoryRecalled` row has to be visible before its topic's
 * memories are worth fetching.
 */
function useSupervisorEvents(contact: ChatContact | null): CanonicalEventDto[] {
  const [events, setEvents] = useState<CanonicalEventDto[]>([]);
  const contactId = contact?.id ?? null;
  const contactType = contact?.type ?? null;

  useEffect(() => {
    if (!contact || !contactId) {
      setEvents([]);
      return;
    }
    if (!isTauri()) {
      setEvents([]);
      return;
    }
    const targetSeatId = contact.type === 'seat' ? backendSeatIdFromContact(contact) : undefined;
    let cancelled = false;
    api
      .listSupervisorMessages({ targetSeatId, limit: 400 })
      .then((rows) => {
        if (!cancelled) setEvents(rows);
      })
      .catch((e) => {
        console.error('[useSupervisorEvents] listSupervisorMessages failed:', e);
        if (!cancelled) setEvents([]);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contactId, contactType]);

  useEffect(() => {
    if (!contact || !contactId) return;
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    onCanonicalAppended((evt) => {
      if (cancelled) return;
      const targetFromPayload =
        (evt.payload as { target_seat_id?: unknown } | null)?.target_seat_id ?? null;
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
      setEvents((prev) => (prev.some((e) => e.id === evt.id) ? prev : [...prev, evt]));
    })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
      })
      .catch((e) => {
        console.error('[useSupervisorEvents] onCanonicalAppended failed:', e);
      });
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contactId, contactType]);

  return events;
}

export function useMessages(contact: ChatContact | null): ChatMessage[] {
  const events = useSupervisorEvents(contact);
  const ctx = useEventTextContext(events);

  return useMemo(
    () =>
      events.map((evt) => {
        const fromUser = (evt.actorRef ?? '').toLowerCase().startsWith('human');
        return {
          id: evt.id,
          contactId: contact?.id ?? '',
          from: fromUser ? ('user' as const) : ('contact' as const),
          content: eventToText(evt, ctx),
          time: formatTime(evt.occurredAt),
          type: 'text' as const,
        };
      }),
    [events, ctx, contact?.id],
  );
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
