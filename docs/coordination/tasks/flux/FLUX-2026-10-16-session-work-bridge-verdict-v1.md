# Flux verdict — the session↔work bridge (Onyx's view 022 + Nimbus's read model), adversarially

- **Dispatch**: `disp-session-key-verify` (issued by seat-lyra-001). Edge
  `ev-session-key-readmodel-landed` measured **true** — the correct edge for a verifier.
- **Verdict event**: `ev-verdict-session-key` — **pre-declared id, emitted exactly as required**
  (`ReviewVerdictIssued`).
- **Verdict**: **FAIL** — one major defect. The contract's substance is right and I could not break
  the load-bearing wall where the read answers; but the read **panics** on exactly the failure mode
  the contract exists to catch, and the panic kills the process and an entire session's lineage with
  it.
- **Slices under test**:
  - Onyx's view: `022_session_work_bridge_view.sql` @ `0e32054` on `packet/onyx/topic-opened-019`
  - Nimbus's read model: `703c863` on `packet/nimbus/gate-vs-announcement` (branch tip `744030f`
    differs only by the delivery doc — `git diff --stat 703c863 744030f` = 1 file, 271 insertions)
- **Contract**: `LYRA-2026-10-16-session-work-bridge-contract-v1` (Lyra,
  `packet/lyra/milestone-summary` @ `8601167`). My acceptance criteria are §11.2 **slice 5**.
- **Where I worked**: my branch is `packet/flux/intent-verdict` and has none of this code, so I
  verified in Nimbus's worktree, **read-only** — nothing committed there, no tracked file touched.
  Behaviour was measured by driving the real `seatloom-mcp-emit` binary over stdio against a
  live-shaped scratch database built by Nimbus's own proof. The live `seatloom` database was touched
  only by read-only `SELECT`s.

---

## §14 captures (COORDINATION_RULES §14)

```
$ git -C <flux worktree> rev-parse --abbrev-ref HEAD  -> packet/flux/intent-verdict
$ git -C <flux worktree> rev-parse HEAD               -> d1099c1
$ git -C <flux worktree> status --short               -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
$ git -C <nimbus worktree> rev-parse HEAD             -> 744030f
$ git -C <nimbus worktree> status --short             -> M one delivery doc (pre-existing), ?? .scratch/*, .seatloom/rollout-ack.txt (tracked tree clean apart from that doc)
```

---

## 0. Gates, run by me at the pin before any behavioural claim

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | **clean** |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | **clean** |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | **clean** |
| `cargo test -p seatloom-core` | lib **183 passed / 0 failed / 2 ignored** — 9 of them this slice |
| `cargo test -p seatloom-mcp-emit` | **90 passed / 0 failed / 22 ignored** — 6 of them this slice |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | **16 passed / 0 failed** |
| `cargo check --workspace --all-targets` | **clean** |
| Nimbus's own `proof/session_work_readmodel_proof.sh` | **ALL POINTS PASS** (8 points, live-shaped scratch) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **RED** — see F5 |

Every gate number in Nimbus's delivery note reproduces exactly. The proof is honest and it passes.
Everything below is **outside** its eight points — the scratch it builds has no `ref_only`, `none`,
`disagree` or ambiguous-path row, so those states are constructed in my attacks.

---

## 1. What PASSES — the contract's substance holds wherever the read answers

### 1.1 The load-bearing wall holds

Measured on the live-shaped scratch, three sessions side by side:

```
(a) a corpus session with NO records        numbers={total:0,  resolved:0, unresolved:0}  records=0  session=null
(b) a session whose records do NOT resolve  numbers={total:3,  resolved:0, unresolved:3}  records=3  session=row
(c) a session with a MIXED record           numbers={total:13, resolved:12,unresolved:1} records=13 session=row
```

- (a) carries the note *"no distilled records are attributed to this session. Two different reasons
  produce this state and they are not merged: the session was outside the washing pipeline's scope,
  or it was washed and contributed no qualifying record. Nothing here guesses which."*
- (b) carries *"3 of 3 records attributed to this session do not fully resolve: at least one of their
  two recorded keys points at no row. Each one names which key and the exact value that failed. This
  state is deliberately distinct from 'no records'."*
- (c) shows **both halves at once**, so a reader cannot collapse the two states by looking at one
  number.
- I tested the wall from every side: `records == []` **iff** `numbers.total == 0`, for all three
  sessions; no empty list ever accompanies a non-zero total; no non-empty list ever accompanies
  `total == 0`; and `(a).numbers != (b).numbers` on the wire.

**This is the contract's §7.2 requirement and it is met.**

### 1.2 A misspelled `source_ref` is reported, and names what is missing

The 3 live `path_only` memories each report:

```
mem-Gih4q6JxjyyJmQJCHJmO8  path_only  resolvedBy=[a1_declared_path]
  missing: the recorded session reference "codex" + "019dda18-07f6-79e0-8008-e4eb16a5448d"
           resolves against no archived_sessions row
```

The drift number (**3**) rides along on every answer, together with the join_state breakdown across
the whole view (`both_agree=87, path_only=3`), so a session-local read cannot hide a corpus-wide
break.

### 1.3 A topic id is never a key

```
session_work {"harness":"claude-code","id":"wash-seg-cc-01565715-t000"}
  -> REFUSED: no archived session ("claude-code", "wash-seg-cc-01565715-t000") exists — the archive
     is keyed by (harness, id), and this session is not in it.
session_work {"harness":"claude-code","id":"wash-sess-cc-621ae382"}   (the form the contract names)
  -> REFUSED, the same way
memory_session {"memory_id":"wash-seg-cc-01565715-t000"}  -> null
```

And the topic id rides along as context with a note that says so: *"resolved by key, never by topic
id. The topic id "aegis-gate-rulings" rides along as context; a topic is a segmentation — a
revisable label — not an identity."* There is no field on either shape that joins on a topic.

### 1.4 An empty topic is unreachable — correctly

10 of the scratch's topics hold no memories. Because a topic is never a key, an empty topic produces
no answer rather than a wrong one. That is the right behaviour, not a gap.

### 1.5 An unknown session, and half a key, are refusals that say what they are

```
session_work {"harness":"codex","id":"does-not-exist-at-all"}
  -> REFUSED: no archived session ("codex", "does-not-exist-at-all") exists — … That is a different
     fact from a session that IS in the archive and produced no records.
session_work {"harness":"","id":"x"}            -> REFUSED: … one of them is empty …
session_work {"harness":"codex","id":"   "}     -> REFUSED: … one of them is empty …
```

The refusal explicitly separates "not in the archive" from "in the archive and produced no records",
which is the distinction the whole read is built on.

### 1.6 The fan-out guard works

With a second `archived_sessions` row declaring the same `raw_path` (one cursor directory really is
declared by two sessions on live — 272 rows, 271 distinct paths):

```
joinState: ref_only   resolvedBy: [a2_session_ref]
missing: the corpus path "raw/claude-code/01565715-….jsonl" is declared by 2 archived_sessions
         rows, so it is ambiguous — counted, never resolved by picking one
view rows: 91 for 91 memories — the memory is NOT double-counted
```

A plain join on `raw_path` would have fanned that memory into two rows and inflated the load-bearing
number. It does not.

### 1.7 The `disagree` state works

A memory whose PATH resolves to one session and whose REFERENCE resolves to another:

```
joinState: disagree   resolvedBy: [a1_declared_path, a2_session_ref]   session shown: A1's row
...and it is attributed to BOTH sessions, each counting it unresolved:
  (claude-code, 01565715-…) -> {total:14, resolved:13, unresolved:1}   1 disagree row
  (codex, 019dce05-…)       -> {total:3,  resolved:2,  unresolved:1}   1 disagree row
```

Contradiction is reported, never settled by picking one.

### 1.8 Malformed references are named precisely

| constructed `source_ref` | reported `missing` |
|---|---|
| `no-colon-at-all` | `the recorded session reference "<none>" has no colon, so it names neither a harness nor an id` |
| `claude-code:` | `the recorded session reference "claude-code" + "" resolves against no archived_sessions row` |
| `stepcode:claude:<uuid>` | `the recorded session reference "stepcode" + "claude:<uuid>" resolves against no archived_sessions row` — the wrapped id survives **whole**, as 022's split-on-first-colon intends |

### 1.9 Read-only, and one implementation

Measured, not asserted: `events/memories/archived_sessions/topics` unchanged before and after six
calls including the refusal cases. One implementation: `cmd_session_work` / `cmd_memory_session`
shell out to the same two tools (`session_work_cmds.rs:88,106`), the MCP arms call
`session_work::session_work(db, a)` / `::memory_session(db, a)` (`main.rs:212,218`), and **no second
SQL against `v_memory_session_join` exists anywhere in `src-tauri/` or `ui/`**.

### 1.10 No transcript bodies

The record shape is exactly `{memoryId, title, kind, confidence, status, sourcePath, lineStart,
lineEnd, joinState, resolvedBy, missing, topicId}` — pointers, a line range, and which key. No field
can hold a body.

---

## 2. F1 — MAJOR: the read **panics** on "the path pointer is wrong", the exact silent break the contract exists to catch

### The mechanism, at file:line

022 computes the A1 verdict as:

```sql
(a1.path_rows = 1) AS a1_resolves
```

`a1` is a `LEFT JOIN LATERAL … LIMIT 1`. When **no** `archived_sessions` row declares the memory's
corpus path, the lateral produces no row, so `a1.path_rows` is `NULL`, and `NULL = 1` is `NULL` —
**`a1_resolves` is NULL, not false**. (`a2_resolves` is `(a2.id IS NOT NULL)` and so is never NULL;
measured: 0 rows with a NULL `a2_resolves`.)

The Rust reader declares the field as a plain `bool`:

```rust
// crates/seatloom-core/src/db/models.rs:718
pub a1_resolves: bool,
```

so `r.get(15)` in `row_to_memory_session_join` (`crates/seatloom-core/src/db/repositories.rs:3288`)
fails to deserialize the NULL and **panics**.

### The measurement

```
input : memory_session {"memory_id":"mem-flux-panic"}     # l2_file_path = raw/claude-code/no-such-session.jsonl
stderr: thread 'main' (23605059) panicked at crates/seatloom-core/src/db/repositories.rs:3288:24:
        error retrieving column 15: error deserializing column 15
stdout: 1 line — the initialize response ONLY. The tool call receives NO RESPONSE AT ALL.
```

The view's own `join_state` for that row is **correct** (`ref_only`) — the SQL is right, the reader is
wrong. So the fix is local and small; the defect is that it was never exercised.

### It is not contained

Two calls in **one** process:

```
call 1  memory_session {"memory_id":"mem-flux-refonly"}   -> panics, no response
call 2  memory_session {"memory_id":"mem-Q0J4SeWMxAFhxyY5_T6de"}   (a perfectly good memory)
        -> NEVER RUNS. stdout holds only the initialize response.
```

The panic kills the MCP server. So one broken path key takes down **every tool in that process**, and
the Tauri command's stdio shell-out receives nothing to render.

### It is not one memory

The **forward** read of a session that merely *owns* one such memory panics too:

```
input : session_work {"harness":"claude-code","id":"01565715-3e7f-48c8-9ff0-fdd8edbfdf9f"}
        (a session with 14 records, 13 of them fine)
output: error retrieving column 15: error deserializing column 15   — no response
```

One broken path makes an entire session's lineage unreadable.

### The exact boundary

| A1 state | `a1_path_rows` | `a1_resolves` | the read |
|---|---|---|---|
| exactly one row declares the path | 1 | true | answers |
| **two or more** rows declare it | 2 | false | answers (`ref_only`, ambiguity named) |
| **zero** rows declare it | NULL | **NULL** | **PANICS** |
| the path is neither the `~`-rooted nor the bare form | NULL | **NULL** | **PANICS** |

So `ref_only` and `none` — two of the view's five join states, and the two that mean *the pointer
itself is wrong* — are reachable without panicking **only** through the ambiguity route. The
straightforward "the pointer names no row" route kills the read.

### Reachability, stated honestly

Measured read-only on live:

```
archived_session memories                                      90
a1_corpus_path IS NULL (neither accepted form)                  0
a1_corpus_path matches NO archived_sessions row (the panic case) 0
a1_corpus_path matches MORE THAN ONE row (ambiguity)             0
```

**On live today, nothing hits it.** So this is a landmine, not a current outage, and I say so
plainly. But it sits exactly where the contract's load-bearing wall is:

- `s5_writein.py` writes `l2_file_path` as `~/Documents/GitHub/AgentHistory/<corpus-relative path>`.
  A repository move, a different home, a different producer prefix, or **one typo in one path** turns
  every read into a panic.
- The contract's stated purpose (§1, §11.2 slice 1) is to catch a key that resolves against nothing
  and *report* it. This read **dies** on it instead.
- 022's own header says the number exists because *"3 of 6 codex references resolved against nothing
  and nothing in the product reported it"*. The A2 half of that break is reported beautifully. The
  A1 half takes the product down.

### What would fix it

Make the field `Option<bool>` (or express the view's verdict as `NOT a1.path_rows IS DISTINCT FROM
1` / `COALESCE(a1.path_rows = 1, false)`), then add a regression test that reads a row whose path
resolves against zero rows — a test the current suite cannot have, because the scratch has no such
row.

---

## 3. F2 — MINOR: the read never checks the disk, so "the source file is gone" is never said

The dispatch's attack list names this case: *"a session whose source file is no longer on disk (show
the record, say the file is missing; do not hide the record)"*.

Measured, with `HOME` pointed at a directory where `~/Documents/GitHub/AgentHistory/…` provably does
not exist, on a memory whose path **does** resolve (so no panic):

```
joinState: both_agree   resolvedBy: [a1_declared_path, a2_session_ref]   missing: []
notes:   [ "resolved by key, never by topic id. …" ]
does ANY part of the answer mention the disk, or that the file is missing?  False
```

The record **is** shown, not hidden — that half of the criterion is met. But nothing says the file is
gone, and there is no disk I/O anywhere in the read path (grep for `is_file` / `Path::new` /
`std::fs` across `session_work.rs` in both crates: no hits). The sibling read
`read_history_session_turns` does compute `source_present = Path::new(&source_path).is_file()` and
reports it — the pattern exists in this codebase and this read does not use it.

## 4. F3 — MINOR: a session with no records answers `session: null` although the row exists

`session_work` checks `archived_session_exists` first, so the caller has already been told the
session is in the archive. But the answer's `session` field is derived from `rows.first()`
(`session_work.rs`), so with zero records it is `null`:

```
(a) numbers={total:0,…}  records=0  session=null
```

The struct's own doc says *"The session's own row facts, when the session exists"* — which the
implementation does not deliver for the zero-records case. A reader could take `session: null` for
"there is no such session", which is the exact distinction the rest of the read works hard to keep.

## 5. F4 — MINOR: the session-level note is imprecise for the ambiguity case

The note reads *"at least one of their two recorded keys points at no row"*. For an ambiguous A1 the
key points at **two** rows. The per-record `missing` text is exact (*"declared by 2 archived_sessions
rows, so it is ambiguous — counted, never resolved by picking one"*); only the session-level sentence
is loose.

## 6. F5 — the workspace clippy gate is red, outside both slices under test

`cargo clippy --workspace --all-targets -- -D warnings` fails with two errors, both in
`src-tauri/src/commands/acceptance_cmds.rs`:

```
error: empty line after outer attribute          --> src-tauri/src/commands/acceptance_cmds.rs:25:1
error: this block may be rewritten with the `?` operator --> src-tauri/src/commands/acceptance_cmds.rs:83:5
```

Both come from `e5dbea8` (the merge-criterion fix), which is an ancestor of the read-model commit but
is **not** one of the two slices under test, and the tracked file is clean. Nimbus's delivery note
claims per-crate clippy for `seatloom-core` and `seatloom-mcp-emit` and `cargo check` (not clippy)
for `seatloom-tauri`, so this is **not a false claim** — but a reader running the workspace gate gets
red, and it should be fixed before this branch is anyone's integration base.

---

## 7. What I could not do, stated plainly

- **I did not run the Tauri app.** `cmd_session_work` / `cmd_memory_session` were read and their
  stdio shell-out verified by grep; no window was opened.
- **The panic is latent on live** — 0 of 90 live memories hit it today. I demonstrated it by
  constructing rows on a disposable scratch. A reader who wants to reproduce it must construct the
  same row; the live-shaped scratch alone will not show it.
- **I did not apply 022 to live.** The `-live` gate is deferred for exactly that reason and emitting
  it is Nimbus's act, not mine. All live access was read-only `SELECT`.
