# Delivery: D2 acceptance gate follow-on — the merge base is named and verified, and a pending card can be withdrawn

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-d2-acceptance-base-and-cancel-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, d2, acceptance-gate, governed-dispatch, defect-fix |
| governing dispatch | `disp-E8BP2vKlOwMHLomm0eX8J` (Aegis; D2 blocking defects 1 + 2) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/waker-hardening` (this worktree's packet branch) |
| scratch database | `seatloom_scratch_gate` (001–012 + seed + 013/014 from `track/infra-foundation` + scratch 016; NEVER live `seatloom`) |

---

## 0. Summary

Two blocking defects in the D2 acceptance gate are fixed on the Nimbus lane:

1. **Defect 1 — the merge base did not exist.** `request_acceptance` recorded the
   hardcoded literal `'track'`, which is not a branch in this repository (the
   integration branch is `track/infra-foundation`). The delivery-branch check
   existed, but nothing ever checked the base, so the failure only appeared when
   the operator clicked Accept as git's `could not check out base "track"`.
   The base is now an **explicit, validated input**: named by the caller or by
   `SEATLOOM_INTEGRATION_BRANCH`, refused when neither names it (never invented),
   and proven to exist as a local branch **at request time**, symmetric with the
   delivery-branch check the merge path performs. The Tauri merge path gained the
   same base check as a backstop.

2. **Defect 2 — a pending card could not be closed through the product.** The
   only exits were accept (merge) and reject (auto-enqueue rework), so the
   bootstrap-install card could only be settled by hand-editing the database.
   `cancel_acceptance` (MCP) now lets the **requesting seat** withdraw its own
   pending request: `pending -> cancelled` (terminal) + `AcceptanceCancelled`
   in one transaction, with a mandatory reason. No merge, no rework — the card is
   closed as a recorded void instead of a rejection.

Both fixes are covered by tests that ran green against a scratch database with
migrations 001–016 applied (§4).

---

## 1. Defect 1 — the base is named and verified before a card exists

### What changed

| File | Change |
|---|---|
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `RequestAcceptanceArgs` gains `base: Option<String>`; new `resolve_base()` (argument > `SEATLOOM_INTEGRATION_BRANCH` > **refuse**); new guards `MissingBase` / `UnknownBase`; the INSERT writes the resolved base instead of `'track'`; the `AcceptanceRequested` payload carries `base` |
| `crates/seatloom-mcp-emit/src/protocol.rs` | `request_acceptance` schema documents `base` and the env fallback; `emit_event` enum 21 -> 22 (`AcceptanceCancelled`) |
| `crates/seatloom-core/src/git/ops.rs` | New shared helpers `resolve_repo_root()` (`SEATLOOM_REPO_ROOT`, else CWD) and `local_branch_exists()` — the one fact both the request-time guard and the merge path need, with ONE wording |
| `src-tauri/src/commands/acceptance_cmds.rs` | `merge_branch_into_base` step 3b: the base must exist too, before anything is checked out or merged |

### The resolution rule (no guessing)

1. the `base` argument, if non-blank;
2. else `SEATLOOM_INTEGRATION_BRANCH` (explicit per-launch configuration, the
   same pattern as `SEATLOOM_ACTOR_REF` / `SEATLOOM_PROJECT_ID`);
3. else **refuse** (`MissingBase`), naming both sources in the error.

`'track'` is never re-introduced as a default. On this repository the value is
`track/infra-foundation`; the seat or the operator states it, and the request
refuses rather than recording a branch that cannot be merged.

### Why the check lives in the MCP server

Worktrees share the `refs/heads` namespace, so a branch that exists on the main
checkout is visible from a seat's own worktree — the request-time guard can be
answered without a push and without trusting the caller's word. A root that is
not a work tree is an **error, not a false**: a mistyped `SEATLOOM_REPO_ROOT`
must not read as "the branch is missing", because the two need different fixes.

---

## 2. Defect 2 — `cancel_acceptance`, the sanctioned void

### What changed

| File | Change |
|---|---|
| `crates/seatloom-mcp-emit/src/dispatch.rs` | New tool `cancel_acceptance(acceptance_id, reason)`; guarded on the acceptance being `pending` **and** gating a dispatch owned by the resolved `SEATLOOM_ACTOR_REF`; lands `status='cancelled'` + `feedback=reason` + `decided_by/decided_at` and appends `AcceptanceCancelled` in ONE transaction |
| `crates/seatloom-mcp-emit/src/protocol.rs` | Tool schema (both arguments required, no identity parameter) |
| `crates/seatloom-mcp-emit/src/main.rs` | Tool wired into `tools/call`; startup banner and the unknown-tool message list seven tools |
| `crates/seatloom-core/src/db/event_domain.rs` | `EventType::AcceptanceCancelled` (ALL 21 -> 22, `FromStr`, `Display`, pinned-set test) |
| `crates/seatloom-core/src/db/models.rs`, `src-tauri/src/dto.rs` | Status-domain doc comments: `'pending' \| 'accepted' \| 'rejected' \| 'cancelled'` |
| `.scratch/016_acceptance_cancel.sql` | **Scratch copy (NOT FOR COMMIT)** — the DDL Onyx lands: `acceptances_status_vocab` 3 -> 4 (+`cancelled`), `canonical_events_event_type_vocab` 21 -> 22 (+`AcceptanceCancelled`), plus a `feedback` COMMENT refresh |

### Why a distinct transition and not "reject without rework"

The dispatch offered both shapes. Reusing `reject` needs no DDL, but it records
a **void as a rejection**. In an append-only ledger one word for two meanings can
never be split once rows carry both — the same reasoning that kept
`ArtifactProduced` out of the event vocabulary. "This card was satisfied by other
means, do not merge it" is not "this delivery is not good enough, redo it".

The shape taken mirrors what `013` already signed for dispatches
(`SeatDispatchCancelled`, issuer-side withdrawal, terminal): the acceptance
analogue is `AcceptanceCancelled` — the requesting **seat** pulls back its own
hand-over, terminal and immutable, exactly one event for exactly one transition
(contract rule 2).

### Authority

Only the seat whose dispatch the acceptance gates may withdraw it, resolved from
`SEATLOOM_ACTOR_REF` server-side (never a parameter). Terminal rows are
immutable, so a decided card can never be re-cancelled. The reason is mandatory:
a void with no recorded why would be an unexplained edit to the ledger.

---

## 3. Validation

Commands run (Rust toolchain 1.95.0; scratch DB `seatloom_scratch_gate` with
001–012 + seed 001–004 + 013/014 from `track/infra-foundation` + scratch 016
applied in `apply-db.sh`'s canonical order; `seatloom-cli reconcile` run first,
as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| compile | `cargo check --workspace --all-targets` | PASS |
| unit + DB tests (core) | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (109 + 6 D2 + 27 baseline + 3 seed-consistency) |
| unit + DB tests (mcp-emit) | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (51, incl. 5 new/updated) |

New / updated tests:

- `dispatch::tests::request_acceptance_names_and_verifies_the_base_before_the_db`
  — an unnamed base refuses, a non-existent base refuses, an existing base gets
  past the guard into the DB, and the env var supplies the base when the argument
  omits it. No database needed (the dead pool proves the guard ordering).
- `dispatch::tests::resolve_base_prefers_the_argument_then_the_env_and_never_invents`
  — the resolution order, with blank values treated as absent.
- `dispatch::tests::cancel_acceptance_requires_a_reason_before_the_db`,
  `cancel_acceptance_args_require_the_reason`, `request_acceptance_args_base_is_optional_on_the_wire`
  — argument-level guards and wire backward compatibility.
- `dispatch::db_itests::cancel_acceptance_withdraws_own_pending_card_and_is_terminal`
  — the withdrawal lands `cancelled` + the reason + the event
  (`AcceptanceRequested`, `AcceptanceCancelled`), a second withdrawal is refused,
  **no rework dispatch is created**, and another seat cannot withdraw it.
- `dispatch::db_itests::request_acceptance_guards_then_happy_path_and_rework_pullable`
  — extended to assert the recorded base and the `base` field in the event payload.
- `git::ops::tests::local_branch_exists_separates_present_absent_and_non_git`
  — present / absent / not-a-work-tree stay three different answers.
- `event_domain` pinned-set tests — 22 event types incl. `AcceptanceCancelled`,
  and the void stays a word of its own (`AcceptanceVoided` is not admitted).
- `acceptance_cmds` merge tests — unchanged and still green with the new step 3b.

Scratch-DB schema verification (the queries 016's header asks for):

```
SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='acceptances_status_vocab';
-- CHECK ((status = ANY (ARRAY['pending','accepted','rejected','cancelled'])))
SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='canonical_events_event_type_vocab';
-- 22 values, incl. 'AcceptanceCancelled'
```
016 applied twice: the second apply performs no catalog write (idempotent).

---

## 4. Escalations (not worked around)

1. **Value-domain sign-off — REQUESTED from Lyra.** `AcceptanceCancelled` and the
   `cancelled` status widen a vocabulary Lyra signed in
   `LYRA-2026-10-09-d2-acceptance-loop-design-v1` §3 ("contract extension,
   mine"). The Rust mirror lands with this dispatch (as the 013/014 mirrors did);
   the **migration is a scratch copy** and the vocabulary widening needs Lyra's
   sign-off before it is contract truth. If Lyra prefers the alternative shape
   (a reject that skips the rework enqueue), the trade-off is in §2 and the
   change is small — but it records a void as a rejection.
2. **Migration 016 is Onyx's to land.** Slot 016, not 015: 015 is already
   assigned to Onyx's D5 knowledge-layer schema
   (`LYRA-2026-10-10-d5-knowledge-layer-milestone`, Lane A). 016 is POST-SEED,
   requires 014, additive and idempotent; registering it in
   `scripts/apply-db.sh`'s `POST_SEED_SCHEMA` belongs with that slot decision.
   **Until 016 lands, `cancel_acceptance` refuses at the database CHECK** with
   the vocabulary violation — the same honest state the D2 dogfood was in before
   013/014 landed (escalated, not worked around).
3. **The UI has no cancel affordance.** `cmd_cancel_acceptance` was deliberately
   NOT added: a Tauri command with no button is dead code, and the pending-acceptance
   surface is Mira's lane. If the operator (not just the seat) should be able to
   void a card, that is a Mira + Lyra decision on the same vocabulary.
4. **The fix is not live until the MCP binary is rebuilt and redeployed.** The
   seats talk to `seatloom-mcp-emit` from the main checkout
   (`target/debug/seatloom-mcp-emit`); this branch's code changes nothing until
   that binary is rebuilt from a branch that carries them and the deployment is
   redone. Rebuilding/redeploying is an ops action outside the Nimbus lane —
   flagged for Aegis, not done here.
5. **A seat's pi session caches its tool list.** Mira's D2 dogfood hit exactly
   this: `cancel_acceptance` will not appear in a session started before the
   redeploy. The sanctioned fallback (used by Flux's acceptance-loop proof and
   by Mira's dogfood) is to drive the same binary over stdio with the seat's own
   `SEATLOOM_ACTOR_REF` — same binary, same database, same guards, same
   transaction.
6. **Pre-existing docs-gate debt on this branch.** `scripts/verify-docs.sh` is
   already red here for D2-era docs that are not in
   `.gitlab/docs-grandfather.txt` (missing `version`/`tags` header fields, CJK
   text, non-conforming names). Not caused by this delivery; this delivery's doc
   conforms. Reported for whoever owns the grandfather snapshot.

---

## 5. How Aegis settles the bootstrap card (once 016 is live)

The dangerous card is `acc-disp-me6Nm8e9D6ZRe4fKyVokY` (dispatch
`disp-me6Nm8e9D6ZRe4fKyVokY`, branch `packet/mira/dispatch-ui`, base recorded as
the non-existent `'track'`, Flux verdict `ev-UDrRZZqr8LUwfcgEeFdN5`). It is still
`pending` and must not be accepted — accepting it would merge that branch and
revert the track documents and `DispatchInboxView` that the path-qualified
bootstrap install (merge `6aa705a`) already landed.

With this fix, the sanctioned path is: Aegis dispatches **Mira**
(`seat-mira-001`, the seat that requested the card) to call

```
cancel_acceptance(acceptance_id="acc-disp-me6Nm8e9D6ZRe4fKyVokY",
                  reason="already landed on track via the bootstrap install (merge 6aa705a); merging packet/mira/dispatch-ui would revert it")
```

which lands `cancelled` + `AcceptanceCancelled` in one transaction — no merge, no
rework, no hand-edited row. Mr. Zhang still must not click Accept on that card.

Two preconditions before that call can succeed, both escalated above: migration
016 must be live (Onyx) and the redeployed binary must carry this fix (ops). If
Mira's session still advertises the old tool list, the stdio path in escalation 5
is the sanctioned way to make the same call.

---

## 6. Not in this delivery

Lyra's D5 **Lane B** (MCP recall read tools,
`NIMBUS-2026-10-10-d5-mcp-recall-tools-v1`) is a separate packet on a separate
dispatch and was NOT started here: this dispatch made defect 1 the priority and
Lane B is blocked on Onyx's 015 schema (Lane A) in any case.
