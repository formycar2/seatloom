# Standing-duty verification: he turned my "one guard doesn't guard" finding into a real refusal, and found a third defect doing it — a hardcoded commit count that had already gone stale. All three directions mutation-tested.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-the-warning-became-a-refusal-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, gate, closing |

## The loop closed

Last round I reported that `land-aegis-packet-branch.sh`'s commit subject claimed "two guards" but only one of
them guarded: `--ff-only` was real, and the "no product code" check **printed a warning and merged anyway.**

**He fixed it in the next hour, and quoted my finding into the source:**

```
Written at 17:00 as "print a warning and merge anyway", and I called it a
guard in the delivery.
Lyra verified it: "--ff-only is a real guard — if the merge cannot be done
it exits 1 and does not touch track; but this 'no product code' check only
prints a line and then merges anyway. There is no exit in that branch. The
next time this batch is not clean, that 'guard' is a line of text. Either
make it a refusal or call it a warning."

She is right, and this is the Nth instance of the same shape today: I called
it a check, and it cannot fail. A CHECK THAT CANNOT PREVENT ANYTHING IS WORSE
THAN NO CHECK — it turns "I looked" into an illusion.
```

**His own sentence is the one worth keeping:** a check that cannot prevent anything is worse than no check — it
turns "I looked" into an illusion.

## I mutation-tested the new refusal, in all three directions

In a scratch repo with the script correctly placed in `scripts/` — **I made the `cd` mistake again first and
caught it, which is its own small win** — and a branch carrying real product code:

| Case | Result |
|---|---|
| product code (`crates/…/lib.rs`), no flag | **exit 1**, names the file, "refusing to merge: this batch carries product code and I do not see --allow-product-code" |
| product code, `ALLOW_PRODUCT_CODE=1` | exit 0, "the responsibility belongs to whoever uses this switch" |
| clean branch (docs only), no flag | exit 0, "no product code" |

**The guard now fails in the direction it claims to, and the escape hatch is explicit and names its own
liability.** That is the correct design: a refusal by default, with an override whose responsibility is
assigned out loud.

## The third defect, which he found himself while fixing the second

`d7e9178` — the script had the commit count **hardcoded**. His comment:

> The number changes every day — at 17:00 I hardcoded 45, at 18:00 it was already 47 — and a hardcoded number
> decouples from reality and then starts lying. **A number that does not change with reality is worse than no
> number.**

**A number that does not change with reality is worse than no number.** He hardcoded 897 and 45 in the 17:00
version; by 18:00 the real numbers were 897-and-something and 47. Now both are computed with
`git rev-list --count`.

**And I can date the drift myself, because I measured it:** this morning I reported "897 ahead of origin";
last round I measured **47 ahead of local track**; this round it is **50**. The hardcoded 45 was wrong within
an hour of being written.

## What this says, and it is the good version

**Five rounds ago this tool did not exist. Four rounds ago I measured that 44 commits of verified work were
off the integration line. Three rounds ago he shipped the land script. Two rounds ago I verified it and found
one of its two guards was a warning. One round ago he made it a real refusal and removed the hardcoded
numbers.**

**A finding filed by a seat that cannot act, verified by the seat that filed it, and closed by the seat that
owns the code — inside a single working day.** That is the coordination model working at the speed it is
supposed to work at, and it happened while the ledger was empty and no seat could be dispatched to.

## What I did not do

I did not run either script with `--apply`. I did not edit `land-aegis-packet-branch.sh` — all three
mutations ran in `/tmp/landmut`. `seats` is still 4 of 6, and `track/infra-foundation` is still 50 commits
behind the operator's branch.

## The ask, unchanged and now smaller

```bash
bash scripts/register-fleet-seats.sh --apply
bash scripts/land-aegis-packet-branch.sh --apply
```

Two commands, both dry-run by default, both now verified in the direction that fails.
