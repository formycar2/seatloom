// Data provenance — the one place that records which surfaces show live,
// captured truth and which still render seed fixtures.
//
// UM0 context. Every V1 view already reads out of PostgreSQL, so "it came from
// the database" is NOT the same as "it is live". The seed corpus
// (infra/postgres/seed/001_real_collaboration_baseline.sql) is a fixture set with
// no live backing, and those tables carry no `source` column to key off — so
// provenance cannot be derived per row. It is a property of the *view*, and it
// is declared here so there is exactly one list to keep honest.
//
// Three states, deliberately distinguished:
//   'live'    mirrored from a real tmux pane right now (tmux-mirror path)
//   'seed'    read from Postgres for real, but the rows are seed fixtures
//   'fixture' in-memory mock — the backend was not reachable at all
//
// 'seed' and 'fixture' are both "not real". We still separate them because the
// operator's next action differs: 'seed' means the capture pipeline has not
// filled this surface yet, 'fixture' means the app never reached the backend.

export type Provenance = 'live' | 'seed' | 'fixture';

/**
 * Tabs whose content is mirrored from a live tmux pane. Everything not listed
 * here is seed- or fixture-backed until its capture path lands.
 */
export const LIVE_TABS: ReadonlySet<string> = new Set(['live-seats', 'sessions']);

/**
 * Provenance of a tab. `backendHydrated` comes from useDataStore and tells us
 * whether the Postgres read actually succeeded.
 */
export function provenanceForTab(tab: string, backendHydrated: boolean): Provenance {
  if (LIVE_TABS.has(tab)) return 'live';
  return backendHydrated ? 'seed' : 'fixture';
}

export interface ProvenanceMeta {
  /** Short badge text. */
  badge: string;
  /** The headline claim, stated plainly. */
  headline: string;
  /** Why this surface reads the way it does, and what would change it. */
  detail: string;
  /** Foreground / background / border, from the shared token set. */
  fg: string;
  bg: string;
  border: string;
}

// NOTE: the 'SAMPLE DATA — not yet from live capture' wording is the contract
// string from the UM0 packet. Keep it verbatim in both non-live states.
const SAMPLE_HEADLINE = 'SAMPLE DATA — not yet from live capture';

export const PROVENANCE_META: Record<Provenance, ProvenanceMeta> = {
  live: {
    badge: 'LIVE',
    headline: 'LIVE — mirrored from tmux as it happens',
    detail:
      'Every pane below is a real tmux session on this machine, read through pipe-pane. ' +
      'Output moves as the seat works; keystrokes you type are forwarded into the real pane.',
    fg: 'var(--sl-green)',
    bg: 'var(--sl-green-subtle)',
    border: 'var(--sl-green)',
  },
  seed: {
    badge: 'SAMPLE',
    headline: SAMPLE_HEADLINE,
    detail:
      'These rows are read from PostgreSQL for real, but they are seed fixtures — ' +
      'a hand-written baseline, not captured activity. Nothing here reflects what the seats ' +
      'are doing now. The live mirror is under 实时席位.',
    fg: '#8A6100',
    bg: 'var(--sl-amber-subtle)',
    border: 'var(--sl-amber)',
  },
  fixture: {
    badge: 'MOCK',
    headline: SAMPLE_HEADLINE,
    detail:
      'The backend was not reachable, so this view is falling back to in-memory mock data ' +
      'compiled into the frontend. Not a database read, and not live capture.',
    fg: '#8A6100',
    bg: 'var(--sl-amber-subtle)',
    border: 'var(--sl-amber)',
  },
};
