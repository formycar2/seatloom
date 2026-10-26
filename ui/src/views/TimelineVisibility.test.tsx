// TimelineView: the event-type visibility control.
//
// The defect (AEGIS-2026-10-20-ui-memory-and-desync-v1 item 2): `templateFilter` was a
// useState that no filter ever read — a switch that looked live and did nothing. Worse
// than no switch, because it tells the operator the filter is already applied.
//
// Pinned here: the filter is real; the first-run default collapses memory/topic
// traffic; the operator's choice is remembered per project across a remount; and what
// is collapsed is named on screen with a one-click expand.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  projects: [
    {
      id: 'seatloom',
      name: 'SeatLoom',
      createdAt: '',
      workerBudgetTokens: 0,
      supervisorBudgetTokens: 0,
    },
  ],
  events: [] as unknown[],
  activeProjectId: 'seatloom' as string | null,
  stored: {} as Record<string, string>,
}));

vi.mock('../lib/api', () => ({
  isTauri: () => true,
  api: {
    listProjects: async () => state.projects,
    listSeats: async () => [],
    listRoleBindings: async () => [],
    listWorkitems: async () => [],
    listHandoffs: async () => [],
    listArtifacts: async () => [],
    listSessions: async () => [],
    listEvents: async () => state.events,
    listDocuments: async () => [],
  },
}));

import TimelineView from './TimelineView';
import { useDataStore } from '../stores/useDataStore';

const ev = (id: string, eventType: string) => ({
  event_id: id,
  event_type: eventType,
  occurred_at: '2026-10-20T00:00:00Z',
  actor_ref: 'seat:seat-mira-001',
  object_refs: [],
  evidence_refs: [],
  payload: {},
});

function setProject(id: string | null) {
  state.activeProjectId = id;
  useDataStore.setState({ activeProjectId: id, backendHydrated: true });
}

describe('TimelineView event-type visibility', () => {
  beforeEach(() => {
    cleanup();
    state.stored = {};
    state.events = [
      ev('e1', 'SeatDispatchIssued'),
      ev('e2', 'MemoryCreated'),
      ev('e3', 'MemoryRecalled'),
      ev('e4', 'TopicOpened'),
      ev('e5', 'SeatDispatchCompleted'),
    ];
    // A localStorage double that records writes, so "remembered" is observable.
    const store = state.stored;
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        getItem: (k: string) => (k in store ? store[k] : null),
        setItem: (k: string, v: string) => {
          store[k] = String(v);
        },
        removeItem: (k: string) => {
          delete store[k];
        },
        clear: () => {
          for (const k of Object.keys(store)) delete store[k];
        },
        key: () => null,
        length: 0,
      },
    });
    setProject('seatloom');
    useDataStore.setState({
      projects: state.projects as never,
      projectData: { seatloom: { events: state.events } as never },
    });
  });

  afterEach(() => cleanup());

  it('collapses memory and topic traffic by default, and NAMES what is hidden', async () => {
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());
    // The two work rows survive; the collapsed types appear ONLY as chips naming what
    // is hidden — never as rows.
    expect(screen.getByTitle('显示 MemoryCreated')).toBeTruthy();
    expect(screen.getByTitle('显示 MemoryRecalled')).toBeTruthy();
    expect(screen.getByTitle('显示 TopicOpened')).toBeTruthy();
    expect(screen.getAllByText('SeatDispatchIssued')).toHaveLength(1);
    expect(screen.getAllByText('SeatDispatchCompleted')).toHaveLength(1);
    expect(screen.queryAllByText('SeatDispatchIssued')).toHaveLength(1);
  });

  it('the toggle actually filters — the dead switch is gone', async () => {
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());

    // One click on a collapsed type's chip brings just that type back — the filter
    // is real, which the old templateFilter never was.
    fireEvent.click(screen.getByTitle('显示 MemoryRecalled'));
    await waitFor(() => expect(screen.getAllByText('MemoryRecalled')).toHaveLength(1));
    expect(screen.queryByTitle('显示 MemoryRecalled')).toBeNull();
    // The other two stay collapsed and stay named with counts.
    expect(screen.getByText(/已收起 2 类共 2 条/)).toBeTruthy();
    expect(screen.getByTitle('显示 MemoryCreated')).toBeTruthy();
    expect(screen.getByTitle('显示 TopicOpened')).toBeTruthy();

    // "全部展开" empties the hidden set entirely, and the strip disappears with it.
    fireEvent.click(screen.getByText('全部展开'));
    await waitFor(() => expect(screen.queryByText(/已收起/)).toBeNull());
    expect(screen.getAllByText('MemoryCreated')).toHaveLength(1);
    expect(screen.getAllByText('TopicOpened')).toHaveLength(1);
  });

  it("remembers the operator's choice across a remount", async () => {
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());
    fireEvent.click(screen.getByText('全部展开'));
    await waitFor(() => expect(screen.queryByTitle('显示 MemoryCreated')).toBeNull());

    // A remount is what a restart looks like to a component.
    cleanup();
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.queryByTitle('显示 MemoryCreated')).toBeNull());
    expect(screen.queryByText(/已收起/)).toBeNull();
  });

  it('keeps each project’s choice to itself', async () => {
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());

    // Another project, no record of its own: it gets the first-run default, not the
    // first project's "show all".
    setProject('ql_delivery');
    useDataStore.setState({
      projects: [
        ...state.projects,
        {
          id: 'ql_delivery',
          name: 'QL',
          createdAt: '',
          workerBudgetTokens: 0,
          supervisorBudgetTokens: 0,
        },
      ] as never,
      projectData: { ql_delivery: { events: state.events } as never },
    });
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());
  });

  it('a corrupt stored choice falls back to the default instead of throwing', async () => {
    state.stored['seatloom.timeline.visibility.seatloom'] = '{not json';
    render(
      <TimelineView
        onSelectObject={() => {}}
        onOpenArtifact={() => {}}
        filters={{}}
        onFiltersChange={() => {}}
      />,
    );
    await waitFor(() => expect(screen.getByText(/已收起 3 类共 3 条/)).toBeTruthy());
  });
});
