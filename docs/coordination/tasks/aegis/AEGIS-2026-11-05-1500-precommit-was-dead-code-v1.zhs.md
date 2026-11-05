# Delivery for the 15:00 heartbeat — the pre-commit hook's card check had been dead code since the day it was installed, and I proved it with a trace

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-1500-precommit-was-dead-code |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | heartbeat, guard, dead-code, my-own-bug |

**Changed**: `.git/hooks/pre-commit` (not tracked in the repo — it lives in the git common dir, installed by `scripts/sync-hooks.sh`).
**Headline**: the card-stranding guard printed "OK" on every commit since 10-28 while **never running once**. Found by tracing, not by reading.

---

## 1. How I found it — and why reading did not find it

This round `verify-docs.sh` exited 0 with no violations, where the previous round reported 27. My first move was to check whether the docs had been fixed. They had not.

The real cause: **the default gate picks its baseline as `merge-base HEAD origin/track/infra-foundation`** — and because I pushed at 14:00, HEAD *is* that merge-base. So the gate saw zero changed docs and printed "nothing to check". **Pushing made the default gate blind.**

That is a genuine finding, but it is not the one that mattered. Chasing it led me to read the pre-commit hook, and there I found something worse.

## 2. The defect: line 121 was an unconditional `exit 0`

```
119  echo "pre-commit: OK - all pending cards will still contain this commit" >&2
120  echo "pre-commit: OK - 没有其他席位的文件被 glob 带走" >&2
121  exit 0                          <-- everything after this was unreachable
```

**79 lines of card-stranding measurement never executed. Not once, since 2026-10-28.**

Why this is worse than a missing guard: on 11-04 I recorded that I broke my own pending cards four times by committing delivery notes, and noted that the hook printed "OK - all pending cards will still contain this commit" each time. **I believed that OK.** The measurement it describes is real code — but the code was downstream of an `exit 0`, so the promise was printed and never kept. A guard that prints success and exits converts "I did not check" into "I checked".

## 3. Why a trace found it and reading did not

I read that file three times. What exposed it was `bash -x` on a staged probe file, which shows the actual path taken. **A misspelled ref does not raise an error; only following the execution reveals it.**

And the trace immediately caught the *next* one — my own, introduced one edit earlier:

```
+ '[' track/infra-foundation '!=' track/infa-foundation ']'
+ exit 0
```

I had written `track/infa-foundation`, missing an `r`. That made the "continue on the integration branch" test always true, so the card check was skipped *again* — this time by my own hand. **This is the fourth or fifth recurrence of the same error family in the protocol** (typo'd ref → a branch that should continue exits early → looks completely normal). It is now recorded in the hook itself.

## 4. What the hook does now

| order | check | behaviour |
|---|---|---|
| rule 2 | no other seat's files taken by a glob | refuse, listing paths |
| **docs** | `verify-docs.sh HEAD <probe-commit>` — only the docs *this commit* touches | refuse, showing violations |
| cards | after building the would-be commit, every pending card's branch must contain it | refuse, naming cards |

The doc gate uses the same technique as the card check — build the commit that *will* exist with `commit-tree` (no ref moves, no push, working tree untouched) and measure that. It deliberately does **not** run the whole-repo gate: that would surface 207 grandfathered entries plus 27 unregistered ones, and be slow enough that nobody runs it.

The two OK lines were reworded. They used to claim cards were safe; now they claim only what was actually measured at that point.

## 5. Proof, both directions — and one thing I refused to do

**Proven the control flow now reaches the card check**: `bash -x` shows the `podman` query executing and reporting "no pending cards, nothing this commit can strand".

**Proven the measurement itself discriminates**, in an isolated repo (`/tmp/hooktest`), driving the hook's own predicate `git merge-base --is-ancestor $FUTURE $branch`:

| state | result |
|---|---|
| branch lacks the future commit | **refused** |
| branch after merging the trunk | **allowed** |

**What I refused to do**: insert a fake pending card into the live `acceptances` table to test end-to-end. That is a live-database write, it is permission-capped, and it would have put a fabricated row in the operator's ledger. The isolated-repo proof of the predicate is honest; a fake card would not have been.

## 6. What is still open, and it is not mine to close

**26 of Mira's heartbeat reports and `BUDGET_POOL_MODEL_MAP.md` carry non-standard headers** and are **not** in `.gitlab/docs-grandfather.txt` (207 legitimate entries exist; these 27 are not among them). They entered track through commits where the doc gate did not exist.

**I am not adding them to the grandfather list.** They are her deliverables, the format is her convention, and deciding that her work is permanently exempt is not mine to decide — the same boundary that stops me merging her branch. The hook now catches *new* occurrences; the existing 27 need either her to fix the headers or the operator to rule them grandfathered.

**And the standing item is unchanged**: Nimbus's deletion guard (`6e04dae`) is still not on track, so `cmd_accept_delivery` still merges without a guard.

## 7. Checks

| check | exit |
|---|---|
| `bash -n .git/hooks/pre-commit` | 0 |
| `bash -x` trace reaches the card check | confirmed |
| isolated-repo predicate, both directions | refuse / allow as required |
| `check-ledger-exists` (35 tables) | 0 |
| `fleet-status.sh` | 0 |
| `check-salvage-fresh` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable` / `test-clickability-criterion` | 0 each |
| `verify-docs.sh HEAD <probe>` (as the hook calls it) | 0 |
| working tree after all probes | clean except the known untracked `scripts/bind-seatloom-seats.sh` |

## 8. For the operator — three lines

**变了什么**：我发现提交前的那个守卫从 10-28 安装起就没真正运行过——它每次都打印「卡安全」然后退出，后面的检查一步没进。我 11-04 还据那条 OK 相信过它。现已修好，并把文档门也接上了。修的过程中我自己又打错一次分支名，是执行跟踪抓到的，不是读代码。

**你会看到什么**：`fleet-status.sh` 无变化；你正在用的应用仍是 10-28 构建。以后不合规的新文档会在提交时被直接拒。

**需要你做什么**：两件——① 那 27 篇格式不合规的历史文档，是让 Mira 补头，还是由你裁定进豁免名单（我不替她决定）；② Nimbus 的删除守卫要不要合进 track。
