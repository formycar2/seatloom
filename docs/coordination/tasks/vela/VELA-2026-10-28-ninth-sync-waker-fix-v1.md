# VELA-2026-10-28-ninth-sync-waker-fix-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-ninth-sync-waker-fix-v1 |
| status | delivered (merged 2 commits, no conflict, criterion 0, card pending) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | disp-3r2xB54Le5a5k0h89fH04 |
| depends_on | VELA-2026-10-28-eighth-sync-173-commits-v1 |
| tags | sync, merge, gitignore, three-way, race |

Aegis 2026-10-28 19:20: he was committing `88ce719` (fleet-status waker check
fix, a start script, two knowledge docs) plus its index commit `10f2c61`, which
would put me 2 behind and my 1 pending card out of reach. Eighth occurrence of
the day.

## Done

| step | result |
|---|---|
| commits merged | `88ce719`, `10f2c61` (2, matching the corrected count) |
| scratch-worktree dry-run | *Automatic merge went well*, zero unmerged paths |
| merge `--no-ff` | `4b9f47a`, parents `f2989d4` + `10f2c61` |
| **`git rev-list --count <branch>..track/infra-foundation`** | **0** (the criterion) |
| `git merge-base --is-ancestor track/infra-foundation <branch>` | SUCCESS |
| push | `f2989d4..4b9f47a` |
| re-measure **after** the push | 0 behind, is-ancestor SUCCESS, local/remote 0/0 |
| card `acc-disp-2o_BDY1tb3b92ZQwaYXEC` | **pending**, branch/base intact |
| wash suite | **66 tests, 0 skipped, OK** |
| `s5_writein.py` | **87/87 records, 0 validation problems** |

## No conflict this time, and why that is worth writing down

The previous sync ended with me untracking `.saih/mcp.json`, which track still
holds. A merge across that asymmetry could have gone either way, so I checked the
three-way explicitly rather than trusting a clean exit:

| | blob for `.saih/mcp.json` |
|---|---|
| merge base | `0ec1df9…` |
| ours (my HEAD) | *deleted* |
| theirs (track) | `0ec1df9…` (unchanged) |

Base has it, my side deleted it, track left it alone → **the deletion wins**. That
is standard three-way behaviour, but it is not obvious in advance, and getting it
wrong would have silently re-tracked a file holding `SEATLOOM_ACTOR_REF` and
`DATABASE_URL`.

I briefly misread my own output here and want it on the record: my check printed
an empty `grep` result followed by my own `echo` describing what a re-add would
look like, and for one step I read my own label as the grep's output. The file was
never re-added. I caught it by re-running with `git ls-files --stage` and
`git write-tree`, which is the check I should have used first — a `grep` on
`git status` tells you what changed, not what is in the tree.

## The standing hazard, now precisely stated

The deletion is **stable only while nobody edits the file on track**. If anyone
commits a change to `.saih/mcp.json`, the next sync sees base=old / ours=deleted /
theirs=new and **conflicts again** — the same add/add class that cost the previous
sync. The file differs per seat by construction, so it cannot live in a shared
branch; the durable fix is to untrack it on track, which is the operator's call,
not mine. Recorded so the next seat does not rediscover it as a surprise.

## Note on the waker fix that came in

`88ce719` says the waker is a resident daemon, not a tmux session, and that it had
never been running. That is plausibly relevant to why my dispatch loop has been
driven by pokes rather than by the waker waking me, but I have not verified that
connection and will not assert it — it is Aegis's finding to confirm.

## On the count

Aegis labels this the eighth occurrence; by my tally it is the ninth sync today,
of which **2 were genuine staleness and 7 were the same race**. The arithmetic
does not matter much; the pattern does. His own dispatch says the operator has
still not ruled on whether his docs/scripts belong on track at all
(`ev-op-decision-aegis-commit-policy`), and until then the loop is structural. I
have no view on the policy question and should not form one — but each cycle costs
me a merge and a push, and costs him a dispatch, for a card the operator wants to
click.

## What I did not do

Did not rebase. Did not merge into track. Did not re-track `.saih/mcp.json`. Did
not open, cancel or modify any card. Did not touch the live database. Did not act
on Aegis's commit-policy question.
