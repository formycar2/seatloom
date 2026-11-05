# Delivery for the 16:00 heartbeat — the "unclaimed work" query reported the same five false rows every round, and my first fix was still wrong

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-1600-unclaimed-work-query |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | heartbeat, query-false-positive, my-own-error |

**Changed**: `scripts/check-unclaimed-work.sh` (new).
**Headline**: protocol step 2b's query reported 9 rows every round; 5 of them were dispatches that can never have a card. My first fix was also wrong and reported 4 more false rows — all four of them mine.

---

## 1. The false positive, and why it matters more than it looks

Protocol step 2b asks for dispatches that are `done` with no acceptance card. Measured on live: **9 rows**. Five of them were coordination dispatches whose entire payload is *"please open a card for X"* — they are the card step, so a card of their own is a category error.

A query that reports the same five false rows every round trains the reader to skip it. **Then the real row is skipped with them.** That is the failure mode this fixes.

## 2. 🔴 My first fix was wrong, and it was wrong in my favour

I classified by payload *shape*: `packet` or `kind` present ⇒ implementation work that owes a card. It reported **4 rows — all four of them mine**, all `kind='verification'`, none of which owes a card.

**The shape does not separate them.** My verification dispatches carry `kind`, `note`, *and* `packet` — the same keys as real work. I had built a classifier that flagged exactly my own dispatches and nothing else, which is the most flattering possible error and the easiest to believe.

What separates them, measured on live:

| `payload->>'kind'` | done dispatches | with a card |
|---|---|---|
| `verification` | 4 | **0** — correct, a verification request is a question, not a deliverable |
| absent (`note,acceptance` = coordination) | 7 | 2 — and those 2 are exactly the product work the operator accepted |

So the value, not the shape, is the signal. The final predicate excludes `kind='verification'` **and** payloads carrying `acceptance`.

## 3. What the script does

| aspect | behaviour |
|---|---|
| read proven first | `select count(*) from dispatches` must succeed and be non-empty, else **exit 2**, never a silent 0 |
| reported as a defect | done, not verification, not coordination, no card, within 48h |
| reported as information | verification + coordination with no card, labelled *"correct, they owe no card"* |
| `--quiet` | prints only on failure |

Current live answer: **`OK: no implementation-grade dispatch is stranded without a card`** plus 9 correctly-excluded rows.

## 4. Proof it can fail — both directions

| mutation | result |
|---|---|
| remove the `acceptance` exclusion | reports the 5 coordination rows — the predicate has discriminating power |
| disable the kind exclusion entirely | reports `OK` — an over-broad exclusion is caught by the same test |

An earlier draft of this round also nearly recorded a false exit code: I piped the mutated script through `head`, which swallowed its status and printed `exit 0` for a failing run. Re-measured without the pipe. **This is the protocol's "check exit codes explicitly" rule earning its keep again.**

## 5. What I did not do

- Did not open cards for anything. The 2c query (`pass` verdict with no card citing it) returns **0 rows**, so nothing is verified-and-unclaimed.
- Did not dispatch Flux. Nothing implementation-grade is stranded, so there is nothing to verify.
- Did not merge `6e04dae` — still the operator's Accept to give.
- Did not add the 27 non-conforming docs to the grandfather list — still not mine to decide.

## 6. Checks

| check | exit |
|---|---|
| `check-ledger-exists` (35 tables, proven non-empty) | 0 |
| `fleet-status.sh` | 0 |
| `check-salvage-fresh` / `check-cards-clickable` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable` / `test-clickability-criterion` | 0 each |
| **pre-commit hook, all three checks** | rule 2 + docs + cards, all printed and run |
| `check-unclaimed-work.sh` (new) | 0, with 9 rows correctly excluded |
| protocol 2b query, re-read with the fix | 0 stranded |
| protocol 2c query | 0 rows |

## 7. For the operator — three lines

**变了什么**：我把一个每次都误报的查询修好了。它原先每轮报 9 件「干完了没人接」，其中 5 件是协调派单、根本不该有卡——这种查询每轮喊狼来了，真正的问题会被一起跳过。我第一版修法还错，而且错得对我有利：它只挑出我自己的 4 件派单。

**你会看到什么**：`fleet-status.sh` 无变化；提交前的守卫三条检查都在真实运行（这轮提交的输出为证）。

**需要你做什么**：还是那两件——① 27 篇格式不合规的历史文档补头还是裁定豁免；② Nimbus 的删除守卫要不要合进 track。
