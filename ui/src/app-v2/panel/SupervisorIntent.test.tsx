// SupervisorIntent — slice 4 (wiring) of LYRA-2026-10-13-supervisor-intent-entry-contract-v1.
//
// Under test is this surface's behaviour now that the 派活 entry CALLS the live
// intent tool (api.planIntent → cmd_plan_intent → plan_intent, actor human:supervisor):
//   1. wiring honesty — a MEASURED probe, never a hand-maintained boolean.
//   2. continuity-delta / decision-answer — real reads; empty says why + what next.
//   3. the REAL proposal — every field from the tool's output, plus the computed
//      "why this seat" rationale; nothing hardcoded survives.
//   4. Confirm writes through the governed path (enqueueDispatch) and shows the
//      dispatch id; on failure the error is verbatim and NO id is shown.
//   5. a proposal carrying a dependency edge the write path cannot express is
//      REFUSED explicitly, never silently dropped (contract §3.3, V5).
//   6. plan_intent's clarification renders the real question + choices — it ASKS,
//      it never guesses. A rejection is shown verbatim.
//   7. two modes, never blended.
//
// The api layer is mocked at the module boundary exactly as the component consumes it.

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
  // Per-project acceptance rows, so the mock can answer the way the Rust command
  // does — by project_id. Without this a client-side scoping bug is invisible.
  acceptancesByProject: {} as Record<string, unknown[]>,
  projects: [] as Array<{ id: string; name: string }>,
  dispatches: [] as unknown[],
  handoffs: [] as unknown[],
  events: [] as unknown[],
  checkpoints: [] as unknown[],
  tree: { projectId: 'seatloom', topics: [] as unknown[] },
  memories: null as unknown,
  rawSent: [] as string[],
  // plan_intent / enqueueDispatch
  planRaw: '' as string,
  planThrows: '' as string,
  enqueueId: 'disp-new-1',
  enqueueThrows: '' as string,
  planCalls: [] as string[],
  enqueueCalls: [] as unknown[][],
}));

vi.mock('../../lib/api', () => ({
  isTauri: () => true,
  api: {
    listSeats: vi.fn(async () => {
      if (!state.seatsOk) throw new Error('db unreachable');
      return state.seats;
    }),
    // Scoped server-side, like the real command: it takes a project_id and filters
    // by it. Answering the whole list regardless would make a client-side scoping
    // bug invisible — any code asking about the wrong project would still get the
    // cards. `acceptancesByProject` holds the rows for projects OTHER than the
    // active one, which is what the "empty here, waiting elsewhere" test needs.
    listAcceptances: vi.fn(async (projectId: string) =>
      projectId === 'seatloom'
        ? (state.acceptances as never[])
        : ((state.acceptancesByProject[projectId] as never[]) ?? []),
    ),
    listDispatches: vi.fn(async () => state.dispatches),
    listHandoffs: vi.fn(async () => state.handoffs),
    listEvents: vi.fn(async () => state.events),
    listCheckpoints: vi.fn(async () => state.checkpoints),
    getKnowledgeTree: vi.fn(async () => state.tree),
    getTopicMemories: vi.fn(async () => state.memories),
    planIntent: vi.fn(async (intent: string) => {
      state.planCalls.push(intent);
      if (state.planThrows) throw new Error(state.planThrows);
      return state.planRaw;
    }),
    enqueueDispatch: vi.fn(async (...args: unknown[]) => {
      state.enqueueCalls.push(args);
      if (state.enqueueThrows) throw new Error(state.enqueueThrows);
      return state.enqueueId;
    }),
  },
}));

import { SupervisorIntent } from './SupervisorIntent';
import { useDataStore } from '../../stores/useDataStore';

const proposalJson = (over: Record<string, unknown> = {}) =>
  JSON.stringify({
    contract: 'LYRA-2026-10-13-supervisor-intent-entry-contract-v1',
    outcome: 'proposal',
    wrote: false,
    proposal: {
      intentRestated: '给 Nimbus 派活,把唤醒服务兜底接上',
      proposedObject: {
        kind: 'dispatch',
        title: '接唤醒服务兜底',
        payloadNote: '把唤醒服务硬底兜底接上',
        suggestedDependencies: [],
      },
      ownership: {
        seatId: 'seat-nimbus-001',
        seatName: 'nimbus',
        role: 'backend',
        capabilityMatch: ['backend'],
        authorityDocs: [],
        constraints: [],
      },
      evidence: [{ label: '席位', source: 'seats' }],
      budget: { seatInFlightDispatches: 0, note: 'budget read from projects' },
      impact: ['收件箱出现新派单回到 Nimbus'],
      routingRationale: [
        {
          seatId: 'seat-nimbus-001',
          seatName: 'nimbus',
          eligible: true,
          matchedCapabilities: ['backend'],
          inFlightDispatches: 0,
          selected: true,
          source: 'seats',
        },
        {
          seatId: 'seat-mira-001',
          seatName: 'mira',
          eligible: false,
          matchedCapabilities: [],
          excludingConstraint: 'no backend capability',
          inFlightDispatches: 2,
          selected: false,
          source: 'project_role_bindings',
        },
      ],
      precondition: {},
      actions: ['confirm', 'edit', 'cancel'],
      actor: 'human:supervisor',
      ...over,
    },
  });

const clarificationJson = () =>
  JSON.stringify({
    contract: 'LYRA-2026-10-13-supervisor-intent-entry-contract-v1',
    outcome: 'clarification',
    wrote: false,
    clarification: {
      ruleId: 'verb-unclear',
      question: '这句话的意图不在可执行集合里,你要做哪一种?',
      reason: 'verb not in the supported set',
      choices: [
        { value: 'create', label: 'create', why: '建一个新的东西' },
        { value: 'route', label: 'route', why: '把一件事指给某个席位' },
      ],
    },
  });

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
const typeAndSubmit = (v: string) => {
  fireEvent.change(screen.getByPlaceholderText(/说你想干什么/), { target: { value: v } });
  fireEvent.click(screen.getByText('拆解'));
};

describe('SupervisorIntent', () => {
  beforeEach(() => {
    cleanup();
    // Declared precondition, not an inherited default: every read in this surface
    // is project-scoped, so the tests say which project they run under. They used
    // to rely on useDataStore's launch default — a SEED_PROJECTS_FOR_TESTS id that
    // named no project in any database.
    useDataStore.setState({
      activeProjectId: 'seatloom',
      backendHydrated: true,
      projects: [{ id: 'seatloom', name: 'SeatLoom' }, ...state.projects] as never,
    });
    state.seatsOk = true;
    state.acceptances = [];
    state.acceptancesByProject = {};
    // The OTHER projects the elsewhere-check reads. Set per test; the active
    // project ('seatloom') is always added by the store setup below.
    state.projects = [];
    // Re-seed the store, because the beforeEach above ran before these were set.
    useDataStore.setState({
      activeProjectId: 'seatloom',
      backendHydrated: true,
      projects: [{ id: 'seatloom', name: 'SeatLoom' }, ...state.projects] as never,
    });
    state.dispatches = [];
    state.handoffs = [];
    state.events = [];
    state.checkpoints = [];
    state.tree = { projectId: 'seatloom', topics: [] };
    state.memories = null;
    state.rawSent = [];
    state.planRaw = '';
    state.planThrows = '';
    state.enqueueId = 'disp-new-1';
    state.enqueueThrows = '';
    state.planCalls = [];
    state.enqueueCalls = [];
  });

  // 1. Wiring honesty — measured, both directions.
  it('reports the backend as reachable (measured) when the probe call succeeds', async () => {
    mount();
    await waitFor(() => expect(screen.getByText('后端可达(实测)')).toBeTruthy());
  });
  it('reports the backend as unavailable (measured) with the real error when the probe fails', async () => {
    state.seatsOk = false;
    mount();
    await waitFor(() => expect(screen.getByText('后端不可达(实测)')).toBeTruthy());
    expect(screen.getByText('后端不可达(实测)').getAttribute('title')).toContain('db unreachable');
  });

  // 2. continuity-delta — real reads; empty says why + what next.
  it('continuity-delta reads real commands and links each row to its ledger object', async () => {
    state.acceptances = [acc()];
    state.dispatches = [disp()];
    mount();
    fireEvent.click(screen.getByText('读取'));
    await waitFor(() => expect(screen.getByText(/知识面板验收/)).toBeTruthy());
    expect(screen.getByText(/packet\/mira\/knowledge-panel/)).toBeTruthy();
    expect(screen.getByText(/disp-x/)).toBeTruthy();
  });
  it('continuity-delta empty blocks say why empty and what next, never a blank frame', async () => {
    mount();
    fireEvent.click(screen.getByText('读取'));
    await waitFor(() => expect(screen.getByText(/没有待运营者判定的验收卡/)).toBeTruthy());
    expect(screen.getByText(/新交付经 Flux 验绿后会在这里出现/)).toBeTruthy();
  });

  it('an empty acceptance read says WHERE the cards are instead of "none waiting"', async () => {
    // The failure this closes, measured: the operator's three cards all belong to
    // seatloom while his active project held none, so this panel showed an empty
    // "no pending cards" — which is worse than not showing him, because it
    // actively told him there was nothing to do.
    //
    // Lyra then measured one worse: his active project was `p-1`, the first entry
    // of SEED_PROJECTS_FOR_TESTS, which has no row in projects at all. So the read
    // was not scoped to the wrong project — it was scoped to a project that does
    // not exist.
    //
    // Two independent reasons, one shape: a scoped read that comes back empty must
    // not be reported as "nothing is waiting for you". The two readings demand
    // opposite actions from him.
    //
    // The mock answers per project, exactly like the Rust command does — it takes
    // a project_id and filters by it. Answering the whole list regardless would
    // make the scoping bug invisible, because any code asking about the wrong
    // project would still get the cards.
    state.acceptances = [];
    state.acceptancesByProject = { other: [acc({ id: 'acc-elsewhere' })] };
    state.projects = [{ id: 'other', name: 'Other' }];
    useDataStore.setState({
      activeProjectId: 'seatloom',
      backendHydrated: true,
      projects: [
        { id: 'seatloom', name: 'SeatLoom' },
        { id: 'other', name: 'Other' },
      ] as never,
    });
    mount();
    fireEvent.click(screen.getByText('读取'));

    // It must NOT claim there is nothing waiting.
    await waitFor(() => expect(screen.queryByText(/没有待运营者判定的验收卡/)).toBeNull());
    // It says how many and under which project — a number he can act on.
    await waitFor(() => expect(screen.getByText(/别处有 1 张在等你/)).toBeTruthy());
    expect(screen.getByText(/other/)).toBeTruthy();
    // And it points at the surface that shows them, rather than leaving him to
    // guess where a count lives. Matched on the full sentence, because '派单'
    // alone matches several surfaces.
    expect(screen.getByText(/左边栏「派单」上的数字就是它们/)).toBeTruthy();
  });

  // 3. decision-answer — real reads; no active memories is a meaningful empty.
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
    expect(screen.getByText('supervisor affirmation')).toBeTruthy();
  });
  it('decision-answer with a topic that has no active memories is a meaningful empty', async () => {
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

  // 4. The REAL proposal — plan_intent is called and every field is its output.
  it('structured submit calls plan_intent and renders the real proposal (nothing hardcoded)', async () => {
    state.planRaw = proposalJson();
    mount();
    typeAndSubmit('给 Nimbus 派个活,把唤醒服务兜底接上');
    await waitFor(() => expect(screen.getByText(/给 Nimbus 派活,把唤醒服务兜底接上/)).toBeTruthy());
    expect(state.planCalls).toEqual(['给 Nimbus 派个活,把唤醒服务兜底接上']); // the sentence went to the tool
    expect(screen.getByText(/nimbus\(seat-nimbus-001\)/)).toBeTruthy(); // ownership from the tool
    expect(screen.getByText(/worker .* supervisor .* 在途 0/)).toBeTruthy(); // computed budget
    // the computed "why this seat" disclosure is present, with per-seat facts
    fireEvent.click(screen.getByText(/为什么是这个席位\?/));
    await waitFor(() => expect(screen.getByText(/no backend capability/)).toBeTruthy()); // excluding constraint, a fact
  });

  // 5. Confirm writes through the governed path and shows the dispatch id.
  it('Confirm calls enqueueDispatch and shows the returned dispatch id', async () => {
    state.planRaw = proposalJson();
    state.enqueueId = 'disp-real-777';
    mount();
    typeAndSubmit('给 Nimbus 派个活,把唤醒服务兜底接上');
    await waitFor(() => expect(screen.getByText('确认并建卡')).toBeTruthy());
    fireEvent.click(screen.getByText('确认并建卡'));
    await waitFor(() => expect(screen.getByText(/disp-real-777/)).toBeTruthy());
    expect(state.enqueueCalls[0]).toEqual([
      'seat-nimbus-001',
      '接唤醒服务兜底',
      '把唤醒服务硬底兜底接上',
    ]);
  });

  // 6. A rejection from plan_intent is shown verbatim, no dispatch id.
  it('a plan_intent rejection is shown verbatim and shows no dispatch id', async () => {
    state.planThrows = 'plan_intent rejected: project "seatloom" does not exist';
    mount();
    typeAndSubmit('给 Nimbus 派个活');
    await waitFor(() => expect(screen.getByText(/project "seatloom" does not exist/)).toBeTruthy());
    expect(screen.queryByText(/确认并建卡/)).toBeNull();
    expect(state.enqueueCalls).toEqual([]); // nothing written
  });

  // 7. A dependency edge the write path cannot express is refused, not dropped.
  it('a proposal carrying a dependency edge is refused explicitly, never silently dropped', async () => {
    state.planRaw = proposalJson({
      proposedObject: {
        kind: 'dispatch',
        title: '接唤醒服务兜底',
        payloadNote: '…',
        suggestedDependencies: [
          { depKind: 'event', depValue: 'ev-verdict-supervisor-intent-tools' },
        ],
      },
    });
    mount();
    typeAndSubmit('等 Flux 判定落地再接唤醒服务兜底');
    await waitFor(() => expect(screen.getByText(/无法表达依赖边/)).toBeTruthy());
    expect(screen.getByText(/ev-verdict-supervisor-intent-tools/)).toBeTruthy(); // the edge is named
    expect(screen.queryByText('确认并建卡')).toBeNull(); // no silent confirm
    expect(state.enqueueCalls).toEqual([]); // nothing written
  });

  // 8. plan_intent clarification — it ASKS with the real choices, never guesses.
  it('a clarification renders the real question and choices, no guessed proposal', async () => {
    state.planRaw = clarificationJson();
    mount();
    typeAndSubmit('把这个弄一下');
    await waitFor(() => expect(screen.getByText('需要你澄清 · 我不猜')).toBeTruthy());
    expect(screen.getByText(/这句话的意图不在可执行集合里/)).toBeTruthy(); // the tool's question
    expect(screen.getByText('create')).toBeTruthy(); // real choice
    expect(screen.getByText('route')).toBeTruthy();
    expect(state.enqueueCalls).toEqual([]); // no write on a stop
  });

  // 9. Two modes, never blended.
  it('dispatch mode submits to plan_intent; chat mode sends raw and clears', async () => {
    state.planRaw = proposalJson();
    mount();
    // chat mode → raw send
    fireEvent.click(screen.getByText('说句话'));
    fireEvent.change(screen.getByPlaceholderText(/说给当前席位/), {
      target: { value: '你好 nimbus' },
    });
    fireEvent.click(screen.getByText('发送'));
    expect(state.rawSent).toEqual(['你好 nimbus']);
    // back to dispatch → 拆解 present, 发送 gone
    fireEvent.click(screen.getByText('派活'));
    expect(screen.getByText('拆解')).toBeTruthy();
    expect(screen.queryByText('发送')).toBeNull();
  });
});

// 2026-10-14 张先生实测:「点了 create 没反应」。
// 根因:pick 只把选项文字填回输入框、清掉卡片,不重新拆解 —— 用户必须自己再按一次拆解。
// 修好后的行为:点了选项直接再调一次 planIntent(用模块级 state.planCalls 断言调用次数)。
describe('澄清选项被点后必须直接继续,不要把活推回给用户', () => {
  it('pick 之后 planIntent 被再调一次,用户不需要按第二次拆解', async () => {
    state.planRaw = JSON.stringify({
      outcome: 'clarification',
      clarification: {
        question: '这句话里没有我能确定执行的动作。你要做下面哪一种?',
        choices: [
          {
            label: 'create — 给 nimbus 派个活,把唤醒服务的硬底兜底接上',
            value: 'create',
            why: 'one of the five verbs',
          },
        ],
      },
    });
    render(<SupervisorIntent onRawSend={vi.fn()} />);
    fireEvent.change(screen.getByPlaceholderText(/说你想干什么/), {
      target: { value: '给后端席位派一张单' },
    });
    fireEvent.click(screen.getByText('拆解'));
    await screen.findByText(/你要做下面哪一种/);
    expect(state.planCalls).toHaveLength(1);
    // 点选项 —— 这一下就必须自己往下走
    fireEvent.click(screen.getByText(/create — 给 nimbus 派个活/));
    await waitFor(() => expect(state.planCalls).toHaveLength(2));
    // 第二次调用带上了选项原文(而不是空、也不是原来那句)
    expect(state.planCalls[1]).toContain('create');
  });
});
