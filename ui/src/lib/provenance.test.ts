// UM0 close-out evidence, in code:
//   1. the healthy path — the five canonical PI worker seats parse cleanly;
//   2. HOLD-3 — the live mirror only wears LIVE after discovery returns seats.

import { describe, expect, it } from 'vitest';
import { LIVE_MIRROR_TAB, provenanceForTab } from './provenance';
import { parseSeatSession, sortSeats } from './tmux-seats';
import type { TmuxSessionInfo } from './types-dto';

const CANONICAL_SEATS: Array<[session: string, seat: string, role: string]> = [
  ['Flux-Quality&Ops-seatloom', 'Flux', 'Quality&Ops'],
  ['Lyra-po-seatloom', 'Lyra', 'po'],
  ['Mira-UX/UED-seatloom', 'Mira', 'UX/UED'],
  ['Nimbus-TechArchi-seatloom', 'Nimbus', 'TechArchi'],
  ['Onyx-data-seatloom', 'Onyx', 'data'],
];

const info = (session_name: string): TmuxSessionInfo => ({
  session_name,
  created_at: 0,
  attached: false,
});

describe('parseSeatSession — the five canonical PI worker seats', () => {
  it.each(CANONICAL_SEATS)('parses %s', (session, seat, role) => {
    const parsed = parseSeatSession(info(session));
    expect(parsed.sessionName).toBe(session);
    expect(parsed.seatName).toBe(seat);
    expect(parsed.role).toBe(role);
  });

  it('keeps all five discoverable and stably ordered', () => {
    const parsed = sortSeats(CANONICAL_SEATS.map(([s]) => parseSeatSession(info(s))));
    expect(parsed.map((p) => p.seatName)).toEqual(['Flux', 'Lyra', 'Mira', 'Nimbus', 'Onyx']);
  });
});

describe('provenanceForTab — live mirror refuses a false LIVE badge (HOLD-3)', () => {
  it('declares live only once discovery has returned seats', () => {
    expect(provenanceForTab(LIVE_MIRROR_TAB, true, 'ok')).toBe('live');
  });

  it('empty discovery renders no-live, never live', () => {
    expect(provenanceForTab(LIVE_MIRROR_TAB, true, 'empty')).toBe('no-live');
    expect(provenanceForTab(LIVE_MIRROR_TAB, false, 'empty')).toBe('no-live');
  });

  it('failed discovery renders discovery-failed, never live', () => {
    expect(provenanceForTab(LIVE_MIRROR_TAB, true, 'failed')).toBe('discovery-failed');
  });

  it('leaves every non-mirror surface exactly as before', () => {
    expect(provenanceForTab('inbox', true)).toBe('seed');
    expect(provenanceForTab('inbox', false)).toBe('fixture');
    expect(provenanceForTab('sessions', true, 'empty')).toBe('live');
  });
});
