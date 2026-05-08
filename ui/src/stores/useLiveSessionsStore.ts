// Shared zustand store tracking which SeatLoom seat names currently have live
// PTY sessions — so the Supervisor IM can route chat messages into the right
// session.
//
// Keyed by seat name (lowercase; e.g. 'lyra', 'nimbus') because that's the
// stable identifier that spans backend SeatDto and frontend ChatContact.
//
// SessionsWorkspace registers on launch; the PTY exit listener deregisters.
// SupervisorPanel reads via sessionFor(seatName) to decide whether to pipe a
// chat message into the PTY.

import { create } from 'zustand';

interface LiveSessionsState {
  sessionBySeat: Record<string, string>; // seatName -> sessionId
  setSessionForSeat: (seatName: string, sessionId: string) => void;
  clearSessionForSeat: (seatName: string) => void;
  clearSession: (sessionId: string) => void;
  sessionFor: (seatName: string | undefined) => string | null;
}

export const useLiveSessionsStore = create<LiveSessionsState>((set, get) => ({
  sessionBySeat: {},

  setSessionForSeat: (seatName, sessionId) => {
    const key = seatName.toLowerCase();
    set((state) => ({
      sessionBySeat: { ...state.sessionBySeat, [key]: sessionId },
    }));
  },

  clearSessionForSeat: (seatName) => {
    const key = seatName.toLowerCase();
    set((state) => {
      const next = { ...state.sessionBySeat };
      delete next[key];
      return { sessionBySeat: next };
    });
  },

  clearSession: (sessionId) => {
    set((state) => {
      const next: Record<string, string> = {};
      for (const [k, v] of Object.entries(state.sessionBySeat)) {
        if (v !== sessionId) next[k] = v;
      }
      return { sessionBySeat: next };
    });
  },

  sessionFor: (seatName) => {
    if (!seatName) return null;
    return get().sessionBySeat[seatName.toLowerCase()] ?? null;
  },
}));
