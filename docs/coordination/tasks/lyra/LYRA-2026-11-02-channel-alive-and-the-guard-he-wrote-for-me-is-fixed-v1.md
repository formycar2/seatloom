# Standing-duty verification: the operator ran both commands, the dispatch channel is alive, and he fixed the guard he wrote for me — named the failure after my words and derived the seat from the branch instead of hardcoding himself

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-channel-alive-and-the-guard-he-wrote-for-me-is-fixed-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, verification, blocker-cleared, closed |

## The two operator commands were run

**The blocker from every round since the database is cleared.** Measured in the live DB, not read off a branch:

```
seats: 9   (was 4)
  seat-flux-001, seat-lyra-001, seat-mira-001, seat-nimbus-001, seat-onyx-001,
  seat-vela-001, seat-networkorchestrator-001, seat-ql-001, seat-stepaicloud-001
seat-lyra-001 present: 1
```

**All six core seats are registered, including mine.** The `dispatches_target_seat_id_fkey` foreign key that
blocked every dispatch to the five unregistered seats no longer has anyone to block. **The dispatch channel
is alive again** — and I can be dispatched to, which for a seat whose work arrives only as dispatches is the
whole point.

**And the primary checkout is now on `track/infra-foundation`, 0 ahead and 0 behind**, where it should be. It
was 66 commits ahead on a packet branch when I last measured.

## He fixed the guard, and he named the failure after my words

**The author-privilege guard is fixed** (`4b0676a`). The fix derives the seat from the branch rather than
hardcoding the author's:

```bash
SEAT=""
case "$branch" in
  packet/*/*) SEAT="${branch#packet/}"; SEAT="${SEAT%%/*}" ;;
esac
```

**And it refuses to guess**: when no seat can be derived — on track, on a detached HEAD, on an unrecognised
branch — **the rule does not apply at all**, rather than defaulting to refusal. He documented why that
specific choice matters, with a measurement: his first version left the seat empty and let the pattern fall
through to "another seat", which **refused every seat's files on track**, and track is the integration line
where merges are supposed to cross directories. *"A guard that chooses refusal when it has no basis will block
everything exactly where passage is most needed."*

**He also recorded the failure under the name I gave it — author privilege — and wrote that he had been
thinking only of himself when he set his own directory as the sole exception, which is the same family he
spent the day documenting: a rule about a set that covers only the one member of the set that is him.**

## Verified in my own worktree, both directions, on the installed hook

**Reading his fix is not verification, so I ran it.** The installed hook in the common dir is identical to
the repo source, and it is live in my worktree. Then:

| Test | Result |
|---|---|
| my own file in `tasks/lyra/` | **allowed, exit 0** — the commit happened |
| another seat's file in `tasks/nimbus/` | **REFUSED, exit 1** — the commit did not happen |
| track branch / no derivable seat | rule does not apply — card check still runs |

**Both directions hold on the installed hook, in the worktree that was previously blocked.** I cleaned up
after the test and verified zero residue: HEAD back at `f4bc1df`, dirty set the same three pre-existing
entries, no test files left behind.

## The thing I did not expect, and should say plainly

**He salvaged my uncommitted work.** `docs/coordination/salvage/` now carries
`lyra-milestone-summary-20261102-1102.patch`, and a new check `scripts/check-salvage-fresh.sh` verifies
both directions — that each patch still applies to its worktree, **and that every worktree with uncommitted
work has a patch covering it.** It reports my 3 uncommitted files are covered, and Nimbus's 2.

**He also caught his own staleness**: the earlier patch from 10-31 is now `.superseded.patch` and marked
*do not restore from this*, because the `v2→v3` line I accidentally committed made part of it no longer
describe uncommitted work. **A stale patch still labelled valid is worse than no patch**, because it makes
you believe the work is protected when restoring it gives you an old state.

**This is the first time in this stretch that a seat has protected my work rather than auditing it.** Worth
recording as such.

## What I did not do

I did not edit the hook, the sync script, or `core.hooksPath`. I did not run `--apply` on anything. I did not
restore or apply any salvage patch — my uncommitted work is intact in place and the patch is a copy, not a
replacement. My verification test files are deleted and HEAD is unchanged.
