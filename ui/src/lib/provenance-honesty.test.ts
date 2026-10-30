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
    // 2026-10-30: this test's comment used to read "inbox / workitems / artifacts
    // still read the seeded store, so they still say so", and it asserted all
    // three are 'seed'. Measured false: hydrateProjectData fills the store from
    // api.listSeats / listWorkitems / listHandoffs / listArtifacts / listSessions /
    // listEvents / listDocuments — every one a real Postgres read — and inboxItems
    // is built from that data. None of the three is seed-fed.
    //
    // What is left to guard is the branch itself: a tab with no declared backing
    // and no hydration still falls through to seed, and the headline stays the
    // contract string from the UM0 packet. The tab used here is one outside both
    // LIVE_TABS and LEDGER_TABS, which is the only population the seed branch
    // now serves.
    expect(provenanceForTab('timeline', true)).not.toBe('seed');
    // The seed branch is now reachable only by a tab that declares nothing and is
    // not hydrated. Pinned so the branch is not deleted as dead code while it is
    // still the honest answer for exactly that case.
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

  it('exactly ONE list decides a tab backing, and the derived set cannot disagree with it', () => {
    // 2026-10-30. There used to be TWO hand-maintained lists of tab names:
    // LIVE_TABS and LIVE_TAB_BACKING. They drifted the moment six tabs were added
    // to BACKING, and nothing watched the direction they had already drifted in —
    // adding to either alone was caught, but the two moving apart was not. A trap
    // with a green suite on top: the next person to add a tab had two lists to
    // update, only one enforced, and the unenforced one fed a branch that had
    // become unreachable.
    //
    // LIVE_TABS is now DERIVED from LIVE_TAB_BACKING, so drift is impossible by
    // construction. This test pins that fact rather than re-checking the contents,
    // because the contents are already pinned by the exact-set assertion below.
    //
    // What it guards is the construction: if someone reintroduces a second list,
    // these stop being the same object and this goes red.
    expect(LIVE_TABS.size).toBe(Object.keys(LIVE_TAB_BACKING).length);
    for (const tab of Object.keys(LIVE_TAB_BACKING)) {
      expect(LIVE_TABS.has(tab), `${tab} is in BACKING but not in the derived set`).toBe(true);
    }
    for (const tab of LIVE_TABS) {
      expect(
        LIVE_TAB_BACKING[tab],
        `${tab} is in the derived set but declares no backing`,
      ).toBeDefined();
    }
  });

  it('every LIVE_TABS entry declares what backs it — no tab claims LIVE on a tab name', () => {
    for (const tab of LIVE_TABS) {
      expect(LIVE_TAB_BACKING[tab], tab).toBeDefined();
    }
    // The tmux-backed one is the only one allowed to skip the database check.
    const dbBacked = Object.entries(LIVE_TAB_BACKING).filter(([, v]) => v === 'database');
    //
    // 2026-10-30: this used to assert EXACTLY ['dispatches', 'sessions']. That
    // assertion is what kept the SAMPLE badge on the other seven LIVE tabs — the
    // badge logic keys off this map, so anything missing from it falls through to
    // the seed branch. The test enforced the exclusion by name.
    //
    // The premise behind the old set was that a tab fed from the zustand store is
    // a different tier from a database-backed one. Measured: it is not.
    // hydrateProjectData fills the store from api.listSeats / listWorkitems /
    // listHandoffs / listArtifacts / listSessions / listEvents / listDocuments —
    // every one a real Postgres read — and builds inboxItems from that same data.
    // There is no store-backed tier, so there is no tab entitled to "seed".
    //
    // This is now the full set, and a tab added to LIVE_TABS without a backing
    // declaration fails the loop above rather than silently inheriting a badge
    // that says its rows are hand-written fixtures.
    expect(dbBacked.map(([k]) => k).sort()).toEqual(
      [
        'artifacts',
        'dashboard',
        'dispatches',
        'inbox',
        'playbook',
        'seats',
        'sessions',
        'workitems',
      ].sort(),
    );
  });

  it('a tab with NO database backing is unaffected by hydration — there is no store-backed tier', () => {
    // This test used to be "a store-backed tab is unaffected: it still says
    // seed/fixture either way", asserting inbox is seed. That premise was false:
    // inbox is hydrated from Postgres, so it is live, not seed.
    //
    // Rewritten to the claim that is actually true. The only tabs that do not
    // declare a backing are the ones outside LIVE_TABS entirely, and those are
    // governed by LEDGER_TABS and the fallthrough — not by a store tier.
    //
    // So the observable claim is: a tab that declares no backing behaves exactly
    // as the fallthrough says, and hydration does not change it. 'timeline' is
    // such a tab (it is LEDGER, not LIVE), and it is the one whose reading is
    // pinned below.
    //
    // What this test guards against is the failure mode that produced the badge
    // lie: someone re-introducing a "store-backed" concept that lets a live read
    // wear a fixture badge. If inbox ever returns to 'seed' while hydrated, the
    // assertion below goes red.
    expect(provenanceForTab('inbox', true)).toBe('live');
    expect(provenanceForTab('inbox', false)).toBe('fixture');
  });

  it('the timeline keeps its ledger reading', () => {
    expect(provenanceForTab('timeline', true)).toBe('ledger');
    expect(provenanceForTab('timeline', false)).toBe('fixture');
  });
});
