# Standing-duty measurement of the cost of the criterion I ruled on, so the ruling is not just correct but priced: the level-with-base proxy made the operator's two cards unclickable **three times today**, in windows of 62 minutes, 59 minutes and about one minute. It cost Mira three merge commits. And the thing the proxy stands in for is now genuinely checked at the merge, so the proxy is a pure cost with no remaining benefit.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-cost-of-the-proxy-priced-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, measured, ruling, gate |

## Why I priced it

Two rounds ago I ruled that clickability must not depend on a delivery branch being level with an hourly-moving
mainline, and I measured that the criterion is unimplementable as written. **A ruling that says "this is wrong"
without saying what the wrongness costs is an opinion.** So this round measures the cost.

## The cost, measured from the commits rather than from anyone's account

**Three unclickable windows today**, each opened by an Aegis documentation commit on `track` and each closed by a
Mira merge:

| # | opened by | when | closed by | when | window |
|---|---|---|---|---|---|
| 1 | `a267b8d` docs(aegis) | 16:02 | `29ebc90` merge | 17:04 | **62 min** |
| 2 | `b73c037` docs(aegis) | 18:03 | `0d1cd1e` merge | 19:02 | **59 min** |
| 3 | Aegis's 21:00 note | ~21:00 | `d27c4a6` merge | 21:02 | **~1 min** |

**So the operator was unable to click either of his two cards for roughly two hours today**, in windows the length
of Aegis's heartbeat interval — because the interval between "a documentation commit lands on track" and "Mira
notices and merges" is the interval at which he works.

**And Mira paid three merge commits for it.** Measured: `git rev-list --count --merges --since=16:00` on her branch
returns **3**. None of those merges carried product code; each absorbed documentation.

**The third window was short only because she was already awake and watching.** That is luck, not design.

## What the proxy was standing in for is now genuinely checked

**This is what makes the cost unjustifiable rather than merely annoying.** Nimbus's `6e04dae` put
`merge_would_delete` on the accept path, before the merge, reusing the same exemption helper the card-opening gate
uses. I verified it in both mutation directions myself.

**So the danger the proxy was a proxy for — a merge that deletes files tracked on the base — is now checked at the
moment the merge happens, with the current state of both trees.** The proxy at the card gate was never checking
that; it was checking the calendar. **It now has no remaining benefit and a measured cost of about two hours of the
operator's time and three of Mira's merges per day.**

## The ruling, restated as a decision with its price attached

**Drop the level-with-base requirement from the clickability gate.** Keep it in `request_acceptance`, where it
costs nothing because the seat opening a card can merge immediately.

**What the clickability gate should check instead, all three of which already exist and are already measured:**

| check | where it already exists |
|---|---|
| the branch exists in the gate's repo | `check-cards-clickable.sh:110` |
| the merge would not delete non-exempt files tracked on the base | `acceptance_cmds.rs:101-112`, new |
| the verdict is green and about this dispatch | re-checked at decision time, `acceptance_cmds.rs:65-77` |

**And the acceptance I set on Nimbus's dispatch already proved the third one cannot over-refuse**: his
`a_safe_branch_is_not_refused` test carries my measurement of the two real cards — 24 added, 17 modified, **0
deleted** — and I re-measured it against the live branches myself.

## What I am explicitly not doing, and why it is not obstruction

**I am not editing `check-cards-clickable.sh`.** Two reasons, and both are real:

1. **The ruling is still awaiting the operator's approval**, which I asked for two rounds ago. Implementing my own
   ruling in a shared script while it is unapproved is the silent scope expansion my role forbids.
2. **The script is a gate on the operator's own action.** Changing what gates his click is a change to his
   instrument, and he is the one who uses it.

**What I am doing is making the approval cheap**: the price is measured, the replacement checks all exist, and the
prerequisite is verified. **There is nothing left to decide except whether to do it.**

## What I did not do

I did not edit any script or product code. I did not merge, push, accept, cancel, or re-open any card, restart any
application, or write to the live database. My measurements were reads: `git log` with `--since` and `--date` for
the three opening commits and the three merges on Mira's branch, `git rev-list --count --merges --since=16:00`,
`check-cards-clickable.sh` run myself (exit 0, because the third window is closed), and the Aegis notes at 17:00,
19:00 and 21:00 for their own accounts of the same events. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 26.
