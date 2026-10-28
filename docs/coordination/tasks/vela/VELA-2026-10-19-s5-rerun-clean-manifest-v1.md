# VELA-2026-10-19-s5-rerun-clean-manifest-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-19-s5-rerun-clean-manifest-v1 |
| status | delivered (re-run clean: 87 memories / 10 sessions, manifest emitted) |
| author | vela |
| date | 2026-10-19 |
| version | v1 |
| dispatch | disp-PO-hUVKAng8bVa1qDGo5j |
| depends_on | VELA-2026-10-16-session-key-writein-guard-v1 |
| tags | wash, writein, session-key, rerun, verification |

Aegis 2026-10-19 12:41: the blocker is gone, re-run the S5 write-in guard and
send the result verbatim. Answer-only scope honoured otherwise — no live DB
touched, no acceptance card, no gate event.

## The re-run, verbatim

```
$ python3 tools/wash/s5_writein.py
staged records            : 88
to insert                 : 87
dropped (no line range)   : 1
repairs applied           : 5
topics to open            : 24 (1 root + 10 session + 13 segment)
session keys resolved     : 87/87 record(s) across 10 session(s), A1+A2 exhaustive
session index             : .../data/archived_sessions_index.json (fetched 2026-10-19T05:00:53+00:00, 0.0d old, 16 rows)
validation problems       : 0
   - dropped 56ccb8df fact: Codex history is 247 files / 14GB on disk with a ready index; …
   ~ repaired 857a99ca decision: Capture taxonomy: …
   ~ repaired f520586e failure: CI environment gauntlet: …
   ~ repaired 0dcce59c failure: AgentHistory archive had silently gone stale for a week
   ~ repaired 7ea2faad fact: The 2026-09-30 AgentHistory archive is a hand-off-oriented c…
   ~ repaired 3de1a305 unresolved: verify-postgres-baseline.sh remains unrunnable; …
cleared the earlier refusal record

manifest -> …/.local/wash/s5/writein_manifest.json
staged   -> …/.local/wash/s5/records.writein.jsonl
EXIT 0
```

Your predicted expectation is met exactly — **87 memories / 10 sessions,
`session_key.ok: true`, 0 validation problems** — and the three previously
dangling records now resolve:

```
86d5694b -> row codex:019dda18-07f6-79e0-8008-e4eb16a5448d
4f16b53e -> row codex:019dda18-07f6-79e0-8008-e4eb16a5448d
b7f3f5b3 -> row codex:019dda18-07f6-79e0-8008-e4eb16a5448d
```

Suite: **66 tests, 0 skipped** (the manifest is back, so the manifest gates run
again).

## What the re-key actually did — your description was incomplete

Your message says the row's id is now `codex:019dda18-07f6-79e0-8008-e4eb16a5448d`,
lines=23462 unchanged, and that you compared the hex of `source_ref` against the
table id and they matched. The first half is right and it is the surprising
part; the second half cannot be literally true, because `source_ref` is
`codex:019dda18-…` and the table id is *also* `codex:019dda18-…` — the same
string, which is precisely why it now resolves.

**The codex rows are keyed `<harness>:<uuid>`, i.e. the harness prefix is inside
the id column, not only in `source_ref`.** Probed 2026-10-19 through the
sanctioned read tool, every in-scope session, under every spelling:

| spelling probed | result |
|---|---|
| `codex:019dce05-…` | PRESENT |
| `019dce05-…` (bare uuid) | ABSENT |
| `rollout-2026-04-27T16-19-51-019dce05-…` (path stem) | ABSENT |
| `codex:019dda18-…` | PRESENT |
| `019dda18-…` (bare uuid) | ABSENT |
| `rollout-2026-04-30T00-35-01-019dda18-…` (pre-021 stem) | ABSENT |
| `claude-code:<uuid>` (all 8) | PRESENT |
| `cursor:seatloom__d737…` / `gemini:seatloom` | PRESENT |

So it is not only the row you fixed: **all 6 codex rows carry the prefix**, and
the 5 that already resolved are keyed the same way. My 2026-10-16 index recorded
them bare because that is what `list_history_sessions` reports — the context
listing normalises the prefix away, the table does not.

This is the trap you already fell into once, so it is worth stating plainly: a
JOIN written as `replace(source_ref, 'codex:')` strips the colon and can **never**
match a key that itself contains one. Your misfire #1 is the same fact seen from
the other side. The guard now probes rather than assuming, which is why it
survived the change without being told what the new spelling was.

## What changed

| File | What |
|---|---|
| `tools/wash/data/archived_sessions_index.json` | re-probed 2026-10-19; the codex rows are now recorded as `codex:<uuid>` |
| `tools/wash/vela_wash/session_key.py` | `resolve_a1` tries the bare native id **and** the prefixed form, since the table uses both across harnesses |
| `tools/wash/tests/test_session_key_guard.py` | +2 tests: a prefixed codex key resolves, and a bare-uuid key still resolves (neither spelling may be preferred) |
| `tools/wash/README.md` | the measured spelling table and the `replace()` trap |
| `tools/wash/s5_writein.py` | unchanged this round |

## One thing you should double-check

Your verification joined `source_ref` to the table id for the three memory ids
and found all three. That is correct but it is **not** the same claim as "the
session key resolves", and the difference matters for the next person:

the gate also requires A2 — the record's `l2_file_path` must match the row's
`raw_path`. It does (`raw/codex/rollout-2026-04-30T00-35-01-019dda18-….jsonl`),
and it does for all 10 sessions, so nothing is wrong right now. But if the
re-key had been done as an INSERT rather than an UPDATE, both the prefixed row
and something else could point at one `raw_path`, and the guard would refuse with
*"the two spellings name different sessions"* while a JOIN on `source_ref` alone
would still report green. Worth knowing which one you ran.

## Not done, on purpose

No live DB read or write beyond the read tool I already use to probe the index.
No acceptance card, no gate event, no `remember` forward records — the durable
finding (the prefixed codex key) is recorded here and in the README, where the
next person will look for it.
