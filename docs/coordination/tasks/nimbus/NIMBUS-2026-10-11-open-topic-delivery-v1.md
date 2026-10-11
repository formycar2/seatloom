# Delivery: open_topic + the verdict-event-id contract

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-11-open-topic-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-11 |
| version | v1 |
| tags | nimbus, open-topic, topic, verdict-event-id, aegis-ruling, d5 |
| governing dispatch | `disp-open-topic-nimbus` |
| governing ruling | `docs/coordination/tasks/aegis/AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md` (on `track/infra-foundation`; not on this branch) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/open-topic` @ `29df563` |
| branch lineage | `packet/nimbus/memory-tools-018` (which carries waker-hardening -> lineage -> recall -> memory-tools) -> `packet/nimbus/open-topic` (this). Single-direction, no `dispatch.rs` divergence. |
| scratch database | `seatloom_019fresh` (001–012 + seed + 013/014 from track + 015 + 016 + 018 from Onyx's branches + scratch 017 + scratch 019; NEVER live `seatloom`) |
| proof | `crates/seatloom-mcp-emit/proof/open_topic_proof.sh` (7/7 PASS, re-runnable) |

---

## 0. Summary

Both halves of the Aegis ruling are delivered.

**Ruling 1 — `open_topic`.** Production had no path that could create a topic:
`memories.topic_id` is NOT NULL with an FK to `topics(id)`, `topics` had zero
rows, and every `INSERT INTO topics` in the repository was a test fixture — so
every `remember` failed on the foreign key, blocking Vela's S5 and every seat's
forward-recording obligation. The new tool opens a scope deliberately:

| Contract | Implementation |
|---|---|
| args | `topic_id`, `title` required; `project_id` optional (default `SEATLOOM_PROJECT_ID`, never fabricated); `parent_topic_id` optional |
| `opened_by` | resolved from `SEATLOOM_ACTOR_REF` server-side, **never a parameter** |
| idempotent | a second open of the same id succeeds, changes nothing, emits no event |
| refusals | parent missing / project missing / parent in another project / self-or-descendant cycle — typed, zero writes |
| event | row + `TopicOpened` in ONE transaction, refs subject = `topic` |

**Ruling 2 — the verdict-event-id contract.** No second NOTIFY site: the existing
`blocked -> ready` trigger already provides the wake, provided a dependency edge
points at a pre-declared verdict event id — the edge is ledger data (queryable,
visible, auditable), a NOTIFY site is implicit code behaviour. So the
verification path now MUST produce the id: `enqueue_dispatch` mints one when
`payload.kind == "verification"` and the caller omitted it, validates a
caller-supplied one, writes it INTO the payload (so it is ledger data), and
returns it in the result with the gating instruction.

---

## 1. The event-type decision the ruling asked for

The 27 PascalCase types had no `TopicOpened`. Both options, with their costs:

**Reuse an existing type.** There is no honest candidate. `WorkItemCreated` would
record a topic as a work item; `ArtifactCreated` as an artifact;
`SupervisorMessage` as an escalation. In an append-only ledger one word for two
meanings can never be split once rows carry both — the same reasoning that kept
`ArtifactProduced` out of the vocabulary and that gave `dispatch` (012),
`acceptance` (014) and `memory` (015/018) their own types. Cost of reusing: a
permanently ambiguous ledger, and every future "show me every topic opening"
query returns noise.

**Add `TopicOpened`.** Cost: one migration, one Lyra value-domain sign-off, and
the hand-written `FromStr`/`Display` the codebase already uses for every TEXT
enum. `ref_type` 'topic' already exists (010), so no ref_type widening is needed.

**Chosen: add.** `EventType::ALL` is pinned at 28, the DDL is a scratch copy
(`.scratch/019_topic_opened.sql`, NOT FOR COMMIT — 019 is the next free slot;
015/016/017/018 are taken), and **until Onyx lands it, `open_topic` refuses at the
database CHECK with the vocabulary violation** — escalated, not worked around.
The proof ran against a scratch DB with the scratch 019 applied, which is how the
Rust half is verified before Onyx's file reaches track (the 013–018 pattern).

---

## 2. The verdict-id contract, precisely

```
enqueue_dispatch(payload={"kind":"verification", ...}, verdict_event_id?)
  -> "enqueued disp-x (status=ready) | verdict_event_id=ev-y —
      the verifier MUST emit ReviewVerdictIssued with exactly this id;
      hang the gated self-dispatch on dep_kind=event, dep_value=ev-y"
```

- **Minted** when omitted; **validated** when supplied (`ev-` + a non-empty,
  whitespace-free token; a malformed id is refused, never normalised).
- **Uniqueness**: an id already pre-declared by another dispatch (it lives in the
  payload) or already present in the ledger is refused — two dispatches sharing
  one gate would let the second verdict silently resolve the first requester's
  dependency.
- **Scope**: the argument is refused for a non-verification payload — accepting it
  anywhere else would create an id nothing will ever resolve.
- An ordinary enqueue is byte-for-byte unchanged: no id in the payload, no mention
  in the result.

Not done (per the ruling): no new NOTIFY site, and no retro-fit code for Part 1's
manually-built card. The waker's periodic sweep (`e65d438`) remains the safety net
for a missed pre-declaration; it does not replace the contract.

---

## 3. Two bugs the tests caught, fixed here

**Bug A — the uniqueness check looked in the wrong place.** It queried
`canonical_events` only, so a second dispatch could pre-declare the same id: the
pre-declaration lives in the dispatch's PAYLOAD, not in the ledger, until the
verdict is emitted. Both are now checked. Caught by the DB test's second case.

**Bug B — the topic test's event queries were unscoped.** They matched any
`TopicOpened` event, so a concurrent proof run's rows made `query_one` ambiguous
(RowCount / multiple rows). Both queries are now project- and ref-scoped.

---

## 4. Validation

Commands (Rust 1.95.0; scratch DB `seatloom_019fresh` = 001–012 + seed + 013/014
from track + 015 + 016 + 018 from Onyx's branches + scratch 017 + scratch 019, in
`apply-db.sh`'s canonical order; `seatloom-cli reconcile` run first, as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (116 + 3 lineage + 6 D2 + 27 baseline + 3 seed) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (67, incl. the two new DB tests) |
| UI | `pnpm typecheck` + `pnpm test` | PASS (46 tests) |
| proof | `proof/open_topic_proof.sh` | PASS 7/7 (re-runnable) |

New / updated tests:

- `dispatch::db_itests::open_topic_creates_idempotently_and_refuses_malformed_calls`
  — the whole tool against a real database: the four refusals with
  `canonical_events` unmoved; the row + event with `opened_by` from the
  server-side identity and the subject ref naming the topic; the idempotent
  re-open (no second event); the child; the cross-project refusal; and — the
  point of the whole ruling — **a `remember` that succeeds once a topic exists**.
- `dispatch::db_itests::enqueue_mints_and_validates_the_verdict_event_id` — the
  six-case matrix: minted-into-the-payload-and-returned, taken, malformed,
  wrong-payload-kind, ordinary-untouched, well-formed-supplied.
- `db::event_domain` — `EventType::ALL` pinned at 28 with `TopicOpened` in the
  verbatim list; the thirteen new types admitted and distinct.

Proof output (abridged):

```
POINT 0  BEFORE open_topic: remember -> ERR (the topic FK)
POINT 1  OK opened topic ot-topic-root ... opened by seat-nimbus-001 ... one TopicOpened event
         opened_by: seat-nimbus-001 | subject ref: topic ot-topic-root
POINT 2  already open — unchanged, no event | TopicOpened events: still 1
POINT 3  3 typed refusals; canonical_events before=1 after=1; topics rows: 1
POINT 4  child opened; the cross-project parent refused; topics rows still 2
POINT 5  AFTER open_topic: remember -> OK; recall sees 1 memory
POINT 6  verdict_event_id=ev-... minted into the payload and returned; a duplicate,
         a malformed id and a non-verification payload all refused; an ordinary
         enqueue untouched
         tools: [... 16 tools ...]; open_topic exposes no identity parameter
```

---

## 5. Escalations and notes

1. **Onyx must land 019** (`TopicOpened` in `canonical_events_event_type_vocab`,
   27 → 28). Until then `open_topic` refuses at the CHECK with the vocabulary
   violation — the tool is deployed but not live-usable, which is exactly the
   state the codebase has been in before 013–018 landed. The DDL is ready in
   `.scratch/019_topic_opened.sql` in the house style (guarded widening carrying
   the full post-018 list — a first draft carried a stale list and would have
   reverted 015/018; caught while writing it).
2. **The gate event `ev-typed-memory-tools-landed` remains un-sent**, per the
   ruling: `remember` only becomes truly usable once `open_topic` is live AND 019
   has landed. The ruling's distinction applies — the gate's condition is
   "live-usable", not "code-landed". This delivery's proof demonstrates the
   end-to-end path on a scratch DB; the live emission waits on the real thing.
3. **A seat restart is needed** for the new tool to appear in a seat's tool list
   (the pi processes hold older MCP child images) — an ops action, flagged rather
   than performed. The stdio fallback (drive the real binary with the seat's own
   actor ref) remains available in the meantime, as it was for `request_acceptance`.
4. **Forward recording for this dispatch**: `remember` is still not callable from
   this session (the memories table is live but the seat's tool list predates the
   typed fields, and no topic is open yet). Recorded here instead of skipped:
   - *decision*: "a scope is opened deliberately, never as a side effect of
     writing a memory — `opened_by` NOT NULL is the schema saying so". Evidence:
     §1 and the proof's POINT 0/5 pair.
   - *failure*: "a pre-declared id lives in the dispatch's PAYLOAD, so a
     uniqueness check that only queries the ledger lets a second dispatch reuse
     it". Evidence: §3 Bug A.
   - *unresolved*: whether Lyra wants the pre-declaration to also be enforced on
     the verdict side (an emit-time check that the id was pre-declared) — that is
     Flux's lane and was not in this ruling.
5. No `infra/**` change, no whole-branch merge, no live-DB write.
