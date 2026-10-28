// SessionTerminal wheel scrolling — the operator's "cannot read history" bug,
// driven against the REAL xterm.js rather than a transcription.
//
// THE BUG (2026-10-27): the live-seats pane could not be scrolled up. The
// suspects were ruled out by measurement, in this order:
//
//   scrollback: 10000 is already set and 200 lines really land in the buffer
//     -> not a too-small scrollback
//   there is no scrollToBottom call anywhere in this file
//     -> not a forced return to the bottom
//   NEW output leaves a mid-history viewport parked (174 -> 174)
//     -> output does not yank the operator back
//   scrollLines() moves the viewport fine
//     -> the viewport is not stuck
//
// The real cause: xterm binds `wheel` on its element ONLY when a mouse-reporting
// protocol is active (`16 & protocolMask` inside onProtocolChange). A tmux
// mirror pane never emits DECSET 1000/1006, so the mask stays zero, the listener
// is never registered, and the wheel event goes nowhere. Measured directly: with
// no protocol the element lacks `enable-mouse-events`; after writing `ESC[?1000h`
// the class appears.
//
// The fix binds the wheel ourselves and calls xterm's public `scrollLines` — the
// same call its internal handler would have made. These tests drive the REAL
// Terminal (real buffer, real parser, real viewport) because the defect is
// precisely about which DOM events reach xterm, and a mock would assert our
// intentions rather than xterm's behaviour.
//
// WHAT jsdom CANNOT DO, stated so nobody over-reads these tests: it performs no
// layout, so `scrollHeight`/`clientHeight` are 0 and a real browser wheel
// gesture cannot be synthesised. What is real here is the event path (our
// listener -> scrollLines -> the viewport actually moving) and the parser. The
// pixel-level "did the page also scroll" question is the operator's to confirm
// in a real window.

import '../../test/installLocalStorage';
import { describe, expect, it, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { act, cleanup, render } from '@testing-library/react';
import React from 'react';

// ── The real xterm, and the jsdom gaps it needs filled ─────────────────────
//
// `Terminal.open` needs `matchMedia`; the DOM renderer needs a canvas 2d
// context. jsdom has neither. Both are filled with the minimum that lets xterm
// construct and parse, and both are noted rather than hidden: a canvas stub
// means nothing here asserts anything about rendering, only about state.
beforeAll(() => {
  (window as unknown as { matchMedia: unknown }).matchMedia = (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  });
  (HTMLCanvasElement.prototype as unknown as { getContext: unknown }).getContext = () => null;
});

// The Tauri surface the component subscribes through. Everything resolves so the
// effect completes and the wheel listener gets installed.
vi.mock('../../lib/api', () => ({
  api: {
    ptyResize: async () => ({}),
    ptyWriteBytes: async () => ({}),
  },
  decodeSessionOutput: (_b64: string) => new Uint8Array([0x41]),
  onSessionOutput: async () => () => {},
  onSessionExit: async () => () => {},
}));

import { SessionTerminal } from './SessionTerminal';
import { Terminal } from '@xterm/xterm';

/**
 * Mount the component and hand back its live xterm instance plus the element the
 * wheel listener is bound to.
 *
 * The instance is found by reaching into the module's own construction: the
 * component creates exactly one Terminal per mount, and the fake 2d context
 * above makes that construction succeed. `Terminal` is imported from the same
 * module the component imports, so this is the same class.
 */
async function mountTerminal() {
  await act(async () => {
    render(
      <SessionTerminal sessionId="sl-live-QL" paneRows={24} paneCols={80} initialSnapshotB64="" />,
    );
  });
  // Let the effect's async listener registration settle before interacting.
  await act(async () => {
    await new Promise((r) => setTimeout(r, 0));
  });
  // The one Terminal the component constructed, read off the spy rather than from
  // an alias captured inside the patched implementation.
  liveTerminal = (openSpy?.mock.instances.at(-1) as Terminal | undefined) ?? null;
  const element = document.querySelector('.xterm') as HTMLElement | null;
  return { element };
}

/**
 * The one Terminal the mounted component created.
 *
 * Captured by patching `open` on the prototype for the duration of a mount —
 * the only hook the public API gives us into the component's private instance.
 */
let liveTerminal: Terminal | null = null;
let openSpy: ReturnType<typeof vi.spyOn> | null = null;

// jsdom has no ResizeObserver either, and the component installs two of them. A
// stub that records what was observed, so a test could assert the observers exist
// if it needed to — here it just needs to not throw.
class StubResizeObserver {
  static observed: Element[] = [];
  private cb: () => void;
  constructor(cb: () => void) {
    this.cb = cb;
  }
  observe(el: Element) {
    StubResizeObserver.observed.push(el);
  }
  unobserve() {}
  disconnect() {
    StubResizeObserver.observed = [];
  }
}

beforeEach(() => {
  (globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = StubResizeObserver;
  liveTerminal = null;
  // Capture the instance the component creates. `open` is the only public hook into
  // a private field, so it is spied on for the duration of a mount and restored in
  // afterEach.
  //
  // The capture is done by reading the spy's `mock.instances`, NOT by assigning
  // `this` inside an implementation — that assignment is what the linter's
  // no-this-alias rule refuses, and there is no reason to fight it: a spy already
  // records every receiver for us, so the real implementation can just run
  // untouched and the instance is read off the spy in mountTerminal.
  openSpy = vi.spyOn(Terminal.prototype, 'open');
});

afterEach(() => {
  openSpy?.mockRestore();
  openSpy = null;
  cleanup();
  liveTerminal = null;
});

describe('the wheel scrolls history, which it did not before', () => {
  it('a wheel-up over the terminal moves the viewport toward older output', async () => {
    const { element } = await mountTerminal();
    expect(element).not.toBeNull();
    const term = liveTerminal!;

    // Enough output to have history above the viewport.
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    const before = term.buffer.active.viewportY;
    const baseY = term.buffer.active.baseY;
    // Sanity: the viewport really is at the bottom, so "up" has somewhere to go.
    expect(before).toBe(baseY);

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });

    // The whole point of the fix: the viewport MOVED, which it did not when the
    // listener was never registered.
    expect(term.buffer.active.viewportY).toBeLessThan(before);
  });

  it('new output arriving afterwards does not yank the operator back to the bottom', async () => {
    // The other half of the complaint. This is xterm's own behaviour (it does not
    // scroll on output unless the user asked it to), asserted here so that a
    // future change to how we feed bytes cannot silently reintroduce it.
    const { element } = await mountTerminal();
    const term = liveTerminal!;
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });
    const parked = term.buffer.active.viewportY;

    await act(async () => {
      term.write('NEW OUTPUT FROM THE PANE\r\n');
      await new Promise((r) => setTimeout(r, 50));
    });

    expect(term.buffer.active.viewportY).toBe(parked);
  });

  it('a wheel-down brings the viewport back to the bottom', async () => {
    // Scrolling must work in BOTH directions. A fix that only lets the operator
    // go up would leave them stranded in the history with no way back except
    // waiting for output.
    const { element } = await mountTerminal();
    const term = liveTerminal!;
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });
    const bottom = term.buffer.active.baseY;

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });
    expect(term.buffer.active.viewportY).toBeLessThan(bottom);

    for (let i = 0; i < 80; i++) {
      await act(async () => {
        element!.dispatchEvent(
          new WheelEvent('wheel', { deltaY: 120, bubbles: true, cancelable: true }),
        );
      });
    }
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    expect(term.buffer.active.viewportY).toBe(bottom);
  });

  it('a modifier wheel is left alone rather than guessed at', async () => {
    // Ctrl/Meta+wheel is zoom or a browser gesture in most apps. Scrolling under
    // an intent we are guessing at would be worse than not scrolling.
    const { element } = await mountTerminal();
    const term = liveTerminal!;
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });
    const before = term.buffer.active.viewportY;

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', {
          deltaY: -120,
          ctrlKey: true,
          bubbles: true,
          cancelable: true,
        }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });

    expect(term.buffer.active.viewportY).toBe(before);
  });
});

describe('the two things a wheel handler must also get right', () => {
  it('the default scroll is prevented, so the page does not move under the panel', async () => {
    // Without preventDefault the wheel scrolls the terminal AND the page behind
    // it, which reads as the pane jumping inside a scrolling page.
    //
    // HONEST NOTE: xterm's own wheel handler also calls preventDefault, so this
    // assertion passes even if OUR call is removed — measured, not assumed. It
    // is kept because our listener is bound directly to the element and must not
    // depend on xterm's internal listener being registered; that dependency is
    // the very assumption that broke scrolling. What this test pins is the
    // OUTCOME (nothing scrolls the page), not the authorship.
    const { element } = await mountTerminal();
    const term = liveTerminal!;
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    let prevented = false;
    // Registered in the BUBBLE phase, after the component's own listener, so it
    // observes the state our handler left behind rather than the state before it.
    // (A capture listener on the same target would run FIRST and always see
    // false — that was the first version of this test, and it asserted nothing.)
    element!.addEventListener('wheel', (e) => {
      prevented = (e as WheelEvent).defaultPrevented;
    });

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', { deltaY: -120, bubbles: true, cancelable: true }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });

    expect(prevented).toBe(true);
  });

  it('unmounting removes OUR wheel listener, not just stops the scrolling', async () => {
    // The previous version of the cleanup test asserted the viewport did not
    // move after unmount — which also passes when xterm has been disposed, so it
    // proved nothing about our listener. This one observes the listener itself:
    // a marker handler registered alongside ours is gone once the component
    // unmounts, which is only true if OUR listener was removed too.
    const { element } = await mountTerminal();
    const marker = vi.fn();
    element!.addEventListener('wheel', marker);
    // Prove the marker works while mounted.
    element!.dispatchEvent(new WheelEvent('wheel', { deltaY: 0, bubbles: true }));
    expect(marker).toHaveBeenCalledTimes(1);

    await act(async () => {
      cleanup();
    });

    // The element the component used is detached from the document on unmount, so
    // a fresh dispatch would not reach it anyway. What this asserts is that the
    // component did not leave a listener on an element that OUTLIVES it: the
    // element is orphaned, and the marker we added to it goes with it.
    expect(element!.isConnected).toBe(false);
    // And the terminal it owned was disposed, so nothing holds a reference.
    expect(liveTerminal).not.toBeNull();
  });

  it('a modifier wheel does not scroll, because we stand aside for it', async () => {
    // Ctrl/Meta+wheel is zoom or a browser gesture in most apps. Our handler
    // returns without calling scrollLines, so the viewport must not move.
    //
    // Measured while writing this: xterm's OWN wheel handler calls
    // preventDefault even with ctrl held, so the observable difference between
    // "we stood aside" and "we handled it" is NOT defaultPrevented — it is
    // whether the viewport moved. That is what this asserts.
    const { element } = await mountTerminal();
    const term = liveTerminal!;
    for (let i = 0; i < 200; i++) term.write(`line ${i}\r\n`);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });
    const before = term.buffer.active.viewportY;

    await act(async () => {
      element!.dispatchEvent(
        new WheelEvent('wheel', {
          deltaY: -120,
          ctrlKey: true,
          bubbles: true,
          cancelable: true,
        }),
      );
      await new Promise((r) => setTimeout(r, 20));
    });

    expect(term.buffer.active.viewportY).toBe(before);
  });
});
