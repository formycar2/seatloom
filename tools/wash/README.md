# vela-wash — SeatLoom corpus washing pipeline (S0-S2.5 zero-LLM, S3 extraction, S5 write-in)

Seat (Vela) tooling for dispatch `disp-wash-vela-p1`, packet
`docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-v1.md` (Lane C,
D5 Phase 2). Scope is fixed by operator decision: the **seatloom project only**
(16 sessions / 58,015 lines / 2026-04-24 → 2026-10-08). Expanding scope is NOT
automatic.

## Stages

| Stage | What | Module |
|---|---|---|
| S0 | normalize per-harness transcripts into a canonical turn stream | `vela_wash/s0_normalize.py` |
| S1 | redact secrets — after normalization, **before any LLM or table write** | `vela_wash/s1_redact.py` |
| S2 | topic segmentation (two-level tree; self-built heuristic — the weakest link) | `vela_wash/s2_segment.py` |
| S2.5 | prune for the LLM stage (tool_result bodies, retry rounds, meta rows) | `vela_wash/s25_prune.py` |
| S3 | LLM extraction of typed records (kind / confidence / L2 range) | `author_*.py` + `vela_wash/s3_extract.py` |
| S4 | layering validation (L0 one sentence / L1 decision-grade / L2 complete) | `vela_wash/s5_store.py` |
| S5 | write-in driver: validate the whole write set, emit the insert manifest | `s5_writein.py` |
| S5-gate | the SESSION KEY: both spellings resolve against `archived_sessions`, exhaustively | `vela_wash/session_key.py` |

## Run

```bash
cd tools/wash
python3 -m vela_wash.run --out .local/wash        # all 16 sessions, ~20 s
python3 s5_writein.py                             # validate the write set, emit the manifest
python3 -m unittest tests.test_l2_pointers tests.test_s0_adapters \
                  tests.test_s1_gate2 tests.test_s2_segment \
                  tests.test_s5_writein tests.test_session_key_guard -v   # 66 tests
```

`python3 -m unittest discover -s tests` does NOT work (no `tests/__init__.py`);
name the modules explicitly as above.

## The session key — two spellings, and why A1 is primary

Every washed memory names its session twice, and `archived_sessions` is keyed
`PRIMARY KEY (harness, id)` (011), so the two names resolve differently:

| | Field | Shape | Resolves by |
|---|---|---|---|
| **A1** (primary) | `source_ref` | `<harness>:<native session id>` | the table's primary key |
| **A2** | `l2_file_path` | `<AgentHistory>/raw/<harness>/<file>` | the row's `raw_path` |

**A1 is primary** because it names the session's own identity — the native id the
transcript carries — and migration 021 re-keyed `archived_sessions.id` TO that
native id for 182 rows, so A1 tracks the table's key by construction. A2 does
not: it is derived from a filename, and the codex filename is
`rollout-<timestamp>-<uuid>`, so a path-derived id is only correct if you know to
strip the timestamp prefix. A2 already broke once, silently: codex references
resolved against no row at all and nothing complained.

Measured 2026-10-19 (`data/archived_sessions_index.json`), after Aegis re-keyed
the row this gate originally refused: **the codex rows are keyed
`codex:<uuid>` — the harness prefix is inside the id column, not only in
`source_ref`** — while the claude-code rows carry the bare native uuid and
cursor/gemini carry the full `<project>__<hash>` / directory name. So `a1_parse`
strips exactly one separator colon and `resolve_a1` tries both the bare native id
and the prefixed form. The trap: a JOIN that does `replace(source_ref, 'codex:')`
strips the colon and can never match a key that itself contains one — that
misfire produced a false negative in review, which is why the guard probes
instead of assuming.

### What the gate does

`s5_writein.py` refuses to emit a manifest unless **every** record in the write
set passes all of this — no sampling:

1. `source_ref` parses as `<harness>:<native session id>` (harness in 011's
   closed set);
2. that `(harness, id)` resolves against `archived_sessions`;
3. `l2_file_path` resolves against `archived_sessions` by `raw_path` — a raw file
   on disk is **not** a row in the table (271 files on disk vs 261 rows reachable
   through the read tools);
4. the two spellings name the **same** row — two rows for one `raw_path` (a
   re-key done as INSERT instead of UPDATE) is an ambiguity, not a choice;
5. `source_kind` is `archived_session`. An unknown kind fails closed: this gate
   has no resolution rule for it, and a silent skip is how the last one got in.

A dangling session key is invisible to the L2 gates: the anchors recompute
perfectly against a file that has no row behind it, so the session looks like it
produced nothing while every other check is green.

### The finding this gate made on its first run

`python3 s5_writein.py` refuses the current write set: **3 records**
(`86d5694b`, `4f16b53e`, `b7f3f5b3`) cite
`codex:019dda18-07f6-79e0-8008-e4eb16a5448d`, and no `archived_sessions` row
carries that key. The row for that raw path exists but is keyed by the pre-021
stem `rollout-2026-04-30T00-35-01-019dda18-…` and reports *"the historical corpus
is not loaded in this database — archived_sessions.message_count does not
exist"*, i.e. migration 021 re-keyed 182 rows and missed this one. Those 3
records are already live as memories, so the dangling key is already in the
table — reported to Aegis, not edited in place (015: a correction is
forget + remember, and rows are never hard-deleted or UPDATEed).

### Refreshing the archived-session index

The gate needs the live key space and this driver is offline and must not touch
the database (AGENTS.md: no psql). So it reads
`data/archived_sessions_index.json`, fetched through the seat's sanctioned read
tools — **PURE READ** `get_history_session_turns(harness, id,
contexts_dir=…/resume-contexts)`, cross-checked against
`list_history_sessions('seatloom')`. It records a row under **every** id spelling
that resolves, because the table does not use one consistently.

To refresh: probe each in-scope session under both spellings (the native id and
the raw-path stem — `index_probe_plan()` prints the list from the archive
manifest), keep the spellings that answer with a row rather than *"no archived
session (harness, id)"*, and write them with `write_index()`. The gate
**refuses to run** when the index is missing, malformed, or older than
`--max-index-age-days` (default 14): a guard that silently trusts a stale
snapshot is the same defect with extra steps. There is deliberately no
"skip the session-key check" flag.

## S5 write-in — read this before trusting a staged record

`s5_writein.py` is the gate between "staged" and "inserted". It re-validates every
record against `~/Documents/GitHub/AgentHistory/raw/` and **refuses to emit a manifest**
if anything is off, so a bad record can never reach `remember`. It found three real
defects that a sampled spot check had reported as clean:

1. **Anchors numbered against the live transcript, not `raw/`.** Two records cited
   lines past the end of their raw file; their `l2_body_digest` was `e3b0c442…`, the
   sha256 of an empty input — the signature of a range that read no lines. **A digest
   equal to sha256(b"") means the anchor is fake; check it explicitly.**
2. **Stale idempotency keys.** Three `record_id`s did not equal the content digest of
   their own fields, so a re-run would have inserted duplicates.
3. **A claim no single line range can carry.** One record's assertion spans four
   disjoint raw regions; it is dropped with a stated reason rather than force-anchored.

The driver also normalises `l2_file_path` to the `~`-rooted corpus path **before**
computing `record_id`, so the idempotency key is a function of exactly the bytes that
get written. (015/018 document the column as "repo-relative", but the corpus of record
lives outside this repo; a bare `raw/...` path resolves to nothing here.)

Outputs, in the gitignored scratch dir:

```
.local/wash/s5/writein_manifest.json     the insert plan: topics + memories + repair log
.local/wash/s5/records.writein.jsonl     the corrected records the manifest was built from
.local/wash/s5/writein_refusal.json      written instead, when the write set is refused
```

On refusal the driver also **deletes any manifest left by an earlier run** — a
stale plan sitting next to a refusal is the most dangerous artifact in that
directory, because it still looks executable. `tests/test_s5_writein.py` skips
while a refusal record is present, and says so instead of claiming nobody ran
the driver.

`tests/test_s5_writein.py` asserts these gates against the real manifest — **every** L2
pointer, not a sample — plus the accounting (`inserted + dropped == staged`, every drop
states why), the topic forest (one root, parents before children, no empty scope), and
the cycle detector's actual behaviour. Note `deep_cycle_detect` walks the *simple-name*
space (what 015's 1-cycle CHECK does not cover), not the full-path space.

`tests/test_session_key_guard.py` covers the session key: both spellings, every refusal
reason, and the driver itself refusing a non-resolving reference end-to-end on a
synthetic write set (so it needs neither the staged records nor the live table).

Read-only on `~/Documents/GitHub/AgentHistory/raw`. Artifacts land in the
scratch dir (`.local/wash`, gitignored) — Part 2 (S3/S5) consumes them:

```
.local/wash/<harness>__<session_id>/s0.canonical.jsonl   raw canonical stream (never to an LLM)
.local/wash/<harness>__<session_id>/s1.redacted.jsonl    redacted canonical stream
.local/wash/<harness>__<session_id>/s25.llm.jsonl        pruned LLM-facing stream
.local/wash/<harness>__<session_id>/stats.json           per-stage stats
.local/wash/stats.json                                   per-session + totals
.local/wash/topics.json / topics.md                      S2 topic tree (gate 1 eyeball list)
.local/wash/s1_report.json                               redaction counters + post-scan
```

## Canonical turn shape

Core spec: `{seq, ts, role, kind, text, tool, refs}` — plus provenance fields
mandated by the L2 red line and the S2 signals: `cwd`, `src_file`,
`src_line`, `meta`. `kind` ∈ message | tool_call | tool_result | reasoning |
event | meta.

## Adapter mapping policy

**codex** — `response_item` is canonical (the model's actual I/O); `event_msg`
is the UI mirror. Plaintext `agent_reasoning` is the only readable reasoning
(`response_item/reasoning` is encrypted) so it is kept; every other event_msg
row duplicates a response_item and is recorded as meta. `turn_context` carries
the cwd (S2 signal).

**claude-code** — user/assistant message rows are canonical; `attachment` rows
(hook outputs, 2,297 in the largest session) are meta. Sidechain rows are kept
(real sub-agent work).

**cursor** — reads the existing `chat_dump.json` (no sqlite).

**gemini** — register-only: one meta row (harness, session id, path).
`history/` holds `.project_root` only; extraction deferred. Must not block.

Unknown row shapes are recorded as meta and counted (`unknown_rows` /
`unparseable_rows`); no adapter ever fails a file.

## Redaction (S1) — calibration notes

Marker: `⟪REDACTED:<type>:<fingerprint>⟫`, fingerprint = sha256(secret)[:6].

Patterns: ak-* / sk-* / ghp_* / AKIA* / JWT (+ fragment) / PEM / Bearer /
Basic / 65-char stepplan key / cookie / session token / intranet domains /
high-entropy sweep. Decisions calibrated against THIS corpus (see module
docstring for measurements):

- `sk-`/`ak-` prefixes are also k8s object names here (`sk-db-baseline`,
  `sk-notification` — 900+ occurrences). The rule requires a key-shaped tail
  (≥16 chars; mixed-case, or dash-free with digits; entropy-gated). Real keys
  hit (`ak-39pa6bj25stc8…` in claude-code/b0100cff); k8s names survive.
- Pure-hex runs of any length are exempt: the only long hex strings in scope
  are 64-char sha256/Docker content digests — public hashes, like git SHAs.
- Packet-id-shaped names (`ROLE-YYYY-MM-DD-…`) are exempt: those names ARE the
  recall content and must survive washing.
- Intranet domains are replaced host-only (scheme + path survive).

Gate 2 (packet §3): `tests/known_keys.jsonl` holds 21 known-key samples across
every pattern; every one must redact with the right type and fingerprint, and
redaction must be idempotent. An independent grep audit of the washed output
finds zero surviving secret shapes (the only `eyJ` grep hits are a coincidental
substring inside codex `call_id` provenance, not secrets).

## L2 pointers (red line #2)

Every canonical turn carries `src_file` + `src_line` back into `raw/`.
`tests/test_l2_pointers.py` re-reads the raw files and structurally verifies
sampled turns resolve to their source rows (tool_use/tool_result ids, codex
call_ids, cursor array indices). Pruning (S2.5) never touches these pointers:
compact tool_result records keep `{tool, ok, bytes, l2:[file, line]}`.

**The sampling is the weak point.** The 2026-10-11 write-in found 3 bad anchors
that a 5-record random sample had reported as clean, and the prior delivery's
"0 records dropped for no line number" was an under-count produced by that
sampling. Any gate protecting a per-record invariant must be exhaustive over the
write set — see `s5_writein.py` and `tests/test_s5_writein.py`, which check all
of them.
