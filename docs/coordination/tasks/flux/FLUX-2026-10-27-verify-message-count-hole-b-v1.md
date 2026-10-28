# Flux verdict — Vela's message_count hole-B investigation: the 89 NULL rows are the sentinel

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-message-count-hole-b-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | verification, read-only-report, sentinel-not-defect, adversarial, own-error, seat-flux |

Dispatch `disp-2o_BDY1tb3b92ZQwaYXEC-flux-verify`.
Subject: `disp-2o_BDY1tb3b92ZQwaYXEC` — Vela's investigation of the 89 NULL `message_count` rows, commit `129f6b2` (docs only, one new markdown file).

## Verdict: PASS — and the dispatch's own premise is wrong, which Vela was right to say

Vela was asked to drive `WHERE message_count IS NULL` from 89 toward 0. He did not perform
the write, and the reason he gives is correct. **Every one of his five claims reproduces
against live and against the shipped code.** But the more important finding is that the
dispatch's premise — that 89 NULLs means "the corpus migration was not applied" — is
contradicted by the gate event it is meant to satisfy. I checked that adversarially and
Vela's conclusion survives it.

## His five claims, each re-measured

| # | Claim | My measurement | |
|---|---|---|---|
| 1 | NULL is the loaded sentinel, not an unapplied migration | 021's live `COMMENT ON COLUMN` says exactly that | **confirmed** |
| 2 | Filling them would double-count every project | the guard is in the shipped SQL, with the measured cost written beside it | **confirmed** |
| 3 | 89 = codex 60 / stepcode 19 / gemini 9 / opencode 1 | my own CSV+disk implementation gives the same, and a stronger equivalence | **confirmed** |
| 4 | 0 of 89 fillable from data that exists today | 51+19 files on disk but never extracted; 9 absent; gemini marker-only; opencode absent | **confirmed** |
| 5 | Therefore the gate's criterion is wrong, not the data | the gate event already records the 89 as expected | **confirmed, and stronger than he states** |

### Claim 1 — the column's own definition, from the live database

I did not read the migration file; I read the comment off the live column, so this is the
definition the running database actually carries:

```
Messages in this session, counted from resume-contexts/by-session/<session>.md message
count header (021). Sum over the corpus is 36680. NULL = no extracted conversation
exists for this row — that is what distinguishes the 183-row corpus from the 271-row
archive inventory, so a project-grouped read must filter on this column being NOT NULL
to avoid double counting the 89 inventory-only rows. Never a transcript body: 011
forbids storing bodies and this is a count only.
```

Every element of his claim is in the column's own words, including the double-count
warning. And the stated sum is real: `sum(message_count) WHERE NOT NULL` returns
**36680** exactly.

### Claim 2 — the guard, and why I am not worried he missed one

This was the question he asked me to attack, so I looked for a second guard or a
double-count path he might have missed. The filter is at
`crates/seatloom-core/src/db/repositories.rs:970`, injected into the join:

```sql
LEFT JOIN archived_sessions a ON a.project = p.id AND a.message_count IS NOT NULL
```

and the comment above it records what happens without it, measured rather than
hypothesised: joining the inventory in made `sum(sessionCount)` 261 against a corpus of
183 and reported **58 sessions under `sf-code/storage` for a truth of 2**. The comment
also names the verdict that measured it. So the guard is not incidental — its absence has
a recorded, measured cost, and 78 of the 89 NULL rows carry a project slug that matches
one of the 38 project ids, which is the mechanism of the inflation.

I looked for a second grouped read and found the same filter at line 1018. I found no
unguarded path. **His claim 2 is true and he did not miss a guard.**

### Claim 3 — I implemented his rule myself, and got an equivalence he did not claim

His matching rule is his own, not 021's, and he flagged it as such. So I wrote my own
implementation from the rule as stated: a DB `raw_path` is in-corpus if it equals a
`SESSIONS-FLAT.csv` `file` column entry, or — for cursor dir-copies — is the directory
that file lives in. The result:

```
harness      msg_count  verdict n
claude-code  COUNTED    corpus  18
codex        COUNTED    corpus  150
codex        NULL       NOT     60
cursor       COUNTED    corpus  14
gemini       NULL       NOT     9
opencode      NULL       NOT     1
stepcode     COUNTED    corpus   1
stepcode     NULL       NOT     19

COUNTED & in-corpus     : 183   (expect 183)
COUNTED & NOT in-corpus : 0
NULL    & in-corpus     : 0     <- the equivalence
NULL    & NOT in-corpus : 89    (expect 89)
```

**The rule does not merely reproduce 89 — it makes `message_count IS NOT NULL` and
`in-corpus` the same predicate.** Zero rows fall on the diagonal. That is stronger than
his claim: it means the 89 are not an arbitrary set that happens to be NULL, they are
exactly and only the rows outside the 183-session corpus. No CSV file is claimed by more
than one DB row, so the rule is not double-counting either.

My first attempt at this was wrong and I am recording it, because it is the same trap he
warned about. I compared the CSV's absolute paths against the DB's `raw/`-relative paths
and got 0 matches; my "fix" of prefix-stripping over-matched and produced 79 NULL rows
inside the corpus. Both were my rule being wrong, not his. The faithful implementation
above is what reproduces.

### Claim 4 — scope decision, not extraction bug. Settled from the extractor's input

This was the question that could have collapsed his whole argument, so I checked it
rather than reasoning about it. `scripts/extract-session-context.sh` takes its input
from `--index` (a CSV, defaulting to `resume-contexts/INDEX.csv`) and iterates
`csv.DictReader(open(idx))`. So the extractor's universe **is** the CSV. The 70 files on
disk that were never extracted were never in that universe.

Measured: `resume-contexts/by-session/` holds **183** extracted `.md` files, and of the
70 NULL codex/stepcode rows whose raw file is on disk, **0** have an extracted `.md`. So
extraction did not fail on them — it was never pointed at them. That is a scope decision,
exactly as he says.

Per-harness causes, each checked on disk:

| harness | n | his cause | my check |
|---|---|---|---|
| codex | 60 | 51 raw files on disk, 9 absent | **51 present / 9 absent** — exact |
| stepcode | 19 | all 19 raw files on disk, none in the CSV | **19 present / 0 absent** — exact |
| gemini | 9 | register-only; `raw/gemini/<project>` holds only `history/` + `.project_root` | 9 projects, `find -type f` returns **10 files, all `.project_root`** — nothing to count |
| opencode | 1 | no `raw/opencode/` directory exists | `raw/opencode` does not exist — confirmed |

So "0 of 89 fillable from data that exists today" holds for 19 rows by impossibility, and
holds for the other 70 only under the current scope. That distinction is the honest one
and he draws it.

### Claim 5 — the gate event already says what he says it should say

This is the part he did not check, and it makes his conclusion stronger. The dispatch says
the gate `ev-history-sessions-read-live` "fails its own re-measurement because 89 of 272
rows are NULL". I read the gate event's own payload. Its `verified` field, written
2026-10-16, already records:

> Also measured on live directly: archived_sessions 272 rows total, 183 with message_count
> not null, **the 89 inventory-only rows still present and counted by no project-grouped
> read**; projects 38; ...

And its `conditions.a_fix` names Nimbus's filter as the fix. **The gate does not expect 0
NULLs — it expects the 89 to be present and counted by nothing.** So the failing
re-measurement is measuring a designed sentinel as a defect.

I went one step further and looked for anything in the tree that asserts 0 NULLs. The
proof script `crates/seatloom-mcp-emit/proof/history_read_path_proof.sh` asserts the
opposite — it *requires* the inventory to exist:

```bash
[ "$CORPUS" = "183" ] || { echo "FAIL: ${CORPUS} corpus rows, expected 183"; exit 1; }
[ "$INVENTORY" -gt 0 ] || { echo "FAIL: no inventory-only rows — this is the fresh shape wearing a live label"; exit 1; }
LEAK=$(psql "SELECT count(*) FROM archived_sessions WHERE message_count IS NULL AND project IN (SELECT id FROM projects);")
[ "$LEAK" -gt 0 ] || { echo "FAIL: no inventory row shares a project slug — the live shape is not loaded"; exit 1; }
```

**Driving the 89 to 0 would make the shipped proof FAIL.** So this is not merely
permissible-to-leave-alone: filling the 89 breaks a test that exists specifically to
catch a database where the inventory was lost. That is the strongest possible form of his
claim 2, and he did not have it.

## A correction to the dispatch's premise, stated plainly

The dispatch's summary says "NULL indicates the corpus migration was not applied, not a
fabricated zero, so these 89 rows read no content". The first half is wrong and it matters:
NULL indicates **no extracted conversation exists**, which is the column's own definition,
and 021 *is* applied — the corpus rows carry counts summing to 36680. What the 89 rows
"read" is correct behaviour, not missing data: they are the archive inventory, and a
project-grouped read must not count them.

The dispatch's own guardrail ("must not fill 0 to pass it off as real") is the right
instinct, and Vela followed it. But the framing that sent him looking for a hole to fill
is itself the error, and his delivery corrects it.

## Is "escalate instead" the right call?

Yes, and for a reason beyond his own red line. He cites AGENTS.md — live-DB changes
escalate to Aegis. That is sufficient. But there is a second reason that survives even if
that rule did not exist: **there is nothing to write.** His recommendation 3 says do not
write 0 and do not backfill `lines`/`bytes` as a proxy, and I confirmed why a proxy is
tempting — all 89 rows have non-NULL `lines` and `bytes`, so a plausible-looking number is
available for every one of them. That is exactly what makes the fabrication dangerous:
it would be indistinguishable from real data to any reader that does not know the
column's definition. The honest answer is that the value is unknown, and unknown is not
zero.

His recommendation 1 (restate the gate's criterion to 183 non-NULL + 89 NULL) is correct
and is the actual fix. His recommendation 2 (the 70 extractable rows are an operator scope
decision) is correctly sequenced: scope decision, then extraction pass, then follow-up
migration — three steps, none of them his.

## What I did not verify

- **I did not run the extraction pass** to confirm the 70 would extract cleanly. That
  would be a scope expansion I am not authorised to perform, and it is not needed to
  confirm his claim.
- **The 89 are reproducible from the CSV plus disk, but the CSV is a snapshot.** If
  `SESSIONS-FLAT.csv` changes, the equivalence I measured is a statement about today's
  CSV, not an invariant. The proof script's `INVENTORY > 0` assertion is the durable
  guard; my equivalence check is not.
- **I did not check whether the operator has acted on this yet.** The live DB still reads
  183 / 89 / 272, so as of now no write has happened.

## One unrelated observation, reported not acted on

Live `projects` now returns **40**, while the gate event and the proof both say 38. The
two extra rows are `networkOrchestrator` (2026-10-26) and `stepaicloud` (2026-10-27), both
created after the gate was written. This is not drift in the 89-row question and does not
affect any claim above — the proof measures a scratch database, not live — but it means
the gate event's "38 projects" is now stale as a description of live. Noting it so nobody
later reads 38 as current.

## Files

No files of mine changed for this request; the verdict and this note are on
`packet/flux/intent-verdict`.
