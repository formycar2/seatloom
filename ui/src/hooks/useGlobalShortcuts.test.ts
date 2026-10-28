// useGlobalShortcuts — the guard that lets a `?` be a question mark.
//
// THE DEFECT, MEASURED FIRST (2026-10-26): the operator could not type `?` anywhere.
// `window`-level `keydown` handler + `event.key === '?'` = every Shift+/ opened the
// help dialog and `preventDefault()` swallowed the character. This file drives the
// REAL hook against a REAL input and REAL contenteditable nodes and asserts the
// character survives — because the operator's bug was invisible to every test that
// only checked "does the dialog open".
//
// The assertions are two-sided on purpose:
//   - focused in a field: NO shortcut fires, NO preventDefault, the key belongs to
//     the user. A guard that simply blocked all keyboard input would pass the first
//     half of these and break every text field in the app.
//   - focus elsewhere: every shortcut still fires exactly once. The bug is not fixed
//     by never opening help.
//
// The IME case (`isComposing`) is asserted too: on Chinese input a keypress during
// pinyin composition is a candidate selection, not a command, and firing one here
// both fires the shortcut and corrupts the composition.
//
// `preventDefault` is watched through the real event object: a listener installed
// during capture (bubble phase) can call it before the field ever sees the key, so
// "did the handler call preventDefault" is the thing that actually decides whether
// the character lands — not merely whether a callback ran.

import { describe, expect, it, vi, afterEach } from 'vitest';
import { renderHook, act, cleanup } from '@testing-library/react';
import { useGlobalShortcuts } from './useGlobalShortcuts';

interface Handlers {
  onCloseDetail: ReturnType<typeof vi.fn>;
  onToggleTab: ReturnType<typeof vi.fn>;
  onToggleTerminal: ReturnType<typeof vi.fn>;
  onShowHelp: ReturnType<typeof vi.fn>;
  onOpenCommandBar: ReturnType<typeof vi.fn>;
}

const handlers = (): Handlers => ({
  onCloseDetail: vi.fn(),
  onToggleTab: vi.fn(),
  onToggleTerminal: vi.fn(),
  onShowHelp: vi.fn(),
  onOpenCommandBar: vi.fn(),
});

/** Mount the real hook so its effect installs the real window listener. */
const mount = (h: Handlers) => {
  renderHook(() =>
    useGlobalShortcuts(
      h.onCloseDetail,
      h.onToggleTab,
      h.onToggleTerminal,
      h.onShowHelp,
      h.onOpenCommandBar,
    ),
  );
};

/** A keydown, dispatched on a real target, with preventDefault observed. */
const press = (init: KeyboardEventInit, target: EventTarget) => {
  const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
};

/** A plain keydown on window — focus nowhere in particular. */
const pressNowhere = (init: KeyboardEventInit) => press(init, window);

afterEach(() => {
  cleanup();
});

describe('while the user is typing, the shortcut stands down', () => {
  it('Shift+/ in a text input leaves the ? to the input and opens no help', () => {
    const h = handlers();
    mount(h);
    const input = document.createElement('input');
    document.body.appendChild(input);
    input.focus();

    // macOS reports Shift+/ as key '?'; this is the operator's exact keypress.
    const event = press({ key: '?', shiftKey: true }, input);

    expect(h.onShowHelp).not.toHaveBeenCalled();
    // preventDefault is what actually eats the character; asserting the callback
    // alone would let a "silently blocked but not prevented" version pass.
    expect(event.defaultPrevented).toBe(false);
    input.remove();
  });

  it('every other shortcut also stands down while typing, not just ?', () => {
    const h = handlers();
    mount(h);
    const textarea = document.createElement('textarea');
    document.body.appendChild(textarea);
    textarea.focus();

    // Cmd+K is the second reported half of the bug: it is a modifier chord, so it
    // produces no character — but it still stole focus into the command bar while
    // the user was mid-sentence. The target rule covers both, by construction.
    const cmdK = press({ key: 'k', metaKey: true }, textarea);
    const backtick = press({ key: '`', metaKey: true }, textarea);
    const f1 = press({ key: 'F1' }, textarea);
    const esc = press({ key: 'Escape' }, textarea);

    expect(h.onOpenCommandBar).not.toHaveBeenCalled();
    expect(h.onToggleTerminal).not.toHaveBeenCalled();
    expect(h.onShowHelp).not.toHaveBeenCalled();
    expect(h.onCloseDetail).not.toHaveBeenCalled();
    expect(cmdK.defaultPrevented).toBe(false);
    expect(backtick.defaultPrevented).toBe(false);
    expect(f1.defaultPrevented).toBe(false);
    expect(esc.defaultPrevented).toBe(false);
    textarea.remove();
  });

  it('a keystroke inside a contenteditable host is typing too', () => {
    const h = handlers();
    mount(h);
    const host = document.createElement('div');
    host.setAttribute('contenteditable', 'true');
    const leaf = document.createElement('span');
    host.appendChild(leaf);
    document.body.appendChild(host);
    leaf.focus();

    // jsdom reports isContentEditable as undefined, so this case also proves the
    // closest()-walk branch of the guard — the part a real browser inherits.
    const event = press({ key: '?', shiftKey: true }, leaf);

    expect(h.onShowHelp).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
    host.remove();
  });

  it('contenteditable="false" is not a typing context', () => {
    const h = handlers();
    mount(h);
    const off = document.createElement('div');
    off.setAttribute('contenteditable', 'false');
    document.body.appendChild(off);
    off.focus();

    // A subtree explicitly switched back off must not disable the shortcuts: the
    // `:not([contenteditable="false"])` half of the selector, not a naive match.
    const event = press({ key: '?', shiftKey: true }, off);

    expect(h.onShowHelp).toHaveBeenCalledTimes(1);
    expect(event.defaultPrevented).toBe(true);
    off.remove();
  });

  it('an IME composition in progress does not trigger any shortcut', () => {
    const h = handlers();
    mount(h);

    // Deliberately dispatched on a NON-editable target: during pinyin composition the
    // focused field can be unmounted (a candidate pick re-renders the composer), and
    // the remaining keydowns land on body. That is the case where `isComposing` is
    // the ONLY thing standing the shortcut down — with an <input> target the
    // editable-element branch would answer first and this test would prove nothing.
    const composingQ = press({ key: '?', shiftKey: true, isComposing: true }, document.body);
    const composingCmdK = press({ key: 'k', metaKey: true, isComposing: true }, document.body);

    expect(h.onShowHelp).not.toHaveBeenCalled();
    expect(h.onOpenCommandBar).not.toHaveBeenCalled();
    expect(composingQ.defaultPrevented).toBe(false);
    expect(composingCmdK.defaultPrevented).toBe(false);
  });

  it('a composing keypress aimed at a field is typing by both rules at once', () => {
    const h = handlers();
    mount(h);
    const input = document.createElement('input');
    document.body.appendChild(input);
    input.focus();

    const composing = press({ key: '?', shiftKey: true, isComposing: true }, input);

    expect(h.onShowHelp).not.toHaveBeenCalled();
    expect(composing.defaultPrevented).toBe(false);
    input.remove();
  });

  it('a key that reached no element at all still arms the shortcuts', () => {
    const h = handlers();
    mount(h);

    // event.target === window here — no tagName, not a field. The guard must not
    // mistake "nowhere" for "typing", or the shortcuts would never fire.
    const event = pressNowhere({ key: '?', shiftKey: true });

    expect(h.onShowHelp).toHaveBeenCalledTimes(1);
    expect(event.defaultPrevented).toBe(true);
  });
});

describe('with focus outside any field, the shortcuts are unchanged', () => {
  it('? opens help and F1 still opens help', () => {
    const h = handlers();
    mount(h);

    const q = pressNowhere({ key: '?', shiftKey: true });
    const f1 = pressNowhere({ key: 'F1' });

    expect(h.onShowHelp).toHaveBeenCalledTimes(2);
    expect(q.defaultPrevented).toBe(true);
    expect(f1.defaultPrevented).toBe(true);
  });

  it('Cmd+K opens the command bar and Cmd+` toggles the terminal', () => {
    const h = handlers();
    mount(h);

    const cmdK = pressNowhere({ key: 'k', metaKey: true });
    const backtick = pressNowhere({ key: '`', metaKey: true });

    expect(h.onOpenCommandBar).toHaveBeenCalledTimes(1);
    expect(h.onToggleTerminal).toHaveBeenCalledTimes(1);
    expect(cmdK.defaultPrevented).toBe(true);
    expect(backtick.defaultPrevented).toBe(true);
  });

  it('Escape closes the detail pane', () => {
    const h = handlers();
    mount(h);

    pressNowhere({ key: 'Escape' });

    expect(h.onCloseDetail).toHaveBeenCalledTimes(1);
  });

  it('a plain character key does nothing outside a field', () => {
    const h = handlers();
    mount(h);

    // The negative case that keeps the guard honest: standing down for typing must
    // not turn into swallowing every key.
    pressNowhere({ key: 'a' });

    expect(h.onShowHelp).not.toHaveBeenCalled();
    expect(h.onOpenCommandBar).not.toHaveBeenCalled();
    expect(h.onToggleTerminal).not.toHaveBeenCalled();
    expect(h.onCloseDetail).not.toHaveBeenCalled();
  });

  it('Ctrl chords work too, not just Cmd', () => {
    const h = handlers();
    mount(h);

    pressNowhere({ key: 'K', ctrlKey: true });

    // The chord is matched case-insensitively, so the Linux/Windows path is live.
    expect(h.onOpenCommandBar).toHaveBeenCalledTimes(1);
  });
});

describe('the listener is cleaned up', () => {
  it('unmounting stops the shortcuts', () => {
    const h = handlers();
    const view = renderHook(() =>
      useGlobalShortcuts(
        h.onCloseDetail,
        h.onToggleTab,
        h.onToggleTerminal,
        h.onShowHelp,
        h.onOpenCommandBar,
      ),
    );

    act(() => view.unmount());
    pressNowhere({ key: '?', shiftKey: true });

    // Without the cleanup, a remount leaves a duplicate listener behind and help
    // opens twice per keypress.
    expect(h.onShowHelp).not.toHaveBeenCalled();
  });
});
