# Delivery: 018 memory tools — typed record fields, supersede edges, resolve closure

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-memory-tools-018-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, d5, phase-2, memory, 018, recall, resolve, supersede |
| governing dispatch | `disp-mem018-nimbus` (D5 Phase 2 Lane A2) |
| governing packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-memory-tools-018-v1.md` (on `packet/lyra/milestone-summary`; not on this branch) |
| value-domain contract | `VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1` (Lyra sign-off 2026-10-10), §G1–§G5 |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/memory-tools-018` @ `77d858a` |
| branch lineage | `packet/nimbus/d5-mcp-recall` (which carries waker-hardening -> lineage -> recall) -> `packet/nimbus/memory-tools-018` (this). Single-direction, no `dispatch.rs` divergence. |
| scratch database | `seatloom_018fresh` (001–012 + seed + 013/014 from track + 015 + 016 + 018 from Onyx's branches + scratch 017; NEVER live `seatloom`) |
| proof | `crates/seatloom-mcp-emit/proof/memory_018_proof.sh` (5/5 PASS, re-runnable) |

---

## 0. Summary

Onyx's 018 added the columns (`kind`, `supersedes`, `memory_relations`,
`resolved_by`/`resolved_at`, `confidence`, `l2_line_start`/`l2_line_end`, and the
`MemoryResolved` event). This delivery lands the tool surface that can write and
read them — the second half of the D5 Phase 2 hard prerequisite, without which
the forward-recording protocol cannot write typed records and the washing
pipeline's S3/S5 have no path.

| Tool | Change |
|---|---|
| `remember` | **REQUIRED** `kind` (6 values) and `confidence` (quoted\|inferred); optional `supersedes` (create-time only); optional `relations` (4-value vocab); `l2_line_start`/`l2_line_end` required for a file+archived_session pointer |
| `recall` | **excludes superseded rows by default**; new `include_superseded` opt-in annotates each with `superseded_by` |
| `resolve` | **NEW** — the only closure path for an `unresolved` record: `active -> resolved` + `MemoryResolved` in one transaction |

`status` / `resolved_by` / `resolved_at` remain caller-unsettable. There is no
background sweep and no automatic anything: closure is an explicit tool act.

---

## 1. What landed

### 1.1 `remember` — the typed record fields

- **`kind` (REQUIRED, 6 values)** and **`confidence` (REQUIRED, quoted|inferred)**
  — both hand-written `FromStr`/`Display` in `objects/memory.rs` with the
  contract's values pinned verbatim; no serde derive across the Rust<->PG
  boundary. The additive DB defaults (`'fact'`, `'inferred'`) are a backfill
  only: the tool refuses a missing kind/confidence rather than letting the
  default become a write path.
- **`supersedes`** — create-time only. The edge is written in **BOTH spellings**
  (`memories.supersedes` AND a `memory_relations` row with `rel='supersedes'`)
  in the SAME transaction, so they can never disagree. The proof asserts the two
  spellings agree.
- **`relations` [{to_id, rel}]** — same transaction. A target that does not
  exist, or a duplicate edge in one call, is a typed refusal (checked before any
  DB contact, so the caller gets a reason rather than a constraint name). A
  self-edge is structurally impossible here (the new id is server-generated and
  every target must already exist); the refusal variant exists for the day a
  tool takes a caller-supplied id, and is documented as such.
- **`l2_line_start` / `l2_line_end`** — REQUIRED when `l2_ref_kind='file'` AND
  `source_kind='archived_session'`, mirroring 018's CHECK with a refusal that
  names what is missing ("the evidence is somewhere in this file" is NOT
  evidence). `end >= start` always. A document pointer may omit them.

### 1.2 `recall` — a stale decision must not be served as current

The default now excludes superseded rows. `include_superseded=true` returns them,
each annotated with `superseded_by` — which memory replaced it — so the caller
can judge for itself.

### 1.3 `resolve` — the closure path

`active -> resolved` + `MemoryResolved` in ONE transaction; `resolved_by` comes
from `SEATLOOM_ACTOR_REF` (never a parameter), `resolved_at` is now. Refused: an
unknown id, an already-resolved record, a forgotten tombstone — and the refusal
names the actual status (the row is read first so the message says "resolved" or
"forgotten", not a generic "not active").

---

## 2. Two bugs the tests caught, fixed here

**Bug A — the superseded filter was backwards.** The first implementation
filtered on `supersedes IS NULL`. That column marks the memory that DOES the
superseding, so the filter hid the NEW row and kept the OLD one — precisely the
failure mode the feature exists to prevent (a stale decision served as current).
Fixed to exclude rows that are the TARGET of a supersedes edge
(`NOT EXISTS (SELECT 1 FROM memory_relations r WHERE r.to_id = memories.id AND
r.rel = 'supersedes')`). The DB test asserted the default recall returns the new
row, which is what caught it.

**Bug B — the memory results were snake_case while every other tool surface (and
the Tauri boundary) is camelCase.** `RecalledMemory` / `RecallResult` /
`TopicChainEntry` now carry `#[serde(rename_all = "camelCase")]`; both proof
scripts were updated to match.

---

## 3. Validation

Commands (Rust 1.95.0; scratch DB `seatloom_018fresh` = 001–012 + seed + 013/014
from track + 015 + 016 + 018 from Onyx's branches + scratch 017, in
`apply-db.sh`'s canonical order; `seatloom-cli reconcile` run first, as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (116 + 3 lineage + 6 D2 + 27 baseline + 3 seed) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (64, incl. the new 018 DB test) |
| proof | `proof/memory_018_proof.sh` | PASS 5/5 (re-runnable) |
| regression | `proof/d5_memory_recall_proof.sh` | PASS |

New / updated tests:

- `dispatch::db_itests::memory_tools_018_typed_fields_supersede_and_resolve` —
  the whole 018 surface against a real database: the six negative branches
  (document pointer without an id, kind out of vocab, confidence as a score, the
  missing line range, the inverted range, a relation to nothing) each a typed
  refusal with `canonical_events` unmoved; kind/confidence landing on the row;
  the supersede edge in both spellings with the equality asserted; recall's
  default exclusion and the opt-in annotation; resolve landing
  `resolved_by`/`resolved_at` + `MemoryResolved` and refusing a second call.
- `objects::memory::tests` — the three new domains round-trip, reject unknowns
  (including "0.9" as a confidence), and pin the contract's values verbatim.
- `db::event_domain` — `EventType::ALL` pinned at 27 with the 018 value in the
  verbatim list; the twelve new event types admitted and distinct.

Proof output (abridged):

```
POINT 1  6 typed refusals; canonical_events before=0 after=0; memories rows: 0
POINT 2  old row kind=failure confidence=quoted; new row kind=decision confidence=inferred
         supersedes COLUMN: mem-... (expect mem-...)
         supersedes RELATION: 1   refines RELATION: 1
         the two spellings agree: t
POINT 3  default recall: [the new row only] — superseded excluded: PASS
         opt-in recall: [(new, None), (old, 'the new row')] — supersededBy annotated: PASS
POINT 4  resolved memory ... (kind decision, closed by seat-nimbus-001 ...)
         MemoryResolved events: 1; a second resolve and an unknown id both refused
POINT 5  l2_line_start=41 l2_line_end=57; a document pointer may omit the lines: False
         tools: [... 15 tools ...]; remember requires kind + confidence: PASS
```

---

## 4. Escalations and notes

1. **Gate event emitted**: `ev-typed-memory-tools-landed` (the exact id the
   packet specifies) — it is one of the two gates for Lane C part 2.
2. **018 is Onyx's file and is not on this branch.** It was applied to the
   scratch DB from `packet/onyx/memory-hardening-018` for verification;
   `infra/**` is untouched here. The Rust pin (`EventType::ALL` at 27) matches
   018's full PG list verbatim.
3. **The wire shape changed to camelCase** for the three memory-result structs
   (Bug B). Any consumer reading `l0_summary` from a recall result must move to
   `l0Summary`; both proof scripts in this repo already do.
4. **Forward recording for this dispatch** (the protocol now in AGENTS.md): the
   memories table is still not live (018 is on Onyx's branch), so `remember`
   cannot be called from this session. Recorded here instead of skipped:
   - *decision*: "a `supersedes` COLUMN marks the memory that DOES the
     superseding; an 'exclude superseded' filter must test the reverse edge, not
     the column" — evidence: §2 Bug A.
   - *failure*: "a filter written on the natural-looking column inverted the
     feature's whole purpose; the DB-level assertion (default recall returns the
     NEW row) is what caught it" — evidence: the same.
   - *unresolved*: the wire-shape change (§4.3) may affect an external consumer
     of the recall result; none is known in this repo.
5. No whole-branch merge, no live-DB write, no `infra/**` change.
