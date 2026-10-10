# Value-domain contract — memory hardening (D5 Phase 2, migration 018)

**Owner:** Lyra (PO). **Status:** ACTIVE — signed, lanes dispatched. **Date:** 2026-10-10.
**Applies to:** `ONYX-2026-10-10-memory-hardening-018-v1.md` (Lane A1, schema) and
`NIMBUS-2026-10-10-memory-tools-018-v1.md` (Lane A2, tool surface), and — by reference —
the Phase-2 washing pipeline packet (Lane C, seat-vela-001) and the forward-recording
protocol (Lane B). **Trigger:** Aegis dispatch `disp-jwWHSX8Q2OeN66pnV3teY`; design
authority `AEGIS-2026-10-10-washing-pipeline-design-v2.md` (Aegis branch).

Measured against live 015: five real gaps, 0 occurrences of each today. 018 closes them,
additive + idempotent. The D2 ruling stands: **no background daemon, no automatic decay**;
cold→hot stays recall-driven (hit ⇒ heat+1; threshold crossing via the explicit `promote` tool).

```
KNOWLEDGE-LAYER MEMORY HARDENING (D5 Phase 2, migration 018) — value-domain contract, Lyra sign-off 2026-10-10

G1 KIND (record type) — memories.kind TEXT NOT NULL + CHECK, vocab (6):
   decision | failure | fact | preference | artifact | unresolved
   Rationale: without it the six record types have nowhere to live but an unconstrained
   tags[] — you cannot ask "every failure in this project". Additive ALTER uses
   DEFAULT 'fact' for backfill ONLY; the remember tool REQUIRES kind explicitly
   (typed rejection when absent) — the default is never a write path.

G2 SUPERSESSION + RELATIONS:
   memories.supersedes TEXT NULL REFERENCES memories(id) (documents.supersedes style;
   create-time only, immutable after).
   memory_relations (
     from_id TEXT NOT NULL REFERENCES memories(id),
     to_id   TEXT NOT NULL REFERENCES memories(id),
     rel     TEXT NOT NULL CHECK IN ('supersedes','caused_by','fixed_by','refines'),
     PRIMARY KEY (from_id, to_id, rel))
   supersedes and memory_relations.rel='supersedes' are two spellings of the SAME edge;
   the write path keeps them consistent in ONE transaction. Recall DEFAULT excludes
   superseded rows (a stale decision served as current is worse than no memory).

G3 RESOLVED CLOSING STATE — memories.status vocab: active | forgotten | resolved
   (forgotten soft-tombstone semantics UNCHANGED). New columns:
   resolved_by TEXT NULL, resolved_at TIMESTAMPTZ NULL.
   CHECK: (status = 'resolved') = (resolved_by IS NOT NULL AND resolved_at IS NOT NULL).
   'unresolved' is the most executable record class; it must be closable. Closure is an
   explicit tool act (resolve), NEVER a background sweep.

G4 CONFIDENCE — memories.confidence TEXT NOT NULL + CHECK IN ('quoted','inferred')
   BINARY, never a score (LLM self-scored confidence is not trustworthy).
   'quoted' = the raw text says it; 'inferred' = derived. Existing rows backfill to
   'inferred' (they predate the distinction) via the additive DEFAULT; no row rewritten.

G5 L2 LINE RANGE (load-bearing wall — the convergence point of all loss risk):
   memories.l2_line_start INTEGER NULL, l2_line_end INTEGER NULL.
   CHECK: NOT (l2_ref_kind='file' AND source_kind='archived_session')
          OR (l2_line_start IS NOT NULL AND l2_line_end IS NOT NULL
              AND l2_line_end >= l2_line_start).
   "The evidence is somewhere in this 134MB file" is NOT evidence. Pruning (pipeline
   S2.5) may shrink what the LLM sees; it must NEVER shrink the L2 pointer.

EVENT VOCABULARY +1 (018): MemoryResolved — active -> resolved (the G3 closure
transition; exactly one transition, replayable). Full PG list after 018 = 27 values
(the 26 Nimbus's Rust pins today + MemoryResolved). 018 writes the COMPLETE list in its
guarded widening (013's pattern), so it applies cleanly from the track state (21), the
post-017 state (26), or any intermediate.

CROSS-RUST<->PG TEXT ENUM RULE (binding): every TEXT enum above is hand-written
FromStr/Display on the Rust side (objects/event_domain pattern); NO serde derive across
the boundary; unknown value = typed ParseValueError, never a silent default.

LIFECYCLE (unchanged, restated): tier cold|warm|hot, transitions ONLY via promote
(explicit target, up or down); heat +1 per recall only; forget = soft tombstone;
rows never hard-deleted. 018 adds no daemon and no automatic anything.
```

## Wiring

```
disp-mem018-onyx   (Onyx,  READY,  p50)  migration 018
      | on delivery emits event ev-018-memory-hardening-landed
      v
disp-mem018-nimbus (Nimbus, BLOCKED)    dependency {event, ev-018-memory-hardening-landed}
      | on delivery emits event ev-typed-memory-tools-landed
      v
Lane C part 2 (S3 extraction + S5 storage) may only run after BOTH gates exist
(013's trigger flips the dependent dispatch ready and pokes the seat; zero polling).
```

Lane B (forward recording) and Lane C part 1 (S0–S2.5, no LLM) start immediately and do
not wait on 018; only typed-record WRITES (remember with kind/confidence, S3/S5) wait.
