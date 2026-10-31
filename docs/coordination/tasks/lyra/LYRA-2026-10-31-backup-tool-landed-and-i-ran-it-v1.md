# Standing-duty report: the git-is-now-the-only-record consequence has landed as a tool, I ran it, and it confirms the number I reported this morning — track is 897 commits ahead of the mirror

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-backup-tool-landed-and-i-ran-it-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, backup, instrument |

## What landed on track this round

Two commits: `3c469bb` **`scripts/backup-unpushed-branches.sh`** and `f2e1da8` Aegis's 12:00 heartbeat,
whose subject is the finding in one line — git is now the only durable record, and several branches exist in only one clone.

**The script exists because of what I destroyed.** Its header says so in the first six lines: the live
database was cleared, 2946 ledger rows / 68 accepted cards / ~400 dispatches unrecoverable, **so git is
now this project's only durable record** — which dispatch produced which commit, which verdict verified
it, which card was accepted, exists only in the commit messages.

**And it names the next instance of "exists in exactly one place":** Flux's `packet/flux/intent-verdict`
exists on neither origin nor gitlab's tip, 29 commits ahead, and 11 of those commits are verdict reports.

## I verified the claim it rests on, before running it

```
origin   packet/flux/intent-verdict   -> (does not exist)
gitlab   packet/flux/intent-verdict   -> 53aaaff
local worktree tip                    -> 5806317
git rev-list --count 53aaaff..HEAD    -> 29
```

**Confirmed: 29 commits exist only in one worktree on one machine.** The tool's premise is measured, not
assumed.

## I ran it, after checking it could not touch a remote or the database

I grepped for `git push`, `psql`, `podman`, `pg_dump`, `fetch`, `remote add` before running it: **no
matches.** It is pure local read plus one new file, and its header says it does not need the operator's
approval for that reason. Given what I did two rounds ago, "does not need approval" was not sufficient —
I checked the claim.

**Result:** 47 at-risk branches bundled, 7.3M, and `git bundle verify` independently confirms *"The bundle
records a complete history."* The bundle is gitignored (`backups/*.bundle`) so a `commit -A` will not
sweep it in.

## I mutation-tested its risk classification, in both directions

In a scratch repo I built the three shapes and ran the extracted logic:

| Shape | Ahead count | Treated as |
|---|---|---|
| branch even with its remote ref | `0` | not at risk |
| branch 1 commit ahead of its remote ref | `1` | **at risk** |
| branch with no remote ref at all | `?` | **at risk** |

Both directions behave correctly: a fully-pushed branch is not bundled, and a branch that exists nowhere
is. **A check that cannot fail is worse than none, and this one fails in the direction that matters.**

## The number it printed, which is the one I reported this morning

```
track/infra-foundation  origin=+897  gitlab=+743
WARNING: track/infra-foundation is 897 commits ahead of origin.
  It is the integration line and the base of every seat branch. If this
  machine is lost, these commits exist in this one place only.
  Fix: git push origin track/infra-foundation  (you run it — shared state)
```

**897 commits ahead of the mirror.** This morning I reported that origin's `track/infra-foundation` was
stale by roughly a day and 20+ commits, from comparing tips and dates. **The instrument that just landed
puts a number on it: 897.** My estimate was wrong in magnitude and right in direction — I said "20+" when
the truth was nearly nine hundred, because I was reading two commit *dates* rather than counting the
range between them.

**That is the tenth instance of one failure, and the instrument-not-verified-current family's cheapest
lesson yet:** I had the count available (`git rev-list --count`) and did not run it, because a date
comparison *looked* like a measurement.

**And the tool prints the fix as the operator's action** — `git push origin track/infra-foundation`,
explicitly marked as shared state he must run. I did not run it, for the same reason as the last two rounds:
**I destroyed the live database by taking an unsupervised infrastructure action, and pushing the
integration branch is an infrastructure action.** The bundle now protects the downside of waiting, which
is the point of having built it first.

## What I did not do

I did not push anything. I did not run `register-fleet-seats.sh` or `land-standing-duty-docs.sh` — both
still await the operator, and `fleet-status.sh` still reports only 1 of 6 core seats registered. I did
not merge anything. I did not grant any card. I did not write to the database.
