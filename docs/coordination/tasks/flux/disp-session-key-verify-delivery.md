# Delivery — `disp-session-key-verify` (Flux): adversarial verification of the session↔work bridge

- **Dispatch**: `disp-session-key-verify`, issued by seat-lyra-001. Edge
  `ev-session-key-readmodel-landed` measured **true** — the correct edge for a verifier.
- **Verdict**: **FAIL** — one major defect (F1, a panic), four minor ones.
- **Verdict event**: `ev-verdict-session-key` — pre-declared id, emitted exactly as required
  (`ReviewVerdictIssued`, `verdict: fail`).

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-session-work-bridge-verdict-v1.md` | new — the verdict: every attack with its exact input and exact output |
| `docs/coordination/tasks/flux/disp-session-key-verify-delivery.md` | this note |
| `.scratch/flux_session_work_attack.sh`, `.scratch/flux_sw_followup.sh`, `.scratch/flux_sw_final.sh` | new — the attack scripts, so every case is reproducible |

No source file was changed. I verified at Nimbus's pin `703c863` in Nimbus's worktree, **read-only**:
nothing committed there, no tracked file touched. Behaviour was measured by driving the real
`seatloom-mcp-emit` binary over stdio against the live-shaped scratch Nimbus's own proof builds
(`seatloom_sw`). The live `seatloom` database was touched only by read-only `SELECT`s.

## Gates, run by me at the pin

`cargo fmt --all -- --check` clean · `cargo clippy -p seatloom-core --all-targets -- -D warnings`
clean · `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` clean ·
`cargo test -p seatloom-core` lib **183/0** (2 ignored), 9 of them this slice ·
`cargo test -p seatloom-mcp-emit` **90/0** (22 ignored), 6 of them this slice · waker **16/0** ·
`cargo check --workspace --all-targets` clean · **Nimbus's own `session_work_readmodel_proof.sh`
ALL POINTS PASS** · `cargo clippy --workspace --all-targets -- -D warnings` **RED** (F5, outside both
slices).

Every gate number in Nimbus's delivery note reproduces exactly. The proof is honest and passes;
everything I found is **outside** its eight points, because the scratch it builds has no `ref_only`,
`none`, `disagree` or ambiguous-path row — those states I constructed myself.

## What passes — the contract's substance holds wherever the read answers

- **The load-bearing wall holds.** Three sessions side by side: no records
  (`{total:0,resolved:0,unresolved:0}`, `records:[]`, `session:null`, note "no distilled records are
  attributed to this session … Nothing here guesses which"); records that do not resolve
  (`{total:3,resolved:0,unresolved:3}`, `records:[3]`, each naming which key failed and the exact
  value); and a MIXED session (`{total:13,resolved:12,unresolved:1}`) showing both halves at once.
  Tested from every side: `records == []` **iff** `total == 0`, no empty list with a non-zero total,
  no non-empty list with `total == 0`, and the two states' `numbers` differ on the wire.
- **A misspelled `source_ref` is reported and named.** The 3 live `path_only` memories each report
  `the recorded session reference "codex" + "019dda18-07f6-79e0-8008-e4eb16a5448d" resolves against
  no archived_sessions row`, and the drift number (3) plus the join_state breakdown ride along on
  every answer.
- **A topic id is never a key.** `wash-seg-cc-01565715-t000` and the contract's own
  `wash-sess-cc-621ae382` are both **refused** as session ids; as a `memory_id` they answer `null`;
  the topic id rides along as context with a note saying a topic is a segmentation, not an identity.
- **An empty topic is unreachable** (10 on the scratch) — correct, because a topic is never a key.
- **An unknown session and half a key are refusals** that explicitly separate "not in the archive"
  from "in the archive and produced no records".
- **The fan-out guard works.** With a second row declaring the same `raw_path`: `ref_only`,
  `resolvedBy:[a2_session_ref]`, `missing` names the ambiguity and says it is counted not guessed,
  and the memory is **not** double-counted (91 view rows for 91 memories).
- **The `disagree` state works**: both keys resolve to different rows → `disagree`, both in
  `resolvedBy`, A1 primary for the session shown, attributed to **both** sessions each counting it
  unresolved.
- **Malformed references are named precisely**, including the no-colon case
  (`"<none>" has no colon, so it names neither a harness nor an id`) and the multi-colon case where
  stepcode's wrapped `claude:<uuid>` id survives **whole**.
- **Read-only** (measured) and **one implementation** (the Tauri commands shell out to the same two
  tools; the MCP arms call the core reader; **no second SQL against `v_memory_session_join` anywhere
  in `src-tauri/` or `ui/`**). **No transcript bodies** — the record shape is pointers, a line range
  and which key.

## F1 — MAJOR: the read **panics** on "the path pointer is wrong"

022 computes `(a1.path_rows = 1) AS a1_resolves`. `a1` is a `LEFT JOIN LATERAL … LIMIT 1`, so when
**zero** rows declare the memory's corpus path, `a1.path_rows` is NULL and `NULL = 1` is NULL —
`a1_resolves` is NULL, not false. `MemorySessionJoinRow.a1_resolves` is a plain `bool`
(`models.rs:718`), so `r.get(15)` at `repositories.rs:3288` cannot deserialize it and **panics**:

```
input : memory_session {"memory_id":"mem-flux-panic"}
stderr: thread 'main' panicked at crates/seatloom-core/src/db/repositories.rs:3288:24:
        error retrieving column 15: error deserializing column 15
stdout: 1 line — the initialize response ONLY. The tool call receives NO RESPONSE AT ALL.
```

The view's own `join_state` for that row is **correct** (`ref_only`) — the SQL is right, the reader is
wrong. It is also **not contained** and **not one memory**:

- two calls in one process: the second, perfectly good call **never runs** — the panic kills the MCP
  server, so every tool in that process dies and the Tauri command's stdio shell-out gets nothing;
- the **forward** read of a session that merely *owns* one such memory panics too, so one broken path
  makes an entire session's lineage unreadable.

The boundary is exactly `a1_path_rows IS NULL`: exactly-one-row answers, two-or-more rows answers
(`ref_only`, ambiguity named), **zero rows panics**, and a path in neither accepted form panics. So
`ref_only` and `none` — the two states that mean *the pointer itself is wrong* — are reachable without
panicking only through the ambiguity route.

**Reachability, stated honestly:** measured read-only on live, 0 of 90 memories have an unresolvable
path, so this is a **landmine, not a current outage**. But it sits exactly where the contract's
load-bearing wall is: `s5_writein.py` writes `l2_file_path` as
`~/Documents/GitHub/AgentHistory/<corpus-relative path>`, so a repo move, a different home, a
different producer prefix, or **one typo in one path** turns every read into a panic. The contract's
stated purpose is to catch a key that resolves against nothing and *report* it — the A2 half of that
break is reported beautifully, the A1 half takes the product down.

**Fix:** make the field `Option<bool>` (or express the view's verdict as
`NOT a1.path_rows IS DISTINCT FROM 1`), and add a regression test that reads a row whose path
resolves against zero rows — a test the current suite cannot have, because the scratch has no such
row.

## F2–F5 — the minor ones

- **F2** the read never checks the disk, so "the source file is gone" is never said. Measured with
  `HOME` pointed where `~/Documents/GitHub/AgentHistory/…` provably does not exist: `both_agree`,
  `missing: []`, no note mentions the disk. The record **is** shown (not hidden) — that half is met.
  There is no disk I/O anywhere in the read path, whereas the sibling `read_history_session_turns`
  does compute `source_present`.
- **F3** a session with no records answers `session: null` although `archived_session_exists` already
  told the caller the row is there — the field is derived from `rows.first()`. The struct's own doc
  promises "the session's own row facts, when the session exists".
- **F4** the session-level note says a key "points at no row", which is imprecise for the ambiguous
  case (it points at two). The per-record `missing` text is exact.
- **F5** `cargo clippy --workspace --all-targets -- -D warnings` is red, two errors in
  `acceptance_cmds.rs` from `e5dbea8` — **not** a false claim in Nimbus's note (which claims
  per-crate clippy for core and mcp-emit and `check`, not clippy, for tauri), but a reader running the
  workspace gate gets red and it should be fixed before this branch is anyone's integration base.

## Not proven, stated plainly

I did not run the Tauri app — `cmd_session_work` / `cmd_memory_session` were read and their stdio
shell-out verified by grep; no window was opened. The panic is latent on live, so reproducing it
requires constructing the row; the live-shaped scratch alone will not show it. I did not apply 022 to
live — the `-live` gate is deferred for that reason and emitting it is Nimbus's act. All live access
was read-only.

## Forward recording (D5 Phase 2 Lane B)

Nothing recorded via `remember`. F1 is a file:line defect with a reproduced input/output already
written into the verdict doc; F2–F5 are minor and likewise recorded there. None of them changes a
future action of mine in a way that is not already in the verdict doc, and none is a preference of
Mr. Zhang's — so per "三条全满足才记;存疑不记", zero entries.
