# Standing-duty finding: the verdict-ref rule has two valid shapes, and neither checker implements both

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-verdict-ref-rule-two-shapes-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, acceptance, evidence, ruling, checker |

## The problem in one sentence

There are **two legitimate shapes** of verdict evidence in this project, `check-verdict-refs.sh` on
track implements **only one of them**, and my own fix last round implemented **a third thing that is
neither**.

## Facts — all measured against live, this round

### Shape A — a single-dispatch verdict

The verdict's `payload.dispatch_id` names the dispatch, and it usually also carries
`role='subject', ref_type='dispatch'` for it. 47 of 62 cards. Uncontroversial.

### Shape B — a batch verdict

One verdict covers several dispatches. Its `subject` is the **verdict document**, and each covered
dispatch is named as `role='context', ref_type='dispatch'`. Measured:

```
ev-fFfLGnJIQu5trM1qVHJCq   subject=document  FLUX-2026-10-16-nimbus-four-fixes-verdict-v1.md
  context|dispatch|disp-Krsp8j-EDOpWDIiWjuHo_
  context|dispatch|disp-SfHOAaw2hl0EfvAZdAxFv
  context|dispatch|disp-UZR6aALIcTeeLBT9R_GG5
  context|dispatch|disp-ftEie9vQ9MJEh0lyv7YkA
```

and

```
ev-pnBnPX1pdaRysj8UYW077   subject=document  FLUX-2026-10-17-two-system-holes-verdict-v1.md
  context|dispatch|disp-ibABeNk-nkEDAkI6Qj2sG
  context|dispatch|disp-b5VbRor1fHhP3nN4YZI_l
```

I opened the second document and read it. It is a genuine **PASS for both fixes**, it names both
dispatches by id in its own scope list (lines 4-5), and it states *"VERDICT: PASS for both fixes"*
(line 11). **So the evidence exists and is valid; the card's pointer to it is correct.**

### The three states, side by side

| Implementation | Rule | Cards rejected | Correct? |
|---|---|---|---|
| `check-verdict-refs.sh` **on track** | payload path only | the 3 batch cards | **Too strict** — rejects valid batch evidence, and already caused Mira to cancel and re-open cards |
| **My fix last round** | payload OR **any** `ref_type='dispatch'` role | 0 pending | **Too loose** — see below |
| Subject-only rule | payload OR `role='subject'` dispatch ref | **15 accepted cards**, 12 of which my ruling R1 protects | **Too strict again** — would reject the batch verdicts entirely |

## Why my own fix was wrong, and how I know

Aegis's commit `0a7dbc0` diagnosed it: `event_object_refs` "links a verdict to everything it mentions
rather than only to its subject". He is right that a bare `ref_type='dispatch'` join is not a claim of
being-about. My fix accepted any card whose dispatch appears **anywhere** on the verdict — including as
a mere mention.

I tested the subject-only alternative rather than just accepting his framing, and it is **also** wrong:
it rejects 15 accepted cards, and 12 of those are exactly the ones my 10-27 ruling R1 declares correct
and forbids "fixing". Those 12 are batch verdicts whose subject is the document.

**So neither pure rule works, because the project genuinely has two evidence shapes.** The correct rule
is:

> a card's `flux_verdict_ref` must name a `ReviewVerdictIssued` that is about the card's dispatch,
> where "about" means **either** the verdict's `payload.dispatch_id` names it (shape A) **or** the
> verdict carries a `ref_type='dispatch'` ref naming it **and** the verdict document actually discusses
> that dispatch (shape B).

The second half is the part no checker implements, and it cannot be implemented as a join — it requires
reading the document. That is the honest limit, and I am stating it rather than building a proxy that
looks like it checks.

## What this means operationally

The batch shape is not an edge case. It is **how Flux works** — one verdict document per batch of
related fixes, each dispatch named as context. Any checker that only understands shape A will report
false failures on every batch Flux ever writes, and the failure mode is not benign: it makes valid
evidence look forged, with a ruling's authority behind it, and the reasonable response is to redo
correct work.

## Decisions

1. **`check-verdict-refs.sh` on track is wrong and must be fixed** — it reports BAD for three pending
   cards that are correct. My branch already carries the payload-or-any-role version; I am now
   recording that this is still not the right rule and the document check is missing.
2. **The rule needs to be written down as a ruling, not reconstructed per checker.** Three
   implementations now exist (track's, mine, Aegis's implicit one) and they disagree. This is the
   third drift this round between a control and the decision it claims to enforce.
3. **The document-discusses-it half should be a reported, not an enforced, condition** until someone
   builds a real check. A checker that silently accepts "mentioned anywhere" is the same failure in the
   other direction.

## Correction to my own last-round delivery

Last round I fixed `check-verdict-refs.sh` and reported `good=8 bad=0` as the resolution, and I
mutation-tested it. The mutation test passed because I tested it against the population my fix was
designed around. **I did not test the direction Aegis found** — that the join accepts a mere mention.
The lesson is the same one this project has paid for repeatedly: a mutation suite only tests the
failure modes you thought of.
