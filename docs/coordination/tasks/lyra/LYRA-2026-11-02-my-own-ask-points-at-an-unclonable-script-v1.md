# Standing-duty verification: my own operator ask 1b points at a script that does not exist in git — and the same is true of the standing-duty landing script, so two of the asks in my ROADMAP are unclonable

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-my-own-ask-points-at-an-unclonable-script-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, self-correction, ask, defect |

## What he found that I missed

**Aegis's 19:00 heartbeat notes, almost in passing, that `scripts/bind-seatloom-seats.sh` is still an untracked
file — `??` in `git status`.** He was reporting why the binding defect had not moved. **I read it as a status
note. It is a defect in my own ask, and I only saw it because he wrote it down.**

## Measured, and it is worse than one script

```
register-fleet-seats.sh            tracked
bind-seatloom-seats.sh             UNTRACKED
land-aegis-packet-branch.sh        tracked
land-standing-duty-docs.sh         UNTRACKED
backup-unpushed-branches.sh        tracked
check-ledger-exists.sh             tracked
check-salvage-fresh.sh             tracked
fix-doc-headers.sh                 tracked
rename-stepplan-providers.sh       tracked
```

**Two of the scripts named in my ROADMAP's operator asks are not in git.**

- **`bind-seatloom-seats.sh`** — Aegis's, in ask 1b. `git log --all -- scripts/bind-seatloom-seats.sh` returns
  **nothing: it has never been committed anywhere.** It exists only in the primary checkout's working tree.
- **`land-standing-duty-docs.sh`** — **mine**, in ask 3. It is on `packet/lyra/milestone-summary`, which is not
  merged and which I already documented as 429 commits behind track. So it is not on track either.

**The consequence, stated plainly: if the operator clones this repository fresh, or works from any other
checkout, or the primary checkout's working tree is ever cleaned, both of those asks become unrunnable.** The
command in my ROADMAP returns `No such file or directory`. **An ask that names a file which is not in the
repository is not an ask — it is a note that was true in one working tree.**

## This is the failure I have been documenting all stretch, committed by me, in the artifact I wrote for the operator

**I have spent two days documenting that "a governance artifact is delivered when the consuming process can
read it, not when it is committed."** The consuming process here is the operator, in whatever checkout he uses.
**A script in one working tree's untracked set is exactly the artifact I said was not delivered.**

**And I wrote ask 1b into the ROADMAP this evening**, with the command, and the evidence link, and the
explanation of why the view is correct and the table is one row deep. **Everything in that ask is right except
that the thing it tells him to run does not exist in the repository he would clone.**

**The specific error: I measured the script's dry-run output — which worked, because I ran it in the one
checkout where the file exists — and concluded the ask was actionable.** That is the instrument-not-verified-
current family again, fifteenth instance, and it is the same shape as the hook that was installed in one
worktree and not the others: **verified where it exists, presented as a state.**

## What the corrected ask is

**Ask 1b is now two steps, and the first one is not the operator's:**

1. **Aegis commits `scripts/bind-seatloom-seats.sh`** to track. It is his script, his lane, and he has already
   verified it in both directions on a sandbox database. **I am not committing it for him** — that is his file
   and his shared-tooling decision.
2. **Then the operator runs `bash scripts/bind-seatloom-seats.sh --apply`**, which is the live-database write.

**Ask 3 is the same shape, and that one is mine to fix:** `land-standing-duty-docs.sh` needs to be on track,
which means it goes through the same landing path the documents do. **I am recording it as mine and I am not
doing it unsupervised**, for the same reason I have not rebased the docs branch: it is a shared-state write and
I report those with the command attached.

## What I did not do

I did not commit either script. I did not run `bind-seatloom-seats.sh --apply`. I did not edit the ROADMAP to
delete the asks — they are correct once the scripts exist, and hiding them would be worse than flagging them.
My measurements were `git ls-files --error-unmatch` on nine scripts, `git log --all` on the bind script, and
`git ls-tree` on my branch. `seats` is 9, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
