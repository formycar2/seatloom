# VELA-2026-10-26-message-count-hole-b-not-mine-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-26-message-count-hole-b-not-mine-v1 |
| status | delivered (investigated + escalated; the write itself NOT performed — it is a live-DB change) |
| author | vela |
| date | 2026-10-26 |
| version | v1 |
| dispatch | disp-2o_BDY1tb3b92ZQwaYXEC |
| depends_on | VELA-2026-10-19-s5-rerun-clean-manifest-v1 |
| tags | wash, message_count, archived_sessions, escalation, red-line, measurement |

Aegis 2026-10-26: gate `ev-history-sessions-read-live` fails its own
re-measurement because 89 of 272 `archived_sessions` rows have
`message_count IS NULL`, and the dispatch asks for the count to be driven from 89
toward 0.

**I did not perform the write, and this note explains why and hands back
everything needed to do it correctly.** The short version: the 89 rows are not a
hole that can be filled. Filling them is arithmetically impossible, because
`message_count` is defined as "messages counted from an extracted
conversation", and for those 89 rows no extracted conversation exists. Writing 89
numbers there would be fabrication — the exact thing the dispatch itself forbids
("no filling in 0 to pass it off as real").

## Why I stopped: this is a live-database change

AGENTS.md, the seat's own red line: *"changing the live database (creating tables,
running migrations) is never this seat's job — escalate to Aegis"*. The dispatch asks for rows in the live `archived_sessions` table to
be updated. That is the migration/DDL class of action, so it escalates rather
than proceeds. I did read-only investigation to make the escalation actionable
instead of a refusal.

## What `message_count` actually means — from 021's own words

`infra/postgres/schema/021_history_sessions_corpus.sql` (on the main checkout,
not on my branch) defines the column. The comment below is quoted with its one
Chinese header label rendered in English (`message count`); the semantics are
021's, not mine:

```sql
COMMENT ON COLUMN archived_sessions.message_count IS
  'Messages in this session, counted from resume-contexts/by-session/<session>.md
   message count header (021). Sum over the corpus is 36680. NULL = no extracted
   conversation exists for this row — that is what distinguishes the 183-row
   corpus from the 271-row archive inventory, so a project-grouped read must
   filter on this column being NOT NULL to avoid double counting the 89
   inventory-only rows. Never a transcript body: 011 forbids storing bodies and
   this is a count only.';
```

So NULL is not "the migration did not run". NULL is the column's **loaded
sentinel**, and it is load-bearing: it is what makes `COUNT(*) WHERE
message_count IS NOT NULL` equal 183 instead of 272. The 89 are the archive
*inventory*; the 183 are the *corpus*. Any fix that drives NULL to 0 destroys
that distinction and double-counts every project.

## I reproduced Aegis's 89 independently, from disk

Not taken on trust. Corpus membership is defined by
`resume-contexts/SESSIONS-FLAT.csv` (183 rows, each naming its source file); a
manifest row is in-corpus if its `raw_path` matches a CSV `file`, or — for
cursor dir-copies — is the directory that file lives in.

| harness | in corpus | not in corpus |
|---|---|---|
| claude-code | 18 | 0 |
| codex | 150 | 60 |
| cursor | 13 | 0 |
| gemini | 0 | 9 |
| stepcode | 1 | 19 |
| opencode | 0 | 1 |
| **total** | **182** | **89** |

**Aegis's live measurement: 89 = codex 60 / stepcode 19 / gemini 9 / opencode 1.
Mine: 89 = codex 60 / stepcode 19 / gemini 9 / opencode 1. Exact match**, derived
from the CSV plus the on-disk archive with no database read at all.

## Why each of the 89 was NULL, per harness

| harness | n | why NULL | could it ever be filled? |
|---|---|---|---|
| codex | 60 | no extracted conversation. Of these, **51 have their raw `.jsonl` on disk and 9 do not**. The 9 absent ones were archived then pruned; the 51 present ones were never extracted because they fall outside the 183-session corpus scope. | the 51 present ones *could* be extracted and then counted honestly. That is a scope expansion, not a bug fix. |
| stepcode | 19 | all 19 raw files are on disk; none is in the corpus CSV. | same — extraction is a scope decision. |
| gemini | 9 | **AGENTS.md says gemini is register-only this round.** `raw/gemini/<project>` holds only `history/` + `.project_root`; there is no transcript to count. | no. Nothing to count. |
| opencode | 1 | no `raw/opencode/` directory exists at all. The manifest row points at a path that is not on disk. | no. |

The honest summary: **0 of the 89 can be filled from data that exists today.**
The 70 with files on disk (51 codex + 19 stepcode) could be counted only after a
new extraction pass, which is the operator's scope decision — and my AGENTS.md
records that washing scope has been fixed since 2026-10-16 and is not mine to
widen.

## What this means for the gate, stated plainly

The gate `ev-history-sessions-read-live` fails its re-measurement because it
expects 0 NULLs. It should not. The correct expectation is:

```
message_count IS NOT NULL  -> 183 rows  (the corpus, sum 36680)
message_count IS NULL      -> 89 rows   (the inventory, never counted)
total                      -> 272
```

The gate's *condition* was already recorded correctly by Nimbus's fix — "a
project-grouped read filters on `message_count IS NOT NULL`" — which only works
if the 89 stay NULL. The failing re-measurement is measuring the sentinel as if
it were a defect. **The gate does not need 89 rows filled; it needs its success
criterion restated** so that NULL-by-design reads as designed rather than as
missing.

## Recommended resolution (Aegis / operator, not this seat)

1. Restate the gate's criterion: expect 183 non-NULL and 89 NULL, not 0 NULL.
   The 89 are enumerated above with per-harness reasons, so the number is
   auditable rather than a shrug.
2. If the operator wants those 70 extractable rows (51 codex + 19 stepcode)
   counted, that is a **scope expansion** requiring his decision, then an
   extraction pass, then a follow-up migration. It is three steps and none of
   them is mine.
3. Do **not** write 0, and do not backfill `lines`/`bytes` into `message_count`
   as a proxy. Both are fabrications that happen to look like numbers.

## What I did and did not do

Did: read-only investigation against the on-disk archive, the manifest, the
corpus CSV and 021's own definition; reproduced the 89 and the per-harness split
independently; enumerated the per-harness causes and the file-present/absent
split; located 021 on the main checkout to get the column's exact semantics.

Did not: touch the live database in any way; write, run or draft a migration;
modify any of the 272 rows; fill anything with 0; open an acceptance card; emit
a gate event; widen the washing scope.

## Before/after — not applicable, and why that is the honest answer

The dispatch asks for 3 rows' before/after. There is no "after", because no row
was changed. The before, for the whole table, is the table above: 183 non-NULL,
89 NULL, 272 total — and I verified it is unchanged by everything I did, since I
changed nothing.
