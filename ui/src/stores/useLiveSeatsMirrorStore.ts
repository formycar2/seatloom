// useLiveSeatsMirrorStore — the live-seats mirror state that must outlive the view.
//
// Why this exists (defect, fixed 2026-10-16):
//
//   `LiveSeatsView` is conditionally rendered by `App.tsx` (`activeTab ===
//   'live-seats'`), so leaving the tab unmounts it and coming back remounts it.
//   The attachments — which seat is mirrored by which SeatLoom session — lived in
//   the component's own `useState`, so they evaporated on unmount while the
//   backend sessions kept running. The operator saw "reconnect every seat" and
//   reasonably assumed it was by design. It was not: the view had simply
//   forgotten what the backend still held.
//
// The state that has to survive is exactly the state the backend does NOT
// re-announce: the seat↔session mapping, the byte/chunk counters, and which
// pane is on screen. Hoisting it here makes the view a projection of durable
// state rather than the owner of it.
//
// Scope discipline: this store is new and is imported by `LiveSeatsView` only.
// `useLiveSessionsStore` (seatName → sessionId, for Supervisor IM routing)
// already survives unmount and is left exactly as it was — this file does not
// change any other view's behaviour.

import { create } from 'zustand';
import type { LiveSessionDto } from '../lib/types-dto';
import type { ParsedSeat } from '../lib/tmux-seats';

/** One mirrored seat: what it is, and the SeatLoom session streaming its pane. */
export interface Attachment {
  seat: ParsedSeat;
  dto: LiveSessionDto;
}

/** What the real `session:output` stream has delivered for one session. */
export interface Activity {
  bytes: number;
  chunks: number;
  lastAt: number | null;
}

export const EMPTY_ACTIVITY: Activity = { bytes: 0, chunks: 0, lastAt: null };

interface LiveSeatsMirrorState {
  attachments: Attachment[];
  activity: Record<string, Activity>;
  activeName: string | null;
  /**
   * Whether the first-seat auto-attach has already fired in this app session.
   *
   * This is the deliberate design that must NOT regress: on a genuinely first
   * open the view attaches the first discovered seat so it is already moving
   * instead of asking for a click to prove itself. It used to be a `useRef`,
   * which reset on every remount — so a remount re-attached seat 0 on top of a
   * connection that was still alive. The backend inserts unconditionally, so
   * that second attach started a second `pipe-pane` for the same pane. Making
   * the flag durable fixes the duplicate without touching the design.
   */
  autoAttached: boolean;

  addAttachment: (attachment: Attachment) => void;
  removeAttachment: (sessionId: string) => void;
  setActiveName: (name: string | null) => void;
  /** Fold one batch of observed bytes into a session's counters. */
  addBytes: (sessionId: string, bytes: number, chunks: number, lastAt: number) => void;
  /** Drop attachments whose session id is not in `alive`. Returns how many. */
  pruneDead: (alive: Set<string>) => number;
  markAutoAttached: () => void;
}

export const useLiveSeatsMirrorStore = create<LiveSeatsMirrorState>((set, get) => ({
  attachments: [],
  activity: {},
  activeName: null,
  autoAttached: false,

  addAttachment: (attachment) =>
    set((state) => ({
      attachments: [
        ...state.attachments.filter((a) => a.seat.sessionName !== attachment.seat.sessionName),
        attachment,
      ],
    })),

  removeAttachment: (sessionId) =>
    set((state) => {
      const next = state.attachments.filter((a) => a.dto.id !== sessionId);
      const activity = { ...state.activity };
      delete activity[sessionId];
      return {
        attachments: next,
        activity,
        // If the pane on screen went away, fall back to the last remaining one
        // rather than leaving the terminal area blank.
        activeName:
          state.activeName && !next.some((a) => a.seat.sessionName === state.activeName)
            ? next.length
              ? next[next.length - 1].seat.sessionName
              : null
            : state.activeName,
      };
    }),

  setActiveName: (name) => set({ activeName: name }),

  addBytes: (sessionId, bytes, chunks, lastAt) =>
    set((state) => {
      const cur = state.activity[sessionId] ?? EMPTY_ACTIVITY;
      return {
        activity: {
          ...state.activity,
          [sessionId]: {
            bytes: cur.bytes + bytes,
            chunks: cur.chunks + chunks,
            lastAt,
          },
        },
      };
    }),

  pruneDead: (alive) => {
    const { attachments } = get();
    const dead = attachments.filter((a) => !alive.has(a.dto.id));
    if (dead.length === 0) return 0;
    for (const a of dead) get().removeAttachment(a.dto.id);
    return dead.length;
  },

  markAutoAttached: () => {
    if (!get().autoAttached) set({ autoAttached: true });
  },
}));
