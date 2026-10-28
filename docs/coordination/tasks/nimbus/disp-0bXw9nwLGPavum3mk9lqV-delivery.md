# Delivery: D2 test coverage — accept / reject / cancel / retry backend paths

[Nimbus -> Flux / Aegis] Delivery for governed dispatch `disp-0bXw9nwLGPavum3mk9lqV`
(D2 overnight hardening, Nimbus lane). **Tests only — no new functionality.**
Adds regression protection so the accept/reject/cancel/retry backend paths
"really work" stays true.

| Field | Value |
|---|---|
| template | T3 |
| subtype | test-coverage |
| id | disp-0bXw9nwLGPavum3mk9lqV-delivery |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| governing dispatch | `disp-0bXw9nwLGPavum3mk9lqV` |
| delivery branch | `packet/nimbus/waker-hardening` |
| scratch DB | `seatloom_d2_nimbus` (001–014; NEVER live — a name guard refuses `seatloom`) |
| status | **COMPLETE** — gates green; integration suites green on scratch |

---

## 0. What was added (tests only)

Three layers, following the repo's established pattern (pure unit tests run in
`cargo test --workspace`; DB-backed tests are `#[ignore]` and run explicitly
against a scratch DB, like `tests/db_baseline_integration.rs`):

### Pure unit (always run, no DB)
- `crates/seatloom-mcp-emit/src/dispatch.rs` — `decide_completion` table
  (done; failed-within-budget → retry; failed-budget-exhausted and default-0 →
  terminal; the three event types are distinct); `CompleteArgs`/`EnqueueArgs`
  serde backward-compat; `lifecycle_event_with_payload` shape. (pre-existing,
  retained)
- `crates/seatloom-core/src/db/event_domain.rs` — **new** test pinning the six
  `013`/`014` event types are admitted, round-trip, and are distinct (each one
  transition), `acceptance` ref_type admitted, unknown types rejected.
- `src-tauri/src/commands/acceptance_cmds.rs` — the `AcceptanceResult` wire-shape
  test (both paths) and the merge conflict-refusal integration test (scratch git
  repo). (pre-existing, retained)

### DB integration — `crates/seatloom-core/tests/d2_acceptance_integration.rs` (NEW, 6 `#[ignore]`)
1. **accept** — `decide_acceptance("accepted", merge_commit)` ⇒ status=accepted,
   merge_commit, decided_by, decided_at all land; exactly one `AcceptanceGranted`
   subject event (same transaction).
2. **reject** — ⇒ status=rejected, feedback, rework_dispatch_id written back;
   one `AcceptanceRejected` event.
3. **terminal immutability** — a second decision of a decided acceptance is
   refused (`AcceptanceNotPending`); the row and its single event are unchanged.
4. **partial unique index** — a second `pending` acceptance for one dispatch is
   rejected by `uq_acceptances_one_pending_per_dispatch`.
5. **flux verdict existence** — `event_or_artifact_exists` finds the seeded
   verdict and reads a missing ref as absent (未验绿不可合).
6. **read model** — the joined card carries the dispatch title/seat and the
   verdict ("pass") + occurred_at; pending-first.

### DB integration — `crates/seatloom-mcp-emit/src/dispatch.rs::db_itests` (NEW, 5 `#[ignore]`)
(seatloom-mcp-emit is a bin crate — no `tests/` lib target — so these live in a
`#[cfg(test)]` module with crate-internal access to the tools + pool.)
1. **retry → terminal** — `complete(outcome="failed")` within budget ⇒
   in_flight→ready, retry_count+1, `SeatDispatchRetried`; pull again, exhaust ⇒
   in_flight→failed (terminal), `SeatDispatchFailed`; the full event sequence is
   asserted.
2. **default budget** — `max_retries=0` ⇒ the first failure is terminal.
3. **cancel issuer-only + slot release** — a non-issuer (the seat) is refused
   (`NotCancellable`); the issuer cancels ⇒ cancelled + `SeatDispatchCancelled`;
   the released in_flight slot lets the seat pull a fresh dispatch.
4. **request_acceptance guards + happy path + rework pullable** — not-completed
   and pending-exists guards fire; the happy path creates the pending
   `acceptances` row; a rework dispatch (as the reject path enqueues it) is
   pullable via `get_next_dispatch`.
5. **live_status terminal-as-stored** — a `failed` row with no unmet deps is
   NOT recomputed to ready and is never delivered.

## 1. Coverage map to the dispatch's five points

| Point | Covered by |
|---|---|
| 1 accept path | core itest #1 |
| 2 reject path + rework pullable | core itest #2 + mcp-emit itest #4 |
| 3 guards (accept-no-verdict, dup-pending, cancel issuer-only, request_acceptance ownership/done/no-pending/actor) | core itests #4/#5, mcp-emit itests #3/#4 |
| 4 013 transitions (retry/fail/cancel/terminal-immutable/live_status-as-stored) | mcp-emit itests #1/#2/#3/#5, core itest #3 |
| 5 event vocabulary (6 new types, one transition each, unknown rejected) | event_domain unit test + `decide_completion` distinct-types test |

## 2. Gates

- `cargo fmt --all --check` — **green**
- `cargo clippy --workspace --all-targets -- -D warnings` — **green** (tests included)
- `cargo test --workspace` — **green** (105 core + 40 mcp-emit + 4 tauri + 3;
  the 6+5 new DB tests are `#[ignore]`, so the gate passes without a DB)
- Integration suites against the scratch DB:
  - `DATABASE_URL=…/seatloom_d2_nimbus cargo test -p seatloom-core --test d2_acceptance_integration -- --ignored --test-threads=1` → **6 passed**
  - `DATABASE_URL=…/seatloom_d2_nimbus cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit db_itests -- --ignored --test-threads=1` → **5 passed**
  - Both REFUSE the live `seatloom` database (a name guard) and skip gracefully
    if 013/014 are not applied.

## 3. Notes

- The DB tests are env-serialised (the tools resolve identity from
  `SEATLOOM_ACTOR_REF`); run `--test-threads=1`, documented in each module
  header.
- Same stale-branch caveat as the prior D2 deliveries applies — Aegis should
  integrate the Rust backend path-scoped, not a whole-branch merge (see
  `disp-58ZddqlToyI5dgkC1jiIP-delivery.md` §2; a `SupervisorMessage` already
  stands for Aegis).
