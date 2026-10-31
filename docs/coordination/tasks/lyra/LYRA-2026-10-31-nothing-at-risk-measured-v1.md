# Standing-duty report: nothing new to act on, so I measured the thing that would hurt most if this machine died — and it is protected, with numbers

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-nothing-at-risk-measured-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, backup, measurement |

## The state, measured

```
now                       10-31 18:39
check-ledger-exists.sh    exit 0
seats registered          4 of 6
track/infra-foundation    052f6ff   (HEAD is the operator's packet branch, 50 commits ahead)
new commits since 18:00   none
```

**No dispatch, no new work from any seat, and both blockers unchanged.** By my own protocol, that is not a
reason to invent work — it is a reason to check the thing whose loss would cost the most and then stop.

## What I checked, and why this one

**Since the database was destroyed, git is the only durable record, and the loss of a branch that exists in
one clone is the next "exists in exactly one place."** Aegis built `backup-unpushed-branches.sh` for that, and
I have run it twice. This round I ran it again and then verified what it actually protects, seat by seat,
rather than trusting the count.

### Every seat's tip, and whether it exists anywhere else

| Seat | Tip | Committed | On origin | On gitlab | Unique commits |
|---|---|---|---|---|---|
| mira | `c9a7fa7` | 10-31 08:11 | yes | +17 behind | 0 (all in her branch + bundle) |
| nimbus | `a7353e9` | 10-28 11:06 | **no ref** | **no ref** | **0 — fully contained in the operator's branch** |
| flux | `5806317` | 10-31 08:13 | **no ref** | at `53aaaff`, **+29 behind** | **29 — in no other branch** |
| onyx | `e73f6d0` | 10-30 15:07 | yes | equal | 0 |
| vela | `dd41974` | 10-29 11:43 | **no ref** | equal | 0 |

**The two facts that matter:**

1. **Nimbus's branch exists on neither remote, but it is not at risk** — its tip is a strict ancestor of
   `packet/aegis/heartbeat-1800-un-gated-merges`, so `rev-list --count <operator-branch>..HEAD` is **0**. It has
   no unique commits. A branch with no remote ref and no unique commits is a name, not a loss.
2. **Flux's 29 commits exist in no other branch** — `rev-list --count` against the operator's branch, track, and
   Mira's branch all return 29. That is the real single point of failure, and it is exactly what Aegis's tool
   named this morning.

### And the protection is real, verified rather than assumed

I ran the tool, then checked the bundle independently:

```
git bundle verify backups/git-20261031-184051.bundle
  -> is okay
  -> The bundle records a complete history.
```

**And it contains Flux's branch at its actual tip, `5806317`** — the same 8 chars as his worktree HEAD. So the
29 unique commits are in the bundle.

**And every seat's tip predates the bundle** (mira 08:11, flux 08:13, both before 12:30), so nothing has been
committed since the last snapshot that this new one does not already cover. 6 bundles, 44M, all gitignored.

## What I did not do

I did not run `register-fleet-seats.sh --apply` or `land-aegis-packet-branch.sh --apply` — both still write
shared state and both are still the operator's. I did not merge anything. I did not push anything. I did not
edit any script. I did not write a finding, because there is nothing failing.

## The result, stated as a result

**The thing that would hurt most is protected, and I can name the one branch that is genuinely single-homed:
`packet/flux/intent-verdict`, 29 commits, in the bundle and nowhere else.** If this machine died tonight,
those 29 commits would be recoverable from `backups/git-20261031-184051.bundle`.

**That is a pass with numbers, and it is the whole of this round.**
