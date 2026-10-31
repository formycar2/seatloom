# Standing-duty correction: I read a stale value this round and nearly reported that a fix had been lost. It has not been — it is still uncommitted, exactly as I said last round, and both directions are now proven

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-my-own-stale-read-correction-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, self-correction, instrument, uncommitted |

## What happened, and it is the point of this document

This round I ran a check on `backup-unpushed-branches.sh` and it reported **zero occurrences of `bundle_args`**,
which is the marker of the fix. I concluded the fix had been lost, and started investigating a "regression."

**It had not been lost. The file was modified, not reverted.** Re-running the same check a few commands later
gave **three** occurrences. The first reading was wrong.

**This is the instrument-not-verified-current failure, in my own hands, in the same round I was checking
someone else's tooling.** And it is the twelfth instance of one family this stretch — but it is the first one
where the stale reading was *mine*, about *my own* check, rather than about someone else's artifact.

**What made it a stale read rather than a wrong one:** I ran the check with a relative path from a shell whose
working directory had changed between commands, and the file on disk had not changed. The check itself was
correct; **the thing it was pointed at was not the thing I thought it was pointed at.** That is the same shape
as the `git status --porcelain` false clean and the gate that could not see its own repository.

## The correction, with both directions proven

**The state is exactly what I reported last round: the fix is uncommitted.**

```
HEAD (c239074), the committed version:   bundle_args count 0   -> the BUG is committed
the working tree:                        bundle_args count 3   -> the FIX is here
git diff HEAD --stat:                    1 file changed, 14 insertions(+), 1 deletion(-)
```

**And the two versions behave differently, proven on a scratch repo with no orphan tags:**

| Version | No orphans present | Result |
|---|---|---|
| working tree (fixed) | yes | **exit 0, bundle produced** |
| `HEAD` / `c239074` (buggy) | yes | **exit 1, `orphans[@]: unbound variable`, no bundle** |

**So the recovery path on this machine is correct, and the committed path is broken.** If this machine died
tonight, whatever ran `backup-unpushed-branches.sh` from a fresh clone of the committed branch would fail in
the state where there is nothing to salvage — which is precisely the state a fresh clone is in.

## The ask, unchanged and still the narrowest thing outstanding

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
git add scripts/backup-unpushed-branches.sh
git commit -m "fix(backup): an empty array no longer fails the whole script — conditional concatenation, not [@]:-"
git push origin packet/aegis/heartbeat-1800-un-gated-merges
```

**One file, one commit, one push.** Then, in order:

```bash
bash scripts/register-fleet-seats.sh --apply
bash scripts/land-aegis-packet-branch.sh --apply
```

**The second command carries this fix into the integration line**, so the two asks close together.

## The rule I am adding to my own list

**Before reporting that something changed, re-read it with an absolute path and print the value.** Not because
the first reading is usually wrong, but because *"it changed"* is a claim about a transition, and a transition
requires two readings that agree about what they were reading. I had one reading and an assumption about the
second.

**And specifically: a check whose output is a count is a snapshot. A snapshot is not a state.** I have written
that sentence about other people's instruments eleven times this stretch. This is the first time I caught
myself doing it inside the same round, and the only reason I caught it is that the second reading disagreed
with the narrative I had already started building.

## What I did not do

I did not commit, stage, or revert anything in the primary checkout. I did not delete any orphan tag — I
explicitly declined to, because deleting tags to reproduce a failure state on shared state is the class of
action I destroyed the live database with. Both versions were tested in `/tmp/eb2` with the script copied in.
`seats` is still 4 of 6, `check-ledger-exists.sh` is exit 0.
