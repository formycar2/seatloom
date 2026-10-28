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
  /** Hold the startup query's promise so a subscription can fail first. */
  holdQuery: false,
  listenCalls: [] as string[],
  statusCalls: 0,
  /** Every unlisten fn handed to the app, so a leak is observable. */
  unlistenCalls: [] as string[],
  /** Resolve `listen` late, to test the unmount window. */
  deferDetached: false,
  deferredResolve: null as ((fn: () => void) => void) | null,
  /** Set while the startup query is being held, so the test can fail it later. */
  pendingQueryReject: null as ((e: unknown) => void) | null,
  /**
   * Hold the `supervisor:reembedded` SUBSCRIPTION's promise so the STARTUP QUERY
   * can be failed first and this subscription failed afterwards.
   *
   * This is the ordering that exposes the `prev?.kind` guard. It exists because
   * the app invokes the query before the two subscriptions, so in production the
   * query's rejection always lands first and the guard never fires — see the long
   * note at the guard in App.tsx. Holding the subscription is the only way to
   * make the worse failure arrive SECOND, which is exactly what F2 was.
   */
  holdReembedListen: false,
  pendingReembedReject: null as ((e: unknown) => void) | null,
}));

vi.mock('./lib/api', () => ({
  isTauri: () => true,
  api: {
    supervisorWindowStatus: async () => {
      state.statusCalls += 1;
      if (state.holdQuery) {
        // Never settles until the test flips holdQuery and lets the microtask
        // queue drain; the reject below is reached on the next call.
        return new Promise((_resolve, reject) => {
          state.pendingQueryReject = reject;
        });
      }
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
    if (state.holdReembedListen) {
      // Held until the test releases it: the mechanism that lets a subscription
      // fail AFTER the startup query has already failed.
      return new Promise((_resolve, reject) => {
        state.pendingReembedReject = reject;
      });
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
  state.holdQuery = false;
  state.pendingQueryReject = null;
  state.listenCalls = [];
  state.unlistenCalls = [];
  state.statusCalls = 0;
  state.deferDetached = false;
  state.deferredResolve = null;
  state.holdReembedListen = false;
  state.pendingReembedReject = null;
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

// ── BOTH failing at once (Flux's F2) ────────────────────────────────────────
//
// The two reporters used to write the same single slot, so the last writer won —
// and the subscription rejections land in a LATER microtask, so they always did.
// Flux measured the consequence: with the query AND the subscriptions failing,
// the operator saw `stale` only. They were told "you had an answer that went
// quiet" when the truth was "you never got one", the query's error text — the
// evidence they are asked to hand to Aegis — was gone, and the one useful action
// (the retry) was not offered.
//
// These tests pin all four combinations, because the defect only shows up in the
// fourth and a regression in any of the others would be just as invisible.
//
// The rule the fix follows, and the rule these tests enforce: the KIND is the
// WORST of what happened, and every REASON is kept. A failure that arrives
// second must neither overwrite the worse kind nor drop its own text.

/** Drive one combination through the real app and report what the strip says. */
async function observe({ queryFails, subsFail }: { queryFails: boolean; subsFail: boolean }) {
  state.rejectStatus = queryFails;
  state.rejectListenFor = subsFail ? ['supervisor:detached', 'supervisor:reembedded'] : [];
  await mountApp();
  const strip = await waitFor(() => {
    const el = screen.queryByTestId(STRIP);
    return el ?? null;
  });
  return {
    kind: strip?.getAttribute('data-kind') ?? null,
    text: strip?.textContent ?? '',
    hasRetry: screen.queryByRole('button', { name: /再问一次/ }) !== null,
  };
}

describe('the four failure combinations', () => {
  it('query OK + subscriptions OK -> no strip', async () => {
    const seen = await observe({ queryFails: false, subsFail: false });
    expect(seen.kind).toBeNull();
    expect(seen.hasRetry).toBe(false);
  });

  it('query OK + subscriptions fail -> yellow stale, no retry, the channel reasons', async () => {
    const seen = await observe({ queryFails: false, subsFail: true });
    expect(seen.kind).toBe('stale');
    expect(seen.hasRetry).toBe(false);
    expect(seen.text).toContain('event bus refused supervisor:detached');
    expect(seen.text).toContain('event bus refused supervisor:reembedded');
    // It must NOT claim the query failed — it did not.
    expect(seen.text).not.toContain('查询监督面板位置失败');
  });

  it('query fails + subscriptions OK -> red never-answered, with retry', async () => {
    const seen = await observe({ queryFails: true, subsFail: false });
    expect(seen.kind).toBe('never-answered');
    expect(seen.hasRetry).toBe(true);
    expect(seen.text).toContain('backend refused the status query');
  });

  it('query fails AND subscriptions fail -> the worse one wins, and nothing is lost', async () => {
    // THE F2 CASE. The subscriptions reject in a later microtask than the query,
    // so under the old single-slot design they overwrote it.
    const seen = await observe({ queryFails: true, subsFail: true });

    // 1. The WORSE failure decides the shape, even though it was reported first.
    expect(seen.kind).toBe('never-answered');
    // 2. The one useful action is offered.
    expect(seen.hasRetry).toBe(true);
    // 3. The query's error text survives — the evidence the operator is asked to
    //    hand over. Losing it is what made F2 more than cosmetic.
    expect(seen.text).toContain('backend refused the status query');
    // 4. The subscriptions' reasons survive too. Two failures, two pieces of
    //    evidence, and they are not interchangeable.
    expect(seen.text).toContain('event bus refused supervisor:detached');
    expect(seen.text).toContain('event bus refused supervisor:reembedded');
    // 5. The copy says BOTH are down rather than only naming the query's half.
    expect(seen.text).toContain('实时推送也接不上');
    // 6. And it must NOT fall back to the stale wording, which asserts an answer
    //    existed. There was none.
    expect(seen.text).not.toContain('启动时查到的那一次');
  });

  it('a subscription arriving second does not demote the kind', async () => {
    // The ordering half of F2, asserted directly rather than through the
    // combination: the query's rejection resolves first, so if severity were
    // decided by arrival order the shape would end up stale.
    const seen = await observe({ queryFails: true, subsFail: true });
    expect(seen.kind).toBe('never-answered');
    expect(seen.hasRetry).toBe(true);
  });
});

describe('the retry does not hide a failure it did not fix', () => {
  it('a successful query with subscriptions still down demotes to stale, keeping their reasons', async () => {
    // The mirror image of F2: the first version cleared the whole record on a
    // successful query, which would hide a live subscription failure behind an
    // unrelated success. Same disease, other direction.
    state.rejectStatus = true;
    state.rejectListenFor = ['supervisor:detached'];
    await mountApp();
    await waitFor(() =>
      expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('never-answered'),
    );

    // The query starts working; the subscription does not.
    state.rejectStatus = false;
    const retry = await screen.findByRole('button', { name: /再问一次/ });
    await act(async () => {
      retry.click();
    });

    await waitFor(() => expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('stale'));
    const text = screen.getByTestId(STRIP).textContent ?? '';
    // The query's failure is gone (it was fixed) ...
    expect(text).not.toContain('backend refused the status query');
    // ... and the subscription's is still there (it was not).
    expect(text).toContain('event bus refused supervisor:detached');
    // No retry button on the stale shape: re-reading cannot fix an event bus.
    expect(screen.queryByRole('button', { name: /再问一次/ })).toBeNull();
  });

  it('a successful query with everything else fine removes the strip entirely', async () => {
    state.rejectStatus = true;
    await mountApp();
    await screen.findByTestId(STRIP);

    state.rejectStatus = false;
    const retry = await screen.findByRole('button', { name: /再问一次/ });
    await act(async () => {
      retry.click();
    });

    await waitFor(() => expect(screen.queryByTestId(STRIP)).toBeNull());
  });
});

describe('the worse failure wins regardless of arrival order', () => {
  it('a subscription failing BEFORE the query does not steal the shape', async () => {
    // The M2 gap this test closes. Severity must be a function of WHAT failed,
    // not of who reported first. The mock above rejects both in the same tick,
    // which does not distinguish the two rules — so this one makes the
    // subscription reject strictly FIRST by holding the query back.
    //
    // A late-resolving query is the honest way to say "the query answered after
    // the subscriptions had already given up", which is the ordering the old
    // single-slot design got wrong.
    state.holdQuery = true;
    state.rejectListenFor = ['supervisor:detached'];

    await mountApp();

    // The subscription has already failed and reported, while the query is still
    // in flight and has answered nothing.
    await waitFor(() => expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('stale'));

    // Now the held query REJECTS. It must UPGRADE the shape rather than being
    // ignored — severity is about what failed, not who reported first.
    expect(state.pendingQueryReject).not.toBeNull();
    await act(async () => {
      state.pendingQueryReject!(new Error('backend refused the status query'));
      await new Promise((r) => setTimeout(r, 0));
    });

    await waitFor(() =>
      expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('never-answered'),
    );
    // Both reasons still present, and the retry is offered.
    const text = screen.getByTestId(STRIP).textContent ?? '';
    expect(text).toContain('event bus refused supervisor:detached');
    expect(screen.queryByRole('button', { name: /再问一次/ })).not.toBeNull();
  });

  it('a subscription failing AFTER the query does not demote the shape', async () => {
    // The mirror of the test above, and the ONLY thing that exercises the
    // `prev?.kind` half of the guard.
    //
    // Why this ordering does not happen in production: the app invokes
    // askSupervisorWindowStatus() BEFORE it registers the two subscriptions, so
    // the query's rejection always lands first. To make the worse failure arrive
    // SECOND — which is precisely what F2 was — the subscription has to be held
    // back, which is what `holdReembedListen` does. Without that switch this
    // guard is unreachable and no test in this suite touches it.
    //
    // What this pins: when the query has already said never-answered, a later
    // subscription failure must ADD its reason without DEMOTING the shape to
    // stale. The `prev?.kind === 'never-answered'` disjunct is what does that;
    // `failure.kind` alone would read 'stale' and overwrite.
    state.rejectStatus = true;
    state.holdReembedListen = true;

    await mountApp();

    // The query failed first: the red never-answered shape, retry offered.
    await waitFor(() =>
      expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('never-answered'),
    );

    // Now the held subscription rejects. It must not demote the shape.
    expect(state.pendingReembedReject).not.toBeNull();
    await act(async () => {
      state.pendingReembedReject!(new Error('event bus refused supervisor:reembedded'));
      await new Promise((r) => setTimeout(r, 0));
    });

    await waitFor(() =>
      expect(screen.getByTestId(STRIP).getAttribute('data-kind')).toBe('never-answered'),
    );
    // The subscription's reason is still recorded — the guard keeps the severity,
    // it does not throw the evidence away.
    const text = screen.getByTestId(STRIP).textContent ?? '';
    expect(text).toContain('event bus refused supervisor:reembedded');
    expect(screen.queryByRole('button', { name: /再问一次/ })).not.toBeNull();
  });
});
