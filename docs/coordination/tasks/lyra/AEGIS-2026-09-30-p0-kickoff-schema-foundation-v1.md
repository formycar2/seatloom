# Dispatch: P0 kickoff — schema foundation (step 1)

[Aegis -> Lyra] Runtime-triple is fully reconciled to the GitLab `track/infra-foundation`
(`b6de097`) and the CI hard gate is green end-to-end (lint / test on real Postgres with
migrations+seed / docs ratchet). Mr. Zhang's standing directive was "push runtime-triple
through, then start P0." Both are now true. This is the GO for P0.

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-09-30-p0-kickoff-schema-foundation-v1 |
| status | dispatched |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | P0 capture loop — one real captured event |
| depends_on | `docs/coordination/reviews/2026-09-30-aegis-p0-build-plan-and-gap-register-v1.md` |
| tags | p0, dispatch, schema, data-model |

## Authoritative scope

The build plan is the single source of truth for P0:
`docs/coordination/reviews/2026-09-30-aegis-p0-build-plan-and-gap-register-v1.md`
(with its three dependencies: p0-scope-takeover-and-capture, data-flow-and-layering,
object-model-and-traceability). Read the build plan §2–§4 before authoring the packet.

P0 delivers **one true thing that does not exist today**: a record in SeatLoom because an
agent, on some harness, did something — filed under its project and topic, linked to what it
touched, retrievable, and independent of which harness produced it.

## Your task (PO)

Author and dispatch **the first packet only**: build plan §3 step 1, **Onyx — schema
foundation**. Do not dispatch steps 2–5 yet; they serialize behind this one (one shared
working tree — never two code packets in flight, per project memory).

### What the Onyx packet must cover (build plan §3 step 1, §4)

Bounded, additive, idempotent (applies twice with zero second-run change), rides as a
post-seed migration in the next free slot per `scripts/apply-db.sh` ordering, existing seed
still loads:

- **G2** — event envelope: `event_type` closed vocab + `schema_version INT NOT NULL` (per
  type, starts at 1) + validated `payload`.
- **G3** — `event_object_refs.role` closed domain `{subject, actor, context}`, exactly one
  `subject` per event; constrain `ref_type` to the object vocabulary.
- **G5** — **verify, do not add.** `sessions.project_id` already exists (migration 008).
  The packet confirms it and wires it into the contract; it must not re-add the column.
- **G6** — `topics(id, project_id, title, status, opened_by, opened_at, closed_at,
  parent_topic_id?)` + the event→topic ref (reuse the A3 role fabric, `role='subject'`,
  `ref_type='topic'`). Reserve the link now; auto-segmentation is P1.
- **G12** — reserve the T2 join key `(session_id, turn_seq)` on the events table now (column
  reserved; T2 capture is P1).

### The non-negotiable discipline (why runtime-triple stayed aligned)

Carry a **verbatim value-domain contract** in the packet — the same table reproduced word-for-word
in the Onyx packet and in the downstream Nimbus write-API packet: for every enum crossing the
Rust⇄PostgreSQL TEXT boundary, list Rust type, serialization domain, PG column, nullability.
This is the mechanism that kept Onyx and Nimbus's runtime-triple sides identical. Enums crossing
that boundary get **hand-written `FromStr`/`Display`** (ENGINEERING_STANDARDS), not serde derive —
see `crates/seatloom-core/src/objects/session.rs::parse_custom_tagged`.

### Acceptance for this packet

Flux verifies: migration idempotent (twice clean), value-domain contract stated and enforced
by a `SELECT DISTINCT` assertion, existing seed still loads, and the new CI `schema`/`contract`
stages pass on the branch.

## Sequencing after this (for your planning, do not dispatch yet)

step 2 Nimbus write API (A1, `append(event)→projections`, projections rebuildable from ledger)
→ step 3 `external_ref` (Onyx+Nimbus, pin seat sessions) → step 4 MCP emit server (Nimbus)
→ step 5 first real captured event (Nimbus + Flux verify). Steps 1–2 are the critical path.

## Open questions — decided as reached, not now (build plan §6)

Takeover strength, capture granularity, default-topic-per-session, topic↔workitem distinctness.
None blocks step 1. If Onyx hits one, surface it to me; do not stall the schema packet on it.
