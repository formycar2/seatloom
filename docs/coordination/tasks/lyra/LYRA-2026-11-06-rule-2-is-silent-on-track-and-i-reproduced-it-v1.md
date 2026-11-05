# Standing-duty verification of Aegis's rule-2 finding, reproduced independently and in the exact state I created last night: he proved that the pre-commit hook's rule 2 — the guard against taking another seat's files with a glob — **does not apply on `track/infra-foundation` at all**, because `SEAT` is derived from a `packet/<seat>/...` branch prefix and there is none. **I reproduced it: a file from `docs/coordination/tasks/lyra/` staged in the primary checkout's index, on `track`, and the hook prints three OKs and exits 0.** It would have committed. Removed immediately, nothing committed, working tree and HEAD back as found.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-rule-2-is-silent-on-track-and-i-reproduced-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, measured, hooks, verification |

## Why this round is mine to do rather than his

**I caused the state he found.** Last night I staged a file of mine in the primary checkout's index to verify the doc
gate, and he saw it appear and traced it honestly, recording it as unexplained rather than inventing a mechanism. **This
round he recreated that state deliberately and measured what the hook does with it.** So the finding's origin is my
conduct and its measurement is his, and the verification is mine to close.

## What I measured, in the exact state I created

**On `track/infra-foundation`, in the primary checkout, a file from another seat's directory staged in the index:**

```
$ bash .git/hooks/pre-commit
EXIT=0
pre-commit: OK - rule 2: no other seat's files were taken by a glob
pre-commit: OK - docs touched by this commit conform
pre-commit: OK - no pending cards, nothing this commit can strand
```

**Three OKs, exit 0, and the file would have been committed.** Note that the second OK is also vacuous in this case —
the document I staged is a real, conforming one, so the doc gate had nothing to refuse. **Rule 2 is the only check that
could have caught it, and rule 2 is the one that is disabled here.**

## The mechanism, read rather than assumed

```bash
SEAT=""
packet/*/*) SEAT="${branch#packet/}"; SEAT="${SEAT%%/*}" ;;
...
docs/coordination/tasks/${SEAT}/*) continue ;;          # own seat: allow
docs/coordination/tasks/*/*) staged_elsewhere="..." ;;  # another seat: flag
```

**On `track/infra-foundation` the `packet/*/*` case never matches, so `SEAT` stays empty.** And with `SEAT` empty,
`docs/coordination/tasks/${SEAT}/*` expands to `docs/coordination/tasks//*`, which does not match a real path, while
`docs/coordination/tasks/*/*` still does — so the flag would be set.

**But it is not, because the whole block is behind `if [ -n "$SEAT" ]`.** Measured, the guard at line 94 wraps the
loop. So on `track` the rule is skipped entirely, by design, with the comment saying as much: *if no seat can be
derived, do not refuse — there is no basis for saying whose directory this is.*

## Why the design is right and the hole is still real

**The design is correct for its original purpose, and I am the one who earned it.** On 11-01 I showed that a rule which
refused when it could not attribute ownership would block every merge on the integration line, and merges must cross
directories. The name I gave that bug was *author privilege* and the fix was right.

**But it leaves the hole exactly where the most commits happen.** `track` is the integration line. Nothing on it checks
whether a file from another seat's directory was staged by something unintended. **Last night's entry would have sailed
through**, and so would mine.

**And the specific shape of the risk is not hypothetical for me**: had Aegis run `git commit -a` while my file was
staged in his index, my document would have landed on `track` under his authorship. That is the 11-01 failure — a change
that was not mine, in a commit, because of a glob — arriving through the one branch where the glob guard is off.

## What I am not doing, and it is the same line as the last three rounds

**I am not fixing the hook.** Two candidate fixes exist and both change a guard whose current shape was set by a ruling
that is not Aegis's — and is not mine either. **Choosing between them is the operator's, because the trade-off is his**:
either refuse on `track` when a foreign seat directory is staged (which risks blocking legitimate merges, the very thing
my 11-01 finding prevented), or make the hook state plainly that rule 2 does not apply here instead of printing OK.

**The second option is smaller and I think it is the honest one** — a guard that cannot run should say so, which is the
same principle I have applied to the exit-2 branch in the notification class and to the `behind` note in the
clickability gate. **But it is still a change to a shared instrument on `track`, in the hour he is working, so it goes
out as a dispatch rather than an edit.**

## What I did not do

I did not edit the hook, the gate, or any script. I did not commit anything — the staged file was removed from the index
and the worktree immediately after the measurement, verified with `git status --porcelain` (only the pre-existing
`?? scripts/bind-seatloom-seats.sh` remains) and `git log -1` (HEAD unchanged at `efe5044`). I did not merge, push,
accept, cancel, or re-open any card, restart any application, or write to the live database. My measurements were reads
plus one staged file with cleanup: `grep` on the hook for `SEAT` and its guard, the hook run on `track` with a foreign
file staged, and the restoration checks. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **46**.
