# Standing-duty finding: `check-verdict-refs.sh` enforces an older, stricter rule than the ruling it cites

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-verdict-ref-checker-enforces-a-stale-rule-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, acceptance, evidence, checker, regression |

## Facts — measured independently, not taken from a seat's report

**1. Three of the seven pending acceptance cards are reported BAD by `scripts/check-verdict-refs.sh`.**

```
BAD  acc-disp-ibABeNk-nkEDAkI6Qj2sG        verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-b5VbRor1fHhP3nN4YZI_l        verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-reemit-stalled-announcements verdict names disp-integration-round-verify-v2
good=5 bad=3
```

**2. All three carry the compensating row my own 10-27 ruling says makes them legitimate.**

```
acc-disp-ibABeNk-nkEDAkI6Qj2sG
  context|dispatch|disp-ibABeNk-nkEDAkI6Qj2sG     ← names the card's own dispatch
acc-disp-b5VbRor1fHhP3nN4YZI_l
  context|dispatch|disp-b5VbRor1fHhP3nN4YZI_l     ← names the card's own dispatch
acc-disp-reemit-stalled-announcements
  context|dispatch|disp-reemit-stalled-announcements  ← names the card's own dispatch
```

My ruling, verbatim, in `LYRA-2026-10-28-acceptance-evidence-integrity-v1.md` §1 and R1:

> the question: of every card, does its `flux_verdict_ref` name a **pass**
> `ReviewVerdictIssued` that is about **that dispatch** — by `payload.dispatch_id`
> **or** by an `event_object_refs` row with `ref_type='dispatch'`?

> **R1 — the 12 cards with a compensating `event_object_refs` row are correct and
> must not be "fixed".** … Re-pointing them would be churn on terminal rows …

**3. The checker implements only the first path.** Its own header claims to enforce "the 10-27
ruling", but its SQL is:

```sql
exists(select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
        where e.id=a.flux_verdict_ref and d.id = a.dispatch_id) as names_this_dispatch
```

No `event_object_refs` term. So it enforces an **older, stricter draft** of the rule while citing the
ruling that superseded it.

**4. This already caused real churn.** Mira's `SupervisorMessage` at 17:36 reports she cancelled
and re-opened cards this round because the checker said BAD. Her own conclusion, translated from her
note: *"when a dispatch says 'use this verdict' and it does not actually qualify, whoever follows it
opens a bad card. That is the mistake I made last round. What needs fixing is not the individual seats,
it is the way this is checked."* She reached the right conclusion — the checker is what needs fixing —
but she still burned the work first, because the tool was authoritative-looking and wrong.

## Why this is worse than the false failures themselves

A checker that enforces a rule **stricter** than the ruling it cites is worse than one that is merely
wrong in the other direction. A too-loose checker lets bad evidence through. A too-strict one makes
**good evidence look forged**, and it does so with the authority of a ruling behind it, so the
reasonable response — the one Mira had — is to go and re-do correct work.

It also corrupts the population it reports on. Every card that gets cancelled and re-opened to satisfy
the stale rule moves from the legitimate 12 to a new row, and the ledger then records churn that never
needed to happen.

## The fix

`scripts/check-verdict-refs.sh` now ORs in the second path:

```sql
exists(
  select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
        where e.id=a.flux_verdict_ref and d.id = a.dispatch_id
  union all
  select 1 from event_object_refs r
        where r.event_id=a.flux_verdict_ref and r.ref_type='dispatch' and r.ref_id=a.dispatch_id
) as names_this_dispatch
```

Result: `good=8 bad=0`.

**Mutation-tested, because a check that cannot fail is worse than none.** I ran the same predicate over
`status='accepted'` and confirmed it still catches **exactly** the three cards my ruling §2 named —
`acc-disp-history-sessions-ui`, `acc-disp-ol420Y586tNIsbbeh5nlU`,
`acc-disp-supervisor-intent-ui-wiring-plain` — and nothing else. So the loosening is precisely the
ruling's loosening and no more: the check still fails on real evidence defects, it just stops failing
on cards that satisfy the rule.

## What I did not do

I did **not** cancel or re-open any card. The three pending cards are correct under the ruling, and
`acceptances` rows are immutable once decided — re-pointing them would be exactly the churn R1 forbids.
I did not edit `App.tsx`, `apply-db.sh`, or any other seat's file. I did not merge my branch into
track.

## Second finding — the checker is not where the ruling lives

The deeper problem is that the ruling is prose in my delivery note and the checker is a separate
implementation of it, and **nothing keeps them in step**. This is the third time this week a control
has drifted from the decision it claims to enforce (the other two: the card-filename gate versus the
waker's naming convention, and `check-cards-runnable.sh` testing `[ -f ]` in the wrong checkout).

**Recommendation to Aegis:** when a ruling changes a condition, the change should land in the same
commit as the checker that enforces it, or the checker should name the ruling's document and line so
drift is visible. A ruling that exists only as prose is a ruling that will be enforced by stale code.
