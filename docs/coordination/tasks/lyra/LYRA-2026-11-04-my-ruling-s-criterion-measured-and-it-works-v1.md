# Standing-duty verification of my own ruling: I ruled last round that clickability should ask "would this merge drop the base's commits" rather than "is this branch level with the base", and named the mechanism — `merge-tree --write-tree` previewed without moving any ref, then `git diff --name-status base..tree`. I had not measured it. I have now, on both pending cards, and the ruling is implementable exactly as written.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-my-ruling-s-criterion-measured-and-it-works-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, ruling, measured, gate |

## Why I measured this

Last round I ruled on the structural conflict Aegis escalated, and I named the replacement criterion with a
specific mechanism. **I wrote the mechanism without running it.** That is the same half-finished claim I have been
catching all week, in my own ruling, one round after catching it in someone else's.

**So this round measures it.**

## The mechanism, measured on both pending cards

`crates/seatloom-core/src/git/ops.rs:124-175` already implements exactly this — `merge_would_delete` runs
`git merge-tree --write-tree base branch`, takes the resulting tree oid from the first line, then
`git diff --name-status base..tree` and collects every `D` line. **The product already has the instrument; the
clickability gate does not use it.**

**Measured on the real branches:**

```
$ git merge-tree --write-tree track/infra-foundation packet/mira/knowledge-panel
139a52f8aeb464130bfbeca26c0db5d1e84c6290        (exit 0 — no conflict)

$ git diff --name-status track/infra-foundation..139a52f8
24 A    17 M    0 D
```

**Zero deletions.** So merging `packet/mira/knowledge-panel` into `track/infra-foundation` adds 24 files, modifies
17, and **removes nothing tracked on the base.** The merge is safe in exactly the sense that matters.

## The three checks I ruled should gate a card, all measured

| check | measured |
|---|---|
| the branch exists in the gate's repo | `git show-ref --verify refs/heads/packet/mira/knowledge-panel` → yes |
| the merge would not delete files tracked on the base | **0 deletions** (above) |
| the verdict is green and about this dispatch | both cards: `verdict = pass`, and `payload->>'dispatch_id' = a.dispatch_id` → true for both |

**All three hold.** So under my ruling, both cards are clickable — which they also are under the current gate,
because Mira merged Aegis's 18:00 note for the third time today.

## The difference between the two criteria, stated as the reason the ruling matters

| | current criterion | the ruled criterion |
|---|---|---|
| what it asks | is the branch level with the base? | would the merge delete the base's files? |
| what its truth depends on | **whether anyone committed to the base in the last hour** | **the two trees themselves** |
| after the card is accepted | permanently false — the branch is behind the mainline it was merged into | still true — the merge already happened and deleted nothing |
| what it actually protects | nothing directly; it is a proxy | the thing that was worth protecting |

**That middle row is the one that makes the current criterion indefensible.** A branch that has been accepted can
never again satisfy it, so any criterion built on it is measuring the calendar, not the merge.

## What this does not do

**It does not change the gate.** I ruled on the criterion's meaning; the script is a shared tool and the change
needs the operator's approval or Aegis's hand. **I am not editing it, and I am not going to implement my own
ruling in someone else's script while the ruling is still awaiting the approval I asked for.**

**It also does not retire `BranchNotSynced` at request time.** My ruling kept that: a seat opening a card should be
current, and refusing there costs nothing because the seat can merge immediately. **What I ruled out is only its
use as a gate on the operator's click.**

## One thing worth recording about the instrument

**`merge_would_delete` exists and is tested, and it is already called on the accept path** — I read it in
`acceptance_cmds.rs`'s containment preview. **So the system already asks the right question at the moment it
matters, and asks the wrong question at the moment it blocks someone.** That is a more precise statement of the
defect than "the criterion is wrong": the right check is present and wired into the merge, and an unrelated proxy
is wired into the gate.

## What I did not do

I did not edit `check-cards-clickable.sh` or any other script or product code. I did not merge, push, accept,
cancel, or re-open any card, and did not write to the live database. My measurements were reads:
`git merge-tree --write-tree` and `git diff --name-status` on the real branches, `git show-ref --verify`,
`psql` joining the two cards to their verdict events, `git merge-base --is-ancestor`, and reading
`crates/seatloom-core/src/git/ops.rs:124-175`. The ledger check is exit 0, `seats` is 10, `canonical_events` is 25.
