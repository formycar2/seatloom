# Task: P0 step 6 (Nimbus half) — ready-aware scheduler + MCP pull delivery

[Lyra -> Nimbus] Aegis dispatch `AEGIS-2026-10-08-p0-step6-governed-dispatch-v1`: the scheduler half of governed dispatch (G19). **One contract governs both step-6 packets** — `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md`, carried verbatim into Onyx's packet and this one. Do not re-derive it. Onyx lands the migration in parallel; your write sets are disjoint.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| milestone | P0 — governed dispatch (no dropped hops) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step6-governed-dispatch-v1.md` |
| shared contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` (verbatim, both packets) |
| parallel packet | `docs/coordination/tasks/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1.md` |
| depends_on | migration `010`; write API A1 (merged); **step 4 MCP emit `af3a0f7` (merged)**; Onyx's `012_governed_dispatch.sql` |
| base branch | `track/infra-foundation` @ `b200cd3` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` (currently on the step-4 branch — start the new one from track) |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-delivery-v1.md` |
| tags | nimbus, p0, g19, governed-dispatch, scheduler, mcp, pull |

---

## 0. Why this step exists

Seats finish one hop and stop; dispatches get dropped; a human nudges every hop. Today Aegis had to merge step 4 by hand because Lyra dropped the merge hop. This is the step that lets the loop self-run: a seat **pulls** its next task when idle, so nobody has to remember to push it.

## 1. Scope — extend `crates/seatloom-mcp-emit`, do not fork it

Your step-4 crate is the sanctioned seat write path and it is merged. Add to it; do not create a second server, and do not change `emit_event`'s contract.

### 1.1 Tools to add

| tool | who calls it | what it does |
|---|---|---|
| `enqueue_dispatch` | the issuer (Aegis / Lyra) | inserts a `dispatches` row + its `dispatch_dependencies` rows and appends **`SeatDispatchIssued`** in the **same transaction** |
| `get_next_dispatch` | the seat, when idle (end of turn) | returns the **next ready, highest-priority** dispatch for the calling seat and marks it `in_flight`, appending **`SeatDispatchDelivered`** in the same transaction |
| `complete_dispatch` | the seat, when done | marks it `done`, appending **`SeatDispatchCompleted`** in the same transaction |

Notes:

- **The calling seat's identity must not be self-asserted.** `get_next_dispatch` must resolve the seat from the server's own context (`SEATLOOM_ACTOR_REF` / a seat env var), not from a caller-supplied `seat_id` parameter that any seat could spoof. State exactly how you resolve it and why it cannot be forged. If the only available identity is env-configured per server instance, say so and say what that costs.
- **Every status transition is an event in the same transaction as the row change** (contract rule 2). The `dispatches` row is a projection; `canonical_events` is the ledger. Reuse A1's `append_event`; do not add a second write path.
- `turn_seq` stays `None`. `schema_version` and `role` are set explicitly, per the contract and the step-4 rulings.

### 1.2 The ready-aware pull (the actual algorithm)

1. Resolve the calling seat.
2. If that seat already has an `in_flight` dispatch, return it (one-in-flight is a fact, not a preference) — do not hand out a second.
3. Otherwise select that seat's dispatches where `status = 'ready'`, ordered by `priority DESC, issued_at ASC`, take the first.
4. **Blocked items must not head-of-line-block.** A `blocked` dispatch is skipped, never waited on. Readiness is evaluated by **existence query** over `dispatch_dependencies` (`event` → `canonical_events.id`, `artifact` → `artifacts.id`, `dispatch` → `dispatches.id`), not by FK — Onyx's migration has no FK on `dep_value` by design.
5. Mark it `in_flight` + append the event, atomically.
6. Return the dispatch (id, title, payload, priority, dependencies satisfied) to the caller as the MCP tool result.

### 1.3 Cycle / stall detection

- **Cycle:** build the dependency graph over `dispatches` (including cross-seat edges via `dep_kind='dispatch'`) and detect a cycle. On detection: emit a **`SupervisorMessage`** event with `actor_ref` = the scheduler, `refs` = `[subject=<the offending dispatch>, actor=<the scheduler>]`, and **park a line in `docs/coordination/memory/YYYY-MM-DD.md`**. Never hang, never deadlock, never silently drop.
- **Stall:** a seat whose every dispatch is `blocked`, with no reachable ready item, is a stall — same escalation path. Distinguish "blocked, and something else is ready" (normal, not a stall) from "blocked, and nothing is ready" (stall, escalate).
- State your detection algorithm and its cost. A full graph walk per pull is acceptable for the first slice if you say so; an incremental check is better if it is not much more code.

### 1.4 What is NOT in the first slice

Auto-resolution, preemption, priority tuning, retry/backoff, multi-seat fan-out, and any UI. Decide them as reached — Aegis's dispatch is explicit that the first slice is **one seat, one dispatch, end to end**.

## 2. Verification

```
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Plus, on a **scratch** database (never the live `seatloom` — this rule has held for every gate so far and it holds here):

**The end-to-end first slice.** Aegis/Lyra enqueue → seat idle → pulls → acts → marks done, **all as events**:

1. `enqueue_dispatch` two items for one seat: one `ready`, one `blocked` on an event id that does not exist yet. Both append `SeatDispatchIssued`.
2. The seat calls `get_next_dispatch()` → it gets the **ready** one, **skipping the blocked one** (prove the blocked one was skipped, not merely that the ready one was returned).
3. The item is now `in_flight`; a second `get_next_dispatch()` from the same seat returns **the same item**, not a new one.
4. A second seat's `get_next_dispatch()` gets its own item — one-in-flight is per seat.
5. The seat calls `complete_dispatch()` → `done`, `SeatDispatchCompleted` appended.
6. When the dependency event lands (emit it), the previously blocked dispatch becomes ready and is pulled next. Show the transition.
7. **Every transition is an event.** `SELECT` `canonical_events` for the project and show the `SeatDispatchIssued` / `SeatDispatchDelivered` / `SeatDispatchCompleted` rows with their refs and roles.
8. **Atomicity:** force a failure mid-transition (e.g. a bad ref) and show the `dispatches` row and the event roll back together.
9. **Cycle:** fabricate a cross-seat cycle (A depends on B's dispatch, B depends on A's dispatch) and show it is **detected and surfaced** — an emitted `SupervisorMessage` and a parked daily-log line — not hung.
10. **Stall:** a seat with only blocked dispatches and nothing ready → detected and escalated, not hung.
11. **One-in-flight is a database fact:** attempt a second `in_flight` for the same seat directly in SQL and show the partial unique index rejects it.
12. **Identity cannot be forged:** attempt `get_next_dispatch` with a spoofed seat parameter (if your design accepts one at all) and show it does not return another seat's dispatch.
13. **The `dispatches` row is rebuildable from the ledger.** Rebuild it from `canonical_events` alone and show it matches — this is the A1 property, and it is the reason all three event types exist.

## 3. Integration with Onyx's migration

Onyx lands `012_governed_dispatch.sql` in parallel. **Rebase your branch onto Onyx's landed migration before delivery** and report the rebase. Your write set (`crates/seatloom-mcp-emit/**`) and Onyx's (`infra/postgres/schema/**`, `scripts/apply-db.sh`) are disjoint, so no conflict is expected — if one appears, STOP and report rather than resolving across the two packets' boundaries.

You may start on the scheduler logic, framing, and tests immediately against the contract; you cannot run the behavioural proof until `012` exists. Say in your delivery which parts were built before the migration landed.

## 4. Blockers

- None at dispatch time.
- **Report rather than work around** if the contract's shape is inexpressible without an A1 change or a Rust enum value that does not exist yet. `event_domain.rs` needs the three new `EventType` values and the new `RefType` value to compile against `012` — that file is A1's accepted file, so **flag it to me and I will sequence the change**; do not edit it unilaterally.

---

*Dispatched by Lyra · 2026-10-08 · P0 step 6, scheduler half · extends the merged step-4 crate, never forks it · delivery is MCP pull only, no mid-turn tmux injection · every transition an event in the same transaction · cycle/stall escalate, never hang*
