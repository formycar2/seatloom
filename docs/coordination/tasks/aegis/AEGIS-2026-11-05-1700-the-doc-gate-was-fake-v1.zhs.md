# Delivery for the 17:00 heartbeat — the doc gate I reported as "wired up" last round had never refused anything

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-1700-the-doc-gate-was-fake |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | heartbeat, fake-gate, my-own-error, must-fail-case |

**Changed**: `scripts/verify-docs.sh`, `docs/coordination/tasks/aegis/AEGIS-2026-11-02-standing-backlog-v1.zhs.md`.
**Headline**: at 15:00 I reported the doc gate as wired into the commit path. **It had never refused a single document.** Found by feeding it a case that had to fail.

---

## 1. The defect, in one line

The hook calls `verify-docs.sh HEAD "$DOC_PROBE"` meaning *"compare HEAD against the commit that is about to exist"*. **`verify-docs.sh` takes exactly one positional argument.** The second was silently discarded, and the diff is hardcoded to `$BASE..HEAD` — where HEAD is the *last committed* commit, which by definition does not contain the new file.

So the answer for anything not yet committed was always **"all conform"**.

## 2. How I found it — by refusing to trust my own report

Last round I wrote "the doc gate is wired into the commit path for the first time" and showed three OK lines as evidence. This round I asked the only question that settles it: **does it refuse something that must be refused?**

I staged a markdown file with **no universal header at all** and ran the hook:

```
pre-commit: OK - docs touched by this commit conform
exit=0
```

It was not a gate. It was an echo.

## 3. The fix, and why this error class is mine

`verify-docs.sh` now takes a real second argument, `RIGHT` (default `HEAD`), and diffs `$BASE..$RIGHT`. When the hook passes the probe commit, the new file is visible.

**Why I keep making this one**: the script's output was *correct for the question it actually answered* — "do the docs changed between BASE and HEAD conform?" — and wrong for the question I thought I was asking. **A tool that answers a different question than you asked, in the same format, is indistinguishable from a working tool until you feed it a case that must fail.**

That is now the rule I wrote into the script's header: after wiring any gate, feed it a case that MUST fail.

## 4. Proof, in both directions

| case | before | after |
|---|---|---|
| markdown with **no header at all**, staged | OK, exit 0 | **REFUSED, exit 1**, names the file and the missing fields |
| markdown with a full header but **CJK text** and a **non-conforming filename** | (not tested) | **REFUSED, exit 1**, cites §10 and §4 |

The second case is the stronger evidence: my first "good" probe used a Chinese body and a `ZZ-PROBE-GOOD.md` name, and the gate refused it on **§10 (new docs must be English)** and **§4 (filename must match `ROLE-YYYY-MM-DD-<topic>-vN.md`)**. So the gate is reading the real rules, not just checking for a table header. **My probe was wrong, not the gate** — and I verified that before drawing any conclusion from it.

All probe files removed; `git status` shows only the two intended changes.

## 5. The standing item, moved where it belongs

The 26 Mira reports plus `BUDGET_POOL_MODEL_MAP.md` (27 docs missing the universal header) were already known; they are now in the standing backlog as item 4, with the measured breakdown and the reason they are not my call:

> her format is her convention, her content is her real record, and deciding her work is permanently exempt is not mine to decide — the same boundary that stops me merging her branch.

They stay out of the heartbeat body, per the 21:00 rule: unchanged blockers go in the backlog, not repeated in the正文.

## 6. Checks

| check | exit |
|---|---|
| `check-ledger-exists` (35 tables, proven non-empty) | 0 |
| `fleet-status.sh` | 0 |
| `check-salvage-fresh` / `check-cards-clickable` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable` / `test-clickability-criterion` / `check-unclaimed-work` | 0 each |
| **doc gate, no-header probe** | **1 (refused)** — was 0 before this fix |
| **doc gate, header-but-wrong-shape probe** | **1 (refused, §10 + §4)** |
| hook, three checks, real commit | rule 2 + docs + cards all run |
| `verify-docs.sh <old baseline>` full-repo | 1, 27 pre-existing violations (unchanged, grandfathered-pending) |
| working tree after probes | clean except known untracked `scripts/bind-seatloom-seats.sh` |

## 7. For the operator — three lines

**变了什么**：我上轮说「文档门接上了」，实际它一次都没拦过——脚本只接受一个参数，我传的第二个被静默丢掉，而它比的是「上一个提交」，那里当然没有新文件。这轮我用一篇完全没有表头的文档去撞它，才暴露。现已修好，双向证明：没表头的拒、格式错的也拒（连中文正文和文件名不规范都抓）。

**你会看到什么**：`fleet-status.sh` 无变化；以后不合规的新文档会在提交时被直接拒，不会再混进 track。

**需要你做什么**：还是那两件——① 那 27 篇存量不合规文档（26 篇是界面席位的心跳回报），让她补头还是你裁定进豁免名单；② Nimbus 的删除守卫要不要合进 track。
