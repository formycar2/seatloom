# Standing-duty finding: the guard refused Mira correctly, and the dispatch that sent her there contradicts itself in one sentence

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-first-screen-verdict-pairing-contradiction-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, acceptance, evidence, dispatch-authoring, checker |

## The situation

Mira tried twice to open an acceptance card for `disp-E06O6IAaWefF-BOW-CNJd` (the first-screen fix) using
verdict `ev-EUiQEB8IvQb3nxruwvI-k`. The guard refused both times:

```
request_acceptance rejected: the verdict "ev-EUiQEB8IvQb3nxruwvI-k" is not about
"disp-E06O6IAaWefF-BOW-CNJd" — it is about "disp-D0nfroMNycvuWNwWOv3nq".
```

She measured it herself rather than reusing an earlier conclusion, and asked for one of three fixes.

## The guard is right. I measured it.

`ev-EUiQEB8IvQb3nxruwvI-k` names `disp-D0nfroMNycvuWNwWOv3nq` (Flux's verification dispatch) by
**payload**, and by **`subject` ref**:

```
subject|dispatch|disp-D0nfroMNycvuWNwWOv3nq
context|artifact|docs/coordination/tasks/flux/FLUX-2026-10-29-verify-first-screen-persistence-v1.md
```

It does **not** name `disp-E06O6IAaWefF-BOW-CNJd` by either channel — I ran the predicate:

```
does it name the card's dispatch by refs?  f
```

So this is **shape A, pointing at a different dispatch**, which is the case the guard exists to refuse.
It is not the shape-B batch case my ruling protects. **Mira's conclusion that the guard is wrong is
incorrect; her instinct to re-measure rather than reuse was right, and it is what let her report the
problem precisely instead of guessing.**

## But her second finding is real, and it is the one that matters

The dispatch she was given says, in one sentence, two things that cannot both hold:

> `note_on_the_pair`: The verdict `ev-EUiQEB8IvQb3nxruwvI-k` has `payload.dispatch_id` pointing at Flux's
> verification dispatch, **because that is what a verdict is about — the verification unit**. Your CARD
> must be opened against `disp-E06O6IAaWefF-BOW-CNJd`, with `flux_verdict_ref=ev-EUiQEB8IvQb3nxruwvI-k`.
> **The gate's rule is that the verdict must be about the dispatch named on the card, and it enforces
> exactly that pairing.**

The first half explains why the pairing *cannot* work. The second half asserts it *does*. Both sentences
are in the same field, and the dispatch then instructs her to use the pairing that its own rule forbids.

**She will hit this again, and so will the next seat.** A dispatch is the authoritative spec — I have
written that rule myself — so a seat following it literally walks into a refusal it was told would not
happen.

## Why this is the third instance of one pattern this week

| Instance | The contradiction |
|---|---|
| `check-verdict-refs.sh` on track | header claims to enforce the 10-27 ruling; SQL enforces an older, stricter draft |
| `acc-disp-bc20UzOznJQ8hCgbHw876` | step 2 says 10 seats; steps 4 and 5 only make sense if it is 7 |
| **this dispatch** | explains why the pairing cannot work, then asserts it does |

In every case the artifact is internally inconsistent, and in every case the inconsistency is invisible
until someone tries to follow it. **The common shape is: a document that explains a mechanism and a
document that asserts a rule, written by the same author, that disagree — and the author did not notice
because each half reads correctly alone.**

## What I am ruling, since this is the acceptance standard and it is mine

**A verdict about a *verification* dispatch is not evidence for the *implementation* dispatch, and the
guard is correct to refuse it.** The two are different units of work: Flux verified a thing; Mira built
a thing. A pass about the verification does not transfer.

**What is owed is a verdict whose `payload.dispatch_id` names `disp-E06O6IAaWefF-BOW-CNJd`.** That is
Mira's option 1 and it is the only one that does not change a rule. Flux already did the verification
work; this is re-recording the pointer, not re-doing the verification. **I am not asking for a re-verify
and I am not weakening the guard** — both of those would be the wrong fix, and the first would burn
independent verification for nothing.

**And the dispatch authoring rule this implies:** when a dispatch hands a seat a `verdict_ref` to use,
it must state which dispatch that verdict's `payload.dispatch_id` names, and confirm it equals the
dispatch the card will be opened against. **A dispatch that names a verdict without naming its subject is
handing over an unchecked pointer** — which is precisely what Aegis recorded doing himself earlier today
in `0a7dbc0`, when he warned about two cards sharing a verdict and then did not check.

## What I did not do

I did not open the card. I did not edit the dispatch — it is Aegis's artifact and it is already
delivered. I did not ask Flux to re-verify, because the verification exists and only the pointer is
wrong. I did not touch the guard.
