// The mirror-lost path: when the backend cannot prove the pipe still belongs to
// us, the UI must STOP SHOWING CONTENT. That is the acceptance criterion and it
// is the only thing that makes this fix real.
//
// The defect being fixed (2026-10-27, AEGIS packet
// AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1): an agent ran
// `tmux pipe-pane` with no argument — believing it was a query, when it is a
// STOP — then pointed another pane at the FIFO the app believed was this seat's.
// `#{pane_pipe}` stayed 1 the whole time. The UI rendered that seat's slot full
// of the other seat's output for three hours and reported nothing.
//
// So the tests below are deliberately about the NEGATIVE space: what the UI must
// NOT do when the event arrives. "It logs the reason" is not a pass.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { act } from 'react';
import { useLiveSeatsMirrorStore } from './useLiveSeatsMirrorStore';
import type { LiveSessionDto } from '../lib/types-dto';
import type { ParsedSeat } from '../lib/tmux-seats';

function dto(id: string, over: Partial<LiveSessionDto> = {}): LiveSessionDto {
  return {
    id,
    seatId: 'nimbus',
    runtime: 'tmux-mirror',
    command: 'tmux pipe-pane',
    args: [],
    workingDir: '/w',
    transcriptPath: '/t',
    tmuxSessionName: 'Nimbus-TechArchi-seatloom',
    fifoPath: '/tmp/seatloom-mirror/nimbus.fifo',
    paneRows: 30,
    paneCols: 120,
    initialSnapshotB64: '',
    ...over,
  };
}

function seat(name: string): ParsedSeat {
  return {
    seatName: 'Nimbus',
    sessionName: name,
    ...({} as Record<string, never>),
  } as unknown as ParsedSeat;
}

const SEAT = seat('Nimbus-TechArchi-seatloom');

describe('useLiveSeatsMirrorStore — the mirror-lost path', () => {
  beforeEach(() => {
    useLiveSeatsMirrorStore.setState({
      attachments: [],
      activity: {},
      activeName: null,
      mirrorLost: {},
      autoAttached: false,
    });
  });

  it('drops the attachment AND records why, so the terminal cannot keep rendering', () => {
    // The two halves are both required:
    //   · no removal  -> the bytes keep rendering (the original defect)
    //   · no record   -> the slot looks like an ordinary disconnect and the
    //                    operator has no idea the content was untrustworthy
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });

    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'cross_wired', 'detail');
    });

    const s = useLiveSeatsMirrorStore.getState();
    expect(s.attachments).toHaveLength(0);
    expect(s.mirrorLost['Nimbus-TechArchi-seatloom']).toBeDefined();
    expect(s.mirrorLost['Nimbus-TechArchi-seatloom'].reason).toBe('cross_wired');
  });

  it('also drops the activity counters, so no stale "last output" clock lingers', () => {
    // A counter that keeps ticking after the mirror is unproven is a second,
    // quieter version of the same lie: it says the stream is alive.
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    act(() => {
      useLiveSeatsMirrorStore.getState().addBytes('ses-a', 5000, 12, Date.now());
    });
    expect(useLiveSeatsMirrorStore.getState().activity['ses-a']).toBeDefined();

    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'lost', 'detail');
    });

    expect(useLiveSeatsMirrorStore.getState().activity['ses-a']).toBeUndefined();
  });

  it('falls back to another attachment when the broken one was on screen', () => {
    const other = seat('Flux-Quality&Ops-seatloom');
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: other, dto: dto('ses-b') });
    useLiveSeatsMirrorStore.getState().setActiveName('Nimbus-TechArchi-seatloom');

    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'lost', 'detail');
    });

    // Not null (a blank terminal area) and not the dead seat.
    const name = useLiveSeatsMirrorStore.getState().activeName;
    expect(name).toBe('Flux-Quality&Ops-seatloom');
  });

  it('activeName becomes null when the last mirror is unproven', () => {
    // There is nothing to fall back to. Leaving the old name would make the view
    // look for an attachment that is gone and render nothing at all — silently.
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    useLiveSeatsMirrorStore.getState().setActiveName('Nimbus-TechArchi-seatloom');

    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'lost', 'detail');
    });

    expect(useLiveSeatsMirrorStore.getState().activeName).toBeNull();
  });

  it('leaves the active pane alone when a background seat breaks', () => {
    const other = seat('Flux-Quality&Ops-seatloom');
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: other, dto: dto('ses-b') });
    useLiveSeatsMirrorStore.getState().setActiveName('Nimbus-TechArchi-seatloom');

    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Flux-Quality&Ops-seatloom', 'ses-b', 'lost', 'detail');
    });

    expect(useLiveSeatsMirrorStore.getState().activeName).toBe('Nimbus-TechArchi-seatloom');
    expect(useLiveSeatsMirrorStore.getState().mirrorLost['Flux-Quality&Ops-seatloom']).toBeDefined();
  });

  it('a reconnect clears the banner so it cannot sit over a working terminal', () => {
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'lost', 'detail');
    });
    expect(useLiveSeatsMirrorStore.getState().mirrorLost['Nimbus-TechArchi-seatloom']).toBeDefined();

    act(() => {
      useLiveSeatsMirrorStore.getState().clearMirrorLost('Nimbus-TechArchi-seatloom');
    });

    expect(useLiveSeatsMirrorStore.getState().mirrorLost['Nimbus-TechArchi-seatloom']).toBeUndefined();
  });

  it('clearing a seat that was never broken is a no-op, not a crash', () => {
    // Reconnect can fire for a seat with no prior failure (first-ever attach).
    // This must not throw and must not wipe another seat's record.
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Flux-Quality&Ops-seatloom', 'ses-b', 'lost', 'detail');
    });
    act(() => {
      useLiveSeatsMirrorStore.getState().clearMirrorLost('Nimbus-TechArchi-seatloom');
    });
    expect(useLiveSeatsMirrorStore.getState().mirrorLost['Flux-Quality&Ops-seatloom']).toBeDefined();
  });

  it('a broken mirror does not survive a prune of the session it belonged to', () => {
    // pruneDead is the remount reconciliation. If a session died while the view
    // was away, its banner must not come back and claim the mirror is broken.
    useLiveSeatsMirrorStore.getState().addAttachment({ seat: SEAT, dto: dto('ses-a') });
    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', 'lost', 'detail');
    });
    act(() => {
      useLiveSeatsMirrorStore.getState().pruneDead(new Set(['ses-z']));
    });
    // Nothing left attached; the record is stale information about a session the
    // backend no longer has, and is dropped rather than kept.
    expect(useLiveSeatsMirrorStore.getState().attachments).toHaveLength(0);
  });
});

describe('the three reasons the backend can report', () => {
  beforeEach(() => {
    useLiveSeatsMirrorStore.setState({
      attachments: [],
      activity: {},
      activeName: null,
      mirrorLost: {},
      autoAttached: false,
    });
  });

  // The reason string is passed through verbatim from Rust's
  // `MirrorVerdict::as_str()`. Renaming it here would break log correlation, so
  // this test pins the exact spellings.
  it.each([
    ['lost', 'no writer at all — someone ran the bare pipe-pane, or our writer was replaced'],
    ['cross_wired', 'more than one pane writes our FIFO, so the bytes are not all ours'],
    ['probe_failed', 'the check itself could not run — reported, never assumed healthy'],
  ])('records %s verbatim', (reason) => {
    act(() => {
      useLiveSeatsMirrorStore
        .getState()
        .markMirrorLost('Nimbus-TechArchi-seatloom', 'ses-a', reason, 'detail');
    });
    expect(useLiveSeatsMirrorStore.getState().mirrorLost['Nimbus-TechArchi-seatloom'].reason).toBe(
      reason,
    );
  });
});
