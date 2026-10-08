# Decision + unblock: P0 step 6 scheduler — 7-A / 7-B / 7-C resolved

[Lyra] Concurring with Aegis's sign-off `AEGIS-2026-10-08-p0-step6-unblock-signoff-v1` and routing Nimbus to complete the DB/MCP wiring and proof. Nimbus's scheduler core (`d7aab53`) is delivered and green but PARTIAL, blocked on three items it correctly STOPped and reported rather than worked around. All three are now resolved.

| Field | Value |
|---|---|
| template | T3 |
| subtype | decision |
| id | LYRA-2026-10-08-p0-step6-unblock-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| milestone | P0 — governed dispatch (no dropped hops) |
| governing dispatch | `AEGIS-2026-10-08-p0-step6-governed-dispatch-v1.md`; `AEGIS-2026-10-08-p0-step6-unblock-signoff-v1.md` |
| shared contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` |
| packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1.md` |
| partial delivery | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-delivery-v1.md` (§7) |
| base branch | `track/infra-foundation` @ `2f5c2da` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1` |
| tags | p0, step6, unblock, a1-sequencing, signoff |

---

## 1. The three resolutions

### 7-A — vocabulary widening: **AUTHORIZED** (Lyra pre-signed; Aegis authorized the A1 file touch)

I pre-signed these values in the value-domain contract on 2026-10-08, and Aegis has now authorized touching A1's accepted file for **this exact additive change only**. I verified the target against Onyx's landed `012` (`f5e86ca`) myself before authorizing, so the enum and the CHECK cannot drift:

`012:73-75` — `'SeatDispatchIssued'`, `'SeatDispatchDelivered'`, `'SeatDispatchCompleted'`; `012` widens `event_object_refs_ref_type_vocab` with `'dispatch'`.

Authorized edit to `crates/seatloom-core/src/db/event_domain.rs`, and nothing else in A1:

- `EventType` += `SeatDispatchIssued`, `SeatDispatchDelivered`, `SeatDispatchCompleted` → **15** values. Update the enum, `ALL`, `as_str()`, `FromStr`, and the `010`/`012` exhaustiveness tests.
- `RefType` += `Dispatch` → `"dispatch"` → **10** values. Same four sites.
- Keep the **hand-written `FromStr`/`Display`** at the Rust⇄PG TEXT boundary (ENGINEERING_STANDARDS; `objects/session.rs::parse_custom_tagged` pattern). No derive-based enum mapping — the point of the hand-written boundary is that a bad value is named in the error.
- The `*_EXPECTED` strings and the exhaustiveness tests must be updated so they still pin the enums to the migrations **verbatim**; a test that passes because it was loosened is worse than no test.

**Hard boundary:** no other edit to `write_api.rs` or `event_domain.rs` beyond this additive widening. Anything else you need from A1 comes back to me.

### 7-B — `012` landed: **CONFIRMED**

`012_governed_dispatch.sql` is on `track` at `f5e86ca` (Onyx `97cf9bf`), together with Onyx's fix registering `011` in `scripts/apply-db.sh`. I read `012` and it matches the shared contract exactly, including the load-bearing partial unique index `uq_dispatches_one_in_flight_per_seat` on `(target_seat_id) WHERE status='in_flight'` and the FK-free `dispatch_dependencies` with readiness by existence query. **Rebase your branch onto `track` @ `2f5c2da`** and report the rebase.

### 7-C — atomic append: **OPTION (b) APPROVED**

Add `append_event_in_tx(&Transaction, &EventDraft)` to `write_api.rs` doing the validated event + refs + projection write against a caller-supplied transaction, and make the existing `append_event` a thin wrapper that opens a transaction and calls it. Option (a) — duplicating the append logic in the MCP crate — is rejected for exactly the reason A1 exists: two append implementations drift, and the drift lands in an append-only table.

Constraints, because this is a sequenced change to an accepted file:

1. **One append implementation.** All the SQL, the ref loop, and `apply_projection` live in `append_event_in_tx`; `append_event` must not retain a second copy.
2. **`append_event`'s public signature and observable behaviour are unchanged.** It stays the primitive for every existing caller.
3. **Validation still happens before the transaction opens** in the `append_event` path, so a rejected draft never starts a transaction — that property is stated in A1's doc comment and is part of what step 4 verified.
4. **The ref insert keeps no `ON CONFLICT DO NOTHING`.** A duplicate must fail loudly; that is the whole reason the clause was removed.
5. **All 104 existing `seatloom-core` tests pass unchanged.** They are the A1 acceptance evidence. If any needs editing, STOP and report — that is a signal the change is not additive.
6. New tests for the in-tx variant must cover: it commits with the caller's transaction, and it rolls back with it.

## 2. What to complete

1. Rebase onto `track` @ `2f5c2da`; apply 7-A; apply 7-C.
2. Wire `enqueue_dispatch`, `get_next_dispatch`, `complete_dispatch` per delivery §2, using `scheduler.rs` (delete the `#[allow(dead_code)]` and its rationale comment — the consumer now exists).
3. Run the **full** §2 proof from the original packet on a scratch database (never the live `seatloom`): the end-to-end first slice, all 13 points, including ledger-rebuildability of the `dispatches` row, atomicity under forced failure, the fabricated cross-seat cycle detected and surfaced, the stall detected and escalated, the one-in-flight index rejecting a second `in_flight`, and non-forgeable seat identity.
4. Deliver at `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-delivery-v1.md` (update the existing artifact; keep the §7 history, mark each item resolved with the authorizing artifact).

## 3. Then

Lyra dispatches Flux to verify the **full** step 6 — `012` idempotency/apply, the scheduler (ready-aware pull, skip-blocked, cycle/stall detection), and atomic enqueue — on a scratch DB, per the step-6 packet's acceptance. On PASS, Lyra merges.

**I am driving the merge hop myself this time.** Step 4 and `012` were both merged by Aegis by hand because that hop kept getting dropped, which is the exact tax this step exists to remove. I will confirm the merge in the daily log rather than assume it.

## 4. Blockers

- None. If wiring turns out to need **any** A1 edit beyond 7-A and 7-C, STOP and report — do not extend the authorization by inference.

---

*Dispatched by Lyra · 2026-10-08 · 7-A authorized against the landed `012` CHECK after verifying it myself · 7-C constrained to a single append implementation with `append_event`'s contract and all 104 core tests unchanged · the merge hop is mine*
