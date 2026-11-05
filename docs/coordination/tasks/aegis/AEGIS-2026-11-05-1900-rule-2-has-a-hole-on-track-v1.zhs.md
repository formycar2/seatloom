# Delivery for the 19:00 heartbeat — the push recovered on its own, and I found that rule 2 does not apply on track at all, which is where I commit most

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-1900-rule-2-has-a-hole-on-track |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | heartbeat, guard-hole, my-own-error, unexplained-state |

**Changed**: no repo files this round (the probe was removed; nothing needed fixing in code yet — see §4).
**Headline**: origin sync recovered by itself. And I proved that **rule 2 — the guard against taking another seat's files with a glob — does not apply on `track/infra-foundation` at all**, which is the branch I commit to most.

---

## 1. Ledger and fleet

35 tables, query proven non-empty (`dispatches` = 11). Six seats online, pane children all `saih`. `in_flight` = 0, queue empty. All eight checks exit 0. Protocol 2b and 2c direct queries both return **0**.

## 2. The push recovered, and I did not have to do anything

Last round GitHub rejected with `remote: fatal error in commit_refs`, twice, and I proved it was server-side by pushing an unrelated ref and getting the same failure. This round a single push succeeded: origin is now at `fb99fda`, `0 0` ahead/behind, 0 pending cards.

**Recorded so the diagnosis is not rewritten later**: the failure was transient server-side, the local work was never at risk, and no retry-storm was needed.

## 3. Last round's unexplained state, re-investigated — and one mechanism eliminated

Last round a file from Lyra's branch appeared **staged** in my index with no `git add` of mine, then vanished minutes later. I recorded it as unexplained rather than guessing.

This round I tested the obvious candidate instead of speculating:

| hypothesis | test | result |
|---|---|---|
| another worktree's git commands refresh my index | run `git status` in `lyra` worktree, compare main checkout's `.git/index` mtime | **eliminated** — mtime unchanged |
| an active background writer | sample `.git/index` mtime over 30s | **eliminated** — unchanged |
| my own `git status` refreshed it | observed mtime advancing 8s after my own read-only command | **confirmed as sufficient explanation for this round's observation** |

**What I can honestly say now**: the index mtime advances when I run read-only git commands, which is normal git behaviour. **I still cannot explain last round's staged entry**, and I am not going to invent a mechanism for it. The useful outcome is not the cause — it is §4.

## 4. 🔴 The real defect this led to: rule 2 has a hole exactly where I work

I said last round that the pre-commit hook would have caught that staged file. **That was wrong, and I proved it wrong this round.**

I recreated the state — a markdown file from `docs/coordination/tasks/lyra/` in the index — and ran the hook on `track/infra-foundation`:

```
pre-commit: OK - rule 2: no other seat's files were taken by a glob
pre-commit: REFUSED - the docs this commit touches do not conform.
exit=1
```

**Rule 2 printed OK and let it through.** The refusal came from the doc gate, for an unrelated reason (my probe lacked English text and a conforming filename). Had the probe been a well-formed document, it would have committed.

**Why**: `SEAT` is derived from the branch name via `packet/<seat>/...`. On `track/infra-foundation` there is no such prefix, so `SEAT` is empty, and the whole rule is skipped by design:

```
# 推不出席位就不拦:没有依据说这个目录是谁的。
if [ -z "$SEAT" ]; then
  : # 规则不适用
```

That design is **correct for its original purpose** — Lyra earned it on 11-01 by showing that a rule which refused when it could not attribute ownership would block every merge on the integration line, and merges must cross directories. "Author privilege" was the name she gave the bug, and the fix was right.

**But it leaves a hole in the one place I commit most**: on track, nothing checks whether a file from another seat's directory has been staged by something I did not intend. Last round's unexplained entry would have sailed through.

## 5. Why I am not fixing this myself

The two candidate fixes both change a guard whose current shape was set by a ruling that is not mine:

- **A** — on track, refuse any staged file under `docs/coordination/tasks/<other-seat>/` unless explicitly named. Risk: a legitimate merge onto track necessarily brings other seats' files, and this would block every merge, which is precisely what Lyra's correction prevented.
- **B** — keep the current behaviour, but make rule 2 print *why* it is not applying instead of printing OK. Risk: near-zero; it converts a silent hole into a visible one.

**B is clearly safe and I could do it. A changes a governed guard.** I am not choosing A on my own — that is the same boundary as not merging another seat's branch and not grandfathering her documents. **But I want the operator to know the hole exists, because I reported its protection as working last round and it is not.**

## 6. What I did not do

- Did not change the hook.
- Did not commit, stage, or delete anything belonging to another seat (the probe was mine and is removed).
- Did not merge `6e04dae`.
- Did not write to the ledger, restart anything, or open a card.

## 7. Checks

| check | exit |
|---|---|
| `check-ledger-exists` (35 tables, proven non-empty) | 0 |
| `fleet-status.sh` | 0 |
| `check-salvage-fresh` / `check-cards-clickable` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable` / `test-clickability-criterion` / `check-unclaimed-work` | 0 each |
| protocol 2b (stranded implementation work) | 0 rows |
| protocol 2c (pass verdict, no card) | 0 rows |
| `git push origin track/infra-foundation` | 0, origin now `fb99fda` |
| worktree-index hypothesis | eliminated by measurement |
| **rule 2 on track, another seat's file staged** | **printed OK — the hole, demonstrated** |
| working tree at end | clean except known untracked `scripts/bind-seatloom-seats.sh` |

## 8. For the operator — three lines

**变了什么**：GitHub 自己好了，origin 已同步。另外我发现上轮说「那个守卫会拦住意外暂存的别人文件」是错的——我造出那个场景实测，它打印 OK 放行了。原因是这条守卫只在能从分支名推出席位时才生效，而我在 track 上提交时推不出来，所以整条不跑。这是 11-01 为了避免拦死所有合并而刻意设计的，原设计没错，但它正好在我最常提交的地方有个洞。

**你会看到什么**：`fleet-status.sh` 无变化。

**需要你做什么**：三件——① 这个洞怎么补：严格版（在 track 上拒别席位目录的文件，但可能拦死合并，需要你判断）还是只让它说清「我为什么没在检查」；② 那 27 篇存量不合规文档；③ Nimbus 的删除守卫要不要合进 track。
