// Regression cover for the dispatch-inbox redesign.
//
// The old layout was a stack of cards; the new one is a sortable dense table
// with folded done-rows, in-place expansion and a live-refresh indicator. Those
// are the behaviours most likely to silently regress, so they are asserted here
// rather than left to a screenshot review:
//   1. one row per dispatch, grouped by seat, done folded away by default;
//   2. sortable columns, free-text search, in-place expansion;
//   3. the 5s refresh is pausable and its liveness is visible;
//   4. the preserved paths — the status filter, the new-dispatch form, and the
//      row → form "issue again" shortcut — still reach cmd_enqueue_dispatch.
//
// The api layer is mocked at the module boundary: everything below it (SQL,
// the scheduler) is out of scope for a view test.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  seats: [
    {
      id: 'seat-mira-001',
      name: 'Mira',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
      capabilityTags: [],
      status: 'active',
      createdAt: '2026-01-01T00:00:00Z',
    },
    {
      id: 'seat-nimbus-001',
      name: 'Nimbus',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
      capabilityTags: [],
      status: 'active',
      createdAt: '2026-01-01T00:00:00Z',
    },
  ],
  rows: [] as Record<string, unknown>[],
  enqueue: vi.fn(async () => 'disp-new'),
}));

vi.mock('../lib/api', () => ({
  api: {
    listSeats: async () => state.seats,
    listDispatches: async () => state.rows,
    enqueueDispatch: (...args: unknown[]) => state.enqueue(...(args as [])),
  },
}));

import DispatchInboxView from './DispatchInboxView';

const NOW = Date.now();
const iso = (msAgo: number) => new Date(NOW - msAgo).toISOString();

/** A dispatch with the shape cmd_list_dispatches returns. */
const dispatch = (over: Record<string, unknown>) => ({
  id: 'disp-x',
  targetSeatId: 'seat-mira-001',
  issuedBy: 'aegis',
  projectId: 'seatloom',
  title: '标题',
  payload: { note: '指令正文' },
  status: 'ready',
  priority: 5,
  issuedAt: iso(90_000),
  startedAt: null,
  completedAt: null,
  deps: [],
  ...over,
});

const TITLE_INPUT = '例如:为派单页加失败态徽章';
const INSTR_INPUT = '把席位需要的全部上下文写在这里(自足:别依赖它当前分支没有的文件)。';

describe('DispatchInboxView', () => {
  beforeEach(() => {
    cleanup();
    state.enqueue.mockClear();
    state.rows = [
      dispatch({ id: 'd1', title: 'Alpha', status: 'in_flight', startedAt: iso(120_000) }),
      dispatch({ id: 'd2', title: 'Beta', status: 'ready', priority: 1 }),
      dispatch({
        id: 'd3',
        title: 'Gamma',
        targetSeatId: 'seat-nimbus-001',
        status: 'blocked',
        priority: 9,
        deps: [{ dispatchId: 'd1', depKind: 'after', depValue: 'd1' }],
      }),
      dispatch({
        id: 'd4',
        title: 'Delta',
        targetSeatId: 'seat-nimbus-001',
        status: 'done',
        startedAt: iso(300_000),
        completedAt: iso(60_000),
      }),
    ];
  });

  it('renders one row per dispatch, grouped by seat, with done folded away', async () => {
    const { container } = render(<DispatchInboxView />);
    await screen.findByText('Alpha');

    // Three active rows only — the done one is behind the 已完成 fold.
    expect(container.querySelectorAll('tbody tr')).toHaveLength(3);
    expect(screen.queryByText('Delta')).toBeNull();

    // Each seat group is its own table, and the header is a triage line:
    // seat name plus per-status counts, so the operator never has to read rows
    // to learn whether a seat needs attention.
    expect(screen.getAllByText('Mira').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Nimbus').length).toBeGreaterThan(0);
    expect(screen.getByText('进行中 1')).toBeTruthy();
    expect(screen.getByText('阻塞 1')).toBeTruthy();
    expect(screen.getByText('已完成')).toBeTruthy();
  });

  it('sorts, searches, expands in place, folds done and pauses the live refresh', async () => {
    render(<DispatchInboxView />);
    await screen.findByText('Alpha');

    // Sortable columns: priority desc puts Gamma (P9) first.
    fireEvent.click(screen.getAllByLabelText('按优先级排序')[0]);
    expect(screen.getByText(/按优先级/).textContent).toContain('降序');

    // The 5s refresh stays visible and pausable.
    fireEvent.click(screen.getByRole('button', { name: '暂停自动刷新' }));
    expect(screen.getByText('自动刷新已暂停')).toBeTruthy();

    // Free-text search narrows the table without touching the status counts.
    fireEvent.change(screen.getByPlaceholderText(/搜索标题/), { target: { value: 'Gamma' } });
    expect(screen.getByText('Gamma')).toBeTruthy();
    expect(screen.queryByText('Alpha')).toBeNull();
    fireEvent.change(screen.getByPlaceholderText(/搜索标题/), { target: { value: '' } });

    // Expansion reveals the full payload and provenance inline.
    fireEvent.click(screen.getByRole('button', { name: /^Beta/ }));
    expect(screen.getByText('发出者')).toBeTruthy();
    expect(screen.getByText('aegis')).toBeTruthy();

    // Done rows open on demand.
    fireEvent.click(screen.getByRole('button', { name: /已完成/ }));
    expect(screen.getByText('Delta')).toBeTruthy();
  });

  it('re-issues an existing dispatch into the preserved new-dispatch form', async () => {
    render(<DispatchInboxView />);
    await screen.findByText('Alpha');

    // Default order is seat-then-lifecycle, so Alpha (Mira, in_flight) leads.
    fireEvent.click(screen.getAllByLabelText('以此为模板新建派单')[0]);
    expect((screen.getByPlaceholderText(TITLE_INPUT) as HTMLInputElement).value).toBe('Alpha');
    expect((screen.getByPlaceholderText(INSTR_INPUT) as HTMLTextAreaElement).value).toBe(
      '指令正文',
    );
  });

  it('still submits a brand-new dispatch through the preserved form', async () => {
    render(<DispatchInboxView />);
    await screen.findByText('Alpha');

    fireEvent.click(screen.getByRole('button', { name: '新建派单' }));
    fireEvent.change(screen.getByPlaceholderText(TITLE_INPUT), { target: { value: '新任务' } });
    fireEvent.change(screen.getByPlaceholderText(INSTR_INPUT), { target: { value: '指令' } });
    fireEvent.change(screen.getAllByRole('combobox')[0], {
      target: { value: 'seat-nimbus-001' },
    });
    fireEvent.click(screen.getByRole('button', { name: /派发/ }));

    await vi.waitFor(() =>
      expect(state.enqueue).toHaveBeenCalledWith('seat-nimbus-001', '新任务', '指令', 5),
    );
  });

  it('keeps the status filter as a full-inbox summary', async () => {
    render(<DispatchInboxView />);
    await screen.findByText('Alpha');

    fireEvent.click(screen.getByRole('button', { name: /^阻塞/ }));
    expect(screen.getByText('Gamma')).toBeTruthy();
    expect(screen.queryByText('Alpha')).toBeNull();
    // The chips still describe the whole inbox, not the current selection.
    expect(screen.getByRole('button', { name: /^全部/ }).textContent).toContain('4');
  });

  it('distinguishes "nothing ever dispatched" from "nothing matches"', async () => {
    state.rows = [];
    render(<DispatchInboxView />);
    expect(await screen.findByText('暂无派单')).toBeTruthy();
    expect(screen.getByText('点右上「新建派单」把任务派给某个席位')).toBeTruthy();

    cleanup();
    state.rows = [
      dispatch({
        id: 'only',
        title: 'Solo',
        status: 'done',
        startedAt: iso(60_000),
        completedAt: iso(0),
      }),
    ];
    render(<DispatchInboxView />);
    expect(await screen.findByText('无待处理的派单')).toBeTruthy();
    expect(screen.getByRole('button', { name: /显示已完成/ })).toBeTruthy();
  });
});
