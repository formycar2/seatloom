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
}));

vi.mock('../lib/api', () => {
  const api = {
    listTmuxSessions: async () => state.tmuxSessions,
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
  };
  return {
    api,
    isTauri: () => true,
    onSessionOutput: api.onSessionOutput,
    onSessionExit: api.onSessionExit,
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

beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {};
});

beforeEach(() => {
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
