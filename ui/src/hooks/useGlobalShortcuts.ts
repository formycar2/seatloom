import { useEffect } from 'react';

// Is this keystroke aimed at text the user is typing?
//
// THE BUG THIS GUARDS (2026-10-26, reported by the operator): on macOS Shift+/ has
// `event.key === '?'`, so the bare-`?` help shortcut below fired on EVERY question
// mark the user typed — `preventDefault()` ate the character and popped the help
// dialog over the field they were filling in. The `?` was unreachable in any input.
//
// The same shape applies to every shortcut this hook owns: a listener on `window`
// sees keystrokes that were addressed to a field. The rule is one line and it is
// about the TARGET, not the key: if the user is typing, the shortcut stands down.
//
// Three reasons a keystroke is "typing", each measured rather than assumed:
//
//   1. `isComposing` — an IME is mid-composition (pinyin). A keypress that selects
//      a candidate is not a command, and firing one here would insert a character
//      the user did not finish choosing.
//   2. an editable element — INPUT, TEXTAREA, or anything inside a contenteditable.
//      xterm's hidden helper textarea lands here too, so typing in the embedded
//      terminal is covered by the same rule.
//   3. `event.target` may be `window` or `document` itself (a synthetic dispatch, or
//      a key that reached no element) — not editable, so the shortcuts stay live.
//
// `isContentEditable` is deliberately OR'd with a `closest()` walk rather than used
// alone: a nested element inside a contenteditable host inherits editability in a
// real browser, and jsdom — the environment the tests run in — reports
// `isContentEditable === undefined` for every element, so the DOM-level check is
// what makes this branch observable at all. Neither alone is the whole truth.
const isTypingContext = (event: KeyboardEvent): boolean => {
  if (event.isComposing) return true;

  const target = event.target as HTMLElement | null;
  // `window` and `document` carry no tagName; they are not fields.
  if (!target || typeof target.tagName !== 'string') return false;

  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return true;

  return (
    target.isContentEditable === true ||
    (typeof target.closest === 'function' &&
      target.closest('[contenteditable]:not([contenteditable="false"])') !== null)
  );
};

export const useGlobalShortcuts = (
  onCloseDetail: () => void,
  onToggleTab: () => void,
  onToggleTerminal: () => void,
  onShowHelp: () => void,
  onOpenCommandBar: () => void,
) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Someone is typing: this keystroke belongs to them, not to the app.
      if (isTypingContext(event)) return;

      // Help: ? (Shift + /) or F1
      if ((event.key === '?' && event.shiftKey) || event.key === 'F1') {
        event.preventDefault();
        onShowHelp();
        return;
      }

      if (event.key === 'Escape') {
        onCloseDetail();
      }

      // Cmd/Ctrl + K: Supervisor Command Bar
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        onOpenCommandBar();
      }

      // Cmd/Ctrl + ` : Toggle Operator Console
      if ((event.metaKey || event.ctrlKey) && event.key === '`') {
        event.preventDefault();
        onToggleTerminal();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onCloseDetail, onToggleTab, onToggleTerminal, onShowHelp, onOpenCommandBar]);
};
