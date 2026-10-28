---
template: T5
subtype: delivery
doc_id: NIMBUS-2026-10-14-acceptance-reissue-id-param-delivery-v1
title: Delivery — request_acceptance takes an optional acceptance_id, so a cancelled card can be re-opened under the same id
status: active
author: nimbus
doc_date: 2026-10-14
version: v1
depends_on: AEGIS-2026-10-14-acceptance-reissue-id-param, docs/coordination/tasks/vela/disp-acceptance-cards-plain-language-vela-delivery.md
tags: delivery, acceptance-gate, reissue, acceptance-id, d2
dispatch: disp-acceptance-reissue-id-param
branch: packet/nimbus/gate-vs-announcement
---

# Delivery — the `acceptance_id` parameter on `request_acceptance`

**Dispatch:** `disp-acceptance-reissue-id-param`.

**The defect, verbatim from the washing seat's dispatch:**

> duplicate key value violates unique constraint acceptances_pkey,
> Key (id)=(acc-disp-wash-vela-p1/p2) already exists (sqlstate 23505)

**Root cause:** `request_acceptance` had no id parameter — the id was derived
deterministically as `acc-<dispatch_id>` — and `cancel_acceptance` moves a row to
the terminal `cancelled` state **without deleting it** (ledger discipline). So a
cancelled card keeps occupying its primary key. The "no pending acceptance for
this dispatch" guard passes, and the deterministic primary key refuses the
insert. Cancel-then-re-request was unreachable.

The washing seat did not work around it — no raw SQL, no side file, no live-DB
edit — and failed its own dispatch rather than mark `done` on a success condition
that did not exist. That is the behaviour this mechanism exists to reward.

---

## 1. What changed

| File | Change |
|---|---|
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `RequestAcceptanceArgs` gains `acceptance_id: Option<String>` (`#[serde(default)]`); a new Guard 4 resolves and validates the id; the write path branches between INSERT (fresh id) and a guarded UPDATE (re-open); four new typed rejections; the success message distinguishes a re-open; the `AcceptanceRequested` payload records `reopened` |
| `crates/seatloom-mcp-emit/src/protocol.rs` | the `request_acceptance` tool schema advertises `acceptance_id` with the accepted/refused rules, and the description explains the defect it closes |
| `crates/seatloom-mcp-emit/proof/acceptance_reissue_id_proof.sh` | new — six-point proof on a scratch DB |

No schema migration: every column the re-open needs (`feedback`,
`decided_by`, `decided_at`, `merge_commit`, `rework_dispatch_id`) already exists
on `acceptances` (014/017). No `ui/**` change — the operator-facing surface is
unchanged.

### The write path, precisely

A re-open is an **UPDATE of the existing row**, never a delete-plus-insert:

```sql
UPDATE acceptances
   SET status = 'pending', branch = $2, base = $3, how_to_verify = $4,
       flux_verdict_ref = $5, acceptance_plan = $6,
       feedback = NULL, decided_by = NULL, decided_at = NULL,
       merge_commit = NULL, rework_dispatch_id = NULL
 WHERE id = $1 AND status = 'cancelled'
```

The cancel reason is cleared because a pending card must not carry
`cancelled because …` — that would be a lie on a live row. The history is not
lost: the ledger holds `AcceptanceCancelled` followed by `AcceptanceRequested`
under the same id, and the event payload now carries `"reopened": true` so a
reader never has to infer it. The `WHERE … status = 'cancelled'` clause is the
race guard: a row that left the cancelled state between the guard read and the
write yields `n != 1` and a typed refusal rather than a silent resurrection.

## 2. The guards, and why each one is where it is

Read in **one** query (`status`, `dispatch_id`, `target_seat_id` joined), so the
decision cannot be made against a stale pair of reads:

| Id state | Outcome |
|---|---|
| fresh (no row) | INSERT — the historical path |
| `pending` | **refuse** `AcceptanceIdTaken`, naming the occupant dispatch **and** its owner. A pending card is somebody's live work |
| `accepted` / `rejected` | **refuse** `AcceptanceIdTerminal`. A decided card is a recorded fact — the merge happened, or the operator's feedback exists |
| `cancelled`, dispatch owned by another seat | **refuse** `AcceptanceIdNotYours`. The seat-identity guard is not bypassed by choosing an id |
| `cancelled`, owned by this seat, different dispatch | **refuse** `AcceptanceIdWrongDispatch`. An id encodes the dispatch it gates; re-pointing it would break the one-card-per-dispatch lineage |
| `cancelled`, owned by this seat, same dispatch | **re-open** |

Every refusal happens **before** any write, so a refused call changes nothing —
proved, not assumed (POINT 3 of the proof compares the row before and after).

## 3. Acceptance criteria, answered

| Criterion | Evidence |
|---|---|
| 1. Omitting the id leaves every existing caller's behaviour unchanged | `acceptance_id_omitted_keeps_the_deterministic_id_untouched` (DB test) + proof POINT 1: the derived id, the same message, `reopened` absent from the event, ledger `[AcceptanceRequested]` |
| 2. A cancelled card re-opens under the same id | `a_cancelled_card_re_opens_under_the_same_id` + proof POINT 2: one row (not two), contents refreshed, cancel reason cleared, ledger `AcceptanceRequested → AcceptanceCancelled → AcceptanceRequested` |
| 3. An id held by a pending card is refused and nothing changes | `a_pending_id_is_never_taken_over` + proof POINT 3: occupant row byte-identical before/after, and **zero** rows for the refused dispatch |
| 4. Another seat's id is refused — the identity guard is not bypassed | `another_seats_id_is_refused_and_a_terminal_id_is_not_reusable` + proof POINT 4: three sub-cases (pending → `Taken`, cancelled-by-Flux → `NotYours` naming `seat-flux-001`, decided → `Terminal`) |
| 5. A real path works: the washing seat re-opens p1/p2 | **Not taken by me** — see §4. The path is proven open and every live precondition is verified read-only |

## 4. Criterion 5 — proven open, not taken

Re-opening Vela's cards is Vela's act: the plan is Vela's, the `how_to_verify`
is Vela's, and the operator's Accept is the operator's. I did not re-open them,
did not sync Vela's branch, did not touch Vela's worktree, and did not author any
part of the acceptance plan.

What I did verify, **read-only, against the live database**:

| Fact | Measurement |
|---|---|
| `acc-disp-wash-vela-p1` | `cancelled`, gates `disp-wash-vela-p1`, owner `seat-vela-001`, dispatch `done` |
| `acc-disp-wash-vela-p2` | `cancelled`, gates `disp-wash-vela-p2`, owner `seat-vela-001`, dispatch `done` |
| either id held by a pending card | no (0 rows) |
| `flux_verdict_ref` for p1 | `ev-SWfX7pNZH8PObLcGxg3pY` — `ReviewVerdictIssued`, verdict `pass`, 2026-10-10T17:17:27Z |
| `flux_verdict_ref` for p2 | `ev-flux-verdict-wash-p2` — `ReviewVerdictIssued`, verdict `pass`, 2026-10-10T18:05:33Z |

So the new id guard will pass for Vela. **One other guard will not**, and it is
measured rather than guessed: `packet/vela/washing-pipeline` is at `49921ff` and
does **not** contain `track/infra-foundation` at `13b33e3` (merge-base
`9f408d89`), so `request_acceptance` will refuse with `BranchNotSynced` before it
ever reaches the id guard. Vela must sync first — `git merge
track/infra-foundation` **into** the delivery branch, never the reverse, which is
the recorded lesson and also what keeps the card valid against a moving base.

The exact call, the measured verdict refs, and the sync requirement are in
`ev-nimbus-acceptance-reissue-ready-for-vela`, addressed to the washing seat.

## 5. Reaching a seat at all

This is on my packet branch and is **not integrated**, so the running fleet
binary does not advertise `acceptance_id` (measured: the integration binary's
`request_acceptance` schema has no such property). Until Flux's verdict and
Aegis's integration land — and a seat reload picks the new binary up, which is
Aegis's act — the call is refused as an unknown argument. That is the correct
outcome, and it is the same shape as the intent tool surface's `-live` gate: the
code existing is not the capability being reachable.

## 6. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 146 passed / 0 failed (2 ignored, need Postgres) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit` | 76 passed / 0 failed (19 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit -- --ignored` (scratch DB, single-threaded) | **19 passed / 0 failed** — the 5 new DB tests plus the 14 pre-existing ones |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | 9 passed / 0 failed |
| `cargo check -p seatloom-tauri` | clean |
| `proof/acceptance_reissue_id_proof.sh` | **ALL POINTS PASS** on scratch DB `seatloom_accid` (001-014 + seed + 015-019 from `track/infra-foundation`, built in the canonical pre-seed/seed/post-seed order) |

The scratch database is built by the proof itself, in `scripts/apply-db.sh`'s
canonical grouping — `017_acceptance_cancel` is what puts `cancelled` into the
acceptances status vocabulary, and it lives on the integration branch, so the
proof reads 015-019 from there.

## 7. Known limits, stated

* **A re-open does not create a new ledger object.** It reuses the id and the
  row, on purpose: the point is that the operator sees the *same* card come back.
  If the fleet ever wants "a second card for the same delivery", that is a
  different feature with a different id scheme, and it should be decided rather
  than drifted into.
* **The re-open clears `rework_dispatch_id` and `merge_commit`.** Both are
  `NULL` on a cancelled row today, so this changes nothing observable; it is
  defensive, and it is stated here so nobody has to discover it.
* **The dispatch-kind readiness gap I reported earlier** (`finding_4`: a
  `dispatch` edge is satisfied by the referent row's mere existence) is untouched
  by this delivery and still awaits Aegis's ruling.
