# VELA-2026-10-27-acceptance-card-blocked-no-verdict-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-27-acceptance-card-blocked-no-verdict-v1 |
| status | delivered (card NOT opened — blocked on a verdict that does not exist yet; Flux verification issued) |
| author | vela |
| date | 2026-10-27 |
| version | v1 |
| dispatch | disp-NLKM7uxSqT9IdosS21iXa |
| depends_on | VELA-2026-10-26-message-count-hole-b-not-mine-v1 |
| tags | acceptance, gate, blocked, verdict, escalation |

Aegis 2026-10-27: one of my completed dispatches (`disp-2o_BDY1tb3b92ZQwaYXEC`, the
89-NULL `message_count` investigation) has no acceptance card. Open one with
`base=track/infra-foundation`; the criterion is a row in `acceptances` with
`status=pending`.

**I did not open the card, and this note says why and what I did instead.** The
short version: the card cannot be opened, because `request_acceptance` refuses
without a Flux verdict and no verdict for this dispatch exists. I issued the Flux
verification dispatch that would produce one. The card is one tool call away, the
moment Flux answers.

## Why the card cannot open

`request_acceptance`'s guards, quoted from the tool:

> `flux_verdict_ref` must name an existing `ReviewVerdictIssued` event ... whose
> `payload->>'verdict'` is `'pass'` AND which is about THIS dispatch
> (`payload->>'dispatch_id' = <dispatch_id>` ...)

I measured the ledger before trying anything: 59 `ReviewVerdictIssued` events,
**0 bound to `disp-2o_BDY1tb3b92ZQwaYXEC`**. My two prior passing verdicts are
bound to `disp-wash-vela-p1` and `disp-wash-vela-p2`:

| event | verdict | dispatch_id |
|---|---|---|
| `ev-SWfX7pNZH8PObLcGxg3pY` | pass | `disp-wash-vela-p1` |
| `ev-flux-verdict-wash-p2` | pass | `disp-wash-vela-p2` |

Borrowing either is exactly what the dispatch forbids ("never borrow another dispatch's verdict") and
exactly what the guard refuses. So the dispatch's own instruction — "you must
first have Flux verify it" — is not a preamble, it is the blocking step. I took it
literally rather than reaching for a nearby ref.

## What I did instead

Issued a verification dispatch to Flux, the same mechanism I used for
`disp-wash-vela-p1-flux-verify` and `disp-wash-vela-p2-flux-verify` (both
`SeatDispatchIssued` by `seat:seat-vela-001`, 2026-10-10, and both succeeded):

| field | value |
|---|---|
| dispatch id | `disp-2o_BDY1tb3b92ZQwaYXEC-flux-verify` |
| target seat | `seat-flux-001` |
| status | **ready** (no dependencies, so it is pullable now) |
| pre-declared verdict id | `ev-verdict-vela-message-count-hole-b` |

The payload hands Flux the five claims of the delivery and asks it to attack
them, specifically: whether the sentinel really is load-bearing (claim 2 —
double-counting), whether my CSV-matching rule is the right one (claim 3 — the
89 is derived from *my* rule, not 021's), whether "never extracted" is a scope
decision or an extraction bug (claim 4 — if a bug, the 89 changes), and whether
"nothing to count" holds for gemini/opencode. It also asks the meta-question: was
escalating instead of performing the write the right call.

`verdict_event_id` is pre-declared, so Flux must emit its verdict under exactly
that id — the tool mints and validates it, and a gated self-dispatch can hang on
`dep_kind=event, dep_value=ev-verdict-vela-message-count-hole-b`.

## The card, ready to fire

Everything except the verdict is settled, so that nothing is left to reconstruct
later:

| parameter | value |
|---|---|
| `dispatch_id` | `disp-2o_BDY1tb3b92ZQwaYXEC` |
| `base` | `track/infra-foundation` |
| `flux_verdict_ref` | `ev-verdict-vela-message-count-hole-b` (once emitted) |
| `acceptance_id` | omit — derived as `acc-disp-2o_BDY1tb3b92ZQwaYXEC` |

**`base` is verified against the real remote.** A trap here worth recording:
`origin/track/infra-foundation` is a stale GitHub ref from **2026-09-30** (tip
`4c13ffc`), and diffing against it makes this branch look 244 commits ahead. The
live remote is `gitlab`; its `track/infra-foundation` tip is `b09ec6e`
(2026-10-09). Against the real base my branch is **0 behind / 90 ahead** — it
contains the base tip, which is the branch-contains-base guard the acceptance
tool reads *at request time*. My 2026-10-14 delivery recorded the same lesson
("a sync made yesterday is not good enough today"), so I re-checked rather than
assuming.

The acceptance plan will follow the plain-language shape I used for the two cards
I re-issued on 2026-10-14 (scenario / steps / expected / preconditions, shape
v1), written for an operator clicking through a read-only investigation rather
than a code change.

## What I did not do

Did not open an acceptance card without a verdict. Did not fabricate, invent or
borrow a verdict ref. Did not modify any row or touch the live database. Did not
merge anything. Did not write a migration. Did not widen the washing scope.

## Honest note on the criterion

The dispatch's stated criterion — a row in `acceptances` with `status=pending` —
**cannot be met by me in this turn**, and not for a reason that more effort would
fix: the gate needs a second seat's verdict, and that seat has not run yet. If
the fleet counts this dispatch as incomplete because the row is absent, that is a
measurement artefact, not unperformed work. I would rather report that than
manufacture a green.

This also interacts with the rule question Aegis mentions assigning to Lyra: a
pure read-only investigation is arguably a "verification-exempt" class, in which
case the verdict requirement is the wrong shape for this dispatch entirely. That
ruling is Lyra's, and until it lands the old rules apply — which is what I
followed.
