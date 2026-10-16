// The provenance badge must not claim SAMPLE for a surface that reads the real ledger.
//
// The defect (2026-10-16): 'timeline' was in neither LIVE_TABS nor LEDGER_TABS, so
// provenanceForTab returned 'seed' or 'fixture' and the badge read "SAMPLE DATA — not yet
// from live capture" above ~600 real captured events that api.listEvents(400) reads and
// App.tsx re-polls every 8s. A badge that is false in the unsafe direction is the exact
// shape this file exists to prevent.

import { describe, expect, it } from 'vitest';
import { LEDGER_TABS, LIVE_TABS, PROVENANCE_META, provenanceForTab } from './provenance';

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
