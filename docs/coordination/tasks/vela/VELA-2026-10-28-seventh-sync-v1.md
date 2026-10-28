# VELA-2026-10-28-seventh-sync-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-seventh-sync-v1 |
| status | delivered (merged, criterion 0, card pending and clickable) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | disp-0mRmhJyH_HlDExQ3YdzVw |
| depends_on | VELA-2026-10-28-hole-b-acceptance-card-opened-v1 |
| tags | sync, merge, race, verdict-ref, acceptance |

Aegis 2026-10-28 15:25, priority 99: I was 2 commits behind (`c01277d`,
`ab44408`), which made my 1 pending card unclickable. Merge, push, re-measure to
0. No rebase, no merging into track.

## Done

| step | result |
|---|---|
| scratch-worktree dry-run | *Automatic merge went well*, zero unmerged paths |
| merge `--no-ff` | `537fb1e`, zero conflicts |
| **`git rev-list --count packet/vela/washing-pipeline..track/infra-foundation`** | **0** (the dispatch's criterion) |
| `git merge-base --is-ancestor track/infra-foundation packet/vela/washing-pipeline` | SUCCESS |
| push to gitlab | `e90fb0a..537fb1e` |
| re-measure **after** the push | 0 behind, is-ancestor SUCCESS |
| card `acc-disp-2o_BDY1tb3b92ZQwaYXEC` | **pending**, branch/base intact |

The wash suite on the merged tree is **66 tests, 0 skipped, OK**, and
`s5_writein.py` still resolves **87/87 records across 10 sessions with 0
validation problems** — the merge touched Aegis scripts and knowledge docs, not
the washing pipeline.

## The commit that came in is the fix for what I reported an hour earlier

Worth stating plainly, because it closes a loop rather than just being a sync.

`c01277d` is `feat(aegis): check that a card's verdict ref is a verdict about
THAT card`, adding `scripts/check-verdict-refs.sh`. Its header records the bug
class it was written for:

> I found a verdict whose `payload.dispatch_id` was `disp-B4Y5oAhMQwtXm5Vva1fw` —
> zero rows in `dispatches`. The real id is `disp-B4Y5oAohMQwtXm5Vva1fw`: letter
> `o` vs digit `0`, one character, in a nanoid alphabet that contains both.

That is the same failure shape as the discrepancy I reported in my previous
delivery — the verdict id I was handed (`ev-verdict-disp-2o_BDY1tb3b92ZQwaYXEC`)
was **not** the one I had pre-declared (`ev-verdict-vela-message-count-hole-b`).
I found it by looking the verdict up by `payload.dispatch_id` instead of trusting
the id I had asked for, which is the only reason my card is correct. A seat that
trusted its own pre-declared string would either block forever or, worse, bind a
verdict to the wrong dispatch.

I ran the new script against the merged tree rather than taking its existence as
sufficient:

```
== pending acceptance cards: is each verdict ref a verdict about THIS dispatch? ==
  good=28 bad=0
...
  ev-usP7dX7geulRhaDy1eXwD     WILD   names disp-B4Y5oAhMQwtXm5Vva1fw which has 0 rows
OK: every pending card's verdict ref is a verdict about that very dispatch.
```

`good=28 bad=0` — my card is in the 28. And the wild pointer it describes is real
and still in the ledger: **5 verdicts point at nothing**, one of them `WILD`
rather than merely empty. The script notes they are not currently referenced by
any pending card, but a future dispatch bearing one of those ids would inherit
them as evidence. That is a latent hazard the fleet should know is still live,
not just one that was fixed.

## On the seventh occurrence

Aegis's own dispatch says it well: this keeps happening because he follows the
existing commit protocol and dispatches a sync in the same turn, pending an
operator ruling on whether his docs/scripts belong on track at all
(`ev-op-decision-aegis-commit-policy`). I have no view on that policy question
and should not form one — but the mechanical pattern is now measured, not
asserted:

| # | cause | who |
|---|---|---|
| 1–2 | genuine staleness | me |
| 3 | race (push landed between my merge and my request) | me |
| 4 | race | me |
| 5 | race | me |
| 6 | race | me |
| 7 | race | me |

Two of seven were real staleness. Five were the same race, and the fifth
occurrence is the one that would have cost the operator a click. The control that
would close it is not another sync dispatch — it is checking "has track moved
since I started" immediately before the request, which is what I did here by
accident of ordering rather than by design.

## What I did not do

Did not rebase. Did not merge into track. Did not open, cancel or modify any
card. Did not touch the live database. Did not act on Aegis's commit-policy
question, which is the operator's to answer.
