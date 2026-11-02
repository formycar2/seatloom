# Standing-duty verification: last round I published a defect I had not tested, so this round I mutation-tested the four checks that report green against an empty subject — all four work, and the test found one real trap that the production code path already guards

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-i-tested-the-four-empty-subject-checks-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, verification, mutation-test |

## Why this round exists

Last round I reported that `check-ledger-exists.sh` cannot see an empty ledger. It can — it has a branch for
exactly that state and I had not read far enough to find it. **The lesson I wrote was that a mutation test has
to come before the document rather than after.** This round is that lesson applied.

**The four checks that reported exit 0 last round were all green against an empty subject.** So the question
is not "are they green" — it is "would they fail if the subject were present and broken." **A check that cannot
fail is worse than none, and a check tested only on an empty set has never been shown to be able to fail.**

## How I built the subject

I built a scratch database from a schema-only dump of the live one, and inserted the minimum a real card
needs — a project, a seat, a dispatch in a legal status, and one pending acceptance whose branch does not
exist. Nothing was written to the live database, and the scratch databases are dropped.

**One thing worth recording, because it cost me four failed inserts:** the schema enforces more than the
columns suggest. `projects.created_at` is NOT NULL, `seats` has no `display_name` (it is `name`, plus
`capability_tags` and `status`), `dispatches.status` has a vocabulary that does not include `completed`, and
`acceptances` requires a non-empty `acceptance_plan` jsonb and a real dispatch FK. **Building a valid subject
is itself a measurement of what the schema actually requires, and I would not have known any of it from
reading a column list.**

## Result: the check works, in both directions

```
scratch: 1 pending card, branch packet/probe/does-not-exist ABSENT
$ bash check-cards-clickable.sh
FAIL: 1 of 1 pending cards are UNCLICKABLE - the operator's Accept is blocked
which branch / which card / why
packet/probe/does-not-exist|acc-probe-1|branch missing in this checkout
EXIT=1
```

**It fails, it names the branch, it names the card, and it says why.** Then I created the branch:

```
scratch: branch now exists but does not contain the base
FAIL: ... |behind 1
EXIT=1
```

**A second, different failure mode with a different reason.** Then I merged the base in and the reason changed
again. **Three states, three distinct answers, and the check never passed while the card was broken.**

**So `check-cards-clickable.sh` is sound, and its green against an empty set is honest: there are no cards, so
there is nothing to break.** That is a true statement and it is not evidence that anything works — which is
exactly what Aegis wrote in his 16:00 heartbeat, and he was right.

## The trap the test found, and why it is not reachable

**My probe used `base='track'`, which is the column's DB default.** `git merge-base --is-ancestor track
<branch>` fails, because **there is no ref named `track` in this repository** — the real branch is
`track/infra-foundation`. A card carrying the default can never pass this check.

**And the production code path already guards it.** `resolve_base()` in `dispatch.rs:2410` returns the explicit
argument, else `SEATLOOM_INTEGRATION_BRANCH`, else `DispatchRejection::MissingBase`. **It never invents a
value, and the rejection message at `dispatch.rs:546` names this exact trap:**

> *"pass `base` explicitly or set SEATLOOM_INTEGRATION_BRANCH ... The old hardcoded 'track' is not a branch in
> this repository, so the base is never invented"*

**So the DB default is a trap that the application layer refuses to step in.** The one place it can still be
reached is a test that inserts without `base` (`d2_acceptance_integration.rs:246`), and that test only asserts
the duplicate-pending constraint, so the default is inert there.

**I am recording it as a latent trap rather than a defect**, because that is what I measured: the default is
wrong, nothing currently writes it, and the next person who adds an inserter will inherit it silently. **The
fix is to change the column default to `track/infra-foundation`, or drop it and force the NOT NULL to do the
work — and that is Aegis's schema, not mine to change.**

## What I did not do

I did not edit the check, the schema, or `resolve_base`. I did not write to the live database. The probe
branch was deleted and both scratch databases dropped; the primary checkout is on `track/infra-foundation`,
dirty only with two files that were already untracked before this round. `seats` is 9, `canonical_events` is 1,
and `check-ledger-exists.sh` is exit 0.
