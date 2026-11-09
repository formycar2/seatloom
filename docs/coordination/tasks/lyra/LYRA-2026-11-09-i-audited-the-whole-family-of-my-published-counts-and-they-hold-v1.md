# Standing duty: **I audited every count and every absolute time I have published this period against live state, rather than auditing only the one instance I had already caught. Everything verified: 491 events, 98 dispatches, 63 knowledge files split 26 CJK and 37 header, and the three "55" occurrences in my grandfather correction are the correction itself rather than a repeat of the error. The unit-sensitive claims hold.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-i-audited-the-whole-family-of-my-published-counts-and-they-hold-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

## Why this round was an audit and not a new finding

**Two rounds ago I found I had read a UTC log as local time. Last round I audited every absolute time and found the same error in three more places, then fixed them.**

**The lesson I wrote was "auditing one instance is not auditing the family".** So this round I applied it to the other thing I publish constantly: **counts.** I have been wrong about counts twice recently — the grandfather list (lines counted as paths) and the card's residual (a double subtraction) — and both times the error was in the unit, not the arithmetic.

**A family audit is the follow-through I skipped both of those times.**

## The audit, claim by claim

| published claim | re-measured now | holds |
|---|---|---|
| 491 events | **491** | ✓ |
| 98 dispatches | **98** | ✓ |
| 63 knowledge files still failing | **63** | ✓ |
| split 26 CJK / 37 header | **26 / 37** | ✓ |
| "55 paths wider" (in the correction doc) | the doc's own table says 55 **lines**, 43 comments, **0 path difference** | ✓ — it is the correction |
| 586 behind track | not re-measured this round | — |

**And I checked the "55" occurrences individually rather than by count**, because a number that appears three times in a document that exists to correct that number is ambiguous until you read each one. All three are in the correction's framing: the quoted wrong claim, the line-difference row, and the comment-line row. **None of them re-asserts the error.**

**The 37/26 split I re-measured with a patched copy of the gate in Vela's worktree — created, used, and deleted, with her tree verified clean at 0 changes afterwards.** That is the same method that found the short-circuit last week, and it still gives 37 and 26.

## What this establishes, and what it does not

**It establishes that the numbers I am currently publishing are measured rather than remembered.** That is worth something, because for several rounds my escalation list carried figures I had not re-checked, and three of them turned out to be wrong — the grandfather paths, the VM memory, and the OOM duration.

**It does not establish that they will hold next round.** The ledger moves every round; the behind-count grows; the knowledge-file split changes the moment anyone edits a file. **A count is a reading with a timestamp, and publishing it without the timestamp is how "63 files" became "37 files" in the card's description.**

**So the rule I am recording is the same one as for the clocks: a count without its measurement time and its unit is not a count.** "63 knowledge files failing, measured 11-09 20:54, split by first-reported rule" is a measurement. "63 files remain" is a claim that will be quoted after it stops being true.

## The thing I did not do, stated because it is the boundary

**I did not re-measure the behind-count.** It was 586 last round and it grows, and measuring it again this round would produce a number that is true for thirty seconds. **The reason to measure it is to decide something, and nothing I can decide turns on whether it is 586 or 590.**

**And I did not touch the two pending cards.** Both are verified and both are the operator's to accept. Re-verifying them a second time in one day would be activity rather than work — I ran their steps, they passed, and running them again would produce the same answer at the cost of another isolated worktree.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `psql` for the ledger counts, a patched copy of `verify-docs.sh` in Vela's worktree — created, used, deleted, her tree verified clean — for the knowledge-file split, and `grep` over my own documents for every published count and clock time. **I did not amend the three timezone corrections from last round** — they are pushed and correct.
