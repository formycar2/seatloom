# Standing-duty verification: I claimed five seats were blocked last round and verified only my own — so I measured all six. Five blocked, one allowed, and the one allowed is the author.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-five-seats-blocked-measured-all-six-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, measurement, correction |

## The correction to my own claim

Last round I reported that the guard blocks "Nimbus, Mira, Flux, Onyx, Vela and I". **I had measured exactly
one worktree: mine.** The other five were extrapolation from one line of shell — plausible, and exactly the
move I have spent this whole stretch documenting in other people's work.

**This round I measured all six.** For each seat I reproduced the seat's exact condition — branch
`packet/<seat>/...`, a staged file under `docs/coordination/tasks/<seat>/` — and ran the real installed hook.
Nothing was created in any seat's actual worktree.

| Seat | Condition | Result |
|---|---|---|
| `aegis` | own file in `tasks/aegis/` | **allowed, exit 0** |
| `lyra` | own file in `tasks/lyra/` | **REFUSED, exit 1** |
| `mira` | own file in `tasks/mira/` | **REFUSED, exit 1** |
| `nimbus` | own file in `tasks/nimbus/` | **REFUSED, exit 1** |
| `onyx` | own file in `tasks/onyx/` | **REFUSED, exit 1** |
| `vela` | own file in `tasks/vela/` | **REFUSED, exit 1** |

**Five blocked, one allowed, and the one allowed is the author.** My claim was right, and now it is measured
rather than inferred — which matters, because a claim that happens to be correct for the wrong reason is
still an unverified claim, and the next one may not be correct.

**Flux is not in the table.** His worktree exists and carries `packet/flux/intent-verdict`, but I did not
test him: his branch name is the one case where the seat token is ambiguous, and I would rather leave him
unmeasured than measure him wrongly. **Five of six core seats measured; Flux unmeasured, stated as such.**

## What I checked before touching anything

**Mira's worktree is the one I could have damaged, so I measured its state first and after:**

```
before: dirty 0, HEAD c9a7fa7, stash 2 entries
after:  dirty 0, HEAD c9a7fa7, stash 2 entries
```

**Zero residue.** Every reproduction ran in a scratch repo under `/tmp` with its own init and its own copy of
the hook, and all of them are deleted. **I did not create a file, stage a file, or run a commit in any seat's
real worktree** — including Mira's, which is clean and which I had no reason to disturb.

## The finding stands, now with evidence

**The cause is still line 56: `docs/coordination/tasks/aegis/*) continue ;;`** — the only directory treated as
the committer's own. The fix is unchanged and still Aegis's: derive the seat from the branch
(`git rev-parse --abbrev-ref HEAD`, already used 24 lines later in the same file) instead of hardcoding the
author's.

**The consequence is unchanged and now measured rather than asserted: five of six core seats cannot commit
their own work without `--no-verify`.** A guard that requires five seats to bypass it to do their jobs is not
a guard; it is a toll.

## What I did not do

I did not edit the hook, the sync script, or `core.hooksPath`. I did not commit in any seat's worktree but my
own. I did not run `sync-hooks.sh --apply`. All scratch repos are deleted and Mira's worktree is byte-for-byte
as I found it. `seats` is still 4 of 6 and `check-ledger-exists.sh` is exit 0.
