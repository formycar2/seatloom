// The conversation accumulator, against the four things it must not get wrong.
//
// The two that matter most are the counter-examples: an unconcluded turn is not
// silently dropped, and the terminal's bytes are never rewritten.

import { describe, expect, it } from 'vitest';
import {
  MAX_TURN_TEXT_BYTES,
  MAX_TURNS_KEPT,
  useConversationStore,
  whereIsItStored,
} from './useConversationStore';
import type { SessionTurnEvent } from '../lib/api';

const b64 = (s: string) => btoa(unescape(encodeURIComponent(s)));

const reset = () => useConversationStore.getState().reset();

describe('the conversation store accumulates a turn', () => {
  it('turnStarted then chunks accumulate into one turn', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('hello '), frame: false });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('world'), frame: true });
    const { turns, activeIndex } = useConversationStore.getState();
    expect(turns).toHaveLength(1);
    expect(turns[0].text).toBe('hello world');
    expect(turns[0].chunks).toBe(2);
    expect(turns[0].closed).toBe(false);
    expect(activeIndex).toBe(0);
  });

  it('turnEnded closes the turn and returns to idle', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('done'), frame: false });
    apply({ kind: 'turnEnded', sessionId: 's1' });
    const s = useConversationStore.getState();
    expect(s.turns[0].closed).toBe(true);
    expect(s.activeIndex).toBe(-1);
  });

  it('two turns are kept separately, in order', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('first'), frame: false });
    apply({ kind: 'turnEnded', sessionId: 's1' });
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('second'), frame: false });
    const { turns } = useConversationStore.getState();
    expect(turns.map((t) => t.text)).toEqual(['first', 'second']);
    expect(turns[0].closed).toBe(true);
    expect(turns[1].closed).toBe(false);
  });
});

describe('the counter-examples that make the above meaningful', () => {
  it('bytes with no open turn are KEPT, not dropped', () => {
    // If this failed, the UI would silently omit the start of a turn whose
    // boundary the heuristic missed — the same failure as a fabricated value.
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'chunk', sessionId: 's1', data: b64('orphan bytes'), frame: false });
    const { turns } = useConversationStore.getState();
    expect(turns).toHaveLength(1);
    expect(turns[0].text).toBe('orphan bytes');
  });

  it('a turnEnded with no open turn does nothing rather than inventing one', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnEnded', sessionId: 's1' });
    expect(useConversationStore.getState().turns).toHaveLength(0);
  });

  it('turnEnded closes only the LAST turn, never an already-closed one', () => {
    // A second turnEnded must not reach back and re-close an earlier turn: doing so
    // would make a closed turn look freshly finished, and a UI keyed on that would
    // re-render history as if it had just happened.
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('a'), frame: false });
    apply({ kind: 'turnEnded', sessionId: 's1' });
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('b'), frame: false });
    apply({ kind: 'turnEnded', sessionId: 's1' });
    apply({ kind: 'turnEnded', sessionId: 's1' }); // stray, must be a no-op
    const { turns, activeIndex } = useConversationStore.getState();
    expect(turns.map((t) => t.closed)).toEqual([true, true]);
    expect(activeIndex).toBe(-1);
    // The tell: the stray turnEnded must not have changed anything.
    expect(turns.map((t) => t.text)).toEqual(['a', 'b']);
    expect(turns.map((t) => t.chunks)).toEqual([1, 1]);
  });
});

describe('the caps engage only on abnormal output', () => {
  it('an oversized turn is trimmed from the FRONT and stays under the cap', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    const big = 'x'.repeat(64 * 1024);
    for (let i = 0; i < 8; i += 1) {
      apply({ kind: 'chunk', sessionId: 's1', data: b64(big), frame: false });
    }
    const { turns } = useConversationStore.getState();
    expect(turns).toHaveLength(1);
    // Exactly at the cap, not merely under it: `<=` alone is satisfied by a store
    // that never trims at all, which would pass this test while growing unbounded.
    expect(turns[0].text.length).toBe(MAX_TURN_TEXT_BYTES);
    // The newest bytes survive; the oldest are the ones dropped.
    expect(turns[0].text.endsWith('xxxx')).toBe(true);
    expect(turns[0].bytes).toBe(8 * 64 * 1024); // the byte count is NOT trimmed
  });

  it('a normal turn is never trimmed', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('a short answer'), frame: false });
    expect(useConversationStore.getState().turns[0].text).toBe('a short answer');
  });

  it('only MAX_TURNS_KEPT turns are retained', () => {
    reset();
    const { apply } = useConversationStore.getState();
    for (let i = 0; i < MAX_TURNS_KEPT + 10; i += 1) {
      apply({ kind: 'turnStarted', sessionId: 's1' });
      apply({ kind: 'chunk', sessionId: 's1', data: b64(`t${i}`), frame: false });
      apply({ kind: 'turnEnded', sessionId: 's1' });
    }
    const { turns, totalSeen } = useConversationStore.getState();
    expect(turns).toHaveLength(MAX_TURNS_KEPT);
    expect(totalSeen).toBe(MAX_TURNS_KEPT + 10);
    // The NEWEST turns are the ones kept.
    expect(turns[turns.length - 1].text).toBe(`t${MAX_TURNS_KEPT + 9}`);
  });

  it('reset clears everything', () => {
    reset();
    const { apply } = useConversationStore.getState();
    apply({ kind: 'turnStarted', sessionId: 's1' });
    apply({ kind: 'chunk', sessionId: 's1', data: b64('x'), frame: false });
    reset();
    const s = useConversationStore.getState();
    expect(s.turns).toHaveLength(0);
    expect(s.activeIndex).toBe(-1);
    expect(s.totalSeen).toBe(0);
  });
});

describe('where the accumulation lives is stated, not implied', () => {
  it('says it is memory-only, so nobody assumes a durable record', () => {
    // If this string ever changes to claim persistence, the claim must be true.
    expect(whereIsItStored).toContain('memory');
    expect(whereIsItStored).toContain('app session only');
  });
});
