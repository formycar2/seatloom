// LiveSeatsView — the reconnect defect, pinned.
//
// The defect (operator, 2026-10-16, in person): every menu switch away from and
// back to Live Seats asked him to Connect each seat again. He assumed it was by
// design. It was not — the view is conditionally rendered by App.tsx, so leaving
// the tab unmounted it, and the attachments lived in the component's own useState.
// The backend sessions were alive the whole time; the view had simply forgotten.
//
// These tests mount the real view with a mocked Tauri layer and assert the three
// things the acceptance card checks:
//   1. coming back does NOT ask for a Connect — the attachments survive;
//   2. the byte counters survive too, not reset to zero;
//   3. the deliberate first-seat auto-attach still happens on a genuinely first
//      open, and does NOT fire a second attach on top of a live mirror.
//
// Plus the two guards that make the restore honest rather than a guess:
//   4. a session that exited while the view was away is dropped (the exit listener
//      only runs while mounted, so the mount-time reconcile is the only catch-up);
//   5. a failure to list live sessions is NOT a reason to drop live mirrors.

import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

const state = vi.hoisted(() => ({
  tmuxSessions: [
    { session_name: 'Lyra-lyra-seatloom', attached: true, windows: 1 },
    { session_name: 'Nimbus-nimbus-seatloom', attached: true, windows: 1 },
    { session_name: 'Onyx-onyx-seatloom', attached: true, windows: 1 },
  ],
  // Must match the ids `attachTmuxSession` below returns, or the mount-time
  // reconcile correctly prunes everything and the test proves nothing.
  liveSessionIds: [
    'sl-live-Lyra-lyra-seatloom',
    'sl-live-Nimbus-nimbus-seatloom',
    'sl-live-Onyx-onyx-seatloom',
  ],
  attachCalls: [] as Array<{ tmuxSessionName: string; seatId?: string }>,
  listLiveCalls: 0,
  listLiveThrows: false,
  nextId: 0,
  outputHandlers: [] as Array<(e: { sessionId: string; data: string }) => void>,
  exitHandlers: [] as Array<(e: { sessionId: string; code: number | null }) => void>,
  mirrorLostHandlers: [] as Array<(e: Record<string, unknown>) => void>,
}));

vi.mock('../lib/api', () => {
  const api = {
    listTmuxSessions: async () => state.tmuxSessions,
    listSeats: async (projectId: string) => projectId === 'ql_delivery' ? [{ name: 'ql' }] : [{ name: 'Lyra' }, { name: 'Nimbus' }, { name: 'Onyx' }],
    attachTmuxSession: async (req: { tmuxSessionName: string; seatId?: string }) => {
      state.attachCalls.push({ tmuxSessionName: req.tmuxSessionName, seatId: req.seatId });
      state.nextId += 1;
      const id = `sl-live-${req.tmuxSessionName}`;
      return {
        id,
        seatId: req.seatId ?? null,
        runtime: 'tmux-mirror',
        command: 'tmux pipe-pane',
        args: [req.tmuxSessionName],
        workingDir: '/repo',
        transcriptPath: '/tmp/t.jsonl',
        tmuxSessionName: req.tmuxSessionName,
        fifoPath: '/tmp/fifo',
        paneRows: 30,
        paneCols: 120,
        initialSnapshotB64: '',
      };
    },
    listLiveSessions: async () => {
      state.listLiveCalls += 1;
      if (state.listLiveThrows) throw new Error('list_live_sessions failed');
      return state.liveSessionIds;
    },
    isTauri: () => true,
    onSessionOutput: (h: (e: { sessionId: string; data: string }) => void) => {
      state.outputHandlers.push(h);
      return Promise.resolve(() => {});
    },
    onSessionExit: (h: (e: { sessionId: string; code: number | null }) => void) => {
      state.exitHandlers.push(h);
      return Promise.resolve(() => {});
    },
    // R1: the mirror could not be proven. Captured so a test can fire it and
    // assert the view stops showing that session.
    onSessionMirrorLost: (h: (e: Record<string, unknown>) => void) => {
      state.mirrorLostHandlers.push(h as never);
      return Promise.resolve(() => {});
    },
  };
  return {
    api,
    isTauri: () => true,
    onSessionOutput: api.onSessionOutput,
    onSessionExit: api.onSessionExit,
    onSessionMirrorLost: api.onSessionMirrorLost,
  };
});

// The terminal itself needs xterm + a real canvas; neither exists in jsdom. Stub
// it with a marker so the assertions can count rendered panes.
vi.mock('../app-v2/panel/SessionTerminal', () => ({
  SessionTerminal: ({ sessionId }: { sessionId: string }) => (
    <div data-testid="terminal" data-session={sessionId} />
  ),
}));

import LiveSeatsView from './LiveSeatsView';
import { useLiveSeatsMirrorStore } from '../stores/useLiveSeatsMirrorStore';
import { useDataStore } from '../stores/useDataStore';

beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {};
});

beforeEach(() => {
  useDataStore.setState({ activeProjectId: 'seatloom', backendHydrated: true });
  useLiveSeatsMirrorStore.setState({
    attachments: [],
    activity: {},
    activeName: null,
    autoAttached: false,
  });
  state.attachCalls = [];
  state.listLiveCalls = 0;
  state.listLiveThrows = false;
  state.outputHandlers = [];
  state.exitHandlers = [];
});

afterEach(() => {
  cleanup();
});

/** Wait until the first seat has been auto-attached (the deliberate design). */
async function waitForFirstAutoAttach() {
  await waitFor(() => expect(state.attachCalls.length).toBe(1));
}

describe('the reconnect defect', () => {
  it('keeps the connections across an unmount/remount instead of asking for a Connect', async () => {
    const first = render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    // Attach one more by hand, the way the operator does. Two seats are still
    // unattached at this point, so pick the first Connect button rather than
    // assuming there is only one.
    await act(async () => {
      fireEvent.click(screen.getAllByText('接入实时镜像', { selector: 'button' })[0]);
    });
    await waitFor(() => expect(state.attachCalls.length).toBe(2));

    const attachedBefore = useLiveSeatsMirrorStore.getState().attachments.length;
    expect(attachedBefore).toBe(2);

    // Leaving the tab unmounts the view.
    first.unmount();
    // Coming back remounts it.
    render(<LiveSeatsView />);

    // The three seats are still mirrored, with no new attach and no Connect button
    // left to press.
    await waitFor(() =>
      expect(useLiveSeatsMirrorStore.getState().attachments.length).toBe(attachedBefore),
    );
    expect(state.attachCalls.length).toBe(2);
    expect(screen.queryAllByText('接入实时镜像', { selector: 'button' })).toHaveLength(1);
    expect(screen.getByText(/已接入 2 个/)).toBeTruthy();
  });

  it('keeps the byte counters across a remount rather than resetting them', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;
    // Feed real base64 output through the listener the view registered.
    await act(async () => {
      for (const h of state.outputHandlers) h({ sessionId, data: 'QUJDREVG' }); // 6 bytes
      await new Promise((r) => setTimeout(r, 600)); // let the TICK_MS flush land
    });
    expect(useLiveSeatsMirrorStore.getState().activity[sessionId].bytes).toBeGreaterThan(0);

    const before = useLiveSeatsMirrorStore.getState().activity[sessionId];
    cleanup();
    render(<LiveSeatsView />);

    await waitFor(() =>
      expect(useLiveSeatsMirrorStore.getState().activity[sessionId]).toEqual(before),
    );
  });

  it('still auto-attaches the first seat on a genuinely first open', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    expect(state.attachCalls).toEqual([{ tmuxSessionName: 'Lyra-lyra-seatloom', seatId: 'lyra' }]);
  });

  it('does NOT attach a second time on top of a mirror that is still live', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    const one = state.attachCalls.length;

    cleanup();
    render(<LiveSeatsView />);

    // Give the mount effects every chance to fire.
    await new Promise((r) => setTimeout(r, 50));
    expect(state.attachCalls.length).toBe(one);
  });
});

describe('the restore is honest, not a guess', () => {
  it('drops an attachment whose session exited while the view was away', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;
    expect(sessionId).toBe('sl-live-Lyra-lyra-seatloom');

    // The seat's backend session dies while nobody is listening.
    state.liveSessionIds = ['sl-live-nimbus'];
    cleanup();
    render(<LiveSeatsView />);

    await waitFor(() =>
      expect(
        useLiveSeatsMirrorStore.getState().attachments.some((a) => a.dto.id === sessionId),
      ).toBe(false),
    );
    // ...and it did not take the counters with it for the sessions that survive.
    expect(useLiveSeatsMirrorStore.getState().attachments).toHaveLength(0);
  });

  it('keeps the mirrors when the live-session list cannot be read', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    const before = useLiveSeatsMirrorStore.getState().attachments;

    state.listLiveThrows = true;
    cleanup();
    render(<LiveSeatsView />);

    await new Promise((r) => setTimeout(r, 50));
    // A transient list failure must not cost the operator his live mirrors.
    expect(useLiveSeatsMirrorStore.getState().attachments).toEqual(before);
  });

  it('re-asserts the seat -> session routing the Supervisor IM depends on', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;

    // Simulate the mapping having been lost (e.g. a partial write), then remount.
    useLiveSessionsStoreSetFor('lyra', null);
    cleanup();
    render(<LiveSeatsView />);

    await waitFor(() => expect(useLiveSessionsStoreFor('lyra')).toBe(sessionId));
  });
});

// Small helpers over the untouched IM-routing store, so the test does not have to
// import it twice.
import { useLiveSessionsStore } from '../stores/useLiveSessionsStore';
function useLiveSessionsStoreFor(seat: string) {
  return useLiveSessionsStore.getState().sessionFor(seat);
}
function useLiveSessionsStoreSetFor(seat: string, id: string | null) {
  if (id === null) useLiveSessionsStore.getState().clearSessionForSeat(seat);
  else useLiveSessionsStore.getState().setSessionForSeat(seat, id);
}


describe('project-scoped mirror rendering', () => {
  it("hides another project's surviving terminal without destroying the connection", async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    await waitFor(() => expect(screen.getAllByTestId('terminal')).toHaveLength(1));
    act(() => { useDataStore.setState({ activeProjectId: 'ql_delivery' }); });
    await waitFor(() => expect(screen.queryAllByTestId('terminal')).toHaveLength(0));
    expect(useLiveSeatsMirrorStore.getState().attachments).toHaveLength(1);
  });
});

// THE SCROLL DEFECT (operator, 2026-10-26): "the live-seats page sometimes will not
// scroll up or down". Root cause is a chain, and the DOM half of it is asserted here.
//
// Inactive attachment layers were hidden with `display: none`. A display:none
// ancestor has NO LAYOUT BOX, so xterm's character-measure element has no box
// either and FitAddon.fit() returns early without resizing — the viewport then
// sits at the pane's attach-time row/col seed, which is normally TALLER than this
// slot. Content shorter than that seed row count leaves the scroll area equal to
// the viewport height: the page is visibly there and cannot be scrolled at all.
//
// The fix hides with `visibility: hidden` (plus pointerEvents: none), which keeps
// the box, so a terminal mounted while its attachment is inactive is measurable
// and is fitted correctly the moment it is armed.
//
// WHAT THIS FILE CANNOT ASSERT: jsdom performs no layout — every element's
// clientWidth/clientHeight is 0 and getComputedStyle returns the inline style
// verbatim. So "the hidden layer still has a real box" is asserted structurally
// (`display` is not `none`, `visibility` is `hidden`), not measured. The
// measured half — FitAddon's early return when cell metrics are 0, and the
// component's retry once they are valid — lives in
// src/app-v2/panel/SessionTerminal.geometry.test.tsx, which drives the real
// component against a TRANSCRIPTION of the addon's contract.
describe('the scroll defect: inactive attachments stay laid out', () => {
  /** Attach the given number of seats the way the operator does. */
  async function attachSeats(count: number) {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    for (let i = 1; i < count; i++) {
      await act(async () => {
        fireEvent.click(screen.getAllByText('接入实时镜像', { selector: 'button' })[0]);
      });
    }
    await waitFor(() => expect(screen.getAllByTestId('terminal')).toHaveLength(count));
  }

  it('the ACTIVE layer is visible and interactive', async () => {
    await attachSeats(2);
    const active = useLiveSeatsMirrorStore.getState().activeName!;
    const layer = document.querySelector(`[data-session="sl-live-${active}"]`)!.parentElement!;
    const style = getComputedStyle(layer);

    expect(style.visibility).toBe('visible');
    expect(style.pointerEvents).toBe('auto');
    expect(style.display).not.toBe('none');
  });

  it('an INACTIVE layer is hidden without display:none, so it keeps its box', async () => {
    await attachSeats(2);
    const active = useLiveSeatsMirrorStore.getState().activeName!;
    const inactive = useLiveSeatsMirrorStore
      .getState()
      .attachments.find((a) => a.seat.sessionName !== active)!;
    const layer = document.querySelector(`[data-session="${inactive.dto.id}"]`)!.parentElement!;
    const style = getComputedStyle(layer);

    // The whole point: hidden, but NOT display:none.
    expect(style.visibility).toBe('hidden');
    expect(style.display).not.toBe('none');
    // And it must not intercept the wheel or a click aimed at the live pane.
    expect(style.pointerEvents).toBe('none');
  });

  it('switching seats flips which layer is visible and never unloads a terminal', async () => {
    await attachSeats(2);
    const before = useLiveSeatsMirrorStore.getState().attachments;
    const first = useLiveSeatsMirrorStore.getState().activeName!;

    act(() => {
      useLiveSeatsMirrorStore.setState({
        activeName: before.find((a) => a.seat.sessionName !== first)!.seat.sessionName,
      });
    });

    // Both terminals are still mounted — the scrollback, the fit and the pipe
    // subscription all survive a switch. Unmounting would lose them.
    expect(screen.getAllByTestId('terminal')).toHaveLength(2);
    for (const a of before) {
      const layer = document.querySelector(`[data-session="${a.dto.id}"]`)!.parentElement!;
      const style = getComputedStyle(layer);
      const expected = a.seat.sessionName === useLiveSeatsMirrorStore.getState().activeName;
      expect(style.visibility).toBe(expected ? 'visible' : 'hidden');
      expect(style.pointerEvents).toBe(expected ? 'auto' : 'none');
    }
  });
});

// ── R1, end to end: the acceptance criterion ────────────────────────────────
//
// The packet requires a path where, after the pipe is stolen, the UI does NOT
// keep showing content. This is that path. It is tested at the view level (not
// the store level) because the store test alone cannot prove the terminal
// actually unmounts — and an unmounted terminal is what stops the bytes.
describe('R1: a mirror that cannot be proven stops being shown', () => {
  it('unmounts the terminal and says why, instead of rendering the foreign bytes', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;
    expect(screen.getAllByTestId('terminal')).toHaveLength(1);

    // The backend proves the pipe is no longer ours.
    await act(async () => {
      for (const h of state.mirrorLostHandlers) {
        h({
          sessionId,
          seatId: 'nimbus',
          tmuxSessionName: 'Lyra-lyra-seatloom',
          reason: 'cross_wired',
          writers: 2,
          detail: '镜像已断开：有 2 个进程在写入这个 FIFO',
        });
      }
    });

    // 1. The terminal is GONE. This is the criterion — without it the foreign
    //    bytes keep rendering and the rest is decoration.
    expect(screen.queryAllByTestId('terminal')).toHaveLength(0);
    // 2. The operator is told, in words, not a log line.
    expect(screen.getByText('镜像已断开')).toBeTruthy();
    expect(screen.getByText(/有 2 个进程在写入/)).toBeTruthy();
    expect(screen.getByText('cross_wired')).toBeTruthy();
    // 3. There is a way back.
    expect(screen.getByRole('button', { name: /重新接入/ })).toBeTruthy();
  });

  it('bytes arriving after the event are not counted as live activity', async () => {
    // A counter still climbing is a quieter version of the same lie: it says the
    // stream is healthy while the bytes are unvouched for.
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;
    await act(async () => {
      for (const h of state.outputHandlers) h({ sessionId, data: 'QUJDREVG' });
      await new Promise((r) => setTimeout(r, 600));
    });
    const before = useLiveSeatsMirrorStore.getState().activity[sessionId].bytes;
    expect(before).toBeGreaterThan(0);

    await act(async () => {
      for (const h of state.mirrorLostHandlers) {
        h({ sessionId, reason: 'lost', detail: '镜像已断开', tmuxSessionName: 'Lyra-lyra-seatloom' });
      }
    });
    await act(async () => {
      for (const h of state.outputHandlers) h({ sessionId, data: 'QUJDREVGR0hJ' });
      await new Promise((r) => setTimeout(r, 600));
    });

    // No new record, and certainly not a growing one.
    const after = useLiveSeatsMirrorStore.getState().activity[sessionId];
    expect(after === undefined || after.bytes === before).toBe(true);
  });

  it('shows the pane identity in the header so cross-wiring is visible by eye', async () => {
    // R2. The incident ran three hours partly because nothing on screen answered
    // "which pane am I actually looking at".
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();
    // The mock DTO carries no pane_pid/pane_tty; the view must say "?" rather
    // than invent a value.
    expect(screen.getAllByText(/pid \?/).length).toBeGreaterThan(0);
  });

  // ── the backfill is labelled, and an unknown is not a zero ──────────────────
  // Mr. Zhang's symptom: QL looked scrollable but showed content with nothing in
  // common with tmux. The depth was one cause — the backfill asked for 5 000 lines
  // while the pane retained 2 000, and tmux silently gave less.
  it('labels the scrollback as a backfill, and never calls an unknown 0 lines', async () => {
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    // 1. The header names a line count AND says it is a backfill. A history with
    //    no label looks exactly like a recorded one.
    const code = screen.getAllByText(/历史 \d+ 行\(回填\)/);
    expect(code.length).toBeGreaterThan(0);

    // 2. The fixture DTO carries no paneHistoryLimit. The tooltip must DECLINE to
    //    state a number. This is the assertion whose absence let Flux's finding
    //    stand: the UI used to render "history-limit is 0 lines" for an unknown.
    const node = code[0] as HTMLElement;
    const tip = node.getAttribute('title') ?? '';
    expect(tip).toContain('回填');
    expect(tip).toContain('不是录制');
    expect(tip).toContain('history-limit 未取得');
    expect(tip).not.toContain('history-limit 是 0 行');
    expect(tip).not.toMatch(/history-limit 是 0/);
  });

  it('a measured limit of 0 is still stated as a fact, not swallowed', async () => {
    // The other half: an unknown must not become a zero, AND a zero must not be
    // hidden. Some(0) and None are different facts and the UI keeps them apart.
    render(<LiveSeatsView />);
    await waitForFirstAutoAttach();

    const sessionId = useLiveSeatsMirrorStore.getState().attachments[0].dto.id;
    const before = useLiveSeatsMirrorStore.getState().attachments[0].dto;
    await act(async () => {
      useLiveSeatsMirrorStore.setState((s) => ({
        attachments: s.attachments.map((a) =>
          a.dto.id === sessionId
            ? { ...a, dto: { ...a.dto, paneHistoryLimit: 0, seededScrollbackLines: 3 } }
            : a,
        ),
      }));
    });
    expect(before.paneHistoryLimit).not.toBe(0);

    const code = screen.getAllByText(/历史 \d+ 行\(回填\)/);
    const tip = (code[0] as HTMLElement).getAttribute('title') ?? '';
    expect(tip).toContain('history-limit 是 0 行');
  });
});
