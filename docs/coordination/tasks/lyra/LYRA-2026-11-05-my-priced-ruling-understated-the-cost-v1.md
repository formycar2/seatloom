# Standing-duty correction of my own priced ruling, and the finding it exposes: my "three unclickable windows, 62 / 59 / 1 minutes" was **wrong**. Measured properly — windows computed only from the moment the cards existed, and time zones reconciled, because the database stamps UTC and git stamps local — there are **three real windows of 61.5, 58.9 and 61.1 minutes, plus two short ones I missed entirely**. The total is **5.0 hours**, not the two hours I published. **My number was wrong in the safe direction: it understated the cost by more than half.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-my-priced-ruling-understated-the-cost-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, correction, measured, gate |

## What I published last round, and what was wrong with it

I published three windows — 62, 59 and about 1 minute — and concluded the operator lost roughly two hours. Two
errors, both mine:

**Error 1 — I did not filter by whether the cards existed.** I took every track commit and the merge that followed
it. But the cards were opened at 07:15:50 UTC, which is **15:15:50 local**, and several of the windows I counted
opened before that. *A window in which nothing was blocked is not a window.*

**Error 2 — I mixed time zones without noticing.** The database stamps UTC; git stamps local (UTC+8). When I read
`a267b8d` at "16:02" and the card at "07:15" I treated them as the same clock, which they are not. **Two clocks read
as one is the instrument error I have recorded all week, and I made it in the round where I was measuring cost.**

**What the error did not do is flatter me.** It understated the cost.

## The corrected measurement

Windows counted only from the moment the cards existed (15:15:50 local), each window being one track commit to the
next Mira merge that absorbs it:

| window | opened | closed | length | note |
|---|---|---|---|---|
| 1 | 16:02 `a267b8d` | 17:04 `29ebc90` | **61.5 min** | the one I had |
| 2 | 18:03 `b73c037` | 19:02 `0d1cd1e` | **58.9 min** | the one I had |
| 3 | 20:01 `683422b` | 21:02 `d27c4a6` | **61.1 min** | **I had this as "about 1 minute"** |
| 4 | 22:03 `be81df8` | 00:02 `6c434a1` | **119.4 min** | **I had missed this entirely** |
| — | 17:03, 19:01, 00:01 | under a minute | ~0 | absorbed while she was already awake |

**Total: 5.0 hours, not 2.** Three merges were genuinely needed after the cards existed, and the third and fourth
windows are the longest of the day.

**Window 3 was the one I mis-read.** I saw "Aegis's 21:00 note, Mira's 21:02 merge, about one minute" and recorded
the short one — because the short one is the one that follows a long one, and I matched commit messages to merge
messages instead of computing the interval. **The 61-minute window at 20:01 was in front of me and I did not count
it.**

**Window 4 is the one I did not see at all**, because it closes after midnight and my `--since` stopped at "now."

## The finding this exposes, which is bigger than the arithmetic

**Aegis has now recorded the same structural conflict four times in his own notes, and he has stopped arguing it.**
His `AEGIS-2026-11-02-standing-backlog-v1.zhs.md` carries the rule he set for himself: *an unchanged obstacle should
not be re-argued in every heartbeat; only the status table updates, and a change is what reaches the body.* He moved
the conflict there and stopped restating the options.

**And that rule is now costing the operator five hours a day, measured.** It is the correct rule applied to a
situation where the obstacle is not going to resolve itself — so the correct behaviour is to stop repeating and the
correct outcome is that the operator keeps paying. **Both of those are true, and neither one is a bug in Aegis.**
He is doing what his own rule says. **The defect is that nothing converts a repeated obstacle into an escalation
that the operator must answer.**

## What that means for my ruling

**My ruling stands, and the case for it is now more than twice as strong as I argued it.** The proxy costs 5 hours
a day of the operator's time and three of Mira's merges, and the thing it proxies for is already checked at the
merge by a guard I verified in both mutation directions myself.

**But I am not going to quietly replace the wrong number with the right one.** The wrong number is published,
pushed, and cited. **The correction is this document, and the earlier one is not rewritten** — it is a snapshot that
was wrong, and the honest record is that it was wrong, not that it was always right.

## What I did not do

I did not edit my previous document, edit any script or product code, merge, push, accept, cancel, or re-open any
card, restart any application, or write to the live database. My measurements were reads: `git log --since` with
`--format=%ct` for epoch seconds on both track and Mira's branch, the card timestamps from `acceptances`, `date -j -u`
to convert, and the window arithmetic computed rather than eyeballed. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 26.
