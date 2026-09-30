# Acceptance: P0 step 1 — schema foundation (migration 010)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | acceptance-2026-09-30-p0-schema-foundation |
| status | **ACCEPTED** |
| author | lyra |
| date | 2026-09-30 |
| verdict | **ACCEPTED** |
| packet | `docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-v1.md` |
| sign-off | `docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-step1-signoff-v1.md` |
| **accepted commit** | **`7226333f6ebf714596703aca84375eef05d6b173`** |
| merge commit | `cc662da` into `track/infra-foundation` |
| verification | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-schema-foundation-verification-delivery-v1.md` (PASS) |
| CI | pipeline **905464** @ `7226333` — success, all 5 jobs |
| tags | acceptance, onyx, schema-010, p0, step-1 |

---

## 1. Verdict

**ACCEPTED** at `7226333`, merged as `cc662da`. **P0 step 1 is complete.** Steps 2–5 are unblocked
in sequence; step 2 (Nimbus write API) is next and stays serial on the one shared tree.

Delivered: `canonical_events.schema_version` + closed 12-value `event_type` vocabulary +
`turn_seq` reserved; `event_object_refs.role` with the `{subject, actor, context}` domain and a
widened PK; `topics` reusing the existing ref fabric; `sessions.project_id` verified, not re-added.

## 2. Why this PASS is trustworthy

The gate named its own likely failure mode in advance, and Flux excluded it **by observation
rather than by assertion** — the distinction this project has had to learn repeatedly.

`010` creates its constraints under `IF NOT EXISTS (SELECT 1 FROM pg_constraint …)`. Correct for
idempotency, but it means applying `010` to a database still carrying the earlier **13-value**
constraint would silently leave the old one in place and produce a false PASS. Flux enumerated
`pg_database` first (only the live `seatloom` existed — no scratch DB to reuse), created fresh,
and then made the argument from the result rather than the procedure:

> the constraint read back out of this database carries 12 values with no `ArtifactProduced`. Had
> I reused a database from the earlier 13-value run, the guard would have left the old constraint
> in place and that read would have shown 13. It shows 12.

### 2.1 Both negative paths bite

A constraint that never rejects anything is not evidence it works — the same principle that made
the Nimbus parse-boundary check meaningful.

- **Vocabulary:** an out-of-vocab `event_type` is rejected by constraint name, while valid values
  insert. It **discriminates** — not rejecting everything, not accepting everything. Rejecting
  `ArtifactProduced` specifically also re-proves the 12-value version is the one under test.
- **Partial unique index:** a second `role='subject'` row for `ev-001` is rejected on duplicate
  key. At-most-one-subject is SQL's job here; **exactly-one is the step-2 write API's per-append
  obligation** — that boundary must survive into the Nimbus packet.

### 2.2 Backfill discipline held

`schema_version = 1`; `turn_seq` **NULL**, never `0`; every legacy ref `role='context'` and
**zero** backfilled to `subject`. The last one matters: asserting a subject the data cannot support
is the same error class as writing `Stepcode` onto an April session row, and it would have
poisoned the attribution layer P0 exists to build.

## 3. Decisions ratified in this packet

| Decision | Outcome |
|---|---|
| `event_type` vocabulary | **12 values.** `ArtifactProduced` dropped — the P0 first slice reuses the existing `ArtifactCreated`, which is already in committed data (`ev-050`–`ev-055`), rather than adding a near-synonym to an append-only table. Retired value survives only as a comment explaining why. |
| `event_object_refs` PK | Widened to `(event_id, ref_type, ref_id, role)`, endorsed as delivered. Onyx's reasoning stands: self-referential events are ordinary, and the old key silently dropped the second role via `ON CONFLICT DO NOTHING`. |
| G5 | **Verify-only.** `sessions.project_id` already exists from `008`; the column was not re-added. |
| Apply order | `010` registered post-seed with a justification string, **and 008 ordered before 010** — Onyx added an ordering constraint I had not specified, because `010`'s G5 guard raises if `project_id` is missing or nullable. The migration fails loudly rather than proceeding on a half-built schema. |

## 4. Corrections and process notes

1. **Aegis's kickoff CI claim was wrong and is now corrected in the record.** It made acceptance
   conditional on "the new CI `schema`/`contract` stages"; no such stages exist
   (`stages: [lint, test, docs]`). Aegis confirmed the correction in its sign-off §4. Acceptance
   was anchored to `rust:test` + `docs:verify`, and CI was **observed** at pipeline 905464, not
   inferred.
2. **"One-line edit" was not one line.** Dropping `ArtifactProduced` would have left
   `'SeatResponse',` with a dangling comma before `))` — a syntax error — and left `(c)` commentary
   describing a group that no longer existed. Flagged before dispatch; Onyx handled all three.
3. **COORDINATION_RULES §12 amended** (`26ca840`): Aegis is a supervisor *session*, not a worker
   seat, and has no `Aegis-*-seatloom` tmux session by design. My earlier "broken invariant"
   reading was wrong; the rule now states the real escalation route.

## 5. Open items

| # | Item | Owner |
|---|---|---|
| 1 | `project_id` on seed 001 session tuples — `apply-db.sh` still not idempotent across full runs (expected and re-observed at this gate; not a 010 defect) | Onyx |
| 2 | `verify-postgres-baseline.sh` → `run`/`rm`/`volume rm` form (podman has no compose provider) | Nimbus + Flux |
| 3 | Host `psql` client — exercised in CI, shimmed locally | Mr. Zhang |
| 4 | Ordered-migration runner + separate idempotent seed | Onyx |
| 5 | `seat-dispatch.sh` hardcodes "AEGIS dispatch", misattributing other seats | Aegis |
| 6 | A dedicated `schema`/`contract` CI stage — possible future packet, explicitly **not** a P0 blocker | Aegis |

## 6. Next

Aegis reconciles `track/infra-foundation` to GitLab through the gate. Then **step 2 — Nimbus write
API (G1/A1)**: one `append(event) → projections` path, no repository mutating a projection
directly, projections rebuildable from `canonical_events` alone. It must carry the §4 value-domain
contract verbatim and own the exactly-one-subject-per-append obligation §2.1 leaves to it.

---

*Lyra · 2026-09-30 · P0 step 1 COMPLETE · accepted `7226333`, merged `cc662da`, CI 905464 green ·
trap excluded by observation, both negative paths bite*
