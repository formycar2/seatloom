// Parsing for the SeatLoom tmux session naming convention.
//
// Seat sessions are named `<Seat>-<Role>-seatloom`, e.g. `Mira-UX/UED-seatloom`
// or `Flux-Quality&Ops-seatloom`. The `-seatloom` suffix is load-bearing:
// crates/seatloom-core/src/pty/mod.rs discovers sessions by that suffix, so the
// backend only ever hands us names that end in it.
//
// The role segment may itself contain `-`, `/` and `&`, so we peel the suffix
// first and split on the FIRST hyphen only.

import type { TmuxSessionInfo } from './types-dto';

const SUFFIX = '-seatloom';

export interface ParsedSeat {
  /** Full tmux session name, as the backend reported it. Attach key. */
  sessionName: string;
  /** Seat name, e.g. 'Mira'. Falls back to the whole stem if unparseable. */
  seatName: string;
  /** Role segment, e.g. 'UX/UED'. Empty when the name carries no role. */
  role: string;
  /** True when a tmux client is currently attached to the session. */
  attached: boolean;
  /** Unix seconds from `#{session_created}`. */
  createdAt: number;
}

/**
 * Split a discovered tmux session into seat + role. Never throws: an
 * unrecognised shape degrades to `seatName = <stem>`, `role = ''` so an
 * unexpected session still renders instead of disappearing.
 */
export function parseSeatSession(info: TmuxSessionInfo): ParsedSeat {
  const name = info.session_name;
  const stem = name.endsWith(SUFFIX) ? name.slice(0, -SUFFIX.length) : name;
  const dash = stem.indexOf('-');
  return {
    sessionName: name,
    seatName: dash > 0 ? stem.slice(0, dash) : stem,
    role: dash > 0 ? stem.slice(dash + 1) : '',
    attached: info.attached,
    createdAt: info.created_at,
  };
}

/** Stable sort so the rail does not reorder under a 3s poll. */
export function sortSeats(seats: ParsedSeat[]): ParsedSeat[] {
  return [...seats].sort((a, b) => a.seatName.localeCompare(b.seatName));
}

/** Store key used by useLiveSessionsStore (seat name, lowercased). */
export function seatKey(seatName: string): string {
  return seatName.toLowerCase();
}
