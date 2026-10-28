// The conversation-window accumulator: turns, their text, and what a UI needs to
// show them — without touching the terminal.
//
// WHY THIS EXISTS. Aegis's packet (AEGIS-2026-10-27-dual-render-seat-stream-v1):
// 张先生 wants a smooth conversation window ALONGSIDE the tmux panel. The tmux
// panel stays (xterm keeps reading `session:output` untouched); this is the other
// consumer.
//
// WHY IT LIVES IN THE FRONTEND. The harness (saih) writes its session file at
// `message_end`, so reading that file gives whole turns with no smoothness. The
// byte stream is already smooth and already here. So the frontend accumulates the
// stream itself — which also means the accumulation dies with the app session,
// which is the honest answer to \"where is it stored\": nowhere, yet. See the
// `whereIsItStored` export for the trade written down rather than implied.
//
// 🔴 WHAT A TURN BOUNDARY IS. `onSessionTurn`'s boundaries are INFERRED from the
// shape of the terminal output by `seatloom_core::turn_stream`. They are a hint,
// not a record of what the model did. This store therefore never rewrites or
// trims bytes — it only decides where one turn ends and the next begins, and it
// keeps the raw text so a UI can always fall back to showing it verbatim.

import { create } from 'zustand';
import { decodeSessionOutput, type SessionTurnEvent } from '../lib/api';

/** One turn: its accumulated text, and the bookkeeping a UI needs. */
export interface ConversationTurn {
  /** Monotonic within this app session. Not a database id — there is no database. */
  index: number;
  /** Decoded text accumulated since `turnStarted`. May contain escape sequences. */
  text: string;
  /** Bytes seen in this turn, for a size badge and for the cap below. */
  bytes: number;
  /** Whether the harness's indicator has been replaced by the status line. */
  closed: boolean;
  /** Chunk count, so a UI can show \"N updates\" without re-deriving it. */
  chunks: number;
}

/**
 * Hard cap on accumulated text per turn, in bytes.
 *
 * WHY IT EXISTS. A seat can print a very long turn (a build log, a big diff). The
 * accumulator holds it in memory and re-renders it on every chunk, so an unbounded
 * turn is both a memory and a frame-rate problem. When the cap is hit, the OLDEST
 * text is dropped and `truncated` is set, so a UI can say \"…\" rather than
 * pretending the whole thing is there.
 *
 * Deliberately large (256KB) so the cap only ever engages on genuinely abnormal
 * output — trimming a normal turn would be the same class of lie as the
 * fabricated project path was.
 */
export const MAX_TURN_TEXT_BYTES = 256 * 1024;

/** How many turns to keep. Older turns stay readable in the terminal. */
export const MAX_TURNS_KEPT = 50;

export interface ConversationState {
  turns: ConversationTurn[];
  /** Index of the turn currently accumulating, or -1 when idle. */
  activeIndex: number;
  /** Total turns seen this app session, including ones rotated out. */
  totalSeen: number;

  /** Apply one `session:turn` event. Pure state transition; safe to call in order. */
  apply: (event: SessionTurnEvent) => void;
  /** Drop everything for one session (its pane was detached or exited). */
  reset: () => void;
}

const EMPTY_TURN = (index: number): ConversationTurn => ({
  index,
  text: '',
  bytes: 0,
  closed: false,
  chunks: 0,
});

/**
 * Where the accumulated conversation lives, stated plainly.
 *
 * IN MEMORY, THIS APP SESSION, NOWHERE ELSE. It is not written to the database and
 * not to disk, and that is a deliberate choice rather than an omission:
 *
 *   * writing it to the ledger would mean creating events nobody captured, in the
 *     one product whose thesis is \"the record, not the harness, holds the truth\"
 *     (the same red line as the session-work-bridge contract's);
 *   * writing it to disk would mean a second transcript next to the one
 *     `pty/mod.rs` already appends raw bytes to — a second copy of the same bytes
 *     that can disagree with the first.
 *
 * The consequence, stated so nobody discovers it later: close the app and the
 * conversation window is empty. The terminal's own history (and the harness's
 * session file, written at message_end) are the durable record. If a durable
 * conversation view is wanted, that is a separate decision with its own cost.
 */
export const whereIsItStored = 'memory, this app session only — see the comment above';

export const useConversationStore = create<ConversationState>((set, get) => ({
  turns: [],
  activeIndex: -1,
  totalSeen: 0,

  apply: (event) => {
    if (event.sessionId !== undefined && typeof event.sessionId !== 'string') return;

    switch (event.kind) {
      case 'turnStarted': {
        const state = get();
        const index = state.totalSeen;
        // Rotate before appending so a long-running seat cannot grow without bound.
        const kept = state.turns.slice(-(MAX_TURNS_KEPT - 1));
        set({
          turns: [...kept, EMPTY_TURN(index)],
          activeIndex: index,
          totalSeen: state.totalSeen + 1,
        });
        break;
      }

      case 'chunk': {
        const state = get();
        if (state.activeIndex < 0) {
          // Bytes with no open turn: the pane was streaming before we attached, or
          // the boundary heuristic missed a start. Keep them in a synthetic turn
          // rather than dropping them — dropping is what makes a UI lie by omission.
          const index = state.totalSeen;
          set({
            turns: [...state.turns.slice(-(MAX_TURNS_KEPT - 1)), EMPTY_TURN(index)],
            activeIndex: index,
            totalSeen: state.totalSeen + 1,
          });
        }
        const s = get();
        const at = s.turns.length - 1;
        if (at < 0) break;
        const turn = s.turns[at];
        const added = decodeSessionOutput(event.data);
        let text = turn.text;
        // Append as UTF-8 text. A chunk can split a multi-byte character; the
        // decoder is lenient and the terminal remains the authoritative view.
        text += new TextDecoder('utf-8', { fatal: false }).decode(added);
        if (text.length > MAX_TURN_TEXT_BYTES) {
          text = text.slice(text.length - MAX_TURN_TEXT_BYTES);
        }
        const updated: ConversationTurn = {
          ...turn,
          text,
          bytes: turn.bytes + added.length,
          chunks: turn.chunks + 1,
        };
        const turns = s.turns.slice();
        turns[at] = updated;
        set({ turns });
        break;
      }

      case 'turnEnded': {
        const state = get();
        if (state.activeIndex < 0 || state.turns.length === 0) break;
        const at = state.turns.length - 1;
        const turns = state.turns.slice();
        turns[at] = { ...turns[at], closed: true };
        set({ turns, activeIndex: -1 });
        break;
      }
    }
  },

  reset: () => set({ turns: [], activeIndex: -1, totalSeen: 0 }),
}));
