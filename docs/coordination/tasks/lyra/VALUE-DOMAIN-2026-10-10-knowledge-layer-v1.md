# Value-domain contract — SeatLoom knowledge layer (D5)

**Owner:** Lyra (PO). **Status:** ACTIVE — signed, lanes dispatched. **Date:** 2026-10-10.
**Applies to:** `ONYX-2026-10-10-d5-knowledge-layer-schema-v1.md` (Lane A) and
`NIMBUS-2026-10-10-d5-mcp-recall-tools-v1.md` (Lane B).

This is the single contract both D5 lane packets carry **verbatim**, the same way the
governed-dispatch packets were joined by one Lyra-owned value-domain contract
(`VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` + its D2 addendum, both still in
force — D5 extends, never re-derives). Neither seat re-derives it; a deviation is
reported, not silently reconciled.

```
KNOWLEDGE LAYER (D5) — value-domain contract, Lyra sign-off 2026-10-10

SCOPE FRAMEWORK (unchanged ruling): project + topic is the ONLY scoping framework.
Every knowledge object carries BOTH ids. Agent harness + LLM are execution entities
serving project+topic, never a scope of their own.

1. PROJECT TREE — projects.parent_project_id (NEW, migration 015)
   parent_project_id TEXT NULL REFERENCES projects(id)
   NULL = root project. CHECK (parent_project_id IS DISTINCT FROM id).
   Rationale: history already shows hierarchy (sf-code/storage, ai-assistant/sf-package,
   fde/field_evaluation); the tree expresses it without inventing a second framework.

2. PROJECT SLUG NORMALIZATION RULE (archived_sessions.project free text -> project tree)
   R1 exact:   slug == projects.id -> that project.
   R2 path:    'a/b' -> leaf project 'b' under parent 'a'; 'a/b/c' nests likewise.
   R3 the rule CREATES NO ROWS. Materializing the tree is the Phase-2 washing pipeline
      (Aegis lane). 015 ships the rule as an executable object + a verification query
      over all 271 archived_sessions rows; it inserts zero project rows.
   R4 unmapped: unknown slug -> map_status 'unmapped', reported, never guessed.
   R5 '(root)' (13 sessions): 'unmapped' + ESCALATE to Aegis. No ruling is assumed
      here; the strategy decision is requested, not taken.

3. TOPICS — UNCHANGED by 015. The 010 topics table is the topic unit; hierarchy is
   ready via parent_topic_id; status vocab stays open|closed|merged. 015 must not
   alter topics' columns or vocab.

4. MEMORIES (NEW table, 015) — small-granularity distilled knowledge.
   Large refined products stay in documents (002): do not fork that model.
   id                 TEXT PRIMARY KEY
   project_id         TEXT NOT NULL REFERENCES projects(id)
   topic_id           TEXT NOT NULL REFERENCES topics(id)
   title              TEXT NOT NULL
   tags               TEXT[] NOT NULL DEFAULT '{}'
   l0_summary         TEXT NOT NULL          -- ~100 tokens; the ONLY default layer
   l1_overview        TEXT                   -- ~2K tokens; fetched on demand
   l2_ref_kind        TEXT NOT NULL          -- document | file
   l2_document_id     TEXT NULL REFERENCES documents(id)
   l2_file_path       TEXT NULL              -- repo-relative, documents.file_path style
   l2_body_digest     TEXT NULL              -- sha256 hex, documents.body_digest style
   CHECK ((l2_ref_kind = 'document' AND l2_document_id IS NOT NULL)
       OR (l2_ref_kind = 'file' AND l2_file_path IS NOT NULL))
   source_kind        TEXT NOT NULL          -- archived_session | canonical_event | document | manual
   source_ref         TEXT NOT NULL          -- '<harness>:<id>' | event id | doc id | actor note
                                             -- polymorphic TEXT, NO FK (010 ref-fabric reasoning)
   heat               INTEGER NOT NULL DEFAULT 0    -- +1 per recall; NO automatic decay, ever
   recall_count       INTEGER NOT NULL DEFAULT 0
   last_recalled_at   TIMESTAMPTZ NULL       -- NULL = never recalled
   tier               TEXT NOT NULL DEFAULT 'cold'  -- cold | warm | hot
   tier_changed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
   status             TEXT NOT NULL DEFAULT 'active' -- active | forgotten
   forgotten_reason   TEXT NULL
   created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
   updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()

   LAYER DOMAIN (3): l0 | l1 | l2 — progressive disclosure. recall returns l0 by
     default; l1/l2 only on explicit depth. The token budget belongs to the caller.
   TIER DOMAIN (3): cold | warm | hot. Transitions ONLY via the promote tool
     (explicit target tier, up or down). No background daemon, no auto-decay.
   STATUS DOMAIN (2): active | forgotten. forget is a SOFT tombstone (reason
     required). Rows are never hard-deleted (ledger discipline).

5. EVENT VOCABULARY +4 (015) — each maps to EXACTLY ONE transition:
   MemoryCreated    -- (none) -> active/cold; actor = the remembering seat
   MemoryRecalled   -- active: heat+1, recall_count+1, last_recalled_at=now
                       (projection UPDATE backed by the event, WorkItemStatusChanged
                       pattern; payload carries the heat delta)
   MemoryPromoted   -- tier change; payload carries from_tier/to_tier (up or down)
   MemoryForgotten  -- active -> forgotten; payload carries reason (soft tombstone)
   Full PG list after 015 = 25 values (the 21 live values + these 4). 015 writes the
   COMPLETE list in its guarded widening (013's pattern), so it applies cleanly from
   either the track state (15 values) or the live state (21 values).

6. REF TYPE VOCABULARY +1 (015): memory — subject of the four memory events (same
   reasoning 010 used for 'delegation', 012 for 'dispatch', 014 for 'acceptance').
   Full PG list after 015 = 12 values (the 11 live values + memory).

7. CROSS-RUST<->PG TEXT ENUM RULE (binding on both packets): every TEXT enum above is
   hand-written FromStr/Display on the Rust side (seatloom-core event_domain.rs
   pattern); NO serde derive across the Rust<->PG boundary; an unknown value is a
   typed ParseValueError, never a silent default. The Rust EventType::ALL
   exhaustiveness test must pin 25; RefType must gain Memory.

8. WRITE PATH: memories rows are written ONLY through the MCP memory tools, each
   appending its event in the SAME transaction (012 dispatch pattern). No second
   write path, no direct SQL by seats, no file-side authority. No second source of
   truth: refined products live in SeatLoom, never in a bypass file.

9. RECALL AUDIT: every recall call appends exactly one MemoryRecalled event — the
   ledger grows from "what happened" to "what is known". refs: subject = the memory,
   actor = the calling seat, context = the dispatch when recall serves a dispatch.

10. READ TOOLS (Lane B): recall / remember / promote / forget / get_dispatch /
    query_ledger — all MCP tools on seatloom-mcp-emit. Read tools mutate nothing
    except recall's audit event. Zero psql for any seat, ever again.

11. OUT OF SCOPE for D5 (do not build): embeddings / semantic retrieval (recall's
    query is a text filter over title + l0_summary via PG FTS, 002's pattern);
    automatic heat decay or auto-promotion (explicit tools only); memory content
    revision machinery (a correction = forget + remember); any external memory
    system (OpenViking/Mem0/Letta patterns may be studied, code never imported —
    SeatLoom is the base; a base inside a base is a contradiction).
```

---

## Wiring (how the two lanes connect — data, not polling)

Lane B is **blocked** on one dependency edge, expressed as data at enqueue time:

```
dispatch disp-d5-lane-b-nimbus
  dependencies: [ { dep_kind: "event", dep_value: "ev-d5-015-knowledge-schema-landed" } ]
```

Lane A's last delivery step is to emit exactly that event (spec in the Lane A packet,
§5). On insert into `canonical_events`, 013's `trg_dispatch_ready_on_event` re-checks
Lane B's dep set, flips `blocked -> ready`, and `pg_notify('dispatch_ready',
'seat-nimbus-001')` on the channel the waker already LISTENs on — Nimbus is poked,
pulls Lane B, and no seat ever polls. If the event id is ever emitted with a typo,
Lane B stays blocked and the stall is visible in the inbox view (blocked is a state,
not an error); Lyra/Aegis can then emit the correctly-named event — the mechanism
never needs a human to poll anything.

## Open decision routed to Aegis (not guessed here)

`(root)` — 13 archived sessions with no project affiliation (contract §2 R5). The
safe default is 'unmapped' + escalate; the strategy ruling (synthetic home project?
explicit exclusion set? leave permanently unmapped?) belongs to Aegis's Phase-0/2
lane. Neither lane packet may invent an answer.
