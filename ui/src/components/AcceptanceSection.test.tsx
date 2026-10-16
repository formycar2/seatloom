// The D2 待验收 section, against the agreed card contract.
//
// Nimbus's commands are not on `track` yet, so the api layer is mocked at the
// module boundary exactly as the view consumes it. What is under test is this
// UI's own behaviour: the not-wired state, the card, the two decisions, the
// closed-loop results, and the verbatim-conflict rule. Nothing here stands in
// for a real backend value — the mock is the seam, not a fixture.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  registered: false,
  cards: [] as unknown[],
  accept: vi.fn(async () => ({})),
  reject: vi.fn(async () => ({})),
}));

// A getter, not a constant: the flag is the seam between "Nimbus's commands are
// on track" and "they are not", and the tests need to cross it both ways.
vi.mock('../lib/api', () => ({
  get ACCEPTANCE_CMDS_REGISTERED() {
    return state.registered;
  },
  api: {
    listAcceptances: async () => state.cards,
    acceptDelivery: (...a: unknown[]) => state.accept(...(a as [])),
    rejectDelivery: (...a: unknown[]) => state.reject(...(a as [])),
  },
}));

import AcceptanceSection from './AcceptanceSection';
import { ACCEPTANCE_CMDS_REGISTERED } from '../lib/api';

const seats: Record<string, string> = {
  'seat-mira-001': 'Mira',
  'seat-nimbus-001': 'Nimbus',
};
const seatName = (id: string) => seats[id] ?? id;

const card = (over: Record<string, unknown> = {}) => ({
  id: 'acc-1',
  dispatch_id: 'disp-7',
  dispatch_title: '重设计派单页交互',
  seat_id: 'seat-mira-001',
  branch: 'packet/mira/dispatch-ui',
  base: 'track/infra-foundation',
  how_to_verify: '打开派单页,待验收区出现一张卡片。',
  flux_verdict: { ref_id: 'ref-flux-9', verdict: 'pass', occurred_at: '2026-10-09T10:00:00Z' },
  status: 'awaiting_acceptance',
  created_at: '2026-10-09T09:00:00Z',
  ...over,
});

/** The same card in the house camelCase convention — what the wired backend
 *  actually sends once Nimbus's DTO lands. Both spellings are exercised so a
 *  wire-shape change cannot silently turn a card blank. */
const camelCard = (over: Record<string, unknown> = {}) => ({
  id: 'acc-1',
  dispatchId: 'disp-7',
  dispatchTitle: '重设计派单页交互',
  seatId: 'seat-mira-001',
  branch: 'packet/mira/dispatch-ui',
  base: 'track/infra-foundation',
  howToVerify: '打开派单页,待验收区出现一张卡片。',
  fluxVerdict: { refId: 'ref-flux-9', verdict: 'pass', occurredAt: '2026-10-09T10:00:00Z' },
  status: 'awaiting_acceptance',
  createdAt: '2026-10-09T09:00:00Z',
  ...over,
});

const renderSection = (onLocateDispatch = vi.fn()) => {
  const utils = render(
    <AcceptanceSection seatName={seatName} live onLocateDispatch={onLocateDispatch} />,
  );
  return { ...utils, onLocateDispatch };
};

describe('AcceptanceSection', () => {
  beforeEach(() => {
    cleanup();
    state.accept.mockClear();
    state.reject.mockClear();
    state.cards = [card()];
    // Default precondition: Nimbus's commands are NOT on track. A test that
    // exercises the wired section turns this on itself.
  });

  it('says out loud that the commands are not wired, instead of faking data', () => {
    // ACCEPTANCE_CMDS_REGISTERED is false on the mock above.
    expect(ACCEPTANCE_CMDS_REGISTERED).toBe(false);
    renderSection();
    expect(screen.getByText('待验收')).toBeTruthy();
    expect(screen.getByText(/cmd_list_acceptances/)).toBeTruthy();
    expect(screen.getByText(/不使用 mock 伪装后端真值/)).toBeTruthy();
    // No card, no actions — the section does not pretend to have data.
    expect(screen.queryByText('接受并合并')).toBeNull();
  });

  it('shows the acceptance criterion, the seat, the branch and the Flux verdict', async () => {
    state.registered = true;
    renderSection();
    await screen.findByText('重设计派单页交互');
    // how_to_verify is the decision, so it is on the card verbatim.
    expect(screen.getByText('打开派单页,待验收区出现一张卡片。')).toBeTruthy();
    expect(screen.getByText('Mira')).toBeTruthy();
    // branch → base live in one span with an arrow between them.
    expect(screen.getByText(/packet\/mira\/dispatch-ui/)).toBeTruthy();
    expect(screen.getByText(/track\/infra-foundation/)).toBeTruthy();
    expect(screen.getByText('Flux 验绿')).toBeTruthy();
    expect(screen.getByText('ref-flux-9')).toBeTruthy();
  });

  it('requires a reason before 打回, then sends it', async () => {
    state.registered = true;
    renderSection();
    await screen.findByText('重设计派单页交互');

    const reject = screen.getByRole('button', { name: /打回/ }) as HTMLButtonElement;
    expect(reject.disabled).toBe(true);

    fireEvent.change(screen.getByPlaceholderText(/状态列被截断/), {
      target: { value: '状态列在 1280px 下被截断' },
    });
    expect(reject.disabled).toBe(false);

    fireEvent.click(reject);
    await waitFor(() =>
      expect(state.reject).toHaveBeenCalledWith('acc-1', '状态列在 1280px 下被截断'),
    );
  });

  it('accepting shows the merge commit that closed the loop', async () => {
    state.registered = true;
    state.accept.mockResolvedValueOnce(card({ status: 'accepted', merge_commit: 'abc1234' }));
    renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.click(screen.getByRole('button', { name: /接受并合并/ }));
    await waitFor(() => expect(screen.getByText('已接受并合并')).toBeTruthy());
    expect(screen.getByText('abc1234')).toBeTruthy();
    // Called with the card's acceptance id — nothing else.
    expect(state.accept).toHaveBeenCalledWith('acc-1');
    expect(state.accept).toHaveBeenCalledTimes(1);
  });

  // The widened shape Nimbus landed in 57f7230: status + reworkDispatchId, with
  // an explicit null on the accept half. This is what closed the dead-end where
  // a rejection left the operator nowhere to go.
  it('rejecting renders the widened result: badge, feedback echo and jump link', async () => {
    state.registered = true;
    state.reject.mockResolvedValueOnce({
      ...camelCard(),
      status: 'rejected',
      mergeCommit: null,
      reworkDispatchId: 'disp-8',
      feedback: '状态列在 1280px 下被截断',
    });
    const { onLocateDispatch } = renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.change(screen.getByPlaceholderText(/状态列被截断/), {
      target: { value: '状态列在 1280px 下被截断' },
    });
    fireEvent.click(screen.getByRole('button', { name: /打回/ }));
    await waitFor(() => expect(screen.getByText('已打回')).toBeTruthy());

    // 1. the feedback the operator typed is echoed back, so the rework order is
    //    legible on the card itself.
    expect(screen.getByText('「状态列在 1280px 下被截断」')).toBeTruthy();
    // 2. no dead-end: the rework dispatch is a live link into the inbox above.
    const link = screen.getByRole('button', { name: /新派单 disp-8/ });
    fireEvent.click(link);
    expect(onLocateDispatch).toHaveBeenCalledWith('disp-8');
    expect(state.reject).toHaveBeenCalledWith('acc-1', '状态列在 1280px 下被截断');
  });

  it('accepting with no merge commit says so instead of showing an empty success', async () => {
    state.registered = true;
    state.accept.mockResolvedValueOnce({ ...camelCard(), status: 'accepted' });
    renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.click(screen.getByRole('button', { name: /接受并合并/ }));
    await waitFor(() => expect(screen.getByText('已接受并合并')).toBeTruthy());
    expect(screen.getByText('未返回合并提交号。')).toBeTruthy();
  });

  // 空反馈不可提交 is an invariant, not a styling detail: clearing the field
  // re-arms the guard, so a stray click cannot enqueue a rework order the seat
  // cannot act on.
  it('will not submit a reject with empty feedback, even after text was cleared', async () => {
    state.registered = true;
    renderSection();
    await screen.findByText('重设计派单页交互');

    const field = screen.getByPlaceholderText(/状态列被截断/);
    fireEvent.change(field, { target: { value: '状态列被截断' } });
    const reject = screen.getByRole('button', { name: /打回/ }) as HTMLButtonElement;
    expect(reject.disabled).toBe(false);

    fireEvent.change(field, { target: { value: '   ' } });
    expect(reject.disabled).toBe(true);
    fireEvent.click(reject);
    expect(state.reject).not.toHaveBeenCalled();

    fireEvent.change(field, { target: { value: '' } });
    expect(reject.disabled).toBe(true);
    fireEvent.click(reject);
    expect(state.reject).not.toHaveBeenCalled();
  });

  it('rejecting links the rework dispatch back into the inbox', async () => {
    state.registered = true;
    state.reject.mockResolvedValueOnce(
      card({ status: 'rejected', feedback: '状态列被截断', rework_dispatch_id: 'disp-8' }),
    );
    const { onLocateDispatch } = renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.change(screen.getByPlaceholderText(/状态列被截断/), {
      target: { value: '状态列被截断' },
    });
    fireEvent.click(screen.getByRole('button', { name: /打回/ }));
    await waitFor(() => expect(screen.getByText('已打回')).toBeTruthy());

    const link = screen.getByRole('button', { name: /新派单 disp-8/ });
    fireEvent.click(link);
    expect(onLocateDispatch).toHaveBeenCalledWith('disp-8');
  });

  it('shows a merge conflict verbatim instead of swallowing it', async () => {
    state.registered = true;
    state.accept.mockRejectedValueOnce(
      new Error(
        'Automatic merge failed; fix conflicts and then commit the result.\nCONFLICT (content): Merge conflict in ui/src/App.tsx',
      ),
    );
    renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.click(screen.getByRole('button', { name: /接受并合并/ }));
    await waitFor(() => expect(screen.getByText(/合并冲突/)).toBeTruthy());
    // The operator gets git's own output, hunks included.
    expect(
      screen.getByText(/CONFLICT \(content\): Merge conflict in ui\/src\/App\.tsx/),
    ).toBeTruthy();
    // And the decision is still available — a conflict is not a verdict.
    expect(screen.getByRole('button', { name: /接受并合并/ })).toBeTruthy();
  });

  it('keeps the result readable until the operator closes it', async () => {
    state.registered = true;
    state.accept.mockResolvedValueOnce(card({ status: 'accepted', merge_commit: 'abc1234' }));
    renderSection();
    await screen.findByText('重设计派单页交互');

    fireEvent.click(screen.getByRole('button', { name: /接受并合并/ }));
    // The merge commit must not be a flash: no immediate reload drops the card.
    await waitFor(() => expect(screen.getByText('已接受并合并')).toBeTruthy());
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.getByText('abc1234')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: '关闭这条结果' }));
    await waitFor(() => expect(screen.queryByText('已接受并合并')).toBeNull());
  });

  // A card must be legible before it is actionable: the criterion, the verdict,
  // the lineage, and when it arrived. A card that renders none of that is not a
  // card, however correct its buttons are.
  it('renders the card legibly: criterion, verdict, lineage and time', async () => {
    state.registered = true;
    state.cards = [camelCard()];
    renderSection();
    await screen.findByText('重设计派单页交互');

    // how_to_verify — the sentence the decision is made on.
    expect(screen.getByText('打开派单页,待验收区出现一张卡片。')).toBeTruthy();
    // Flux's technical verdict, demoted to context.
    expect(screen.getByText('Flux 验绿')).toBeTruthy();
    // Lineage: seat, branch → base, dispatch id.
    expect(screen.getByText('Mira')).toBeTruthy();
    expect(screen.getByText(/packet\/mira\/dispatch-ui/)).toBeTruthy();
    expect(screen.getByText(/track\/infra-foundation/)).toBeTruthy();
    expect(screen.getByText('disp-7')).toBeTruthy();
    // When it arrived — so a stale card is visible as stale. Computed the same
    // way the component formats it, so the assertion is not timezone-fragile.
    const at = new Date('2026-10-09T09:00:00Z');
    const p2 = (n: number) => n.toString().padStart(2, '0');
    const expectedAt = `${at.getMonth() + 1}/${at.getDate()} ${p2(at.getHours())}:${p2(at.getMinutes())}`;
    expect(screen.getByText(expectedAt)).toBeTruthy();
    // And it is still awaiting a decision, not silently decided. 「待验收」
    // appears twice on purpose: the section heading and the card's own status
    // pill — the pill is what proves THIS card is undecided.
    expect(screen.getAllByText('待验收')).toHaveLength(2);
  });

  it('says so when the backend sends no how_to_verify', async () => {
    state.registered = true;
    state.cards = [camelCard({ howToVerify: '' })];
    renderSection();
    await screen.findByText('重设计派单页交互');
    expect(screen.getByText('该交付没有填写验证方式。')).toBeTruthy();
  });

  it('has an empty state that explains what lands here', async () => {
    state.registered = true;
    state.cards = [];
    renderSection();
    expect(await screen.findByText('没有待验收的交付')).toBeTruthy();
    expect(screen.getByText(/席位交付并通过 Flux 技术验证后/)).toBeTruthy();
  });

  it('stops polling when the operator pauses the page', async () => {
    state.registered = true;
    render(<AcceptanceSection seatName={seatName} live={false} onLocateDispatch={vi.fn()} />);
    // live=false: no poll is armed, so nothing is fetched and no card appears.
    await waitFor(() => expect(screen.queryByText('重设计派单页交互')).toBeNull());
  });
  // ── The voided-card defect (2026-10-16) ────────────────────────────────────
  //
  // Mr. Zhang counted 7 待决策 where 5 were actually waiting on him. The extra
  // two were cards that had been voided days earlier: `isDecided` did not know
  // `cancelled` was a settled state, so they stayed in the awaiting list AND in
  // the number. The live card set at the time of the fix is 10 cards — 6
  // accepted, 2 cancelled, 2 pending — which with the bug renders as 4 项待决策
  // instead of 2.

  it('does not count a voided card as waiting on the operator', async () => {
    state.registered = true;
    state.cards = [
      card({ id: 'acc-live', dispatch_title: '活一' }),
      card({ id: 'acc-live-2', dispatch_title: '活二' }),
      card({
        id: 'acc-void',
        dispatch_title: '作废的那张',
        status: 'cancelled',
        feedback: 'base 记成不存在的 track;内容早已由 Aegis 合并,按裁决作废。',
      }),
    ];
    renderSection();
    await screen.findByText('活一');
    // Two are waiting, not three.
    expect(screen.getByText('2 项待决策')).toBeTruthy();
    expect(screen.queryByText('3 项待决策')).toBeNull();
    // The voided card is not in the actionable list at all — not even collapsed,
    // because the 已作废 area only opens on a click.
    expect(screen.queryByText('作废的那张')).toBeNull();
  });

  it('keeps a voided card findable, collapsed, with its reason', async () => {
    state.registered = true;
    state.cards = [
      card({ id: 'acc-live', dispatch_title: '活一' }),
      card({
        id: 'acc-void',
        dispatch_title: '作废的那张',
        status: 'cancelled',
        feedback: '原卡的验收描述满是技术术语,张先生看不懂,所以用大白话重开一张。',
      }),
    ];
    renderSection();
    await screen.findByText('活一');

    // Collapsed by default: it is a record, not work.
    const toggle = screen.getByText('已作废 1 项');
    expect(screen.queryByText('作废的那张')).toBeNull();

    fireEvent.click(toggle);
    // Open: the title and the reason are both readable.
    expect(screen.getByText('作废的那张')).toBeTruthy();
    expect(screen.getByText(/原卡的验收描述满是技术术语/)).toBeTruthy();
    expect(screen.getByText(/不算在待决策里/)).toBeTruthy();
  });

  it('says the reason lives in the ledger when the card carries none', async () => {
    state.registered = true;
    state.cards = [
      card({ id: 'acc-live', dispatch_title: '活一' }),
      card({
        id: 'acc-void',
        dispatch_title: '没写理由的那张',
        status: 'cancelled',
        feedback: null,
      }),
    ];
    renderSection();
    await screen.findByText('活一');
    fireEvent.click(screen.getByText('已作废 1 项'));
    expect(screen.getByText(/没记作废理由/)).toBeTruthy();
    expect(screen.getByText(/AcceptanceCancelled/)).toBeTruthy();
  });

  it('leaves already-accepted and already-rejected cards out of the count, as before', async () => {
    // Unchanged behaviour, asserted so the cancelled fix cannot quietly widen:
    // a settled card has never been rendered by this section — only a card the
    // operator decides in this session shows its result (covered by the
    // accept/reject tests above).
    state.registered = true;
    state.cards = [
      card({ id: 'acc-live', dispatch_title: '活一' }),
      card({
        id: 'acc-acc',
        dispatch_title: '已接受的那张',
        status: 'accepted',
        merge_commit: 'abc123',
      }),
      card({
        id: 'acc-rej',
        dispatch_title: '已打回的那张',
        status: 'rejected',
        rework_dispatch_id: 'disp-rework',
      }),
      card({ id: 'acc-void', dispatch_title: '作废的那张', status: 'cancelled', feedback: '撤了' }),
    ];
    renderSection();
    await screen.findByText('活一');
    expect(screen.getByText('1 项待决策')).toBeTruthy();
    // Only the voided one gets a 已作废 area — settled cards are simply absent.
    expect(screen.getByText('已作废 1 项')).toBeTruthy();
    fireEvent.click(screen.getByText('已作废 1 项'));
    expect(screen.getByText('作废的那张')).toBeTruthy();
    expect(screen.queryByText('已接受的那张')).toBeNull();
    expect(screen.queryByText('已打回的那张')).toBeNull();
  });
});
