# Delivery — dispatch `disp-WHcJJ8S6Woo6Mkn5--DC1` (Lyra → Flux): the independent review exists and is excellent, and it is still not a verdict

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | LYRA-2026-11-04-verdict-gap-dispatch-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | dispatch, verdict, measured |

**Filename note.** The AGENTS.md completion protocol names this file `<dispatch-id>-delivery.md`, but
`scripts/verify-docs.sh:74` requires `ROLE-YYYY-MM-DD-<topic>-vN.md` under `docs/coordination/tasks/`. The two
conflict; I follow the gate and record the conflict here. The dispatch id is in the title and tags.

## What I dispatched

`disp-WHcJJ8S6Woo6Mkn5--DC1` → `seat-flux-001`, priority 7. Landed `ready`, now `in_flight`.

**Title:** Your review is not a verdict: the ledger has zero `ReviewVerdictIssued` events, so none of today's
work can reach an acceptance card.

## Why this dispatch exists

**Aegis asked Flux for an independent file-level review of Mira's two deliveries. Flux delivered it, and it is
the strongest artifact produced today.** Nine mutations rather than the two requested; he caught that M-3
survives and then **proved** it equivalent by measuring reachability instead of reporting it as a gap; he caught
that the delivery note's `expect.anything()` rationale is false because arity is pinned.

**And none of it can be used, because the acceptance gate reads the ledger and the ledger has no verdict.**

Measured at dispatch time:

| measurement | value |
|---|---|
| `canonical_events` with any verdict type | **0** |
| `acceptances` | **0** |
| dispatches `done` | **3** |
| `verdict_event_id` on any of the three | **none** |

**`request_acceptance` Guard 2** (`crates/seatloom-mcp-emit/src/dispatch.rs:1843-1875`) requires
`flux_verdict_ref` to name a `ReviewVerdictIssued` that exists, has that exact `event_type`, has
`payload->>'verdict' == "pass"`, and **is about that dispatch**. A markdown file satisfies none of the four.

**So the chain is: dispatch done → Flux emits a pass verdict → the seat opens a card → the operator accepts. It
has been broken at step two for 48 hours, and today's excellent review did not repair it because a review is not
an event.**

## The shape I sent him, measured from the guard's own code

I read `repositories.rs:808-845` so the ask was not a guess. **A verdict counts for a dispatch iff it names the
dispatch, through one of two channels Flux actually writes:**

- **channel A** — `payload->>'dispatch_id'` equals the dispatch id
- **channel B** — an `event_object_refs` row with `event_id = <verdict id>`, `ref_type = 'dispatch'`,
  `ref_id = <dispatch id>`

Either counts. `lane` is free text and the guard explicitly rejects matching on it.

**Two dispatches, two verdicts.** `disp-XpAdx0a9YLL83JVoPesTj` (the four write paths — his §1, §2, R-1, R-2) and
`disp-Ra5YbN4t78bzdbYhYkQiV` (the two frontend cases — his §3, M-1 through M-9). **I told him explicitly not to
emit one verdict and point both at it** — the guard checks the naming channel per dispatch, and a verdict about
a different dispatch is precisely the defect Guard 2 was written to close. Its own comment records three pending
cards that cited a **fail** verdict about a dispatch Flux had never verified.

**And I told him a red verdict is a legitimate outcome.** If he believes either dispatch does not hold, emit
nothing for it. *I would rather have "fail" than a "pass" you do not believe.* The gate exists to prevent an
unearned green, not to produce one.

## What I told him I had already done

So he would not duplicate it: **I independently ran his two Rust mutations and my own two frontend mutations** —
hardcode the call site, and empty the project argument. Both directions red on both, restored byte-identical,
baselines back to 30. **His reproduction and mine agree, which is the point of independent verification.**

## Boundary stated in the dispatch

No acceptance cards (that is Mira's — the guard is on `target_seat_id`), no merge, no push, no operator
application restart, no live-database write beyond the verdict events themselves. And verify afterwards with a
`select` against `canonical_events`, not with a memory of the call.

## Why I did not do this myself

**Emitting a `ReviewVerdictIssued` is a quality judgement about another seat's work, and I am the product
owner.** A verdict I wrote about Mira's code would be the author grading their own homework through a different
name. **The gate is only meaningful if the green comes from the seat whose job it is to be adversarial** — which
is why Flux's review matters and why my verification of it does not substitute for his verdict.

## Verification of the dispatch itself

```
dispatches: disp-WHcJJ8S6Woo6Mkn5--DC1 | seat-flux-001 | in_flight
canonical_events: SeatDispatchDelivered | seat:seat-flux-001   (newest)
```

## What I did not do

I did not emit any event, open any card, modify any seat's worktree, merge, push, or write to the live database
beyond this dispatch. My measurements were reads: `psql` for verdict events, acceptances, dispatches and their
`verdict_event_id`; reading `dispatch.rs:1843-1875`, `repositories.rs:808-845`, and `dispatch.rs:1095-1105`;
and reading Flux's review at `e4736fa` in full.
