# Flux verdict — the `isTypingContext` guard, and her correction of her own Cmd+K claim

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-typing-context-guard-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, shortcuts, own-error, seat-flux |

- **Verifying dispatch**: `disp-KxAVjC1m5rIhLmMFdQ18v` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-Ksm1oC3fHNRnH2zumBN_p` (Mira)
- **Implementation commit**: `e7e327e` on `packet/mira/knowledge-panel`
- **Verdict event**: **`ev-Oc4hMs_bBXlnb52Pr1JJK`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The guard is correct, its reasoning about jsdom is correct, and her self-correction on Cmd+K is
exactly right.** All five mutations I ran kill, and the negative control she did not run is
decisive: **5 of her 13 tests are red on the pre-fix code.**

**The most valuable thing in this delivery is that she caught and corrected her own false claim.**
Her previous delivery said "Cmd/Ctrl+K also does not trigger inside an input" — which is false, and
which **I** had recorded as finding F1 (P2) in my own verdict. Rather than trusting my verdict, she
transcribed `App.tsx:183`'s condition into a probe, measured it, and confirmed my finding. See §4.

---

## 1. The guard — verified, including the part that is easy to get wrong

`useGlobalShortcuts.ts:30-45`:

```ts
const isTypingContext = (event: KeyboardEvent): boolean => {
  if (event.isComposing) return true;
  const target = event.target as HTMLElement | null;
  if (!target || typeof target.tagName !== 'string') return false;   // window/document carry no tagName
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return true;
  return (
    target.isContentEditable === true ||
    (typeof target.closest === 'function' &&
      target.closest('[contenteditable]:not([contenteditable="false"])') !== null)
  );
};
```

**Three details here are right, and each is the kind of thing that gets missed:**

- **`isComposing` is checked first**, before any DOM inspection. An IME composition in progress is
  typing regardless of what the target is, and checking it first means the rest of the function
  never runs for that case.
- **`window` and `document` are excluded explicitly** — they carry no `tagName`, and without that
  guard a keystroke that reached no element would be read as "not typing" and arm every shortcut.
  She has a test for exactly that (`a key that reached no element at all still arms the shortcuts`).
- **`isContentEditable` is OR'd with a `closest()` walk**, and her comment says why: jsdom reports
  `isContentEditable === undefined` for every element, so the DOM-level check is what makes the
  branch observable at all. That is a correct reading of a real environment limitation, not a
  guess.
- **`contenteditable="false"` is excluded** by the `:not()` — a false host is not a typing context,
  and she has a test for it.

Her placement is also right: the guard is the **first line** of the handler, before any shortcut
branch, so there is no path that can be reached while typing.

## 2. Mutations — all five kill

| # | Mutation | Result |
|---|---|---|
| MT1 | the guard removed entirely | **RED 5** |
| MT2 | `isComposing` dropped | **RED 1** — the IME test |
| MT3 | `INPUT`/`TEXTAREA` dropped | **RED 2** |
| MT4 | `contenteditable` dropped | **RED 1** |
| MT5 | `contenteditable="false"` wrongly included | **RED 1** |

Each branch of the guard is independently pinned, which is what you want from a guard whose whole
job is to be conservative.

## 3. The negative control she did not run

I checked out the pre-fix `useGlobalShortcuts.ts` and ran her 13 tests against it:

```
5 failed | 8 passed (13)
```

The five that fail are precisely the typing-context ones (`Shift+/ in a text input`, `every other
shortcut also stands down`, `a keystroke inside a contenteditable host`, `an IME composition in
progress`, `a composing keypress aimed at a field`). The eight that pass are the
shortcuts-still-work-outside-a-field tests, which is correct — the fix should not have changed
those. **So the file is genuinely about the defect and genuinely not about anything else.**

## 4. Her self-correction on Cmd+K — and it confirms my F1

This is the part worth the whole dispatch.

**What she wrote before** (in the implementation delivery): "Cmd/Ctrl+K and Cmd/Ctrl+` also do not
trigger inside an input."

**What is true:** the `useGlobalShortcuts` hook is guarded, so *its* Cmd+K does stand down. But
`App.tsx:183-190` installs a **second, parallel** window-level `keydown` listener with **no typing
guard** at all:

```ts
if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'k') {
  e.preventDefault();
  ...
  setShowSupervisorIM((v) => !v);
}
```

**I verified both halves on the real App.** With focus in a real `<input>`:

```
shift+/  ->  defaultPrevented: false   (the hook stands down — the fix works)
cmd+k    ->  defaultPrevented: true    (App.tsx still fires — F1 alive)
```

**So her correction is exactly right, and so was my F1.** She did not take my verdict on faith
either: she transcribed `App.tsx:183`'s condition into a probe rather than editing the source, and
measured the same thing.

**Her analysis of the impact is also right, and it is the distinction that matters.** She points out
that "does not eat bare characters" is true — Cmd+K produces no character, so the question-mark bug
really did have only one cause (`useGlobalShortcuts`), which my verdict also confirmed. What she
should not have inferred from that is "so it also does not fire inside an input". **A shortcut that
eats no character can still steal focus and `preventDefault()`**, which is a different harm.

**And she was right not to fix it here.** Her reasoning: fixing F1 in the same branch would move
past the verified commit `e7e327e`, and what got submitted would be unverified code. That is the
correct instinct and exactly what the coordination rules require. She asked Aegis for a ruling on
whether Cmd+K should work while typing — a product decision, not an implementation one — and said
she would not touch it before that ruling.

**One thing I should state plainly: the acceptance card she opened records this residual.** Her
preconditions item 3 says Cmd+K still triggers inside an input, and her `how_to_verify` step 3 is
scoped to "focus not in an input area". So the card does not overclaim, which is the thing that
matters when a residual is being carried.

## 5. Gates — every figure exact

At `e7e327e`:
- Her file: **13 passed**.
- Full UI suite: **264 passed / 22 files / 14 errors**.
- `tsc --noEmit` clean.
- **eslint: 143 problems**, and both her files clean. Her "no new lint" claim holds.
- **prettier**: both files clean.
- `vite build` succeeds.

## 6. What I did not do

- Did **not** fix F1. It needs a product ruling from Aegis on whether Cmd+K should work while
  typing, and then its own dispatch, verification, and card. I am re-raising it here rather than
  acting.
- Did **not** touch the hook, the tests, or `App.tsx`. Five mutations were applied and each
  reverted, with the tree confirmed byte-identical to `e7e327e` after each.
- Did **not** merge. `e7e327e` is in the branch history but not on `track`.
- Did **not** touch Mira's worktree beyond a detached worktree at `e7e327e`, removed afterwards;
  her `git status` is empty.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/typing e7e327e
ln -s <mira>/ui/node_modules /private/tmp/typing/ui/node_modules
cd /private/tmp/typing/ui

./node_modules/.bin/vitest run src/hooks/useGlobalShortcuts.test.ts   # 13 passed
./node_modules/.bin/vitest run                                        # 264 / 22 files / 14 errors
./node_modules/.bin/tsc --noEmit                                      # clean
./node_modules/.bin/eslint src                                        # 143
./node_modules/.bin/prettier --check src/hooks/useGlobalShortcuts.ts \
    src/hooks/useGlobalShortcuts.test.ts                             # clean
./node_modules/.bin/vite build                                        # ok

# THE GUARD, AND THE THREE DETAILS THAT ARE EASY TO MISS:
sed -n '25,58p' src/hooks/useGlobalShortcuts.ts
#   isComposing FIRST (before any DOM inspection)
#   window/document excluded (no tagName -> not typing -> shortcuts still arm)
#   isContentEditable OR'd with a closest() walk, because jsdom reports
#     isContentEditable === undefined for every element
#   contenteditable="false" excluded by the :not()

# THE NEGATIVE CONTROL SHE DID NOT RUN:
git checkout e7e327e~1 -- ui/src/hooks/useGlobalShortcuts.ts
./node_modules/.bin/vitest run src/hooks/useGlobalShortcuts.test.ts   # 5 failed | 8 passed
git checkout HEAD -- ui/src

# MUTATIONS (each reverted, tree verified identical afterwards):
#   the guard removed entirely          -> RED 5
#   isComposing dropped                 -> RED 1  (the IME test)
#   INPUT/TEXTAREA dropped              -> RED 2
#   contenteditable dropped             -> RED 1
#   contenteditable="false" included    -> RED 1

# F1, CONFIRMED ALIVE ON THE REAL APP (focus in a real <input>):
#   shift+/  ->  defaultPrevented: false   (the hook stands down — the fix works)
#   cmd+k    ->  defaultPrevented: true    (App.tsx:183 still fires — F1 alive)
grep -n -A9 "metaKey || e.ctrlKey" src/App.tsx | head -12
```

## Verdict event

`ev-Oc4hMs_bBXlnb52Pr1JJK`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-Ksm1oC3fHNRnH2zumBN_p` in the payload.

## Files I did not change

No source file. Five mutations were applied and each reverted, with the tree confirmed
byte-identical to `e7e327e` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
