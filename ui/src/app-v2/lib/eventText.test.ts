// eventText — the translation the Supervisor panel's activity feed depends on.
//
// The defect: `eventToMessage` recognised 22 of the vocabulary's 28 event types, and
// everything else rendered as `• ${etype}`. Measured over the 400 newest rows in the
// Aegis Supervisor column, 319 of them (79 %) fell through — so the operator read a
// wall of bare English type names and concluded the panel was a mockup. It was real
// data with no translation.
//
// Every payload shape asserted below is the one the writer actually emits, taken
// from the Rust (`repositories.rs` / `memory.rs` / `dispatch.rs`) or measured from
// the live ledger — not invented for the test. That matters because a test built on
// a guessed shape would pass while the real rows still fell through.

import { describe, expect, it } from 'vitest';
import { eventToText, type EventTextContext } from './eventText';
import type { CanonicalEventDto } from '../../lib/types-dto';

function evt(
  eventType: string,
  payload: unknown,
  actorRef = 'seat:seat-mira-001',
): CanonicalEventDto {
  return {
    id: 'ev-test',
    eventType,
    occurredAt: '2026-10-16T06:00:00Z',
    actorRef,
    payload,
    createdAt: '2026-10-16T06:00:00Z',
    projectId: 'seatloom',
  };
}

/** A context that resolves everything, the way a successful panel load would. */
const FULL: EventTextContext = {
  dispatches: new Map([
    [
      'disp-abc',
      { title: '给 SeatLoom 做高清应用图标', targetSeatId: 'seat-mira-001', status: 'in_flight' },
    ],
    ['disp-done', { title: '历史会话按项目', targetSeatId: 'seat-mira-001', status: 'done' }],
  ]),
  memories: new Map([
    ['mem-1', { title: '空数组的 bundle.icon 会静默失败', topicId: 'topic-app-icon' }],
  ]),
  seatNames: new Map([
    ['seat-mira-001', 'Mira'],
    ['seat-nimbus-001', 'Nimbus'],
    ['seat-aegis-001', 'Aegis'],
  ]),
};

describe('no line is a bare event type name', () => {
  // The 16 that used to fall through, plus the 12 that were already handled.
  const VOCABULARY = [
    'SessionStarted',
    'WorkItemCreated',
    'WorkItemStatusChanged',
    'ArtifactCreated',
    'HandoffSent',
    'HandoffAccepted',
    'HandoffCompleted',
    'ReviewVerdictIssued',
    'SeatDelegationIssued',
    'SeatDelegationClosed',
    'SupervisorMessage',
    'SeatResponse',
    'SeatDispatchIssued',
    'SeatDispatchDelivered',
    'SeatDispatchCompleted',
    'SeatDispatchFailed',
    'SeatDispatchRetried',
    'SeatDispatchCancelled',
    'AcceptanceRequested',
    'AcceptanceGranted',
    'AcceptanceRejected',
    'AcceptanceCancelled',
    'MemoryCreated',
    'MemoryRecalled',
    'MemoryPromoted',
    'MemoryForgotten',
    'MemoryResolved',
    'TopicOpened',
  ];

  it.each(VOCABULARY)('%s is translated into Chinese, not left as a type name', (type) => {
    const line = eventToText(evt(type, { dispatch_id: 'disp-abc' }), FULL);
    // Not the bare-name shape that was the defect.
    expect(line).not.toBe(`• ${type}`);
    expect(line.startsWith(`• ${type}`)).toBe(false);
    // The requirement was 说人话,全中文 — so the line contains Chinese. (The one
    // exception is the deliberate safety net, which says so in Chinese too.)
    expect(line).toMatch(/[\u4e00-\u9fff]/);
    // And it is not just the type name wearing punctuation.
    expect(line.replace(/^[^\w\s]+\s*/, '')).not.toBe(type);
  });

  it('names the type AND the payload keys when a type has no translation yet', () => {
    // The default branch is the safety net: a future vocabulary addition degrades
    // to "here is what it carried" instead of back to a bare word.
    const line = eventToText(evt('SomeFutureType', { foo: 1, bar: 2 }), FULL);
    expect(line).toContain('SomeFutureType');
    expect(line).toContain('foo');
    expect(line).toContain('bar');
    expect(line).toContain('还没有人话翻译');
  });
});

describe('dispatch lines say which work, to whom, what status', () => {
  it(' SeatDispatchIssued names the work, the target seat and the status', () => {
    const line = eventToText(
      evt('SeatDispatchIssued', { dispatch_id: 'disp-abc' }, 'seat:seat-aegis-001'),
      FULL,
    );
    expect(line).toContain('《给 SeatLoom 做高清应用图标》');
    expect(line).toContain('Mira');
    expect(line).toContain('进行中');
    // ...and does not leak the id as the content.
    expect(line).not.toContain('disp-abc');
  });

  it('SeatDispatchDelivered names the seat that received it', () => {
    const line = eventToText(evt('SeatDispatchDelivered', { dispatch_id: 'disp-abc' }), FULL);
    expect(line).toContain('Mira');
    expect(line).toContain('《给 SeatLoom 做高清应用图标》');
  });

  it('SeatDispatchCompleted names the seat that finished it', () => {
    const line = eventToText(
      evt('SeatDispatchCompleted', { dispatch_id: 'disp-done' }, 'seat:seat-mira-001'),
      FULL,
    );
    expect(line).toContain('Mira');
    expect(line).toContain('《历史会话按项目》');
  });

  it('SeatDispatchFailed quotes the reason and the retry budget', () => {
    const line = eventToText(
      evt('SeatDispatchFailed', {
        dispatch_id: 'disp-abc',
        reason: '根因:request_acceptance 没有 id 入参。',
        retry_count: 0,
        max_retries: 0,
        retries_exhausted: true,
      }),
      FULL,
    );
    expect(line).toContain('没做成');
    expect(line).toContain('request_acceptance 没有 id 入参');
    expect(line).toContain('0/0');
  });

  it('a long reason is cut, and the cut is stated rather than hidden', () => {
    const long = '啊'.repeat(5000);
    const line = eventToText(
      evt('SeatDispatchFailed', { dispatch_id: 'disp-abc', reason: long }),
      FULL,
    );
    expect(line).toContain('截断');
    expect(line).toContain('5000 字');
    expect(line.length).toBeLessThan(1000);
  });

  it('says the reason is missing when it is, instead of inventing one', () => {
    const line = eventToText(evt('SeatDispatchFailed', { dispatch_id: 'disp-abc' }), FULL);
    expect(line).toContain('没记原因');
  });

  it('says the title was not resolved when the dispatch is not in the map', () => {
    const line = eventToText(evt('SeatDispatchIssued', { dispatch_id: 'disp-unknown' }), FULL);
    expect(line).toContain('标题没取到');
    expect(line).toContain('disp-unknown');
  });
});

describe('acceptance lines say what is being verified', () => {
  it('AcceptanceRequested quotes the card scenario, which is already plain language', () => {
    const line = eventToText(
      evt('AcceptanceRequested', {
        dispatch_id: 'disp-abc',
        branch: 'packet/mira/knowledge-panel',
        acceptance_plan: {
          plan_version: 1,
          scenario: '你在 SeatLoom 里打开实时席位,它把各个席位接上。',
        },
      }),
      FULL,
    );
    expect(line).toContain('验收卡');
    expect(line).toContain('打开实时席位');
    expect(line).toContain('packet/mira/knowledge-panel');
  });

  it('AcceptanceGranted names the branch and a short merge commit', () => {
    const line = eventToText(
      evt(
        'AcceptanceGranted',
        {
          branch: 'track/infra-foundation',
          base: 'track/infra-foundation',
          merge_commit: '4cfd23023b4a94a65411b02d41cfe2f90c6b0335',
        },
        'human:supervisor',
      ),
      FULL,
    );
    expect(line).toContain('接受');
    expect(line).toContain('track/infra-foundation');
    expect(line).toContain('4cfd23023b4a');
  });

  it('AcceptanceGranted with no dispatch id says what it knows and guesses nothing', () => {
    // The granted payload is {branch, base, merge_commit} — no dispatch id. The
    // line must not derive one from the acceptance id's shape.
    const line = eventToText(
      evt(
        'AcceptanceGranted',
        { branch: 'track/infra-foundation', merge_commit: null },
        'human:supervisor',
      ),
      FULL,
    );
    expect(line).not.toContain('《');
    expect(line).toContain('没记是哪个活');
    expect(line).toContain('合进 track/infra-foundation');
  });

  it('AcceptanceRejected quotes the feedback and the rework dispatch', () => {
    const line = eventToText(
      evt(
        'AcceptanceRejected',
        {
          dispatch_id: 'disp-abc',
          feedback: '这些都没办法验收,我看不懂',
          rework_dispatch_id: 'disp-rework',
        },
        'human:supervisor',
      ),
      FULL,
    );
    expect(line).toContain('打回');
    expect(line).toContain('我看不懂');
    expect(line).toContain('disp-rework');
  });

  it('AcceptanceCancelled quotes the reason', () => {
    const line = eventToText(
      evt(
        'AcceptanceCancelled',
        { dispatch_id: 'disp-abc', reason: '旧卡作废重开' },
        'human:supervisor',
      ),
      FULL,
    );
    expect(line).toContain('作废');
    expect(line).toContain('旧卡作废重开');
  });
});

describe('memory lines carry the title, not just the id', () => {
  it('MemoryCreated shows the title, kind and topic', () => {
    const line = eventToText(
      evt('MemoryCreated', {
        memory_id: 'mem-1',
        title: '空数组的 bundle.icon 会静默失败',
        kind: 'failure',
        topic_id: 'topic-app-icon',
        confidence: 'quoted',
      }),
      FULL,
    );
    expect(line).toContain('《空数组的 bundle.icon 会静默失败》');
    expect(line).toContain('failure');
    expect(line).toContain('topic-app-icon');
  });

  it('MemoryRecalled resolves the title from the context', () => {
    const line = eventToText(
      evt('MemoryRecalled', {
        memory_id: 'mem-1',
        topic_id: 'topic-app-icon',
        heat: 4,
        heat_delta: 1,
      }),
      FULL,
    );
    expect(line).toContain('《空数组的 bundle.icon 会静默失败》');
    expect(line).toContain('+1');
  });

  it('MemoryRecalled says the title was not resolved when it was not', () => {
    const line = eventToText(evt('MemoryRecalled', { memory_id: 'mem-nope', heat_delta: 1 }), FULL);
    expect(line).toContain('标题没取到');
  });

  it('MemoryPromoted names both tiers', () => {
    const line = eventToText(
      evt('MemoryPromoted', {
        memory_id: 'mem-1',
        topic_id: 'topic-app-icon',
        from_tier: 'cold',
        to_tier: 'warm',
      }),
      FULL,
    );
    expect(line).toContain('cold');
    expect(line).toContain('warm');
  });

  it('MemoryForgotten quotes the reason', () => {
    const line = eventToText(
      evt('MemoryForgotten', {
        memory_id: 'mem-1',
        topic_id: 'topic-app-icon',
        reason: '指针指错了段',
      }),
      FULL,
    );
    expect(line).toContain('忘掉');
    expect(line).toContain('指针指错了段');
  });

  it('MemoryResolved names who closed it', () => {
    const line = eventToText(
      evt('MemoryResolved', {
        memory_id: 'mem-1',
        topic_id: 'topic-app-icon',
        resolved_by: 'seat:seat-mira-001',
      }),
      FULL,
    );
    expect(line).toContain('Mira');
  });

  it('TopicOpened shows the title the opener gave it', () => {
    const line = eventToText(
      evt('TopicOpened', {
        topic_id: 'topic-x',
        title: '实时席位镜像:什么能熬过一次切页签',
        parent_topic_id: null,
      }),
      FULL,
    );
    expect(line).toContain('《实时席位镜像:什么能熬过一次切页签》');
  });
});

describe("a real person's words are not touched", () => {
  it('SupervisorMessage passes the content through byte for byte', () => {
    const content =
      '张先生 2026-10-16 原话:「每次切换 menu,再切回实时席位,都需要重新 Connect 是为什么?」';
    expect(eventToText(evt('SupervisorMessage', { content }), FULL)).toBe(content);
  });

  it('SeatResponse passes the content through byte for byte', () => {
    const content = "BLOCKED (stayed blocked per this dispatch's own rule)";
    expect(eventToText(evt('SeatResponse', { content }), FULL)).toBe(content);
  });

  it('an empty message says so rather than rendering nothing', () => {
    expect(eventToText(evt('SupervisorMessage', {}), FULL)).toBe('[这条是空的]');
  });
});

describe('the emoji prefix style is preserved', () => {
  it('every translated line starts with a non-ASCII marker', () => {
    const lines = [
      eventToText(evt('SeatDispatchIssued', { dispatch_id: 'disp-abc' }), FULL),
      eventToText(evt('SeatDispatchCompleted', { dispatch_id: 'disp-done' }), FULL),
      eventToText(evt('MemoryCreated', { title: 'x' }), FULL),
      eventToText(evt('TopicOpened', { title: 'x' }), FULL),
      eventToText(evt('ArtifactCreated', { artifact_id: 'a' }), FULL),
    ];
    for (const l of lines) expect(l[0]).not.toBe('•');
    // The original style used an emoji per category; keep at least that.
    for (const l of lines) expect(/^[^\w\s]/.test(l)).toBe(true);
  });
});

describe('nothing is invented when the context is missing entirely', () => {
  it('renders honest lines with no resolver at all', () => {
    const lines = [
      eventToText(evt('SeatDispatchIssued', { dispatch_id: 'disp-abc' })),
      eventToText(evt('SeatDispatchDelivered', { dispatch_id: 'disp-abc' })),
      eventToText(evt('MemoryRecalled', { memory_id: 'mem-1' })),
      eventToText(evt('SeatDispatchFailed', { dispatch_id: 'disp-abc' })),
    ];
    for (const l of lines) {
      expect(l).not.toBe('');
      expect(l.startsWith('• ')).toBe(false);
    }
    // The seat falls back to its id, which is what it is — not a guessed name.
    expect(lines[1]).toContain('seat-mira-001');
    expect(lines[2]).toContain('标题没取到');
  });
});
