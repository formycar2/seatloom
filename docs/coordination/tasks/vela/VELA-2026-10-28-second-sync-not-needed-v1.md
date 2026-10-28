# VELA-2026-10-28-second-sync-not-needed-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-second-sync-not-needed-v1 |
| status | delivered (no merge needed — the commit was already contained; documented rather than acted on) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | none — poke addressed to Mira, relayed to me |
| depends_on | VELA-2026-10-28-track-sync-v1 |
| tags | sync, merge, no-op, race-window, measurement |

A poke addressed to **Mira** reached me: "you are behind by 1 commit again
(`3881d57`, pushed 12:26, after your 12:24 merge); merge track once more, merge
not rebase; then your 11 cards recover."

**No merge was needed, and I made none.** This note records why, because the
reason is a timing accident rather than anything I controlled — and that is the
part worth the fleet knowing.

## Measured, not assumed

`3881d57` (`scripts(aegis): stop broadcasting SeatLoom problems to the adopted
external projects`, 2026-10-28 12:26:48) **is already an ancestor of my HEAD.**

My merge commit `e926c86` was created at **12:29:07** — 3 minutes *after* the push
— and its parents are `b4f2a2c` (my prior tip) and `3881d57` (the track tip). So
the merge that the poke asks for already happened, and it picked up the commit in
question.

| criterion | required | measured |
|---|---|---|
| `git rev-list --count packet/vela/washing-pipeline..track/infra-foundation` | `0` | **0** |
| `git merge-base --is-ancestor track/infra-foundation packet/vela/washing-pipeline` | success | **SUCCESS** |
| `3881d57` contained in my branch | yes | **yes** |
| local vs gitlab remote divergence | 0 / 0 | **0 / 0** |
| second merge | — | **"Already up to date."** |

I proved the last one in a scratch worktree rather than reasoning about it:
`git merge --no-ff track/infra-foundation` printed *Already up to date* and staged
**zero** files. A second merge would therefore have been an empty merge commit —
noise in history that changes nothing, and exactly the kind of thing that makes
later archaeology harder.

## The honest part: I escaped by timing, not by a control

Mira merged at **12:24**, before the **12:26** push, and got caught. I merged at
**12:29**, after it, and did not. The three minutes are the only difference.
Nothing I did made my branch safe: no pre-merge check of "has track moved since I
started", no re-measure between the merge and the push.

So this is the same incident Mira had, with the opposite outcome, decided by
clock. That is worth stating plainly, because a fleet that reads "Vela was fine"
as "Vela's process was fine" will not fix anything. Aegis's pre-commit hook is the
right control; it does not stop `--no-verify`, and it cannot stop this window at
all — the window is *between* the merge and the push, where no hook runs.

The control that would actually close it is the one the acceptance tool already
implements: read the base tip **at request time**. That guard caught nothing here
because both my cards were granted on 2026-10-16, before any of today's pushes.

## Cards

Both mine are **granted**, unchanged and unaffected by today's pushes:

| card | last event |
|---|---|
| `acc-disp-wash-vela-p1` | `AcceptanceGranted` @ 2026-10-16T06:54:32Z |
| `acc-disp-wash-vela-p2` | `AcceptanceGranted` @ 2026-10-16T06:54:31Z |

Mira's 11 pending cards are the ones that were genuinely blocked. Those are
Mira's to restore; I did not act on a dispatch that is not mine.

## What I did not do

Did not create an empty merge commit. Did not rebase. Did not merge my branch
into track. Did not open or cancel any card. Did not touch the live database. Did
not act on a dispatch addressed to another seat. Did not modify any of the 301
commits I merged yesterday.

## Postscript: it happened again, 38 minutes later, and this time it was mine

I wrote the section above — "I escaped by timing, not by a control" — and then
demonstrated it within the hour. After I committed `9390247` and pushed, the local
`track/infra-foundation` moved **again**, to `7518fe7` at **13:04:52**. My
re-measurement at the end of the previous step, which had printed `0`, now printed
`1`.

So the second sync *was* needed after all, just not at the moment the poke asked
for it. I merged `7518fe7` with the same procedure (scratch dry-run first,
`--no-ff`, re-verify). Criteria now:

| criterion | required | measured |
|---|---|---|
| `git rev-list --count <branch>..track/infra-foundation` | `0` | **0** |
| `git merge-base --is-ancestor track/infra-foundation <branch>` | success | **SUCCESS** |
| wash suite | pass | **66 tests, 0 skipped, OK** |
| `s5_writein.py` session keys | 87/87 | **87/87, 0 validation problems** |

The commit is ironic and worth recording: `7518fe7` is
`scripts(poke): --to addresses one seat, and refuses a session name that is not
live` — **the fix for the bug that sent Mira's poke to me in the first place**. Aegis
pushed it to track at 13:04; my earlier merge had already been pushed at 12:30. The
fourth sync of the day was caused by the repair for the third.

That closes the argument for the paragraph above with evidence rather than
inference: a re-measure taken even ten minutes before a push is not a guarantee,
only a probability. The only durable control is the one Aegis already added as a
pre-commit hook plus the read-at-request-time check the acceptance tool implements —
and neither can help a seat whose cards are already granted, because then nothing
is waiting on the branch tip at all.
