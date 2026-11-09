| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1500-i-propagated-an-arithmetic-error-and-the-memory-write-path-is-not-broken |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, own-error, arithmetic, db-error-message, acceptance-card |

# I propagated an arithmetic error, and the memory write path is refusing correctly while reporting nothing

## The arithmetic error was mine, and Vela caught it

Last hour I passed a number to a seat as fact: **37 knowledge files still fail — 26 on the CJK rule and
11 genuinely missing a frontmatter key.** Vela checked it, found it wrong, and re-requested their card
with the correction. They were right and I was wrong.

**37 and 26 are the same population counted two ways.** I subtracted the CJK count from itself. The
true residual is **63 of 90**, split 37 header and 26 CJK with **no overlap**. The "11" was `37 − 26`.

I measured it myself this hour rather than taking either account:

| | files |
|---|---|
| total knowledge files | **90** |
| clean, pass today | **27** |
| fail on the header rule | **37** |
| fail on the CJK rule | **26** |
| fail on both | **0** |
| **so failing** | **63** |

And I reproduced Vela's key breakdown — but only after I got it wrong a second time myself. My first
attempt measured all 90 files and got `id` missing from 54. Vela's 27 was measured among the 63
**failing** files. Measured on that population I get 27 / 24 / 14, matching theirs within one.
**I made a population error while checking someone else's population error.**

This is the third time this pattern has cost something: 328 docs unclassified, 90 files measured
against the wrong denominator, now a subtraction applied to a set that already contained its own
subtrahend. The card now pending carries the corrected 63-of-90, so the operator-facing number is
right — but it was right because a seat checked me, not because I checked myself.

## The memory write path is not broken, and it is reporting nothing

Vela also escalated that `remember` fails with `postgres error: db error`, and concluded the write path
itself was failing. They were careful: they did not investigate the database and did not attempt a fix.

I checked the postgres log. Two foreign-key violations, sixty seconds apart on the same backend:

```
2026-11-09 05:49:20 UTC ERROR: insert or update on table "memories" violates foreign key constraint "memories_topic_id_fkey"
2026-11-09 05:49:42 UTC ERROR: insert or update on table "memories" violates foreign key constraint "memories_l2_document_id_fkey"
```

**The write path is refusing correctly.** The defect is that the reason never reaches the caller. It is
one line — `crates/seatloom-mcp-emit/src/memory.rs:345`, the fallback arm `other =>
MemoryRejection::Db(other.to_string())` — and `sqlx`'s `Display` for a DB error is the near-useless
string `db error`.

**The fix already exists in this repo.** `crates/seatloom-mcp-emit/src/dispatch.rs:609` has a `db()`
helper that walks the source chain and renders `sqlstate`, `constraint` and `table`, with a comment
saying a rolled-back transaction is only diagnosable if the cause is reported. The dispatch tools use
it. The memory tools do not.

So a correct refusal is indistinguishable from a broken tool, and a careful seat reported a defect in
the harness instead of a bad argument. Dispatched `disp-Pca6m4qu1RERbP37cwlTL` to Nimbus with the log
lines, the one-line location, the existing helper as the pattern, and a required negative case: an
insert that must be refused must still be refused, and the message must name the constraint.

## A second finding I reported and did not fix

`documents` has **zero rows**, and `memories.l2_document_id` is a foreign key to it. Zero memories use
`l2_ref_kind=document`. So the document form of the L2 pointer can never succeed against this database.

That is the same shape as the typed-memory tools that could not succeed because `topics` had no rows —
a correct guard over an empty table. I put it in the dispatch as a report, not a task, because whether
to populate `documents` or to refuse the document kind early with a reason is a design question for
you. I have not answered it and I did not let the seat answer it either.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- One pending card, `acc-disp-MlhRWjAlGcaPUj2BqMpOr`, carrying the corrected 63-of-90 figure. Clickable,
  merges clean into current track.
- Vela's branch is synced: 11 ahead, **0 behind**, contains `track/infra-foundation`.
- Push: local and remote both at `d0329ca`, confirmed.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The card is waiting for you**, and the number on it is now correct. That correctness is Vela's
  work, not mine.
- **The 26 CJK files and the 37 header failures.** Renaming the 26 breaks 29 inbound references, 24 of
  them in the knowledge index.
- **The `documents` table has zero rows**, so the `document` L2 pointer can never succeed. Populate it,
  or refuse that kind early with a reason — your call, not mine.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
