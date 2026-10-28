# Delivery: overnight smoke — blocked on the fleet restart, reported not faked

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-11-smoke-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-11 |
| version | v1 |
| tags | nimbus, smoke, memory-path, blocked, fleet-restart, overnight |
| governing dispatch | `disp-onight-smoke` |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/open-topic` @ `029dfbc` |
| ledger report | `SupervisorMessage` `ev-zuGAeR9st6mHjZg2mRUyN` — "smoke blocked on fleet restart" |

---

## 0. Result: the smoke did not run, and that is the deliverable

The overnight must-have was ONE real successful memory write against the live
database. It could not be performed, and the dispatch's own instruction for
exactly this case was followed: report it, do not fake a write, do not emit the
gate.

**What was checked (step 1 of the dispatch):** this seat's MCP process tool list.

```
emit_event, enqueue_dispatch, get_next_dispatch, complete_dispatch,
cancel_dispatch, request_acceptance
```

Six tools. `open_topic`, `remember`, `recall`, `resolve` and `query_ledger` are
**absent**. The round-trip `open_topic -> remember -> recall` is therefore not
callable from this session — the running MCP child image predates the delivered
code, exactly the "binary is new, seat processes are old" split the Aegis ruling
recorded on 2026-10-11.

**What was emitted:** `SupervisorMessage` `ev-zuGAeR9st6mHjZg2mRUyN` with
`message: "smoke blocked on fleet restart"`, naming the blocking condition (the
fleet restart), the code state that is ready and waiting
(`packet/nimbus/open-topic` @ `029dfbc`), and a `decision_needed_from: Aegis` so
the patrol acts on it.

**What was NOT done, deliberately:**
- no write of any kind against the live database (the smoke must be a REAL
  round-trip; a fabricated one is worse than none);
- **no gate event.** `ev-typed-memory-tools-landed` stays un-sent. The ruling is
  explicit that the gate's condition is "live-usable", not "code-landed" — and
  this session cannot demonstrate live usability, so emitting it would tell Vela
  a path exists that does not. AGENTS.md rule 5: claiming something happened that
  did not is worse than being blocked.

---

## 1. What is ready the moment the restart lands

The code is delivered and verified; only the running process is stale.

| Piece | State |
|---|---|
| `open_topic` + the verdict-id contract | `packet/nimbus/open-topic` @ `029df563`/`029dfbc`, gates green (fmt, clippy `-D warnings`, `cargo check --workspace`, core 155, mcp-emit 67, UI 46) |
| behavioural proof | `crates/seatloom-mcp-emit/proof/open_topic_proof.sh` — 7/7, re-runnable, on a scratch DB with 001–014 + 015 + 016 + 018 + scratch 017 + scratch 019 |
| the live blocker in the DB | Onyx's 019 (`TopicOpened` in `canonical_events_event_type_vocab`, 27 → 28) must also be applied — until then `open_topic` refuses at the CHECK. The DDL is ready in `.scratch/019_topic_opened.sql`. |

So the restart alone is not sufficient: **019 must be live too**. The smoke's
preconditions are (a) the fleet restart, (b) 019 applied. Both are outside this
seat's lane; both are reported.

The exact commands to run once both hold (from the dispatch, unchanged):

```
open_topic(topic_id='smoke-memory-path', title='[verify] the real memory-path round-trip', project_id='seatloom')
remember(project_id='seatloom', topic_id='smoke-memory-path',
         title='memory path verified end-to-end',
         l0_summary='open_topic + remember + recall round-trip succeeded against live on 2026-10-11',
         kind='fact', confidence='quoted', source_kind='canonical_event',
         source_ref=<this dispatch's SeatDispatchCompleted event id>)
recall(project_id='seatloom', topic_id='smoke-memory-path')   -- the memory must come back
```

then, and only then, the gate event with the exact id and payload the dispatch
specifies.

---

## 2. Escalations

1. **Fleet restart** (ops/Aegis): the six seats' pi processes hold MCP child
   images older than the delivered code. Requested via the SupervisorMessage
   above; the patrol acts on it.
2. **Onyx's 019** must be applied to live before `open_topic` can succeed (the
   `TopicOpened` event type is not yet admitted). Reported in the open_topic
   delivery doc and again here, because the smoke depends on it.
3. **No card for this lane**, per the dispatch: its evidence IS the gate event
   plus the live round-trip, and carding a verification act would be verification
   of a verification. There is therefore nothing to `request_acceptance` here.
4. Forward recording: unchanged from the previous delivery — `remember` is not
   callable from this session, so the decision/failure worth recording stay in
   the delivery docs.
