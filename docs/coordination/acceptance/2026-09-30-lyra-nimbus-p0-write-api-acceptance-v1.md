# Acceptance: P0 step 2 — write API (A1)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | acceptance-2026-09-30-p0-write-api |
| status | **ACCEPTED** |
| author | lyra |
| date | 2026-09-30 |
| version | v1 |
| verdict | **ACCEPTED** — three disclosed findings, none blocking |
| packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md` |
| **accepted commit** | **`47360b5df90b2b709865a92dd61843c78fb11a7e`** |
| code commit | `f5a3894` |
| merge commit | `454b795` into `track/infra-foundation` |
| verification | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-write-api-verification-delivery-v1.md` (PASS) |
| tags | acceptance, nimbus, p0, step-2, write-api, a1 |

---

## 1. Verdict

**ACCEPTED** at `47360b5`, merged as `454b795`. **P0 step 2 is complete.** The ledger is no longer
decorative: `append(event) -> projections` is the only way to change `workitems`.

Steps 4 (MCP emit), 6 (governed dispatch) and 5 (first real captured event, UM1) are unblocked in
sequence.

## 2. Why this PASS is trustworthy

Flux re-ran all three behavioural proofs rather than reading Nimbus's and agreeing, and verified
resulting database state **with its own queries rather than the harness's printout**.

| Proof | Why a happy path could not establish it | Result |
|---|---|---|
| **Atomicity** | The claim is that a *partial* write is impossible — success proves nothing | Forced mid-append failure: **0** event rows, **0** refs, **0** projection rows, and nothing else changed |
| **Zero-subject rejection** | `010:332` gives *at-most*-one; the API owes *presence* | Typed `WriteError::NoSubject`, raised before a connection is taken |
| **Rebuild from ledger alone** | The A1 acceptance itself | Field-by-field equality, **10 of 10** |

**The structural premise also survives a hardened grep:** `write_api.rs` is the only module issuing
`INSERT`/`UPDATE`/`DELETE` against `workitems`. I asked for this because Rust privacy is easy to
assert and easy to leak, and a single stray write elsewhere would defeat A1 entirely.

## 3. The §5.2 precision — my erratum overstated the guard

**This is the most important thing in this acceptance, and it corrects me.**

In ERRATUM `8eb55c2` I wrote that `serialized_sets_match_schema_010_checks` is "the guard against
this recurring." Flux checked, because I told it to, and **it is not** — not in the direction that
matters.

The test asserts equality with a **hardcoded `vec![…]` literal inside the test**, annotated with a
comment naming the `010` lines. Nothing reads the SQL: no `include_str!`, no file read, no query
against `pg_constraint`. Flux confirmed the only references to `010_p0_schema_foundation.sql` in
`crates/` or `src-tauri/` are two doc comments.

**Consequence:** the test fires if someone edits the Rust enum without editing the test. It
**cannot** fire if `010` changes and Rust does not — *which is the exact direction the original
eight-vs-nine error travelled.* The SQL admitted `delegation`; my contract table did not. The guard
covers one direction of a two-direction risk.

Flux closed the gap **for today, by hand**, comparing the Rust domains against constraints read out
of a real database built from `010`:

```
event_type : 12 values   == EventType::ALL
role       : 3 values    == RefRole::ALL
ref_type   : 9 values    == RefType::ALL
```

All three match exactly, **including order**. So the values are right today; only the automated
guard is one-directional.

**Follow-up owed (§5):** have the test derive its expected sets from the shipped `.sql` — parse it,
or assert against `pg_constraint` in the DB integration suite — so drift in **either** direction
fails the build.

## 4. Findings accepted, not blocking

1. **`append_supervisor_message` unmigrated** (`repositories.rs:405-433`). Verified independently
   by me: names neither `schema_version` nor `role`, so it is the **sole remaining consumer of both
   `010` defaults**; and it writes only a `ref_type='seat'` ref, making every supervisor message a
   **zero-subject event** — exactly what §2's rejection now refuses. No constraint is breached
   (`010:332` is at-most-one; zero satisfies it), so nothing breaks at runtime. The asymmetry is
   the finding: the ledger holds events the write API itself would refuse. **Own follow-up packet;
   it blocks the P1 default-drop.**
2. **`ref_type` is nine, not eight.** **My contract table was wrong, not the code.** `delegation`
   is in committed seed data (`ev-020`/`ev-021`) and merged `010` admits it (`010:265-271`). A
   Rust domain of eight could not read existing history. Nimbus notified me before landing, as the
   contract requires. ERRATUM `8eb55c2`, amended in place. Root cause: I copied my own earlier
   draft instead of re-measuring against the merged migration — the same defect class as the
   runtime-triple errata. **Rule extended: re-derive a value domain from the merged schema, never
   from a previous packet.**
3. **`payload` required by the API**, SQL column still nullable. Approved as delivered: the
   projection is rebuilt *from* the payload, so an event without one is unreplayable by
   construction and would reintroduce the decorative ledger step 2 removes. An API tightening; no
   `.sql` touched.

`artifact_produced_is_not_in_the_vocabulary` is a good defensive touch — it fails loudly if the
value dropped at `010` sign-off is ever silently re-added.

## 5. Open items

| # | Item | Owner |
|---|---|---|
| 1 | **Migrate `append_supervisor_message` onto the write API** — blocks #2 | Nimbus |
| 2 | Drop `010`'s `schema_version`/`role` defaults (P1) — blocked by #1 | Onyx |
| 3 | **Make the domain guard bidirectional** — derive expected sets from the `.sql` or `pg_constraint` (§3) | Nimbus |
| 4 | `project_id` on seed 001 session tuples — `apply-db.sh` not idempotent across full runs | Onyx |
| 5 | `verify-postgres-baseline.sh` → `run`/`rm`/`volume rm` form | Nimbus + Flux |
| 6 | Host `psql` client — exercised in CI, shimmed locally | Mr. Zhang |
| 7 | Ordered-migration runner + separate idempotent seed | Onyx |
| 8 | `seat-dispatch.sh` hardcodes "AEGIS dispatch" | Aegis |
| 9 | Backfill decision for pre-step-2 seeded history — provably lossy, not replayable | deferred |
| 10 | Stray scratch DB `seatloom_s2_nimbus` left on the server (Flux correctly did not drop another seat's) | Nimbus |

## 6. Next

Step 4 (MCP emit server) → step 6 (governed dispatch) → step 5 (first real captured event = UM1).
Mira's UM0 live-mirror packet is running in parallel in its own worktree on a disjoint file set,
with `types-dto.ts`/`dto-to-v1.ts` fenced off in her packet to prevent the one possible collision.

---

*Lyra · 2026-09-30 · P0 step 2 COMPLETE · accepted `47360b5`, merged `454b795` · §3 corrects my own
erratum: the recurrence guard is one-directional*
