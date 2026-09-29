# Task: Runtime Triple Refactor — split `Runtime` into harness / agent / model

[Lyra -> Nimbus] Mr. Zhang ratified the Aegis multi-harness ingestion architecture review and
selected **sequencing option B**: build adapters + index + this `Runtime` triple refactor in
parallel with L1 Supervisor IM, but render **no UI surface** until L1 ships. This packet is
item 1 of review §8. It is a schema/type refactor with zero user-visible surface.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-09-28-runtime-triple-refactor-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-28 |
| to | nimbus |
| priority | P1 (parallel with L1 Supervisor IM; does not block L1) |
| milestone | pre-ingestion schema debt paydown |
| depends_on | `docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md` §4 and §8 item 1; `docs/coordination/memory/2026-09-28.md` Addendum (seat harness convergence) |
| paired_packet | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `9599136d615db0c4cdb15811ccaca5532820870d` |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1` |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-delivery-v1.md` |
| tags | nimbus, schema, runtime-triple, seat-role, ingestion-prep, option-B, no-ui |

---

## 1. Context

### 1.1 Why now

Aegis review §4 established that `crates/seatloom-core/src/objects/session.rs:24` collapses three
orthogonal dimensions into one field:

```rust
pub enum Runtime { ClaudeCode, Codex, CursorCli, GeminiCli, Custom(String) }
```

| Dimension | Meaning | Example today |
|---|---|---|
| harness | process wrapper / credential / proxy layer | `stepcode` (stepgo) |
| agent | the CLI or protocol being driven | Claude Code |
| model | the model id the agent is pinned to | `claude-opus-5[1m]` |

As of 2026-09-28 all six seats are **stepcode harness → Claude Code agent → `claude-opus-5[1m]`**
(evidence: `docs/coordination/memory/2026-09-28.md` Addendum, "Seat state after convergence"
table). `default_runtime = 'ClaudeCode'` cannot express that. A product whose stated purpose is
multi-harness orchestration cannot carry a one-field runtime.

The refactor is required **before** the ingestion adapters are written, otherwise every adapter
encodes the same lossy flattening and the migration cost compounds with every seat and every
seeded row.

### 1.2 The key design resolution

Review §4 flags "missing variants": opencode and stepgo/stepcode have no representation and fall
into `Custom(String)`. Under the triple this resolves cleanly rather than by adding variants to a
single enum:

- `stepcode` / `stepgo` is a **harness**, not an agent.
- `opencode` is an **agent**, not a harness.

One enum could not express that distinction. Three fields can. State this explicitly in the
delivery doc so a future reader does not re-add `Stepcode` as an agent variant.

### 1.3 Facts measured at base commit (do not re-derive, but do re-verify if stale)

| Fact | Location | Value |
|---|---|---|
| `Runtime` enum | `crates/seatloom-core/src/objects/session.rs:23-30` | 5 variants, **no** serde attributes → variant names serialize verbatim (`ClaudeCode`, `Codex`, …; `Custom` as an externally-tagged map) |
| `Session.runtime` | `objects/session.rs:10` | `Runtime` (typed, non-optional) |
| `SeatIdentity.default_runtime` | `objects/seat.rs:16` | `Option<Runtime>` (typed) |
| `SeatRole` | `objects/seat.rs:57-64` | `ProductOwner, Architect, Verifier, Designer, Custom(String)` — **no** `Supervisor`, **no** `DataEngineer`, **no** serde attributes |
| DB seat row | `crates/seatloom-core/src/db/models.rs:19` | `default_runtime: Option<String>` — string passthrough |
| DB session row | `crates/seatloom-core/src/db/models.rs:52` | `runtime: String` — string passthrough |
| Row hydration | `db/repositories.rs:1276`, `db/repositories.rs:1323` | `r.get(2)` — raw text, never parsed into `Runtime` |
| DTO surface | `src-tauri/src/dto.rs:49,60` and `:129,147` | `Option<String>` / `String` passthrough to the wire |
| PG columns | `infra/postgres/schema/001_seatloom_core.sql:24` (`seats.default_runtime TEXT`), `:65` (`sessions.runtime TEXT NOT NULL`) | untyped TEXT |
| Seed role values | `infra/postgres/seed/001_real_collaboration_baseline.sql:40-45` | `supervisor`, `product_owner`, `designer`, `architect`, `verifier`, `data_engineer` — snake_case |
| File-backed seat store | `.seatloom/` | contains only `bootstrap/` and `transcripts/`. **`.seatloom/seats/` and `.seatloom/delegations/` do not exist.** Zero on-disk `SeatIdentity` YAML files. |

**The two paths disagree and nothing breaks today only because the DB path never parses into the
enum.** That is the latent defect this packet closes.

The last fact in that table is load-bearing for §2.4: because there are **zero** on-disk seat
identity files, a serde representation change to `SeatRole` or `Runtime` breaks no existing
artifact. Verify it still holds (`ls .seatloom/`) before you rely on it, and record the result.

---

## 2. Scope

### 2.1 The value-domain contract (authoritative — Lyra-owned, shared with Onyx)

This table is the contract between this packet and
`ONYX-2026-09-28-runtime-triple-migration-v1`. It appears **verbatim and identically** in both
packets. It defines the exact strings that cross the Rust ⇄ PostgreSQL boundary.

| Field | Rust type | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| harness | `Option<Harness>` | `Direct`, `Stepcode`, `Custom(String)` | `TEXT` | yes — `NULL` means *not recorded*, never *none* |
| agent | `Agent` | `ClaudeCode`, `Codex`, `CursorCli`, `GeminiCli`, `Opencode`, `Custom(String)` | `TEXT` | no, on `sessions`; yes on `seats.default_*` |
| model | `Option<ModelId>` (newtype over `String`) | free-form, e.g. `claude-opus-5[1m]` | `TEXT` | yes — `NULL` means *not recorded* |

Binding rules:

1. **Variant names serialize verbatim in PascalCase. Do NOT add `#[serde(rename_all = ...)]` to
   `Harness` or `Agent`.** The existing `sessions.runtime` values in PostgreSQL are already
   PascalCase (`ClaudeCode`, `GeminiCli`, `Custom`). Keeping PascalCase makes Onyx's migration a
   pure column split rather than a value re-encoding, and keeps the diff auditable.
2. `model` is a **`String` newtype, not an enum.** Model ids churn faster than releases; an enum
   would need a migration per model. `ModelId` exists for type-safety at call sites only.
3. `NULL` / `None` for harness and model means *the value was not recorded at the time*. It does
   **not** mean "direct" or "default". Do not backfill a guess. This follows review §2: attribution
   is derived and revisable, never ground truth baked in at ingest time.
4. `Custom(String)` is retained on both `Harness` and `Agent` as the escape hatch, with the same
   externally-tagged serde shape `Runtime::Custom` has today.

If you need to deviate from this table, **post a comment on this packet and notify Onyx in
`Onyx-data-seatloom` before Onyx's migration lands** — do not change it unilaterally and do not
assume Onyx will discover the change. A silent divergence here reproduces exactly the typed-vs-
string-keyed split this packet exists to close.

### 2.2 `crates/seatloom-core/src/objects/session.rs`

1. Introduce `Harness`, `Agent`, `ModelId` per §2.1.
2. Replace `Session.runtime: Runtime` with three fields: `harness: Option<Harness>`,
   `agent: Agent`, `model: Option<ModelId>`.
3. **Delete the `Runtime` enum.** Do not keep it as a deprecated alias. A half-migrated type is
   how the current typed/string-keyed split happened; leaving `Runtime` in place invites the ingest
   adapters to keep using it. If a compatibility shim is genuinely unavoidable for a caller you
   cannot reach in this packet, name that caller in the delivery doc and justify it — do not add
   the shim pre-emptively.

### 2.3 `crates/seatloom-core/src/objects/seat.rs`

1. `SeatIdentity.default_runtime: Option<Runtime>` (line 16) becomes
   `default_harness: Option<Harness>`, `default_agent: Option<Agent>`, `default_model: Option<ModelId>`.
2. Add **`SeatRole::Supervisor`** (review §8 item 1, explicit). Aegis is bound to that role and the
   enum has no variant for it.
3. Add **`SeatRole::DataEngineer`**.

   > **Lyra scope note — this is an explicit, non-silent addition beyond review §8 item 1.**
   > Rationale: onyx was promoted to the sixth registered seat on 2026-09-28
   > (`docs/coordination/memory/2026-09-28.md` Addendum decision 3) and the seed already writes
   > `data_engineer` at `seed/001_real_collaboration_baseline.sql:45`. Adding `Supervisor` while
   > leaving `data_engineer` to fall through to `Custom(String)` would close five sixths of the
   > same defect and leave the sixth to be rediscovered. Recorded here rather than absorbed
   > quietly. If Aegis objects, drop `DataEngineer` and say so in the delivery doc.

4. Add `#[serde(rename_all = "snake_case")]` to `SeatRole`.

   **Decision and rationale (Lyra):** the seed writes snake_case (`product_owner`, `supervisor`,
   `data_engineer`) and PostgreSQL is canonical per AD-011, so the DB values win. The alternative
   — rewriting the seed to PascalCase — would churn historical seed rows for no gain. There are
   **zero** on-disk `SeatIdentity` YAML files (`.seatloom/seats/` does not exist), so this
   representation change breaks no existing artifact. **Confirm that emptiness in the delivery doc
   with an `ls -a .seatloom/` transcript before relying on it.** If seat YAML files have appeared
   since this packet was written, stop and report rather than rewriting them.

   Note the asymmetry with rule §2.1.1 (PascalCase for `Harness`/`Agent`) and state it in the
   delivery doc: `SeatRole` matches existing snake_case DB values, `Harness`/`Agent` match existing
   PascalCase DB values. Both rules follow the same principle — *match what is already in
   PostgreSQL* — and produce opposite casing because the existing data is inconsistent. Do not
   "harmonize" them; harmonizing means rewriting seed rows, which is out of scope.

### 2.4 Reconcile the typed and string-keyed paths

This is the part review §4 calls out as "must be resolved in the same change". Today:

- `storage/seat_registry.rs` round-trips the **typed** `SeatIdentity` through YAML.
- `db/models.rs:19` carries `default_runtime: Option<String>` and `db/repositories.rs:1276` hands
  raw text straight to the DTO — the enum is never involved.

Required end state: **one parse boundary, at row hydration.**

1. `SeatRow` (`db/models.rs:16-23`) gains `default_harness/default_agent/default_model` as
   `Option<String>` — the row struct stays a faithful mirror of the SQL columns. That is correct
   and should not change.
2. `SessionRow` (`db/models.rs:50-64`) replaces `runtime: String` with `harness: Option<String>`,
   `agent: String`, `model: Option<String>`.
3. Add explicit, **fallible** conversions from the row strings into the typed enums —
   `TryFrom<&str>`/`FromStr` for `Harness` and `Agent`, and the same for `SeatRole`. Unknown values
   must produce a typed error, not a silent `Custom(...)` fallback and not a panic. This mirrors
   review §5's adapter rule: *fail loudly on unknown record types rather than skipping them.*
4. Call those conversions at row-hydration time in `db/repositories.rs` (around `:1276` and
   `:1323`) so a malformed DB value surfaces at the boundary with the offending string in the error
   message, naming the table and row id.
5. Add a unit test proving a bad value produces the typed error — e.g. `Agent::try_from("Bogus")`
   is `Err`, and hydrating a `SessionRow` with `agent = "Bogus"` fails with the row id in the
   message.

Whether the DTO layer keeps `String` or carries the typed value is **your call** — state the choice
and the reason in the delivery doc. The invariant Lyra requires is only this: *there is exactly one
place where a string becomes an enum, and it rejects garbage loudly.*

### 2.5 DTO / wire surface

`src-tauri/src/dto.rs` `:49,60` (seat) and `:129,147` (session) must carry the three fields instead
of the collapsed one.

**Serde naming — verify, do not assume.** Read the `#[serde(...)]` annotations on both the
response structs *and* any request structs you touch before writing the TypeScript side. Confirm
the actual wire casing (`defaultHarness` vs `default_harness`) by reading the attribute, not by
inferring it from the Rust field name and not by observing that `tsc` is silent. `tsc --noEmit`
passing proves the TS types agree with each other; it does **not** prove they agree with the Rust
wire format. Paste the relevant `#[serde(...)]` lines verbatim into the delivery doc alongside the
TS declaration, so the pairing can be checked by eye.

### 2.6 Callers and tests

Every construction site of `Runtime` must be updated. Known at base commit:

- `crates/seatloom-core/src/storage/seat_registry.rs:188, 245, 261, 432, 440` (test module)
- `crates/seatloom-core/src/objects/seat.rs:7` (the `use` of `Runtime`)

Sweep for the rest yourself; `rg 'Runtime'` over `crates/` and `src-tauri/` is the minimum.
The existing `seat_identity_round_trips` test at `seat_registry.rs:239` must be updated to assert
the triple round-trips, not just that one field survives.

---

## 3. Out of scope (do NOT do)

- **No UI surface. This is the binding option-B constraint.** Zero `.tsx` files may change. Type
  declarations under `ui/src/**/*.ts` may change **only** where `pnpm exec tsc --noEmit` forces a
  type-only fix from the DTO change. `git show --stat` in the delivery doc must show no `.tsx`
  file. If a `.tsx` file must change to keep the build green, stop and report rather than editing
  it — that would be an L2 surface change and needs Mr. Zhang's sequencing decision, not yours.
- **Do not create `crates/seatloom-ingest/`.** That is review §8 item 3, a separate packet issued
  after this one lands. No adapter, no `HarnessAdapter` trait, no probe code in this packet.
- **Do not write the SQL migration or touch `infra/postgres/`.** That is Onyx's packet. If you
  believe the migration needs a shape the contract table in §2.1 does not allow, say so in
  `Onyx-data-seatloom` and on this packet — do not write the SQL yourself.
- **Do not drop `seats.default_runtime` or `sessions.runtime`** by any route. The column retirement
  is deliberately deferred to a follow-up packet after Flux confirms no reader references them.
- **Do not rewrite historical seed rows** to make anything typecheck. If a historical value cannot
  be represented, that is a finding to report, not a row to edit.
- **Do not add model-specific logic** — no routing, no capability tables, no per-model branching.
  `ModelId` is an opaque string newtype in this packet.

---

## 4. Invariant compliance

Copy this table verbatim into the delivery doc §Compliance with line-pinned evidence in each cell.

| Rule | Required behavior | Verification approach |
|---|---|---|
| Option B — no L2 surface | No rendered surface added | `git show --stat <sha>` shows zero `.tsx` files |
| R1 attach-only | No spawn / no PTY behavior change | `git show <sha> -- crates/seatloom-core/src/pty/` must be empty |
| Review §2 — attribution is revisable | `NULL`/`None` harness+model mean *not recorded*; no guessed backfill in Rust defaults | Show the `Option<...>` types and the absence of any `unwrap_or(Harness::Direct)`-style default |
| Review §5 — fail loudly | Unknown enum string produces a typed error naming the value, not `Custom` fallback, not panic | The §2.4.5 unit test output |
| AD-011 — PG is canonical | Serialized domains match existing PG values per §2.1 | Contract table restated + the `#[serde(...)]` lines pasted verbatim |
| One parse boundary | Exactly one place converts string → enum | Name the file:line; show `repositories.rs` calls it |

---

## 5. Verification plan (Layer A — Nimbus runs, Flux verifies commit-pinned)

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test  -p seatloom-core --lib
cargo clippy -p seatloom-core -- -D warnings
cd ui && pnpm exec tsc --noEmit
cd ui && pnpm build
```

Plus, pasted verbatim:

```
ls -a .seatloom/                                  # §2.3.4 / §1.3 emptiness precondition
rg -n 'Runtime' crates/ src-tauri/                # must show zero hits post-refactor, or each
                                                  # remaining hit justified in the delivery doc
git show --stat <sha>                             # must show zero .tsx files
```

All output verbatim. No summaries, no "all green" without the transcript.

There is **no Layer B runtime step in this packet** — by construction it has no surface to
exercise. Do not invent one.

---

## 6. Delivery doc requirements

Write `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-delivery-v1.md`
containing:

1. T3 frontmatter: template, `subtype=implementation_delivery`, `status=delivered`, base branch,
   base commit, delivery branch, delivery commit SHA, `packet_ref` to this file.
2. `git show --stat <sha>` verbatim.
3. Every §5 command's output verbatim.
4. §4 compliance table with line-pinned evidence in each cell.
5. **The §2.1 contract table restated**, with any deviation called out explicitly and marked with
   the date you notified Onyx.
6. **Serde wire-shape evidence** — the `#[serde(...)]` attribute lines from `dto.rs` pasted
   verbatim next to the corresponding TS declarations (§2.5).
7. The §2.4 parse-boundary decision: where the single string→enum conversion lives, why, and the
   failing-value test output.
8. The §1.2 statement that `stepcode` is a harness and `opencode` is an agent, so a future reader
   does not re-add either as the wrong dimension.
9. Anything you found that contradicts §1.3. The facts there were measured at base commit
   `9599136`; if any is stale, report it rather than working around it.

---

## 7. Done-definition (Lyra acceptance criteria)

This packet is done when **all** of the following hold. Partial completion is not delivery.

1. `Runtime` no longer exists in `crates/` or `src-tauri/`; `rg -n 'Runtime' crates/ src-tauri/`
   returns zero hits, or every remaining hit is individually justified in the delivery doc.
2. `Session` carries `harness: Option<Harness>`, `agent: Agent`, `model: Option<ModelId>`.
3. `SeatIdentity` carries `default_harness` / `default_agent` / `default_model`.
4. `SeatRole` has `Supervisor` and `DataEngineer` variants and `#[serde(rename_all = "snake_case")]`.
5. Serialized value domains match the §2.1 contract table exactly — or the deviation is documented
   **and** Onyx was notified before their migration landed, with the notification timestamp in the
   delivery doc.
6. Exactly one string→enum parse boundary exists, it is called at row hydration in
   `db/repositories.rs`, and it returns a typed error naming the offending value and row id.
7. The §2.4.5 negative test exists and passes.
8. `cargo check` (both crates), `cargo test -p seatloom-core --lib`, and
   `cargo clippy -p seatloom-core -- -D warnings` are all green with output pasted verbatim.
9. `pnpm exec tsc --noEmit` reports zero errors; `pnpm build` exits 0.
10. `git show --stat` shows **zero** `.tsx` files changed and **zero** files under
    `infra/postgres/`.
11. The delivery doc contains all nine items in §6.
12. Flux commit-pinned Layer A verify returns PASS against the exact delivery SHA.

## 8. Evidence expectations

Per COORDINATION_RULES §5.3 and §14, the delivery is rejected without:

- base branch, base commit, delivery branch, delivery commit SHA — all four named explicitly;
- every §5 command's **verbatim** output, not a summary and not a claim;
- `git show --stat <sha>` proving the file scope;
- line-pinned grep/inspection evidence for each row of the §4 compliance table;
- the negative-path test output from §2.4.5 (a passing happy path alone is not evidence that the
  failure mode works);
- an explicit statement for anything **not** done, with the reason.

Evidence-free completion claims are rejected without review (LYRA.md "Must Not Do").

---

## 9. Coordination

- Paired packet: `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md`,
  dispatched to `Onyx-data-seatloom` at the same time as this one.
- The two packets are **parallel, not sequential**, joined only by the §2.1 contract table. Neither
  blocks the other. Onyx writes SQL against the contract; you write Rust against the contract.
- If the contract has to move, it moves in **both** packets, and whoever moves it tells the other
  seat directly in their tmux session before landing. Lyra arbitrates if you disagree.
- Escalate to Aegis (`Aegis-Supervisor-seatloom`) only for architecture arbitration — e.g. if you
  conclude the three-dimension split itself is wrong. Routine questions come to
  `Lyra-po-seatloom`.

---

*Dispatched by Lyra · 2026-09-28 · Aegis review §8 item 1 · option B (no UI surface until L1 ships)
· paired with ONYX-2026-09-28-runtime-triple-migration-v1 via the §2.1 value-domain contract*
