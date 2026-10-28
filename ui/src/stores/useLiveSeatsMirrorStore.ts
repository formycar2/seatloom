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
   * Seats whose mirror the backend could no longer prove, keyed by seat name.
   *
   * This is the whole point of the 2026-10-27 fix. The incident was not "the app
   * showed an empty slot" — it was "the app showed one seat's output in another
   * seat's slot for three hours and said nothing". A mirror that cannot be
   * proven has to be VISIBLE as broken, and the failure has to be attributable to
   * a seat so the operator knows which one to reconnect.
   *
   * Deliberately separate from `attachments` rather than a flag on it: a broken
   * attachment is REMOVED from `attachments` (that is what stops the terminal
   * from rendering), and this map is what remains to say why.
   */
  mirrorLost: Record<string, MirrorLostState>;
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
  /**
   * Record that a seat's mirror is no longer provably ours, and drop its
   * attachment so the terminal stops rendering bytes we cannot vouch for.
   *
   * Both halves matter: the attachment removal is what makes the UI stop showing
   * foreign content, and the record is what makes the breakage visible and
   * attributable instead of looking like an ordinary disconnect.
   */
  markMirrorLost: (
    seatName: string,
    sessionId: string,
    reason: string,
    detail: string,
  ) => void;
  /** Clear a seat's mirror-lost state (operator reconnected). */
  clearMirrorLost: (seatName: string) => void;
  markAutoAttached: () => void;
}

/// Why a mirror stopped being shown, and what the operator should do.
///
/// `reason` is the backend's `MirrorVerdict::as_str()` — `lost`, `cross_wired`
/// or `probe_failed` — passed through verbatim rather than mapped to a friendlier
/// enum here, because a renamed reason is a reason nobody can grep the logs for.
export interface MirrorLostState {
  sessionId: string;
  reason: string;
  detail: string;
  at: number;
}

export const useLiveSeatsMirrorStore = create<LiveSeatsMirrorState>((set, get) => ({
  attachments: [],
  activity: {},
  activeName: null,
  mirrorLost: {},
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

  markMirrorLost: (seatName, sessionId, reason, detail) => {
    const { attachments, activeName } = get();
    const wasActive = attachments.some(
      (a) => a.seat.sessionName === seatName && a.seat.sessionName === activeName,
    );
    // Remove first: this is what unmounts the terminal and stops the bytes.
    get().removeAttachment(sessionId);
    set((state) => ({
      mirrorLost: {
        ...state.mirrorLost,
        [seatName]: { sessionId, reason, detail, at: Date.now() },
      },
      // If the seat on screen just lost its mirror, fall back to another
      // attachment rather than leaving a blank terminal area — the banner above
      // the terminal is where the operator learns what happened.
      activeName: wasActive
        ? (get().attachments.length
            ? get().attachments[get().attachments.length - 1].seat.sessionName
            : null)
        : state.activeName,
    }));
  },

  clearMirrorLost: (seatName) =>
    set((state) => {
      if (!(seatName in state.mirrorLost)) return state;
      const mirrorLost = { ...state.mirrorLost };
      delete mirrorLost[seatName];
      return { mirrorLost };
    }),

  markAutoAttached: () => {
    if (!get().autoAttached) set({ autoAttached: true });
  },
}));
