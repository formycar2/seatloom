# Task: P0 step 4 — MCP emit server (`emit_event`), the sanctioned seat write path

[Lyra -> Nimbus] Aegis dispatch `AEGIS-2026-10-08-p0-step4-mcp-emit-v1`: build the capture substrate the whole end-state rides on. This is the **only sanctioned way a seat writes a governed event**. Route: you (PI/opus), your worktree. On delivery Lyra dispatches Flux to verify on real Postgres; on PASS Lyra merges.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-10-08-p0-step4-mcp-emit-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| milestone | P0 capture loop — events flow (end-state) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step4-mcp-emit-v1.md` |
| depends_on | migration `010_p0_schema_foundation.sql` (vocab / role / schema_version); write API A1 `NIMBUS-2026-09-30-p0-write-api-v1` (merged) |
| base branch | `track/infra-foundation` @ `c495ffa` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` (currently on `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` — create the new branch from track) |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-delivery-v1.md` |
| tags | nimbus, p0, mcp, emit, capture, step4, end-state |

---

## 0. What already exists — reuse, do not rebuild

I read all of this on `track/infra-foundation` before dispatching, so you do not have to re-derive it:

**The write API (A1, step 2 — merged and accepted).** `crates/seatloom-core/src/db/write_api.rs`:

- `SeatloomDb::append_event(&EventDraft) -> Result<AppendOutcome, WriteError>` is **the** primitive. It calls `EventDraft::validate()` **before** taking a connection, then in **one transaction** inserts `canonical_events`, inserts every `event_object_refs` row with `role` named explicitly (no `ON CONFLICT DO NOTHING` — a duplicate must fail loudly), applies the projection, and commits.
- `EventDraft` fields: `event_id`, `event_type`, `schema_version`, `occurred_at`, `actor_ref`, `project_id`, `turn_seq: Option<i32>`, `payload: serde_json::Value` (**required, not Option**), `refs: Vec<ObjectRefDraft>`.
- `ObjectRefDraft::new(ref_type, ref_id, role)` — role is mandatory by type; `ObjectRefDraft::subject(ref_type, ref_id)` is the convenience constructor.
- `validate()` already enforces: non-empty `event_id`/`actor_ref`/`project_id`/`ref_id`; `schema_version >= 1`; `turn_seq >= 0` when present; `payload` must be a JSON **object**; **exactly one** `role='subject'` ref (0 → `NoSubject`, ≥2 → `MultipleSubjects`); `WrongSubjectType` for a subject ref_type that does not match the event_type; typed payload contracts per `(event_type, schema_version)` (`WorkItemCreatedV1`, `WorkItemStatusChangedV1`, `UnsupportedPayloadVersion` otherwise).
- Errors are typed: `WriteError::{NoSubject, MultipleSubjects, SchemaVersionOutOfRange, TurnSeqNegative, PayloadNotAnObject, EmptyField, WrongSubjectType, PayloadShape, UnsupportedPayloadVersion, ProjectionRowMissing, UnreplayableHistory, Db, Postgres}`. All render a message via `thiserror`.
- Pool: `seatloom_core::db::{create_pool, SeatloomDb}`; `DATABASE_URL` env or `DEFAULT_DATABASE_URL` (`crates/seatloom-core/src/db/connection.rs`).
- Reference harness to copy the discipline from: `crates/seatloom-core/examples/p0_write_api_proof.rs` — it refuses to run against a database named `seatloom`, and proves atomicity both ways plus zero-subject rejection.

**The vocabulary.** `crates/seatloom-core/src/db/event_domain.rs` — `EventType` (12), `RefRole` (3), `RefType` (9), each with `ALL`, `as_str()`, hand-written `FromStr`/`Display` and `TryFrom<&str>` at the Rust⇄PG TEXT boundary, per ENGINEERING_STANDARDS. Exhaustiveness tests pin the enums against migration `010`.

**The schema.** `infra/postgres/schema/010_p0_schema_foundation.sql` — `canonical_events.schema_version` (`NOT NULL`, `>= 1`, `DEFAULT 1` which you must **not** rely on), the closed `event_type` vocabulary CHECK, `event_object_refs.role` (`NOT NULL`, `DEFAULT 'context'` which you must **not** rely on), `role IN ('subject','actor','context')`, `ref_type IN (…9 values…)`, PK `(event_id, ref_type, ref_id, role)`, and the **partial unique index** `uq_event_object_refs_one_subject_per_event` — at most one subject per event.

## 1. Value-domain contract — carry this VERBATIM into the packet, the tool description, and the delivery

This is the same treatment the runtime-triple packet got: one contract, quoted identically in every artifact, so no seat re-derives it.

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

## 2. Scope

### 2.1 The server

1. A **new workspace member** — suggested `crates/seatloom-mcp-emit` with a binary named `seatloom-mcp-emit`. Your call on layout; the requirement is that it is a **separate binary** that does not bloat `src-tauri` or `seatloom-core`, and that it depends on `seatloom-core` for the write API rather than re-implementing it.
2. **MCP over stdio** (JSON-RPC 2.0), enough of the protocol for pi to connect and call one tool:
   - `initialize` / `notifications/initialized`
   - `tools/list` — advertise exactly one tool, `emit_event`
   - `tools/call` — invoke `emit_event`
   - `ping` if cheap
   - Respond to `notifications/*` without emitting a response.
3. **One tool: `emit_event(type, payload, refs)`.** Parameters (JSON Schema in `tools/list`):
   - `type` — string, must parse to `EventType` (12 values).
   - `payload` — object, must match the typed contract for `(type, schema_version)`.
   - `refs` — array of `{ ref_type, ref_id, role }`; `ref_type` must parse to `RefType`, `role` to `RefRole`; **exactly one** entry with `role: "subject"`.
   - `schema_version` — integer, optional, **default 1**, must be `>= 1`. Reject `< 1` and reject a `(type, version)` pair with no contract.
   - `actor_ref` — string, **required unless `SEATLOOM_ACTOR_REF` is set in the server's env**. Never silently invent one; an empty/absent actor is a typed rejection, not a default.
   - `project_id` — string, optional, defaults to `SEATLOOM_PROJECT_ID` env, else the A1 default project. Same rule: never fabricate.
   - `event_id` — optional; when absent, generate (nanoid is already a dependency).
   - `occurred_at` — optional RFC 3339; when absent, `Utc::now()`.
   - `turn_seq` — optional integer `>= 0`; **omit it in P0** rather than sending `0`.
4. **Route through the write API.** `emit_event` builds an `EventDraft` and calls `SeatloomDb::append_event`. **Never** mutate a projection directly, never insert into `canonical_events` yourself, never bypass `validate()`. Reuse `create_pool()` once at startup.
5. **Reject, do not crash.** A malformed call returns an MCP tool error (`isError: true`) carrying the typed `WriteError` / parse-error message verbatim. A rejection must never write a partial event — that is `append_event`'s transaction, and it is already proven; do not add a second write path that could break it.
6. **Seat wiring — confirm it, don't assume it.** pi reads `~/.pi/agent/mcp.json` (global) or `.pi/mcp.json` (project, trusted only). Add the server with:
   ```
   pi mcp add seatloom-emit --env DATABASE_URL='postgresql://…' -- /abs/path/to/seatloom-mcp-emit
   ```
   Then **run `pi mcp list` and paste the output into your delivery** — it must show the server connected and `emit_event` listed. That is the evidence a seat can actually call it. Note `pi --no-mcp` disables MCP entirely and `-t/--exclude-tools` can hide `mcp__*` tools; if a seat reports the tool missing, those are the first two causes. Do **not** edit another seat's live `~/.pi/agent/mcp.json` — record the exact command for Lyra to roll out.
7. **Crate availability is an open risk — resolve it and report.** The internal Artifactory sparse index (`.cargo/config.toml`) may or may not carry an MCP crate. Try one; if it resolves, use it. **If no MCP crate resolves, hand-roll the JSON-RPC 2.0 stdio framing** (newline-delimited JSON, `initialize` / `tools/list` / `tools/call`, correct `content` array shape) — that is a small, well-specified surface and is the sanctioned fallback. State in your delivery which path you took and the exact `cargo add` / vendoring evidence. Do **not** introduce a direct overseas registry fetch: every dependency must resolve internally or domestically (the 2026-09-29 rule), or you must vendor it.

## 3. Hard constraints

- **Never run against the live `seatloom` database.** Use a scratch database (e.g. `seatloom_mcp_nimbus`) and say so in the delivery, exactly as `p0_write_api_proof.rs` does. Migration `010` must be applied there.
- **Do not weaken A1.** No new write path, no projection mutation, no relaxed validation, no swallowing a `WriteError` into a success. If you think A1 needs a change, **STOP and report** — sequence it, do not fold it in.
- **No schema changes, no `.sql`.** `010` is closed.
- **Zero UI.** No `.tsx`. Mira owns `ui/src`.
- **Do not disturb the live seats**: no tmux changes, no seat session touched.
- Work in your worktree on the new branch from `track/infra-foundation` @ `c495ffa`.

## 4. Verification (you run; Flux gates on real Postgres)

```
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Plus, on the scratch database:

1. **Happy path:** `emit_event(ArtifactCreated, {…}, refs=[subject=workitem:wi-…, actor=seat:…])` → the event lands in `canonical_events` **via the write API**, its projection updates **in the same transaction**, and it is retrievable (`SELECT` the event and its refs; show the `role` values).
2. **Rejections — each must fail with a typed error and write nothing:**
   - `type` outside the 12-value vocabulary;
   - zero subject refs;
   - two subject refs;
   - missing `schema_version` is fine (defaults to 1), but `schema_version: 0` and a `(type, version)` pair with no contract must be rejected;
   - `payload` that is not an object;
   - a subject `ref_type` that does not match the event_type.
   After every rejection, prove nothing was written (`SELECT count(*)` before/after).
3. **Atomicity:** a forced failure mid-append (e.g. a duplicate ref, or a dead DB) rolls back event **and** refs **and** projection together.
4. **`pi mcp list` output** showing the server connected with `emit_event` available.
5. `git show --stat <sha>` proving zero `.tsx`, zero `.sql`, and no changes under `crates/seatloom-core/src/db/write_api.rs` or `event_domain.rs` (those are A1's accepted files — read them, do not edit them).

## 5. Done-definition

1. A seat can call `emit_event(type, payload, refs)` through pi's MCP and it lands via the write API in one transaction, projection included, retrievable.
2. Every malformed event is rejected with a typed error and writes nothing.
3. `schema_version` and `role` are always set explicitly; exactly one subject per event.
4. The value-domain contract (§1) is carried verbatim in the tool description, the packet, and the delivery.
5. Layer A green, verbatim.
6. Never run against the live database; scratch DB named in the delivery.
7. Zero `.tsx`, zero `.sql`, no A1/enum files edited, no live seat disturbed.
8. Delivery artifact at the header path with the exact delivery commit and the evidence above.

## 6. Blockers

- None at dispatch time.
- **Report rather than work around** if a line number or wrapper has moved, or if A1 needs a change to make `emit_event` expressible — three of the last five gates turned up a stale fact in a packet, and a wrong assumption here lands in an append-only table.

---

*Dispatched by Lyra · 2026-10-08 · P0 step 4, the capture substrate for the end-state · step 6 governed dispatch and step 5 first captured event both ride on this · reuse A1's `append_event`, never rebuild it*
