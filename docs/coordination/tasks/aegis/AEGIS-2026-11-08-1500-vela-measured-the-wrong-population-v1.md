| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1500-vela-measured-the-wrong-population |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, docs-gate, measurement, stuck-dispatch |

# Vela's doc-gate change is right in shape and measured the wrong population, and their dispatch has been stuck for thirteen hours

## The stuck dispatch

`disp-pTLbm6ciZmDivez6F7rZJ` is `in_flight`, started 2026-11-07 19:13:37. It has a
`SeDispatchDelivered` event from 19:13:37 — the same second. So the work was handed over, and
the status was never closed.

That is precisely the defect the heartbeat protocol's closing step was written for, on the seat
side of it this time. I did **not** rewrite their ledger row — that is not mine to do. I poked
their pane with a comment line through the same channel the self-wake uses.

## The work itself

Vela's change to `scripts/verify-docs.sh` is careful and the shape is right. It teaches the gate
the knowledge convention, applies the convention's measured defaults for the keys frontmatter
cannot supply, and — the part I care most about — it keeps refusing. I verified all three
directions in an isolated copy:

| case | result |
|---|---|
| knowledge file with full frontmatter | refused — **but should have passed** |
| knowledge file with no frontmatter at all | **refused** — correct |
| knowledge file with frontmatter missing required keys | **refused**, naming what is missing — correct |

The two must-fail directions work. The must-pass direction does not.

## Why, measured

Vela's comment says the five keys frontmatter cannot supply are constant, verified with `sort -u`
across the files that carry a header table. That measurement is sound **for that population** and
then generalised past it:

| population | count | carries `author`? |
|---|---|---|
| knowledge files **with** a header table | 27 | yes — in the table |
| knowledge files, frontmatter only | 62 | **no `author` line at all** |

So `author` is not a constant of the convention. It is absent from the population the change is
meant to help.

Of the 62 frontmatter-only files, the keys that actually block are:

| blocking key | count |
|---|---|
| `id` | 27 |
| `tags` | 9 |
| `date` | 1 |

So **37 of the 62 still fail** after the change, down from 63. The change is real progress and it
is not the fix it reads as.

Separately, 26 of the files fail on CJK text and "template not in T1–T7" — those are the `.md`
(non-`.zhs`) knowledge files carrying Chinese titles, a different rule, not this one.

## What I did not do

I did not edit Vela's file, did not commit for them, and did not add the remaining failures to the
grandfather list. Adding 37 lines to make the number look smaller would be the same move I
refused two hours ago.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Both pending cards clickable.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.
- Push: 2 commits local-only; the retry loop is still working through the intermittent network.

## Needs you

- **The sandbox rebuild.** The guard is now correct across every path spelling and fails closed,
  so my side is unblocked. It is still a 95-byte fake, and six verified deliverables are waiting
  behind it.
- **Which remote is the source of truth.** gitlab is 935+ commits behind and I have not pushed
  there.
