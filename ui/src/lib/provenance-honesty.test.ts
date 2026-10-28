// The provenance badge must not claim SAMPLE for a surface that reads the real ledger.
//
// The defect (2026-10-16): 'timeline' was in neither LIVE_TABS nor LEDGER_TABS, so
// provenanceForTab returned 'seed' or 'fixture' and the badge read "SAMPLE DATA — not yet
// from live capture" above ~600 real captured events that api.listEvents(400) reads and
// App.tsx re-polls every 8s. A badge that is false in the unsafe direction is the exact
// shape this file exists to prevent.

import { describe, expect, it } from 'vitest';
import {
  LEDGER_TABS,
  LIVE_TABS,
  LIVE_TAB_BACKING,
  PROVENANCE_META,
  provenanceForTab,
} from './provenance';

describe('the timeline badge tells the truth', () => {
  it("reads 'ledger', not 'seed'/'fixture', when the backend answered", () => {
    // backendHydrated = true is what useDataStore reports after a successful read.
    expect(provenanceForTab('timeline', true)).toBe('ledger');
    expect(provenanceForTab('timeline', true, 'ok')).toBe('ledger');
  });

  it("falls back to 'fixture' when the backend did NOT answer — never 'ledger'", () => {
    expect(provenanceForTab('timeline', false)).toBe('fixture');
  });

  it('claims LIVE with a headline that says ledger, not tmux mirror', () => {
    const meta = PROVENANCE_META.ledger;
    expect(meta.badge).toBe('LIVE');
    expect(meta.headline).toContain('canonical ledger');
    // The old false string must not survive anywhere in the live-path copy.
    expect(meta.headline).not.toContain('SAMPLE DATA');
    expect(meta.detail).toContain('8 seconds');
  });

  it('does not move the mirror tabs or the dispatch tab', () => {
    expect(provenanceForTab('live-seats', true, 'ok')).toBe('live');
    expect(provenanceForTab('sessions', true)).toBe('live');
    expect(provenanceForTab('dispatches', true)).toBe('live');
  });

  it('leaves a seed-backed tab exactly as it was', () => {
    // inbox / workitems / artifacts still read the seeded store, so they still say so.
    expect(provenanceForTab('inbox', true)).toBe('seed');
    expect(provenanceForTab('workitems', true)).toBe('seed');
    expect(PROVENANCE_META.seed.headline).toContain('SAMPLE DATA');
  });

  it('the live-mirror discovery states are untouched', () => {
    expect(provenanceForTab('live-seats', true, 'empty')).toBe('no-live');
    expect(provenanceForTab('live-seats', true, 'failed')).toBe('discovery-failed');
  });

  it('the two tab sets are disjoint — a tab cannot be both', () => {
    for (const t of LEDGER_TABS) expect(LIVE_TABS.has(t)).toBe(false);
  });
});

// ── The LIVE claim must depend on the read that backs it ──────────────────────
//
// The defect (2026-10-17): `if (LIVE_TABS.has(tab)) return 'live'` returned BEFORE
// `backendHydrated` was ever consulted, so a surface whose data came from a failed
// read still wore a green LIVE badge. Its correctness depended on nothing.
//
// The tests below FAIL the read — a happy-path test does not test this.

describe('the LIVE badge depends on the read backing the tab', () => {
  it('a database-backed tab does NOT claim LIVE when the read failed', () => {
    // This is the measured live state: api.listSeats() rejects (all 8 live seat rows
    // carry default_harness='Pi', outside the Harness domain), hydration's
    // Promise.all rejects, the store keeps its seeds — yet live-seats, sessions and
    // dispatches used to still claim LIVE.
    expect(provenanceForTab('sessions', false)).not.toBe('live');
    expect(provenanceForTab('dispatches', false)).not.toBe('live');
    // And it says WHY rather than staying silent about it.
    expect(provenanceForTab('sessions', false)).toBe('fixture');
    expect(provenanceForTab('dispatches', false)).toBe('fixture');
  });

  it('the same tabs DO claim LIVE once the read succeeded', () => {
    expect(provenanceForTab('sessions', true)).toBe('live');
    expect(provenanceForTab('dispatches', true)).toBe('live');
  });

  it('the live mirror is gated on tmux discovery, not on the database read', () => {
    // Measured, not assumed: cmd_list_tmux_sessions takes no Tauri `State` and
    // shells out to tmux. So a failed Postgres read must not downgrade it, and an
    // empty discovery must not let it claim LIVE either.
    expect(provenanceForTab('live-seats', false, 'ok')).toBe('live');
    expect(provenanceForTab('live-seats', true, 'empty')).toBe('no-live');
    expect(provenanceForTab('live-seats', true, 'failed')).toBe('discovery-failed');
  });

  it('every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name', () => {
    for (const tab of LIVE_TABS) {
      expect(LIVE_TAB_BACKING[tab], tab).toBeDefined();
    }
    // The tmux-backed one is the only one allowed to skip the database check.
    const dbBacked = Object.entries(LIVE_TAB_BACKING).filter(([, v]) => v === 'database');
    expect(dbBacked.map(([k]) => k).sort()).toEqual(['dispatches', 'sessions']);
  });

  it('a store-backed tab is unaffected: it still says seed/fixture either way', () => {
    expect(provenanceForTab('inbox', true)).toBe('seed');
    expect(provenanceForTab('inbox', false)).toBe('fixture');
  });

  it('the timeline keeps its ledger reading', () => {
    expect(provenanceForTab('timeline', true)).toBe('ledger');
    expect(provenanceForTab('timeline', false)).toBe('fixture');
  });
});
