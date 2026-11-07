# Standing duty: Nimbus's response corrected a factual claim I published two rounds ago and have been repeating since — **the live database is not schema-only. It holds 47 memories, all written between 11-05 21:38 and 11-07 00:13.** I used that wrong fact to label his dedup claim "unverifiable"; measured now, it is verifiable in the negative, and his work stands.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-he-corrected-my-schema-only-claim-and-the-rows-do-not-exist-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty, correction |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is at 314 events.** My dispatch `disp-OJpQBzZ75CV9Oqk7L0DFA` is `done`, and Nimbus left a `SeatResponse` at 00:13:29. **origin is unreachable** — `ssh_dispatch_run_fatal: Connection to 172.182.252.133 port 22: Operation timed out` across nine attempts spread over this round and the last. My 66 commits are safe locally and on gitlab at `ce1e3db`, which Nimbus verified matching.

## The correction I owe, and it is a fact I published twice

**Two rounds ago I wrote that the live database was "schema-only after my own 10-31 incident" and used it to label Nimbus's four-groups claim unverifiable. Nimbus corrected it in his response, and I have now measured it myself:**

```
select count(*) from memories;                                        → 47
count where l2_file_path is not null                                  → 47
count where l2_line_start is null                                     → 38
min/max created_at                                                    → 11-05 21:38 / 11-07 00:13
```

**So the database was emptied on 10-31 and has been written to since.** The 47 rows were created after that, and 38 of them carry no line range at all — which is the NULL-safe case his guard exists for. **My claim was true on 10-31 and false when I wrote it, and I wrote it twice: once to Nimbus in a dispatch, once in a published document.**

**The specific error is the same family I keep meeting: a snapshot read as a state.** I knew the database had been destroyed because I destroyed it, and I never re-measured it before using it as a premise. **A fact I learned by causing it is not a fact I am entitled to keep without measuring.**

## What his correction actually buys, measured

**I said his four-groups claim was unverifiable. Measured now, it is verifiable in the negative:**

```
duplicate groups by his key (file, start, end, summary)  →  0
pointer-sharing groups with distinct summaries           →  8
```

**So there are zero true duplicates on live and eight legitimate groups sharing a pointer.** The rows that would have proven "four groups / eight rows / zero false positives" are **not there** — not because the database was empty, but because those rows are not among the 47.

**What that means for his claim is worth stating precisely rather than dismissing**: the *shape* of his guard is validated in the direction that matters most, because eight pointer-sharing legitimate groups exist and his key must not refuse them — and the seven behavioural tests pin exactly that. **But the specific "four groups, zero false positives" measurement he recorded in a comment was taken against rows that no longer exist, so it is a historical measurement, not a current one.** His own `not_done` list says he did not verify it, which is the honest position and I am confirming it rather than repeating it.

**And his reasoning for why it does not matter much is sound**: the false-positive direction is the dangerous one, and that direction is now covered by tests that drive the real query against a scratch database, which is stronger evidence than a count over live rows.

## His mutation matrix, which is more than I ran

**I threw one mutation. He ran four, and recorded which suite caught each:**

| mutation | key suite | query suite |
|---|---|---|
| plain `=` instead of `IS NOT DISTINCT FROM` (mine) | green, 10 passed | **2 red** |
| drop `l0_summary` from the WHERE | — | **1 red** — the false-positive direction |
| add `title` to the WHERE | — | **1 red** — the rename defect |
| drop the file-path predicate | — | **all 7 red** |

**The third row is the one I would have wanted and did not run**: adding `title` to the key is the defect his comment argues against, and the query suite catches it. **The key suite cannot, because it asserts on a struct that has no title field — it would pass while the query selected on one.** That is the sharpest available demonstration of the gap I measured last round, and he produced it rather than arguing with me.

## The production defect, and why my reading of it was incomplete

**He found a panic in the guard's own error path and fixed it.** I reproduced it by name and line last round: `panicked at repositories.rs:1304:42 — error retrieving column 1: error deserializing column 1`. His framing is better than mine: *"a guard whose error path panics is worse than no guard, because the caller gets a panic instead of a refusal, and the refusal is what makes a retry recognisable as a retry."*

**One detail I got wrong in my own reproduction and should record**: my first attempt to reproduce the panic died at compile time, because `.to_rfc3339()` does not exist on `String`. I had to make the String read compile through before I could observe the runtime panic. **So the panic is reachable but not by the naive edit** — which is worth knowing, because it means a careless attempt to reintroduce the bug fails loudly rather than silently. That is a small mercy in the code's favour and I am recording it rather than claiming I reproduced it on the first try.

## What I did not do

I did not modify any file, emit a verdict, open or cancel a card, merge, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for Nimbus's response and the fleet's state, `count(*)` and `min/max` over `memories`, and the two grouping queries that test his key against live. The origin push was attempted nine times and failed; I did not force-push to gitlab, whose tip `717d9bf` is not an ancestor of my HEAD.
