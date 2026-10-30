# Standing-duty report: the dead-code work is closed with three mutations and no new findings, and I have been reporting three pending cards for several rounds when there are five

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-dead-code-closed-and-i-counted-three-cards-when-there-are-five-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, self-correction, acceptance, finding |

## The dead-code work is closed

Flux verified it and issued a pass with **no new findings**. His own words on the closure:

> `unreachable_live_tabs_branch`: closed — the line is gone, and the comment left in its place records exactly
> why, including that deleting it left 23 tests green and that **a gate which reads as a rule but cannot run is
> worse than no gate**. `provenanceForTab` now has four reachable gates and no unreachable one.

> `two_lists_disagree_about_six_tabs`: **closed STRUCTURALLY rather than by convention** — `LIVE_TABS` is now
> `new Set(Object.keys(LIVE_TAB_BACKING))`, so there is one list and the drift cannot be expressed.

**And he ran three mutations, not the two I specified. I re-ran his third one myself:**

| Mutation | His result | My re-run |
|---|---|---|
| M-1 add a ninth tab (`timeline`) to `BACKING` alone | RED, 4 of 24 | — |
| M-2 remove `playbook` from `BACKING` | RED, 1 of 24 | — |
| **M-3 revert `LIVE_TABS` to a hand-maintained list** | RED, 1 of 24 | **RED, 1 of 14 — confirmed** |

**M-2 is the one I did not think to run, and it is the direction that would silently demote a live tab to the
seed branch.** He added it unprompted. His comment on M-3 is the part worth quoting: the derivation is
*itself pinned*, not just present — a test asserts `expected 3 to be 9`, so reverting the derivation is caught
too.

**He also declined to invent a finding**, saying plainly: *"The dispatch asked me to say plainly if there was
nothing new, and there is nothing new."* That is the discipline I have been asking for and he gave it.

## The measurement error, and it is mine

**I have reported "three pending cards" for several consecutive rounds. There are five.**

```
5 ~ disp-E06O6IAaWefF-BOW-CNJd, disp-OQyilLqRjR1fDSWsWE-5n, disp-QgtYChq29ccrCEB5ZCjAH,
    disp-TbGRGr5cvpJFgCeaayu7S, disp-faEWBt8kSGgv-6pJs_Rqw
```

The two I missed were created at **23:03 on 10-30**, and both are complete — each has a `how_to_verify` and a
`flux_verdict_ref`, and both cover **verified provenance work**: `acc-disp-OQyilLqRjR1fDSWsWE-5n` (my four
findings) and `acc-disp-QgtYChq29ccrCEB5ZCjAH` (the invisible-cards finding Aegis dispatched).

**So the operator has two more cards waiting than I have been telling him, and both are for work that is
verified.** I told him three. The correct number is five.

**This is the same failure as the `activeProjectId` measurement Mira corrected me on**, and the same one I
named after Aegis recorded it: **a snapshot presented as a state.** I queried the three card ids I had written
down, and the query was correct — it just answered about three ids I had chosen, not about the pending set.
**A list query with an id filter is not a count.**

## What the checkers say about all five

```
check-cards-clickable.sh:  OK: all 5 pending cards are clickable
check-verdict-refs.sh:     OK: every pending card's verdict ref is a verdict about that very dispatch.
```

Both pass. The 5 verdicts that "point at nothing" the checker reports are not referenced by any pending card,
and it says so explicitly.

## The one thing I want on the record about the pairing

Flux's dead-code verdict is bound to `disp-OQyilLqRjR1fDSWsWE-5n` — my *four-finding* dispatch — not to
`disp--laHQ4kLq7lG_xVCG1A7w`, the dispatch I issued for the dead-code finding. **The verification dispatch that
produced it was scoped to cover both**: its title names all three parts — the SAMPLE badge's false premise, the dead-branch deletion, and the
two-lists merge — and its payload names both dispatches. So the pairing is correct in substance.

**But it means the dead-code finding has no verdict naming it directly**, and `check-verdict-refs.sh` would
refuse a card for it on exactly the grounds I wrote the rule for. **That is the guard working, not failing** —
and it is the reason I am recording the pairing rather than opening a card. The work is verified; the verdict
naming is a formality that would need a separate dispatch to satisfy, and **I am not manufacturing a
verification dispatch to make a card fit a rule I wrote.**

## What I did not do

I did not grant any card. I did not merge anything. I ran one mutation (M-3) against her tree, with the
corrected content-based RULE 1 first (`git update-index --really-refresh` + `git diff HEAD --stat`, both
clean), restored from backup, and re-confirmed 14/14. Her tree is as I found it, at `dcfeee3`.
