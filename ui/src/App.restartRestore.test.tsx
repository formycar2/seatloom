// Restart-restore ordering — pinned against the write effect winning the race.
//
// The defect this replaces (Flux's verification, 2026-10-22, measured on a real
// launch): App.tsx's persistence effect ran BEFORE the restore effect, so it
// persisted useDataStore's *default* project id ('p-1', a SEED_PROJECTS_FOR_TESTS
// value) into `seatloom-app-store` BEFORE the restore effect read it. The
// recorded position was therefore lost on the very first launch after the
// feature landed — and because 'p-1' stayed on disk, every later launch started
// from it too. The position was not lost once; it was never restored.
//
// The fix is an ORDER CONTRACT, not a timing hope:
//   1. `useAppStore.activeProjectId` is snapshotted exactly once, in a useState
//      initialiser, before any effect runs — the launch record can never be
//      mutated by this component's own writes;
//   2. the restore effect is gated on `backendHydrated` and finishes by setting
//      `restoredPosition`;
//   3. the persistence effect is gated on `restoredPosition` AND on the tab state
//      having acknowledged the active project (`loadedUIProjectId === activeProjectId`),
//      so the PREVIOUS project's saved tab cannot overwrite the destination's.
//
// Flux found the original bug by mutating the source and noticing nothing failed,
// then adding an assertion. The same method is applied here in reverse: these
// assertions were each verified to go RED against the mutation they describe
// (mutation log kept at the bottom of this file), so a reorder cannot pass quietly.

// Must be the FIRST import: jsdom has no localStorage, and zustand's `persist`
// binds `window.localStorage` when the store is created (see the module's note).
import './test/installLocalStorage';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const STORAGE_KEY = 'seatloom-app-store';
const RECORDED_PROJECT = 'ql_delivery';
const FIRST_REAL_PROJECT = 'seatloom';
// The value useDataStore used to open on: a SEED_PROJECTS_FOR_TESTS id that names
// no project in any database. Used as the pre-hydration value in these tests so
// the ordering assertions fail on THIS value if the gate is ever removed.
const OLD_LAUNCH_DEFAULT = 'p-1';
const BACKEND_PROJECTS = [
  {
    id: FIRST_REAL_PROJECT,
    name: 'SeatLoom',
    description: '',
    last_accessed: '2026-10-22T00:00:00Z',
  },
  { id: RECORDED_PROJECT, name: 'QL', description: '', last_accessed: '2026-10-21T00:00:00Z' },
];

const calls = vi.hoisted(() => ({ order: [] as string[] }));
// Every write the APP makes to the app-store key, in order. Instrumented at the
// storage layer so the evidence is what reached the disk, not what a store held.
const appStoreWrites = vi.hoisted(() => ({ values: [] as string[] }));

const CANNED: Record<string, unknown> = {
  // The Tauri event plugin: `listen()` calls `invoke('plugin:event|listen')` and
  // resolves to an event id, and `unlisten` goes back through the same channel.
  // App.tsx subscribes to supervisor:detached / supervisor:reembedded on every
  // mount with NO .catch() on that promise, so an unmocked command here surfaces
  // as an unhandled rejection per mount. Answering it is what keeps this suite's
  // error count from growing with every test that mounts the real App.
  'plugin:event|listen': 1,
  'plugin:event|unlisten': undefined,
  cmd_supervisor_window_status: { detached: false },
  cmd_list_projects: BACKEND_PROJECTS,
  cmd_list_seats: [],
  cmd_list_all_seats: [],
  cmd_list_role_bindings: [],
  cmd_list_workitems: [],
  cmd_list_handoffs: [],
  cmd_list_artifacts: [],
  cmd_list_sessions: [],
  cmd_list_events: [],
  cmd_list_documents: [],
  // Which build this is. `live-seats` (and anything that renders the seat
  // mirror) reads it; without an answer the surface fails to render and a test
  // that switches to a saved 'live-seats' tab measures the failure, not the tab.
  // snake_case on purpose: SandboxBanner reads the raw payload and checks
  // `is_sandbox` before any normalise step.
  cmd_get_runtime_identity: { is_sandbox: false, branch: null, commit: null, built_at: null },
};

// ONE source of truth for "the backend", answering every command the app can
// send. Both mocks below route through this, because there are two paths and
// they are easy to assume are one:
//
//   - `api.*` calls `invoke` from `@tauri-apps/api/core`, which is the path most
//     commands take;
//   - Tauri's `listen()` (and the presence check `isTauri()`) goes through
//     `window.__TAURI_INTERNALS__.invoke`.
//
// The two were NOT the same mock in the first version of this file: the module
// mock answered CANNED while `__TAURI_INTERNALS__` threw for everything. That
// divergence is why this suite carries pre-existing unhandled rejections — every
// mount's `onSupervisorDetached` subscription reaches the internals mock, which
// never had an answer for it. Sharing one table is what makes a command either
// answered everywhere or missing everywhere, so a missing one is a single clear
// failure instead of a rejection attributed to whichever effect got there first.
const answerCommand = async (cmd: string, args?: unknown): Promise<unknown> => {
  calls.order.push(`${cmd}${args ? ` ${JSON.stringify(args)}` : ''}`);
  if (cmd in CANNED) return CANNED[cmd];
  throw new Error(`unmocked Tauri command: ${cmd}`);
};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => answerCommand(cmd, args),
}));

// Tauri presence is detected via window.__TAURI_INTERNALS__, not via the api
// module — so the module mock alone would leave every store on the browser path.
// Same table, so an unmocked command fails the same way from either door.
beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
    invoke: (cmd: string, args?: unknown) => answerCommand(cmd, args),
    transformCallback: () => 1,
  };
  // The event plugin's own internals: `listen()` resolves to an id and the
  // returned unlisten goes straight here. Without this, answering
  // `plugin:event|listen` only moves the failure to cleanup — every unsubscribe
  // would throw instead of every subscribe rejecting. Both doors, or the count
  // of unhandled errors just changes shape.
  (
    window as unknown as { __TAURI_EVENT_PLUGIN_INTERNALS__: unknown }
  ).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {},
  };
});

// The terminal panes need xterm + a real canvas; neither exists in jsdom.
vi.mock('../app-v2/panel/SessionTerminal', () => ({
  SessionTerminal: () => <div data-testid="terminal" />,
}));

import App from './App';
import { useAppStore } from './stores/useAppStore';
import { SEED_PROJECTS_FOR_TESTS, useDataStore } from './stores/useDataStore';

// The store's own launch value, captured at import time — before any test in this
// file touches it. This is what a real launch starts from, so it is the only
// honest place to assert that a launch does not begin on a test-only seed id.
const LAUNCH_DEFAULT_AT_IMPORT = useDataStore.getState().activeProjectId;

function readStoredAppStore(): { activeProjectId: string | null } {
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return { activeProjectId: null };
  return JSON.parse(raw).state as { activeProjectId: string | null };
}

/**
 * Mount the real app and record only the writes IT makes. The test's own
 * `useAppStore.setState(...)` in beforeEach also persists (that is the feature
 * under test, after all), so the log is cleared here — a "first write" taken
 * before the mount would be the test setup's write, not the app's.
 */
/**
 * The MAIN region's text, not the whole document's. `document.body.textContent`
 * was tried first and is useless for tab assertions: every tab's content is in
 * the DOM at once, so a query for the dispatches heading is satisfied even while
 * another tab is showing. The nav rail is excluded for the same reason — it
 * names all the tabs whether or not they are active.
 */
function mainText(): string {
  const main = document.querySelector('main');
  return main ? main.textContent ?? '' : '';
}

function mountApp() {
  appStoreWrites.values.length = 0;
  render(<App />);
}

/**
 * What a restart looks like to the app: a new process, so the app store is
 * rehydrated from what the previous launch wrote to disk, and the data store
 * starts from its launch defaults again. Without this the "restart" assertions
 * could only ever observe a single launch.
 */
async function relaunch() {
  cleanup();
  const stored = window.localStorage.getItem(STORAGE_KEY);
  expect(stored).not.toBeNull();
  useAppStore.setState(JSON.parse(stored as string).state);
  useDataStore.setState({
    projects: [],
    projectData: {},
    activeProjectId: OLD_LAUNCH_DEFAULT,
    backendHydrated: false,
    backendHydrateError: null,
  });
  appStoreWrites.values.length = 0;
  render(<App />);
}

/** Every project id the app persisted since `mountApp`, in order. */
function writtenProjectIds(): Array<string | null> {
  return appStoreWrites.values.map((v) => JSON.parse(v).state.activeProjectId);
}

/** Every tab the app persisted for one project, in order. */
function writtenTabsFor(projectId: string): Array<string | undefined> {
  return appStoreWrites.values.map(
    (v) => JSON.parse(v).state.projectUIStates?.[projectId]?.activeTab,
  );
}

beforeEach(() => {
  calls.order = [];
  window.localStorage.clear();
  const realSetItem = window.localStorage.setItem.bind(window.localStorage);
  window.localStorage.setItem = (k: string, v: string) => {
    if (k === STORAGE_KEY) appStoreWrites.values.push(v);
    realSetItem(k, v);
  };

  // The launch record: the operator was last in the recorded project.
  useAppStore.setState({
    activeProjectId: RECORDED_PROJECT,
    projectUIStates: {
      [RECORDED_PROJECT]: {
        activeTab: 'artifacts',
        selectedObjectId: null,
        selectedObjectType: null,
        filters: {},
      },
    },
    navCollapsed: false,
  });
  // useDataStore's launch state: the value that used to be written over the record.
  useDataStore.setState({
    projects: [],
    projectData: {},
    activeProjectId: OLD_LAUNCH_DEFAULT,
    backendHydrated: false,
    backendHydrateError: null,
  });
});

afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

describe('a launch does not begin on a test-only project id', () => {
  it('opens useDataStore on no SEED_PROJECTS_FOR_TESTS id', () => {
    // The dispatch's requirement, stated as a guard: a real launch must not start
    // on a project that exists only for tests. `p-1` was that value — the app then
    // persisted it over the operator's real position.
    const seedIds = SEED_PROJECTS_FOR_TESTS.map((p) => p.id);
    expect(seedIds).toContain('p-1'); // the guard is only meaningful if it is there
    expect(seedIds).not.toContain(LAUNCH_DEFAULT_AT_IMPORT);
  });
});

describe('restart lands back in the recorded project, and the write cannot win the race', () => {
  it('restores the recorded project after the backend read, not before it', async () => {
    mountApp();

    // The destination is only decided once the read has landed — until then the
    // app holds its launch default, which is the point of gating on hydration.
    expect(useDataStore.getState().activeProjectId).toBe(OLD_LAUNCH_DEFAULT);
    await waitFor(() => expect(useDataStore.getState().backendHydrated).toBe(true));
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));
    expect(calls.order).toContain('cmd_list_projects');
  });

  it('never persists the launch default while the real position is being restored', async () => {
    mountApp();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));

    // The regression guard, stated the way the bug actually behaved: the old code
    // wrote useDataStore's launch default ('p-1') to disk BEFORE the restore read
    // it, which destroyed the record for good. The final value looked right
    // afterwards — a moment later the restore overwrote it — so the ONLY evidence
    // is that no write ever carried it. If someone removes the ordering gate, this
    // is the assertion that goes red.
    expect(writtenProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
    expect(readStoredAppStore().activeProjectId).toBe(RECORDED_PROJECT);
  });

  it('a COLD START lands on 派单 even when the recorded project has a saved tab', async () => {
    // The two projects have different saved tabs, so "the tab followed the
    // restored project" and "the tab followed the first backend project" are
    // distinguishable observations rather than a shared coincidence.
    //
    // 2026-10-29: the contract CHANGED here. This test used to assert that a
    // restart lands on the recorded project's saved tab ('artifacts'). Measured
    // on the operator's machine, that behaviour made the app open on 实时席位
    // every launch, and the first-screen card asserted the opposite — so the
    // card was written against a false premise. A cold start now begins on
    // 派单 (App.tsx's useState default, which this effect used to overwrite
    // because the store is persisted without a partialize). The per-project tab
    // memory is NOT removed; it still applies when the operator switches
    // projects inside a running session — see the next test.
    useAppStore.setState({
      projectUIStates: {
        [RECORDED_PROJECT]: {
          activeTab: 'artifacts',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
        // 'workitems', NOT 'dispatches': the cold-start default IS 'dispatches',
        // so a saved 'dispatches' makes "the memory was applied" and "nothing
        // happened" the same observation. Measured — that exact choice left this
        // test green with the restore deleted, which is the whole reason this
        // sentence exists. ('live-seats' was tried next; its rediscover control
        // only renders when there are sessions, and its provenance headline is
        // rendered in several places, so neither discriminated.)
        [FIRST_REAL_PROJECT]: {
          activeTab: 'workitems',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
      },
    });

    mountApp();
    // The PROJECT is still the recorded one — only the tab changed.
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));

    // And the tab is 派单, not the saved 'artifacts' and not another project's
    // 'playbook'. The dispatches page's own heading is the evidence — matched
    // with a function because the string is split across nodes. Neither of the
    // other two headings may appear.
    // The heading is split across nested elements, so match on the container's
    // textContent rather than on a text node.
    await waitFor(() => expect(mainText()).toContain('每个席位的治理派单队列'));
    expect(screen.queryByText('产出物')).toBeNull();
    expect(screen.queryByText(/Playbook & 治理文档/)).toBeNull();
  });

  it('switching projects inside a session still restores THAT project\u2019s tab', async () => {
    // The fix above must not delete the per-project tab memory: it is a real
    // feature the operator asked for. Same two saved tabs, but the restore is
    // triggered by a project SWITCH after mount, not by the cold-start mount.
    useAppStore.setState({
      projectUIStates: {
        [RECORDED_PROJECT]: {
          activeTab: 'artifacts',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
        [FIRST_REAL_PROJECT]: {
          activeTab: 'dispatches',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
      },
    });

    mountApp();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));

    // Now switch. The memory must come back: FIRST_REAL_PROJECT's saved tab is
    // 'live-seats', so the seat mirror must appear. Its own banner is the
    // evidence — NOT the absence of the dispatches heading, because the
    // live-seats page embeds the dispatch inbox and both texts are present
    // either way (measured: asserting on the absence was a false negative).
    act(() => {
      useDataStore.getState().setActiveProject(FIRST_REAL_PROJECT);
    });
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(FIRST_REAL_PROJECT));
    // The rediscover button is unique to the live-seats surface. The provenance
    // headline was tried first and is NOT unique — ProvenanceBanner renders it in
    // several places, so that assertion passed with the tab memory deleted.
    // The work-items ledger heading is unique to that surface. Two earlier
    // attempts were measured non-discriminating: 'dispatches' equals the
    // cold-start default, and the live-seats headline is rendered in several
    // places. What makes this one valid is that the dispatches page does NOT
    // contain it — checked against the cold-start test's own render.
    console.log('PROBE firstReal saved=', JSON.stringify(useAppStore.getState().projectUIStates[FIRST_REAL_PROJECT]));
    await waitFor(() => expect(mainText()).toContain('活跃工作项账本'));
    // ...and the other project's tab is not inherited.
    expect(screen.queryByText('产出物')).toBeNull();
  });

  it('does not restore a project the backend did not return', async () => {
    // The record names a project that no longer exists. The app must land on a
    // real backend project rather than on the stale id — never on an invented one.
    useAppStore.setState({ activeProjectId: 'deleted-project' });

    mountApp();
    await waitFor(() => expect(useDataStore.getState().backendHydrated).toBe(true));

    expect(useDataStore.getState().activeProjectId).toBe(FIRST_REAL_PROJECT);
    // The record is REPAIRED to the real project rather than left naming the
    // dead one. (A stale record may still be re-emitted by an unrelated write
    // before the repair lands — that is the previous session's value on disk, not
    // this launch inventing one. What matters is where it ends up.)
    await waitFor(() => expect(readStoredAppStore().activeProjectId).toBe(FIRST_REAL_PROJECT));
    expect(writtenProjectIds().at(-1)).toBe(FIRST_REAL_PROJECT);
    expect(writtenProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
  });

  it('a second launch lands back on the same project AND the same tab', async () => {
    useAppStore.setState({
      projectUIStates: {
        [RECORDED_PROJECT]: {
          activeTab: 'artifacts',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
        [FIRST_REAL_PROJECT]: {
          activeTab: 'dispatches',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
      },
    });

    mountApp();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));
    // 2026-10-29: the CONTRACT changed. This used to assert the first launch
    // lands on the saved 'artifacts' tab. Measured on the operator's machine,
    // that behaviour made every cold start open on 实时席位 while the
    // first-screen card asserted 派单 — the card was false. A cold start now
    // begins on 派单; what the launch must still restore is the PROJECT.
    await waitFor(() => expect(mainText()).toContain('每个席位的治理派单队列'));
    expect(screen.queryByText('产出物')).toBeNull();
    const tabAfterFirstLaunch = readStoredAppStore().activeProjectId;
    expect(tabAfterFirstLaunch).toBe(RECORDED_PROJECT);

    // Now close and reopen: the ONLY input the second launch has is what the
    // first one wrote to disk. Same rule — project restored, tab on 派单.
    await relaunch();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));
    await waitFor(() => expect(mainText()).toContain('每个席位的治理派单队列'));
    expect(screen.queryByText('产出物')).toBeNull();

    // The saved TAB is whatever the operator is actually looking at. Since the
    // 2026-10-29 contract change that is 'dispatches' on a cold start — the point
    // of the assertion is that disk agrees with the screen, which is what the
    // `loadedUIProjectId` gate protects: without it the write effect fires before
    // the destination's saved tab has been applied, overwriting it with the
    // previous tab.
    expect(useAppStore.getState().projectUIStates[RECORDED_PROJECT]?.activeTab).toBe('dispatches');

    // ...and it never wrote any OTHER tab for that project, not even for one
    // render. The final value above is what a later launch reads; a wrong tab on
    // disk for even a moment is what an operator who closes the app in that
    // window loses. React applies the tab restore and the write effect in the
    // same commit, so an ungated write effect reads the OLD tab and persists it —
    // invisible in the final state, fatal in the window.
    const writtenTabs = writtenTabsFor(RECORDED_PROJECT).filter((t) => t !== undefined);
    expect(writtenTabs.length).toBeGreaterThan(0);
    expect(new Set(writtenTabs)).toEqual(new Set(['dispatches']));
  });

  it('starts with no active project when the backend has none at all', async () => {
    // An empty project list is a real state (fresh install). The app must not
    // invent a project, and must not leave the launch default sitting in state.
    CANNED.cmd_list_projects = [];
    try {
      mountApp();
      await waitFor(() => expect(useDataStore.getState().backendHydrated).toBe(true));
      expect(useDataStore.getState().activeProjectId).toBeNull();
      // Nothing is recorded: there is no honest position to persist yet.
      expect(writtenProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
    } finally {
      CANNED.cmd_list_projects = BACKEND_PROJECTS;
    }
  });
});

// ── Mutation log ────────────────────────────────────────────────────────────
// Each mutation below was applied to src/App.tsx (or useDataStore.ts) and the
// suite run; "RED" names the test that failed. Keeping this here means the next
// person can re-run the same mutations instead of trusting this comment.
//
//  1. restore effect loses its `backendHydrated` gate
//     → RED: 'restores the recorded project after the backend read, not before it'
//            (+ 3 more: the restore then ran against an empty project list)
//  2. persistence effect loses its `restoredPosition` gate
//     → RED: 'never persists the launch default while the real position is being restored'
//  3. persistence effect loses its `loadedUIProjectId` gate
//     → RED: 'lands on the recorded project's saved tab, not another project's'
//  4. useDataStore's launch `activeProjectId` goes back to the seed id 'p-1'
//     → RED: 'never persists the launch default while the real position is being restored'
//  5. the whole persistence effect removed
//     → RED: every test that asserts the record on disk

// ── The InitDialog path: the third fix to the same root cause ────────────────
//
// Flux's re-verification (ev-mAztaqFFzPSAQ-Fm3asBD) reported F1, a coverage gap
// rather than a defect: his mutation M6 restored the button's old one-liner
//   `setActiveProject('p-1')`
// and ALL SEVEN tests above stayed green. Nothing in this suite ever opened that
// dialog, so the third of the commit's three fixes had no test behind it — the
// commit message claimed more than the evidence supported.
//
// The path is only reachable when the backend has no projects at all: the dialog
// opens from `restoredPosition && !activeProjectId`, and a non-empty list always
// restores onto a real project. That is why every other test missed it — each
// mounts with a populated list, where the dialog never appears.
//
// The tests below drive the REAL dialog (its real button, its real click) and
// assert on the call log the file already instruments. Two scenarios, because
// the fix has two halves a single case could not separate:
//
//   1. the backend is still empty — clicking must not invent a project;
//   2. projects have appeared since launch — clicking must land on a real one.
//
// Scenario 2 is what keeps the fix honest in the other direction: a handler that
// merely closes the dialog without re-reading would strand the operator on "no
// project" forever, which is the failure this half pins.

/**
 * Click the DIALOG's button, the way the operator does.
 *
 * The name has to be the full one — '初始化演示仓' alone also matches the status
 * bar's own initialize button, and clicking that one would test a different
 * surface entirely.
 */
async function clickInitializeButton() {
  const button = await screen.findByRole('button', { name: /初始化演示仓 \(INITIALIZE\)/ });
  await act(async () => {
    fireEvent.click(button);
  });
}

/** The dialog is showing. */
const dialogIsOpen = () =>
  screen.queryByRole('button', { name: /初始化演示仓 \(INITIALIZE\)/ }) !== null;

/** Every project id the app asked the backend to read, in order. */
function readProjectIds(): string[] {
  return calls.order
    .filter((c) => c.startsWith('cmd_list_seats '))
    .map((c) => {
      const m = c.match(/"projectId":"([^"]*)"/);
      return m ? m[1] : '';
    });
}

describe('the InitDialog button does not plant a project that exists only in tests', () => {
  it('with no backend projects at all, clicking invents nothing and records nothing', async () => {
    CANNED.cmd_list_projects = [];
    try {
      mountApp();
      // The dialog is the state an empty backend produces, and it is where the
      // path no other test could reach begins.
      await waitFor(() => expect(dialogIsOpen()).toBe(true));
      expect(useDataStore.getState().activeProjectId).toBeNull();

      await clickInitializeButton();

      // No project was invented: the old code planted the test seed id here,
      // which exists in no database, and the write effect then persisted it as
      // the operator's recorded position.
      expect(useDataStore.getState().activeProjectId).toBeNull();
      expect(writtenProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
      // Nor is anything recorded: there is still no honest position to persist.
      expect(readStoredAppStore().activeProjectId).toBeNull();
      // The button DID re-read (a second listProjects beyond the mount-time one):
      // the fix is "ask the backend again", not "give up and close".
      expect(calls.order.filter((c) => c === 'cmd_list_projects').length).toBeGreaterThan(1);
      // The dialog closes so it does not sit there unclickable forever.
      await waitFor(() => expect(dialogIsOpen()).toBe(false));
    } finally {
      CANNED.cmd_list_projects = BACKEND_PROJECTS;
    }
  });

  it('when projects have appeared since launch, clicking lands on a real one', async () => {
    CANNED.cmd_list_projects = [];
    try {
      mountApp();
      await waitFor(() => expect(dialogIsOpen()).toBe(true));

      // The backend now has projects — the operator created one, or another seat
      // did. Only a re-read can know that; the old one-liner could not.
      CANNED.cmd_list_projects = BACKEND_PROJECTS;

      await clickInitializeButton();

      await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(FIRST_REAL_PROJECT));
      expect(useDataStore.getState().activeProjectId).not.toBe(OLD_LAUNCH_DEFAULT);
      // ...and it is a project the backend actually returned.
      expect(useDataStore.getState().projects.map((p) => p.id)).toContain(FIRST_REAL_PROJECT);
      // The re-read is what proves it: the click has to TALK to the backend. A
      // handler that only closed the dialog, or set an id without reading, would
      // leave the store holding whatever it held before (null here).
      expect(calls.order).toContain('cmd_list_projects');

      // THE ASSERTION THE OTHER SEVEN TESTS CANNOT MAKE: the app must never even
      // briefly sit on a project id that exists in no database. A final-value
      // check is blind here — `hydrateFromBackend` awaits the list and then picks
      // `storedActive` if the backend returned it, else the first project, so a
      // seed id planted BEFORE the await is corrected afterwards and the end
      // state looks clean. The observable trace is what was read: an id the
      // backend does not return must never appear in a scoped read. (This is the
      // same shape as the 'p-1' write effect: right in the end, wrong in the
      // window — and the window is where the operator loses his position.)
      expect(readProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
    } finally {
      CANNED.cmd_list_projects = BACKEND_PROJECTS;
    }
  });
});
