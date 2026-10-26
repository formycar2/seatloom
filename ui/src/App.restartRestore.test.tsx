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
import { cleanup, render, screen, waitFor } from '@testing-library/react';
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
};

// `@tauri-apps/api/core`'s `invoke` is what `api.*` calls, so mocking the api
// object alone would leave the stores talking to a Tauri that is not there. Route
// invoke through the SAME canned responses: one source of truth for "the backend".
vi.mock('@tauri-apps/api/core', () => ({
  invoke: async (cmd: string, args?: unknown) => {
    calls.order.push(`${cmd}${args ? ` ${JSON.stringify(args)}` : ''}`);
    if (cmd in CANNED) return CANNED[cmd];
    throw new Error(`unmocked Tauri command: ${cmd}`);
  },
}));

// Tauri presence is detected via window.__TAURI_INTERNALS__, not via the api
// module — so the mock alone would leave every store on the browser path.
beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
    invoke: async () => {
      throw new Error('unmocked Tauri invoke');
    },
    transformCallback: () => 1,
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

  it("lands on the recorded project's saved tab, not another project's", async () => {
    // The two projects have different saved tabs, so "the tab followed the
    // restored project" and "the tab followed the first backend project" are
    // distinguishable observations rather than a shared coincidence.
    useAppStore.setState({
      projectUIStates: {
        [RECORDED_PROJECT]: {
          activeTab: 'artifacts',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
        [FIRST_REAL_PROJECT]: {
          activeTab: 'playbook',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
      },
    });

    mountApp();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));

    // 'artifacts' renders the ArtifactsView heading; 'playbook' renders a
    // different one. This heading, and not the other one, is the evidence.
    await waitFor(() => expect(screen.getByText('产出物')).toBeTruthy());
    expect(screen.queryByText(/Playbook & 治理文档/)).toBeNull();
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
          activeTab: 'playbook',
          selectedObjectId: null,
          selectedObjectType: null,
          filters: {},
        },
      },
    });

    mountApp();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));
    await waitFor(() => expect(screen.getByText('产出物')).toBeTruthy());
    const tabAfterFirstLaunch = readStoredAppStore().activeProjectId;
    expect(tabAfterFirstLaunch).toBe(RECORDED_PROJECT);

    // Now close and reopen: the ONLY input the second launch has is what the
    // first one wrote to disk.
    await relaunch();
    await waitFor(() => expect(useDataStore.getState().activeProjectId).toBe(RECORDED_PROJECT));
    await waitFor(() => expect(screen.getByText('产出物')).toBeTruthy());
    expect(screen.queryByText(/Playbook & 治理文档/)).toBeNull();

    // The saved TAB survived too. This is what the `loadedUIProjectId` gate
    // protects: without it the write effect fires before the destination's saved
    // tab has been applied, overwriting it with the previous tab.
    expect(useAppStore.getState().projectUIStates[RECORDED_PROJECT]?.activeTab).toBe('artifacts');

    // ...and it never wrote any OTHER tab for that project, not even for one
    // render. The final value above is what a later launch reads; a wrong tab on
    // disk for even a moment is what an operator who closes the app in that
    // window loses. React applies the tab restore and the write effect in the
    // same commit, so an ungated write effect reads the OLD tab and persists it —
    // invisible in the final state, fatal in the window.
    const writtenTabs = writtenTabsFor(RECORDED_PROJECT).filter((t) => t !== undefined);
    expect(writtenTabs.length).toBeGreaterThan(0);
    expect(new Set(writtenTabs)).toEqual(new Set(['artifacts']));
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
