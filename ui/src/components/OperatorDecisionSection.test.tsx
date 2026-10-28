// The 020 「等你裁定」 section, against the object that answers "how many things
// are waiting on the human?".
//
// The api layer is mocked at the module boundary, exactly as the view consumes
// it. What is under test is this UI's own behaviour: the empty state, the card
// (target + options + consequence), the ruling, the verbatim-error rule, and —
// most of all — that an operator decision is NOT rendered as an acceptance.
//
// WHY THAT LAST ONE IS A TEST. Before 020 a product-direction ruling existed
// only as a SupervisorMessage, and the 待验收 section read acceptances alone.
// The tempting shortcut here is to render obligations in the same list as
// acceptance cards, because they look similar and it saves a section. That
// shortcut makes the operator's count wrong, which is the exact defect the
// object exists to fix — so the separation is asserted, not assumed.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';
import { useDataStore } from '../stores/useDataStore';

const state = vi.hoisted(() => ({
  rows: [] as unknown[],
  resolve: vi.fn(async () => ({})),
}));

vi.mock('../lib/api', () => ({
  api: {
    listOperatorDecisions: async () => state.rows,
    resolveOperatorDecision: (...a: unknown[]) => state.resolve(...(a as [])),
  },
}));

import OperatorDecisionSection from './OperatorDecisionSection';

const decision = (over: Record<string, unknown> = {}) => ({
  id: 'dec-containment-exemption',
  projectId: 'p-1',
  target: '要不要给合并防护加删除豁免',
  choices: ['加', '不加'],
  decider: 'human:supervisor',
  consequence: '加了才能合掉那个死代码删除;不加就得改派单',
  evidence: ['disp-DFeH6Ut_aNfzIbuuR21V5'],
  status: 'pending',
  createdBy: 'seat:seat-aegis-001',
  createdAt: '2026-10-22T09:00:00Z',
  chosen: null,
  resolvedBy: null,
  resolvedAt: null,
  ...over,
});

const renderSection = () => render(<OperatorDecisionSection live />);

describe('OperatorDecisionSection', () => {
  beforeEach(() => {
    cleanup();
    // Declared precondition, not an inherited default: this section reads under a
    // project, so the tests say which one. They used to rely on useDataStore's
    // launch default ('p-1', a SEED_PROJECTS_FOR_TESTS id naming no real project);
    // that default became null on 2026-10-22 when the restart-restore fix landed.
    useDataStore.setState({ activeProjectId: 'seatloom', backendHydrated: true });
    state.rows = [];
    state.resolve = vi.fn(async () => ({}));
  });

  it('says there is nothing waiting, and why other things do not belong here', async () => {
    renderSection();
    expect(await screen.findByText('没有等你裁定的事。')).toBeTruthy();
    // The sentence that keeps this from becoming a catch-all: a blocked dispatch
    // or a failed read belongs to a seat or to Aegis, not to the operator.
    expect(screen.getByText(/被卡住的派单、失败的读取、打回重做/)).toBeTruthy();
  });

  it('shows the three things that make a decision possible', async () => {
    state.rows = [decision()];
    renderSection();
    expect(await screen.findByText('要不要给合并防护加删除豁免')).toBeTruthy();
    // The options, as buttons — there is no free-text ruling.
    expect(screen.getByRole('button', { name: '加' })).toBeTruthy();
    expect(screen.getByRole('button', { name: '不加' })).toBeTruthy();
    // The consequence, stated BEFORE the buttons: what follows from deciding is
    // the thing that makes it a decision rather than a coin flip in the dark.
    expect(screen.getByText(/定了之后:/)).toBeTruthy();
    expect(screen.getByText('加了才能合掉那个死代码删除;不加就得改派单')).toBeTruthy();
    // And the support chain.
    expect(screen.getByText(/disp-DFeH6Ut_aNfzIbuuR21V5/)).toBeTruthy();
  });

  it('rules with one of the offered options and shows what was recorded', async () => {
    state.rows = [decision()];
    state.resolve = vi.fn(async () => ({
      decisionId: 'dec-containment-exemption',
      status: 'resolved',
      chosen: '加',
      resolvedBy: 'human:supervisor',
      target: '要不要给合并防护加删除豁免',
    }));
    renderSection();
    fireEvent.click(await screen.findByRole('button', { name: '加' }));

    await waitFor(() =>
      expect(state.resolve).toHaveBeenCalledWith('dec-containment-exemption', '加'),
    );
    // The ruling is shown in place — not reloaded away before it can be read.
    expect(await screen.findByText('已裁定')).toBeTruthy();
    expect(screen.getByText(/裁定:/)).toBeTruthy();
  });

  it('shows a refusal verbatim rather than swallowing it', async () => {
    state.rows = [decision()];
    state.resolve = vi.fn(async () => {
      throw new Error('operator decision "dec-x" is already resolved — a recorded ruling is immutable');
    });
    renderSection();
    fireEvent.click(await screen.findByRole('button', { name: '不加' }));

    // "already resolved" and "not the decider" call for different actions, so
    // neither may be collapsed into a generic failure.
    expect(await screen.findByText(/already resolved/)).toBeTruthy();
  });

  it('does not render an acceptance card shape', async () => {
    state.rows = [decision()];
    renderSection();
    await screen.findByText('要不要给合并防护加删除豁免');
    // No acceptance vocabulary anywhere on this section.
    expect(screen.queryByText('待验收')).toBeNull();
    expect(screen.queryByRole('button', { name: /接受/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /打回/ })).toBeNull();
    // And the header names it as a different question.
    expect(screen.getByText('等你裁定')).toBeTruthy();
    expect(screen.getByText(/和上面的「待验收」不是一回事/)).toBeTruthy();
  });
});
