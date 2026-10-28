// Terminal links must actually open — the operator's 2026-10-27 bug, pinned.
//
// THE BUG: `new WebLinksAddon()` was loaded with no handler, so it fell back to
// xterm's default, which is `window.open()`. Tauri's webview blocks window.open
// by design, so the addon still recognised the URL and still received the click,
// but the open was a no-op that only logged a warning. A link in a live-seats
// pane looked clickable and did nothing.
//
// THE FIX: pass a handler that calls `api.openExternalUrl`, which routes to a
// Rust command that shells out to the platform opener — the same mechanism
// `cmd_open_local_path` already uses. (Not @tauri-apps/plugin-shell's JS open():
// that JS dep is declared in package.json but the Rust plugin is registered
// nowhere, so using it would mean adding a Cargo dep, a capability, and a second
// way to open things.)
//
// HOW THIS IS TESTED, and what jsdom can and cannot do here:
//
// jsdom performs no layout, so xterm's DOM renderer cannot compute the pixel
// range of a link and a synthesised click cannot land on one. The link range is
// therefore NOT exercised. What IS exercised, and is the part that was broken:
// that a handler exists at all, that it returns true (so the addon does not also
// run its own default), and that it routes the URL to openExternalUrl. Those are
// asserted against the REAL WebLinksAddon instance the component loaded — the
// handler is read off the addon object, not re-created in the test.
//
// The default-handler path (window.open, the actual bug) is asserted as a
// separate test that runs the real addon's own default, so the contrast is
// measured rather than described in a comment.

import '../../test/installLocalStorage';
import { describe, expect, it, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { act, cleanup, render } from '@testing-library/react';
import React from 'react';

// xterm's `open` needs matchMedia; the DOM renderer needs a canvas 2d context.
// jsdom has neither. Nothing here asserts anything about rendering — only about
// the handler wiring — which is stated so the canvas stub cannot be mistaken for
// "we tested the visuals".
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
      openExternalUrl: vi.fn(async () => {}),
      ptyWriteBytes: vi.fn(async () => {}),
    },
  };
});

import { SessionTerminal } from './SessionTerminal';
import { Terminal } from '@xterm/xterm';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { api } from '../../lib/api';

/**
 * Mount the component and return the REAL WebLinksAddon instance it loaded.
 *
 * `loadAddon` is spied so the addon object itself is captured — reading the
 * handler off the addon is what makes this a test of the wiring rather than a
 * restatement of it: the handler the component passed in is the one asserted on.
 */
async function mountAndCaptureWebLinksAddon() {
  const loadSpy = vi.spyOn(Terminal.prototype, 'loadAddon');
  await act(async () => {
    render(
      <SessionTerminal sessionId="sl-live-QL" paneRows={24} paneCols={80} initialSnapshotB64="" />,
    );
  });
  await act(async () => {
    await new Promise((r) => setTimeout(r, 0));
  });

  let captured: WebLinksAddon | null = null;
  for (const call of loadSpy.mock.calls) {
    const arg = call[0] as unknown;
    if (arg instanceof WebLinksAddon) captured = arg;
  }
  loadSpy.mockRestore();
  return captured as unknown as WebLinksAddon & { _handler: unknown };
}

beforeEach(() => {
  (globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = StubResizeObserver;
  vi.mocked(api.openExternalUrl).mockClear();
});

afterEach(() => {
  cleanup();
});

describe('a URL clicked in a live-seats pane opens in the browser', () => {
  it('the addon is loaded WITH a handler', async () => {
    // The regression this pins: `new WebLinksAddon()` with no argument. Without a
    // handler the addon's default is window.open, which Tauri blocks — the exact
    // no-op the operator reported.
    const addon = await mountAndCaptureWebLinksAddon();
    expect(addon).not.toBeNull();
    expect(typeof addon._handler).toBe('function');
  });

  it('the handler routes the URL to openExternalUrl', async () => {
    const addon = await mountAndCaptureWebLinksAddon();
    const handler = addon._handler as (event: MouseEvent, uri: string) => boolean;

    let returned: boolean | undefined;
    await act(async () => {
      returned = handler(new MouseEvent('click'), 'https://github.com/seatloom');
      await new Promise((r) => setTimeout(r, 40));
    });

    expect(vi.mocked(api.openExternalUrl).mock.calls.map((c) => c[0])).toContain(
      'https://github.com/seatloom',
    );
    // Returning true tells the addon we took responsibility, so it does not ALSO
    // run its own default — which is the window.open that does nothing here.
    expect(returned).toBe(true);
  });

  it('a rejected open is reported, not swallowed', async () => {
    // The Rust command refuses non-http(s) schemes with an error naming the
    // scheme. Swallowing that would recreate the original bug's worst property:
    // clicking does nothing and nothing says so.
    vi.mocked(api.openExternalUrl).mockRejectedValueOnce(
      new Error('refused to open file:///etc/passwd: only http and https are opened'),
    );
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

    const addon = await mountAndCaptureWebLinksAddon();
    const handler = addon._handler as (event: MouseEvent, uri: string) => boolean;

    await act(async () => {
      handler(new MouseEvent('click'), 'file:///etc/passwd');
      await new Promise((r) => setTimeout(r, 40));
    });

    // The handler still returns true (it took the click), and the failure is
    // logged with the URL attached rather than dropped.
    expect(consoleError).toHaveBeenCalled();
    const logged = consoleError.mock.calls.flat().join(' ');
    expect(logged).toContain('openExternalUrl failed');
    expect(logged).toContain('file:///etc/passwd');
    consoleError.mockRestore();
  });

  it('the unhandled default really is window.open — the bug, measured', async () => {
    // Contrast case, run against the REAL addon: constructed with no handler, its
    // stored handler is the xterm default. This is what the component used to do,
    // asserted rather than described, so nobody has to take the comment's word for
    // what the default was.
    const bare = new WebLinksAddon() as unknown as { _handler: unknown };
    expect(typeof bare._handler).toBe('function');
    // The default handler returns null after calling window.open (per
    // @xterm/addon-web-links@0.11.0), and window.open returns null in a Tauri
    // webview. A spy proves the default reaches window.open at all.
    const openSpy = vi.spyOn(window, 'open').mockReturnValue(null);
    const ret = (bare._handler as (e: MouseEvent, u: string) => unknown)(
      new MouseEvent('click'),
      'https://example.com',
    );
    expect(openSpy).toHaveBeenCalled();
    openSpy.mockRestore();
    // FALSY, i.e. "I did not handle it" — which is why our handler must return
    // true. The value is `undefined`, not null: the default handler is
    // `function i(e,t){const n=window.open(); if(n){...} else console.warn(...)}`
    // with no return statement at all. (The dispatch described it as "returns
    // null"; measured, it returns undefined. Same consequence — falsy — but the
    // assertion states what the code does.)
    expect(ret).toBeFalsy();
  });
});
