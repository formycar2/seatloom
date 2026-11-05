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
// Five states, deliberately distinguished:
//   'live'             mirrored from a real tmux pane right now (tmux-mirror path)
//   'seed'             read from Postgres for real, but the rows are seed fixtures
//   'fixture'          in-memory mock — the backend was not reachable at all
//   'no-live'          the live mirror ran discovery and it returned zero seats
//   'discovery-failed' the live mirror's discovery call itself threw
//
// 'seed' and 'fixture' are both "not real". We still separate them because the
// operator's next action differs: 'seed' means the capture pipeline has not
// filled this surface yet, 'fixture' means the app never reached the backend.
// The two discovery states exist so the live mirror cannot claim LIVE when
// there is provably nothing live behind it (UM0 HOLD-3).

export type Provenance =
  | 'live'
  /** The canonical ledger, read for real — not a tmux mirror, and not a fixture. */
  | 'ledger'
  /** A tab whose backing was never declared. NOT the same as seed — see below. */
  | 'not-declared'
  | 'seed'
  | 'fixture'
  | 'no-live'
  | 'discovery-failed';

/**
 * Live seat discovery status, published by LiveSeatsView while it polls
 * `cmd_list_tmux_sessions`:
 *   'unknown' the first discovery is still in flight (transient)
 *   'ok'      discovery returned at least one `-seatloom` seat
 *   'empty'   discovery returned zero seats
 *   'failed'  the IPC call itself threw
 */
export type SeatDiscovery = 'unknown' | 'ok' | 'empty' | 'failed';

/**
 * Tabs whose rows come from the canonical ledger, read for real.
 *
 * 2026-10-16: the timeline was NOT in this set and NOT in LIVE_TABS, so
 * `provenanceForTab('timeline', …)` returned 'seed' or 'fixture' and the badge read
 * "SAMPLE DATA — not yet from live capture". That is false in the unsafe direction:
 * `useDataStore.refreshEvents()` calls `api.listEvents(400)` and `App.tsx` polls it
 * every 8s, so those rows ARE captured events from the ledger — including the ones
 * other processes emit through MCP. Measured before changing anything: the read is
 * real, the cadence is real, and the only way the surface is not live is the backend
 * being unreachable, which `backendHydrated` already reports.
 */
export const LEDGER_TABS: ReadonlySet<string> = new Set(['timeline']);

/** The live-mirror tab: the one surface whose LIVE claim depends on tmux discovery. */
export const LIVE_MIRROR_TAB = 'live-seats';

/**
 * What backs each `LIVE_TABS` entry, measured rather than assumed.
 *
 * `live-seats` is the odd one out: `cmd_list_tmux_sessions` takes no Tauri `State`
 * and shells out to tmux, so it is NOT database-backed and its liveness is
 * `discovery` (handled by the `LIVE_MIRROR_TAB` branch above).
 *
 * `sessions` and `dispatches` both reach Postgres — `cmd_list_history_projects`
 * takes `State` and drives the history read tools, and `DispatchInboxView` polls
 * `cmd_list_dispatches` directly. So for those two a failed hydration means their
 * rows are not live either, and `backendHydrated` is the fact that says so.
 */
export const LIVE_TAB_BACKING: Readonly<Record<string, 'tmux' | 'database'>> = {
  'live-seats': 'tmux',
  // Every other LIVE tab is backed by Postgres.
  //
  // The set below used to be exactly [sessions, dispatches], and a test asserted
  // that exact set — which is how the SAMPLE badge stayed on seven surfaces that
  // read live PostgreSQL. The premise was that a tab fed from the zustand store is
  // somehow different from a database-backed one. It is not: hydrateProjectData
  // fills the store from api.listSeats / listWorkitems / listHandoffs /
  // listArtifacts / listSessions / listEvents / listDocuments, every one a real
  // Postgres read, and it builds inboxItems from that data too. There is no
  // store-backed tier. The distinction the badge logic was built on does not exist.
  //
  // Measured against the live database: 10 seats, 7 workitems, 15 artifacts,
  // 2946 events — all behind tabs that were telling the operator "SAMPLE DATA —
  // not yet from live capture".
  sessions: 'database',
  dispatches: 'database',
  dashboard: 'database',
  inbox: 'database',
  workitems: 'database',
  seats: 'database',
  artifacts: 'database',
  playbook: 'database',
};

/**
 * Tabs whose content is mirrored from a live tmux pane. Everything not listed
 * here is seed- or fixture-backed until its capture path lands.
 *
 * 2026-10-30: this used to be a hand-written `new Set([...])` — a SECOND list of
 * tab names, next to `LIVE_TAB_BACKING`. The two drifted immediately once six tabs
 * were added to BACKING, and nothing watched the direction they had already
 * drifted in. Worse, the branch that consulted this set
 * (`if (LIVE_TABS.has(tab)) return 'live'`) became unreachable the moment BACKING
 * covered the same tabs, and deleting it left 23 tests green.
 *
 * So it is now DERIVED from the one list that decides anything. There is exactly
 * one place to add a tab, and this set cannot disagree with it by construction.
 * A reader auditing provenance sees one gate list, not three.
 */
export const LIVE_TABS: ReadonlySet<string> = new Set(Object.keys(LIVE_TAB_BACKING));

/**
 * Provenance of a tab. `backendHydrated` comes from useDataStore and tells us
 * whether the Postgres read actually succeeded. `discovery` is the live seat
 * discovery status from useLiveSessionsStore.
 *
 * HOLD-3: the live mirror may only wear the LIVE badge once discovery has
 * actually returned seats. An empty or failed discovery renders an explicit
 * no-live / discovery-failed state instead — a green LIVE bar sitting above an
 * error message is exactly the false-live read UM0 exists to kill.
 *
 * 2026-10-17: every branch that can return `'live'` now requires the read backing
 * it to have succeeded. It used to be `if (LIVE_TABS.has(tab)) return 'live'`,
 * which returned BEFORE `backendHydrated` was ever consulted — so a surface whose
 * data came from a failed read still wore a green LIVE badge. The badge's
 * correctness depended on nothing, which is the defect. `live-seats` is exempt for
 * a measured reason, not by accident: its read is tmux (`cmd_list_tmux_sessions`
 * takes no `State`), and its liveness is already gated on `discovery` by the
 * `LIVE_MIRROR_TAB` branch below. The other two are database-backed, so a failed
 * hydration means they are not live either.
 */
export function provenanceForTab(
  tab: string,
  backendHydrated: boolean,
  discovery: SeatDiscovery = 'unknown',
): Provenance {
  if (tab === LIVE_MIRROR_TAB) {
    if (discovery === 'empty') return 'no-live';
    if (discovery === 'failed') return 'discovery-failed';
    return 'live';
  }
  if (LIVE_TAB_BACKING[tab] === 'database') {
    // The read backing this tab is Postgres, so a failed hydration means its rows
    // are not live either. This line used to be
    // `if (LIVE_TABS.has(tab)) return 'live'` — a tab-name list, consulted before
    // `backendHydrated`, so a failed read still wore a green LIVE badge.
    return backendHydrated ? 'live' : 'fixture';
  }
  // 2026-10-30: `if (LIVE_TABS.has(tab)) return 'live';` used to sit here.
  // It became unreachable the moment LIVE_TAB_BACKING covered the same tabs —
  // every member of LIVE_TABS is caught by the mirror branch above or the database
  // branch above that — and deleting it left 23 tests green. A gate that reads as
  // a live rule but can never run is worse than no gate: a reader auditing
  // provenance counts three gates and reasonably assumes the third is reachable.
  //
  // It is gone, and LIVE_TABS is now derived from LIVE_TAB_BACKING rather than
  // hand-maintained beside it, so there is nothing left to drift.
  if (LEDGER_TABS.has(tab)) return backendHydrated ? 'ledger' : 'fixture';
  // A tab that declares NO backing is not a tab whose rows are hand-written
  // fixtures. It is a tab nobody has said anything about, and the honest answer to
  // "where does this data come from?" is "nobody has declared it yet".
  //
  // This used to fall through to `seed`, which claims the opposite: that the rows
  // ARE a hand-written baseline. Measured: provenanceForTab('brandnew', true,
  // 'ready') returned 'seed' — a brand-new tab with a fully hydrated backend behind
  // it told the operator "these rows are a hand-written baseline, not captured
  // activity". That is the same false-lie direction the whole module exists to
  // kill, and it needs nobody to write a wrong assertion: it only needs someone to
  // add a tab and forget the second place.
  //
  // It is a DEFAULT, which is what makes it worse than a one-off mistake.
  return 'not-declared';
}

// Flux's M-1: deleting the seed branch left 24 tests green, and his reading was
// that dead code describing a correct state is better than no code — wait until
// the state actually arrives. He is right about `seed`, and that is why the seed
// meta below survives: when a genuinely seed-fed surface lands, the wording
// already exists and is already correct.
//
// What is gone is the FALL-THROUGH that let an undeclared tab borrow that
// wording. An unreachable branch and a reachable branch nobody reaches yet are
// not the same thing, and only the second one is honest.

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
  ledger: {
    badge: 'LIVE',
    headline: 'LIVE — read from the canonical ledger',
    detail:
      'These rows are real captured events, read from PostgreSQL through ' +
      'cmd_list_events and re-polled every 8 seconds — including events other processes ' +
      'append through MCP. Not a fixture, and not a tmux mirror. It stops being live only ' +
      'if the backend becomes unreachable, and then the badge says so instead of showing ' +
      'an empty list.',
    fg: 'var(--sl-green)',
    bg: 'var(--sl-green-subtle)',
    border: 'var(--sl-green)',
  },
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
  'not-declared': {
    badge: '未声明',
    headline: '这个视图的来源还没有人声明',
    detail:
      '没有任何名单说明这一页的数据从哪里来 —— 不是数据库,不是 tmux 镜像,也不是 ' +
      '手工基线。这是「没人说过」,不是「这是假数据」。如果你是新加了这一页, ' +
      '请在 LIVE_TAB_BACKING 里声明它读什么;在声明之前,请不要把这里显示的任何 ' +
      '行当成实时数据,也不要把它们当成示例数据 —— 它们的来源是未知的。',
    fg: 'var(--sl-amber-subtle)',
    bg: 'var(--sl-amber-subtle)',
    border: 'var(--sl-amber)',
  },
  seed: {
    badge: 'SAMPLE',
    headline: SAMPLE_HEADLINE,
    detail:
      'This view is not wired to a live read yet: it has no declared database backing, ' +
      'so what you see is a hand-written fixture rather than captured activity. ' +
      'It is not that the database was unreachable — a failed read says MOCK instead — ' +
      'it is that nothing here has been connected. Treat every row as a placeholder.',
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
  'no-live': {
    badge: 'NO LIVE SEATS',
    headline: 'No live seats found — discovery returned empty',
    detail:
      'The mirror asked the backend for tmux sessions ending in -seatloom and none came back. ' +
      'Nothing on this surface is live. Start the worker seats, then press 重新发现.',
    fg: '#8A6100',
    bg: 'var(--sl-amber-subtle)',
    border: 'var(--sl-amber)',
  },
  'discovery-failed': {
    badge: 'DISCOVERY FAILED',
    headline: 'Live seat discovery failed',
    detail:
      'cmd_list_tmux_sessions did not answer, so this surface cannot claim to be live. ' +
      'Check the Tauri backend, then press 重新发现 to retry.',
    fg: 'var(--sl-red)',
    bg: 'var(--sl-red-subtle)',
    border: 'var(--sl-red)',
  },
};
