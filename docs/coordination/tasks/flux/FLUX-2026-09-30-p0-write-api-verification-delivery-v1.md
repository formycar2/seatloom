# Delivery: Commit-pinned verification — P0 step 2, write API (A1)

[Flux -> Lyra] Verdict: **PASS**, with one precision on §5.2 you should read before relying on it

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-p0-write-api-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-write-api-verification-v1.md` |
| delivery under test | `NIMBUS-2026-09-30-p0-write-api-delivery-v1.md` (on the Nimbus branch) |
| target branch | `packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1` |
| **verified commit** | **`47360b5df90b2b709865a92dd61843c78fb11a7e`** (detached HEAD) |
| code commit | `f5a3894` |
| base | `29adefd` |
| scratch database | `seatloom_s2_flux` — created fresh, dropped after |
| CARGO_TARGET_DIR | **used** — shared checkout's `target/` |
| verdict | **PASS** |
| tags | flux, verification, commit-pinned, p0, write-api, a1, pass |

---

## 0. Verdict

**PASS.** All three behavioural proofs hold when I run them, and I verified the resulting database
state with my own queries rather than reading the harness's printout. The §4 structural claim
survives a hardened grep. Layer A is green including `fmt --check`. Scope holds: zero `.tsx`, zero
`.sql`.

**One precision, and it is the only thing I would not want you to take on trust.** You asked me to
confirm that `serialized_sets_match_schema_010_checks` "genuinely asserts equality with 010",
because you are relying on it to stop the eight-vs-nine error recurring. It does not assert
equality with `010`. It asserts equality with a **hardcoded transcription** of `010`, annotated
with line references. Detail and consequence in §5.2 — not a FAIL, because I separately proved the
values are correct against a real database, but the guard is one-directional and you should know
that before you count on it.

---

## 1. Identity and scope

```
$ git -C …/flux rev-parse HEAD
47360b5df90b2b709865a92dd61843c78fb11a7e     <- MATCHES target
$ git -C …/flux rev-parse --abbrev-ref HEAD
HEAD                                          <- detached, as you arranged
$ git -C …/flux status --short
(empty — clean)
```

Clean before and after. Detached HEAD confirmed — and I agree it is the better state for a
commit-pinned gate: there is no branch ref that could move under me.

```
$ git diff --name-only 29adefd..47360b5 | grep -E '\.(tsx|sql)$'
(empty — ZERO .tsx and ZERO .sql)

$ git diff --stat 29adefd..47360b5
 crates/seatloom-core/examples/p0_write_api_proof.rs |  645 +++++
 crates/seatloom-core/src/db/event_domain.rs         |  374 +++++
 crates/seatloom-core/src/db/mod.rs                  |    6 +
 crates/seatloom-core/src/db/repositories.rs         |    7 +
 crates/seatloom-core/src/db/write_api.rs            |  905 +++++
 …NIMBUS-2026-09-30-p0-write-api-delivery-v1.md      |  741 +++++
 6 files changed, 2678 insertions(+)
```

`git show --stat f5a3894` is the same five code files, 1937 insertions. Both §5 boundaries hold;
§5.3's "no `.sql` touched" is proven by the same filter.

Worktrees at close: Mira's on `324e185`, Nimbus's on `47360b5`, both untouched by me; shared
checkout still on `track`.

## 2. Layer A — re-run by me

```
cargo check  -p seatloom-core           -> EXIT 0
cargo check  -p seatloom-tauri          -> EXIT 0
cargo test   -p seatloom-core --lib     -> EXIT 0   95 passed; 0 failed; 0 ignored
cargo clippy -p seatloom-core -- -D warnings -> EXIT 0
cargo fmt --all --check                 -> EXIT 0
cd ui && pnpm exec tsc --noEmit         -> EXIT 0   (zero output lines)
cd ui && pnpm build                     -> EXIT 0
```

`CARGO_TARGET_DIR` pointed at the shared checkout's `target/` for artifact reuse. Cargo fingerprints
on source content and flags, so the compilation was still of the worktree's sources at `47360b5`.
I did not touch the shared checkout's branch.

---

## 3. The three behavioural proofs — run, then independently verified

I ran Nimbus's harness (`cargo run -p seatloom-core --example p0_write_api_proof`) against a
**fresh** scratch database, and then checked the database myself. Running someone's harness and
reading its `HELD` lines is still trusting the harness; the independent queries in §3.4 are what
make this verification rather than agreement.

Fresh database, proven fresh:

```
$ psql -c 'DROP DATABASE IF EXISTS seatloom_s2_flux;' -c 'CREATE DATABASE seatloom_s2_flux;'
DROP DATABASE
NOTICE:  database "seatloom_s2_flux" does not exist, skipping
CREATE DATABASE
$ DATABASE_URL=…/seatloom_s2_flux bash scripts/apply-db.sh   -> exit 0
```

I used my own scratch name. **Nimbus's `seatloom_s2_nimbus` was neither reused nor dropped** — it
is still on the server, untouched.

Harness exit code **0**, which it returns only if every proof holds.

### 3.1 Atomicity — the rollback, not the success

The fault is injected with a **real constraint, not a test hook**, which is what makes this
evidence: `owner_seat_id` points at a non-existent seat, so the projection INSERT violates
`workitems.owner_seat_id REFERENCES seats(id)` (001:85) **after** `canonical_events` and
`event_object_refs` have already been inserted inside the same transaction. That is exactly the
window a partial write would occupy.

```
append_event returned Err, as required:
  caused by: ERROR: insert or update on table "workitems" violates foreign key constraint
             "workitems_owner_seat_id_fkey"
  sqlstate=23503 constraint=Some("workitems_owner_seat_id_fkey") table=Some("workitems")
  detail: Key (owner_seat_id)=(seat-does-not-exist-999) is not present in table "seats".

after the failed append:
  canonical_events  rows for ev-p0-s2-rollback = 0   (must be 0)
  event_object_refs rows for ev-p0-s2-rollback = 0   (must be 0)
  workitems         rows for wi-p0-s2-doomed   = 0   (must be 0)
  proof-1 projection row still present and unchanged: status=in_progress
```

That last line matters as much as the zeros: the rollback removed the failed append and **nothing
else**. A transaction that rolled back too much would also show three zeros.

### 3.2 Zero-subject rejection — typed, pre-database

```
typed error WriteError::NoSubject
  event_id   = ev-p0-s2-nosubject
  event_type = WorkItemCreated
  ref_count  = 2
  rendered   = event "ev-p0-s2-nosubject" (WorkItemCreated) carries no subject ref:
               exactly one ref with role='subject' is required, found 0 of 2 ref(s)
canonical_events rows for ev-p0-s2-nosubject = 0   (rejected pre-database)
```

Typed variant, not a panic and not a silent drop. `ref_count = 2` is the right shape for the test —
the event *had* refs, just none with `role='subject'`, so this rejects on the subject rule rather
than on emptiness. Together with `010:332`'s at-most-one, that is exactly-one.

### 3.3 Rebuild from the ledger alone

```
rebuild_workitems_projection(p0-s2-proof) -> deleted 1 row(s), replayed 2 event(s)

field-by-field equality (10 of 10):
  id, title, goal, acceptance_criteria, owner_seat_id, status, priority,
  created_at, updated_at, project_id      — all ==
  status = done          updated_at = 2026-09-30T12:00:00+00:00
```

`status='done'` and `updated_at=12:00` could only come back if the **second** event replayed, not
just the CREATE. That is the difference between a ledger that is causal and one that is decorative.

### 3.4 My own verification of the resulting state

Independent of the harness's own assertions:

```
        check         | n
----------------------+---
 rollback_event       | 0
 rollback_refs        | 0
 rollback_projection  | 0
 nosubject_event      | 0
 nosubject_refs       | 0
 events_in_project    | 2
 workitems_in_project | 1
```

And I re-derived the projection from the raw ledger myself, to check the rebuild was real rather
than a row that merely survived:

```
ev-p0-s2-create  WorkItemCreated        10:00  payload: title "P0 step 2 write API",
                                               status in_progress, priority high,
                                               owner_seat_id seat-nimbus-001, goal, acceptance_criteria
ev-p0-s2-status  WorkItemStatusChanged  12:00  payload: {from_status: in_progress, to_status: done}

refs:  ev-p0-s2-create -> workitem wi-p0-s2-001 SUBJECT, seat actor, project context
       ev-p0-s2-status -> workitem wi-p0-s2-001 SUBJECT, seat actor

projection: status=done, created_at=10:00, updated_at=12:00, title/priority/owner from the CREATE payload
```

Reading only the ledger, the expected projection is: fields from the CREATE payload, `status` from
the **later** status event, `created_at` = CREATE's `occurred_at`, `updated_at` = latest event's
`occurred_at`. That is precisely the row present. Each event carries exactly one `subject` ref.
**The projection is derivable from the ledger by my own reading, not just by the harness's claim.**

---

## 4. The structural claim — checked by reading and grepping

Nimbus's claim: *nothing outside `db::write_api` writes a projection table.*

**Privacy, confirmed:**

```
crates/seatloom-core/src/db/write_api.rs:518:async fn apply_projection(       <- no `pub`
crates/seatloom-core/src/db/write_api.rs:557:async fn replay_workitem_event(  <- no `pub`

$ grep -n 'pub ' crates/seatloom-core/src/db/mod.rs
pub use write_api::{AppendOutcome, EventDraft, ObjectRefDraft, ProjectionEffect, WriteError};
```

Only types are re-exported — no projection-writing function escapes the module.

**Every write statement to a projection or ledger table in `crates/` + `src-tauri/`:**

```
write_api.rs        :: INSERT INTO workitems / UPDATE workitems / DELETE FROM workitems
write_api.rs        :: INSERT INTO canonical_events / INSERT INTO event_object_refs
repositories.rs     :: INSERT INTO canonical_events / INSERT INTO event_object_refs
p0_write_api_proof.rs :: DELETE FROM workitems / event_object_refs / canonical_events
```

Because a single-line grep can miss SQL split across lines, I also read **every** occurrence of
`workitems` in every `.rs` file outside `write_api.rs`. All of them are: read-only `SELECT`s
(`repositories.rs:211-247`), integration tests, the proof fixture, the separate `.seatloom/` YAML
file-store (`storage/workitem_store.rs`, not the Postgres projection), and Tauri commands that only
call `list`/`get`. **No `INSERT`, `UPDATE` or `DELETE` on `workitems` exists outside `write_api.rs`
and the proof example.**

**The claim holds as stated — and here is the precision it depends on.** `repositories.rs` *does*
write `canonical_events` and `event_object_refs` from outside the write API. Those are **ledger**
tables, not projection tables, so Nimbus's sentence is exactly true. But the complementary property
— *the ledger only grows through `append`* — is **not** true, and that gap is finding §5.1. For
supervisor messages there is no projection, so nothing can diverge today; the A1 premise is intact.
It is worth stating precisely because "only the write API can change state" is the natural
paraphrase, and that paraphrase is false.

---

## 5. The three disclosed findings — confirmed, not re-litigated

### 5.1 `append_supervisor_message` unmigrated — accurately described

Read at `repositories.rs:405-455`. Every element of your description checks out:

```rust
"INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload, project_id) …"
    // names no schema_version -> depends on 010:87 DEFAULT 1
"INSERT INTO event_object_refs (event_id, ref_type, ref_id) VALUES ($1, 'seat', $2) ON CONFLICT DO NOTHING"
    // names no role -> depends on the 'context' default; ref_type is 'seat' only
```

So every supervisor message is a zero-subject event, `010:332` is at-most-one, zero satisfies it,
nothing breaks at runtime. Out of scope, not fixed, not re-litigated.

**Two additional facts for the follow-up packet's scope** — offered because they widen it slightly,
not to re-open the ruling:

1. The two INSERTs are **not in a transaction** — separate `client.execute` calls with no `BEGIN`.
   So this path is non-atomic in exactly the way §3.1 proves `append_event` is not.
2. The refs INSERT result is **discarded** (`let _ = client.execute(…)`), so a failure to write the
   ref is silently ignored.

Given Supervisor IM is the L1 surface, the follow-up is worth scoping to "migrate onto
`append_event`" rather than "name the two columns".

### 5.2 `ref_type` is nine — confirmed; the guard is weaker than its name

`RefType` carries **exactly nine**: `WorkItem, Session, Handoff, Artifact, Document, Seat, Topic,
Project, Delegation`. Confirmed.

**But `serialized_sets_match_schema_010_checks` does not assert equality with `010`.** It asserts
equality with a hardcoded `vec![…]` literal inside the test, carrying a comment that *names* the
010 lines:

```rust
assert_eq!(refs, vec!["workitem","session",…,"delegation"],
           "must equal 010:270-276 event_object_refs_ref_type_vocab");
```

Nothing reads the SQL. I checked: the only references to `010_p0_schema_foundation.sql` anywhere in
`crates/` or `src-tauri/` are two doc comments — no `include_str!`, no file read, no query against
`pg_constraint`.

**Consequence:** the test fires if someone edits the Rust enum without editing the test. It cannot
fire if `010` changes and Rust does not — which is the exact direction the original eight-vs-nine
error travelled (the SQL admitted `delegation`; the contract table did not). As a guard against
recurrence it covers one direction of a two-direction risk.

**So I closed the gap for today by hand** — compared the Rust lists against the constraints in a
real database built from `010`:

```
DB canonical_events_event_type_vocab : SessionStarted WorkItemCreated WorkItemStatusChanged
    ArtifactCreated HandoffSent HandoffAccepted HandoffCompleted ReviewVerdictIssued
    SeatDelegationIssued SeatDelegationClosed SupervisorMessage SeatResponse      (12)
DB event_object_refs_role_vocab      : subject actor context                       (3)
DB event_object_refs_ref_type_vocab  : workitem session handoff artifact document
                                       seat topic project delegation               (9)
```

All three match `EventType::ALL`, `RefRole::ALL` and `RefType::ALL` **exactly, including order**.
The values are right today. Suggested follow-up, your call and not this gate's business: have the
test derive the expected sets from the shipped `.sql` (parse it, or assert against
`pg_constraint` in the DB integration suite) so drift in either direction fails the build.

`artifact_produced_is_not_in_the_vocabulary` is a nice touch — it fails loudly if the dropped value
is silently re-added.

### 5.3 `payload` required by the API, column nullable — confirmed

An API tightening only. **Zero `.sql` files** in the delta (§1), so the column is untouched and
still nullable in the schema. Accurately described.

---

## 6. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | HEAD == target, detached | `47360b5` | `47360b5df90b2b7…`, detached | PASS |
| 1 | worktree clean before/after | empty | empty both | PASS |
| 1 | zero `.tsx` | 0 | 0 | PASS |
| 1 | zero `.sql` | 0 | 0 | PASS |
| 2 | `cargo check` core / tauri | 0 | 0 / 0 | PASS |
| 2 | `cargo test --lib` | green | 95 passed, 0 failed | PASS |
| 2 | `clippy -D warnings` | 0 | 0 | PASS |
| 2 | `cargo fmt --all --check` | 0 | 0 | PASS |
| 2 | `tsc --noEmit` | 0 errors | 0 | PASS |
| 2 | `pnpm build` | 0 | 0 | PASS |
| 3 | fresh scratch DB | fresh | NOTICE confirms; Nimbus's untouched | PASS |
| 3 | harness exit code | 0 | 0 | PASS |
| **3.1** | **forced failure rolls BOTH back** | both absent | event 0, refs 0, projection 0 | **PASS** |
| 3.1 | fault is a real constraint | real | FK `workitems_owner_seat_id_fkey`, sqlstate 23503 | PASS |
| 3.1 | rollback removed nothing else | unchanged | proof-1 row intact | PASS |
| **3.2** | **zero-subject typed rejection** | typed | `WriteError::NoSubject`, 0 rows written | **PASS** |
| **3.3** | **rebuild equals pre-truncate state** | field-equal | 10/10 fields equal, 2 events replayed | **PASS** |
| 3.4 | state verified independently | matches | my own queries agree | PASS |
| 3.4 | projection derivable from ledger | derivable | re-derived by hand; matches | PASS |
| 4 | projection fns module-private | private | no `pub` on either | PASS |
| 4 | no projection write outside write_api | none | none (hardened grep + full read) | PASS |
| 5.1 | supervisor-message finding accurate | accurate | accurate; +2 extra facts | PASS |
| 5.2 | `RefType` carries nine | 9 | 9 | PASS |
| 5.2 | guard test asserts equality with 010 | asserts | **asserts vs a transcription** | **precision — see §5.2** |
| 5.2 | enums match a real DB built from 010 | match | exact, including order | PASS |
| 5.3 | payload tightening, no `.sql` | none | none | PASS |
| — | scratch DB dropped | yes | dropped | PASS |
| — | live DB never written | untouched | see §7 | PASS |
| — | other seats' worktrees untouched | untouched | Mira `324e185`, Nimbus `47360b5` | PASS |

The known second-full-`apply-db.sh` `project_id` failure: noted, not re-run this gate — I proved it
pre-existing with a control experiment in the v3 delivery and confirmed it again at the 010 gate.
Unchanged, still its own follow-up.

---

## 7. Closing state

```
$ psql -tAc "SELECT datname FROM pg_database WHERE datname LIKE 'seatloom%'"
seatloom
seatloom_s2_nimbus            <- Nimbus's, deliberately left alone

$ psql -d seatloom -tAc "…"
seats=6 sessions=5 topics_table=0 canonical_events_has_project_id=0
```

The live database is not merely unwritten — it is **provably un-migrated**: it has no `topics`
table and `canonical_events` has no `project_id` column, so it has never seen `008` or `010`, let
alone this write API. My first attempt to count `p0-s2-proof` rows there failed with
`ERROR: column "project_id" does not exist`, which is the cleanest proof available that nothing
from this gate reached it.

```
flux    47360b5df90b2b709865a92dd61843c78fb11a7e (detached)  clean
nimbus  47360b5 [packet/nimbus/…]     untouched
mira    324e185 [packet/mira/…]       untouched
shared  track/infra-foundation        clean, branch never switched
```

Scratch database `seatloom_s2_flux` dropped; the temporary `psql` shim removed. I did not run
`verify-postgres-baseline.sh`, did not fix any finding, did not edit `.sql`, and created no stash.
Tooling caveat unchanged: still no `psql` client on this host, so SQL ran through a shim to
`podman exec`.

---

## 8. Recommendation

**Merge.** Step 2 is sound and the three proofs are the right three — each genuinely cannot be
established by a happy path, and each holds under my own re-run and my own queries.

Two things to carry, neither blocking:

1. **§5.2's guard is one-directional.** Worth a small follow-up so the vocabulary test derives its
   expectation from the shipped `.sql` rather than a transcription. Today's values are verified
   correct against a real database, so this is about preventing recurrence, which is exactly what
   you said you were relying on that test for.
2. **Scope the supervisor-message follow-up to "migrate onto `append_event`"**, given §5.1's two
   extra facts (non-transactional, discarded ref error) and that this is the L1 surface.

---

*Flux · 2026-09-30 · PASS · verified commit `47360b5df90b2b709865a92dd61843c78fb11a7e` ·
detached-HEAD worktree run · three proofs re-run and then independently re-verified against the
database · A1 premise checked structurally, including the ledger/projection distinction it rests on*
