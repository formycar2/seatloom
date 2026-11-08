# Standing duty: the two pending cards' branches have fallen from 3 to 6 commits behind track while I measured them, and **none of the six commits touches either deliverable file — the subjects are byte-identical to the commits their verdicts verified.** So the cards remain valid and the gap is bookkeeping, not substance. The gap grows because nothing merges, and merging is not mine to do.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-cards-are-still-valid-and-the-gap-is-bookkeeping-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 409 → 421 events and 80 → 83 dispatches.** My containment dispatch `disp-qPfPgpcXldYZ9SiX8Ksvw` is `in_flight`. `get_next_dispatch` returned nothing ready. **Two cards pending, and both instruments agree they are clickable.**

**And origin is unreachable again** — `Connection closed by 172.182.252.133 port 22` across several attempts. **My incident document `b42855b` is committed locally and the working tree is clean**, so the fourth-destruction record is safe; it is just not pushed. That is the same intermittent SSH condition recorded earlier and it is a transport problem, not a data problem.

## The measurement, and what it rules out

**Last round I reported both card branches 2 commits behind track. They are now 6:**

| branch | behind / ahead |
|---|---|
| `packet/nimbus/work-list-snapshot-actors` | **6 behind / 6 ahead** |
| `packet/nimbus/git-status-honesty` | **6 behind / 4 ahead** |

**The six commits are all coordination and incident work plus two real fixes:**

```
2389269  salvage: Nimbus landed the containment
973b699  correction(aegis): the verification fa
3455e92  finding(aegis): the gate hole I report
94d4c54  incident(aegis): the database was OOM-
0862921  fix(status): a missing waker log is no
90dcb29  fix(hooks): the card guard asked the q
```

**And I checked each of the six against both deliverable files: none touches them.** I also re-compared the deliverable blobs against the commits their verdicts name:

| file | verdict commit → branch tip |
|---|---|
| `scripts/work-list.sh` | **IDENTICAL** |
| `scripts/test-work-list.sh` | **IDENTICAL** |
| `scripts/prepare-verification-build.sh` | **IDENTICAL** |
| `scripts/test-prepare-verification-build-dry-run.sh` | **IDENTICAL** |

**So the cards' subjects are exactly what was verdicted, and the behind-count measures how much unrelated work has landed on track since — not any drift in the delivered work.** The corrected clickability criterion asks whether the merge would drop commits or delete files, and it says both cards are clickable, which is consistent with what I measured directly.

## What the gap actually is, stated plainly

**It is bookkeeping that grows on a timer.** Every commit that lands on track widens it, and nothing narrows it, because the only action that narrows it is a merge into the card branches — which writes shared state on two branches that have pending cards. **I have declined that action three times now, correctly, and the cost of declining is that the number keeps rising and looks worse than it is.**

**So the honest report is: the number is 6 and rising, the subjects are unchanged, and the merge that would reset it is one action belonging to the operator.** I am recording the number with its cause attached rather than either suppressing it or letting it imply the work has drifted.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `git rev-list --left-right --count` for both branches against track, `git log` for the six behind commits, `git show --name-only` for each of those six against both deliverable paths, and `git rev-parse <commit>:<path>` for the four blob comparisons. The push was attempted and failed; I did not force-push to gitlab, whose tip `717d9bf` is not an ancestor of my HEAD.
