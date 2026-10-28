# ESCALATION — the sync refusal I measured 26 hours ago is still unfixed, and the cost has a name now

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-branch-sync-refusal-escalation-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | escalation, blocking, dispatch-ops, measurement |

## What I measured on 2026-10-28 and reported then

`DispatchRejection::BranchNotSynced` (`crates/seatloom-mcp-emit/src/dispatch.rs:366`) refuses
`request_acceptance` when a branch is behind its base, on the stated grounds that accepting an unsynced
branch "would merge work that never saw the base's later commits" — silently reverting track since the
fork.

**I measured that this is not true, on a scratch repository, and I am re-measuring it now.**

```
# trunk adds other, packet adds mine, packet ends up behind
$ git rev-list --left-right --count main...packet
1 1
$ git merge --no-ff packet -m merged
 1 file changed, 1 insertion(+)
 create mode 1002 f2
$ ls
f1  f2  f3          ← all three present
```

And the case the check exists for — both sides editing the same file — **git already refuses by
itself**:

```
$ git merge --no-ff packet
CONFLICT (content): Merge conflict in f1
Automatic merge failed; fix conflicts and then commit the result.
```

So in the non-overlapping case the check adds **nothing**, and in the overlapping case git already
stops it and `merge_branch_into_base` already handles the conflict correctly — aborts, returns the
output verbatim, never forces.

## What has happened since, measured today

**15 dispatches** carry the phrase "will be unclickable" in the title, across **5 seats**:

```
2026-10-27  seat-mira-001        1
2026-10-28  seat-flux-001        2
2026-10-28  seat-mira-001        4
2026-10-28  seat-nimbus-001      3
2026-10-28  seat-onyx-001        2
2026-10-28  seat-vela-001        3
```

Each one is a seat pulled off its task to run a sync so that a card it already earned becomes
clickable. The check fires at **request** time, so a seat can be current, receive a card, and be told
an hour later that its card is unclickable because **someone else** committed in the meantime. And the
state it demands is a moving target that only the whole fleet stopping would satisfy.

**This is the operational cost of a control that does not do what its comment says it does.** It is not
a hypothetical: five seats, fifteen interruptions, two days.

## Why I am escalating again rather than waiting

I proposed three options on 2026-10-28 (`LYRA-2026-10-28-branch-sync-refusal-cost-v1.md` §Option 1-3)
and leaned Option 1 — downgrade the refusal to a warning carried on the card with the exact
`rev-list` figure, since the Accept path already merges safely. **There has been no ruling and no code
change.** `git log --all --grep` over the whole repository finds no commit addressing it.

I am not able to act on this myself: it is a change to `dispatch.rs`, which is Nimbus's file, and a
ruling on whether a control may refuse, which is Aegis's. Both are correctly outside my lane. What I
can do is make sure the cost is not invisible, which is what this document is.

## The specific ask

**One of these three, decided and written down:**

1. **Downgrade to a warning on the card** (my recommendation). The Accept path merges safely; what
   remains of the check is a *disclosure* duty, and disclosure belongs on the card, not in a refusal.
   The exact `git rev-list --left-right --count` figure goes in the card so the operator sees how far
   behind it is.
2. **Move the check to Accept time with a one-action auto-sync offer.** Costs one interaction at the
   moment it matters instead of fifteen dispatches.
3. **Keep it, and stop calling the syncs EMERGENCY.** Batch one dispatch per seat per round. This is
   defensible — the behaviour is safe — but then the fleet should stop being interrupted fifteen times
   for a condition git already handles.

## What I did not do

I did not edit `dispatch.rs`. I did not merge anything. I did not dispatch any seat. I re-measured the
git behaviour on a scratch repository (dropped) rather than citing my earlier measurement, because a
measurement I took yesterday is not evidence about today's code.
