# Standing-duty finding: the backup tool is broken again — the fix I verified last round is gone from the working tree, and it ran ~90 times in a retry loop before dying. The recovery path has been unguarded for an hour.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-backup-tool-broken-again-and-ran-90-times-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, blocking, finding, uncommitted, loop |

## The state, measured three times with an absolute path

```
HEAD (c239074) bundle_args count:      0     <- the BUG is committed
working tree bundle_args count:        0     <- the FIX is gone from disk
git status --porcelain for the file:   0     <- not modified; it matches the buggy HEAD
line 150:  git bundle create "$OUT" "${at_risk[@]}" "${orphans[@]}"
```

**Three consecutive reads of the same absolute path agree.** Last round I reported the fix was uncommitted but
present in the working tree. **It is now gone from the working tree as well**, and the file matches the buggy
committed version exactly. Something reverted or discarded it between 20:42 and now.

**And the tool fails:**

```
$ bash scripts/backup-unpushed-branches.sh
scripts/backup-unpushed-branches.sh: line 150: orphans[@]: unbound variable
"bundle creation failed"
EXIT=1
```

## The loop, and why it matters more than the revert

**`backups/` holds 90 bundles, all from today, at roughly one per minute from 20:01 to 21:17 — then nothing.**

The driver was **Aegis's self-waking heartbeat in its own tmux pane**, which is erroring in a retry loop:

```
Error: All fallback models failed (last: network unreachable).
        Chain: stepplan/step-5-preview -> stepplan-wash/step-5-preview
Error: Retry failed after 3 attempts: Connection error.
Error: Connection error.   (repeated)
Aegis self-waking heartbeat — Read .../AEGIS-HEARTBEAT.md ...
```

**Each heartbeat invocation ran the backup script. Each run produced a 7.8M bundle. Ninety of them.** The
network went unreachable, the seat could not complete its turn, and it retried — and each retry did the
backup work again, because the backup is part of the standing duty it was told to perform.

**So the loop is not a bug in the backup script. It is a seat that cannot finish its turn, retrying, and
doing expensive work on every attempt.** 90 bundles × 7.8M ≈ **700MB of duplicated data in an hour**, on a
machine whose only copy of two weeks of integration history is already the risk.

**And then it stopped at 21:17 and has produced nothing since.** The last valid bundle is
`git-20261031-211639.bundle` — verified okay, complete history, but carrying only **1** orphan ref where the
19:42 one carried 6.

## The consequence, stated plainly

**The recovery path is currently unguarded.** The tool that produces the only off-machine copy of this
repository's history fails with exit 1 in the state a fresh clone is in. It has been failing since the working
tree reverted, and **the last successful bundle was 56 minutes ago.**

**And the fix that closes it exists in exactly one place in the history of this project: my own delivery
documents quote it verbatim** (`bundle_args=("${at_risk[@]}")` / `if [ "${#orphans[@]}" -gt 0 ]; then
bundle_args+=("${orphans[@]}")` / `fi`). It is not in any commit. **I verified that fix works, in three
directions, last round.**

## The ask, and it is now the only thing I am asking for

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
# re-apply the fix I verified last round, then commit and push it:
#   line 150 ->  bundle_args=("${at_risk[@]}")
#                 if [ "${#orphans[@]}" -gt 0 ]; then bundle_args+=("${orphans[@]}"); fi
#                 git bundle create "$OUT" "${bundle_args[@]}"
git add scripts/backup-unpushed-branches.sh
git commit -m "fix(backup): an empty array no longer fails the whole script"
git push origin packet/aegis/heartbeat-1800-un-gated-merges
```

**One file, one commit, one push.** The fix is quoted verbatim in
`docs/coordination/tasks/lyra/LYRA-2026-10-31-fix-is-correct-and-uncommitted-v1.md` and in
`LYRA-2026-10-31-my-own-stale-read-correction-v1.md`, and I mutation-tested it in three directions.

**And separately: Aegis's heartbeat retry loop needs a bound.** A seat that cannot complete its turn should
not retry expensive work every minute for 76 minutes. That is Aegis's to fix, and it is a different problem
from the empty array.

## What I did not do

I did not edit `backup-unpushed-branches.sh`, commit anything, or revert anything. **I did not delete any of
the 90 bundles** — that is destructive cleanup on the operator's disk and it is not my call, even though they
are 700MB of duplication. I did not run `--apply` on either operator script. `seats` is still 4 of 6,
`check-ledger-exists.sh` is exit 0.
