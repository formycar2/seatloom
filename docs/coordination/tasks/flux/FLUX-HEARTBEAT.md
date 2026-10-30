# FLUX-HEARTBEAT — standing duty

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | FLUX-HEARTBEAT-v1 |
| status | active |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | flux |
| tags | standing-duty, heartbeat, flux |

> Standing duty for the flux seat, issued 2026-10-28 under
> `disp-8ekIOxWQQE852HwLwCX8y` (Aegis escalated the question: deciding what a seat is FOR when
> nobody dispatches it is a product-and-roles judgement, not a scheduling one). This file is the
> standing duty; `docs/coordination/roles/FLUX.md` remains the role.

# FLUX-HEARTBEAT.md — the verifier seat's standing duty

> Written 2026-10-28 by Lyra under `disp-8ekIOxWQQE852HwLwCX8y`. The role stays in
> `docs/coordination/roles/FLUX.md`. When the queue is empty, the waker calls you here.

## What you are for when nobody is dispatching you

**You are the seat that notices when a green verdict in the ledger is not evidence.** The queue sends
you work to verify. Your standing job is the evidence nobody asked you to re-check: verdicts that point
at nothing, cards whose criterion was never run, and checks that report OK over a history they have not
read.

You are the only seat whose standing duty is to distrust the ledger. That is not an insult to anyone —
it is the job. Every other seat's output is a claim, and claims are what you test.

## Standing checks — run these in order, each one able to FAIL

**1. No verdict in the ledger points at nothing.** You already wrote the instrument:
`scripts/check-verdict-refs.sh`. Run it, and read **all** of its output, not just the first section.

- PASS: zero verdicts that name a non-existent dispatch, and zero verdicts with no `dispatch_id` at
  all.
- FAIL, report: each wild or empty pointer, with the event id, the emitter, and whether any card
  references it.

**Measured today, so this check is not hypothetical:** that script currently reports **5 verdicts that
point at nothing** — `ev-030`, `ev-031`, `ev-XBrImD2cxJfuEGtgf7go2`, `ev-rKtx-DQARUF142u6lN3x5`
(EMPTY, they name nothing), and `ev-usP7dX7geulRhaDy1eXwD` (WILD — it names
`disp-B4Y5oAhMQwtXm5Vfa1fw`, one character off from the real `disp-B4Y5oAohMQwtXm5Vva1fw`). A wild
pointer is worse than an empty one: the moment a dispatch appears with that id, the verdict silently
becomes evidence for unrelated work.

**2. The verdict-ref check covers the whole ledger, not only pending cards.** Its first section filters
`status='pending'`, so its "OK" is silent about decided cards.

- PASS: a run over accepted and cancelled cards too, reported separately, that either finds nothing or
  names every card whose evidence does not name it.
- FAIL, report: decided cards whose `flux_verdict_ref` is not a verdict about them.

**Measured today:** 20 cards cite a verdict whose `payload.dispatch_id` names a different dispatch.
**12 of those are legitimate** — they carry a compensating `event_object_refs` row naming their own
dispatch, which the 2026-10-27 ruling accepts. **3 accepted cards genuinely have no valid evidence
about them** (`acc-disp-history-sessions-ui` and `acc-disp-ol420Y586tNIsbbeh5nlU` cite a **FAIL**
verdict; `acc-disp-supervisor-intent-ui-wiring-plain` cites a pass about a narrower unit). Lyra's
`scripts/check-cards-runnable.sh --include-accepted` is the same shape for a different question and can
be copied.

**3. No acceptance card's criterion is un-runnable.** Every command and every git ref named in a
pending card's `acceptance_plan` and `how_to_verify` must resolve.

- PASS: `bash scripts/check-cards-runnable.sh` exits 0.
- FAIL, report: the card and the exact string that does not resolve.

**Measured today, the defect this check was built for:** three cards told the operator to run
`git rev-parse track/infa-foundation` — missing an `r` — so he would have died on step 1 of a card whose
claim was true. The gate validates the verdict ref, the ownership and the plan's *shape*; it never
validated that the refs the plan *names* exist.

**4. Every gate event that claims a measurement contains the measurement.** A `-live` gate must carry
`gate` / `statement` / `verified` / `does_not_assert`, where `verified` names the call made, what it
returned, on which database, when.

- PASS: every gate event's `verified` field names a real call and a real result.
- FAIL, report: a gate whose `verified` is prose, or whose type does not match its id suffix
  (`-landed` must be `ArtifactCreated`, `-live` must be `SupervisorMessage`).

## When all four pass and there is still no dispatch

**Stop.** Do not re-verify work that already has a valid verdict about it — that burns the
independence you exist to provide, and a verdict issued twice is not stronger evidence, it is the same
evidence with a newer timestamp. Do not weaken a criterion to make something pass. Do not write a
verdict about a dispatch you were not asked to verify.

If a check fails, the output is a **finding with the exact query and its output**, never a repair: name
the event id, the card, the number. Evidence defects go to Lyra (she owns the acceptance standard);
routing goes to Aegis. You do not become a second Aegis by having standing duties — and you never
issue a verdict about your own dispatch.
