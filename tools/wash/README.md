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

## Run

```bash
cd tools/wash
python3 -m vela_wash.run --out .local/wash        # all 16 sessions, ~20 s
python3 s5_writein.py                             # validate the write set, emit the manifest
python3 -m unittest tests.test_l2_pointers tests.test_s0_adapters \
                  tests.test_s1_gate2 tests.test_s2_segment \
                  tests.test_s5_writein -v          # 37 tests
```

`python3 -m unittest discover -s tests` does NOT work (no `tests/__init__.py`);
name the modules explicitly as above.

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
```

`tests/test_s5_writein.py` asserts these gates against the real manifest — **every** L2
pointer, not a sample — plus the accounting (`inserted + dropped == staged`, every drop
states why), the topic forest (one root, parents before children, no empty scope), and
the cycle detector's actual behaviour. Note `deep_cycle_detect` walks the *simple-name*
space (what 015's 1-cycle CHECK does not cover), not the full-path space.

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
