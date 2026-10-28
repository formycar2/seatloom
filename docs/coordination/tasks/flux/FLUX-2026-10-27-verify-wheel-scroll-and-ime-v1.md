# Flux verdict — the live-seats wheel scroll, and the IME `shift+key` finding

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-wheel-scroll-and-ime-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, xterm, ime, seat-flux |

- **Verifying dispatch**: `disp-99Sr4c7oTzs1LRBW4ldp7` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-wa7F61uczSw8ebZzeXwml` (Mira)
- **Subject commit**: `56e7614` on `packet/mira/knowledge-panel` — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-WirH7Qh1eNDP7dZSV5uVs`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The fix works, the tests are honest, and her decision not to touch code for the IME finding is
correct.** All four mutation results she disclosed reproduce exactly, including the two she
reported as *staying green* with an explanation — which is the part most deliveries get wrong.

**Two corrections, neither changing the verdict:**

1. **Her root cause is half right, and the half she missed means her test cannot prove the fix is
   necessary.** There is a **second, ungated** `wheel` listener inside xterm
   (`Terminal.ts:800`) that handles the no-protocol case. I measured it executing. See §2.
2. **Her "xterm 6.0.0 is word-for-word identical" claim is false** — 6.0.0 adds
   `keyCode === 20` (CapsLock) to the allowlist. Her *conclusion* (upgrading does not fix
   `shift+,`) is still correct, because 188 is not in either list. See §5.

---

## 1. The protocol gating — confirmed exactly at the source

I did not take the `16 & protocolMask` claim on faith; I read xterm 5.5.0's source:

- `CoreMouseEventType.WHEEL = 16` (`Types.d.ts:360`) — her number is exact.
- The `wheel` listener is added and removed **inside** the `onProtocolChange` callback
  (`Terminal.ts:745-751`), gated on `events & CoreMouseEventType.WHEEL`.
- `CoreMouseService.ts:47-48`: the `VT200` protocol's `events` are
  `DOWN | UP | WHEEL` (1|2|16 = 19).
- `InputHandler.ts:1900`: DECSET 1000 sets `activeProtocol = 'VT200'`.

And I reproduced the observable consequence on the real component:

```
no protocol written      -> activeProtocol "NONE", element lacks `enable-mouse-events`
after ESC[?1000h         -> activeProtocol "VT200", `enable-mouse-events` present
```

**So the claim is correct: with no mouse protocol active, the protocol-gated wheel listener is
never registered.**

## 2. Correction: the wheel event does not "go nowhere" — there is a second, ungated listener

Her delivery says the listener is never registered and "the wheel event goes nowhere". That is
where the analysis stops short. `Terminal.ts:800` registers a **second** `wheel` listener that is
**not** inside the protocol gate:

```js
this.register(addDisposableDomListener(el, 'wheel', (ev: WheelEvent) => {
  // do nothing, if app side handles wheel itself
  if (requestedEvents.wheel) return;          // <- only skips when a protocol IS active
  ...
  // normal viewport scrolling
  if (this.viewport!.handleWheel(ev)) {
    return this.cancel(ev);
  }
}, { passive: false }));
```

`handleWheel` does `viewportElement.scrollTop += amount` and returns `_bubbleScroll(...)`, which
calls `preventDefault()` when there is room to scroll. **I measured this listener executing**:
with *our* listener removed, a dispatched wheel still set `scrollTop` to `-120` and left
`defaultPrevented === true`. So the event was received and acted on — just not in a way that
moved the buffer in jsdom.

**Why this matters, and what it does not change:**

- It does **not** invalidate the fix. The operator's symptom was real, and our listener calls
  `scrollLines()`, which moves the buffer directly rather than relying on the DOM-scroll →
  `scroll`-event → buffer-sync round trip.
- It does mean her stated root cause is **incomplete**, and more importantly it means **her test
  cannot distinguish "the fix is necessary" from "the fix is redundant"**: in jsdom, xterm's
  native path sets `scrollTop` but the buffer does not follow, because jsdom does not fire
  `scroll` events for a programmatic `scrollTop`. So the test's central assertion — "the viewport
  MOVED, which it did not when the listener was never registered" — is measuring our
  `scrollLines` call and is blind to whether a real browser needed it.

That is an honest limit of the harness, and it is the same class of limit she documented for
`preventDefault`. I am recording it because the next person to touch this code should know the
fix's necessity is argued from the operator's symptom plus the source reading, **not** proven by
the test.

**The fix itself is well-built and I would not change it:** it deliberately does *not* enable
mouse reporting (which would change the wrapped program's behaviour — far beyond "let me read my
history"), it is scoped to `term.element` rather than the container so a wheel over the padding
is not a wheel over the terminal, it uses `{ passive: false }` because `preventDefault` requires
it, and it stands aside for modifier wheels (ctrl/meta/alt) rather than guessing at zoom.

## 3. Mutations — all four reproduce, including the two she reported as green

| # | Mutation | Her claim | My measurement |
|---|---|---|---|
| M1 | our wheel listener removed | RED 3 | **RED 3** |
| M2 | scroll direction reversed | RED 3 | **RED 3** |
| M3 | `preventDefault` removed | **green** | **green** |
| M4 | listener not removed on unmount | **green** | **green** |

**Both of her "stays green" disclosures are accurate, and the reasons she gives are the right
ones.** I verified M3's reason independently rather than taking it: xterm's protocol-gated wheel
handler ends in `return this.cancel(ev, true)`, and `cancel` is `ev.preventDefault();
ev.stopPropagation();`. So the call is redundant *while that listener is registered*. Her
comment ("kept because ours is bound directly to the element and must not depend on xterm's
internal listener being registered — that dependency is the very assumption that broke
scrolling") is exactly the right justification, and her test comment states plainly that the
assertion passes without our call.

M4's disclosure is equally careful, and better than most: she says her original unmount test
proved nothing (a disposed xterm also stops scrolling), rewrote it to assert the element is
detached and the component held a Terminal, and wrote in the comment that it proves "the
component did not leave the listener on an element outliving it", **not** that the listener was
removed — because jsdom has no hook to observe that. **A test that says what it does not prove is
worth more than one that implies more than it checks.**

One extra check I ran on M3 that she did not, and it is worth recording: with our
`preventDefault` removed, `defaultPrevented` is still `true` — but a capture-phase observer sees
`false` and a bubble observer sees `true`, with the `enable-mouse-events` class **absent**. So in
the production condition (no protocol) the prevention comes from the **ungated** listener at
`Terminal.ts:800`, not from the protocol-gated one her comment names. Her conclusion (keep the
call, it is cheap insurance against a dependency on xterm's internal binding) stands; only the
attribution in the comment is slightly off.

## 4. Gates — every figure exact

- Her file: **7 passed**.
- Full UI suite: **321 passed / 26 files / 0 errors** (baseline 314 + her 7 — arithmetic matches).
- `tsc --noEmit` clean.
- **eslint: 143 problems before AND after**, diffed against `56e7614~1` on the whole `src` tree.
  Her "identical to baseline, 0 on my two files" claim holds.
- The commit is **60 insertions, 0 deletions** in `SessionTerminal.tsx` plus the new test file —
  a pure addition, so nothing existing was disturbed.
- I did not run `pnpm build`; the change is a DOM listener and the type-check plus the suite
  cover it, and her build claim is not load-bearing for this verdict.

## 5. The IME finding — her conclusion is right, one detail is wrong

**Her analysis of the `keyCode >= 48` gate is exact.** I read `Keyboard.ts:381`:

```ts
} else if (ev.key && !ev.ctrlKey && !ev.altKey && !ev.metaKey && ev.keyCode >= 48 && ev.key.length === 1) {
```

`,` has `keyCode` 188, which passes. Her own false negative (a first test that dispatched
`keydown` without a `keyCode`, so it was 0 and got filtered) is a genuinely useful thing to have
written down — it looks exactly like "the character is swallowed" and is a test defect.

**Her CompositionHelper analysis is exact.** I read the same code in 5.5.0
(`CompositionHelper.ts:94-108`): while composing, `229` and the modifiers `16/17/18` are allowed
through, and **anything else** calls `_finalizeComposition(false)`. `shift+,` has `keyCode` 188,
which is not in either list, so the composition is force-finalised and the character is lost.

**Her "upgrading does not help" conclusion is right, but her evidence for it is wrong.** I
downloaded xterm 6.0.0 and diffed the function. It is **not** word-for-word identical:

```
5.5.0:  if (ev.keyCode === 229) {
6.0.0:  if (ev.keyCode === 20 || ev.keyCode === 229) {     // 20 is CapsLock
```

So 6.0.0 *did* change this function — it added CapsLock to the pass-through list. **But 188 is
still absent**, so `_finalizeComposition(false)` still fires for `shift+,` and her conclusion
survives: upgrading does not fix it.

**The distinction matters** because "identical, therefore upgrading is pointless" is a weaker
argument than "changed, but not in the way that would help". The first would be refuted by any
diff; the second is the actual state. Her recommendation — do not touch xterm internals without a
dispatch, and route this to whoever owns terminal selection — is right regardless.

## 6. Her "cannot test" disclosure is complete and correctly framed

She states plainly that real Chinese-IME values for `isComposing` / `target.tagName` / `key` were
not obtained, that jsdom cannot drive a real IME composition loop, and that her conclusion is
"read from code, not measured on a machine". She also lists every layer she checked for a SeatLoom
interceptor and found none. **That is the correct way to report an unclosed criterion**, and it is
materially better than claiming it.

## 7. What I did not do

- Did **not** change the fix, the comments, or the tests. The corrections in §2 and §5 are
  reported, not applied — it is her file and the dispatch said report rather than fix.
- Did **not** merge. `56e7614` is not on `track`; the running binary predates it and contains
  none of this.
- Did **not** touch Mira's worktree beyond a detached worktree at `56e7614`, removed afterwards;
  her `git status` is empty.
- Did **not** verify the real-machine scroll. That needs a rebuilt, reinstalled app and a human
  hand on a trackpad.
- Did **not** write to the live database.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/wheel 56e7614
ln -s <mira>/ui/node_modules /private/tmp/wheel/ui/node_modules
cd /private/tmp/wheel/ui

./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.wheel.test.tsx   # 7 passed
./node_modules/.bin/vitest run                                                   # 321 / 26 files
./node_modules/.bin/tsc --noEmit                                                 # clean
./node_modules/.bin/eslint src                                                   # 143 (same as 56e7614~1)

# THE PROTOCOL GATING, AT THE SOURCE (xterm 5.5.0):
#   Types.d.ts:360            CoreMouseEventType.WHEEL = 16
#   Terminal.ts:745-751       the wheel listener is added/removed INSIDE onProtocolChange,
#                             gated on `events & CoreMouseEventType.WHEEL`
#   CoreMouseService.ts:48    VT200.events = DOWN | UP | WHEEL   (= 19)
#   InputHandler.ts:1900      DECSET 1000 -> activeProtocol = 'VT200'
# Measured on the real component:
#   no protocol -> activeProtocol "NONE", no `enable-mouse-events` class
#   ESC[?1000h  -> activeProtocol "VT200", class appears

# THE MISSED SECOND LISTENER (Terminal.ts:800, NOT protocol-gated):
#   `if (requestedEvents.wheel) return;` then viewport.handleWheel(ev)
# Measured with OUR listener removed: scrollTop set to -120, defaultPrevented === true,
# capture observer sees false / bubble observer sees true, and the mouse class is ABSENT —
# so in the production condition the prevention comes from THIS listener, not the gated one.

# MUTATIONS (each reverted, tree verified identical afterwards) — all match her claims:
#   our listener removed        -> RED 3
#   direction reversed          -> RED 3
#   preventDefault removed      -> GREEN (redundant while xterm's gated listener runs)
#   listener not removed        -> GREEN (jsdom cannot observe it)

# THE IME GATE, AT THE SOURCE:
#   Keyboard.ts:381            `... && ev.keyCode >= 48 && ev.key.length === 1`
#   CompositionHelper.ts:94-108  while composing: 229 and 16/17/18 pass, everything else
#                                calls _finalizeComposition(false). 188 is in neither list.
#   xterm 6.0.0 diff            NOT identical: adds `keyCode === 20` (CapsLock).
#                               188 still absent, so her conclusion (upgrading does not fix
#                               shift+,) survives — but the "word-for-word" claim does not.
```

## Verdict event

`ev-WirH7Qh1eNDP7dZSV5uVs`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-wa7F61uczSw8ebZzeXwml` in the payload.

## Files I did not change

No source file. Four mutations were applied and each reverted, with the tree confirmed
byte-identical to `56e7614` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
