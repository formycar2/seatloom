# SeatLoom Multi-Harness Session Ingestion — Architecture Review

| Field | Value |
|---|---|
| template | T4 |
| subtype | architecture_review |
| id | review-2026-09-28-multi-harness-ingestion |
| status | draft — pending Mr. Zhang direction on sequencing |
| author | aegis |
| date | 2026-09-28 |
| version | v1 |
| depends_on | `docs/architecture-decisions.md` (AD-011), `docs/coordination/reviews/2026-09-28-session-recovery-lookup-and-response-method.md`, `.seatloom/bootstrap/source-map.yaml` |
| tags | ingestion, multi-harness, session-continuity, adapter, data-layer |

---

## 0. Goal

Make SeatLoom the single place where AI-driven work continues, regardless of which
harness produced it. Concretely: every session already on this machine becomes
discoverable, searchable, and resumable from SeatLoom, and new work continues there.

This review establishes the evidence basis and the adapter architecture. It does not
authorize implementation; sequencing against the L1 Supervisor IM rule is an open
question (§7).

---

## 1. Inventory — measured, not estimated

Measured on this machine, 2026-09-28.

| Harness | Store | Format | Volume | Sessions | Date range |
|---|---|---|---|---|---|
| Claude Code | `~/.claude/projects/<path-slug>/<uuid>.jsonl` | NDJSON, heterogeneous record types | 257 MB | 27 files / 9 projects | 2026-07-25 → 2026-09-28 |
| Codex | `~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<uuid>.jsonl` | NDJSON `{timestamp, type, payload}` | 14 GB | 247 rollouts / 30 cwds | 2026-02-25 → 2026-09-24 |
| Gemini CLI | `~/.gemini/history/<project>/`, `~/.gemini/tmp/<project>/logs.json` | JSON logs + checkpoints | 854 MB | 12 projects, 12 log files | — |
| stepgo/stepcode | `~/.stepgo/sessions/*.jsonl` | NDJSON | 108 KB | 16 | — |
| opencode | `~/.local/share/opencode/storage/{session_diff,migration}` | per-session files | 57 MB | 36 files | — |
| Cursor | `~/.cursor` | mixed, incl. workspace state | 649 MB | not yet probed | — |

Approximately **16 GB and ~350 machine-readable sessions spanning eight months**.

Codex additionally carries structured state that has no analogue in the other
harnesses and is the most valuable single target:

- `~/.codex/session_index.jsonl` — `{id, thread_name, updated_at}`, a ready-made catalog
  where `thread_name` is a human-meaningful title derived from the opening prompt.
- `~/.codex/thread_history_1.sqlite` — tables `thread_items`, `thread_turns`,
  `thread_realtime_items`, `thread_history_projection_state`.
- `~/.codex/memories_1.sqlite`, `state_5.sqlite`, `goals_1.sqlite`, `queue_1.sqlite`.

### 1.1 Record shapes confirmed by sampling

Claude Code transcript record types observed in one file: `user`, `assistant`, `system`,
`attachment`, `file-history-snapshot`, `cost-state`, `mode`, `permission-mode`,
`queue-operation`, `last-prompt`, `atis-latch`. Note the first line of a transcript is
**not** a session header — it is often `last-prompt`. Session metadata must be
recovered by scanning for the first record carrying `sessionId` / `cwd` / `gitBranch`.

Codex rollout line 1 is `type: session_meta` with `payload.{id, timestamp, cwd, originator}`.
This is a genuine header and makes Codex cheap to index.

---

## 2. The actual problem

It is not format conversion. Five harnesses produce five incompatible transcripts, but
each is individually parseable. The hard problem is **identity**: there is no shared key
across harnesses. A single piece of work — "the GPU capacity planner mainline" — exists
as Codex rollouts, Claude Code transcripts, and tmux scrollback with no common id.

What actually joins them is the triple:

```
(project cwd, wall-clock window, git branch)
```

All three harnesses record cwd. Codex and Claude Code both record timestamps.
Claude Code records `gitBranch`. This triple is the join key, and it is approximate —
the design must treat attribution as **derived and revisable**, never as ground truth
baked into the object at ingest time.

---

## 3. Proposed architecture

A new read-only crate, `crates/seatloom-ingest/`, sitting beside `seatloom-core`.

### 3.1 Adapter trait

```
trait HarnessAdapter {
    fn id(&self) -> HarnessId;
    fn probe(&self) -> Result<Vec<StoreRoot>>;          // locate stores on this machine
    fn enumerate(&self, root: &StoreRoot) -> Result<Vec<SessionRef>>;  // headers only
    fn project(&self, r: &SessionRef) -> Result<CanonicalSession>;     // full parse, on demand
}
```

`SessionRef` is deliberately tiny: `{harness, native_id, cwd, started_at, ended_at, title, byte_size, source_path}`.

### 3.2 Three layers, in this order

1. **Index** — run `probe` + `enumerate` across all adapters, persist `SessionRef` rows
   into PostgreSQL. Headers only. 16 GB of transcripts yields a catalog measured in
   megabytes. This layer alone delivers global cross-harness search, which is most of
   the user-visible value.
2. **Projection** — call `project()` lazily when a session is actually opened, producing
   a canonical turn stream `{role, content, tool_calls, tokens, timestamp}`. Nothing is
   bulk-converted; the 14 GB never enters PostgreSQL.
3. **Attribution** — derive `(session → project → seat → workitem)` links from the §2
   triple. Stored as a separate, revisable table with a confidence field, never merged
   destructively into the session row.

### 3.3 Non-negotiable invariant

Ingestion is **strictly read-only** against every source store. This is the same
invariant that governs the tmux mirror: SeatLoom observes, it does not own. If SeatLoom
is deleted, every harness store is untouched and every harness keeps working. Adapters
open files `O_RDONLY`; sqlite is opened in immutable/read-only mode.

---

## 4. Schema gap that blocks this today

`crates/seatloom-core/src/objects/session.rs:24`:

```rust
pub enum Runtime { ClaudeCode, Codex, CursorCli, GeminiCli, Custom(String) }
```

Two problems.

**Missing variants.** opencode and stepgo/stepcode have no representation and would fall
into `Custom(String)`, losing type safety exactly where the product's core claim lives.

**Conflated concepts.** `Runtime` collapses three orthogonal dimensions into one field:

| Dimension | Example today |
|---|---|
| harness (process wrapper, credential/proxy layer) | `stepcode` (stepgo) |
| agent (the CLI / protocol) | Claude Code |
| model | `claude-opus-5[1m]` |

All six seats are currently **stepcode harness → Claude Code agent → Opus 5 (1M)**, and
`default_runtime = 'ClaudeCode'` cannot express that. A tool whose stated purpose is
multi-harness orchestration cannot carry a one-field runtime. This needs to become a
triple before ingestion is built, otherwise every adapter encodes the same lossy
flattening and the migration cost compounds.

Note the file-backed and database paths currently disagree and this must be resolved in
the same change: `db/models.rs:19` carries `default_runtime` as `Option<String>` and
passes it straight through to the DTO, while `storage/seat_registry.rs` uses the typed
`Runtime` enum. The seed writes snake_case role strings (`product_owner`, `supervisor`)
that do not match `SeatRole`'s default serde naming, and `SeatRole` has no `Supervisor`
variant at all despite aegis being bound to that role. Today nothing breaks only because
the DB path never parses into the enum.

---

## 5. Risks

| Risk | Mitigation |
|---|---|
| 14 GB Codex store pulled into PostgreSQL | Index headers only; lazy projection. Hard rule: transcript bodies never persist to the DB. |
| `~/.gemini` has 174,438 files, mostly not transcripts | Targeted probe of `history/` and `tmp/*/logs.json` only; never walk the tree blind. |
| Upstream format drift silently corrupts the index | Each adapter pins a format version and fails loudly on unknown record types rather than skipping them. |
| Attribution guesses become treated as truth | Confidence field, separate table, always revisable, always shows its evidence. |
| Ingestion competes with L1 Supervisor IM for delivery capacity | Open question — see §7. |

---

## 6. Why SeatLoom is well positioned

The object model already fits: `Session`, `WorkItem`, `Artifact`, `Handoff` are exactly
the objects a cross-harness archive needs. AD-011 already establishes PostgreSQL as
canonical structured truth with file stores as cache. The tmux mirror work (B1) already
proved the read-only-observer pattern end to end. The ingestion layer is an adapter
tier on top of an object model that was, as it happens, designed for this.

---

## 7. Open question — sequencing against the L1 rule

Standing rule: the Supervisor IM is L1 and must be complete before any L2 surface
(Sessions / Documents / main shell) is built.

The Index layer (§3.2 step 1) is backend and produces no new surface, so it does not
obviously violate the rule. But a cross-harness session archive is only useful once
something renders it, and that renderer is an L2 surface.

Three options, for Mr. Zhang to choose:

- **A — L1 first, strictly.** Finish Supervisor IM. Ingestion starts after. Cost: the
  archive stays unavailable while more sessions accumulate.
- **B — Index now, surface later.** Build adapters + index + the `Runtime` triple
  refactor in parallel with L1, but render nothing until L1 ships. The data is captured
  and the schema debt is paid while the surface waits.
- **C — Ingestion becomes the L1 surface.** Argue that a cross-harness session archive
  *is* the supervisor's primary view, and merge the two tracks.

Recommendation: **B**. The §4 schema gap has to be fixed regardless of sequencing, and
it gets more expensive with every seat and every seeded row. Indexing is read-only,
carries no UI surface, and the catalog is small. It pays down debt that L1 will
otherwise inherit.

---

## 8. Next actions if approved

| # | Owner | Action |
|---|---|---|
| 1 | Nimbus | Refactor `Runtime` into the harness/agent/model triple; reconcile the typed and string-keyed paths; add `SeatRole::Supervisor` |
| 2 | Onyx | Migration + seed realignment for the new runtime triple; drift check between `storage/seat_registry.rs` and `db/repositories.rs` |
| 3 | Nimbus | `crates/seatloom-ingest/` skeleton + `HarnessAdapter` trait + Codex adapter (best headers, largest corpus) |
| 4 | Nimbus | Claude Code adapter (note: first line is not a header) |
| 5 | Flux | Read-only invariant verification: prove every source store is byte-identical after a full index run |
| 6 | Lyra | Acceptance criteria for "session is discoverable and resumable from SeatLoom" |
| 7 | Mira | Deferred until L1 ships, per §7 option B |
