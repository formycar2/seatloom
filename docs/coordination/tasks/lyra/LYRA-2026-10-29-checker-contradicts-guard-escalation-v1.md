# ESCALATION — the checker on track contradicts the product's own guard, and it has already cost three cards

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-checker-contradicts-guard-escalation-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | escalation, blocking, acceptance, evidence, merge-request |

## The situation in one paragraph

`scripts/check-verdict-refs.sh` on **track** checks only `payload.dispatch_id`. The product's own
acceptance guard — `flux_verdict_facts` in `crates/seatloom-core/src/db/repositories.rs:779` — accepts
**either** `payload.dispatch_id` **or** an `event_object_refs` row with `ref_type='dispatch'`. The
script is the outlier, and it says so in the loudest available words: *"Do not let the operator click
those."* **Three cards have been cancelled because of it**, all three correct under the guard.

## Evidence, measured just now

**1. The guard accepts both channels.** `repositories.rs:779`, its own doc comment states the rule:

> Both are things Flux actually writes — `payload->>'verdict'` and
> `payload->>'dispatch_id'`, or an `event_object_refs` row with `ref_type='dispatch'` — so neither
> criterion is invented. … **Either channel counts.**

and the code:

```rust
let names_dispatch = payload_dispatch_id.as_deref() == Some(dispatch_id) || names_via_ref;
```

**2. The script on track accepts only one.**

```
$ git show track/infra-foundation:scripts/check-verdict-refs.sh | grep -c event_object_refs
0
```

**3. Three cards were cancelled by it, and each cancellation names the script.**

```
acc-disp-ibABeNk-nkEDAkI6Qj2sG        18:04:32
acc-disp-b5VbRor1fHhP3nN4YZI_l        18:04:32
acc-disp-reemit-stalled-announcements 18:04:48
```

Each `feedback` field reads, translated: *"Aegis's check script (payload channel only) ruled BAD and
told the operator 'do not click'."*

**4. Mira's objection is correct and I am recording it rather than paraphrasing it.** She did not
simply obey the script; she checked and pushed back. Her `feedback` on
`acc-disp-reemit-stalled-announcements` states that the verdict's payload carries a
`re_emitted_events` key whose `both_edges_exist` names `ev-blocked-on-whom-dto-landed` and
`ev-reemit-stalled-announcements-landed` individually and whose `both_slices_left_blocked` names the
two dispatches leaving blocked — **so it genuinely verified her re-emission work**. And she cites
precedent: `ev-AbuUI51WIx1Ks-Eb_4FZp` was accepted through the refs channel by two cards that are
already merged.

Her words for why she cancelled anyway, translated: *"a card the operator cannot click is not a card."* **She cancelled correct work because the tool that was supposed to protect it made it
unclickable.**

## What is already fixed, and where

My branch `packet/lyra/milestone-summary` carries:

| Artifact | State |
|---|---|
| `scripts/check-verdict-refs.sh` | implements all three conditions, accepts both channels, prints which shape matched and names the verdict document for every shape-B card, has an `--include-accepted` scope. **Mutation suite 6/6 through the script.** |
| `LYRA-2026-10-29-verdict-ref-validity-ruling-v1.md` | the written ruling, both shapes named, the un-checkable half stated |
| `LYRA-2026-10-29-verdict-ref-rule-two-shapes-v1.md` | the analysis that produced it, including the correction of my own first fix |

**None of it is on track.** `on_track=0` for all of it. That is the whole problem: the fix exists and
the fleet is still being governed by the broken checker.

## Why this is not just "merge my branch"

I am **not** asking for my branch to be merged wholesale — that is the operator's call and it contains
five standing-duty documents plus four unrelated findings. What I am asking for is the narrow thing:

> **Take `scripts/check-verdict-refs.sh` from `packet/lyra/milestone-summary` and land it on track.**

That single file is the one standing between three correct cards and their acceptance, and between the
fleet and a checker that contradicts the product's own guard.

If the preference is for Aegis to write it rather than take mine, that is fine — but the content is
not in question any more. The guard's own doc comment states the two-channel rule; the script is the
only thing that disagrees with it.

## What I did not do

I did not edit `scripts/check-verdict-refs.sh` on track. I did not cancel or re-open any card — the
three are cancelled and `acceptances` rows are immutable once decided, so re-pointing them is churn. I
did not dispatch any seat.

## Second point, offered as a finding and not a demand

Aegis's `3d1f6b3` lands `scripts/check-card-verifications.sh`, which screens a card's
`how_to_verify` for steps that cannot fail. **That is the same class of defect I have been chasing all
round** — a check that cannot report an error — and it is the right instinct. It also means two of us
are now auditing checkers. I would rather that converge than duplicate: the verdict-ref rule is
written down in my ruling, and if `check-card-verifications.sh` is going to name scripts by path, it
should know that `check-verdict-refs.sh`'s rule changed.
