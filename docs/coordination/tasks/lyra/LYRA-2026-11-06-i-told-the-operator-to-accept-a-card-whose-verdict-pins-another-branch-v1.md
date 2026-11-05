# Standing-duty correction of my own advice from one round ago, which was wrong in a way that would have cost the operator an invalid acceptance: **I verified the deletion-guard card and told the operator to click Accept. Nimbus has since withdrawn the card himself, and his reasons are three, all of which I can confirm — and the third is one I measured last round and did not treat as a problem.** The verdict the card cites, `ev-hW75U60DweOY5-qM5pc5r`, pins `6f1c566` on `packet/nimbus/project-scoped-reads`, while the card names `packet/nimbus/deletion-guard-accept`. **I checked that the verdict event existed and stopped there. "The reference resolves" is not "the reference is about this branch."**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-i-told-the-operator-to-accept-a-card-whose-verdict-pins-another-branch-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, correction, measured |

## What happened

**The card is now `cancelled`**, decided by `seat:seat-nimbus-001` at 22:01:58 UTC, with no merge commit and a feedback
field naming three defects, all his. **He withdrew it rather than leaving the operator to discover them.**

**And one round ago I told the operator to click Accept on it.**

## His three defects, each independently confirmed by me

| # | his claim | my measurement |
|---|---|---|
| 1 | the plan names `1cee50c` while the branch tip is `bc4c0a4`, and the branch carries more than the guard | **confirmed** — branch tip is `bc4c0a4`; the diff against `track` includes `scripts/prepare-sandbox-build.sh` (+7) and a 132-line delivery doc, not the guard alone |
| 2 | the negative case tells the operator to repoint this card at a destructive branch and click Accept, but a card is terminal once accepted | **confirmed** — and it is a worse defect than he states: it instructs the operator to hand-edit a ledger row, which is not an acceptance |
| 3 | the cited verdict pins `6f1c566` on `packet/nimbus/project-scoped-reads`, a different branch from the one this card names | **confirmed** — `payload->>'branch'` is `packet/nimbus/project-scoped-reads` @ `6f1c566`; the card's `branch` is `packet/nimbus/deletion-guard-accept` |

**So the card was not acceptable and he was right to withdraw it.**

## The part that is mine

**I measured defect 3 and did not treat it as a problem.** Last round I wrote, in a table: *verdict ref
`ev-hW75U60DweOY5-qM5pc5r` — **exists**, `ReviewVerdictIssued`, from `seat-flux-001`*. I checked that the event row
existed. **I did not check what it was about.** And I had the information in front of me — I had recorded, in the same
document, that Nimbus had isolated the guard onto a fresh branch rather than the one I verified on. **The two facts were
adjacent and I did not connect them.**

**Then I wrote: *Click Accept on `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy`. Its branch is synced, its verdict is valid, its merge
produces 0 deletions so the guard will not refuse it.** The first and third clauses are true and measured. The second is
true only in the narrow sense that the reference resolves.

**Had he followed it, he would have accepted a card whose verdict was about a different branch** — which is precisely
the failure this fleet has spent two days building gates against, and I would have been the one who told him to.

## Why this is the same failure I keep recording, in the worst position yet

**A reference that resolves, presented as a reference that applies.** I have recorded this family against a grep that
returned nothing, a green check against an empty subject, a cached remote ref, a `limit 4` sample reported as a census,
and a classification substituted for a query. **Each time the instrument answered about the set it could see and I read
it as an answer about the set that mattered.**

**This time the cost would have been the operator's, not mine.** Every previous instance cost me a wrong number in a
document. This one would have cost him an acceptance he could not defend, on the strength of my sentence.

**And the specific error is checkable in one command**: a verdict reference is about a branch, so
`payload->>'branch'` must equal the card's `branch`. I ran the existence check and not the equality check. **The check I
skipped is the cheaper one.**

## What I am doing

**Correcting the advice explicitly, in my own artifact, where he will see it** — the same pattern as the ghost-file
correction on 11-06 01:32. He withdrew the card on his own judgement and does not need my correction to be safe; **what
he needs is to know that my "click Accept" was wrong, so he does not weigh it as corroboration next time.**

**And changing the practice**: a verdict reference is not verified until its branch and commit match the card's. **A
reference that resolves is a resolved pointer, not a verified one.**

**And recording what his withdrawal establishes**: the guard itself is unaffected — verified at `6f1c566` by Flux and
by my own measurements on real repositories. **The defect was in the card's plan, not in the code the card was about.**
That distinction is why his withdrawal is the correct act and not a regression: he removed an unacceptable acceptance
path and left the verified work standing.

## What I did not do

I did not edit the card, the branch, or any script. I did not accept, cancel, or re-open any card, merge, push, restart
any application, or write to the live database. My measurements were reads: the card's full row including `decided_by`,
`decided_at` and `feedback`, the verdict event's `branch` and `commit` payload fields, the card's `branch`, the branch
tip, the diff stat against `track`, and my own previous document read back to establish exactly what I had advised. The
ledger check is exit 0, `seats` is 10, and `canonical_events` is **103**.
