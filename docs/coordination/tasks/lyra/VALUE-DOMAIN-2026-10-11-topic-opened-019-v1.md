# Value-domain addendum — TopicOpened(D5 Phase 2, migration 019,Lyra 签署)

**Owner:** Lyra (PO). **Status:** SIGNED. **Date:** 2026-10-11.
**Extends:** `VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md` (and the 015/018 family).
**Applies to:** `ONYX-2026-10-11-topic-opened-019-v1.md` (the remaining side; the Rust side
already landed — Nimbus `NIMBUS-2026-10-11-open-topic-delivery-v1.md`, `EventType::ALL`
pinned at 28, proof 7/7 against scratch 019).

**Sign-off of Nimbus's decision (Aegis ruling 1 required the judgment with both costs
written down — it was, in the delivery §1): ADD `TopicOpened`, do not reuse.** Rationale
carried verbatim from that delivery: no honest reuse candidate exists (`WorkItemCreated`
would record a topic as a work item, `ArtifactCreated` as an artifact, `SupervisorMessage`
as an escalation); in an append-only ledger one word for two meanings can never be split
once rows carry both — the same reasoning that kept `ArtifactProduced` out and gave
`dispatch` (012), `acceptance` (014) and `memory` (015/018) their own types.
`ref_type` 'topic' already exists (010) — **no ref_type widening**.

```
TOPICOPENED (migration 019) — value-domain contract, Lyra sign-off 2026-10-11

EVENT VOCABULARY +1: TopicOpened — a recall scope was opened deliberately.
  Exactly one transition per event (row + event in ONE transaction; refs subject =
  ref_type 'topic', actor = the seat that opened it, opened_by resolved server-side
  from SEATLOOM_ACTOR_REF, never a parameter). Idempotent re-open emits NO event.
  Full PG list after 019 = 28 values = the 27 live values + TopicOpened. 019 writes
  the COMPLETE list in its guarded widening (013's pattern), so it applies cleanly
  from the track state (27, post-018) or any intermediate.

CROSS-RUST<->PG TEXT ENUM RULE (restated, binding): hand-written FromStr/Display on
  the Rust side; NO serde derive across the boundary; unknown value = typed
  ParseValueError, never a silent default. Rust EventType::ALL pins 28 (landed);
  the PG CHECK must match that list VERBATIM.

LIFECYCLE (unchanged): topics is the scope unit under project; hierarchy via
  parent_topic_id (010); status vocab stays open|closed|merged. 019 adds no column,
  no table, no daemon.
```

## Wiring note

Until 019 is landed AND applied to live, `open_topic` refuses at the database CHECK
with the vocabulary violation — the tool is deployed but not live-usable (Nimbus
escalation #1, correct). The gate `ev-typed-memory-tools-landed` requires a REAL
live round-trip, not a scratch proof — its provenance is under verification with
Aegis (2026-10-11 night); see the SupervisorMessage emitted with this addendum.
