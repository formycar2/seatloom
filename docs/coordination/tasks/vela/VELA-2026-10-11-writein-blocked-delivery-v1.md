# Delivery: night step 3 write-in — BLOCKED on live infrastructure, not attempted (staged and ready)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-11-writein-blocked-delivery-v1 |
| status | delivered |
| author | vela |
| date | 2026-10-11 |
| version | v1 |
| depends_on | VELA-2026-10-10-washing-pipeline-v1, VELA-2026-10-10-washing-pipeline-p1-delivery-v1, VELA-2026-10-10-washing-pipeline-p2-delivery-v1, VELA-2026-10-11-writein-guard-delivery-v1 |
| tags | washing, writein, blocked, stats, seatloom |

Dispatch `disp-onight-writein`. **The writes were not performed — this is an
environment block, not a work failure.** Everything that can be done without
a live write path is done, and the insert is one command away. This note is
written to be readable directly: what was distilled, what was lost, why, and
exactly what is waiting on what.

## 1. Why nothing was written (the short version)

Three independent walls, all re-proven from this process at close-out:

1. **This seat's tool subprocess is the old program.** It started
   2026-10-10 21:39 and holds that moment's image: it has **no `open_topic`
   tool at all**, and its `remember` has none of the 018-typed fields. The
   fleet-wide program swap (`scripts/land-knowledge-live.sh`) has not run, so
   no new tool is callable from here until it does.
2. **The live database is not migrated.** The running database's event-type
   vocabulary is still 27 values and does not contain `TopicOpened`, so
   `open_topic` would fail the database CHECK even if the tool existed here.
3. **The old write signature fails at the database** (`postgres error`) —
   the memories/topics tables are not on live.

Verbatim rejections (sanctioned paths attempted, never bypassed):

- `remember` (018-typed signature): `Error: Validation failed for tool
  "mcp__seatloom__remember": - l2_line_start: schema is false -
  l2_line_end: schema is false - root: must not have additional properties`
- `remember` (the only signature this binary accepts): `memory tool
  rejected: postgres error: db error`
- `open_topic`: absent from this process's tool surface
  (`searchTools("open_topic")` returns only `recall`).

Per both the guard's ordering and this dispatch's own step 1, no write was
attempted once the capability check failed. No raw SQL, no bypass files, no
live-DB action.

## 2. The three required stats

**Pruning (S2.5) — raw lines to lines actually sent to the model:**

| metric | value |
|---|---|
| raw transcript lines (16 sessions) | 58,017 |
| raw bytes | 175,729,835 (167.6 MiB) |
| canonical turns after normalization | 58,084 |
| turns actually sent to the model stage | 25,156 |
| bytes sent to the model stage | 24,726,186 (23.6 MiB) — **14.1% kept, 85.9% pruned** |
| tool_result bodies dropped | 43,270,117 bytes (replaced by tool name + ok + byte count + L2 line range) |
| meta rows dropped | 32,928 |

**Redaction (S1) — hits by type, total 1,026:**

| type | count |
|---|---|
| intranet_domain | 964 |
| stepplan_key65 | 56 |
| jwt_fragment | 2 |
| ak_api_key | 1 |
| basic_auth | 1 |
| session_token | 1 |
| high_entropy | 1 |

Independent grep audit of both washed streams after the run: **0 surviving
secret shapes** (the only `eyJ` grep hits are a coincidental substring inside
codex `call_id` provenance, not secrets). 21/21 known-key samples redact
(gate 2).

**Records dropped for no recoverable L2 line number: 0.** Every canonical
turn carries a verified `src_file` + `src_line`, and every one of the 88
authored records carries an L2 line range into `raw/` with a recomputable
sha256 body digest (5 random records pre-verified: 5/5 exact). Nothing was
dropped for untraceability because nothing was untraceable.

## 3. What was distilled, and what was lost (for Mr. Zhang, plainly)

- **Distilled:** 88 records — the decisions, failures, facts, preferences and
  open items that explain why this project works the way it does. The
  highest-value cluster is the failure class (26 records): the same mistakes
  kept recurring across seats (measuring a dirty working tree and calling it
  a committed fact; a piped exit code read as a green gate; a ratchet that
  fails on files you merely touched; a `pipe-pane -o` flag that is a toggle,
  not a guard). Those are now written down with their fixes.
- **Coverage, honestly:** all claude-code sessions were read exhaustively;
  the two multi-day codex sessions were topic-sampled — 12 of 482 topics
  contributed records. The April-era material is therefore represented by
  its highest-substance windows, not exhaustively. A deeper second pass over
  the codex sessions is the obvious next increment.
- **Lost, by design:** the raw transcripts are not copied anywhere — memories
  are pointers plus distillation, per the layering contract. Nothing else was
  lost: `raw/` is untouched and read-only, and every record points back into
  it.

## 4. What is staged, ready to insert

`tools/wash/.local/wash/s3/records.jsonl` — 88 records, validated:

- idempotent by content digest (re-run inserts nothing twice)
- resumable by per-session sha256 checkpoint
- deep projects-tree cycle detection, unit-proven against `a→b→c→a`
- layering validated 88/88 (L0 one sentence, L1 decision-grade, L2 complete)

When the live path exists, the insert is
`execute_plan(plan, writer=remember)` — one call, then the stats and gate
event (`ev-writein-landed`) this dispatch asks for. No re-extraction needed.

## 5. Disposition

Reported to Aegis via `SupervisorMessage` (verbatim rejections + the
infrastructure block + the staged state). The dispatch is closed as
**blocked-environment**, not as completed work, so the one-in-flight slot is
released rather than left hanging — a hanging dispatch is what made the
waker skip this seat for 17 hours, and that will not be repeated. Re-issue
(or let the restart patrol poke this seat) once
`scripts/land-knowledge-live.sh` has run and Onyx's 019 is applied to live;
the staged output turns into real rows immediately.

Scope unchanged: seatloom only, 16 sessions. No expansion without the
operator's explicit decision.
