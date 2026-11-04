# Delivery — dispatch `disp-KMP7yTq6JRIzlq5vNacGY` (Lyra → Mira): the clean verdict exists and no card cites it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | LYRA-2026-11-04-clean-verdict-no-card-cites-it-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | dispatch, acceptance, measured |

**Filename note.** AGENTS.md names this `<dispatch-id>-delivery.md`; `scripts/verify-docs.sh:74` requires
`ROLE-YYYY-MM-DD-<topic>-vN.md` under `docs/coordination/tasks/`. The two conflict; I follow the gate and record
it here. The dispatch id is in the title and tags.

## What I dispatched

`disp-KMP7yTq6JRIzlq5vNacGY` → `seat-mira-001`, priority 9. Landed `ready`.

**Title:** The clean verdict exists and no card cites it: cancel and re-open
`acc-disp-Ra5YbN4t78bzdbYhYkQiV` on `ev-flux-verdict-expect-anything-comment-corrected-03`.

## Why this dispatch exists

**Every seat did its job and the chain still does not close.** The missing step was nobody's, because a verdict
and a merge both look like the end of a chain.

**Measured at dispatch time:**

```
three ReviewVerdictIssued events:
  ev-flux-verdict-four-write-paths-01    pass  disp-XpAdx0a9YLL83JVoPesTj  (06:45)
  ev-flux-verdict-two-frontend-cases-02  pass  disp-Ra5YbN4t78bzdbYhYkQiV  (06:46)  <- carries the finding
  ev-flux-verdict-expect-anything-comment-corrected-03
                                        pass  disp-Ra5YbN4t78bzdbYhYkQiV  (09:08)  <- CLEAN

the new one: findings = [] (empty array), no finding_1/2/3, carries a no_new_findings key,
commit 2ee173e, subject "the false expect.anything() comment is corrected to fact, and the
new length assertion is honestly labelled documentary"

but the card still cites the old one:
  acc-disp-Ra5YbN4t78bzdbYhYkQiV | pending | flux_verdict_ref = ev-flux-verdict-two-frontend-cases-02
```

**And the check reads exactly that column** — `scripts/check-verdict-defects.sh:85`,
`join canonical_events e on e.id = a.flux_verdict_ref`. **So a clean verdict that no card cites changes
nothing.**

**The chain is: dispatch done → verdict → card citing that verdict → operator accepts.** Flux's new verdict is
sitting at step two with nothing pointing at it.

**Why it went unnoticed:** Aegis asked Flux to re-verify, and Flux did. Aegis asked Mira to merge track, and she
did. **Neither request included the card step.** Emitting a verdict and merging a branch both feel like
completion.

## The sequence I asked for, and why it is ordered

**The guard forces the order** — `request_acceptance` refuses a dispatch that has a pending acceptance:

1. **`cancel_acceptance` on `acc-disp-Ra5YbN4t78bzdbYhYkQiV`.** It is terminal and never deletes the row, which is
   exactly why `request_acceptance` accepts an optional `acceptance_id` — to re-open under the **same** id, so the
   primary key the cancelled row keeps occupying does not force a new one.
2. **`request_acceptance` citing `ev-flux-verdict-expect-anything-comment-corrected-03`**, with
   `acceptance_id = acc-disp-Ra5YbN4t78bzdbYhYkQiV`. **So the operator sees the card id he was already told about,
   not a new one he has no context for.**
3. **Reuse the acceptance plan she already wrote** — it is good and it is honest, and its own preconditions say
   the card brings test coverage the operator cannot see. Update only the commit it names: `2ee173e` is her HEAD,
   not `f26466f`.
4. **Run the gates and report the numbers.**

## What I pre-checked so she would not have to

| check | measured |
|---|---|
| her branch contains `track/infra-foundation` | yes — `merge-base` exit 0 |
| `2ee173e` on her branch and is her HEAD | yes |
| her worktree clean | yes |
| no ready/in_flight dispatch for her | yes |
| the other card unaffected | it cites `ev-flux-verdict-four-write-paths-01`, which carries no finding, and stays pending throughout |

## The risk I named for her, from her own report

**Her 17:00 report records that Aegis's single documentation commit at 16:00 made both cards unclickable by
putting her branch one behind.** That is `BranchNotSynced` doing its job and it is not her fault — **but it means
the window between her cancel and her re-open is a window where the operator has one card instead of two.**

**So I told her: do it in one sitting, and if anything fails halfway, STOP and report rather than retrying. A
half-cancelled card is worse than an unsynced one, and I would rather read "I stopped here" than a recovery
story.**

**And I told her explicitly not to touch the old verdict event** — it is immutable and must stay as the record of
what was true at 06:46. **The point is not to make the old finding disappear; it is to add a record that it was
fixed.**

## What I did not do

I did not cancel or open any card, emit any event, modify any verdict, merge, push, restart any application, or
write to the live database beyond this dispatch. I did not touch her worktree. My measurements were reads: `psql`
for the three verdicts and their payloads and for both cards' `flux_verdict_ref`; `git merge-base --is-ancestor`
and `git rev-parse` for her branch state; `git status --porcelain` on her worktree; reading
`check-verdict-defects.sh:57,85`; and reading Aegis's `caba38f` and Mira's `d2208f7` in full. The ledger check is
exit 0, `seats` is 10, `canonical_events` is 21.
