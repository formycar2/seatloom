# Delivery: D2/Nimbus — request_acceptance MCP + 013 Rust side + Tauri accept/reject + merge mechanism

[Nimbus -> Lyra / Flux] Delivery for governed dispatch `disp-0z1gtwcz1fdWK1o96dczP`
(D2 milestone, Nimbus lane: backend / merge mechanism). Implements the Lyra-signed
contract `LYRA-2026-10-09-d2-acceptance-loop-design-v1` (013 sign-off A, 014
sign-offs B/C) and Onyx's design `ONYX-2026-10-09-dispatch-states-design` §4.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | disp-0z1gtwcz1fdWK1o96dczP-delivery |
| author | nimbus |
| date | 2026-10-09 |
| version | v1 |
| governing dispatch | `disp-0z1gtwcz1fdWK1o96dczP` (D2, Nimbus lane) |
| shared contract | `docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md` (013/014 sign-offs) |
| schema designs | Onyx `ONYX-2026-10-09-dispatch-states-design` (013, exact SQL §4); Lyra §2 (014 DDL) |
| A1-widening precedent | `NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-delivery-v1` §7-A (authorized, same lane pattern) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/waker-hardening` (this worktree's packet branch) |
| scratch database | `seatloom_d2_nimbus` (001–012 + seed + scratch 013/014; NEVER live `seatloom`) |
| status | **COMPLETE** — gates green, scratch-DB smoke green; awaiting Flux verification |

---

## 0. Summary

The D2 acceptance loop's backend is delivered end to end:

- **(A)** MCP `request_acceptance` — the seat hands a COMPLETED delivery to the
  gate; guarded, atomic (row + `AcceptanceRequested` in one transaction).
- **(B)** the `013` Rust side — `DispatchStatus` +`Failed`/`Cancelled`
  (exhaustiveness kept), the `live_status` terminal-arm correctness fix,
  `complete_dispatch`'s `outcome` argument with the retry budget, and the
  issuer-side `cancel_dispatch` tool.
- **(C)** the three Tauri commands — `cmd_list_acceptances`,
  `cmd_accept_delivery` (merge into track, conflict = refuse, never force),
  `cmd_reject_delivery` (auto-enqueued rework through the same write path).
- **(D)** the DTO contract Mira's UI consumes — `AcceptanceCard` /
  `AcceptanceResult` exactly as specified.

The seat never merges its own work; the merge happens only on an explicit
operator accept, `--no-ff` into the acceptance's base.

## 1. Cross-lane note — the A1 half (reported, per the 012 precedent)

The dispatch's file boundary lists `crates/seatloom-mcp-emit/**` and the two
`src-tauri` files. The acceptance tools cannot compile without the Lyra-signed
vocabulary (`AcceptanceRequested/Granted/Rejected`, `RefType::Acceptance`) and
the Tauri commands cannot read/write acceptances without a repository half.
Per the established lane pattern (`012`'s "7-A — vocabulary widening
(AUTHORIZED)" in the step-6 delivery, and the `event_domain.rs` comments citing
it), the A1 touches are part of Nimbus's dispatch lane when a signed contract
widens the vocabulary. The touches are minimal and mechanical:

- `crates/seatloom-core/src/db/event_domain.rs` — `EventType` +6 (→21),
  `RefType` +1 `Acceptance` (→11); `ALL`, `as_str`, hand-written `FromStr`, the
  `*_EXPECTED` strings, and the exhaustiveness test updated and pinned verbatim
  to 010+012+013+014's CHECKs (**not loosened**). No serde derive added.
- `crates/seatloom-core/src/db/models.rs` — `AcceptanceRow` +
  `AcceptanceWithDispatchRow` (the joined read model).
- `crates/seatloom-core/src/db/repositories.rs` — `list_acceptances_for_project`
  / `get_acceptance` (joined read model), `event_or_artifact_exists` (the Flux
  verdict re-check), `decide_acceptance` (the guarded `pending → terminal`
  transition + `AcceptanceGranted`/`AcceptanceRejected` in ONE transaction via
  `append_event_in_tx` — no second write path), plus a boxed
  `DbError::Write(WriteError)` variant (unboxed would make `DbError` and
  `WriteError` recursively sized) and `DbError::AcceptanceNotPending`.

No `infra/**` file is committed: Onyx's `013`/`014` had not reached any branch
when this packet was built, so the exact SQL was materialized **read-only** into
`.scratch/` (untracked, never committed — Onyx's lane) and applied to the scratch
DB only. 013 = Onyx's design §4 verbatim (with the shipped LATERAL form its own
note prescribes for the recompute); 014 = Lyra §2 DDL in the 012
catalog-guarded house style. Both re-apply with zero catalog writes (verified).

## 2. (A) `request_acceptance` — `crates/seatloom-mcp-emit/src/dispatch.rs`

`request_acceptance(dispatch_id, branch, how_to_verify, flux_verdict_ref) ->
acceptance_id`. Identity from `SEATLOOM_ACTOR_REF` (never a parameter). Guards,
all inside the transaction: the dispatch belongs to the calling seat AND is
`done`; `flux_verdict_ref` names an existing event or artifact; no pending
acceptance for the dispatch. Inserts the `acceptances` row (deterministic id
`acc-<dispatch_id>` — one acceptance per dispatch, ever; a rework gets a NEW
dispatch id so the lineage stays one-to-one) and appends `AcceptanceRequested`
(subject = acceptance, actor = seat, context = dispatch) in the same
transaction. The seat never merges anything itself.

## 3. (B) the `013` Rust side

- **`scheduler.rs`** — `DispatchStatus` + `Failed`/`Cancelled`; `as_str`/`parse`
  extended; the round-trip test kept (not loosened) and a new test pins that a
  terminal row is never selected for pull.
- **`dispatch.rs::live_status`** — `Failed`/`Cancelled` added to the
  return-as-stored arm (Onyx's flagged one-line correctness fix: without it a
  terminal row whose deps all exist would be recomputed back to `ready`).
- **`complete_dispatch`** — `outcome` (`"done"` default, fully backward
  compatible; `"failed"` + optional `reason`). The decision is the pure
  `decide_completion(outcome, retry_count, max_retries)`
  (unit-tested table): budget remaining → `in_flight → ready`, `retry_count+1`,
  `started_at = NULL`, `SeatDispatchRetried` (plain `ready`, re-pullable by any
  eligible seat — Lyra's no-stickiness ruling); otherwise `in_flight → failed`
  (terminal), `SeatDispatchFailed`. Each outcome maps to exactly one event type.
- **`cancel_dispatch`** (new tool) — issuer side, guarded on
  `issued_by = <resolved actor_ref>` and non-terminal; → `cancelled` +
  `SeatDispatchCancelled`. Cancelling an `in_flight` dispatch releases the
  one-in-flight slot.
- **`enqueue_dispatch`** — optional `max_retries` (the issuer's retry budget;
  `0` = pre-013 behaviour). A negative/oversized budget is REJECTED
  (`InvalidMaxRetries`), never silently clamped.

## 4. (C) the Tauri commands — `src-tauri/src/commands/acceptance_cmds.rs`

- `cmd_list_acceptances` — the pending-first read model (result states included:
  merge commit / rework link).
- `cmd_accept_delivery(acceptance_id)` — operator ref from
  `SEATLOOM_OPERATOR_REF` (missing = refuse with a config hint, never invented);
  re-checks the Flux verdict exists (未验绿不可合); merges
  `git merge --no-ff --no-edit <branch>` into the acceptance's base in the main
  checkout (`SEATLOOM_REPO_ROOT` via `AppState`); refuses on a non-git root, a
  dirty tracked tree, a missing branch, or a conflict — **a conflict aborts the
  merge and returns git's own text, never forces**; lands
  `accepted`/`merge_commit`/`decided_by` + `AcceptanceGranted` atomically;
  returns the merge sha.
- `cmd_reject_delivery(acceptance_id, feedback)` — feedback required; enqueues
  the rework FIRST through the same write path the UI's enqueue uses (the proven
  MCP `enqueue_dispatch` over stdio: atomic row + `SeatDispatchIssued` +
  `pg_notify`, so the waker wakes the seat; issuer = the operator, target = the
  original seat, title `打回重做:<原标题>`, payload `{rework_of, feedback,
  packet, branch}`, priority +10); then lands `rejected` +
  `AcceptanceRejected` with `rework_dispatch_id` in the same transaction. If the
  enqueue fails nothing is written and the acceptance stays pending.
- Tests: `parse_enqueued_id` shape-strictness; a scratch-git-repo integration
  test proving the clean merge returns the sha AND the conflicting merge is
  refused with the repo left exactly as found (HEAD unmoved, tree clean); the
  non-git-root refusal.

## 5. (D) the DTO contract

`AcceptanceCard { id, dispatch_id, dispatch_title, seat_id, branch, base,
how_to_verify, flux_verdict: { ref_id, verdict, occurred_at }, status,
created_at, merge_commit?, feedback?, rework_dispatch_id? }` and
`AcceptResult { accepted, merge_commit?, error? }` — camelCase, ISO timestamps,
house style. `verdict` is `None` for an artifact referent (artifacts carry no
verdict payload) — stated, not fabricated.

## 6. Gates (7-A)

- `cargo fmt --all --check` — **green**
- `cargo clippy --workspace --all-targets -- -D warnings` — **green**
- `cargo test --workspace` — **green** (104 core + 40 mcp-emit + 3 tauri; the
  exhaustiveness pins updated, none loosened)
- scratch-DB smoke (`seatloom_d2_nimbus`, ad-hoc harness in `/tmp` — the proof
  script itself is Flux's lane) — **green**, observed:
  - six tools advertised; `enqueue` with `max_retries=1` → pull → fail ⇒
    `ready retry=1` + `SeatDispatchRetried`; pull again → fail ⇒ terminal
    `failed` + `SeatDispatchFailed` (budget exhausted);
  - `cancel_dispatch` by the issuer ⇒ `cancelled` + `SeatDispatchCancelled`; a
    seat's attempt to cancel ⇒ refused (not the issuer);
  - the 013 trigger flips a blocked dispatch to **stored** `ready` when its
    event referent lands;
  - `request_acceptance` happy path ⇒ `acc-disp-done` pending, base `track`,
    `AcceptanceRequested` with refs subject=acceptance / actor=seat /
    context=dispatch; the three guards fire (pending-exists, not-completed,
    unknown-verdict); a negative `max_retries` is rejected;
  - `uq_acceptances_one_pending_per_dispatch` rejects a second pending row.

## 7. Deviations / notes for the record

1. Acceptance ids are deterministic (`acc-<dispatch_id>`) per the design's
   comment; a re-request after a rejection is impossible by construction (the
   rework flow creates a new dispatch id).
2. The reject path's rework enqueue shells out to the MCP binary (the
   established UI-enqueue pattern) rather than forking the write path into
   seatloom-core; ordering is enqueue-then-decide so the event can carry the
   rework lineage. The rare enqueue-succeeds/decide-fails window leaves a rework
   dispatch with a still-pending acceptance — recoverable by retrying the
   reject; recorded here rather than papered over.
3. The emit_event tool's schema enums were widened to the full 21/11 vocabulary
   (it previously advertised 12/9) so the tool surface matches the accepted
   contract.
4. Live-DB behaviour of the new tools requires Onyx's `013`/`014` on track
   (retry columns, acceptances table, widened CHECKs); until then the new SQL
   paths will error on a pre-013 database — the schema dependency the dispatch
   itself sequences.

## 8. Dogfood chain

Delivered on `packet/nimbus/waker-hardening`; a Flux verification dispatch
carrying this commit sha is enqueued (per the dispatch's dogfood chain) before
completion.
