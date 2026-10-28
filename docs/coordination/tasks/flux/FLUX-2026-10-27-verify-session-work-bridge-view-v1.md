# Flux verdict — the session<->work bridge view (022), re-verified after my own fail

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-session-work-bridge-view-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | reverification, sql-migration, read-only-view, own-fail-resolved, seat-flux |

Dispatch `disp-06a95xPk7h6E2pj6FDo4O`, request 1 of 3.
Subject: `disp-XnCxvGI88oAbdGcyeefhl` — Nimbus's `v_memory_session_join` (022), commit `f7bfe23`.
Verdict event: `ev-RZkE0mWIuSNWp0FmpJ8nU` (this dispatch's own id, payload naming `disp-XnCxvGI88oAbdGcyeefhl`).
This is a **re-verdict**: I issued `ev-FLUX-AUDIT-GATE-C-SESSION-WORK-BRIDGE` = **fail** on 10-26.

## Verdict: PASS

My prior fail had exactly one cause: the view did not exist on live and the path to it
through `scripts/apply-db.sh` was blocked. That block is now gone and the view is
correct. All four of the original fail's measured conditions are re-measured below.

## The three conditions I named in the fail, re-measured

| # | My 10-26 fail condition | Re-measured now |
|---|---|---|
| 1 | `v_memory_session_join` does not exist on live | **exists** (1 row in `information_schema.views`) |
| 2 | 022 unreachable because 021 is in neither apply-db list | **021 is registered** in `POST_SEED_SCHEMA` (`scripts/apply-db.sh`, line 81) |
| 3 | apply-db.sh's completeness guard refuses to build any DB | **guard passes**: 20 files on disk, 20 registered, both directions empty |

The third one is worth spelling out because it was my own audit finding that caused
the block: I diffed `infra/postgres/schema/*.sql` against the filenames registered in
`PRE_SEED_SCHEMA`/`POST_SEED_SCHEMA` and the symmetric difference is empty in both
directions. `bash -n scripts/apply-db.sh` also parses. The 021 entry's own note records
that it was unclassified until my audit — the block I found was closed by the seat that
owned it, not by me.

## The number the contract asked for, independently re-measured on live

The contract's §11.2 slice-1 requirement is one number: how many memories carry a
session reference that resolves to nothing. Measured on the live DB:

```
 resolved_by | count
-------------+-------
 both        |   87
 a1_path     |    3
 a2_source_ref | 0
```

```
 total | a1 | a2 | missing
-------+----+----+---------
    90 | 90 | 87 |    0
```

These are my numbers, from my own queries, and they match his delivery note exactly.
`total` is also independently confirmable: `SELECT count(*) FROM memories WHERE
source_kind='archived_session'` returns 90, the same count the view returns.

**The load-bearing assertion is "the number is measurable", not "everything resolves"**
— and it holds. The 3 `a1_path` rows are not a rounding difference; they are a real
disagreement between the two keys, and the view names it instead of collapsing it.

## The 3 rows are the point, and I verified what they are

`mem-kyloXn9AY3D_YHA6WQk7h`, `mem-Gih4q6JxjyyJmQJCHJmO8`, `mem-xqtBZmhQd1Iw5t732-tnk`
all carry `source_ref = codex:019dda18-07f6-79e0-8008-e4eb16a5448d` and all three resolve
by path (A1) but not by `source_ref` (A2). So the disagreement is systematic, not
noise: one codex session id referenced three times through a path-shaped reference.

This is exactly the class of silent breakage the migration exists to surface — before
021's re-key those references resolved against nothing and the product reported nothing.
A view that only checked A1 would hide these three rows; a view that only checked A2
would misreport them as orphans. His choice to report both keys plus `resolved_by` plus
a `missing` column is what makes the disagreement countable.

## Criterion-by-criterion

| # | His criterion | My result |
|---|---|---|
| 1 | view exists, `select count(*)` runs | **90** |
| 2 | session/work join gives counts and samples | 87/3/0 above; the 3 named samples are real rows |
| 3 | migration is idempotent | **re-applied the file against live: `DO`, exit 0, view still 1, still 87/3/0** |
| 4 | `sessions` still 5 rows | **5** |
| 5 | (mine) view is read-only | `INSERT` → `ERROR: cannot insert into view "v_memory_session_join"` |
| 6 | (mine) unresolved count | **0** |

On criterion 3 I want to be precise about what I did and did not do. I re-applied
`022_session_work_bridge_view.sql` to the **live** database. It is a catalog-guarded
`CREATE` (015's pattern: `IF NOT EXISTS (SELECT 1 FROM pg_class ...)`) and a view has no
rows, so it writes nothing; I measured that claim rather than assuming it — the apply
returned `DO` with exit 0 and the view's contents and catalog count were unchanged. I
did not rebuild the database from scratch, and I did not apply any other migration.

## What this does not prove

- **I did not verify his full-path rebuild.** He reports building a scratch DB through
  the canonical order (PRE-SEED 6 → 4 seeds → POST-SEED 14) and finding `sessions`
  5 → 5 → 5, plus a second scratch where the 3 codex rows showed `unresolved` until he
  copied in the live-only session. That work is not reproducible from the branch and I
  did not attempt to repeat it; I verified the view against the live data directly
  instead, which is the state that matters for the gate.
- **A2's 3 rows remain unfixed, and that is an operator decision, not a defect.** He
  offers two fixes (rewrite 3 `source_ref` values, or add a fallback match to the view)
  and explicitly declined to pick one. I agree with that refusal: both options touch
  already-written memory rows, and 015/018's rule for that is forget + remember with the
  old row left as a tombstone. The current state is honest — visible, named, counted
  separately from the 87.
- **The contract file is not in the worktree.** `LYRA-2026-10-16-session-work-bridge-contract-v1.md`
  lives on `packet/lyra/milestone-summary`. He read §11.2 from there. I verified the view
  against the numbers, not against the contract text, so a mismatch between the contract
  and the implementation would not have been caught here.

## Findings

**F1 (non-blocking, recorded for the record).** His delivery's escalation 1 says
`scripts/apply-db.sh` "still cannot run" because 021 is unclassified. That was true when
he wrote it and is **no longer true** — 021 is now registered. So that escalation is
resolved rather than open, and I have recorded it as such rather than passing it on.

**F2 (operator decision, re-raised, unchanged).** Whether to repair the 3 `source_ref`
values. Now that the view exists the number is stable and visible, so the decision can
be made on real data rather than on a hypothesis.

## Live-machine hygiene

I ran read-only queries plus one re-apply of 022 to the live DB (a no-op by construction,
measured as described above). No rows were inserted, updated or deleted. No other
migration was applied. No worktree of another seat was modified; I worked from Nimbus's
worktree read-only and created no worktree at all for this request.

## Files

No files of mine changed for this request; the verdict and this note are on
`packet/flux/intent-verdict`.
