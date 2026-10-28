# VELA-2026-10-28-eighth-sync-173-commits-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-eighth-sync-173-commits-v1 |
| status | delivered (merged 173, conflict resolved, criterion 0, card pending) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | disp-1UFBoxL0tcJUOtzHqG1AM |
| depends_on | VELA-2026-10-28-seventh-sync-v1 |
| tags | sync, merge, conflict, gitignore, per-seat-config, race |

Aegis 2026-10-28 19:00, priority 99: I was **173 commits behind** because the
operator accepted 27 cards between 15:15 and 18:18 and those merges landed on
track in one batch. Not drift on my part. Merge, don't rebase, re-measure to 0.

## Done

| step | result |
|---|---|
| scratch-worktree dry-run | **conflict found** — `.saih/mcp.json`, add/add |
| conflict resolved | untracked the file; kept it on disk |
| merge `--no-ff` | `8dabd63`, parents `9e1fff3` + `8e56e44` |
| **`git rev-list --count <branch>..track/infra-foundation`** | **0** (the dispatch's criterion) |
| `git merge-base --is-ancestor track/infra-foundation <branch>` | SUCCESS |
| push to gitlab | `9e1fff3..8dabd63` |
| re-measure **after** the push | 0 behind, is-ancestor SUCCESS, local/remote 0/0 |
| card `acc-disp-2o_BDY1tb3b92ZQwaYXEC` | **pending**, branch/base intact |
| wash suite on the merged tree | **66 tests, 0 skipped, OK** |
| `s5_writein.py` | **87/87 records, 10 sessions, 0 validation problems** |

## The conflict, and the mistake it was

The only conflict in 173 commits was `.saih/mcp.json` — **add/add**, because Flux
committed `seat-flux-001`'s copy to track and my commit `bd19ef3` (2026-10-17
18:13) committed `seat-vela-001`'s. The file holds:

```
SEATLOOM_ACTOR_REF = seat:<whoever's worktree this is>
DATABASE_URL       = postgresql://...
```

**It differs per seat by construction**, so it can never merge cleanly between two
seats. Committing it to a shared branch was simply wrong, and I did it.

My own commit message was the tell, and I missed it: `bd19ef3` is titled *"chore:
ignore `.saih/` local tooling config"* — but the diff is **12 insertions of the
file itself** and **no `.gitignore` entry at all**. I wrote a message describing
the action I thought I was taking, while the diff shows I took the opposite one.
Nothing in my verification caught it, because I checked the intent, not the diff.

The operator had already written the correct rule, and my branch had not synced it
yet: `fa93d0f` (2026-10-17 13:30, `chore: ignore .saih/ project config`) says
*"ignore it so `git add -A` stays clean"*. I verified `fa93d0f` was **not** an
ancestor of `bd19ef3`, so at the moment I committed, my branch's `.gitignore` had
no `.saih/` rule and nothing stopped me.

**Resolution:** `git rm --cached`, not picking a side. Picking a side would have
overwritten one seat's identity with the other's and left the file tracked for the
next seat to trip over. Untracked, it stays on disk (my MCP server needs it), is
now ignored by `.gitignore:45`, and the next sync will not conflict on it.

## One real consequence, caught and repaired

The fast-forward that brought the merge into my worktree **deleted `.saih/mcp.json`
from disk** — the index deletion removed the working-tree file too. I noticed on
the very next command and restored it from `bd19ef3`, then re-verified: valid JSON,
my own seat identity, ignored, untracked, worktree clean.

Recording it because it is the kind of thing that silently breaks the *next*
session rather than this one: the MCP config was gone, and the only reason I
noticed is that I happened to `cat` the file while checking the resolution. Had I
skipped that, the seat would have started its next turn with no MCP server
configured.

## On the structural note

Aegis's observation is right and worth restating in my own words: a branch that is
healthy at 15:15 becomes 173 commits behind at 18:18 without doing anything wrong,
purely because 27 cards were accepted in one batch. The variable that makes this
hurt is **batch size**, not seat diligence.

Counting today's syncs honestly: 2 were genuine staleness, and **6 were the same
race**. Six syncs to stay current with a base that advances whenever the operator
clicks Accept is not a process problem on my side — it is what the merge gate
costs when the acceptance rate is high and the batch is large.

## What I did not do

Did not rebase (173 commits, and my cards name commits on this branch). Did not
merge into track. Did not pick a side in the conflict by choosing a seat identity.
Did not touch the live database. Did not open, cancel or modify any card.
