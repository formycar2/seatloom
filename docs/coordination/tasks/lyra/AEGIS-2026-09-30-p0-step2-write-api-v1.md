# Dispatch: P0 step 2 — write API (A1) to Nimbus, in a worktree

[Aegis -> Lyra] P0 step 1 (schema foundation 010) is Flux-PASSed, merged, accepted, and
reconciled to GitLab (`231d0a5`, CI green). Step 2 is the GO. Toward user-milestone **UM1**
("work leaves a clickable trace").

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-09-30-p0-step2-write-api-v1 |
| status | dispatched |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | P0 capture loop — write side (UM1) |
| depends_on | `docs/coordination/ROADMAP.md`, `infra/postgres/schema/010_p0_schema_foundation.sql` |
| tags | p0, dispatch, write-api, a1 |

## Your task (PO): author + dispatch the Nimbus write-API packet

Build plan §3 step 2 (A1/G1). One packet to Nimbus. Do not dispatch step 4/6 yet.

### Scope

A single write primitive: **`append(event) -> projections`**. Every domain mutation is an
event append **plus** its projection update **in one transaction**. No repository method
mutates a projection table directly. Reads stay on the projections (CQRS-lite).

### Must honour 010 (now merged), verbatim in the packet

- Set `schema_version` **explicitly** on every append (010 has `DEFAULT 1` as a legacy-seed
  crutch only; new code must set it — dropping the default is a P1 cleanup).
- Set `event_object_refs.role` **explicitly** (`subject`/`actor`/`context`); never rely on the
  `context` default.
- **Exactly one `subject` per new event** is the write API's obligation: 010's partial unique
  index guarantees *at most* one; the API enforces *presence* per append (reject an event with
  zero subjects). This is the "exactly-one" half 010 deliberately left to step 2.
- `event_type` ∈ the 12-value vocabulary; `ref_type` ∈ its domain. Enums crossing the
  Rust⇄PostgreSQL TEXT boundary get **hand-written `FromStr`/`Display`** (ENGINEERING_STANDARDS;
  `objects/session.rs::parse_custom_tagged`), not serde derive.
- Carry the **verbatim value-domain contract** (Rust type ⇄ serialized value ⇄ PG column ⇄
  nullability) in the packet — same discipline that kept runtime-triple and 010 aligned.

### Success test (the A1 acceptance)

A projection table can be **rebuilt from `canonical_events` alone**. If it cannot, the ledger
is still decorative and step 2 is not done.

### Isolation — build in a git worktree, NOT the shared tree

Nimbus must work in its own `git worktree` (as Flux did for 010 verification), because the
shared checkout is now hosting a running SeatLoom UI (dev watcher regenerates
`capabilities.json` and dirties the shared tree). Two reasons, one rule: never build a code
packet in the shared tree while the UI dev server runs there. Give Nimbus the
`git worktree add` path and the recovery note for packet files.

### Acceptance (Flux, on real Postgres)

Projections rebuildable from the ledger; a zero-subject append rejected; a normal append writes
event + projection atomically (one txn, rolls back together on failure). Verify in a worktree
against a scratch database, as with 010.

## After this

step 4 (MCP emit) → step 6 (governed dispatch) → step 5 (first real captured event = UM1
perceivable in the UI). Step 2 is the critical-path write side everything else rides on.
