# Delivery: D5 Phase 2 Lane C Part 2 — S3 extraction / S4 layering / S5 storage (seatloom 16 sessions)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-10-washing-pipeline-p2-delivery-v1 |
| status | delivered |
| author | vela |
| date | 2026-10-10 |
| version | v1 |
| depends_on | VELA-2026-10-10-washing-pipeline-v1, VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1, VELA-2026-10-10-washing-pipeline-p1-delivery-v1 |
| tags | washing, extraction, typed-records, layering, storage, seatloom, d5-phase2 |

Dispatch `disp-wash-vela-p2`. The hard order was honored: both gate events
(`ev-018-memory-hardening-landed`, `ev-typed-memory-tools-landed`) existed
before this dispatch was pulled, and Part 1 was already delivered.

**S5 could not execute — the sanctioned write path does not exist in this
fleet yet. Full evidence in §4. S3 and S4 are complete and staged; the insert
is one command away once the write path lands.**

## 1. What was built

`tools/wash/` additions (on `packet/vela/washing-pipeline`):

| File | Role |
|---|---|
| `vela_wash/s3_extract.py` | record shape per the 018 value-domain contract (kind/confidence/L2), digest idempotency, per-session sha256 checkpoints, topic renderer |
| `vela_wash/s5_store.py` | S4 layering validation, DEEP cycle detection on the projects tree, the staged idempotent insert plan + checkpoint state |
| `author_*.py` (9 files) | the extraction itself: one authoring script per session, each record hand-authored from the washed stream with its L2 line range |
| `.local/wash/s3/records.jsonl` | the 88 typed records (scratch, gitignored — Part 2's product) |
| `.local/wash/s3/store_state.json` | insert checkpoint + the recorded S5 blocker |

## 2. S3 — extraction (the only LLM stage; run by the washing seat, zero external LLM calls)

**88 typed records** across 9 of the 16 sessions:

| kind | count | | confidence | count |
|---|---|---|---|---|
| decision | 42 | | quoted | 85 |
| failure | 26 | | inferred | 2 |
| fact | 14 | | | |
| preference | 3 | | | |
| unresolved | 2 | | | |

`failure` is the largest single-theme cluster after decisions, per the packet's
priority rule (empirically the most recurrent class in this corpus: dirty-tree
measurement errors, watcher-broken gates, ratchet granularity, pipe-pane
toggle, truncated dispatch, stale archive — each recorded with its fix).

Confidence honesty: 85 records are `quoted` (the transcript says it); only 2
are `inferred` (the UM0 closeout status and the two pre-existing DB defects'
resolution status — both explicitly marked because the corpus does not contain
their closing events).

Extraction method: per-session dialogue renders (message + reasoning turns,
tool calls condensed) were read in context and each record authored with its
evidence window as an L2 line range into the raw transcript. No raw text is
copied into any record — every record is a pointer plus a distillation.

**Coverage, stated honestly:** the 8 claude-code sessions were read
exhaustively (2,062 dialogue turns); the two large multi-day codex sessions
(6,167 dialogue turns) were **topic-sampled** — the highest-substance topic
windows (product review, frontend refactor direction, infra-only boundary,
storage/ledger foundation, Mira continuity failures) were read in full and
yielded 6 records; the four small codex sessions and the cursor session were
inspected and contain no substantive dialogue (system instructions and
environment context only; cursor is four "hi" turns); gemini is register-only
by packet. 12 of 482 topics contributed records. A full second pass over the
codex sessions is the obvious next increment if the operator wants deeper
coverage — it was not attempted in one turn because exhaustive in-context
reading of 6,167 turns was not feasible at the required care level, and
shallow extraction of the remainder would have produced exactly the
low-confidence filler the value-domain contract exists to prevent.

## 3. S4 — layering (validated, not assumed)

`validate_layering` enforces the contract on every record: L0 is one sentence
(the only default layer), L1 is decision-grade distillation, L2 is
`l2_ref_kind=file` + path + sha256 body digest + line range with
`end >= start`, and L1 is bounded so no record becomes a second copy of raw/.
It caught three real violations during authoring (L0s that had grown to two
sentences); those were tightened and the store now validates clean:
**88/88 records, zero problems**.

Samples (L0 verbatim, L1 truncated here):

- **decision** — "SeatLoom is local-first by definition, not by phase." L1:
  pty/mod.rs shells out to tmux directly and therefore only sees tmux on the
  backend host; the remote/8088 path was abandoned before building it because
  a browser gets isTauri()=false and falls back to mock data. L2:
  raw/claude-code/b0100cff-*.jsonl:853.
- **failure** — "A piped $? captures the wrong exit code - measure gate
  baselines before changing anything." L1: the packet claimed all four UI
  gates green at base; measured properly eslint had 151 problems and prettier
  96 unformatted files, and the earlier green reading came from capturing a
  pipe's tail exit code. L2: raw/claude-code/621ae382-*.jsonl:243-271.
- **unresolved** (inferred, honestly marked) — "UM0 live-mirror closeout state
  at corpus end." L1 states the HOLD reasons and why resolution status is
  inferred rather than quoted. L2: raw/claude-code/b0100cff-*.jsonl:6158-6170.

## 4. S5 — storage: blocked, with the blocker proven at execution time

The insert plan is built, validated and staged (`s5_store.plan()`): 88 records,
idempotent by content digest (record_id = sha256 over kind+title+L0+L2 range),
resumable by per-session sha256 checkpoint, projects-tree deep cycle detection
implemented and unit-proven (catches `a→b→c→a`, not only 1-cycles — 015 only
blocks self-cycles). Validation: **no cycles, layering clean, nothing blocking
except the write path itself.**

The write path was attempted through the only sanctioned channel (the
`remember` tool — row + event in one transaction) and **rejected**, twice over:

1. **The deployed MCP binary predates Lane A2.** `remember` rejects the
   018-mandated typed fields at validation: `l2_line_start`/`l2_line_end`:
   *schema is false*; `kind`/`confidence` likewise. The typed write signature
   this dispatch depends on does not exist in this fleet's deployed binary.
2. **The live database is not migrated.** Even the pre-018 signature fails
   (`postgres error`) because migrations 015-018 are not applied to live —
   still at the 014 state with no memories/topics tables.
3. **No topic-creation path exists in the tool surface.** The packet step
   "segments → topics" has no sanctioned writer at all (the MCP surface
   exposes remember/recall/promote/forget but nothing that creates a topic),
   and `remember` requires an existing `topic_id`.

Per the red lines, no raw SQL, no bypass files, no live-DB action was taken.
The failure is recorded verbatim in `.local/wash/s3/store_state.json` and
escalated (§6). When the write path lands, insertion is
`execute_plan(plan, writer=remember)` — idempotent, checkpointed, and it stops
on the first failure rather than skipping.

## 5. Acceptance gates 1-5 — evidence status

| # | Gate | Status |
|---|---|---|
| 1 | S2 topic list to the operator for eyeball judgment | Delivered in Part 1 (full table in the p1 delivery doc §5); **still awaiting operator judgment** — not self-approved |
| 2 | S1 hits on known-key samples | Proven in Part 1: 21/21 samples redact; independent grep audit of both washed streams finds 0 survivors |
| 3 | Pruning stats submitted | Part 1: 58,017 raw lines / 167.6 MiB → 25,156 LLM turns / 23.6 MiB (14.1% kept) |
| 4 | Sampled human read of L0/L1 + confidence honesty | The 88 records' L0/L1 are the artifact; 85 quoted / 2 inferred with the inference reason stated; layering machine-validated. Human read still owed by the operator/Flux |
| 5 | L2 backtracking, random 5 by path+line | **Pre-verified by this seat**: 5 random records, digests recomputed from raw/ — 5/5 exact, all pointers resolve (ranges are evidence windows; the digest covers the whole window so the check is recomputable). `test_l2_pointers` also structurally re-verifies sampled turns against raw/ |

## 6. Escalations (emit_event + Aegis, never self-approved)

- **S5 is blocked fleet-wide, not for this seat alone.** The deployed MCP
  binary predates the typed memory tools and the live DB predates the memory
  schema; Onyx's terminal acceptance-card failure and Mira's forward-recording
  failure are the same wall. Asks: apply 015-018 to live, redeploy the Lane-A2
  binary fleet-wide, and expose a topic-creation path (or seed topics) so
  "segments → topics" has a sanctioned writer.
- **Gate 1 remains open**: the topic list needs the operator's eyeball. The S2
  heuristic fragmentation on the two multi-day codex sessions (244/193 topics)
  is reported, not tuned away.
- **Coverage**: codex extraction is topic-sampled (12/482 topics contributed).
  A deeper second pass is available on request; shallow exhaustive extraction
  was declined deliberately.

## 7. Completion protocol

- Committed on `packet/vela/washing-pipeline` with the dispatch id in the
  message; no uncommitted changes at idle.
- Records dropped for missing L2 line numbers: **0** — every record carries a
  verified `src_file` + line range, and the 5 random gate-5 digests recompute
  exactly.
- Self-requesting Flux verification, commit-pinned, with the pre-declared
  verdict event id required by the carding automation.
  **`complete_dispatch(disp-wash-vela-p2)` is deliberately NOT called yet** —
  per the payload, the dispatch closes only after the acceptance card exists
  (Lyra's gated self-dispatch fires when Flux emits
  `ev-flux-verdict-wash-p2`).
