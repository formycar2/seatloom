// App's supervisor subscriptions — the failure path, exercised.
//
// WHAT WAS WRONG. `App.tsx` subscribes to `supervisor:detached` and
// `supervisor:reembedded` so the embedded panel and ⌘K follow the panel as the
// operator moves it. Neither subscription had a failure handler: a rejecting
// `listen()` was an unhandled rejection, and — the part that actually matters —
// the state they maintain silently froze at whatever the startup query answered.
// `supervisorDetached === false` then read as "the panel is in this window" when
// the truth had become "I do not know any more".
//
// The neighbouring `supervisorWindowStatus()` call handles its failure with
// `.catch(() => {})`. That is the shape this file exists to REJECT: swallowing
// converts "an error happened" into "nothing happened and the state is real".
// Here the state is load-bearing — it decides whether ⌘K opens a second copy of
// the panel next to the detached one — so the failure is surfaced instead, with
// the concrete consequence named.
//
// WHAT THIS FILE PROVES
//   1. a rejecting `listen()` produces a stated failure, not an unhandled
//      rejection and not a silent one;
//   2. BOTH subscriptions are covered (the first version of the fix could easily
//      have covered one and not the other — they are two separate call sites);
//   3. the non-failure path stays silent, so the strip cannot become noise;
//   4. a subscription that resolves AFTER unmount is dropped rather than leaked
//      — the in-flight window the cleanup used to miss.
//
// The mock is `../lib/api`, so `onSupervisorDetached`/`onSupervisorReembedded`
// are injected directly rather than driven through Tauri's `listen()`. That is
// the narrowest seam: what is under test is App's reaction to a rejecting
// subscription, not Tauri's transport.

import './test/installLocalStorage';
import { describe, expect, it, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

// ── The injected api surface ────────────────────────────────────────────────
//
// `rejectListenFor` names the events whose subscription must reject, and
// `rejectStatus` makes the STARTUP QUERY reject. Everything else resolves, so a
// test states exactly which of the three reads is broken.
const state = vi.hoisted(() => ({
  rejectListenFor: [] as string[],
  rejectStatus: false,
  listenCalls: [] as string[],
  statusCalls: 0,
  /** Every unlisten fn handed to the app, so a leak is observable. */
  unlistenCalls: [] as string[],
  /** Resolve `listen` late, to test the unmount window. */
  deferDetached: false,
  deferredResolve: null as ((fn: () => void) => void) | null,
}));

vi.mock('./lib/api', () => ({
  isTauri: () => true,
  api: {
    supervisorWindowStatus: async () => {
      state.statusCalls += 1;
      if (state.rejectStatus) {
        return Promise.reject(new Error('backend refused the status query'));
      }
      return { detached: false };
    },
    openSupervisorWindow: async () => {},
    closeSupervisorWindow: async () => {},
  },
  onSupervisorDetached: (_h: () => void) => {
    state.listenCalls.push('supervisor:detached');
    if (state.rejectListenFor.includes('supervisor:detached')) {
      return Promise.reject(new Error('event bus refused supervisor:detached'));
    }
    if (state.deferDetached) {
      return new Promise((resolve) => {
        // Resolving with a RECORDING unlisten fn, so "was the late subscription
        // dropped" is observable rather than assumed. The previous version
        // resolved with a no-op, which made this test pass for the wrong reason.
        state.deferredResolve = () =>
          resolve(() => {
            state.unlistenCalls.push('supervisor:detached');
          });
      });
    }
    return Promise.resolve(() => {
      state.unlistenCalls.push('supervisor:detached');
    });
  },
  onSupervisorReembedded: (_h: () => void) => {
    state.listenCalls.push('supervisor:reembedded');
    if (state.rejectListenFor.includes('supervisor:reembedded')) {
      return Promise.reject(new Error('event bus refused supervisor:reembedded'));
    }
    return Promise.resolve(() => {
      state.unlistenCalls.push('supervisor:reembedded');
    });
  },
}));

// The panes need xterm and a canvas; jsdom has neither.
vi.mock('../app-v2/panel/SessionTerminal', () => ({
  SessionTerminal: () => <div data-testid="terminal" />,
}));

import App from './App';
import { useDataStore } from './stores/useDataStore';

const STRIP = 'supervisor-sync-lost';

// The stores detect Tauri through `window.__TAURI_INTERNALS__` (api.ts's isTauri),
// NOT through the api module — so mocking the module alone leaves every store on
// the browser path, where `hydrateFromBackend` records "not in the desktop app"
// and returns before touching anything. Setting this is what makes the app take
// the Tauri branch that the supervisor subscription lives behind.
beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
    invoke: async (cmd: string) => {
      throw new Error(`unmocked Tauri command: ${cmd}`);
    },
    transformCallback: () => 1,
  };
});

beforeEach(() => {
  window.localStorage.clear();
  state.rejectListenFor = [];
  state.rejectStatus = false;
  state.listenCalls = [];
  state.unlistenCalls = [];
  state.statusCalls = 0;
  state.deferDetached = false;
  state.deferredResolve = null;
  useDataStore.setState({ activeProjectId: null, backendHydrated: false });
});

afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

/** Mount the real app and let its mount effects settle. */
async function mountApp() {
  await act(async () => {
    render(<App />);
  });
}

describe('when the supervisor subscription is refused', () => {
  it('the detached subscription being refused is stated, not swallowed', async () => {
    state.rejectListenFor = ['supervisor:detached'];

    await mountApp();

    const strip = await screen.findByTestId(STRIP);
    // The failure is named, with the reason the backend actually returned — not
    // an empty catch that leaves the operator guessing.
    expect(strip.textContent).toContain('event bus refused supervisor:detached');
    // ...and the consequence is stated rather than implied, because the state it
    // was maintaining is now stale and that has a visible effect on ⌘K.
    expect(strip.textContent).toContain('不会再变');
    expect(strip.textContent).toContain('⌘K');
    // The startup query still ran: the strip says "this is the startup answer",
    // which is only true if there was one.
    expect(state.statusCalls).toBe(1);
  });

  it('the reembedded subscription being refused is stated too', async () => {
    // Two separate call sites. A fix that covered the first and not the second
    // is the failure this test exists to catch.
    state.rejectListenFor = ['supervisor:reembedded'];

    await mountApp();

    const strip = await screen.findByTestId(STRIP);
    expect(strip.textContent).toContain('event bus refused supervisor:reembedded');
  });

  it('both subscriptions are attempted, so neither can be the untested one', async () => {
    state.rejectListenFor = ['supervisor:detached', 'supervisor:reembedded'];

    await mountApp();

    await screen.findByTestId(STRIP);
    expect(state.listenCalls).toContain('supervisor:detached');
    expect(state.listenCalls).toContain('supervisor:reembedded');
    // The first refusal already set the message; the second must not be lost —
    // it names its own channel, so a reader can tell both are down.
    const strip = screen.getByTestId(STRIP);
    expect(strip.textContent).toContain('supervisor:reembedded');
  });

  it('the failure is not an unhandled rejection', async () => {
    // The original defect's headline symptom. Vitest reports unhandled
    // rejections in the run's Errors count, so this file's contribution to that
    // count must be zero — the same standard the rest of the suite is held to.
    const seen: unknown[] = [];
    const onUnhandled = (e: PromiseRejectionEvent) => seen.push(e.reason);
    window.addEventListener('unhandledrejection', onUnhandled);
    state.rejectListenFor = ['supervisor:detached', 'supervisor:reembedded'];

    try {
      await mountApp();
      await screen.findByTestId(STRIP);
      // Let the microtask queue drain: a rejection handled in a `.catch` never
      // reaches the window event, and one that is not surfaces here.
      await new Promise((r) => setTimeout(r, 0));
    } finally {
      window.removeEventListener('unhandledrejection', onUnhandled);
    }
    expect(seen).toEqual([]);
  });
});

describe('when the subscriptions are accepted', () => {
  it('nothing is stated — the strip must not become noise', async () => {
    await mountApp();

    // Give the subscriptions every chance to resolve before asserting silence.
    await waitFor(() => expect(state.listenCalls.length).toBe(2));
    expect(screen.queryByTestId(STRIP)).toBeNull();
  });

  it('the startup status is still read and applied', async () => {
    await mountApp();
    await waitFor(() => expect(state.statusCalls).toBe(1));
    // `detached: false` from the mock, so the embedded panel's precondition holds.
    expect(screen.queryByTestId(STRIP)).toBeNull();
  });
});

describe('the cleanup still stands', () => {
  it('unmounting calls the unlisten fns it was handed', async () => {
    let view: { unmount: () => void };
    await act(async () => {
      view = render(<App />);
    });
    await waitFor(() => expect(state.listenCalls.length).toBe(2));
    expect(state.unlistenCalls).toEqual([]);

    await act(async () => {
      view!.unmount();
    });

    // Both channels were open, so both must be closed. The guard `if (offDetached)`
    // is what makes this work, and a regression to a plain call would throw here.
    expect(state.unlistenCalls).toContain('supervisor:detached');
    expect(state.unlistenCalls).toContain('supervisor:reembedded');
  });

  it('a subscription that lands after unmount is dropped, not leaked', async () => {
    // The in-flight window the old cleanup missed: `listen()` still pending when
    // the component goes away. Resolving afterwards used to assign the unlisten
    // fn into a closure nobody would read again, so the listener stayed
    // registered for the life of the page.
    let view: { unmount: () => void };
    state.deferDetached = true;

    await act(async () => {
      view = render(<App />);
    });
    await waitFor(() => expect(state.listenCalls).toContain('supervisor:detached'));
    expect(state.unlistenCalls).toEqual([]);

    await act(async () => {
      view!.unmount();
    });

    // Now the deferred subscription resolves. The handler must notice the effect
    // is gone and unsubscribe immediately instead of storing the fn.
    await act(async () => {
      state.deferredResolve!(() => {});
      await new Promise((r) => setTimeout(r, 0));
    });

    // Both are accounted for, by different routes: reembedded's unlisten was
    // already assigned when unmount ran, so the cleanup called it; detached's
    // landed afterwards and was dropped by the  guard. What must NOT
    // happen is the old behaviour — the late fn stored in a closure nobody reads,
    // leaving the listener registered for the life of the page.
    expect(state.unlistenCalls).toContain('supervisor:detached');
    expect(state.unlistenCalls).toContain('supervisor:reembedded');
  });
});

// ── The STARTUP QUERY's failure path (Flux's F1) ────────────────────────────
//
// The subscription half above was the original dispatch. Flux, verifying it,
// measured the neighbouring path and found it still swallowed:
// `shownWhenQueryFails=false, shownWhenSubFails=true` — not inferred, measured
// by injecting a rejecting `supervisorWindowStatus()` and asserting its reason
// never reached the screen.
//
// It is the SAME confident lie through a different door, and strictly worse:
// this is the FIRST read, so `supervisorDetached` had never been answered at
// all. A stale answer was once right; this one never was.
//
// Why a SEPARATE strip shape rather than reusing the subscription's:
//   - 'never-answered' gets a RETRY button, because re-reading is what fixes a
//     single failed query — the same reason the database read offers one;
//   - 'stale' gets none, because re-subscribing against a broken event bus
//     produces noise and no state.
// Collapsing them would tell the operator "something is wrong" without telling
// them whether there is anything they can do about it, and the actionable half
// is the part that differs.

describe('when the startup query is refused', () => {
  it('the failure is stated, not swallowed, and the state is not left claiming false', async () => {
    state.rejectStatus = true;

    await mountApp();

    const strip = await screen.findByTestId(STRIP);
    // The never-answered shape, not the stale one: this is the first read.
    expect(strip.getAttribute('data-kind')).toBe('never-answered');
    // The backend's error verbatim — not an empty catch that leaves the operator
    // guessing what the desktop said.
    expect(strip.textContent).toContain('backend refused the status query');
    // ...and the wording says the state was NEVER answered, which is the
    // difference from the subscription case. A message that only said
    // "stopped being maintained" would be wrong here: there was nothing to
    // maintain yet.
    expect(strip.textContent).toContain('没问到');
    expect(strip.textContent).toContain('不知道');
    // It must NOT claim the startup answer is what is being shown — there was
    // none. This is the assertion that distinguishes the two shapes.
    expect(strip.textContent).not.toContain('启动时查到的那一次');
    // The consequence is still stated, because the frozen false still drives
    // ⌘K and the embedded-panel gate.
    expect(strip.textContent).toContain('⌘K');
  });

  it('the failure is not an unhandled rejection', async () => {
    const seen: unknown[] = [];
    const onUnhandled = (e: PromiseRejectionEvent) => seen.push(e.reason);
    window.addEventListener('unhandledrejection', onUnhandled);
    state.rejectStatus = true;

    try {
      await mountApp();
      await screen.findByTestId(STRIP);
      await new Promise((r) => setTimeout(r, 0));
    } finally {
      window.removeEventListener('unhandledrejection', onUnhandled);
    }
    expect(seen).toEqual([]);
  });

  it('the retry button re-asks, and succeeding removes the strip', async () => {
    // The retry is only honest if it actually works: a button that re-asks and
    // leaves the warning up would be a lie with a click handler.
    state.rejectStatus = true;

    await mountApp();
    await screen.findByTestId(STRIP);
    expect(state.statusCalls).toBe(1);

    // The transient failure clears — the operator's database came back.
    state.rejectStatus = false;
    const retry = await screen.findByRole('button', { name: /再问一次/ });
    await act(async () => {
      retry.click();
    });

    await waitFor(() => expect(state.statusCalls).toBe(2));
    await waitFor(() => expect(screen.queryByTestId(STRIP)).toBeNull());
  });

  it('the stale shape has no retry button, because re-subscribing cannot fix it', async () => {
    // The other half of the distinction, asserted from the other side: offering
    // "再问一次" on a broken event bus would be a button that cannot work.
    state.rejectListenFor = ['supervisor:detached'];

    await mountApp();

    const strip = await screen.findByTestId(STRIP);
    expect(strip.getAttribute('data-kind')).toBe('stale');
    expect(screen.queryByRole('button', { name: /再问一次/ })).toBeNull();
  });
});
