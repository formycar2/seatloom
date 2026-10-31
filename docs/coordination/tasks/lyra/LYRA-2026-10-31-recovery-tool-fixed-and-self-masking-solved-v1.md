# Standing-duty verification: the recovery tool is fixed, works, and carries 11 orphan refs — and he solved the self-masking problem I had identified but not fixed. Verified in three runs on a scratch repo.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-recovery-tool-fixed-and-self-masking-solved-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, recovery, closed |

## The state, re-measured

```
now                                10-31 23:12
check-ledger-exists.sh             exit 0
seats registered                   4 of 6
track vs operator branch           track ahead 0, HEAD ahead 52
new commits since 19:19            none

scripts/backup-unpushed-branches.sh
  bundle_args count (absolute path):  3
  dirty vs HEAD:                      1   (11 insertions, 3 deletions)
  bash scripts/backup-unpushed-branches.sh -> EXIT 0, bundle produced
newest bundle:  git-20261031-231303.bundle  (verified okay, complete history, 11 orphan refs)
```

**Two rounds ago the tool was broken and produced nothing. It now works, and the newest bundle carries 11
orphan refs** where the 19:42 one carried 6 and the 21:17 one carried 1. **Aegis's heartbeat is `Working`
again** — the network recovered and it is running.

## He did more than the fix I asked for

I reported the empty-array bug and quoted the conditional-`bundle_args` fix. **He applied that, and then
fixed the second problem I had identified but explicitly declined to solve** — the one I called *"consider
not counting the script's own tags as reachable, so the number it reports is a property of the repository
rather than of how many times it has been run."*

Two changes:

```bash
REACHABLE="$(git for-each-ref --format='%(refname)' refs/heads refs/tags refs/remotes \
              | grep -v '^refs/tags/orphan-')"          # own tags no longer mask orphans

done < <({                                                    # previously-tagged commits are
  git fsck --no-reflogs --unreachable 2>/dev/null | awk '/unreachable commit/{print $3}'
  git tag -l 'orphan-*' | sed 's/^orphan-//'                 # re-discovered, so the run is idempotent
} | sort -u)
```

**Both parts are the right shape.** The first makes the reported number a property of the repository. The
second makes re-running idempotent instead of lossy — before it, a second run would find zero orphans because
its own tags had made them reachable, and the tool would break on the empty array.

## I verified all of it, in three consecutive runs

On a scratch repo, with a synthetic orphan carrying a file that exists on no branch:

| Run | Result |
|---|---|
| **no orphans at all** (the empty-array case that broke it) | **exit 0, bundle produced** |
| **run 1** with a real orphan | "found 1 dangling commit", tagged, **exit 0** |
| **run 2** — orphan now tagged, the self-masking case | **still reports 1 orphan**, exit 0 |
| **run 3** | still reports 1 orphan, exit 0 |

**And the orphan is genuinely recoverable, not just reported:** `git bundle verify` says okay with a complete
history, `git bundle list-heads` shows the `orphan-<sha>` ref, and `git cat-file -t <sha>` retrieves the
commit object from the bundle.

**My first two attempts at this test were invalid and I discarded them rather than reporting them** — one
orphan had no file changes so the unique-content filter correctly excluded it, and one `git reset --hard`
deleted the untracked `scripts/` directory. Neither is a property of the tool. **The rule from last round
applied to my own test construction: a probe must both be reachable by the rule and violate it.**

## The remaining ask, narrowed to one command

**The fix is in the working tree and uncommitted.** That is the same state as two rounds ago, and it is the
only thing still standing between this project and a guarded recovery path:

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
git add scripts/backup-unpushed-branches.sh
git commit -m "fix(backup): empty array no longer fails the script, and own tags no longer mask orphans"
git push origin packet/aegis/heartbeat-1800-un-gated-merges
```

**Then the two operator commands, in order** — `register-fleet-seats.sh --apply`, then
`land-aegis-packet-branch.sh --apply` — the second of which carries this fix into the integration line.

## What I did not do

I did not edit, stage, or commit `backup-unpushed-branches.sh` in the primary checkout. All three test runs
happened in `/tmp/ot3` with the script copied in. I did not delete any of the 90 bundles. `seats` is still 4
of 6.
