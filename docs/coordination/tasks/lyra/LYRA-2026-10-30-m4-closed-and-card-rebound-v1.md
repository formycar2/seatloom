# Standing-duty verification: M-4 is closed, the card was re-bound to a verdict measuring the current code, and Aegis caught a staleness I did not

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-m4-closed-and-card-rebound-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, closure, verification, mutation, acceptance |

## What I found this round

Two things happened at 11:01 that I did not cause and would not have caught.

**1. Mira closed M-4.** The surviving coverage gap Flux reported — `chooseTab` is the only writer of
`selectedObjectId` into the persisted `ProjectUIState`, and nothing asserted those fields — now has a test
named for it: *"chooseTab persists the SELECTED OBJECT alongside the tab — M-4"*. `d2fea84`, 74 insertions in
the contract file, `App.tsx` untouched.

**I ran the mutation again to confirm it is actually closed.** Dropped `selectedObjectId` and
`selectedObjectType` from the wrapper:

```
× chooseTab persists the SELECTED OBJECT alongside the tab — M-4
  Tests  1 failed | 12 passed (13)
```

**Red.** The mutation that survived a 12-test suite two rounds ago now fails against a 13-test suite.
Restored, worktree clean, **375/375 across 31 files** — up from 372, because she also added `LiveSeatsView`
tests in the same window.

**2. Aegis caught that the card I had her open was bound to a stale verdict, and had it re-bound.**

## The staleness, and why it matters more than it looks

The card `acc-disp-faEWBt8kSKgv-6pJs_Rqw` was bound to `ev-Dq2vTW8WjIU_hQufDDp2F`, which measured `0f420bb`.
Mira's branch then moved to `c3484a0`. **So the operator would have clicked Accept on a verdict about a
commit that is not the branch tip.**

Aegis's dispatch to her is precise about why this is not a technicality:

> "Do not skip the swap because 'the old verdict was also a pass.' Both are pass, but one measured old code —
> the difference is whether it measured the code Mr. Zhang is actually accepting."

*Do not skip the swap because "the old verdict was also a pass." Both are pass, but one measured old code —
the difference is whether it measured the code Mr. Zhang is actually accepting.*

**And he verified the swap is honest rather than a re-implementation:** `d2fea84` changed only
`ui/src/App.restartRestore.test.tsx`, 74 insertions, and `git diff 0f420bb..HEAD -- ui/src/App.tsx` is
**empty** — `chooseTab` has not changed by one byte since the first verdict. **So the new verdict is not "a
different implementation verified again," it is "the same implementation, with the missing assertion
added."** That distinction is what makes the re-bind a repair rather than a re-do.

## I verified the new binding myself

| | |
|---|---|
| card | `acc-disp-faEWBt8kSKgv-6pJs_Rqw`, **pending** |
| verdict | `ev-15UEVJ08G4I0anhIpCjUJ` |
| verdict value | **pass** |
| `payload.dispatch_id` | `disp-faEWBt8kSKgv-6pJs_Rqw` — names it |
| `event_object_refs` | `subject\|dispatch` names it |
| verdict's commit | `c3484a0` |
| branch tip | `04af484`, **1 commit ahead** |
| what that commit changed | **one delivery note, 73 insertions, zero product code** |

**So the binding is sound.** The verdict measures the code, and the one commit after it is documentation. I
checked that rather than assuming "one ahead" was harmless — it is harmless *because* of what it is, not
because of the count.

## The part I should own

**I did not catch this.** I verified the card's guard conditions last round — verdict pass, names the
dispatch in both channels, dispatch done and owned by Mira — and stopped there. **Every one of those checks
would have passed on a verdict measuring a stale commit.** The check I did not run is the one that would
have caught it: *does the verdict's commit match the branch tip?*

That is the same shape as the week's pattern, and I should name it rather than file it as a one-off: **I
verified the pointer and not the thing it points at.** The card pointed at a real, green, correctly-named
verdict. It also pointed at a commit that was no longer the delivery.

**And Aegis caught it, in the same window, while I was writing a document about how the card was correct.**
I reported the card as verified on the strength of the guard conditions. He went one level down.

## What is now true that was not

| | Before | Now |
|---|---|---|
| M-4 (`chooseTab` drops the selected object) | survives | **caught by a test named for it** |
| Card's verdict measures the branch tip | no, one commit behind | **yes** |
| UI suite on Mira's branch | 372/372 | **375/375** |
| The two contract tests that were red on track | green on her branch only | **still not on track** |

**Track is still red at 2 failed / 370 passed.** That is unchanged and it is a merge question, not a code
question.

## What I did not do

I did not grant, cancel, or re-open any card. I did not edit her tree beyond the one mutation, restored from
backup with the worktree verified clean afterwards. I did not merge anything.
