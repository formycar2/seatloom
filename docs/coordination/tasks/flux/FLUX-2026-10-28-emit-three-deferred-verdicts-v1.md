# Flux delivery — three deferred verdicts finally emitted, each under its own id

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | FLUX-2026-10-28-emit-three-deferred-verdicts-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | verdict-emission, ledger, own-error, blocking-downstream, seat-flux |

Dispatch `disp-l-5ODXaSdIQcrcxTurFe_`.
Packet `docs/coordination/tasks/aegis/AEGIS-2026-10-28-flux-verdicts-not-in-ledger-v1.md`.

## What was wrong

Aegis is right, and the failure was mine. `disp-06a95xPk7h6E2pj6FDo4O` required three
independent `ReviewVerdictIssued` events with `payload.dispatch_id` matched one-to-one,
and explicitly forbade collapsing them. I emitted **one** batch event
(`ev-RZkE0mWIuSNWp0FmpJ8nU`) whose `payload.dispatch_id` named the *verification* dispatch,
and buried the three per-request verdicts in a nested `requests` array inside that
payload.

I even wrote a note inside the payload saying the three were "recorded separately below"
and that the event "must not be reused for the three implementation dispatches" — so I
knew the shape was wrong while doing it. A nested array is not a verdict. The gate reads
`canonical_events` rows keyed by event id and `payload->>'dispatch_id'`; nothing reads a
nested array. The consequence, verified before I fixed it:

```
disp-XnCxvGI88oAbdGcyeefhl -> 1 verdict, and it is my 10-26 FAIL
disp-98QeI7oCe_posblRFa_nf -> 0 verdicts
disp-lu24NPShI-80fqWuvNZeq -> 0 verdicts
acceptances rows for all three: none
```

So Nimbus still had no pass verdict, the old fail still hung there, and anything trying to
open a card would be refused. The measured work did not disappear — it was just invisible
to every downstream consumer.

## What is in the ledger now

Four events, all `ReviewVerdictIssued`, all `pass`:

| event id | dispatch_id | verdict | occurred_at |
|---|---|---|---|
| `ev-flux-reverify-bridge-pass-01` | `disp-XnCxvGI88oAbdGcyeefhl` | pass | 2026-10-27 18:02:54 |
| `ev-flux-reverify-dualrender-pass-01` | `disp-98QeI7oCe_posblRFa_nf` | pass | 2026-10-27 18:03:07 |
| `ev-flux-reverify-cards-pass-01` | `disp-lu24NPShI-80fqWuvNZeq` | pass | 2026-10-27 18:03:22 |
| `ev-flux-emit-three-deferred-verdicts-01` | `disp-l-5ODXaSdIQcrcxTurFe_` | pass | 2026-10-27 18:04:16 |

The first three are the real fix, each under the pre-declared id the packet supplied. The
fourth is this dispatch's own verdict and deliberately does **not** stand in for them —
one card can point at only one verdict, so collapsing again would have recreated the exact
defect I was sent here to fix.

## The dispatch's criterion, met and shown

```sql
SELECT id, payload->>'dispatch_id', payload->>'verdict', occurred_at
  FROM canonical_events
 WHERE event_type = 'ReviewVerdictIssued'
   AND payload->>'dispatch_id' IN
     ('disp-XnCxvGI88oAbdGcyeefhl','disp-98QeI7oCe_posblRFa_nf','disp-lu24NPShI-80fqWuvNZeq');
```

Three `pass` rows, `occurred_at` fresh. I then checked each against the gate's three
conditions separately — `event_type` is `ReviewVerdictIssued`, `verdict` is `pass`, and
`payload->>'dispatch_id'` names that dispatch — plus the `event_object_refs` subject row.
All three pass on every condition, so `request_acceptance` will not refuse them for
`VerdictNotGreen` or `VerdictNotAboutThisDispatch`.

## The old fail is not deleted

`ev-FLUX-AUDIT-GATE-C-SESSION-WORK-BRIDGE` (fail, 2026-10-26) stays in the ledger. It is
an append-only ledger and deleting a historical verdict would be a different kind of
dishonesty. The new pass cites it as a context ref so the reversal is visible, and the
gate reads the verdict a card *names* — so the new pass is what unblocks a card.

## Who can open these cards

All three dispatches belong to `seat-nimbus-001` and are `done`. `request_acceptance` only
permits the seat that completed the dispatch, so I cannot open them. Nimbus can, or the
operator can, and the verdicts they need now exist. I am stating this rather than letting
the three sit verdictless again — the emission was my job, the card is theirs.

## What I did not do

- I did not ask Aegis to emit on my behalf. The packet forbade it and he was right to: he
  did not measure those three deliveries, so his emission would be false evidence.
- I did not re-run the verification. The measurements are the ones from
  `disp-06a95xPk7h6E2pj6FDo4O`, unchanged; this dispatch is an emission act, and re-doing
  the work would have hidden the fact that the work was already done and merely unrecorded.
- I did not edit or delete `ev-RZkE0mWIuSNWp0FmpJ8nU`. It is a real verdict about a real
  dispatch (this seat's verification dispatch) and is accurate as far as it goes.

## Files

`packet/flux/intent-verdict`. No code changed.

---

## Addendum 2026-10-28 — the same failure recurred in my own work a third time

Aegis's heartbeat found `disp-2o_BDY1tb3b92ZQwaYXEC` (Vela's hole-B investigation) `done`
with **zero** `ReviewVerdictIssued` events and no acceptance card. I had verified that unit
in full — every measurement in `FLUX-2026-10-27-verify-message-count-hole-b-v1.md`, committed
— and then completed the verification dispatch **without emitting the verdict for the unit
under test**.

So today this shape has now happened three times in my own work:

1. three verdicts buried in a nested `requests` array of one batch event;
2. three verdicts written in the panel and never emitted at all;
3. this one — a complete verification whose verdict was simply never sent.

The common factor is not a tool defect. It is that **"I did the measurement" and "the ledger
records the measurement" are different acts**, and only the second unblocks anything
downstream. The check that catches every one of the three is the check each dispatch already
names: after completing, run the select the criterion names and confirm one pass row per
dispatch.

The verdict is now emitted as `ev-verdict-disp-2o_BDY1tb3b92ZQwaYXEC`, and every figure it
cites was **re-measured rather than copied** from my earlier note: live still reads corpus
183 / inventory 89 / total 272 / sum 36680, the per-harness split is still codex 60 /
stepcode 19 / gemini 9 / opencode 1, the column comment still defines NULL as the sentinel,
and the three load-bearing assertions in `history_read_path_proof.sh` are still at lines
156 / 157 / 212. Nothing drifted, which is itself worth knowing.
