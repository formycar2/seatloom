// KnowledgeSection — the D5 knowledge panel (overnight headline step 5).
//
// Mr. Zhang wakes up and must see, in the SeatLoom main interface, knowledge
// distilled from his own historical sessions: a project → topic tree; click a
// topic → memory cards; each card defaults to a one-sentence summary, expand for
// detail, expand again for line-level evidence pointing back at the raw file.
//
// What is under test is this component's own behaviour against the DTO contract
// Lyra pinned (the same text Nimbus's read-model lane codes against):
//   1. the measured wiring probe — the read fails ⇒ the panel SAYS SO, verbatim,
//      issues no call, and shows no fake data (house rule, no mocks);
//   2. the topic tree — roots, declared child order, memory counts;
//   3. data mapping — kind/confidence/tier/status badges and the l0 one-liner;
//   4. the three disclosure levels, exactly L0 → L1 → L2 and back;
//   5. the L2 evidence row, including its honest partial cases;
//   6. a MEANINGFUL empty state — why it is empty and what happens next.
//
// The api layer is mocked at the module boundary exactly as the component
// consumes it. That is the seam, not a fixture: nothing here stands in for a real
// backend value.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  topics: [] as unknown[],
  memories: null as unknown,
  /** Every invoke, recorded as [command, arguments] — the wire is the contract. */
  calls: [] as [string, Record<string, unknown>][],
  getTree: vi.fn(async () => ({ projectId: 'seatloom', topics: state.topics })),
  getTopicMemories: vi.fn(async () => state.memories),
}));

// The api layer is mocked at the module boundary exactly as the component
// consumes it. The flag is the seam between "Nimbus's read-model commands are on
// track" and "they are not"; a prop lets a test cross it both ways.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    state.calls.push([cmd, args ?? {}]);
    if (cmd === 'cmd_get_knowledge_tree') return state.getTree();
    if (cmd === 'cmd_get_topic_memories') return state.getTopicMemories();
    throw new Error(`unexpected invoke: ${cmd}`);
  }),
}));

/** The commands actually issued, in order. */
const commands = () => state.calls.map(([c]) => c);
const lastArgs = (cmd: string) => {
  for (let i = state.calls.length - 1; i >= 0; i--) {
    if (state.calls[i][0] === cmd) return state.calls[i][1];
  }
  return undefined;
};

import { KnowledgeSection } from './KnowledgeSection';

const topic = (over: Record<string, unknown> = {}) => ({
  topicId: 'topic-a',
  title: '验收卡改版',
  status: 'open',
  parentTopicId: null,
  childTopicIds: [],
  memoryCount: 2,
  ...over,
});

const memory = (over: Record<string, unknown> = {}) => ({
  memoryId: 'mem-1',
  topicId: 'topic-a',
  title: '坏卡作废需要先回填 plan',
  kind: 'failure',
  confidence: 'quoted',
  tier: 'cold',
  status: 'active',
  l0Summary: 'cancel_acceptance 会被 016 的 CHECK 挡住,因为遗留行的 acceptance_plan 是 {}。',
  l1Overview: '根因:016 的 CHECK 是 NOT VALID,只豁免既有行的首次校验;任何 UPDATE 都会重新检查。',
  l2: {
    refKind: 'file',
    filePath: 'infra/postgres/schema/016_acceptance_plan.sql',
    bodyDigest: '9f2c1ab4',
    lineStart: 70,
    lineEnd: 86,
  },
  sourceKind: 'canonical_event',
  sourceRef: 'ev-p8_O_dH_fzVaV1n4KySRW',
  ...over,
});

// The gate is now a MEASURED probe, not a hand-written boolean — so a test that
// wants the not-wired side injects the probe result rather than flipping a flag.
const probeUnavailable = (error = 'cmd_get_knowledge_tree not found') =>
  ({ kind: 'unavailable', error }) as const;

const renderPanel = () => render(<KnowledgeSection projectId="seatloom" />);

/** Registered commands + a tree + one topic's memories. */
const wired = (over: Record<string, unknown> = {}) => {
  state.topics = [topic(over)];
  state.memories = {
    topicId: 'topic-a',
    topicChain: [
      { topicId: 'topic-root', title: '验收' },
      { topicId: 'topic-a', title: '验收卡改版' },
    ],
    depth: 'l0',
    memories: [memory()],
  };
};

describe('KnowledgeSection', () => {
  beforeEach(() => {
    cleanup();
    state.topics = [];
    state.memories = null;
    state.getTree.mockClear();
    state.getTopicMemories.mockClear();
    state.calls.length = 0;
  });

  // ── 1. The gate: a measured probe, not a hand-written boolean ──────────────
  it('reports the measured failure verbatim and shows no fake data', () => {
    // The shipped default no longer asserts anything: the panel probes for real.
    // This is the probe's unavailable branch — the measured error text is shown,
    // not paraphrased, and not one read of topic data is attempted.
    render(<KnowledgeSection projectId="seatloom" probe={probeUnavailable()} />);
    expect(screen.getByText(/读取命令调不通/)).toBeTruthy();
    expect(screen.getByText(/cmd_get_knowledge_tree not found/)).toBeTruthy();
    expect(screen.getByText(/不用 mock 伪装后端真值/)).toBeTruthy();
    // The probe itself made one call; nothing beyond it.
    expect(commands()).toEqual(['cmd_get_knowledge_tree']);
    expect(screen.queryByText('坏卡作废需要先回填 plan')).toBeNull();
    expect(screen.queryByText(/cancel_acceptance 会被 016/)).toBeNull();
  });

  // ── 2. The topic tree ──────────────────────────────────────────────────────
  it('renders the topic tree with memory counts and loads a topic on click', async () => {
    wired();
    renderPanel();
    const btn = await screen.findByRole('button', { name: /验收卡改版/ });
    expect(screen.getByText('2')).toBeTruthy(); // memoryCount badge
    fireEvent.click(btn);
    await waitFor(() =>
      expect(lastArgs('cmd_get_topic_memories')).toEqual({
        topicId: 'topic-a',
        depth: 'l0',
        includeSuperseded: false,
      }),
    );
    // The ancestor chain is shown ancestors-first, so a memory is never read out
    // of context: the ancestor title appears in the chain, and the topic appears
    // both there and in the tree.
    expect(screen.getByText('验收')).toBeTruthy();
    expect(screen.getAllByText('验收卡改版').length).toBeGreaterThan(1);
    expect(screen.getByText('坏卡作废需要先回填 plan')).toBeTruthy();
  });

  it('orders children by the declared childTopicIds, not by array order', async () => {
    state.topics = [
      topic({ topicId: 't-root', title: '根主题', childTopicIds: ['t-b', 't-a'] }),
      topic({ topicId: 't-a', title: '后到的子题', parentTopicId: 't-root', memoryCount: 0 }),
      topic({ topicId: 't-b', title: '先声明的子题', parentTopicId: 't-root', memoryCount: 1 }),
    ];
    state.memories = { topicId: 't-a', topicChain: [], depth: 'l0', memories: [] };
    renderPanel();
    await screen.findByRole('button', { name: /根主题/ });
    const order = screen.getAllByRole('button').map((b) => b.textContent ?? '');
    const declared = order.findIndex((t) => t.includes('先声明的子题'));
    const other = order.findIndex((t) => t.includes('后到的子题'));
    expect(declared).toBeGreaterThan(-1);
    expect(other).toBeGreaterThan(-1);
    // childTopicIds says ['t-b','t-a'] — that order wins over the array order.
    expect(declared).toBeLessThan(other);
  });

  // ── 3. Data mapping ────────────────────────────────────────────────────────
  it('maps kind / confidence / tier / status onto the card', async () => {
    wired();
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    await screen.findByText('坏卡作废需要先回填 plan');
    expect(screen.getByText('失败')).toBeTruthy(); // kind
    expect(screen.getByText('原话')).toBeTruthy(); // confidence
    expect(screen.getByText('冷')).toBeTruthy(); // tier
    // L0 is the one-sentence summary, and it is the DEFAULT layer.
    expect(screen.getByText(/cancel_acceptance 会被 016 的 CHECK 挡住/)).toBeTruthy();
    // L1 and L2 are NOT on the card until it is opened.
    expect(screen.queryByText(/根因:016 的 CHECK 是 NOT VALID/)).toBeNull();
    expect(screen.queryByText(/infra\/postgres\/schema\/016_acceptance_plan\.sql/)).toBeNull();
  });

  it('labels an unknown kind verbatim rather than inventing one', async () => {
    state.topics = [topic()];
    state.memories = {
      topicId: 'topic-a',
      topicChain: [],
      depth: 'l0',
      memories: [
        memory({ kind: 'something-new', confidence: 'maybe', tier: 'lukewarm', status: 'mystery' }),
      ],
    };
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    await screen.findByText('坏卡作废需要先回填 plan');
    // Unknown vocabulary is shown as recorded — never relabelled, never dropped.
    expect(screen.getByText('something-new')).toBeTruthy();
    expect(screen.getByText('maybe')).toBeTruthy();
    expect(screen.getByText('lukewarm')).toBeTruthy();
    expect(screen.getByText('mystery')).toBeTruthy();
    // And a known kind is still translated for the operator.
    expect(screen.queryByText('failure')).toBeNull();
  });

  // ── 4. The three disclosure levels ─────────────────────────────────────────
  it('walks L0 → L1 → L2 and back, exactly three levels', async () => {
    wired();
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    const card = await screen.findByText('坏卡作废需要先回填 plan');

    // L0: the summary is there, the detail is not.
    expect(screen.getByText(/cancel_acceptance 会被 016 的 CHECK 挡住/)).toBeTruthy();
    expect(screen.queryByText(/根因:016 的 CHECK 是 NOT VALID/)).toBeNull();

    // L0 → L1
    fireEvent.click(card);
    expect(await screen.findByText(/根因:016 的 CHECK 是 NOT VALID/)).toBeTruthy();
    expect(screen.queryByText(/infra\/postgres\/schema\/016_acceptance_plan\.sql/)).toBeNull();

    // L1 → L2 (the l2 evidence is fetched on demand)
    fireEvent.click(card);
    await waitFor(() =>
      expect(lastArgs('cmd_get_topic_memories')).toEqual({
        topicId: 'topic-a',
        depth: 'l2',
        includeSuperseded: false,
      }),
    );
    expect(
      await screen.findByText(/infra\/postgres\/schema\/016_acceptance_plan\.sql/),
    ).toBeTruthy();
    expect(screen.getByText('第 70–86 行')).toBeTruthy();
    expect(screen.getByText(/sha256: 9f2c1ab4/)).toBeTruthy();

    // And back to L0 — a third click collapses, it does not invent a fourth level.
    fireEvent.click(card);
    await waitFor(() => expect(screen.queryByText(/根因:016 的 CHECK 是 NOT VALID/)).toBeNull());
    expect(screen.queryByText(/infra\/postgres\/schema\/016_acceptance_plan\.sql/)).toBeNull();
    // The default layer is still there.
    expect(screen.getByText(/cancel_acceptance 会被 016 的 CHECK 挡住/)).toBeTruthy();
  });

  // ── 5. L2's honest partial cases ───────────────────────────────────────────
  it('names what is missing when the l2 evidence is partial', async () => {
    const openAt = async (l2: unknown) => {
      cleanup();
      state.topics = [topic()];
      state.memories = {
        topicId: 'topic-a',
        topicChain: [],
        depth: 'l0',
        memories: [memory({ l2 })],
      };
      renderPanel();
      fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
      const card = await screen.findByText('坏卡作废需要先回填 plan');
      fireEvent.click(card);
      await screen.findByText(/根因:016 的 CHECK 是 NOT VALID/); // L1
      fireEvent.click(card);
    };

    await openAt({ refKind: 'file' });
    expect(await screen.findByText('未记行号')).toBeTruthy();

    await openAt({ refKind: 'file', lineStart: 12 });
    expect(await screen.findByText('自第 12 行起')).toBeTruthy();

    await openAt({ refKind: 'file', lineEnd: 40 });
    expect(await screen.findByText('至第 40 行止')).toBeTruthy();

    await openAt({ refKind: 'file', lineStart: 3, lineEnd: 9 });
    expect(await screen.findByText('第 3–9 行')).toBeTruthy();
  });

  it('says so when a card carries no l1 and no l2, instead of rendering blanks', async () => {
    state.topics = [topic()];
    state.memories = {
      topicId: 'topic-a',
      topicChain: [],
      depth: 'l0',
      memories: [memory({ l1Overview: null, l2: null })],
    };
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    const card = await screen.findByText('坏卡作废需要先回填 plan');
    fireEvent.click(card);
    expect(await screen.findByText(/该记忆没有详述层/)).toBeTruthy();
    fireEvent.click(card);
    expect(await screen.findByText(/该记忆没有行级证据/)).toBeTruthy();
  });

  it('flags an l2 that names no documentId or filePath', async () => {
    state.topics = [topic()];
    state.memories = {
      topicId: 'topic-a',
      topicChain: [],
      depth: 'l0',
      memories: [memory({ l2: { refKind: 'document', documentId: null } })],
    };
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    const card = await screen.findByText('坏卡作废需要先回填 plan');
    fireEvent.click(card);
    await screen.findByText(/根因:016 的 CHECK 是 NOT VALID/); // L1 is on the card
    fireEvent.click(card);
    expect(await screen.findByText(/但没有记 documentId/)).toBeTruthy();
  });

  it('marks a superseded memory with what replaced it', async () => {
    state.topics = [topic()];
    state.memories = {
      topicId: 'topic-a',
      topicChain: [],
      depth: 'l0',
      memories: [memory({ status: 'forgotten', supersededBy: 'mem-9' })],
    };
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    const card = await screen.findByText('坏卡作废需要先回填 plan');
    expect(screen.getByText('已忘')).toBeTruthy();
    fireEvent.click(card);
    await screen.findByText(/根因:016 的 CHECK 是 NOT VALID/); // L1 is on the card
    fireEvent.click(card);
    expect(await screen.findByText(/已被/)).toBeTruthy();
    expect(screen.getByText('mem-9')).toBeTruthy();
  });

  // ── 6. Meaningful empty states ─────────────────────────────────────────────
  it('explains why a project with no topics is empty, and what happens next', async () => {
    state.topics = [];
    renderPanel();
    expect(await screen.findByText(/还没有任何主题/)).toBeTruthy();
    expect(screen.getByText(/project \+ topic 是唯一的作用域/)).toBeTruthy();
    expect(screen.getByText(/提炼流水线/)).toBeTruthy();
  });

  it('explains an open topic with no memories differently from no topics', async () => {
    state.topics = [topic()];
    state.memories = { topicId: 'topic-a', topicChain: [], depth: 'l0', memories: [] };
    renderPanel();
    fireEvent.click(await screen.findByRole('button', { name: /验收卡改版/ }));
    expect(await screen.findByText(/这个主题下还没有记忆/)).toBeTruthy();
    expect(screen.getByText(/还没有从历史会话里提炼出落在这个主题下的记忆/)).toBeTruthy();
  });

  // ── Errors, verbatim ───────────────────────────────────────────────────────
  it('shows a read refusal verbatim rather than an empty panel', async () => {
    // The probe succeeded (injected), so the panel is past the wiring gate; the
    // failure here is the tree read itself, and its text is shown verbatim.
    state.getTree.mockImplementation(async () => {
      throw new Error('project not found: seatloom');
    });
    render(<KnowledgeSection projectId="seatloom" probe={{ kind: 'ok' }} />);
    await screen.findByText(/读取主题树失败/);
    expect(screen.getByText(/project not found: seatloom/)).toBeTruthy();
  });

  it('reports a probing state instead of asserting the wiring while it measures', () => {
    render(<KnowledgeSection projectId="seatloom" probe={{ kind: 'probing' }} />);
    expect(screen.getByText(/正在实测读取命令是否可用/)).toBeTruthy();
    // Nothing is rendered as if it were wired.
    expect(screen.queryByText(/个主题/)).toBeNull();
  });
});
