// The D2 acceptance card arrives over a cross-lane boundary whose wire spelling
// could not be read off Nimbus's Rust DTO (his commands were not on `track` yet).
// These tests pin the one thing that matters: both spellings the two lanes used
// normalise to the same card, and a missing field stays missing rather than
// becoming an invented value.

import { describe, expect, it } from 'vitest';
import {
  acceptanceStatusStyle,
  fluxVerdictStyle,
  isCancelled,
  isDecided,
  looksLikeMergeConflict,
  normaliseAcceptanceCard,
  normaliseAcceptanceCards,
} from './acceptance';

/** The dispatch that commissioned this lane spelled the fields snake_case. */
const snakeCard = {
  id: 'acc-1',
  dispatch_id: 'disp-7',
  dispatch_title: '重设计派单页交互',
  seat_id: 'seat-mira-001',
  branch: 'packet/mira/dispatch-ui',
  base: 'track/infra-foundation',
  how_to_verify: '打开派单页,待验收区出现一张卡片,点「接受并合并」后显示合并提交号。',
  flux_verdict: { ref_id: 'ref-flux-9', verdict: 'pass', occurred_at: '2026-10-09T10:00:00Z' },
  status: 'awaiting_acceptance',
  created_at: '2026-10-09T09:00:00Z',
};

/** The house convention: camelCase, per types-dto.ts. */
const camelCard = {
  id: 'acc-1',
  dispatchId: 'disp-7',
  dispatchTitle: '重设计派单页交互',
  seatId: 'seat-mira-001',
  branch: 'packet/mira/dispatch-ui',
  base: 'track/infra-foundation',
  howToVerify: '打开派单页,待验收区出现一张卡片,点「接受并合并」后显示合并提交号。',
  fluxVerdict: { refId: 'ref-flux-9', verdict: 'pass', occurredAt: '2026-10-09T10:00:00Z' },
  status: 'awaiting_acceptance',
  createdAt: '2026-10-09T09:00:00Z',
};

const EXPECTED = {
  id: 'acc-1',
  dispatchId: 'disp-7',
  dispatchTitle: '重设计派单页交互',
  seatId: 'seat-mira-001',
  branch: 'packet/mira/dispatch-ui',
  base: 'track/infra-foundation',
  howToVerify: '打开派单页,待验收区出现一张卡片,点「接受并合并」后显示合并提交号。',
  fluxVerdict: { refId: 'ref-flux-9', verdict: 'pass', occurredAt: '2026-10-09T10:00:00Z' },
  status: 'awaiting_acceptance',
  createdAt: '2026-10-09T09:00:00Z',
  mergeCommit: null,
  feedback: null,
  reworkDispatchId: null,
};

describe('normaliseAcceptanceCard', () => {
  it('accepts the snake_case spelling the commissioning dispatch used', () => {
    expect(normaliseAcceptanceCard(snakeCard)).toEqual(EXPECTED);
  });

  it('accepts the camelCase house convention', () => {
    expect(normaliseAcceptanceCard(camelCard)).toEqual(EXPECTED);
  });

  it('carries the decision fields the accept/reject commands populate', () => {
    const decided = normaliseAcceptanceCard({
      ...camelCard,
      status: 'accepted',
      mergeCommit: 'abc1234',
    });
    expect(decided?.status).toBe('accepted');
    expect(decided?.mergeCommit).toBe('abc1234');

    const reworked = normaliseAcceptanceCard({
      ...snakeCard,
      status: 'rejected',
      feedback: '状态列被截断',
      rework_dispatch_id: 'disp-8',
    });
    expect(reworked?.status).toBe('rejected');
    expect(reworked?.feedback).toBe('状态列被截断');
    expect(reworked?.reworkDispatchId).toBe('disp-8');
  });

  // The two shapes cmd_accept_delivery / cmd_reject_delivery actually return, as
  // widened by Nimbus's `57f7230` (status + reworkDispatchId). Both carry an
  // explicit null on the field the other path fills — the UI must render the
  // right result from that, and must not invent the missing half.
  const ACCEPT_RESULT = {
    id: 'acc-1',
    dispatchId: 'disp-7',
    dispatchTitle: '重设计派单页交互',
    seatId: 'seat-mira-001',
    branch: 'packet/mira/dispatch-ui',
    base: 'track/infra-foundation',
    howToVerify: '打开派单页,待验收区出现一张卡片。',
    fluxVerdict: { refId: 'ref-flux-9', verdict: 'pass', occurredAt: '2026-10-09T10:00:00Z' },
    createdAt: '2026-10-09T09:00:00Z',
  };

  it('renders the accepted result from the widened accept shape', () => {
    const c = normaliseAcceptanceCard({
      ...ACCEPT_RESULT,
      status: 'accepted',
      mergeCommit: '9f3c1ab',
      reworkDispatchId: null,
    });
    expect(c?.status).toBe('accepted');
    expect(c?.mergeCommit).toBe('9f3c1ab');
    // The reject half is explicitly null, and stays null — not invented.
    expect(c?.reworkDispatchId).toBeNull();
    expect(c?.feedback).toBeNull();
  });

  it('renders the rejected result from the widened reject shape', () => {
    const c = normaliseAcceptanceCard({
      ...ACCEPT_RESULT,
      status: 'rejected',
      mergeCommit: null,
      reworkDispatchId: 'disp-8',
      feedback: '状态列在 1280px 下被截断',
    });
    expect(c?.status).toBe('rejected');
    expect(c?.reworkDispatchId).toBe('disp-8');
    expect(c?.feedback).toBe('状态列在 1280px 下被截断');
    // The accept half is explicitly null — this is what used to dead-end.
    expect(c?.mergeCommit).toBeNull();
  });

  it('takes the same two shapes snake_cased', () => {
    const accepted = normaliseAcceptanceCard({
      ...snakeCard,
      status: 'accepted',
      merge_commit: '9f3c1ab',
      rework_dispatch_id: null,
    });
    expect(accepted?.mergeCommit).toBe('9f3c1ab');
    expect(accepted?.reworkDispatchId).toBeNull();

    const rejected = normaliseAcceptanceCard({
      ...snakeCard,
      status: 'rejected',
      merge_commit: null,
      rework_dispatch_id: 'disp-8',
      feedback: '状态列被截断',
    });
    expect(rejected?.reworkDispatchId).toBe('disp-8');
    expect(rejected?.mergeCommit).toBeNull();
  });

  // A decision the backend reports but does not enrich: the UI must say so
  // rather than rendering an empty success.
  it('reports a missing merge commit instead of rendering an empty success', () => {
    const c = normaliseAcceptanceCard({ ...ACCEPT_RESULT, status: 'accepted' });
    expect(c?.status).toBe('accepted');
    expect(c?.mergeCommit).toBeNull();
  });

  it('keeps a missing field missing instead of inventing one', () => {
    const bare = normaliseAcceptanceCard({ id: 'acc-2', status: 'awaiting_acceptance' });
    expect(bare).not.toBeNull();
    expect(bare?.howToVerify).toBe('');
    expect(bare?.dispatchTitle).toBe('');
    expect(bare?.mergeCommit).toBeNull();
    // An absent verdict must not read as a pass.
    expect(bare?.fluxVerdict).toEqual({ refId: '', verdict: '', occurredAt: '' });
  });

  it('returns null for anything that is not an object', () => {
    expect(normaliseAcceptanceCard(null)).toBeNull();
    expect(normaliseAcceptanceCard(undefined)).toBeNull();
    expect(normaliseAcceptanceCard('acc-1')).toBeNull();
    expect(normaliseAcceptanceCard(42)).toBeNull();
  });
});

describe('normaliseAcceptanceCards', () => {
  it('maps a list and drops non-objects', () => {
    const cards = normaliseAcceptanceCards([snakeCard, null, 'nope', camelCard]);
    expect(cards).toHaveLength(2);
    expect(cards[0]).toEqual(EXPECTED);
    expect(cards[1]).toEqual(EXPECTED);
  });

  it('returns an empty list for a non-array response', () => {
    expect(normaliseAcceptanceCards(null)).toEqual([]);
    expect(normaliseAcceptanceCards({})).toEqual([]);
  });
});

describe('presentation helpers', () => {
  // Four states, not two. `cancelled` is terminal — `cancel_acceptance` never
  // reopens a card — so treating it as undecided counted voided cards as waiting
  // on the operator. Mr. Zhang's 待验收 read 7 where 5 were real; the extra two
  // were voided days earlier.
  it('treats accepted, rejected AND cancelled as decided', () => {
    expect(isDecided('awaiting_acceptance')).toBe(false);
    expect(isDecided('pending')).toBe(false);
    expect(isDecided('accepted')).toBe(true);
    expect(isDecided('rejected')).toBe(true);
    expect(isDecided('cancelled')).toBe(true);
    // An unknown status is NOT decided — it must surface, not be silently settled.
    expect(isDecided('something-new')).toBe(false);
  });

  it('labels a voided card 已作废 rather than falling through to the raw status', () => {
    expect(acceptanceStatusStyle('cancelled').label).toBe('已作废');
    expect(acceptanceStatusStyle('cancelled').cls).not.toContain('emerald');
    expect(acceptanceStatusStyle('cancelled').cls).not.toContain('rose');
    expect(acceptanceStatusStyle('cancelled').cls).not.toContain('amber');
  });

  it('recognises a voided card without treating an unknown status as one', () => {
    expect(isCancelled('cancelled')).toBe(true);
    expect(isCancelled('accepted')).toBe(false);
    expect(isCancelled('pending')).toBe(false);
    expect(isCancelled('something-new')).toBe(false);
  });

  it('never paints an unknown Flux verdict green', () => {
    expect(fluxVerdictStyle('pass').cls).toContain('emerald');
    expect(fluxVerdictStyle('PASS').cls).toContain('emerald');
    expect(fluxVerdictStyle('fail').cls).toContain('rose');
    expect(fluxVerdictStyle('error').cls).not.toContain('emerald');
    // An absent verdict is the dangerous case: it must stay neutral.
    expect(fluxVerdictStyle('').cls).not.toContain('emerald');
    expect(fluxVerdictStyle('').label).toBe('Flux 未判');
  });

  it('labels known and unknown acceptance statuses', () => {
    expect(acceptanceStatusStyle('awaiting_acceptance').label).toBe('待验收');
    expect(acceptanceStatusStyle('accepted').label).toBe('已接受');
    expect(acceptanceStatusStyle('rejected').label).toBe('已打回');
    expect(acceptanceStatusStyle('mystery').label).toBe('mystery');
    expect(acceptanceStatusStyle('').label).toBe('未知状态');
  });

  it('recognises git conflict output so it can be shown verbatim', () => {
    expect(looksLikeMergeConflict('CONFLICT (content): Merge conflict in ui/src/App.tsx')).toBe(
      true,
    );
    expect(looksLikeMergeConflict('Automatic merge failed; fix conflicts and then commit')).toBe(
      true,
    );
    expect(looksLikeMergeConflict('error: commit is a merge but no -m given')).toBe(false);
    expect(looksLikeMergeConflict('')).toBe(false);
  });
});
