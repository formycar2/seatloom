// SessionTerminal geometry — the scroll defect, measured where it actually lives.
//
// THE OPERATOR'S BUG (2026-10-26): "the live-seats page sometimes will not scroll
// up or down". The DOM half is asserted in LiveSeatsView.test.tsx; this file is
// the MEASURED half, and the two halves are one defect:
//
//   display:none on an inactive attachment
//     -> xterm cannot measure a character cell (its measure element has no box)
//     -> FitAddon.proposeDimensions() returns undefined
//     -> FitAddon.fit() returns early, WITHOUT calling terminal.resize
//     -> the viewport keeps the pane's attach-time row/col seed (the construction
//        seed this component deliberately uses so the first frames are not
//        re-wrapped), which is normally TALLER than the container
//     -> when the pane's content is not longer than that seed row count, the
//        scroll area equals the viewport height: nothing to scroll.
//
// WHY A MOCK AND NOT THE REAL xterm: the real Terminal needs a canvas and a real
// layout engine, and jsdom has neither (every element's clientWidth/clientHeight
// is 0). Every existing test in this repo therefore stubs this component out. That
// is fine for testing the view and useless for testing geometry — so this file
// drives the REAL component against TRANSCRIPTIONS of the two contracts it depends
// on, both read from the installed sources rather than from memory:
//
//   1. FitAddon.fit()  (node_modules/@xterm/addon-fit/src/FitAddon.ts):
//        `const dims = this.proposeDimensions(); if (!dims || ...) return;`
//      i.e. an unmeasurable cell is a SILENT no-op — no resize, no throw, no log.
//   2. FitAddon.proposeDimensions() returns `undefined` when
//        `dims.css.cell.width === 0`.
//
// The transcription is this file's boundary and is stated up front so nobody
// mistakes these assertions for a measurement against real xterm. What IS real
// here is SessionTerminal.tsx — the real effect, the real refs, the real
// pushResize coalescing, the real ResizeObserver wiring.
//
// WHAT IS NOT ASSERTED ANYWHERE IN THIS FILE: real pixels, a real wheel event
// reaching a real scrollback, or a real tmux pane. Those need the operator's
// machine and are his to confirm (the acceptance card says so).

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup, act } from '@testing-library/react';
import React from 'react';

// ── The transcription, hoisted because vi.mock is hoisted ─────────────────────

const fake = vi.hoisted(() => {
  class FakeTerminal {
    static instances: FakeTerminal[] = [];
    options: Record<string, unknown>;
    rows: number;
    cols: number;
    element: HTMLDivElement | null = null;
    /** Fired by the real xterm on every geometry change (xterm.d.ts: onResize). */
    resizeListeners: Array<(e: { cols: number; rows: number }) => void> = [];
    dataListeners: Array<(d: string) => void> = [];
    resizeCalls: Array<{ cols: number; rows: number }> = [];
    disposed = false;
    /** The knob: what proposeDimensions() answers right now (undefined = unmeasurable). */
    proposed: () => { rows: number; cols: number } | undefined = () => undefined;

    constructor(options: Record<string, unknown> & { rows?: number; cols?: number }) {
      this.options = options;
      // The construction seed: the pane's attach-time geometry.
      this.rows = options.rows ?? 24;
      this.cols = options.cols ?? 80;
      FakeTerminal.instances.push(this);
    }
    loadAddon(addon: unknown) {
      // The real Terminal calls activate(this) — that is how FitAddon learns its
      // terminal (FitAddon.ts `activate(terminal)`), not a constructor argument.
      (addon as { activate?: (t: unknown) => void } | undefined)?.activate?.(this);
    }
    open(parent: HTMLElement) {
      // Transcribed from Terminal.open(): xterm CREATES its own element and
      // APPENDS it to the parent — `this.element = document.createElement('div')`
      // then `e.appendChild(this.element)`. So `term.element` is a CHILD of the
      // container, not the container itself. That distinction is what the fix's
      // two observers depend on: the container's box and the terminal element's
      // box are different nodes, and both are watched.
      this.element = parent.ownerDocument.createElement('div');
      this.element.classList.add('xterm');
      parent.appendChild(this.element);
    }
    // Recorded, not stubbed to nothing: the P0 fix (disp-OwOSLvL4EbqoB6YhlBQHi)
    // is precisely about WHICH of these the component calls. clear() and reset()
    // both destroy the scrollback (measured against the real xterm in
    // SessionTerminal.scrollback.test.tsx), so a future re-introduction of either
    // on the geometry path has to fail a test rather than reach the operator.
    writes: string[] = [];
    clearCalls = 0;
    resetCalls = 0;
    write(data: Uint8Array | string) {
      this.writes.push(typeof data === 'string' ? data : new TextDecoder().decode(data));
    }
    clear() {
      this.clearCalls += 1;
    }
    reset() {
      this.resetCalls += 1;
    }
    /**
     * Transcribed from xterm.d.ts:1195 — "Scrolls the display of the terminal
     * to the bottom". Recorded, not stubbed to nothing, because the P0 fix
     * (disp-OwOSLvL4EbqoB6YhlBQHi) replaced `clear()+reset()` with a viewport
     * clear followed by exactly this call: dropping it would make the fix's
     * "the operator lands on the repaint, not mid-history" half unfalsifiable.
     */
    scrollToBottomCalls = 0;
    scrollToBottom() {
      this.scrollToBottomCalls += 1;
    }
    dispose() {
      this.disposed = true;
    }
    focus() {}
    blur() {}
    resize(cols: number, rows: number) {
      const changed = cols !== this.cols || rows !== this.rows;
      this.resizeCalls.push({ cols, rows });
      this.cols = cols;
      this.rows = rows;
      // The real xterm fires onResize after applying, and the component's retry
      // only cares about a real change.
      if (changed) for (const l of this.resizeListeners) l({ cols, rows });
    }
    onResize(cb: (e: { cols: number; rows: number }) => void) {
      this.resizeListeners.push(cb);
      // The real IEvent disposable REMOVES the listener; a no-op dispose here
      // would make "unmount cleans up" unfalsifiable.
      return {
        dispose: () => {
          this.resizeListeners = this.resizeListeners.filter((l) => l !== cb);
        },
      };
    }
    onData(cb: (d: string) => void) {
      this.dataListeners.push(cb);
      return { dispose: () => {} };
    }
  }

  class FakeFitAddon {
    static fitCalls = 0;
    private terminal: FakeTerminal | undefined;
    constructor() {}
    activate(terminal: unknown) {
      this.terminal = terminal as FakeTerminal;
    }
    dispose() {}
    /** Transcribed: `if (!dims || ...) return;` */
    fit() {
      FakeFitAddon.fitCalls += 1;
      const dims = this.proposeDimensions();
      if (!dims) return; // ← the silent no-op the whole defect hinges on
      // Transcribed verbatim from FitAddon.ts:
      //   `if (this._terminal.rows !== dims.rows || this._terminal.cols !== dims.cols) {
      //      core._renderService.clear(); this._terminal.resize(dims.cols, dims.rows); }`
      // That equality short-circuit is what makes fit() idempotent, and it is
      // also what stops the component's retry (which listens on the terminal's
      // own resize event) from re-entering fit() forever.
      const terminal = this.terminal!;
      if (terminal.rows !== dims.rows || terminal.cols !== dims.cols) {
        terminal.resize(dims.cols, dims.rows);
      }
    }
    /** Transcribed: undefined when cell.width === 0. */
    proposeDimensions() {
      return this.terminal?.proposed();
    }
  }

  const ptyResizeCalls: Array<{ sessionId: string; rows: number; cols: number }> = [];

  // jsdom has no ResizeObserver at all. This stub records what was observed and
  // lets a test fire a callback for it — which is the ONLY way to exercise the
  // retry the fix added ("xterm's own element's box changed -> maybe a fit is
  // possible now"). The stub is deliberately dumb about layout (jsdom has none
  // anyway); it is a trigger, not a simulation of when the browser would fire.
  class StubResizeObserver {
    static observed: Array<{ observer: StubResizeObserver; el: Element }> = [];
    private callback: () => void;
    constructor(callback: () => void) {
      this.callback = callback;
    }
    observe(el: Element) {
      StubResizeObserver.observed.push({ observer: this, el });
    }
    unobserve() {}
    disconnect() {
      StubResizeObserver.observed = StubResizeObserver.observed.filter((r) => r.observer !== this);
    }
    /** Test-only: fire as if this element's box changed. */
    static fire(el: Element) {
      for (const record of StubResizeObserver.observed.filter((r) => r.el === el)) {
        record.observer.callback();
      }
    }
    static count(el: Element) {
      return StubResizeObserver.observed.filter((r) => r.el === el).length;
    }
  }

  return { FakeTerminal, FakeFitAddon, ptyResizeCalls, StubResizeObserver };
});

const { FakeTerminal, ptyResizeCalls, StubResizeObserver } = fake;

vi.mock('@xterm/xterm', () => ({ Terminal: fake.FakeTerminal }));
vi.mock('@xterm/addon-fit', () => ({ FitAddon: fake.FakeFitAddon }));
vi.mock('@xterm/addon-web-links', () => ({ WebLinksAddon: class {} }));
vi.mock('@xterm/xterm/css/xterm.css', () => ({}));

// The Tauri surface the component subscribes through. Path is relative to THIS
// file (src/app-v2/panel/), matching what SessionTerminal.tsx imports.
vi.mock('../../lib/api', () => ({
  api: {
    ptyResize: async (sessionId: string, rows: number, cols: number) => {
      fake.ptyResizeCalls.push({ sessionId, rows, cols });
      return {};
    },
    ptyWriteBytes: async () => ({}),
  },
  decodeSessionOutput: (_b64: string) => new Uint8Array([0x41]),
  onSessionOutput: async () => () => {},
  onSessionExit: async () => () => {},
}));

import { SessionTerminal } from './SessionTerminal';

// jsdom performs no layout — clientWidth/clientHeight are 0 for every element,
// which the component (correctly) reads as "no box, do not fit". Give elements a
// box so the geometry path is reachable. A test-environment shim standing in for
// the browser's layout engine; the numbers are arbitrary but stable.
const CONTAINER_W = 800;
const CONTAINER_H = 360;
const giveLayout = (w: number, h: number) => {
  Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
    configurable: true,
    get() {
      return w;
    },
  });
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    get() {
      return h;
    },
  });
};

beforeEach(() => {
  giveLayout(CONTAINER_W, CONTAINER_H);
  // jsdom has no ResizeObserver at all; install the stub defined in the hoisted
  // block, so the component's two observers have something to register with.
  (globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = fake.StubResizeObserver;
});
afterEach(() => {
  cleanup();
  fake.FakeTerminal.instances = [];
  fake.FakeFitAddon.fitCalls = 0;
  fake.ptyResizeCalls.length = 0;
  fake.StubResizeObserver.observed = [];
  delete (globalThis as unknown as { ResizeObserver?: unknown }).ResizeObserver;
});

/** The terminal this render produced. */
const term = () => FakeTerminal.instances[0];
/** Metrics valid, and what a fit proposes for them. */
const measurable = (rows: number, cols: number) => () => ({ rows, cols });

const mountTerminal = async (paneRows = 54, paneCols = 219) => {
  await act(async () => {
    render(
      <SessionTerminal
        sessionId="sl-live-QL"
        paneRows={paneRows}
        paneCols={paneCols}
        initialSnapshotB64="QUJD"
      />,
    );
  });
  return term();
};

describe('when xterm cannot measure a character cell yet', () => {
  it('fit() is a no-op and the viewport stays at the attach-time seed', async () => {
    // The defect's steady state: the pane's seed is taller than the container,
    // and the cell cannot be measured yet.
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = () => undefined;

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    // NOT resized away from the seed — this is exactly what makes it unscrollable.
    expect(mounted.resizeCalls).toEqual([]);
    // ...and nothing reached the backend, so the pane never learned either.
    expect(ptyResizeCalls).toEqual([]);
    expect(mounted.rows).toBe(54);
    expect(mounted.cols).toBe(219);
  });
});

describe('when the metrics are valid', () => {
  it('the viewport is fitted and the fitted geometry reaches the pane', async () => {
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    expect(mounted.resizeCalls).toEqual([{ cols: 140, rows: 30 }]);
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('an impossible fit is retried once the metrics become valid — not left stale', async () => {
    const mounted = await mountTerminal(54, 219);
    // At arm time the cell is unmeasurable (the display:none window).
    mounted.proposed = () => undefined;
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(mounted.resizeCalls).toEqual([]);

    // The attachment becomes visible: xterm re-measures (its own
    // IntersectionObserver) and its element's box changes. The component must
    // notice and fit. Before the fix it waited for an unrelated window resize,
    // which is why the symptom was intermittent.
    mounted.proposed = measurable(30, 140);
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    expect(mounted.resizeCalls).toEqual([{ cols: 140, rows: 30 }]);
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('a geometry change reported by xterm itself also re-syncs an unfitted viewport', async () => {
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = () => undefined;
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(mounted.resizeCalls).toEqual([]);

    // xterm's own onResize (char-size change, dpr change) is the other retry
    // trigger. It must not be a dead subscription.
    mounted.proposed = measurable(24, 100);
    await act(async () => {
      mounted.resize(100, 24);
    });

    expect(mounted.resizeCalls).toEqual([{ cols: 100, rows: 24 }]);
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 24, cols: 100 }]);
  });

  it("xterm's own element changing size is watched, so a fit becomes possible without a window resize", async () => {
    // The browser fires no `resize` event for this: the attachment layer becomes
    // visible, xterm re-measures, and only ITS element's box changes. That is the
    // moment a previously impossible fit becomes possible — and before the fix the
    // component had no way to hear about it, so the viewport sat at the seed
    // geometry until the operator happened to resize the window.
    const mounted = await mountTerminal(54, 219);
    const element = mounted.element!;
    mounted.proposed = () => undefined;
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(mounted.resizeCalls).toEqual([]);

    // The subscription must exist on the terminal's element, not merely be wired.
    expect(StubResizeObserver.count(element)).toBe(1);

    mounted.proposed = measurable(30, 140);
    await act(async () => {
      StubResizeObserver.fire(element);
    });

    expect(mounted.resizeCalls).toEqual([{ cols: 140, rows: 30 }]);
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('a container with no area is not fitted (the second line of the old guard)', async () => {
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);

    // The layer is un-laid-out again (zero box): fitting would propose nonsense
    // and resize the pane to nothing.
    giveLayout(CONTAINER_W, 0);
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    expect(mounted.resizeCalls).toEqual([]);
    expect(ptyResizeCalls).toEqual([]);
  });

  it('a repeat resize at the same geometry does not spam the backend', async () => {
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
      window.dispatchEvent(new Event('resize'));
      window.dispatchEvent(new Event('resize'));
    });

    // One push per real geometry change, not one per resize event.
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('a fit that worked is retried again if the metrics are lost and come back', async () => {
    // The flag must be a LIVE answer, not a one-way latch. Sequence: the viewport
    // is fitted, then xterm loses valid metrics (its element's box collapses —
    // e.g. the panel is dragged to zero height), then the box comes back at a
    // different geometry. A latch that never returns to false would skip the
    // second fit and leave the viewport stale, which is the defect's shape.
    //
    // The recovery is fired through the ELEMENT observer, not a window resize:
    // the window path calls syncGeometry() unconditionally and would hide a
    // broken flag. The observer path is the only one that consults it.
    const mounted = await mountTerminal(54, 219);
    const element = mounted.element!;
    mounted.proposed = measurable(30, 140);
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);

    // Metrics lost: the fit cannot run, so the viewport must be marked unfitted.
    mounted.proposed = () => undefined;
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(mounted.resizeCalls).toHaveLength(1);

    // Box and metrics back, at a DIFFERENT geometry. Reported through the
    // element observer — the channel that only fires a retry when unfitted.
    mounted.proposed = measurable(40, 120);
    await act(async () => {
      StubResizeObserver.fire(element);
    });

    expect(mounted.resizeCalls).toEqual([
      { cols: 140, rows: 30 },
      { cols: 120, rows: 40 },
    ]);
    expect(ptyResizeCalls).toEqual([
      { sessionId: 'sl-live-QL', rows: 30, cols: 140 },
      { sessionId: 'sl-live-QL', rows: 40, cols: 120 },
    ]);
  });

  it('a successful fit clears the retry flag, so a later no-op fit is not retried forever', async () => {
    // The flag is the guard against a fit/notify/fit loop: once the viewport is
    // fitted, an xterm resize that proposes nothing new must NOT re-enter the
    // fit path. If the flag were stuck true, every unrelated xterm resize would
    // re-fit; if it were stuck false, it would re-fit on every single one. The
    // observable difference is the number of backend pushes for one geometry.
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    expect(ptyResizeCalls).toHaveLength(1);

    // Now xterm reports the SAME geometry repeatedly (e.g. a char-size
    // re-measure that lands on identical metrics). Fit is idempotent, so the
    // terminal is not resized and the backend is not spammed.
    await act(async () => {
      mounted.resize(140, 30);
      mounted.resize(140, 30);
      mounted.resize(140, 30);
    });
    expect(ptyResizeCalls).toHaveLength(1);
  });

  it('unmounting disposes the terminal and drops its listeners', async () => {
    let view: { unmount: () => void } | undefined;
    await act(async () => {
      view = render(
        <SessionTerminal
          sessionId="sl-live-QL"
          paneRows={54}
          paneCols={219}
          initialSnapshotB64="QUJD"
        />,
      );
    });
    const mounted = term();

    await act(async () => {
      view!.unmount();
    });

    expect(mounted.disposed).toBe(true);
    expect(mounted.resizeListeners).toEqual([]);
  });
});

// ── P0 (2026-10-27): the scrollback must survive the first geometry sync ──────
//
// The defect: when the fitted geometry differed from the pane seed, this
// component called `terminal.clear(); terminal.reset();`. Both destroy the
// scrollback — measured against the real xterm in
// SessionTerminal.scrollback.test.tsx, which left 40 lines of a 300-line history
// (== the row count) with the oldest line empty. Every seat whose fitted geometry
// differed from its seed therefore had NO history to scroll, which is the
// operator's "除了 ql 这个 seat 实时席位可以上下翻,其他所有 seat 都不能".
//
// These assertions live HERE rather than there because they are about the
// component's DECISION, and reaching that decision requires fit() to produce
// dimensions — which needs real character-cell metrics jsdom cannot produce. The
// transcription is the tool that can drive this branch; the real xterm is the tool
// that shows what survived it. Both halves are one defect and both are needed.
describe('the stale-snapshot clear must not destroy the scrollback (P0)', () => {
  it('a differing fitted geometry clears the VIEWPORT and parks at the bottom — never clear()/reset()', async () => {
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    // The two buffer-destroying calls must be gone from this path.
    expect(mounted.clearCalls).toBe(0);
    expect(mounted.resetCalls).toBe(0);

    // In their place: a viewport clear (ESC[2J ESC[H) that leaves the scrollback
    // alone, then scrollToBottom so the operator lands on the repaint rather than
    // somewhere in the middle of the history.
    expect(mounted.writes.some((w) => w.includes('\x1b[2J'))).toBe(true);
    expect(mounted.scrollToBottomCalls).toBe(1);

    // And the geometry still reaches the pane — the fix must not have cost the
    // operator the bidirectional sync that the whole geometry contract is for.
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('an exact geometry match clears nothing at all — why the active seat always worked', async () => {
    // The difference between seats was never about the seats: for the one the
    // operator was looking at, the fitted geometry equalled the seed, so the clear
    // branch never ran. Asserted so nobody "simplifies" the branch away and starts
    // clearing on every mount, which would take the history off the active seat too.
    const mounted = await mountTerminal(30, 140);
    mounted.proposed = measurable(30, 140);

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    expect(mounted.clearCalls).toBe(0);
    expect(mounted.resetCalls).toBe(0);
    expect(mounted.writes.some((w) => w.includes('\x1b[2J'))).toBe(false);
    expect(mounted.scrollToBottomCalls).toBe(0);
    // The geometry IS still pushed to the pane — but once, not because anything
    // changed: `lastPushedRef` starts empty, so the first sync always reports the
    // seed so the backend has it. That is pre-existing behaviour and is unrelated
    // to the clear; what matters here is that the SCROLLBACK is untouched.
    expect(ptyResizeCalls).toEqual([{ sessionId: 'sl-live-QL', rows: 30, cols: 140 }]);
  });

  it('the clear runs at most once, however many geometry syncs follow', async () => {
    // It guards the attach-time snapshot, which is written once. Re-running it on
    // every later resize would wipe the live output the operator is reading.
    const mounted = await mountTerminal(54, 219);
    mounted.proposed = measurable(30, 140);

    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });
    const writesAfterFirst = mounted.writes.length;

    // A later, different geometry — the operator resizing the panel, say.
    mounted.proposed = measurable(28, 120);
    await act(async () => {
      window.dispatchEvent(new Event('resize'));
    });

    expect(mounted.clearCalls).toBe(0);
    expect(mounted.resetCalls).toBe(0);
    expect(mounted.writes.some((w) => w.includes('\x1b[2J'))).toBe(true);
    // Still exactly the one viewport clear from the first sync.
    expect(mounted.writes.filter((w) => w.includes('\x1b[2J')).length).toBe(1);
    expect(mounted.scrollToBottomCalls).toBe(1);
    expect(writesAfterFirst).toBeGreaterThan(0);
  });
});
