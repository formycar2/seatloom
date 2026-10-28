# Delivery: knowledge read model — topic tree + memories-by-topic

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-11-knowledge-readmodel-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-11 |
| version | v1 |
| tags | nimbus, knowledge, read-model, topic-tree, progressive-disclosure, overnight |
| governing dispatch | `disp-onight-readmodel` (overnight step 4) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/knowledge-readmodel` @ `32b0542` |
| branch lineage | `packet/nimbus/open-topic` (which carries memory-tools-018) -> `packet/nimbus/knowledge-readmodel` (this). Single-direction. |
| scratch database | `seatloom_kmfresh` (001–012 + seed + 013/014 from track + 015 + 016 + 018 from Onyx's branches + scratch 017 + scratch 019; NEVER live `seatloom`) |
| proof | `crates/seatloom-mcp-emit/proof/knowledge_readmodel_proof.sh` (6/6 PASS, re-runnable) |

---

## 0. Summary

The recall capability is now where the UI can call it. Two repository functions,
each exposed twice over ONE implementation:

| Surface | Command / tool |
|---|---|
| Tauri | `cmd_get_knowledge_tree(project_id)` |
| MCP | `get_knowledge_tree` |
| Tauri | `cmd_get_topic_memories(topic_id, depth='l0', includeSuperseded=false)` |
| MCP | `get_topic_memories` |

`tools/list` is 15 → 17. Both are **pure reads**: they write nothing and append
no event — interface queries for a panel, not a recall. `recall` keeps its
`MemoryRecalled` audit; a panel that mutated what it renders would be a lie
about who read what.

### The topic tree

Every topic with its parent and its memory count. The count is **ACTIVE and
not-superseded** — a stale decision served as current is worse than no memory,
so the number a panel shows is the number a recall would return. An empty
project is an empty tree, not an error.

### The memories of a topic

At a disclosure depth, with the topic's **ancestor chain** (ancestors first,
the topic itself last): a memory is never returned out of its context, the same
chain discipline `recall` follows. `depth` l0 (default) returns only the
~100-token summaries; l1 adds the overview; l2 adds the evidence pointer with
the line range. The token budget belongs to the caller — progressive disclosure
is token control, not an optimisation.

Superseded rows are excluded unless `include_superseded` is set, and then each
carries `supersededBy` (which memory replaced it), so the caller can judge for
itself. A **forgotten row IS returned** with its status: the panel shows a
card's status (active | forgotten | resolved), so a tombstone must be visible,
not silently dropped.

### Scope

Hard: project + topic, never an all-projects read. The memories query is scoped
by the **topic's** project, so a row whose `project_id` disagreed with its
topic's could not leak across projects.

### The fence

Held exactly as the dispatch specified (identical text in Mira's): `crates/**`,
`src-tauri/src/commands/knowledge_cmds.rs` (new), `src-tauri/src/dto.rs` and
`main.rs` additive only, `ui/src/lib/types-dto.ts` and `api.ts` additive only.
`ui/src/app-v2/**` untouched — that is Mira's lane, and the DTO contract she
codes against is now landed.

---

## 1. Two bugs the proof caught, fixed here

**Bug A — the memories read was scoped by `topic_id` alone.** Topic ids are
globally unique, but a memory's `project_id` is not cross-checked against its
topic's, so memories of a same-named topic in another project came back too.
The proof's fixture collision made it visible immediately. Now the query is
scoped by `(SELECT project_id FROM topics WHERE id = $1)`.

**Bug B — the chain order.** `topic_ancestor_chain` returns nearest-first
(`recall`'s order), while the knowledge contract says ancestors-first. Added
`topic_chain_ancestors_first` and used it in both surfaces, so the two cannot
disagree.

---

## 2. Validation

Commands (Rust 1.95.0; scratch DB `seatloom_kmfresh` = 001–012 + seed + 013/014
from track + 015 + 016 + 018 from Onyx's branches + scratch 017 + scratch 019,
in `apply-db.sh`'s canonical order; `seatloom-cli reconcile` run first, as CI
does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (116 + 3 lineage + 6 D2 + 27 baseline + 3 seed + 3 knowledge) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (67) |
| UI | `pnpm typecheck` + `pnpm test` | PASS (46 tests) |
| proof | `proof/knowledge_readmodel_proof.sh` | PASS 6/6 (re-runnable) |

New tests:

- `knowledge_readmodel_integration.rs` — the tree counts only live memories;
  l0/l1/l2 disclose exactly their layer (and a memory with no overview stays
  without one — absence is data, not a default); the superseded row is hidden by
  default and annotated when asked for; the forgotten row is returned with its
  status; the reads move no ledger row.

Proof output (abridged):

```
POINT 1  kmproof-root | parent: None | memoryCount: 0
         kmproof-child | parent: kmproof-root | memoryCount: 2
         the count matches what a recall would return: PASS
POINT 2  depth: l0 | topicChain: [(kmproof-root,...), (kmproof-child,...)]
         l0 discloses only l0 + the ancestor chain: PASS
POINT 3  l1 discloses the overview and nothing more: PASS
         l2 discloses the pointer with the line range: PASS
POINT 4  the superseded row, when asked for: ... supersededBy: kmproof-new
         opt-in annotates supersededBy: PASS
POINT 5  canonical_events: before=34 after=34 | pure read: PASS
POINT 6  topic "no-such-topic" not found / depth "l3" is not one of l0|l1|l2 /
         limit 9999 exceeds the hard ceiling 200 / an empty project is an empty tree
         tools: [... 18 tools ...]; both knowledge reads advertised, no identity parameter
```

---

## 3. Escalations and notes

1. **Gate event `ev-knowledge-read-model-landed`** emitted on delivery (the
   exact id and payload the dispatch specifies) so the UI lane can wire against
   the landed surface.
2. **The verdict-id contract for the Flux self-request**: this delivery's
   verification dispatch pre-declares `ev-verdict-knowledge-readmodel` (the id
   Lyra holds a gated self-dispatch on), which is what makes the carding step
   arrive without a human push.
3. **`open_topic` is still not live-usable** (Onyx's 019 pending), so a panel
   showing an empty tree is the expected live state until 019 lands and a seat
   opens a topic — the reads themselves are correct and empty, which is what the
   proof's "empty project is an empty tree" point pins.
4. Forward recording: unchanged — `remember` is not callable from this session,
   so the decision/failure worth recording stay in the delivery docs. The two
   bugs above are the ones worth carrying forward.
5. No `infra/**` change, no whole-branch merge, no live-DB write.
