# Standing-duty verification of Aegis's close-step delivery, and the measurement that makes it better than what I asked for: he implemented the close step with three checkable criteria, and **he deliberately left "has been pushed" out of them, with a reason I measured and confirmed** — pushing `track` while cards are pending makes the cards' branches behind and the operator's Accept gets refused, so a criterion requiring a push could never be satisfied while a card is waiting. I verified all three criteria fail in both directions myself. And the branch is behind 1 again, from his three commits, which is the sixth window — but this time the commits were never pushed, so the window is one Mira merge wide rather than one operator-approval wide.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-his-close-step-omits-push-and-he-is-right-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, verification, measured, structural-conflict |

## What he delivered

`50f5da3` (+30 to `AEGIS-HEARTBEAT-v1.zhs.md`), `a0d0d5e` (the delivery note), `62870d2` (the heartbeat with the
close step in it). **Both of my dispatches are now `done`** — `disp-ZZEb6fpwXPR0zqMQjiHwV` and
`disp-bvCX0jjTVC6rEMD251ODj` — with `SeatDispatchCompleted` events from `seat-aegis-001` at 03:03:51 and 03:05:26 UTC.
**The lifecycle hole I reported is closed, by the seat that could close it, in about fifteen minutes.**

## The three criteria, and that they are checkable rather than felt

His close step requires all three before `complete_dispatch`:

| # | criterion | how it is checked |
|---|---|---|
| 1 | the delivery note exists, under `docs/coordination/tasks/aegis/`, named `AEGIS-<date>-<topic>-vN.md` | `ls` |
| 2 | the work is committed — `git status --porcelain` empty apart from known untracked | `git status` |
| 3 | the commit is on local `track/infra-foundation` | `git merge-base --is-ancestor HEAD <commit>` |

**I verified each fails in the wrong direction, in a scratch repository** — because a criterion that cannot fail is
worse than no criterion:

| direction | measured |
|---|---|
| delivery note deleted | a check for it fails — the criterion is load-bearing |
| a dirty file present | `git status --porcelain` non-empty — the criterion is load-bearing |
| commit on a different branch, not an ancestor | `merge-base --is-ancestor` fails — the criterion is load-bearing |

**And I verified all three hold in the real repository right now**: the note exists, the only untracked file is the
one he names as a known exception (`scripts/bind-seatloom-seats.sh`, deliberately left for the operator to decide
about), and `50f5da3` is an ancestor of HEAD.

## The thing he added that I did not ask for, and it is correct

**He left "has been pushed" out of the criteria, and wrote why:** with cards pending, pushing `track` is forbidden —
the cards' branches go behind and the operator's Accept is refused. So a criterion requiring a push would **never be
satisfiable while a card is waiting**. He names it as the same structural conflict reappearing in a new place, and
makes pushing a separate obligation after the cards clear.

**I measured this and it holds.** His three commits — `50f5da3`, `a0d0d5e`, `62870d2` — are **not on origin**:

```
50f5da3: NOT on origin
a0d0d5e: NOT on origin
62870d2: NOT on origin
```

**So he closed two dispatches without pushing, and the cards are still clickable in the sense that matters** — Mira
had already merged `5ea41ea` at 11:07, so the branch is behind 1, not 4.

**This is the first time this week that a seat has reasoned about the structural conflict and chosen not to pay it.**
Every previous instance — mine included — paid it. He found the move that avoids the payment: complete on local
truth, push later.

## The sixth window, measured precisely

**The branch is behind 1.** The cause is `62870d2` at 11:07, not `5ea41ea`:

```
5ea41ea : IS an ancestor of packet/mira/knowledge-panel   (Mira merged it, ae771ed, 11:07)
62870d2 : is NOT an ancestor                              (his heartbeat commit)
```

**Timeline:** `5ea41ea` 09:15 → `50f5da3` 11:03 → `a0d0d5e` 11:04 → `62870d2` 11:07, and Mira's merge `ae771ed` at
11:07.

**So the window opened at 11:07 by a documentation commit, and it is one commit wide.** That is the smallest these
windows have been — because he did not push, so nothing propagated to origin, so Mira's merge of the earlier commit
left only the latest one outstanding.

**The honest comparison: this window is cheaper than the previous four, and the reason is his push decision, not
luck.**

## What this changes about my standing ask

**My priced ruling still stands and is still unruled** — the operator still cannot click either card, and the proxy
criterion still costs him. But Aegis has now reduced the per-instance cost of the conflict to a single commit, while
leaving the criterion itself untouched, which is what I asked him to do.

**So the remaining ask is narrower than it was**: the criterion change is now only about the criterion, not about
the cost of every documentation commit. **That makes it easier to approve, not harder, and I should say so rather
than keep quoting the five-hour figure as though nothing had changed.**

## What I did not do

I did not edit `AEGIS-HEARTBEAT-v1.zhs.md`, any script, or any product code. I did not merge, push, accept, cancel, or
re-open any card, restart any application, or write to the live database. I did not merge `track` into Mira's branch —
that is her action. My measurements were reads plus one scratch repository: `git log`/`git show` for his three commits
and the criteria, `git branch -r --contains` for push state, `merge-base --is-ancestor` for both commits against the
branch, the dispatch rows and their completion events, and the three reverse-direction criterion checks in a scratch
repo. The ledger check is exit 0, `seats` is 10, `canonical_events` is 31.
