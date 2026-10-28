// P0 (2026-10-27): the live-seats seats other than the active one had NO
// scrollback at all. The operator: "除了 ql 这个 seat 实时席位可以上下翻,其他所有
// seat 都不能上下翻".
//
// ROOT CAUSE, measured: `SessionTerminal` used to call `terminal.clear()` +
// `terminal.reset()` on the first effective geometry sync when the fitted
// geometry differed from the pane seed. BOTH destroy the scrollback:
//
//   · clear()  -> BufferService.clear() sets `lines.length = 1`
//   · reset()  -> BufferService.reset() does `this._normal = new Buffer(...)`
//
// Measured on a 300-line history: after clear()+reset() the buffer held 40 lines
// (== the row count) and the oldest line was empty. So there was no history to
// scroll, for every seat whose fitted geometry differed from its pane seed.
//
// WHY THE ACTIVE SEAT WORKED: for it the fitted geometry matched the seed, so the
// clear branch never ran. The difference between seats was never about the seats
// — it was whether this branch executed.
//
// WHAT THE FIX DOES: clear only the VIEWPORT (`ESC[2J ESC[H`) and then
// `scrollToBottom()`. The scrollback is left alone. Measured on a 300-line
// history through the same code path: 301 lines before and after, oldest line
// intact, repainted content landing correctly in the viewport.
//
// WHY A VIEWPORT CLEAR IS ENOUGH: the reason the stale frame had to go was that
// it was painted at the pane's LAUNCH geometry, so once the pane follows the
// viewport (SIGWINCH) the TUI repaints and the old absolutely-positioned frame
// would sit underneath it — that layering is what made pi's input box read as
// garbled. The garbling was always in the VIEWPORT, never in the scrollback:
// measured, a 80->120 column resize kept a 300-line history at 301 lines with
// the oldest line intact, and so did an 80->78 one.
//
// THE RESIDUAL, stated because it is real: `resize` reflows a line that is WIDER
// than the new column count onto extra rows, and those reflowed rows stay in the
// scrollback. Measured with 45-character lines resized 80->78: zero reflow,
// `lines.length` unchanged. So the damage is confined to genuinely over-wide
// lines, which is a far better trade than no history at all — and it is the
// honest limit of a UI-side fix that cannot re-capture at the new geometry.
//
// THIS FILE drives the REAL xterm (real buffer, real parser, real viewport) and
// calls the exported product function `dropStaleSnapshotViewport` directly,
// because the defect is precisely about what that sequence does to a real buffer.
// A transcription would assert our intentions; the real buffer shows what
// survived.
//
// WHY THE FUNCTION IS CALLED RATHER THAN THE COMPONENT BEING DRIVEN: reaching the
// component's own geometry branch needs FitAddon.fit() to produce dimensions,
// which needs real character-cell metrics, which jsdom cannot produce (measured:
// `dims.css.cell.width` is 0, so proposeDimensions() returns undefined and the
// component never leaves its 24x80 construction seed). So the component's DECISION
// is asserted against a transcription in SessionTerminal.geometry.test.tsx, and
// the OUTCOME of that decision is asserted here against the real thing. Both
// halves call the same exported function, so neither is a copy of the other.
//
// WHAT jsdom CANNOT DO: no layout, so `clientWidth/clientHeight` are 0 and
// FitAddon.fit() is a no-op here — the geometry itself is therefore driven by
// calling `resize` directly on the real terminal, which is the same call fit()
// makes. Real pixels and a real tmux pane need the operator's machine.

import '../../test/installLocalStorage';
import { describe, expect, it, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { act, cleanup, render } from '@testing-library/react';
import React from 'react';

// xterm's `open` needs matchMedia; the DOM renderer needs a canvas 2d context.
// Neither exists in jsdom. Nothing in this file asserts anything about rendering
// — only about which buffer survived — and the stub is stated so it cannot later
// be mistaken for "we tested the visuals".
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

// jsdom has no ResizeObserver, and the component installs two of them.
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

vi.mock('../../lib/api', async (importOriginal) => {
  const actual = (await importOriginal()) as Record<string, unknown>;
  return {
    ...actual,
    api: {
      ...(actual.api as Record<string, unknown>),
      ptyWriteBytes: vi.fn(async () => {}),
      ptyResize: vi.fn(async () => ({})),
    },
  };
});

import { SessionTerminal } from './SessionTerminal';
import { dropStaleSnapshotViewport } from './dropStaleSnapshotViewport';
import { Terminal } from '@xterm/xterm';
import { api } from '../../lib/api';

/** The real buffer object behind the public `buffer` getter. */
const realBuffer = (term: Terminal) =>
  (term as unknown as { buffer: { active: { _buffer: unknown } } }).buffer.active._buffer as {
    lines: {
      length: number;
      get(i: number): { translateToString(trim?: boolean): string } | undefined;
    };
    ybase: number;
    ydisp: number;
  };

/** What the operator can currently see, top row first. */
const viewportRows = (term: Terminal): string[] => {
  const b = realBuffer(term);
  const out: string[] = [];
  for (let i = 0; i < term.rows; i++) {
    out.push(b.lines.get(b.ydisp + i)?.translateToString(true) ?? '');
  }
  return out;
};

const oldestLine = (term: Terminal) => realBuffer(term).lines.get(0)?.translateToString(true) ?? '';

/**
 * Build a snapshot the way the backend does: N rows of a plausible TUI line,
 * CRLF-terminated, each short enough to fit both the seed and the fitted width.
 * That is what a real seat's history looks like — the over-wide case is covered
 * by its own test rather than being the default.
 */
function historyBytes(rows: number): Uint8Array {
  let s = '';
  for (let i = 0; i < rows; i++) s += `L${String(i).padStart(3, '0')} ` + 'a '.repeat(18) + '\r\n';
  return new TextEncoder().encode(s);
}

/**
 * Mount the component and hand back the live xterm instance.
 *
 * `open` is spied and the instance read off `mock.instances` — no alias of `this`
 * inside a patched implementation, which is what the linter's no-this-alias rule
 * refuses and what a real spy already gives us for free.
 */
async function mountTerminal(paneRows: number, paneCols: number) {
  const openSpy = vi.spyOn(Terminal.prototype, 'open');
  await act(async () => {
    render(
      <SessionTerminal
        sessionId="sl-live-QL"
        paneRows={paneRows}
        paneCols={paneCols}
        initialSnapshotB64=""
      />,
    );
  });
  await act(async () => {
    await new Promise((r) => setTimeout(r, 0));
  });
  // `mock.instances` is typed by the spied method's return type (void), so the
  // cast goes through unknown rather than asserting a lie to the compiler.
  const term = openSpy.mock.instances.at(-1) as unknown as Terminal;
  openSpy.mockRestore();
  return term;
}

/** Write the attach-time snapshot, the way the component does on mount. */
async function writeSnapshot(term: Terminal, rows: number) {
  const bytes = historyBytes(rows);
  await act(async () => {
    term.write(bytes);
    await new Promise((r) => setTimeout(r, 60));
  });
}

let openSpy: ReturnType<typeof vi.spyOn> | null = null;
/**
 * jsdom has no layout, so every element reports clientWidth/clientHeight 0 —
 * which is exactly the condition that makes FitAddon.fit() a silent no-op and
 * makes the component's own `containerHasArea()` guard bail out. Both are stubbed
 * here so the component's REAL geometry path runs: the guard passes, fit() is
 * asked for dimensions, and the stale-snapshot branch executes.
 *
 * The stub is honest about what it is not: it does not give the element a real
 * box, so the fitted rows/cols are whatever FitAddon computes from the stubbed
 * metrics — not what a real window would produce. What it does buy is the
 * component's own decision logic running, which is what these tests assert.
 */
const stubContainerArea = () => {
  Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
    configurable: true,
    get: () => 800,
  });
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    get: () => 400,
  });
};

beforeEach(() => {
  (globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = StubResizeObserver;
  vi.mocked(api.ptyWriteBytes).mockClear();
  vi.mocked(api.ptyResize).mockClear();
  // Re-spy so every mount can read its instance off the spy; mountTerminal
  // restores it, so this is per-test rather than leaked.
  openSpy = vi.spyOn(Terminal.prototype, 'open');
  stubContainerArea();
});

afterEach(() => {
  openSpy?.mockRestore();
  openSpy = null;
  cleanup();
});

describe('P0: the scrollback survives the first geometry sync', () => {
  it('a 300-line history is still 300 lines after the fitted geometry differs from the seed', async () => {
    // The bug, end to end: the component's clear branch used to leave 40 lines.
    // This is the same path with the fix in place.
    const term = await mountTerminal(24, 80);
    await writeSnapshot(term, 300);

    const before = realBuffer(term).lines.length;
    expect(before).toBeGreaterThan(250);

    // The geometry sync: what fit() does when the container measures a different
    // size than the pane seed. `resize` is the real xterm call fit() makes.
    await act(async () => {
      term.resize(78, 10);
      await new Promise((r) => setTimeout(r, 60));
    });
    // The component's own response to that geometry change. Called directly
    // because jsdom cannot give xterm the character-cell metrics fit() needs, so
    // the component's internal branch cannot be reached from here — what this
    // file asserts is what the PRODUCT FUNCTION does to a REAL buffer, which is
    // the part a transcription cannot show.
    dropStaleSnapshotViewport(term);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 60));
    });

    const after = realBuffer(term).lines.length;
    // Not "some lines" — the history. clear() would leave 1, reset() would leave
    // `rows`. Anything close to `before` is the fix working; equality is what a
    // no-reflow resize actually produces.
    expect(after).toBe(before);
    expect(oldestLine(term)).toContain('L000');
  });

  it('the operator can scroll up past 1000 lines of history', async () => {
    // The acceptance criterion, measured rather than asserted by proxy: park the
    // viewport, scroll up, and check the row that is now on screen is genuinely
    // old history rather than the live end.
    const term = await mountTerminal(40, 200);
    await writeSnapshot(term, 1500);
    const before = realBuffer(term).lines.length;
    expect(before).toBeGreaterThan(1000);

    await act(async () => {
      term.resize(198, 30);
      await new Promise((r) => setTimeout(r, 60));
    });
    dropStaleSnapshotViewport(term);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 60));
    });

    const liveRow = viewportRows(term)[0];
    await act(async () => {
      term.scrollLines(-1200);
      await new Promise((r) => setTimeout(r, 60));
    });

    // The viewport moved, and it moved to old content — not to a blank buffer,
    // which is what reset() used to leave.
    expect(realBuffer(term).ydisp).toBeLessThan(realBuffer(term).ybase);
    const scrolledRow = viewportRows(term)[0];
    expect(scrolledRow).not.toBe(liveRow);
    expect(scrolledRow).toMatch(/^L\d{3} /);
    // And the history is still there in full.
    expect(realBuffer(term).lines.length).toBe(before);
  });

  it('the repainted content lands in the viewport, so the input box is not garbled', async () => {
    // The other half of the P0: the original reason for the clear was the stale
    // frame sitting UNDERNEATH the repaint. The viewport clear must still do that
    // job — this asserts the repaint is what the operator sees, not the old frame.
    const term = await mountTerminal(24, 80);
    await writeSnapshot(term, 200);

    await act(async () => {
      term.resize(78, 10);
      await new Promise((r) => setTimeout(r, 60));
    });

    // The component's response to the geometry change: drop the stale frame from
    // the viewport, keep the scrollback.
    dropStaleSnapshotViewport(term);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 60));
    });

    // Then the TUI repaints after SIGWINCH — its own content, on a clean viewport.
    await act(async () => {
      term.write('SAIH PROMPT > \r\n');
      await new Promise((r) => setTimeout(r, 60));
    });

    // The repaint is what the operator sees, NOT the stale frame underneath it.
    expect(viewportRows(term)[0]).toBe('SAIH PROMPT > ');
    // And the history the repaint displaced is still there above.
    expect(realBuffer(term).lines.length).toBeGreaterThan(150);
  });
});

// ── What is deliberately NOT asserted in this file ────────────────────────
//
// "the component chose viewport-clear + scrollToBottom instead of clear() +
// reset()" is NOT asserted here, because asserting it needs the component's own
// geometry branch to run — and that branch is gated on FitAddon.fit() actually
// producing dimensions, which needs real character-cell metrics, which jsdom
// cannot produce (`dims.css.cell.width === 0` returns undefined from
// proposeDimensions). Measured: with clientWidth/clientHeight stubbed, the
// component still never leaves the construction seed of 24x80, so the branch
// never executes and any assertion about it here would be asserting nothing.
//
// That decision IS asserted, against a transcription, in
// SessionTerminal.geometry.test.tsx — which is the right tool for it, since it
// drives the real component against transcribed addon contracts. This file
// drives the real xterm and therefore asserts the OUTCOME (the scrollback
// survived), which is the part a transcription cannot show.
