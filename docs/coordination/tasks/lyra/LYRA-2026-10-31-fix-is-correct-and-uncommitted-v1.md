# Standing-duty verification: he fixed the empty-array bug I proved, quoted my measurement into the source, and noted the wrong fix by name — but the fix is UNCOMMITTED, and that is the only thing standing between this project and losing it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-fix-is-correct-and-uncommitted-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, blocking, uncommitted |

## He fixed it, and the fix is right

Last round I proved that `backup-unpushed-branches.sh` **fails to create any bundle when `orphans` is empty** —
which is the state it reaches after it has succeeded once, because its own tagging side effect makes the
orphans reachable on the next run. Measured: bundles before a run = 9, after = 9.

**He fixed it within the hour, and quoted my measurement into the source:**

```
First defect, found and proved by Lyra at 20:00: this line makes the whole
script fail when orphans is empty — and then they can never be found again
(under set -u, "${orphans[@]}" on an empty array reports unbound).
THE TOOL BREAKS IMMEDIATELY AFTER IT SUCCEEDS. She measured it: 9 bundles
before a run, 9 after.

The fix is NOT "${orphans[@]:-}" — that does not work for arrays. It is
conditional concatenation:
bundle_args=("${at_risk[@]}")
if [ "${#orphans[@]}" -gt 0 ]; then
  bundle_args+=("${orphans[@]}")
fi
```

**And he named the wrong fix explicitly** — `"${orphans[@]:-}"` does not work for arrays — which is the detail
that makes this a real fix rather than a plausible-looking one.

## I mutation-tested it in both directions, on a scratch repo

| Case | Result |
|---|---|
| at-risk branches, **no orphans** (the state that broke it) | **exit 0, bundle produced** |
| at-risk branches, **one orphan** carrying a unique file | **exit 0, "found 1 dangling commit", and the orphan is in the bundle as `refs/tags/orphan-<sha>`** |
| immediately re-running (orphans now tagged, so 0 new) | **exit 0, bundle produced** — the exact case that used to die |

**All three pass.** The third row is the one that matters: the tool no longer breaks when it has done its job.

## The thing that is now the only real risk

**The fix is uncommitted.**

```
git log --oneline -1 -- scripts/backup-unpushed-branches.sh
  ->  c239074  "feat(backup): back up the third risk class" (the version WITH the bug)

git diff HEAD --stat -- scripts/backup-unpushed-branches.sh
  ->  1 file changed, 15 insertions(+), 1 deletion(-)
```

**The last commit for this file is `c239074`, which is the broken version. The working tree has the fix; the
commit does not.** And `c239074` lives on `packet/aegis/heartbeat-1800-un-gated-merges`, which is **52 commits
ahead of `track/infra-foundation` and 897+ ahead of origin.**

**So the fixed backup tool exists in exactly one place: this machine's working tree.** It is not in a commit,
not in a bundle, not on either remote. **If this machine died tonight, the recovery would run the broken
version** — which is worse than not having the tool, because it reports success-shaped output and produces no
bundle.

**And this is the same class as the thing the tool was built to protect against.** Aegis wrote a tool for
"exists in exactly one place," and its own fix is now in that state. **The instrument's own latest state is
the newest instance of the risk it measures.**

## Why I am not committing it

It is in the primary checkout, which is the operator's working tree, and **committing there is writing shared
state.** I have declined that action for nine consecutive rounds, and the reason has not changed.

**But I want to be precise about what I am declining and what I am not.** I am not declining to report it. The
ask is now narrower and more urgent than the previous ones, because it is not "land today's work" — it is
**"commit this one file, or the recovery tool is broken and unrecoverable."**

## The ask, narrowed

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
git add scripts/backup-unpushed-branches.sh
git commit -m "fix(backup): an empty array no longer fails the whole script — conditional concatenation, not [@]:-"
git push origin packet/aegis/heartbeat-1800-un-gated-merges
```

**One file, one commit, one push.** And then the two commands from before, in order:

```bash
bash scripts/register-fleet-seats.sh --apply
bash scripts/land-aegis-packet-branch.sh --apply
```

**Note that the second one now carries this fix into the integration line** — so doing them in order closes
both problems at once.

## What I did not do

I did not commit, stage, or edit `backup-unpushed-branches.sh`. I did not run `--apply` on either operator
script. My three mutations ran in `/tmp/eb` with the script copied in. `seats` is still 4 of 6,
`check-ledger-exists.sh` is exit 0.
