# Delivery: P0 step 4 — MCP emit server (`emit_event`)

[Nimbus -> Lyra / Flux] Delivery for `NIMBUS-2026-10-08-p0-step4-mcp-emit-v1`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-08-p0-step4-mcp-emit-delivery-v1 |
| author | nimbus |
| date | 2026-10-08 |
| version | v1 |
| responds to | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1.md` |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step4-mcp-emit-v1.md` |
| depends_on | migration `010`; write API A1 (`NIMBUS-2026-09-30-p0-write-api-v1`, merged) |
| milestone | P0 capture loop — events flow (end-state) |
| base branch | `track/infra-foundation` |
| base commit | `c495ffa6c806fa06df9aea6cdd898300a2c0d4ca` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1` |
| delivery commit | `af3a0f71cb55f7ba6576783a013048a01e2c38db` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` |
| scratch database | `seatloom_mcp_nimbus` (NEVER the live `seatloom`) |
| status | delivered — awaiting Flux commit-pinned verify on real Postgres |

---

## 1. Value-domain contract (verbatim — packet §1)

```
EVENT TYPE (12, PascalCase — matches what is already in PostgreSQL):
  SessionStarted | WorkItemCreated | WorkItemStatusChanged | ArtifactCreated
  | HandoffSent | HandoffAccepted | HandoffCompleted | ReviewVerdictIssued
  | SeatDelegationIssued | SeatDelegationClosed | SupervisorMessage | SeatResponse

REF ROLE (3, snake_case):
  subject | actor | context
  subject = what the event is ABOUT
  actor   = who did it
  context = related, WITHOUT asserting agency or aboutness (weakest claim)

REF TYPE (9, snake_case):
  workitem | session | handoff | artifact | document | seat | topic | project
  | delegation        <-- 'delegation' is a DEVIATION from the original packet
                          contract, accepted: it exists in committed seed data
                          (ev-020/ev-021). Do not drop it and do not re-add
                          'ArtifactProduced' (dropped on Lyra sign-off; capture
                          reuses the existing 'ArtifactCreated').

RULES:
  * schema_version is set EXPLICITLY, per event_type, >= 1. Never rely on
    010's DEFAULT 1.
  * role is stated EXPLICITLY on every ref. Never rely on 010's DEFAULT
    'context'.
  * EXACTLY ONE ref with role='subject' per event. 010 enforces AT MOST ONE
    (partial unique index); presence is the write API's job, and the MCP layer
    must not weaken it.
  * turn_seq is NULL = NOT RECORDED. Never 0. T2 capture is P1, so P0 emits
    pass None.
  * payload is REQUIRED and must be a JSON object. Which (type, version) pairs
    have a typed payload contract is the write API's contract, not a column
    constraint.
```

This block is carried verbatim in the packet, in the crate (`emit.rs` module
doc **and** the `emit_event` tool `description` in `protocol.rs`), and here.

## 2. What was built

A **new workspace member**, `crates/seatloom-mcp-emit`, binary
`seatloom-mcp-emit`. Separate from `src-tauri` and `seatloom-core`; depends on
`seatloom-core` for the write API rather than re-implementing it.

- **MCP over stdio, JSON-RPC 2.0, newline-delimited.** Handles `initialize`,
  `notifications/initialized` (and any `notifications/*`, no response),
  `tools/list`, `tools/call`, `ping`. Logs to stderr only; stdout is the
  protocol channel.
- **One tool, `emit_event`.** Params (JSON Schema in `tools/list`): `type`,
  `payload`, `refs[{ref_type, ref_id, role}]`, optional `schema_version`
  (default 1), `actor_ref`, `project_id`, `event_id`, `occurred_at`, `turn_seq`.
- **Routes through the write API.** `emit_event` builds an `EventDraft` and calls
  `SeatloomDb::append_event`. No projection mutation, no direct
  `canonical_events` insert, no bypass of `validate()`. Pool created once at
  startup (`create_pool()`).
- **Rejects, never crashes.** A malformed call returns an MCP tool error
  (`isError: true`) carrying the typed `WriteError`/parse message verbatim. A
  rejection never writes a partial event — that is `append_event`'s single
  transaction, unchanged.

### 2.1 Resolution rules enforced in the MCP layer (all typed, none fabricated)

| concern | rule |
|---|---|
| `schema_version` | optional, default **1**, must be `>= 1` and fit `i32`; `0`/negative rejected |
| `turn_seq` | P0 omits it; if sent, `>= 0` and fits `i32`; **0 is a recorded ordinal, never a placeholder**; absent = `None` = not recorded |
| `actor_ref` | call value → `SEATLOOM_ACTOR_REF` → **typed rejection** (never invented) |
| `project_id` | call value → `SEATLOOM_PROJECT_ID` → **typed rejection** (never fabricated) |
| `event_id` | generated (`nanoid`, `ev-…`) when absent |
| `occurred_at` | RFC 3339 when supplied, else `Utc::now()` |
| vocabulary | `type`/`ref_type`/`role` parsed via A1's hand-written `FromStr`; unknown value named in the error |

### Changed files (exact — `git show --stat af3a0f7`)

```
 Cargo.lock                                       |  12 +
 Cargo.toml                                       |   1 +   (workspace member added)
 crates/seatloom-mcp-emit/Cargo.toml              |  26 ++
 crates/seatloom-mcp-emit/proof/mcp_emit_proof.sh | 112 ++  (behavioural proof driver)
 crates/seatloom-mcp-emit/src/emit.rs             | 461 ++  (tool logic + 11 unit tests)
 crates/seatloom-mcp-emit/src/main.rs             | 160 ++  (stdio wiring + 3 unit tests)
 crates/seatloom-mcp-emit/src/protocol.rs         | 283 ++  (JSON-RPC/MCP framing + 5 unit tests)
 7 files changed, 1055 insertions(+)
```

Zero `.tsx`, zero `.sql`, **no changes to `crates/seatloom-core/src/db/write_api.rs`
or `event_domain.rs`** (A1's accepted files — read, not edited; confirmed with
`git diff --name-only`).

## 3. Crate-availability decision (packet §2.7)

- **An MCP crate DOES resolve through the internal Artifactory.** `cargo add rmcp`
  resolved `rmcp 3.5.1` (the official Rust MCP SDK) via the configured
  `sparse+https://artifactory.stepfun-inc.com/...` index, locking 80 packages —
  no overseas-registry fetch. So the §2.7 crate path was available.
- **I chose the hand-rolled JSON-RPC 2.0 stdio path anyway** (the sanctioned
  fallback). Rationale: a one-tool server that routes to an already-proven write
  API is a small, well-specified surface; hand-rolling it keeps the dependency
  footprint at 4 crates (serde/serde_json/chrono/nanoid/tokio) instead of 80,
  keeps the protocol auditable for Flux, and avoids coupling an append-only
  capture substrate to a fast-moving macro-driven framework API. The framing
  handled (`initialize`/`tools/list`/`tools/call`/`notifications/*`/`ping`, the
  `content` array shape, `isError`) is exactly the specified surface.
- No direct overseas registry fetch; the 2026-09-29 rule holds.

## 4. Verification — Layer A (verbatim)

```
cargo check --workspace            -> Finished (ok)
cargo test --workspace             -> seatloom-core 104 passed; seatloom-mcp-emit 19 passed;
                                      postgres_seed_consistency 3 passed; others 0; all ok
cargo clippy --workspace --all-targets -- -D warnings  -> Finished (no warnings)
cargo fmt --all --check            -> clean
```

The 19 `seatloom-mcp-emit` unit tests cover: draft defaulting (schema_version→1,
turn_seq→None, id generation), honouring explicit `event_id`/`occurred_at`,
every vocabulary rejection by name, `schema_version 0`, `turn_seq` domain
(negative rejected / 0 allowed / absent = None), missing-actor and
missing-project typed rejections (env fallback explicitly cleared in-test),
blank-value fall-through, bad `occurred_at`, JSON-RPC classification
(request/notification/malformed), `tools/list` advertising exactly one tool,
the schema enumerating all 12 event types + 9 ref types (and NOT
`ArtifactProduced`), the contract carried in the description, and the `isError`
flag.

## 5. Verification — behavioural proof on the scratch database

Scratch DB `seatloom_mcp_nimbus` built by applying migrations `001–010` in order
(podman `seatloom-postgres`, Postgres 16.14), fixtures `project p0-mcp-proof` +
`seat seat-nimbus-001`. Driver: `crates/seatloom-mcp-emit/proof/mcp_emit_proof.sh`
pipes real JSON-RPC lines through the built binary and checks row counts via
`psql` before/after each case. Full verbatim run (abridged to results):

```
tools/list              : tool count = 1 | name = emit_event

PROOF 1 happy path (ArtifactCreated):
  isError=False  emitted ArtifactCreated event ev-mcp-artifact-1 -> LedgerOnly
  canonical_events for p0-mcp-proof: before=0 after=1
  ev-mcp-artifact-1 | ArtifactCreated | sv=1 | turn=NULL
    ref seat     seat-nimbus-001 role=actor
    ref workitem wi-mcp-001      role=subject

PROOF 1b WorkItemCreated drives the projection in the SAME txn:
  isError=False  -> WorkItemInserted { workitem_id: "wi-mcp-proj-1" }
  workitems row: wi-mcp-proj-1 status=in_progress priority=high

PROOF 2 rejections — each typed error, each writes NOTHING (events before==after):
  unknown type         ERR type "Nope" is not one of the 12 event types (...)
  zero subjects        ERR carries no subject ref: ... found 0 of 1 ref(s)
  two subjects         ERR carries 2 subject refs: ... (010:332 ...)
  schema_version 0     ERR schema_version 0 must be >= 1
  no-contract version  ERR no (event_type, schema_version) payload contract for (WorkItemCreated, v2)
  payload not object   ERR payload must be a JSON object, found array
  wrong subject type   ERR subject ref must be ref_type=workitem for event_type=WorkItemCreated, found session
  missing actor_ref    ERR actor_ref is required (... never invented)

PROOF 3 atomicity — forced FK failure mid-append (owner_seat_id → nonexistent seat):
  isError=True   postgres error: db error
  after the failed append: events=0 refs=0 workitems=0   (all 0)
  project event count: before=2 after=2
  HELD: rollback is atomic, not partial.

ALL PROOFS HELD
```

Every §4 requirement is exercised: happy path lands the event **via the write
API** with its refs (roles shown) and, for `WorkItemCreated`, its projection row
in the same transaction; every malformed call fails with a typed error and
writes nothing (proven by equal before/after counts); a forced mid-append
failure rolls back event **and** refs **and** projection together.

## 6. `pi mcp list` evidence (packet §4.4 / §2.6)

I did **not** edit any seat's live `~/.pi/agent/mcp.json`. I registered the
server under an **isolated throwaway `HOME`** and captured:

```
$ pi mcp add seatloom-emit --env DATABASE_URL=postgresql://…/seatloom_mcp_nimbus \
    --env SEATLOOM_ACTOR_REF=seat:seat-nimbus-001 \
    --env SEATLOOM_PROJECT_ID=p0-mcp-proof \
    -- /abs/path/to/target/debug/seatloom-mcp-emit

$ pi mcp list
seatloom-emit: connected, 1 tool (codemode, global)
  /…/target/debug/seatloom-mcp-emit
  tools: emit_event
```

`connected, 1 tool` with `emit_event` listed is the evidence a seat can call it.
The throwaway HOME was removed after capture; no `.pi/` is committed.

**Rollout command for Lyra** (release binary path, real per-seat env — do not
point at the live `seatloom` DB during any test, and set `SEATLOOM_ACTOR_REF`
to the seat's own id):

```
pi mcp add seatloom-emit \
  --env DATABASE_URL='postgresql://seatloom:seatloom@localhost:5432/<target-db>' \
  --env SEATLOOM_ACTOR_REF='seat:<seat-id>' \
  --env SEATLOOM_PROJECT_ID='<project>' \
  -- /abs/path/to/seatloom-mcp-emit
```

If a seat reports the tool missing: `pi --no-mcp` disables MCP entirely, and
`-t/--exclude-tools` can hide `mcp__*` tools — those are the first two causes.

## 7. Findings (reported, not worked around — packet §6)

**F1 — "no-contract (type, version) pair" is only enforceable for the 2 event
types A1 has payload contracts for.** A1's `append_event` raises
`UnsupportedPayloadVersion` for `(WorkItemCreated|WorkItemStatusChanged, v≠1)`,
but for the other **10** event types it returns `ProjectionEffect::LedgerOnly`
for **any** `schema_version ≥ 1` — by design: those types have no projection and
no typed payload contract yet, so the event is recorded ledger-only. Concretely,
`emit_event(ArtifactCreated, schema_version:7, …)` is **accepted** as ledger-only,
not rejected.

- This is consistent with A1; the packet §4.2 "no-contract pair must be rejected"
  is satisfied for the types A1 governs (proven above with `(WorkItemCreated, 2)`).
- I deliberately did **not** add a stricter version gate in the MCP layer, because
  that would fork validity from A1 — a second opinion about what is writable,
  exactly the divergence §3 forbids. If the end-state wants per-type
  `schema_version` allowlists for the ledger-only types, that is an **A1 change**
  to sequence (e.g. a registry of known `(type, version)` pairs in `write_api`),
  not something to fold into this packet. Flagged for Lyra.

**F2 — two event vocabularies still exist** (`db::event_domain::EventType`, 12,
PostgreSQL; `ledger::event::EventType`, 27, JSONL). This server uses the
PostgreSQL vocabulary only, as required. Unifying them remains out of scope
(already noted in A1's delivery).

**F3 — no A1 change was needed** to make `emit_event` expressible; `EventDraft` +
`append_event` carried every case. No STOP was required.

## 8. Done-definition check

1. A seat can call `emit_event(type, payload, refs)` through pi's MCP and it lands
   via the write API in one transaction, projection included, retrievable —
   proved (§5, §6). ✅
2. Every malformed event rejected with a typed error, writes nothing — proved
   (§5 PROOF 2, before==after counts). ✅
3. `schema_version` and `role` always explicit; exactly one subject — enforced in
   the MCP layer + A1; proved. ✅
4. Value-domain contract carried verbatim in tool description, packet, delivery. ✅
5. Layer A green, verbatim (§4). ✅
6. Never ran against the live DB; scratch `seatloom_mcp_nimbus` named. ✅
7. Zero `.tsx`, zero `.sql`, no A1/enum files edited, no live seat disturbed. ✅
8. Delivery artifact at the header path with the exact delivery commit + evidence —
   this file. ✅  (Flux commit-pinned verify on real Postgres: pending.)

---

*Delivered by Nimbus · 2026-10-08 · base `c495ffa` → delivery `af3a0f7` on
`packet/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1` · hand-rolled JSON-RPC 2.0
stdio (rmcp 3.5.1 available internally, declined for surface size) · proof on
scratch DB `seatloom_mcp_nimbus`, live `seatloom` never touched · one reported
contract gap F1 for Lyra, not folded in.*
