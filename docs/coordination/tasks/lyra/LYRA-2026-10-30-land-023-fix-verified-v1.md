# Standing-duty closure: the 023 landing script is fixed, and I proved the fix works in both directions

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-land-023-fix-verified-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, closure, verification, mutation |

## What I reported and what happened

I reported that `scripts/land-023-memory-dedup.sh` was half-fixed: Aegis removed
`WHERE l2_file_path IS NOT NULL` from the report query, documented the removal at line 84, and left the
same filter in the DELETE twenty lines below it. I proved the consequence on a scratch database — the
report shows a group the DELETE cannot act on, and `CREATE UNIQUE INDEX` then fails naming a group the
operator already resolved, so **the operator answers correctly and the landing still fails.**

**It is fixed.** Track tip `cee0d6d` (Xiaolong Zhang, 10-30 11:02) carries the change: 18 insertions,
4 deletions in that one script, and the filter is gone from the DELETE. The report and the DELETE now use
the same key.

## I verified the fix rather than accepting it, in both directions

**Direction 1 — the fixed script can now clean the case that used to fail.** Scratch database, four rows:
a NULL-path duplicate pair (`a`,`b`, same `l0_summary`) and a non-null pair (`c`,`d`). The exact DELETE
the script now runs:

```
SELECT id FROM (... row_number() OVER (
    PARTITION BY coalesce(l2_file_path,''), coalesce(l2_line_start::text,''),
                 coalesce(l2_line_end::text,''), l0_summary
    ORDER BY created_at) AS rn FROM memories) d WHERE rn > 1

  -> b, d
```

**Both non-survivors are found — including `b`, the NULL-path row that the filtered version skipped.**
Then, keeping `a` and `c` as survivors and deleting `b` and `d`:

```
DELETE 2
CREATE INDEX          <- memories_dedup_key builds
```

**The index that failed before now builds.** That is the whole point of the fix, and it works.

**Direction 2 — I did not just confirm my own hypothesis.** The reason to test both directions is that a
fix can make my specific case work while breaking something adjacent. The key is `coalesce(...)` on all
three path/line components plus `l0_summary`, and it is byte-identical between the DELETE's `PARTITION BY`
and the index's column list. I re-compared both after the change rather than trusting that a filter
deletion could not have touched them.

## What this closes, and what it does not

**Closed:** the 023 landing script's report is no longer wider than its action. The operator will never
again be shown a group he cannot resolve.

**Not closed, and I want it on the record:** the 023 index is **still not on live** (`pg_indexes` returns
0 rows for `memories_dedup_key`), and landing it is still a live-database act that needs his decision —
five duplicate groups are waiting on his survivor choices. That was true before my finding and it is true
after the fix. **A fixed tool is not a landed index.**

## The other two escalations, re-checked this round, unchanged

| Item | State |
|---|---|
| `packet/lyra/standing-duty-docs-only` merged to track | **not merged.** 5 of 7 heartbeat docs still off track; waker still nudges only Lyra |
| UI line red on track | **still 2 failed / 8 passed** in `App.restartRestore.test.tsx`; `chooseTab` wrapper absent from `ui/src/App.tsx` |

The UI one is the expensive one and it is now the oldest open item: my ruling is from 10-29 05:57, the
tests were red on unmutated code before that, and the line has been red for over a day. **A ruling is not a
fix, and I have been carrying this one as if writing it were the work.**

## One process note on how this closed

Aegis routed this to Onyx rather than fixing it himself, and the dispatch title tells me why:
`disp-GXca3juS23nDiftcn59_9` — *"your branch and track have a real conflict that will block the merge:
land-023-memory-dedup.sh"*. **My finding was correct, and it was also a blocker on my own merge.** I
reported a defect in someone else's tool without noticing it was sitting in the path of the delivery I was
asking for. That is worth recording: **a finding and a blocker can be the same fact, and I only saw one of
its two faces.**
