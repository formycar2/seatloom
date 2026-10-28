# VELA-2026-10-16-session-key-writein-guard-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-16-session-key-writein-guard-v1 |
| status | delivered (guard landed; write set refused on one dangling key — needs Aegis) |
| author | vela |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-session-key-writein-guard |
| depends_on | VELA-2026-10-11-writein-v2-delivery-v1 |
| tags | washing, writein, session-key, archived_sessions, gate, exhaustive |

Dispatch `disp-session-key-writein-guard` · seat `seat-vela-001` · branch
`packet/vela/washing-pipeline` · packet
`docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`
(§11.2 slice 3). **That packet is not on this branch** — the payload was the
whole task, and this note records what the payload asked for against what the
data actually says.

## What changed

| File | What |
|---|---|
| `tools/wash/vela_wash/session_key.py` | **new** — the session-key resolver: A1/A2 parsing, the live index loader (fail-closed on missing / malformed / stale), and the exhaustive validator |
| `tools/wash/data/archived_sessions_index.json` | **new** — the live `archived_sessions` key space for the 16 in-scope sessions, with provenance and `fetched_at` |
| `tools/wash/s5_writein.py` | wired the gate in; `--records/--topics/--out/--index/--max-index-age-days`; refusal record + stale-manifest removal |
| `tools/wash/tests/test_session_key_guard.py` | **new** — 26 tests: spellings, index loading, every refusal reason, and the driver refusing end-to-end |
| `tools/wash/tests/test_s5_writein.py` | the skip reason now says "the driver REFUSED (N problems)" when a refusal record exists, instead of implying nobody ran it |
| `tools/wash/README.md` | both spellings pinned, why A1 is primary, the gate's five rules, the refresh procedure, the finding |

## The gate

Exhaustive over the write set, never sampled (the README's own lesson: a
5-record random sample reported 3 bad L2 anchors as clean). Every record must:

1. parse `source_ref` as `<harness>:<native session id>` (harness in 011's closed set);
2. resolve that `(harness, id)` against `archived_sessions`;
3. resolve `l2_file_path` against `archived_sessions` **by `raw_path`** — a raw file on disk is not a row in the table;
4. have both spellings name the **same** row;
5. carry `source_kind='archived_session'` — an unknown kind fails closed rather than being skipped.

Anything else and the manifest is refused. On refusal the driver writes
`.local/wash/s5/writein_refusal.json` and **deletes any manifest left by an
earlier run**: a stale plan next to a refusal is the most dangerous artifact in
that directory, because it still looks executable.

Where the key space comes from: `data/archived_sessions_index.json`, fetched
through the seat's sanctioned **PURE READ** MCP tools
(`get_history_session_turns` per session, cross-checked against
`list_history_sessions('seatloom')`) — no psql, no direct DB access. The gate
refuses to run when that index is missing, malformed, or older than
`--max-index-age-days` (default 14). There is deliberately no "skip the
session-key check" flag: an escape hatch is how a guard becomes decoration.

## Verification

```
$ python3 s5_writein.py
staged records            : 88
to insert                 : 87
dropped (no line range)   : 1
repairs applied           : 5
topics to open            : 24 (1 root + 10 session + 13 segment)
session keys resolved     : 84/87 record(s) across 9 session(s), A1+A2 exhaustive
session index             : data/archived_sessions_index.json (fetched 2026-10-16T07:03:46+00:00, 0.0d old, 16 rows)
validation problems       : 3
   ! session-key: 86d5694b: session key does not resolve — A1 'codex:019dda18-07f6-79e0-8008-e4eb16a5448d'
     has no archived_sessions row; A2 does resolve — the row for raw/codex/rollout-2026-04-30T00-35-01-
     019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl is keyed 'rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d'
     (corpus NOT loaded by 021), i.e. the pre-021 filename stem, not the native id
   ! session-key: 4f16b53e: …same…
   ! session-key: b7f3f5b3: …same…
REFUSING to emit a manifest: the write set is not clean.
refusal record -> .local/wash/s5/writein_refusal.json     EXIT 1
```

```
$ python3 -m unittest tests.test_l2_pointers tests.test_s0_adapters \
      tests.test_s1_gate2 tests.test_s2_segment tests.test_s5_writein \
      tests.test_session_key_guard
Ran 64 tests — OK (skipped=9)
```

The 9 skips are `tests/test_s5_writein.py`'s manifest gates: with the write set
refused there is no current manifest to check, and each skip now says
*"the write-in driver REFUSED this write set (3 validation problem(s)) — read
.local/wash/s5/writein_refusal.json"*.

**No false positive.** With the one missing row simulated as re-keyed (021
UPDATEs the id, so the stem row is gone rather than duplicated), the same write
set emits a manifest cleanly — 87 memories / 10 sessions, `session_key.ok: true`,
0 validation problems. The guard's only objection to the current write set is
that one row. Verified with a throwaway index in a temp dir; the committed index
still records the truth.

## Pruning / redaction / dropped-record accounting

Not applicable — this dispatch adds a validation gate, it does not run the
washing pipeline. No new LLM stage, no new text sent anywhere, no new records
staged: 88 staged → 87 to insert → 1 dropped for no honest line range, exactly
as before. Redaction hits in stored text: **0** (the S5 secret-shape gate still
runs and still reports clean). Records dropped for want of line numbers: **0
new** (the pre-existing 1, `56ccb8df`, unchanged).

Rows already written: **untouched**. No `UPDATE`, no `forget`, no re-`remember`.
The 90 live `archived_session` memories are exactly as they were.

## 🔴 Needs Aegis — the gate found a live dangling key

**3 records already live in the database cite a session key that resolves
against nothing.** (87 of the 90 live `archived_session` references resolve;
these 3 do not.)

`86d5694b`, `4f16b53e`, `b7f3f5b3` (memory ids `mem-kyloXn9AY3D_YHA6WQk7h`,
`mem-Gih4q6JxjyyJmQJCHJmO8`, `mem-xqtBZmhQd1Iw5t732-tnk`) all carry
`source_ref = 'codex:019dda18-07f6-79e0-8008-e4eb16a5448d'`. Probed through the
seat's read tools:

```
get_history_session_turns('codex', '019dda18-07f6-79e0-8008-e4eb16a5448d')
  -> "no archived session (\"codex\", \"019dda18-07f6-79e0-8008-e4eb16a5448d\")
      — the archive is keyed by (harness, id)"
get_history_session_turns('codex', 'rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d')
  -> row PRESENT, but "the historical corpus is not loaded in \"this database\"
      — archived_sessions.message_count does not exist. Apply the corpus
      migration (021) first"
```

So the row for that raw path exists but is **keyed by the pre-021 filename
stem**, and it was not loaded by migration 021 — 021 re-keyed 182 rows and
missed this one. Every other in-scope codex session (5 of 6) is keyed by the
native id.

Consequences, in order of what they cost:

1. **The 3 live memories' `source_ref` dangles.** Anything joining memory →
   session returns nothing for them, which is precisely "the session looks like
   it produced nothing". The L2 anchors are fine — they recompute against
   `raw/codex/rollout-2026-04-30T00-35-01-019dda18-….jsonl` (23,462 lines) — so
   no other gate would ever have caught this.
2. **The write set is now refused**, so no new memory can be written until the
   key resolves. That is the specified behaviour, and it is correct: the
   alternative is writing more dangling keys.

Not fixed here, on purpose: editing the live DB is not this seat's job
(AGENTS.md puts every live-database change on the escalation path to Aegis), and
the 90 written rows must not be modified in place (015: a correction is
forget + remember, rows are never hard-deleted; an in-place `UPDATE` of
`source_ref` would rewrite history with no event).

**Suggested shape of the fix, for whoever owns the migration:** re-key that one
row (`UPDATE archived_sessions SET id = '019dda18-07f6-79e0-8008-e4eb16a5448d'
WHERE harness='codex' AND id='rollout-2026-04-30T00-35-01-019dda18-…'`) and load
it through the corpus migration like its 5 siblings. That makes all 3 dangling
references resolve **without touching any memory row** — the records are right,
the table key is wrong. Then `s5_writein.py` emits a manifest on the next run.
If the row is instead re-keyed by INSERT, both rows will exist for one
`raw_path` and the guard will refuse with "the two spellings name different
sessions" — by design, since that is an ambiguity, not a choice.

## Two things the payload got slightly wrong (reported, not silently absorbed)

1. **"A2 already broke once (before 021, 3 of 6 codex references resolved
   against no row, silently)."** Measured today, the surviving break is in the
   *other* direction: 5 of 6 codex rows are keyed by the native id (so A1
   resolves and the stem spelling does **not**), and 1 is keyed by the stem (so
   A2 resolves and A1 does **not**). A1 is still the right primary — it names the
   session's own identity and it is what 021 re-keyed *to* — but it is not
   universally correct today, which is why the gate requires **both** to resolve
   and to agree rather than trusting either.
2. **"migration 021 re-keyed 182 `archived_sessions` rows specifically so those
   references would resolve. The key is real and it works."** It works for 87 of
   the 90 live `archived_session` references. The other 3 do not, and they have
   been silently broken since the write-in landed. The dispatch's premise ("you
   are hardening a producer that already exists and runs today") holds — the
   producer runs — but "the key is real and it works" needed the qualifier.

   Counted from the ledger, not estimated: 90 `MemoryCreated` events in project
   `seatloom` carry `source_kind='archived_session'`, across exactly the 10
   sessions in the write set, and 3 of the 90 name the un-re-keyed session.
   (90 events vs the 87 rows in the current write set: a few memories were
   corrected via forget + remember, so the event count runs ahead of the row
   count.)

## Forward records

Written with `remember` (one per class, kept short, `source_kind=manual` so they
carry no L2 line range — they are operating records, not claims lifted from a
transcript):

| kind | id | record |
|---|---|---|
| failure | `mem-CRUsnyPuTgYeidBQghBD5` | 3 live memories cite an archived_session key that resolves against no row |
| fact | `mem-xJ3k69Lci1-1-Z7RJjy9y` | a raw file on disk is not a row in archived_sessions — the two disagree by 10 |
| decision | `mem-DzbMcEUFJQQj9yUiiO9Sw` | a per-record gate is exhaustive or it is not a gate |
| unresolved | `mem-nzjsQpZmmFoA30TWP9nat` | which spelling is A1 when the table uses both? Aegis to rule |

The A1-vs-A2 spelling discrepancy is the `unresolved` one because it is a
contract question, not something this seat should settle by editing prose.

Emitted `ArtifactCreated` **`ev-vela-session-key-writein-guard`** so the fleet can
see the guard land. Its payload states in as many words that **no dispatch or
acceptance is gated on it** — the 2026-10-12 lesson was that a gate-shaped event
id must be a `SupervisorMessage`, and an artifact notice must never be mistaken
for a gate.

## Notes for the next seat

* The contract packet referenced by the dispatch is not on this branch. If §3
  pins A1/A2 differently from the table above, the table wins — it is measured
  against the live table, and the code points at the measured shapes.
* `list_history_sessions` is **not** a read of the whole table: it lists only
  sessions with an extracted context (183 rows across 38 projects, while
  `list_history_projects` rolls up 261 and the on-disk archive has 271). Use
  `get_history_session_turns(harness, id)` as the existence oracle — it looks
  the row up by primary key with no project filter.
* `list_history_projects` and `list_history_sessions` disagree per project
  (seatloom: 16 vs 14; sf-code/storage: 58 vs 2). Do not use either as a row
  count.
