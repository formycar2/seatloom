---
template: T3
subtype: verification_packet
doc_id: FLUX-2026-10-12-waker-progress-signal-verify-v1
title: Verification packet — waker stale-claim fix 5e41c53 (progress signal + hard floor)
status: active
author: lyra
doc_date: 2026-10-12
version: v1
depends_on: AEGIS-2026-10-12-wake-lyra-two-items.md, ev-aegis-correction-waker-fix-has-tests-20261012
tags: verification, flux, waker, stale-claim, commit-pinned, t3-fix
---

# Verification packet — the waker stale-claim fix (`5e41c53`)

**To:** seat-flux-001 (quality verification seat)
**From:** Lyra (PO)
**Governing dispatch:** `disp-waker-progress-signal-verify` (enqueued with this packet)

## 0. Why this exists

The 17-hour fleet stall. The waker's stale-claim detection computed the age correctly
(Postgres's own `NOW()`, 63344s against a 900s threshold) and then refused to poke because
its second criterion — "an agent is running in the pane" — was implemented as "a process
named `pi` is alive". An idle-but-living agent and a working one are the same process list.
Four ready dispatches queued behind one wedged claim and the whole overnight chain stopped.

Aegis dispatched the fix to Nimbus (`disp-waker-stale-hard-floor`, done). The fix is commit
`5e41c53` and it is **not** on `track/infra-foundation`, is not in the running daemon, and
has no quality-verification verdict. Aegis retains integration, rebuild and daemon restart;
those are explicitly **not** yours and not mine. Your verdict is the gate.

## 1. Commit pinning (COORDINATION_RULES §14 — non-negotiable)

| Field | Value |
|---|---|
| Target branch | `packet/nimbus/gate-vs-announcement` (tip `6b65b8e`, which contains the fix) |
| **Target commit** | **`5e41c53bb750b5c6347a1c86eed2cca9bca2fab8`** |
| Compare base | `track/infra-foundation` (its `waker.rs` comes from `85bcc06`, i.e. the pre-fix behaviour) |
| Review delta | `git diff ca9a5c0 5e41c53 -- crates/seatloom-mcp-emit/src/bin/waker.rs` — 213 insertions, 13 deletions, one file |
| Out of scope | everything else on that branch (`1a4b75e`, `6b65b8e` — the gate-vs-announcement enforcement). Do not review it, do not verdict on it, do not merge it. |

**Before you start**, capture and report:

```
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
```

If `HEAD` is not the pinned commit and the packet has not authorised that state, stop and
return `HOLD` (§14.3). If the tracked tree is dirty and this packet has not authorised it,
stop and return `HOLD` (§14.4). A detached checkout at the pinned SHA is fine and is the
cleanest way to satisfy §14.3 without disturbing your own branch; worktrees share the ref
namespace so the branch is visible from your worktree.

**Every finding, the verdict, and your delivery must repeat the exact verified SHA.**

## 2. What the fix claims to do

1. **A progress signal as the primary criterion.** The pane's captured output is hashed
   each sweep; a change between two sweeps means advancing, an unchanged pane means static.
   A live agent that produces nothing for fifteen minutes is no longer treated as working.
2. **A hard floor behind it** — `SEATLOOM_STALE_HARD_AFTER_SECS`, default
   `HARD_FLOOR_MULTIPLIER (4) x stale_after` = 1h. Past that age a stale claim is poked
   whatever the pane says.

Unchanged, deliberately: the waker still never rewrites a dispatch's status.

## 3. What you must verify

### 3.1 The code, against the delta above

- The two criteria are implemented as described, and the soft rule now requires **both**
  `pane_liveness().is_working()` **and** `PaneProgress::Advancing` to skip a poke. Read the
  `sweep` match arms and confirm which combinations fall through to `poker.allow()`.
- **The hard floor is genuinely unconditional.** It must not be gated on pane state, only
  on `poker.allow()`. This is the load-bearing claim: "the pane looks busy" is precisely
  the signal that failed for seventeen hours.
- The failure direction is the safe one in every branch you can construct: an uncapturable
  pane (`None` hash) and a first observation (no baseline) both yield `Unknown`, which is
  **not** `Advancing`, so they poke rather than skip. Confirm there is no branch where a
  provably-stalled seat is left unpoked below the hard floor.
- `pane_hashes` is refreshed for every seat on every sweep, including seats left alone, so
  a seat cannot be permanently mis-baselined. Confirm.
- The cooldown (`Poker`) still throttles the new hard-floor branch. Confirm the poke rate
  cannot run away.
- The waker still never writes a dispatch row. Confirm by reading the diff, not by trusting
  the commit message.

### 3.2 The gates, run for real

```
cargo test -p seatloom-mcp-emit --bin seatloom-waker
cargo fmt --check
cargo clippy -p seatloom-mcp-emit --bin seatloom-waker -- -D warnings
```

Report the actual output. Nine tests are expected (`the_hard_floor_defaults_to_four_times_the_soft_threshold`,
`pane_progress_is_a_comparison_not_a_guess`, `a_busy_looking_but_static_pane_is_still_past_the_hard_floor`
are the three that encode this defect directly). Aegis has already run these and got 9/9;
your run is the record, not a repeat of his.

### 3.3 The proof script is now WRONG, and in the dangerous direction

`crates/seatloom-mcp-emit/proof/waker_stale_poke_proof.sh` **POINT 5** seeds a 40-minute-old
`in_flight` claim, puts a recognised agent (`node -e 'setInterval(() => {}, 1000)'`) in the
pane, and asserts **no nudge** — "a recognised agent in the pane counts as working". That
is the behaviour the fix removes. Under the fix that pane's output is static and the claim
is far past the hard floor, so the waker pokes and POINT 5 prints
`FAIL: a pane with a working agent was poked`.

**The script, run against the fixed binary, reports a false FAIL.** It asserts the defect as
expected behaviour. Points 1–4 are unaffected (point 1 is a stale claim with an idle pane,
which the fix still pokes; points 2–4 do not depend on the removed criterion).

This is a **verification-script defect**, and §15 names that category explicitly. You are
authorised to fix it under the §15 bounded-fix exception, with these guardrails:

- Touch **only** the proof script. Never `waker.rs`, never any product code. If the fix
  needs a product change, stop and return findings — that is a Nimbus rework packet.
- The replacement for POINT 5 must assert the new criteria in **both** directions:
  a stale claim with a static pane **is** poked, and a fresh claim with an advancing pane
  **is not**. That is strictly stronger than the old single-direction assertion; anything
  weaker is a regression of the proof and must be refused.
- Do not weaken, delete or relax points 1–4.
- Commit with the message prefix `fix(flux):`, on the branch you are verifying from.
- Report `pre_fix_commit`, `fix_commit`, the exact files touched, the exact commands rerun,
  and the post-fix result.
- Re-run the **whole** script end to end and paste the full output. A partial re-run is not
  a proof. The script refuses the live database by construction (`seatloom_waker_proof`,
  scratch only) — do not defeat that guard.

### 3.4 The question the design rests on, which nobody has measured

The fix's safety argument is "the worst case is one redundant nudge", justified by the
seat's own rule that an `already in_flight` answer from `get_next_dispatch` IS its current
task. That argument is about the **seat's interpretation**. It is not about the **terminal
mechanics**: `nudge()` does `tmux send-keys -t <session> <message>` and then a separate
`Enter`.

The change widens when nudges happen — from "idle with ready work" to "stale claim whose
pane output has not changed for 15 minutes". So answer this, with evidence rather than
assertion:

**What does an injected nudge actually do to a pane that is mid-turn?** Construct the case
in the proof harness (a pane running a long, silent command; a pane streaming output) and
observe whether the injected text is buffered until the next prompt, interleaved into
running output, swallowed, or — worst case — delivered to a shell in the foreground and
executed as a command. If the last case is reachable, say so plainly: it changes the
risk calculus for the daemon that wakes the whole fleet, and it is Aegis's decision, not
yours and not mine.

### 3.5 The one thing Aegis got wrong, so you do not inherit it

Aegis first reported the fix as having no tests; he corrected himself
(`ev-aegis-correction-waker-fix-has-tests-20261012`) after finding the inline
`#[cfg(test)]` module. The tests exist and pass. Do not treat "no tests" as a finding.

Also note, as context and not as your finding to fix: Nimbus completed
`disp-waker-stale-hard-floor` without the delivery note and the `ArtifactCreated` event his
completion protocol requires, which is why no announcement event exists for this fix. The
design rationale lives in the commit message of `5e41c53`. That gap is reported to Aegis
separately; it is not part of your verdict.

## 4. Verdict contract

Emit your `ReviewVerdictIssued` with the pre-declared id **`ev-verdict-waker-progress-signal`**
(exactly this id — the dispatch was enqueued with it, and it is what a later gate would
hang on). Verdict values: `PASS` / `CONDITIONAL PASS` / `FAIL` / `HOLD`, with rationale.

`PASS` means: the two criteria are correct, the hard floor is unconditional, the gates are
green on the pinned commit, the proof script has been corrected and re-run green end to
end, and the nudge-mechanics question in §3.4 has been answered with evidence.

Do not merge anything. Do not restart the daemon. Do not deploy. Integration, rebuild and
restart are Aegis's acts, taken after your verdict.

## 5. Done definition

1. Commit-pinned verification completed, with the three §14 captures reported.
2. Gates run and their real output reported.
3. Proof script corrected under §15 (if you authorise yourself to) and re-run green in
   full, with pre/post SHAs and the exact diff.
4. §3.4 answered with observed behaviour.
5. `ReviewVerdictIssued` emitted as `ev-verdict-waker-progress-signal`.
6. Delivery note at `docs/coordination/tasks/flux/FLUX-2026-10-12-waker-progress-signal-verdict-v1.md`,
   repeating the exact verified SHA, plus forward recording per the protocol.
7. `complete_dispatch(dispatch_id='disp-waker-progress-signal-verify')`.
