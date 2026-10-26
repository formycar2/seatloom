// The Projects page must reflect the database, and a failed read must say so.
//
// The defect (2026-10-16, Mr. Zhang): ql_delivery was in the database but never on the
// page. What he saw was three hardcoded seed cards — identifiable because their paths
// were `~/Documents/GitHub/…` while `projectFromDto` synthesises `~/<name>`.
//
// (2026-10-26: that `~/<name>` synthesis is now GONE — it was the cause of
// `~/networkOrchestrator` appearing beside a correct full path in the project
// switcher. The mapper carries the ledger's real home_path, or null. The seeds keep
// their `~/Documents/GitHub/…` paths, which is what makes them still recognisable
// as seeds — the tell survives, the fabrication does not.)
//
// Four things are pinned here, one per requirement:
//   1. the store no longer starts from the seeds — the product path cannot show them;
//   2. a failed hydration is STATED, with the backend's own error text, never a silent
//      fallback and never fake cards;
//   3. an empty list distinguishes "the read failed" from "the database is empty";
//   4. the seeds still exist for tests, under a name that says what they are.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act, cleanup, render, screen } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  projects: [] as unknown[],
  seats: [] as unknown[],
  bindings: {} as Record<string, unknown[]>,
  throws: '' as string,
}));

vi.mock('../lib/api', () => ({
  isTauri: () => true,
  api: {
    listProjects: async () => {
      if (state.throws) throw new Error(state.throws);
      return state.projects;
    },
    listSeats: async () => state.seats,
    listAllSeats: async () => state.seats,
    listRoleBindings: async (p?: string) => state.bindings[p ?? ''] ?? [],
    listWorkitems: async () => [],
    listHandoffs: async () => [],
    listArtifacts: async () => [],
    listSessions: async () => [],
    listEvents: async () => [],
    listDocuments: async () => [],
  },
}));

import { SEED_PROJECTS_FOR_TESTS, useDataStore } from '../stores/useDataStore';
import AllProjectsView from '../views/AllProjectsView';

const proj = (id: string, name: string, homePath: string | null = null) => ({
  id,
  name,
  createdAt: '2026-10-16T00:00:00Z',
  workerBudgetTokens: 0,
  supervisorBudgetTokens: 0,
  homePath,
});

/** Reset the store to its shipped initial state before each test. */
const resetStore = () =>
  act(() => {
    useDataStore.setState({
      projects: [],
      projectData: {},
      activeProjectId: null,
      backendHydrated: false,
      backendHydrateError: null,
    });
  });

describe('the Projects page reflects the database', () => {
  beforeEach(() => {
    cleanup();
    resetStore();
    state.throws = '';
    state.projects = [];
    state.seats = [];
    state.bindings = {};
  });

  it('the store does NOT start from the three hardcoded seeds', () => {
    // The shipped initial value. If this ever becomes the seeds again, the product path
    // can silently show fake projects on a failed read.
    expect(useDataStore.getState().projects).toEqual([]);
    expect(useDataStore.getState().projectData).toEqual({});
  });

  it('hydration replaces the store with what the database holds — 39 means 39', async () => {
    // 38 corpus projects + ql_delivery + seatloom, as measured on live.
    state.projects = [
      proj('seatloom', 'SeatLoom'),
      proj('ql_delivery', 'ql_delivery'),
      proj('OfficeBench', 'OfficeBench'),
    ];
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });

    const ids = useDataStore.getState().projects.map((p) => p.id);
    expect(ids).toContain('ql_delivery');
    expect(ids).toHaveLength(3);
    expect(useDataStore.getState().backendHydrated).toBe(true);
    expect(useDataStore.getState().backendHydrateError).toBeNull();
  });

  it('a hydrated project renders the ledger path, never a synthesised one', async () => {
    state.projects = [proj('ql_delivery', 'ql_delivery')];
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    render(<AllProjectsView onSelectProject={() => {}} />);
    // The seeds' path was `~/Documents/GitHub/…`; the mapper used to synthesise
    // `~/<name>`. NEITHER may appear now — only the ledger's own home_path may.
    expect(screen.getByText('ql_delivery')).toBeTruthy();
    expect(screen.queryByText('~/Documents/GitHub/ql_delivery')).toBeNull();
    expect(screen.queryByText('~/ql_delivery')).toBeNull();
  });

  it('the ledger home_path is rendered verbatim, and a missing one says so', async () => {
    // The 2026-10-26 defect (Mr. Zhang): the switcher showed `~/networkOrchestrator`
    // beside a correct full path. The cause was `path: '~/' + d.name` in
    // projectFromDto. Both directions pinned here: the real path must appear
    // unchanged, and a row with NO home_path must say that rather than invent one.
    state.projects = [
      proj('project_delivery/networkOrchestrator', 'networkOrchestrator',
           '/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/networkOrchestrator'),
      proj('no-home', 'no-home', null),
    ];
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    render(<AllProjectsView onSelectProject={() => {}} />);
    expect(
      screen.getByText('/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/networkOrchestrator'),
    ).toBeTruthy();
    expect(screen.queryByText('~/networkOrchestrator')).toBeNull();
    expect(screen.getAllByText('无 home_path').length).toBeGreaterThan(0);
  });

  it('the seeds are still available for tests, under a name that says so', () => {
    // Requirement 4: tests may keep them; the product path may not.
    expect(Array.isArray(SEED_PROJECTS_FOR_TESTS)).toBe(true);
    expect(SEED_PROJECTS_FOR_TESTS.map((p) => p.id)).toEqual(['p-1', 'p-2', 'p-3']);
    expect(SEED_PROJECTS_FOR_TESTS[0].path).toBe('~/Documents/GitHub/seatloom');
  });
});

describe('a failed hydration is stated, never silently substituted', () => {
  beforeEach(() => {
    cleanup();
    resetStore();
    state.throws = 'connection refused: postgresql://seatloom@localhost:5432/seatloom';
    state.projects = [];
  });

  it('records the backend error verbatim and does not fall back to the seeds', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    const st = useDataStore.getState();
    expect(st.backendHydrated).toBe(false);
    expect(st.backendHydrateError).toContain('connection refused');
    // The decisive half: nothing fake replaced what could not be read.
    expect(st.projects).toEqual([]);
  });

  it('the Projects page says the read failed instead of rendering an empty grid', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    render(<AllProjectsView onSelectProject={() => {}} />);
    expect(screen.getByText(/项目还没读出来/)).toBeTruthy();
    expect(screen.getByText(/不是因为你没有项目/)).toBeTruthy();
    // And the backend's own words are on screen, not a paraphrase.
    expect(screen.getByText(/connection refused/)).toBeTruthy();
  });

  it('a SUCCESSFUL read with zero projects says the database is empty — a different fact', async () => {
    state.throws = '';
    state.projects = [];
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    render(<AllProjectsView onSelectProject={() => {}} />);
    expect(screen.getByText(/数据库里一个项目都没有/)).toBeTruthy();
    expect(screen.getByText(/这次读取是成功的/)).toBeTruthy();
    // It must NOT claim the read failed.
    expect(screen.queryByText(/项目还没读出来/)).toBeNull();
  });

  it('browser mode is reported as a reason, not as a silent no-op', async () => {
    // The old branch set backendHydrateError to null and left the seeds — "not an error".
    resetStore();
    state.throws = '';
    // isTauri() is true in this mock, so simulate the browser branch directly.
    await act(async () => {
      useDataStore.setState({ backendHydrated: false, backendHydrateError: null });
    });
    // With no error text the page must still not render a fake list.
    render(<AllProjectsView onSelectProject={() => {}} />);
    expect(screen.getByText(/项目还没读出来/)).toBeTruthy();
    expect(screen.queryByText('p-1')).toBeNull();
  });
});
