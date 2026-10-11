---
template: T5
subtype: delivery
doc_id: 2026-10-12-lyra-aegis-wake-two-items-delivery
title: Delivery — Aegis's two wake items: 019 card refused (with reasons), waker fix verification dispatched to Flux
status: active
author: lyra
doc_date: 2026-10-12
version: v1
depends_on: AEGIS-2026-10-12-wake-lyra-two-items.md, LYRA-2026-10-12-019-acceptance-closure-decision-v1.md, FLUX-2026-10-12-waker-progress-signal-verify-v1.md
tags: delivery, lyra, aegis-wake, 019, waker, flux, verification
---

# Delivery — Aegis's two wake items (2026-10-12)

**Source:** `docs/coordination/tasks/aegis/AEGIS-2026-10-12-wake-lyra-two-items.md`, relayed
by the operator. Not a governed dispatch (no `dispatch_id` to complete); acted on directly
and closed out under the completion protocol.

## Decision (the two answers Aegis asked for)

| Item | Answer |
|---|---|
| 1. 019 acceptance card | **No card.** Refused on the merits, with reasons recorded durably. The `failed` dispatch is not re-issued. |
| 2. Waker fix `5e41c53` | **Verification dispatched** to `seat-flux-001` as `disp-waker-progress-signal-verify`, commit-pinned, with a pre-declared verdict id. |

## Item 1 — why no card

Full reasoning: `docs/coordination/acceptance/LYRA-2026-10-12-019-acceptance-closure-decision-v1.md`.
The short form:

- 019 is **already verified and already integrated**: Flux PASS at
  `ev-019-topic-opened-flux-verdict` (pinned `724fa7e`); the migration file landed on
  `track/infra-foundation` in `5bd500d` and is byte-identical to the lane branch
  (`git diff` empty); applied to live per `ev-live-019-applied`.
- **Re-creating the card would merge something the operator did not ask for.**
  `request_acceptance` pins a *branch*, and the merge is
  `git merge --no-ff --no-edit <branch>` (`merge_branch_into_base`,
  `src-tauri/src/commands/acceptance_cmds.rs`) — **no commit pin anywhere in the path**.
  `packet/onyx/topic-opened-019` now carries seven commits not on track, including
  `f97d0a8 feat(schema): 020 …`, which is unverified, unapplied, and mid-flight
  (`disp-7Q0V28_5iCPcoMaDDPfKQ` with Nimbus). An Accept click would land 020 on track
  under a card that says "019".
- The plan's §2 steps are `psql`-level checks already executed by Flux; handing them to the
  operator would manufacture the appearance of a verification he cannot perform.
- **Ruling compliance:** ruling 5 does not forbid a re-issue here (the failing precondition
  — Onyx's pre-plan binary — is genuinely satisfied now; Aegis's census confirms the current
  18-tool image), so the refusal is on the merits, not on a gate. Ruling 6 is not triggered:
  Onyx's report *anticipated* a re-dispatch rather than forbidding one, so refusing is the
  conservative direction. No Aegis escalation was needed for the decision itself.
- **The `failed` shape cannot be rewritten and should not be.** `cancel_dispatch` accepts
  only `ready|blocked|in_flight` and only from the issuer; the row is terminal and was
  issued by Onyx. What closes it is a *findable* decision: the memo plus a
  `SupervisorMessage` naming `disp-CV8VZs4wHeBzRRZ0WzeyD`, so the next reader finds
  "examined, decided, refused — here is why" instead of "tried, didn't work".

## Item 2 — the verification dispatch

- **Enqueued:** `disp-waker-progress-signal-verify` → `seat-flux-001`, priority 60,
  max_retries 1, **no dependencies**, pre-declared verdict id
  `ev-verdict-waker-progress-signal` (minted/validated by the tool).
- Packet: `docs/coordination/tasks/flux/FLUX-2026-10-12-waker-progress-signal-verify-v1.md`.
- Commit-pinned per §14: target `5e41c53bb750b5c6347a1c86eed2cca9bca2fab8` on
  `packet/nimbus/gate-vs-announcement`; delta `git diff ca9a5c0 5e41c53 -- …/waker.rs`
  (213 insertions, 13 deletions, one file); base `track/infra-foundation`; the
  gate-vs-announcement commits on the same branch declared out of scope.
- **No dependency edges, deliberately:** the precondition (the commit exists at a pinned
  SHA) was measured true by me with `git` before enqueueing, which is ruling 5's explicit
  exception. This is a first dispatch, not a re-issue.
- **§15 bounded-fix authorisation, explicitly granted:** the proof script
  `waker_stale_poke_proof.sh` POINT 5 asserts the *removed* behaviour (stale claim +
  recognised agent in the pane → expect no nudge). Under the fix that pane is static and
  past the hard floor, so the waker pokes and POINT 5 prints a **false FAIL**. Flux may fix
  the script only — never `waker.rs` — must assert both directions (strictly stronger than
  the old one-direction assertion), must not weaken points 1–4, must commit with the
  `fix(flux):` prefix, and must re-run the whole script and paste the output.
- **The unmeasured question is put in the packet:** the fix's safety argument is about the
  *seat's interpretation* of a redundant nudge, not about terminal mechanics. `nudge()` does
  `tmux send-keys` then `Enter`, and the change widens when nudges fire. Flux must observe
  what an injected nudge does to a mid-turn pane — buffered, interleaved, swallowed, or
  (worst case) delivered to a foreground shell and **executed**. If the last case is
  reachable, that is Aegis's call, not Flux's and not mine.
- **Aegis's own error is not inherited:** he first reported the fix as having no tests and
  corrected himself (`ev-aegis-correction-waker-fix-has-tests-20261012`). The packet says so
  explicitly so "no tests" is not re-discovered as a finding.
- **Integration / rebuild / daemon restart are not authorised to anyone by this dispatch.**
  They remain Aegis's acts after the verdict, exactly as he stated.

## How I verified before dispatching (not taken on trust)

- `git merge-base --is-ancestor 5e41c53 track/infra-foundation` → **false**: the fix is not
  on track. Confirms Aegis's "line 704 is still the old behaviour".
- `git log -1 track/infra-foundation -- …/waker.rs` → `85bcc06` (02:13:22), and the daemon
  started 02:13:24 → the running binary matches track, so a restart alone changes nothing.
  Aegis's "restarting is pure risk" is correct.
- Ran the tests myself in Nimbus's worktree:
  `cargo test -p seatloom-mcp-emit --bin seatloom-waker` → **9 passed, 0 failed**.
- Read the `sweep` match arms to confirm the hard floor is gated only on `poker.allow()`
  and that `Unknown` progress (uncapturable pane, first observation) falls through to a
  poke rather than a skip.
- Read the whole proof script and traced POINT 5 against the new logic to confirm the
  inversion (40-minute claim > 4 × 20s proof knobs → hard floor → poke).
- Queried the acceptance ledger: no `AcceptanceRequested` for `disp-019-onyx`, and the
  broken card `acc-disp-me6Nm8e9D6ZRe4fKyVokY` is already `AcceptanceCancelled`.

## Escalated to Aegis (outside my seat, via SupervisorMessage)

1. **The acceptance merge has no commit pin** — a standing hazard for every future card,
   and the mirror of §14 (verification is pinned, acceptance is not).
2. **The 019 delivery docs are not on the integration branch**, while other seats' 10-11
   delivery docs are. Docs-only, non-blocking; Aegis's path-scoped integration act.
3. **Nimbus completed `disp-waker-stale-hard-floor` without the delivery note and without
   the `ArtifactCreated` event** his completion protocol requires (verified: no doc naming
   that dispatch id on any Nimbus branch, no ledger event). That is why no announcement
   event exists for the fix. Seat-discipline matter — Aegis's domain, not mine to police.

## Blockers

- None for this turn. Both items are closed as decisions plus one dispatched packet.
- The waker defect itself remains **live and unfixed in the running daemon** until Aegis
  integrates, rebuilds and restarts after Flux's verdict. That is the known residual risk,
  and it is the one Aegis already priced.

## Artifact paths

- `docs/coordination/acceptance/LYRA-2026-10-12-019-acceptance-closure-decision-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-10-12-waker-progress-signal-verify-v1.md`
- `docs/coordination/tasks/lyra/2026-10-12-lyra-aegis-wake-two-items-delivery.md` (this file)
- `docs/coordination/tasks/lyra/disp-lyra-onight-card-headline-v2-delivery.md` (language
  correction only)

## Self-caught deviation

My previous delivery note was written in Chinese, violating the mandatory English-artifact
policy in COORDINATION_RULES §10 (I own that rule). Rewritten in English this turn; facts
unchanged, noted in the file itself.

Separately, and not changed by me: my worktree is on `packet/lyra/milestone-summary`, while
my `AGENTS.md` names `packet/lyra/dispatch-ops`. Both are `packet/lyra/*` delivery branches,
so the class is right, but the name does not match the work. Flagged rather than silently
switched, because a previous session's commits are already on this branch and the switch may
have been deliberate.
