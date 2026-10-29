# RULING — persist from operator input, not from an effect. And the structural reason the effect can never work.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-persist-from-input-ruling-v1 |
| status | active |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | ruling, ui, persistence, architecture, first-screen |

## The ruling

**Persist the UI position from operator input, not from an effect.** Call `updateProjectUIState` in the
handlers that change the tab or the project. Remove the effect-driven write.

This is Mira's recommendation and I am adopting it as the ruling, with the reason and with one
constraint she did not ask for.

## Why, and it is structural rather than a preference

**The write effect cannot distinguish "restored" from "chosen", and no condition can make it.** I
verified the mechanism in `ui/src/App.tsx` rather than taking her word:

```
line 502   setLoadedUIProjectId(activeProjectId);     <- the restore effect's last act
line 504-517  the write effect
             if (!restoredPosition || loadedUIProjectId !== activeProjectId) return;
             ... updateProjectUIState(activeProjectId, { activeTab, ... })
```

On the cold-start frame the restore effect runs, **skips** applying the saved tab (her gate, correctly),
and then sets `loadedUIProjectId = activeProjectId`. The write effect's guard is
`restoredPosition && loadedUIProjectId === activeProjectId` — **both now true** — so it fires and writes
`activeTab`, which is still the `useState` **default**. That is the pollution: the operator's saved page
is overwritten by the default he never chose, on the frame he did nothing.

Her pattern is exactly right: *narrow the gate and the pollution persists; widen it and the operator's
legitimate writes are swallowed.* That is not a tuning problem. **The effect is asking a question the
information available to it cannot answer** — "did a human choose this tab, or did a restore leave it
here?" is not derivable from `activeTab`, because both cases produce the same value.

**An operator input is the only place the distinction exists.** In a handler, "the operator clicked this"
is not inferred; it is the reason the handler ran. Moving the write there does not make the effect
cleverer — it removes the need for the effect to be clever.

## The constraint she did not ask for

**Do not spread `updateProjectUIState` across all eight `setActiveTab` call sites.** I counted them:
lines 525, 550, 634, 718, 822, 847, 940 plus the restore at 489. A persist call pasted into each is
eight chances to forget one, and a forgotten one is a silent position loss — the exact class of bug that
took ten rounds to find last time.

**Instead, one wrapper.** A single `chooseTab(tab)` that calls `setActiveTab(tab)` **and**
`updateProjectUIState(activeProjectId, { activeTab: tab, … })`, used by every operator-driven path. The
restore path at 489 keeps calling `setActiveTab` directly — that is the one place the tab changes
*without* operator input, and it must not persist. **One function that means "the operator chose this"
is the whole fix; the call sites become a mechanical substitution.**

## What this does not undo

Her cold-start gate stays. It is correct, it is the better invariant, and it is what makes the first
half of the first-screen ruling work. **This ruling fixes the second half** — the pollution her new test
caught on unmutated code, which is the defect my mutation 2 surfaced and which her suite had no watch on.

I want that on the record plainly: **my mutation found a live defect, and her test is what proved it was
live rather than theoretical.** She wrote it, it went red on code with no mutation applied, and she
stopped and escalated instead of tuning it green. That is the sequence this project is supposed to
produce, and it happened because a mutation survived a green suite.

## What is still owed, unchanged

1. **The test she just wrote must go green** — `switching AWAY and then BACK to the restored project
   still restores its tab`, and `a second launch lands back on the same project AND the same tab`. Both
   red now, both marking a real defect.
2. **Real-machine verification.** All evidence is jsdom plus probes. The ruling's observable behaviour is
   something the operator does with his hands.
3. **A verdict naming `disp-E06O6IAaWefF-BOW-CNJd`**, then the card. She is right to wait; the guard is
   right to refuse a verdict about a different dispatch.

## What I did not do

I did not implement it. I did not edit `App.tsx` or the test file on her branch — I restored my mutation
backup and left her tree exactly as she committed it. I did not write the wrapper for her; it is her
surface. I did not open a card, dispatch a seat, or merge anything.
