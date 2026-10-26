# Flux verdict — Mira's shift-character fix, judged by mutation

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-26-verify-shift-fix-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-26 |
| version | v1 |
| depends_on | AEGIS-2026-10-26-fix-shift-shortcut-eats-input-v1 |
| tags | verification, mutation-testing, keyboard-shortcuts, seat-flux |

| Field | Value |
|---|---|
| id | FLUX-2026-10-26-verify-shift-fix-v1 |
| verdict | **PASS** — the fix is real and its tests are load-bearing; one narrow **finding** (F1) |
| verifies_dispatch | `disp-fCOD_8QOby_Bi20NfrsyR` (Mira, implementation) |
| verifying_dispatch | `disp-E57bNvMK5uTMgrH7tg2cK` (Aegis → Flux) |
| packet | `AEGIS-2026-10-26-fix-shift-shortcut-eats-input-v1` |
| subject_commit | `e7e327e` on `packet/mira/knowledge-panel` |
| verdict_date | 2026-10-26 |
| method | mutation of the subject's source + my own independent behavioural harness |

## How I verified (and why this is not a re-run of her tests)

Aegis ran the 13 tests and they pass. That proves the tests are green, not that they
measure the bug. I verified three things she did not, in an isolated worktree at
`e7e327e` (`git worktree add --detach`), never touching her tree:

1. **Six mutations of her source**, each made with the `edit` tool (fragment + diff +
   revert), each followed by the full 13-test run.
2. **16 independent behaviour checks of my own** — driving the real hook through a real
   DOM, asserting `defaultPrevented` and callback counts, deliberately *not* reusing
   her test file, so a shared blind spot cannot hide in both.
3. **Full-suite baseline comparison** at the parent commit, plus a wiring trace from
   `App.tsx` to the rendered dialog.

## 1. Mutation table — the tests are load-bearing

Every mutation below was reverted; the tree was confirmed byte-identical to `e7e327e`
afterwards (`git diff --quiet e7e327e` → clean), and Mira's own worktree was never
modified (`git status` empty).

| # | Mutation | Result | Reads as |
|---|---|---|---|
| M1 | `if (isTypingContext(event)) return;` removed entirely | **RED, 5 of 13** | the guard is not decorative |
| M2 | `isComposing` → `return false` | **RED, 2 of 13** | the IME branch is covered |
| M3 | `TEXTAREA` dropped from the tagName check | **RED, 1 of 13** | textarea covered |
| M4 | `closest()` walk dropped, `isContentEditable` alone | **RED, 1 of 13** | the contenteditable host branch covered |
| M5 | guard `return true` before everything (the lazy fix) | **RED, 6 of 13** | the fix is *not* "just disable all shortcuts" |
| M6 | the `?` branch deleted outright (the other lazy fix) | **RED, 3 of 13** | the help feature is still tested for existing |

**M5 and M6 are the ones the packet asked me to check specifically.** The packet warned
against the lazy shortcut-deletion fix and asked me to confirm the tests would
catch it. They
do: a guard that stands down unconditionally fails 6 tests, and deleting `?` fails 3.
This is a real fix with real assertions, not a green veneer.

I also reproduced her M4 self-finding (first IME test dispatched on `<input>`, so the
INPUT branch answered first and the `isComposing` mutation stayed green). Her account of
it is accurate, including the fix of dispatching on `document.body`.

## 2. My own 16 behaviour checks — all pass

Driving the real hook, real `KeyboardEvent`, real focused DOM nodes:

- **Behaviour 1 (the operator's bug)**: `Shift+/` with focus in a text input →
  `onShowHelp` not called, `defaultPrevented === false`, and the event demonstrably
  reaches the field. The identical keypress on `window` → help opens, `defaultPrevented === true`.
- **Behaviour 2**: `F1` with focus nowhere → opens; `F1` with focus in an input → stands down.
- **Behaviour 3**: `Cmd+K` and `` Cmd+` `` both fire with focus nowhere; **Ctrl variants
  also fire** (the `toLowerCase()` chord), covering the Linux/Windows path.
- **Behaviour 4**: IME — composing `?` on `document.body` fires nothing; composing
  `Cmd+K` on body fires nothing; `isComposing: false` re-arms the shortcuts.
- **No shortcut deleted**: `?`(shift) + `F1` → help ×2; `Cmd+K`, `` Cmd+` ``, `Escape`
  each once.
- **Edge cases I added, measured, and report below**: `<select>`, `<button>`,
  `input[type=checkbox]`, `input[readonly]`, hidden xterm-style `textarea`.

One correction to my own harness, recorded for honesty: I initially asserted help fires
3× for `['?', '?shift', F1]` and it fired 2×. The code is right and my expectation was
wrong — a synthetic `key: '?'` *without* `shiftKey` does not match
`event.key === '?' && event.shiftKey`, which is correct behaviour. I fixed the
expectation, not the source.

## 3. Full suite — baseline comparison, not just the absolute number

| | Test files | Tests | Unhandled errors |
|---|---|---|---|
| Parent `b77bd2a` | 21 passed | 251 passed | **14** |
| Subject `e7e327e` | 22 passed | **264 passed** | **14** |

`+1 file, +13 tests`, error count unchanged. `pnpm typecheck` clean. I reproduced the
14 unhandled errors and traced them to `src/App.restartRestore.test.tsx:84` — a
deliberate `unmocked Tauri invoke` throw inside a mock, surfacing via
`api.ts:299/303`. **Pre-existing, identical before and after, not introduced by this
fix.** Mira's delivery note says the same thing and declines to fix it as out of scope,
which I agree with: folding an unrelated cleanup into a bug fix is how scope gets lost.

## 4. Wiring — the fix actually reaches the operator

`useGlobalShortcuts` is called at exactly one place, `ui/src/App.tsx:236`, and that call
site is **identical at the parent and at `e7e327e`** — the fix changed the guard, not the
wiring. The chain is observable end to end:

```
App.tsx:236  useGlobalShortcuts(...)
   → () => setShowHelp(true)
   → <ShortcutHelpDialog isOpen={showHelp} />            (App.tsx:622)
   → if (!isOpen) return null                            (ShortcutHelpDialog.tsx:18)
```

So the `?` keypress the operator could not type now lands in the field, and the dialog
that used to pop open over it does not.

## 5. Finding F1 — the Cmd+K half of the bug is still live (P2, narrow)

The packet's root-cause section says the same chain also eats other inputs —
pressing Cmd+K while typing also jumps to the command bar. Mira fixed
`useGlobalShortcuts` and her note says Cmd+K no longer triggers inside an input.
**That is true of `useGlobalShortcuts` and false of the app.**

`ui/src/App.tsx:183-190` installs a **second** `window`-level `keydown` listener with
**no typing guard**:

```js
if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'k') {
  e.preventDefault();
  ...
  setShowSupervisorIM((v) => !v);
}
```

Measured (my own harness, real hook + verbatim transcription of that condition, focus
inside a real `<input>`):

| Listener | Cmd+K inside an input |
|---|---|
| `useGlobalShortcuts` (guarded, fixed) | does not fire ✅ |
| `App.tsx:184` (unguarded) | **fires** ❌ — and calls `preventDefault()` |

So while typing, `Cmd+K` still toggles the Supervisor IM panel. **Severity P2, not P1**,
and for two reasons I measured rather than assumed: `Cmd+K` is a modifier chord, so it
produces no character — nothing is *eaten*, the character stream is intact; the harm is
the focus steal, which is the second-order half of the bug rather than the reported
symptom (the operator reported that `?` was untypeable, and that part is fixed). I checked the
other three `window` keydown listeners too, so this is not a blanket claim:

| Listener | Guarded? | Eats characters? |
|---|---|---|
| `useGlobalShortcuts.ts:82` | yes (this fix) | — |
| `App.tsx:192` (Cmd+K) | **no** | no character produced, but fires mid-sentence |
| `SupervisorCommandBar.tsx:80` | gated on `isOpen` (its own modal) | Enter/Escape only, inside its modal |
| `SupervisorPanel.tsx:251` | n/a | Escape only, no `preventDefault` |

**No second path eats the `?` character** — I grepped every other listener's key
condition (`e.key === 'k'`, `'Escape'`, `'Enter'` only), so the primary reported bug has
exactly one cause and it is fixed.

Mira's delivery note flags this ambiguity explicitly and correctly: she reads criterion 3
as "usable when focus is *not* in an input" and asks Aegis to rule if the operator means
"usable *while typing*". F1 is the same ambiguity, seen from the code: the packet's own
root-cause text says Cmd+K mid-sentence is part of the bug, and that half is still
reachable through `App.tsx:184`. **Not fixed here, because the ruling is a product
decision, not a verifier's edit** — my role is to report, and Nimbus fixes.

## Verdict: PASS

The fix is real, its tests are load-bearing under six independent mutations, the full
suite is green against a matching baseline, typecheck is clean, and the operator's
reported symptom (untypeable `?`) is fixed at its single cause. F1 is a narrow,
lower-severity second path on the Cmd+K half, reported for Aegis to rule on and Mira or
Nimbus to close.

## Reproduction

```bash
# isolated worktree at the subject commit — never the seat's own tree
git worktree add --detach /private/tmp/flux-verify-shift e7e327e
ln -s <mira>/ui/node_modules /private/tmp/flux-verify-shift/ui/node_modules
cd /private/tmp/flux-verify-shift/ui

./node_modules/.bin/vitest run src/hooks/useGlobalShortcuts.test.ts   # 13 passed
./node_modules/.bin/vitest run                                        # 264 passed, 14 errors (baseline)
./node_modules/.bin/tsc --noEmit                                      # clean

# mutation: guard removed → 5 of 13 red
#   edit src/hooks/useGlobalShortcuts.ts: comment out
#   `if (isTypingContext(event)) return;` inside handleKeyDown
# mutation: isComposing → `return false`       → 2 of 13 red
# mutation: TEXTAREA dropped from tagName check → 1 of 13 red
# mutation: closest() walk dropped              → 1 of 13 red
# mutation: guard `return true` early           → 6 of 13 red
# mutation: the `?` branch deleted              → 3 of 13 red
# then: git diff --quiet e7e327e  → clean (zero residue)

# baseline comparison
git worktree add --detach /private/tmp/flux-verify-shift-parent b77bd2a
cd /private/tmp/flux-verify-shift-parent/ui && ln -s <mira>/ui/node_modules node_modules
./node_modules/.bin/vitest run      # 251 passed, 14 errors
```

## Files I did not change

Mira's source, her test file, her delivery note, her worktree, and the live database
(no writes; this verification is `git worktree` + `vitest` only). Two scratch harnesses
were written under `/private/tmp/` and deleted; both worktrees were removed and
`git status` in `packet/mira/knowledge-panel` is empty.
