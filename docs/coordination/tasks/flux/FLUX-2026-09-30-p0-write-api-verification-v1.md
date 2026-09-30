# Task: Commit-pinned verification — P0 step 2, write API (A1)

[Lyra -> Flux] The critical-path write side. `append(event) -> projections` is what steps 4, 6 and
5 all ride on, so this gate matters more than most. Nimbus delivered with three findings disclosed
and one contract deviation it correctly escalated to me before landing.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-p0-write-api-verification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | flux |
| verifies | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md` |
| delivery under test | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-delivery-v1.md` |
| target branch | `packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1` |
| **target commit** | **`47360b5`** (branch tip, carries the delivery doc) |
| code commit | `f5a3894` |
| base | `track/infra-foundation` @ `29adefd` |
| run location | `/Users/…/seatloom-worktrees/flux` — **detached HEAD at `47360b5`**, prepped, clean |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-write-api-verification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, p0, write-api, a1 |

---

## 0. State, verified by me before dispatch

Your worktree is **detached at `47360b5`**, not on the branch — Nimbus's worktree holds that
branch and git refuses two checkouts of one branch. Detached is the better state for a
commit-pinned gate anyway: HEAD is the exact SHA and cannot drift. I prepped `ui/node_modules`
and the `ui/dist` stub (both gitignored); the worktree is clean.

```
/Users/…/seatloom-worktrees/flux    47360b5 (detached HEAD)
/Users/…/seatloom-worktrees/nimbus  47360b5 [packet/nimbus/…]   <- Nimbus's, do not touch
/Users/…/seatloom-worktrees/mira    324e185 [packet/mira/…]      <- Mira's, do not touch
/Users/…/seatloom                   8eb55c2 [track/infra-foundation]
```

**Mira is working in parallel** on a frontend packet (UM0). Her worktree and branch are off
limits. The shared checkout stays on `track` — commit your delivery doc there.

## 1. Identity and scope

Standard §14.2 capture — note HEAD is a bare SHA, not a branch. Then:

```
git rev-parse HEAD                                  # must be 47360b5…
git diff --name-only 29adefd..47360b5 | grep -E '\.(tsx|sql)$'    # must be EMPTY
git show --stat f5a3894
```

Packet §5 forbade `.tsx` and `.sql`. Prove both.

## 2. Re-run Layer A

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test  -p seatloom-core --lib
cargo clippy -p seatloom-core -- -D warnings
cargo fmt --all --check
cd ui && pnpm exec tsc --noEmit
cd ui && pnpm build
```

You may set `CARGO_TARGET_DIR` to reuse artifacts; say if you did.

## 3. The three behavioural proofs — the heart of this gate

Nimbus's §5 claims all three. **Re-run them; do not read and agree.** Each is chosen because a
happy path cannot establish it.

### 3.1 Atomicity — prove the ROLLBACK

A normal append writes event + projection in **one transaction**. A forced failure mid-append must
roll back **both**. The success path proves nothing here: the claim is that a *partial* write is
impossible. After a forced failure, assert **neither** the `canonical_events` row **nor** the
projection row exists.

### 3.2 Zero-subject rejection

An append carrying zero subjects must be rejected with a **typed** error before it reaches the
database. `010:332` gives *at-most*-one; this API owes *presence*. Confirm the error is typed
(`WriteError::NoSubject` per the delivery), not a panic and not a silent drop.

### 3.3 Rebuild from the ledger alone

The A1 acceptance: a projection rebuilds from `canonical_events` + `event_object_refs` **alone**,
field-by-field equal to its pre-truncate state. If it cannot, the ledger is decorative and step 2
is not done.

Use a **fresh scratch database** (`DROP`/`CREATE`, or a new name). Same reasoning as the 010 gate.
Never write to the live `seatloom` DB — Nimbus reports the harness refuses to start against a DB
named `seatloom`; do not test that claim against the live database.

## 4. Check the structural claim, not just the behaviour

Nimbus states *"nothing outside `db::write_api` writes a projection table — `apply_projection` and
`replay_workitem_event` are private to the module, so the only way to change `workitems` is to
append an event."*

**Verify that by reading, not by trusting.** Grep for writes to projection tables outside the write
API module. A single `UPDATE workitems` or `INSERT INTO workitems` elsewhere in `crates/` or
`src-tauri/` defeats the entire premise of A1, and privacy in Rust is easy to assert and easy to
leak.

## 5. Three disclosed findings — confirm, do not re-litigate

Nimbus disclosed these; I have already ruled on them. Confirm each is accurately described and
**out of scope for this gate**.

1. **`append_supervisor_message` is unmigrated** (`repositories.rs:405-433`). I verified it
   independently: its `canonical_events` INSERT does not name `schema_version` (depends on
   `010:87` `DEFAULT 1`), its `event_object_refs` INSERT does not name `role` (depends on the
   `context` default), and it writes only a `ref_type='seat'` ref — so every supervisor message is
   a **zero-subject event**, exactly what §3.2 now rejects. No constraint is breached (`010:332`
   is at-most-one, zero satisfies it), so nothing breaks at runtime. **Ruled out of scope; its own
   follow-up packet.** Do not fix it.
2. **`ref_type` is nine values, not eight.** My contract table was wrong — `delegation` is in
   committed seed data (`ev-020`/`ev-021`) and merged `010` admits it (`010:265-271`). Nimbus
   notified me before landing, as the contract requires. I have issued an ERRATUM (`8eb55c2`).
   **Confirm `RefType` carries nine and that `serialized_sets_match_schema_010_checks` genuinely
   asserts equality with `010`** — that test is the guard against this recurring.
3. **`payload` required by the API** while the SQL column stays nullable. An API tightening, not a
   schema change. Confirm no `.sql` was touched.

## 6. Out of scope

- Do not run `scripts/verify-postgres-baseline.sh` (no compose provider; destructive).
- Do not fix findings. A defect is a Nimbus rework finding.
- Do not touch Mira's or Nimbus's worktrees, or the shared checkout's branch.
- A second full `apply-db.sh` run failing on `sessions.project_id` is **expected** — known
  pre-existing seed defect, own follow-up. Note and move on.

## 7. Verdict and delivery

PASS / HOLD / FAIL. Partial verification is HOLD with the reason, not PASS with a caveat.

Delivery doc in the **shared checkout** on `track`: identity captures (noting detached HEAD); the
§1 scope proof; every §2 command verbatim; **your own** §3.1/3.2/3.3 outputs including the rollback
and the equality assertion; your §4 structural read with the grep evidence; your §5 confirmations;
whether you used `CARGO_TARGET_DIR`; and the verdict.

---

*Dispatched by Lyra · 2026-09-30 · P0 step 2 · §3 re-runs the three proofs rather than accepting
them; §4 checks the A1 premise structurally, because Rust privacy is easy to assert and easy to
leak*
