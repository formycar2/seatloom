# Ruling on the structural conflict Aegis escalated: the criterion is wrong, not the workflow. `check-cards-clickable.sh` requires a delivery branch to contain its base — which, while a card is pending, requires a branch to keep chasing a moving mainline forever. Aegis offered two options and both are wrong, because both treat the symptom. The third option is to fix what the criterion asks, and it is mine to rule on.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-ruling-the-clickable-criterion-requires-chasing-a-moving-mainline-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | ruling, structural-conflict, measured, gate |

## What Aegis escalated, and what I measured myself

His 19:00 note: `check-cards-clickable.sh` exit 1 for the **second time in two hours**, same cause — his own
delivery-note commit on `track` put Mira's branch one behind, so both pending cards went unclickable. **He names it
correctly as a structural conflict between two rules he must both obey:**

1. the completion protocol — commit before going idle, never leave uncommitted work
2. card clickability — a card's branch must contain its base, and **any** `track` commit breaks that

**He refuses to choose between them and asks Mr. Zhang to.** That is the right escalation and I am not going to
overrule the operator's authority on his own workflow — but he asked for a choice between two options, and I
measured the thing both options assume.

## The measurement: the criterion asks the impossible

`scripts/check-cards-clickable.sh:118`:

```bash
if ! git -C "$GATE_REPO_ROOT" merge-base --is-ancestor "$base" "$branch" 2>/dev/null; then
  behind=$(git rev-list --count "$branch".."$base" 2>/dev/null)
```

**`git rev-list --count "$branch".."$base"` counts commits on the base that are not on the branch.** So the
criterion is: *the delivery branch must contain every commit the base has.*

**`track/infra-foundation` is the active mainline.** Measured today: Aegis commits a delivery note every hour.
**So while a card is pending, the criterion requires a delivery branch to keep absorbing an hourly-moving mainline,
forever, until the operator clicks.** That is not a workflow problem — it is a requirement that cannot be
satisfied by any finite amount of work.

**And it gets worse after acceptance.** Once the operator accepts, the branch is merged into the base. From that
moment the base contains the branch's work, and every subsequent base commit makes the branch further behind. **A
branch that has been accepted is permanently behind the mainline it was accepted into.** The criterion can never
again be true of it.

**So the criterion is not measuring what it was written to measure.** What it wants to prevent is the thing
`BranchNotSynced` prevents at request time — merging a branch that would silently drop the base's later commits.
**That is a real danger and it is about the merge, not about the calendar.** A branch one commit behind with
non-overlapping changes merges cleanly and drops nothing; the overlapping case produces a conflict and git refuses
it itself. I measured both of those on 10-29 in a scratch repository and they have not changed.

## The ruling

**The criterion should ask "would this merge drop the base's commits", not "is this branch currently level with the
base".** Concretely, the check that belongs on a pending card is the one the accept path already performs — and
only that one:

| what to check | why |
|---|---|
| the branch exists in the gate's repo | a missing branch is a real blocker |
| the merge would not delete files tracked on the base | this is the actual data-loss risk, and `merge-tree` previews it without moving any ref |
| the verdict is green and about this dispatch | already checked at request **and** at decision time |

**What should be dropped:** the `merge-base --is-ancestor` level-with-base requirement as a *clickability* gate.
`request_acceptance` can keep enforcing it — a seat opening a card should be current, and refusing there costs
nothing because the seat can merge immediately. **But it must not gate the operator's click**, because the
operator's click is the one thing in this system he can do at any hour, and the current criterion makes it
depend on whether Aegis happened to commit in the last hour.

**What must not happen while this is fixed:** nothing. I am ruling on the criterion's meaning, not editing the
script — that is a shared tool and it needs the operator's or Aegis's hand, not mine.

## Why both of Aegis's options are worse than fixing it

**Option A — hold delivery notes uncommitted while cards are pending.** This satisfies the clickability gate by
violating the completion protocol, which exists so that a crash or a compaction does not lose work. **It trades a
guaranteed loss for a cosmetic one.** And it does not even work: holding notes only helps until the next commit
that is *not* a note, and product code commits are exactly the ones that must never be held.

**Option B — keep the status quo, Mira merges every hour.** This is what happened today, three times. **It costs
Mira's context every hour, and it costs the operator a window in which his own action is blocked by someone
else's documentation commit.** That the blocking commit is a delivery note — a record of work, not work — is what
makes it intolerable rather than merely annoying.

**Neither option asks why a documentation commit should be able to block an acceptance.** That is the question,
and the answer is that it should not.

## What the operator gets, in his three sections

**Changed:** nothing yet — this is a ruling, not a change. Two cards are clickable right now because Mira merged
Aegis's 18:00 note, which is the third time today.

**What you will see:** the same thing again within the hour, unless the criterion changes. Aegis's next delivery
note will make both cards unclickable, and Mira will merge again.

**What needs you:** approve the ruling above — that clickability must not depend on a delivery branch being level
with an hourly-moving mainline — and let the criterion be narrowed to what actually protects the merge. **Until
then, option B is the only one that does not risk losing work, and I am not going to pretend otherwise.**

## What I did not do

I did not edit `check-cards-clickable.sh`, any other script, or any product code. I did not merge, push, or write
to the live database. I did not tell Aegis which of his two options to take — I escalated the question they both
rest on, which is mine to rule on as product owner. My measurements were reads: `check-cards-clickable.sh:108-126`
and its header comment, `git rev-list --left-right --count` on the real branches, `check-cards-clickable.sh` run
myself (exit 0, because Mira has already merged), and `git log` for the three commits involved. The ledger check
is exit 0, `seats` is 10, `canonical_events` is 25.
