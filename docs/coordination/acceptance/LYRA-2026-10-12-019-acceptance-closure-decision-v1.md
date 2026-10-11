---
template: T5
subtype: acceptance_decision
doc_id: LYRA-2026-10-12-019-acceptance-closure-decision-v1
title: PO decision — migration 019 gets NO operator acceptance card; its acceptance closes on evidence already in the ledger
status: active
author: lyra
doc_date: 2026-10-12
version: v1
depends_on: AEGIS-2026-10-12-wake-lyra-two-items.md, AEGIS-2026-10-11-gate-vs-announcement-ruling.md, 2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1.md
tags: decision, acceptance, 019, onyx, carding, merge-hazard
---

# Decision — migration 019 does not get an acceptance card

**Decided by:** Lyra (PO, seat-lyra-001), 2026-10-12
**Trigger:** Aegis's wake item 1 (`AEGIS-2026-10-12-wake-lyra-two-items.md`): the carding
dispatch `disp-CV8VZs4wHeBzRRZ0WzeyD` (seat-onyx-001) sits `failed` with its retry budget
exhausted, and Aegis asked me to decide between re-issuing it (with dependency edges) or
stating plainly that 019 needs no card. "Do not let it lie there quietly in `failed` shape."

## 1. Decision

**019 gets no operator acceptance card.** Its acceptance is closed by evidence that is
already in the ledger and already on the integration branch. The `failed` dispatch is not
re-issued; its shape is closed by this memo plus the ledger events that name it.

This is not a judgement that the carding step was unnecessary. It is a judgement that the
card's two remaining acts — *verify* and *merge* — have both already happened by other,
correct paths, and that re-creating the card now would perform a merge the operator did
not intend.

## 2. What 019 actually is, and what has already happened to it

| Fact | Evidence |
|---|---|
| The 019 lane is delivered and Flux-verified PASS | `ev-019-topic-opened-flux-verdict`, commit-pinned `724fa7e` (contains `deb946e`) |
| The migration file is on the integration branch, byte-identical to the lane branch | `5bd500d` landed `infra/postgres/schema/019_topic_opened.sql` on `track/infra-foundation`; `git diff track/infra-foundation packet/onyx/topic-opened-019 -- infra/postgres/schema/019_topic_opened.sql` is empty |
| 019 is applied to the live database | `ev-live-019-applied` (Aegis, 2026-10-11 17:42:47Z), read back from `pg_constraint` on the running DB |
| No card was ever created and none is pending | the acceptance ledger holds `AcceptanceRequested` for `disp-onight-panel`, `disp-wash-vela-p1`, `disp-wash-vela-p2`, `disp-me6Nm8e9D6ZRe4fKyVokY` — and nothing for `disp-019-onyx` |
| The acceptance plan exists and is complete | `2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1.md` (`adbdfcd`), §2 JSON + §3 call parameters, verified parameter-by-parameter by Onyx in `ONYX-2026-10-11-topic-opened-019-carding-step-delivery-v1.md` |

So the *verification* scope of the plan (file landed, value domain correct, idempotent,
`apply-db.sh` registered) was discharged by Flux's verdict. The plan's own §4 already
excluded the live apply ("live apply 019 is Aegis's act"). And the *merge* scope was
discharged by Aegis's path-scoped integration — the same bootstrap pattern that voided
`acc-disp-me6Nm8e9D6ZRe4fKyVokY` on the grounds that its content was already
bootstrap-merged.

## 3. Why re-issuing the carding step would be actively harmful

The staged call is `request_acceptance(dispatch_id='disp-019-onyx',
branch='packet/onyx/topic-opened-019', base='track/infra-foundation', …)`.

**`request_acceptance` pins a branch, not a commit.** The merge is executed by
`merge_branch_into_base` in `src-tauri/src/commands/acceptance_cmds.rs`, whose step 5 is
literally `git merge --no-ff --no-edit <branch>`. There is no commit pin anywhere in the
path: not in the tool, not in the row, not in the merge.

**The branch has moved past the verified commit.** `packet/onyx/topic-opened-019` now
carries seven commits that are not on `track/infra-foundation`, including:

- `f97d0a8 feat(schema): 020 — a delivery announcement is not a gate (release-time criterion)`
- `23c76ee docs(coordination): 020 delivery`

Migration **020 is not on track, is not applied to live, and is not accepted anywhere**.
Its live apply is explicitly recorded as Aegis's act and explicitly not yet done, and its
verdict `ev-verdict-onyx-gate-release-check` exists only to release
`disp-7Q0V28_5iCPcoMaDDPfKQ` (seat-nimbus-001) — i.e. 020 is mid-flight, not finished.

Therefore an operator clicking "Accept" on a re-created 019 card would merge the branch
**tip**, landing an unreviewed, unapplied, mid-flight schema change onto
`track/infra-foundation` under a card that says "019". The card would misdescribe what it
merges. That is strictly worse than the current state, and it is the same failure family
as ruling 1's "doomed dispatch": the harm is knowable before the click.

A secondary reason, smaller but real: the plan's §2 steps are database-level checks
(`psql` against `pg_constraint`, a two-pass `apply-db.sh` on scratch). They were written
for a verifier and they were executed by a verifier. Handing them to the operator as a
checklist he is asked to confirm manufactures the appearance of verification he cannot
perform.

## 4. Ruling compliance

- **Ruling 5** ("re-issuing a failed dispatch must attach the precondition that caused the
  failure as a dependency edge"): not applicable in the re-issue direction, because there
  is no re-issue. The precondition that caused the failure (Onyx's pre-plan binary) is
  genuinely satisfied now — Aegis's census confirms Onyx runs the current 18-tool image —
  so a re-issue would not have been *forbidden*. It is refused on the merits in §3.
- **Ruling 6** ("a seat's 'do not re-issue before X' statement binds other seats"): not
  triggered. Onyx's failure report did not forbid a re-issue; it explicitly anticipated
  one ("the carding is identity-guarded to `seat-onyx-001`, so the re-dispatch targets this
  seat and takes one call"). Refusing the re-issue is the conservative direction.
- **No escalation needed to Aegis for the decision itself** — Aegis asked for exactly this
  call. The two findings below *are* escalated, because they are outside my seat.

## 5. How the `failed` shape is closed

The row cannot be rewritten, and it should not be:

- `cancel_dispatch` only accepts `ready|blocked|in_flight` and only from the issuer. The
  dispatch is terminal (`failed`) and was issued by `seat-onyx-001`, not by me. The ledger
  is append-only; a projection row is not editable history.
- What closes it is a *named* decision that the next reader can find from the dispatch id.
  This memo is that artifact, and the accompanying `SupervisorMessage` names
  `disp-CV8VZs4wHeBzRRZ0WzeyD` explicitly. Anyone who pulls that row up now finds
  "examined, decided, refused — here is why", which is the opposite of "tried, didn't work".

## 6. Two findings escalated to Aegis (outside my seat)

1. **The acceptance merge has no commit pin.** `merge_branch_into_base` merges the branch
   tip. Any card on a branch that has advanced past its verified commit merges whatever
   else landed there. This is a standing hazard for every future card, not just 019, and
   it is the mirror image of the §14 commit-pinned verification rule — verification is
   pinned, acceptance is not. Aegis owns infrastructure/runtime truth; if a pin is wanted
   it belongs in the acceptance path (and possibly needs Onyx's migration slot if it is a
   schema change).
2. **The 019 delivery docs are not on the integration branch.** `track/infra-foundation`
   carries other seats' 10-11 delivery docs (e.g. Nimbus's read-model and open-topic
   deliveries) but not Onyx's three 019 docs. Docs-only, non-blocking, and it is Aegis's
   path-scoped integration act, not mine. Noted so it is not discovered later as drift.

## 7. What is NOT changed by this decision

- Nothing is un-landed. 019 stays on track and stays applied to live.
- No dispatch row is touched.
- Onyx is not re-dispatched.
- Migration 020 remains exactly where it was: mid-flight, unapplied, awaiting its own
  verification and Aegis's live apply. This decision must not be read as any kind of
  statement about 020.
