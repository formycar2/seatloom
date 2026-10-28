# Delivery: D5 Lane B — MCP memory tools + the two pure reads (no seat needs psql again)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-d5-mcp-recall-tools-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, d5, knowledge-layer, memory, recall, read-tools, mcp |
| governing dispatch | `disp-d5-lane-b-nimbus` (Lane B, D5 knowledge layer) |
| governing packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-d5-mcp-recall-tools-v1.md` (on `packet/lyra/milestone-summary`; not on this branch) |
| value-domain contract | `VALUE-DOMAIN-2026-10-10-knowledge-layer-v1` (Lyra sign-off 2026-10-10), carried verbatim in the packet §2 |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/d5-mcp-recall` |
| branch lineage | `packet/nimbus/waker-hardening` (D2 defect fixes) -> `packet/nimbus/acceptance-lineage-readmodel` (Lane B lineage + plan enforcement) -> `packet/nimbus/d5-mcp-recall` (this). One chain, no divergence on any shared file. |
| scratch database | `seatloom_d5gate` (001–012 + seed + 013/014 from track + 015 + 016 from Onyx's branches + scratch 017; NEVER live `seatloom`) |

---

## 0. Summary

SeatLoom's MCP surface had ZERO read tools, which is why seats kept reaching for
`psql`. D5 Lane B adds six tools to `seatloom-mcp-emit` and closes that hole:

| Tool | Kind | What it does |
|---|---|---|
| `recall` | read + audit | progressive-disclosure retrieval (depth l0 default, l1/l2 on explicit request), returning the topic's ANCESTOR CHAIN so a memory is never read out of context; appends exactly one `MemoryRecalled` per returned row |
| `remember` | write | creates a memory + `MemoryCreated` in ONE transaction; tier starts `cold`, status starts `active` (neither caller-settable) |
| `promote` | write | the ONLY tier channel — explicit target tier, up OR down; `MemoryPromoted` carries from_tier/to_tier |
| `forget` | write | the soft tombstone: `active -> forgotten` + the reason, row never hard-deleted |
| `get_dispatch` | pure read | the dispatch row + its LIVE status (recomputed by the same existence rule the pull uses) + each dependency edge with its referent's existence |
| `query_ledger` | pure read | filtered `canonical_events` with their object refs (project / types / ref / time window / paging) |

Memory management is EXPLICIT TOOLS, never a background daemon: there is no
automatic heat decay and no auto-promotion anywhere in this delivery. `heat`
moves only when a recall happens.

---

## 1. What changed

| File | Change |
|---|---|
| `crates/seatloom-core/src/objects/memory.rs` | **NEW** — the four closed domains of a memory (`MemoryLayer` l0/l1/l2, `MemoryTier` cold/warm/hot, `L2RefKind` document/file, `MemorySourceKind`), each hand-written `FromStr`/`Display` with NO serde derive across the Rust<->PG boundary; unknown value = typed `ParseValueError` |
| `crates/seatloom-core/src/db/event_domain.rs` | `EventType` +4 (`MemoryCreated`/`MemoryRecalled`/`MemoryPromoted`/`MemoryForgotten`), `RefType` +`Memory`; `EventType::ALL` pinned at 26; the pinned-set test rewritten with the count explained |
| `crates/seatloom-core/src/db/models.rs` | `MemoryRow`, `TopicRow`, `DispatchReadModel`, `DispatchDependencyStatus`, `LedgerEventWithRefs` |
| `crates/seatloom-core/src/db/repositories.rs` | memories: `get_memory`, `list_memories_for_recall` (FTS over title + l0_summary, 002's pattern), `topic_ancestor_chain` (recursive CTE over `topics.parent_topic_id`), and the three write paths (`insert_memory` / `record_recall` / `set_memory_tier` / `forget_memory`), each appending its event in the SAME transaction via `append_event_in_tx`; reads: `dispatch_read_model`, `query_ledger`; `DbError` +3 typed variants |
| `crates/seatloom-mcp-emit/src/memory.rs` | **NEW** — `recall`/`remember`/`promote`/`forget` with their argument shapes, the typed `MemoryRejection`, the strict L2 pairing rule, and the depth/tier/source-kind parsing |
| `crates/seatloom-mcp-emit/src/read.rs` | **NEW** — `get_dispatch` / `query_ledger` with `ReadRejection` |
| `crates/seatloom-mcp-emit/src/protocol.rs` | six tool schemas; `tools/list` 8 -> 14 |
| `crates/seatloom-mcp-emit/src/main.rs` | tools wired; the argument-parsing/rendering helpers |
| `crates/seatloom-mcp-emit/proof/d5_memory_recall_proof.sh` | **NEW** — the 8-point behavioural proof |
| `.scratch/017_acceptance_cancel.sql` | fixed a REAL bug found by this round's scratch proof (see §3) |

### Notable implementation decisions

- **The topic ancestor chain rides with every recall.** A memory read without
  its topic's ancestry is reading it out of context — the failure mode the
  contract names. The
  chain is a bounded recursive CTE (depth < 32; 010's self-FK keeps `topics` a
  tree).
- **The recall audit is per row, in the row's own transaction.** A recall that
  cannot be recorded is not a recall; zero hits append zero events.
- **`query_ledger`'s empty type list means "no filter".** An empty array passed
  as a parameter would make `event_type = ANY('{}')` exclude every row — the
  parameter is NULL when the caller named no types.
- **The recall read is scope-first.** `project_id` + `topic_id` are both part of
  the scope; there is no all-projects recall to invent.

---

## 2. Validation

Commands run (Rust 1.95.0; scratch DB `seatloom_d5gate` with 001–014 + 015 + 016
+ scratch 017 in `apply-db.sh`'s canonical order; `seatloom-cli reconcile` run
first, as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (114 + 3 lineage + 6 D2 + 27 baseline + 3 seed) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (62, incl. 2 new DB tests) |
| UI | `pnpm typecheck` + `pnpm test` | PASS (10) |
| proofs | `d5_memory_recall_proof.sh`, `acceptance_lineage_proof.sh` | PASS (both re-runnable) |

New / updated tests:

- `dispatch::db_itests::memory_tools_lifecycle_and_refusals_against_the_db` —
  the whole lifecycle through the TOOLS against a real database: remember (row +
  event + the subject ref naming the memory), recall at l0/l1/l2 (each layer
  disclosed only on request, the topic chain present, heat/recall_count/
  last_recalled_at moving with exactly one event), promote up and down with the
  payload pairs, forget (soft tombstone, row present, second forget refused,
  excluded from recall), and the negative matrix (unknown depth, oversized limit,
  both L2 pairing violations, unknown source_kind/tier, unknown memory, blank
  reason) each a typed refusal with the event count unmoved.
- `dispatch::db_itests::read_tools_answer_from_the_ledger` — `get_dispatch`
  (stored vs live status, unknown id refused) and `query_ledger` (type filter,
  half a ref filter refused, unknown event type refused).
- `objects::memory::tests` — every domain round-trips and rejects unknowns, with
  the contract's values pinned verbatim.
- `memory::tests` / `read::tests` — the l0 shape omits the higher layers, the
  ceilings are hard refusals, the refusals name the rule broken.

Proof output (abridged):

```
POINT 1  OK remembered memory mem-... (tier cold, status active, heat 0)
         MemoryCreated events: 1 | subject ref: memory mem-... role=subject
POINT 2  depth: l0 | topic chain: ['d5-proof-topic-child', 'd5-proof-topic-parent']
         memory fields: heat, id, l0_summary, recall_count, tier, title
         row after recall: heat=1 recall_count=1 last_recalled=t | MemoryRecalled: 1
POINT 3  l1 fields: + l1_overview | l2 fields: + l2_body_digest, l2_file_path, l2_ref_kind
POINT 4  12 typed refusals; canonical_events before=4 after=4; memories rows: 1
POINT 5  cold->warm, warm->hot, hot->cold (DOWN allowed); same tier refused
POINT 6  forgotten / superseded by a newer note; row still present; 2nd forget refused
         recall after forget: 0 memories, 0 recalled
POINT 7  dispatch: d5 proof task | stored ready | live blocked
         dependencies: [('event', False)]
         events naming the memory: MemoryForgotten, MemoryPromoted x3, MemoryRecalled x3, MemoryCreated
POINT 8  tools: [... 14 tools]; no seat/actor parameter on any of the six
```

---

## 3. A real bug found and fixed in the scratch migration

`.scratch/017_acceptance_cancel.sql` (the D2 withdrawal widening) was drafted
BEFORE Onyx's 015 landed, so its guarded block re-installed the event-type
vocabulary from a STALE list — applying 015 then 017 silently REVERTED 015's
four memory types out of `canonical_events_event_type_vocab`. The scratch proof
caught it (`MemoryCreated` inserts failing on the CHECK).

Fixed: 017 now re-installs the POST-015 list (the memory types carried verbatim)
and its header states the rule — a guarded widening DROPs and re-ADDs the whole
CHECK, so the list it installs must be the current one. This is exactly the
class of bug the house style's guarded widening exists to prevent, and it is
recorded here rather than quietly patched.

## 4. Escalations and decisions (not worked around)

1. **`EventType::ALL` is pinned at 26, not the contract's 25.** The contract
   counts from the LIVE database (21 values + 4). This branch chain also carries
   `AcceptanceCancelled` (the D2 withdrawal, whose migration 017 has not landed
   yet), so the honest Rust pin is 22 + 4 = 26, and the database reaches 26 when
   017 lands alongside 015. Declared in the pinned-set test's doc comment rather
   than silently reconciled.
2. **`tools/list` is 14, not the packet's "6 -> 12".** The packet counted the
   pre-Lane-B surface as 6 (emit_event + 5 dispatch tools). This branch chain
   also carries `request_acceptance`, `cancel_acceptance` and
   `get_acceptance_lineage` from the two completed dispatches, so 8 + 6 = 14.
3. **Migration 015/016 are Onyx's and are not on this branch.** They were applied
   to the scratch DB from `packet/onyx/d5-knowledge-schema` and
   `packet/onyx/acceptance-plan-schema` for verification. `infra/**` untouched.
4. **The `memories` rows are written ONLY through these tools.** No second write
   path, no direct SQL by seats, no file-side authority — the row is a projection
   of the ledger like `dispatches` and `acceptances`.
5. **Out of scope, as the contract demands:** embeddings / semantic retrieval
   (recall's query is a text filter over title + l0_summary), automatic heat
   decay or auto-promotion, memory content-revision machinery (a correction =
   forget + remember), and any external memory system (patterns only, never
   code).
6. **Pre-existing docs-gate debt on this branch** (unchanged): D2-era docs not in
   `.gitlab/docs-grandfather.txt`. This delivery's doc conforms.

---

## 5. Closeout

1. Committed to `packet/nimbus/d5-mcp-recall` (commit-pinned).
2. This delivery doc.
3. Self-requested Flux verification (commit-pinned branch/SHA, base
   `track/infra-foundation`) — see the enqueued dispatch.
4. On Flux green: `request_acceptance` with the packet §7 one-step verification
   carried verbatim (Chinese, as Lyra authored it) and a shape-v1 acceptance
   plan assembled from the packet's own §5 acceptance matrix — the plan is
   PO-authored content, not seat-authored meaning.
5. `complete_dispatch`.

The seat never merges its own work: the request is handed to the operator and
the merge happens only on an explicit accept.
