# Sign-off: P0 step 1 schema foundation — decisions for Onyx's 010

[Aegis -> Lyra] I reviewed Onyx's delivery (`bc526ba`, `infra/postgres/schema/010_p0_schema_foundation.sql`).
It is high quality — idempotent DO-block guards, correct NULL discipline, G5 verify-only. Here are
the Aegis sign-offs you parked (daily log `c9d9cc7`), so you can dispatch Flux and merge.

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-09-30-p0-step1-signoff-v1 |
| status | dispatched |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | P0 capture loop — one real captured event |
| depends_on | `docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-kickoff-schema-foundation-v1.md` |
| tags | p0, dispatch, schema, sign-off |

## 1. event_type vocabulary — APPROVED as **12 values** (one change)

Mr. Zhang ratified the closed vocabulary. **Drop `ArtifactProduced`.** The P0 first slice
(build plan §3 step 5) **reuses the existing `ArtifactCreated`** instead of introducing a
near-synonym into an append-only table. `ArtifactCreated` is already in committed data, so no
new value is needed for capture. Final list (12):

```
SessionStarted, WorkItemCreated, WorkItemStatusChanged, ArtifactCreated,
HandoffSent, HandoffAccepted, HandoffCompleted, ReviewVerdictIssued,
SeatDelegationIssued, SeatDelegationClosed,   -- (a) present in committed data (10)
SupervisorMessage, SeatResponse               -- (b) written by live code, no row yet (2)
```

Onyx: one edit — remove the `'ArtifactProduced'` line (group c) from the
`canonical_events_event_type_vocab` CHECK. Everything else in the migration stands.

## 2. event_object_refs primary key — ENDORSED as delivered

`(event_id, ref_type, ref_id, role)` with the partial unique index
`WHERE role='subject'` (at-most-one subject; exactly-one is the write API's per-append
obligation in step 2). Onyx's reasoning is correct: self-referential events (an object as both
actor and subject) are ordinary, and the old key silently dropped the second role via
`ON CONFLICT DO NOTHING`. No change. Keep the recorded rationale in the migration.

## 3. Everything else Onyx decided — proceed as delivered

schema_version backfill=1 + explicit-set obligation on the write API; role legacy backfill to
`context` (never `subject`); turn_seq not backfilled (NULL ≠ 0); payload deliberately nullable;
ref_type vocab incl. forward-looking `topic`/`project`. All correct.

## 4. CI-stage correction — you were right, I was wrong

My kickoff's acceptance said "the new CI schema/contract stages pass." **There are no such
stages.** `.gitlab-ci.yml` at `b6de097` declares `stages: [lint, test, docs]`. Anchor Onyx's
acceptance to **`rust:test`** (real Postgres + `apply-db.sh` + DB integration tests — this is
where the value-domain assertions actually run) **and `docs:verify`**, exactly as you did. A
dedicated `schema`/`contract` stage is a *possible future CI packet*, not a P0 blocker — do not
build against stage names that do not exist.

## 5. How to reach Aegis (the escalation gap you flagged)

Correct and important: there is **no `Aegis-*-seatloom` tmux seat** — the empty seat was retired
because **Aegis is the supervisor session** (Mr. Zhang's assistant), not a worker seat. So
COORDINATION_RULES §12's "every seat reachable at `<Name>-<Role>-seatloom`" does **not** apply to
Aegis by design. Escalation model: park the question in the daily log (as you did) and/or raise it
to Mr. Zhang; Aegis observes the track and daily log and dispatches decisions to you directly
(this doc is that channel). Please add a one-line note to §12 that the supervisor is a session,
not a seat, so this stops looking like a broken invariant. Do not wait on a nonexistent Aegis seat.

## 6. Next

Onyx makes the one-line vocab edit (§1) → you dispatch Flux to verify against `rust:test` +
`docs:verify` (§4) → on PASS, merge to `track/infra-foundation`. Then Aegis reconciles the track
to GitLab through the gate. Step 2 (Nimbus write API) stays queued behind this on the one shared
tree — do not dispatch it until 010 is merged.
