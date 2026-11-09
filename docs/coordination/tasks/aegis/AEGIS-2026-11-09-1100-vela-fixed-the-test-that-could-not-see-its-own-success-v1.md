| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1100-vela-fixed-the-test-that-could-not-see-its-own-success |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, docs-gate, verification, stuck-dispatch |

# Vela fixed the test that could not see its own success, and the change is verified from scratch

## What happened to the stuck dispatch

`disp-MlhRWjAlGcaPUj2BqMpOr` is **done**. After I cancelled the trapped one and re-issued, Vela
committed the work and closed the dispatch properly. The structural trap I found last hour is what
was blocking them, not the work.

## The commit

`4927d34` — 310 insertions across `scripts/verify-docs.sh` and a new
`scripts/test-knowledge-convention.sh`.

## The test bug I reported is fixed, and fixed the right way

I told Vela their `scratch_repo()` built an **empty** knowledge directory, so `BEFORE` was
structurally zero. They did not just patch the count — they changed what the test measures:

```
── 1. the header rule: before vs after, counted per RULE ──
  exemption OFF: header=63 cjk=0 other=0
  exemption ON : header=37 cjk=26 other=0
  ✓ the exemption cleared header failures (63 -> 37)
  ✓ the gate still reports knowledge failures (63) — it has not gone quiet
  ✓ 26 file(s) now reach the CJK rule they were previously short-circuited past
```

Counting **per rule** rather than per file is the real fix. A single file can fail several rules,
and a total count hides which one moved. This is the same lesson as my own mistake two nights ago,
where I reported 328 docs without classifying them.

## What I verified independently

I did not take their numbers. I extracted the functions and ran them against the real knowledge
files myself:

| | header failures | CJK failures | total |
|---|---|---|---|
| exemption OFF | 63 | 0 | 63 |
| exemption ON | **37** | **26** | 63 |

Exact match with their measurement. So 26 files were genuinely unblocked — they now reach the CJK
rule, which is a **different** rule and a real remaining issue, not a cover-up.

Then I checked the two directions that matter, in gate mode with real exit codes:

| case | result |
|---|---|
| a **new** knowledge file with good frontmatter | **passes**, exit 0 |
| a **new** knowledge file with no frontmatter | **refused**, exit 1 |
| a file outside `knowledge/` | still flagged — the exemption did not leak |

Their test is 8/8, and it still passes 8/8 after simulating the merge into current track.

## The remaining 26, stated honestly

All 26 are plain `.md` files carrying Chinese content. The CJK rule applies to them **by design** —
`.zhs.md` is the sanctioned form for Chinese, and none of these 26 use it. That is not a gate defect;
it is 26 files that should either be renamed or translated. It needs your decision, and I have not
made it for you.

## Recommended next step, not taken

The branch is 7 ahead and 20 behind track, merges cleanly, and its test passes against current
track. I have **not** merged it. Merging a change to the doc gate onto the mainline is exactly the
kind of thing that should go through the acceptance path rather than my judgement, and the branch
has no card.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Zero seats blocked on in_flight — the trap I found last hour is cleared.
- Push: local and remote both at `1e92514`, confirmed.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Whether to merge the doc-gate change.** It is verified and merges cleanly. If you want it, the
  path is a card, not my merge.
- **The 26 Chinese `.md` knowledge files** — rename to `.zhs.md`, translate, or declare as debt.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
