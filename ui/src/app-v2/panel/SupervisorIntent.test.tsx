// SupervisorIntent — slice 2 of LYRA-2026-10-13-supervisor-intent-entry-contract-v1.
//
// Under test is this surface's own behaviour against the contract:
//   1. wiring honesty — a MEASURED probe, never a hand-maintained boolean. It
//      reports "reachable (measured)" or "unavailable (measured)" from the
//      actual call result; it never asserts a state it did not measure (V8/b3).
//   2. continuity-delta — reads real commands, links each row to its ledger
//      object, and an empty block says WHY it is empty and WHAT is next (never
//      a blank frame).
//   3. decision-answer — exact retrieval over the knowledge read model; a topic
//      with no active memories is a meaningful empty, and every card carries its
//      source.
//   4. the no-guessing rule — the structured entry, whose parser is not yet a
//      callable command, STOPS at a measured clarification. It does not guess a
//      proposal and does not fake a "not wired" flag.
//   5. two modes, never blended — 派活 (structured) and 说句话 (raw IM) submit
//      different things.
//
// The api layer is mocked at the module boundary exactly as the component
// consumes it. Nothing here stands in for a real backend value.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  seatsOk: true,
  seats: [
    {
      id: 'seat-nimbus-001',
      name: 'nimbus',
      capabilityTags: ['backend'],
      status: 'idle',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
      createdAt: '',
    },
    {
      id: 'seat-mira-001',
      name: 'mira',
      capabilityTags: ['ui'],
      status: 'idle',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
      createdAt: '',
    },
  ],
  acceptances: [] as unknown[],
  dispatches: [] as unknown[],
  handoffs: [] as unknown[],
  events: [] as unknown[],
  checkpoints: [] as unknown[],
  tree: { projectId: 'seatloom', topics: [] as unknown[] },
  memories: null as unknown,
  rawSent: [] as string[],
}));

vi.mock('../../lib/api', () => ({
  isTauri: () => true,
  api: {
    listSeats: vi.fn(async () => {
      if (!state.seatsOk) throw new Error('db unreachable');
      return state.seats;
    }),
    listAcceptances: vi.fn(async () => state.acceptances),
    listDispatches: vi.fn(async () => state.dispatches),
    listHandoffs: vi.fn(async () => state.handoffs),
    listEvents: vi.fn(async () => state.events),
    listCheckpoints: vi.fn(async () => state.checkpoints),
    getKnowledgeTree: vi.fn(async () => state.tree),
    getTopicMemories: vi.fn(async () => state.memories),
  },
}));

import { SupervisorIntent } from './SupervisorIntent';

const acc = (over: Record<string, unknown> = {}) => ({
  id: 'acc-1',
  dispatchId: 'disp-1',
  dispatchTitle: '知识面板验收',
  seatId: 'seat-mira-001',
  branch: 'packet/mira/knowledge-panel',
  base: 'track/infra-foundation',
  howToVerify: '看到面板',
  fluxVerdict: { refId: 'ev-v', verdict: 'pass', occurredAt: '' },
  status: 'awaiting_acceptance',
  createdAt: '',
  ...over,
});
const disp = (over: Record<string, unknown> = {}) => ({
  id: 'disp-x',
  targetSeatId: 'seat-nimbus-001',
  issuedBy: 'human:supervisor',
  projectId: 'seatloom',
  title: '接唤醒服务兜底',
  payload: null,
  status: 'in_flight',
  priority: 10,
  issuedAt: '',
  startedAt: null,
  completedAt: null,
  deps: [],
  ...over,
});

const mount = () => render(<SupervisorIntent onRawSend={(t) => state.rawSent.push(t)} />);

describe('SupervisorIntent', () => {
  beforeEach(() => {
    cleanup();
    state.seatsOk = true;
    state.acceptances = [];
    state.dispatches = [];
    state.handoffs = [];
    state.events = [];
    state.checkpoints = [];
    state.tree = { projectId: 'seatloom', topics: [] };
    state.memories = null;
    state.rawSent = [];
  });

  // 1. Wiring honesty — measured, both directions, never a hardcoded claim.
  it('reports the backend as reachable (measured) when the probe call succeeds', async () => {
    mount();
    await waitFor(() => expect(screen.getByText('后端可达(实测)')).toBeTruthy());
  });

  it('reports the backend as unavailable (measured) with the real error when the probe call fails', async () => {
    state.seatsOk = false;
    mount();
    await waitFor(() => expect(screen.getByText('后端不可达(实测)')).toBeTruthy());
    // the measured error is carried on the affordance, not hidden
    const badge = screen.getByText('后端不可达(实测)');
    expect(badge.getAttribute('title')).toContain('db unreachable');
  });

  // 2. Continuity-delta — real reads; empty says why + what next.
  it('continuity-delta reads real commands and links each row to its ledger object', async () => {
    state.acceptances = [acc()];
    state.dispatches = [disp()];
    mount();
    fireEvent.click(screen.getByText('读取'));
    await waitFor(() => expect(screen.getByText(/知识面板验收/)).toBeTruthy());
    expect(screen.getByText(/packet\/mira\/knowledge-panel/)).toBeTruthy();
    expect(screen.getByText(/disp-x/)).toBeTruthy(); // dispatch id linked
    expect(screen.getByText(/接唤醒服务兜底/)).toBeTruthy();
  });

  it('continuity-delta empty blocks say why empty and what next, never a blank frame', async () => {
    mount();
    fireEvent.click(screen.getByText('读取'));
    await waitFor(() => expect(screen.getByText(/没有待运营者判定的验收卡/)).toBeTruthy());
    expect(screen.getByText(/新交付经 Flux 验绿后会在这里出现/)).toBeTruthy();
  });

  // 3. Decision-answer — exact retrieval; no active memories is a meaningful empty.
  it('decision-answer lists topics and shows each memory with its source', async () => {
    state.tree = {
      projectId: 'seatloom',
      topics: [
        {
          topicId: 'topic-a',
          title: '舰队运行',
          status: 'open',
          parentTopicId: null,
          memoryCount: 1,
        },
      ],
    };
    state.memories = {
      topicId: 'topic-a',
      depth: 'l0',
      topicChain: [{ topicId: 'topic-a', title: '舰队运行' }],
      memories: [
        {
          memoryId: 'mem-1',
          topicId: 'topic-a',
          title: '别绕过约束',
          kind: 'preference',
          confidence: 'quoted',
          tier: 'cold',
          status: 'active',
          l0Summary: '被拒就停下上报',
          sourceKind: 'manual',
          sourceRef: 'supervisor affirmation',
          l2: null,
        },
      ],
    };
    mount();
    fireEvent.click(screen.getByText('读取主题'));
    await waitFor(() => expect(screen.getByText(/舰队运行 \(1\)/)).toBeTruthy());
    fireEvent.click(screen.getByText(/舰队运行 \(1\)/));
    await waitFor(() => expect(screen.getByText('别绕过约束')).toBeTruthy());
    expect(screen.getByText(/被拒就停下上报/)).toBeTruthy();
    expect(screen.getByText('supervisor affirmation')).toBeTruthy(); // source ref shown (its own node)
  });

  it('decision-answer with a topic that has no active memories is a meaningful empty, not a blank', async () => {
    state.tree = {
      projectId: 'seatloom',
      topics: [
        {
          topicId: 'topic-a',
          title: '空主题',
          status: 'open',
          parentTopicId: null,
          memoryCount: 0,
        },
      ],
    };
    state.memories = {
      topicId: 'topic-a',
      depth: 'l0',
      topicChain: [{ topicId: 'topic-a', title: '空主题' }],
      memories: [],
    };
    mount();
    fireEvent.click(screen.getByText('读取主题'));
    await waitFor(() => expect(screen.getByText(/空主题 \(0\)/)).toBeTruthy());
    fireEvent.click(screen.getByText(/空主题 \(0\)/));
    await waitFor(() => expect(screen.getByText(/该主题下没有活跃记忆/)).toBeTruthy());
  });

  // 4. The no-guessing rule — the structured entry STOPS, it does not guess.
  it('structured submit stops at a MEASURED clarification, never a guessed proposal or a fake flag', async () => {
    mount();
    const input = screen.getByPlaceholderText(/说你想干什么/);
    fireEvent.change(input, { target: { value: '给 Nimbus 派个活,把唤醒服务兜底接上' } });
    fireEvent.click(screen.getByText('拆解'));
    await waitFor(() => expect(screen.getByText('需要你澄清 · 我不猜')).toBeTruthy());
    // measured fact, not a hardcoded "commands not online" lie:
    expect(screen.getByText(/deterministic intent parser not yet a callable command/)).toBeTruthy();
    expect(screen.getByText(/实测,非猜测/)).toBeTruthy();
    // no fabricated proposal leaked through:
    expect(screen.queryByText(/150,000 tokens/)).toBeNull();
  });

  // 5. Two modes, never blended.
  it('dispatch mode submits to the structured path; chat mode sends raw and clears', async () => {
    mount();
    // chat mode → 发送 button, raw send
    fireEvent.click(screen.getByText('说句话'));
    const input = screen.getByPlaceholderText(/说给当前席位/);
    fireEvent.change(input, { target: { value: '你好 nimbus' } });
    fireEvent.click(screen.getByText('发送'));
    expect(state.rawSent).toEqual(['你好 nimbus']);
    // back to dispatch mode → 拆解 button present, 发送 gone (no blended submission)
    fireEvent.click(screen.getByText('派活'));
    expect(screen.getByText('拆解')).toBeTruthy();
    expect(screen.queryByText('发送')).toBeNull();
  });
});
