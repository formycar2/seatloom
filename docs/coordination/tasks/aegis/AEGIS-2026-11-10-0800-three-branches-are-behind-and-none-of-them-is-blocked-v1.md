| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0800-three-branches-are-behind-and-none-of-them-is-blocked |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, clickability, behind-track, operator-ruling, measured |

# Three branches are behind track and none of them is blocked, and I verified that rather than assuming it

## What I checked

All three pending cards' branches have drifted behind `track/infra-foundation`, and the drift is growing:

| card's branch | ahead | behind | contains track? |
|---|---|---|---|
| `packet/nimbus/candidate-final` | 16 | **18** | no |
| `packet/vela/washing-pipeline` | 11 | **24** | no |
| `packet/mira/knowledge-panel` | 8 | **11** | no |

On the raw containment question, all three now fail. That is exactly the condition that used to make
cards unclickable before you ruled on it.

## What I verified rather than assumed

**Your ruling was that clickability asks only whether the merge would drop mainline commits or delete
files, and that being behind is a note rather than a refusal.** Three branches failing containment is
exactly the case that ruling covers, so I measured whether the ruling still holds:

| check | result |
|---|---|
| `git merge-tree --write-tree` for each branch | **exit 0** — no conflicts |
| `check-cards-clickable.sh` | **OK: all 3 pending cards are clickable** |
| a simulated merge of Mira's branch into track | 9 files, 990 insertions, **and track's tip remains an ancestor of the result** |

So merging any of the three would **not** drop a mainline commit. `git merge --no-ff` cannot drop
commits — that is the mechanism the ruling relied on, and it still holds.

**The "behind" numbers are growing, and that is expected, not a defect.** Each one is my own hourly
delivery commits landing on track while the branches stay where their seats left them. Nothing is
stranded; the merge preview is clean for all three.

## Why this is worth an hour

I have carried "behind track" as a thing to watch for several days, and the number has been climbing
without me re-checking what it actually means. Measured against the ruling, it means **nothing** — it is
a note. Left unmeasured, a rising number reads as a worsening problem, and a rising number nobody
re-checks is how a note becomes a false alarm.

That is the third instance of the same shape this week: a figure that climbs while its meaning goes
unverified. The other two were the unclaimed-work count (13, of which 0 were lost) and the doc-gate
failure count (328, of which accepting one card moves 26 between rules rather than clearing them).

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- All five tests exit 0.
- Three pending cards, all clickable, all merge clean.
- DB reliability: 10/10 this hour.
- Push: local and remote both at `2bbc9fc`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Three cards are waiting**, and none of them is blocked by being behind. Accepting any of them is a
  clean merge.
- **For Vela's card specifically:** accepting it moves 26 files from a masked failure to a visible one and
  reduces the undeclared count by zero.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
