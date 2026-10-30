# Standing-duty finding: the mirror remote's `track/infra-foundation` is 24 hours and 20+ commits behind the real integration line, and I nearly measured the wrong one again

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-mirror-track-is-24h-stale-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, blocking, instrument, remotes, verification |

## What I found while doing something else

I went to check whether the four fixes Mira just made had landed on track. That required knowing where track
is. **It is not where I have been assuming, and the assumption was load-bearing.**

| Ref | Tip | Date |
|---|---|---|
| `track/infra-foundation` (local, the real one) | `052f6ff` | **10-30 17:37** |
| `origin/track/infra-foundation` (the mirror) | `4c13ffc` | **09-30 08:49** |

**The mirror is roughly 24 hours and more than 20 commits behind**, and its tip is a merge from September
30th.

## How I know the local one is authoritative

The reflog settles it — this is a branch the operator commits onto directly, not a tracking ref:

```
052f6ff track/infra-foundation@{0}: commit: docs(knowledge): a check returning empty must prove the query could return non-empty
8804c69 track/infra-foundation@{1}: commit: ops(aegis): supervisor-lane docs and scripts
4cda353 track/infra-foundation@{2}: merge packet/onyx/topic-opened-019: Merge made by the 'ort' strategy
be898cb track/infra-foundation@{3}: merge packet/nimbus/project-scoped-reads: Merge made by the 'ort' strategy
```

Commits and merges landing on it through the day. And the operator's own latest work
(`2a94c5f`, 10-31 03:04, "the operator's three cards cannot be found in the product, and his active project is a test fixture") is on
`packet/aegis/heartbeat-1800-un-gated-merges`, **not** on local track either — he is working on a packet
branch and has not merged it.

## Why this matters more than a stale ref normally would

**Every "is it on track yet" measurement I have made this week has been against the local ref, which is
correct — but I never verified that, and the one time I queried the remote I got a September 30th answer
and would have concluded the line was three weeks stale.** That is the eighth instance of one failure:

> **a measurement taken at a location that was not verified to be the location the claim was about.**

I wrote a whole document about this pattern on 10-30 after Aegis recorded it, and then walked into it again
the same day. **What caught me this time was that I typo'd the branch name** (`track/infa-foundation`), got
`fatal: unknown revision`, and had to re-derive the ref from scratch — which is the only reason I noticed the
remote disagreed.

## The compounding risk, stated plainly

The mirror exists so work is not on one machine. **If the mirror's track is a day stale, then a
"is it on track" check run against the mirror answers about a line that no longer exists** — and worse, a
seat that syncs from the mirror would build on September 30th.

I checked whether my own branches are affected:

```
packet/lyra/milestone-summary        local = origin   (equal)
packet/lyra/standing-duty-docs-only  local = origin   (equal)
```

**My branches are fine.** But I derived "my docs branch is 0 behind track" from the local ref, which is
right, and then separately confirmed the remote's track is stale — **so the two halves of that sentence
were measured against different things, and I only noticed because the second half looked wrong.**

## The ask

**Push `track/infra-foundation` to origin.** It is the primary remote's job to mirror the integration line,
and a mirror a day behind is worse than no mirror because it looks authoritative. The operator has been
pushing his own packet branches to both remotes all day; **track itself appears not to have been pushed
since September 30th.**

I am not doing it myself: pushing the integration branch is an operator action, and a seat pushing track
unasked is the same overreach class I have declined repeatedly.

## What I did not do

I did not push anything. I did not merge anything. I did not re-point any ref. I did not touch the three
pending cards.

## The measurement discipline, restated because I keep having to

**Before answering "is X on track":**
1. resolve the ref and **print its tip and date**;
2. confirm that ref is the one the claim is about — for track, that means the local branch the operator
   commits onto, **verified by reflog, not by name**;
3. if a remote disagrees, **the disagreement is the finding**, not noise to be picked through.

Step 2 is the one I skipped for most of a week.
