# Delivery for the 20:00 heartbeat — rule 2 no longer prints an OK it did not earn, and the hole is visible instead of silent

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-2000-rule-2-says-when-it-is-not-checking |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | heartbeat, guard, false-ok, my-own-error |

**Changed**: `.git/hooks/pre-commit` (git common dir, not a tracked repo file).
**Headline**: the guard that silently skipped itself on `track` now says so out loud, and it no longer prints an OK it did not earn. **The strict variant still needs the operator's ruling and I did not take it.**

---

## 1. Ledger and fleet

35 tables, query proven non-empty (`dispatches` = 11). Six seats online, pane children all `saih`. `in_flight` = 0, queue empty, nothing assigned to me. All eight checks exit 0; protocol 2b and 2c both return 0 rows. origin in sync (`0 0`).

## 2. What I changed, and the boundary I kept

Last round I reported that rule 2 — the guard against a glob taking another seat's files — **does not apply on `track/infra-foundation` at all**, because `SEAT` is derived from `packet/<seat>/` and track has no such prefix. I offered two fixes and said the strict one was not mine to choose.

**This round I did the safe half only:**

| before | after |
|---|---|
| `SEAT` empty → rule skipped **silently**, and the next line printed `OK - rule 2: no other seat's files were taken by a glob` | `SEAT` empty → prints a **NOTE** naming the branch, the reason, and the consequence |
| that OK was printed whether or not the rule ran | the OK is printed **only when `SEAT` is non-empty**, i.e. when the rule actually ran |

**I did not add the strict variant** (refusing other seats' directories on track). It can block every merge onto the integration line, which is precisely the failure Lyra's 11-01 correction prevented. That is a governed guard and the change is the operator's to authorise.

## 3. 🔴 The false OK I found while testing my own fix — same disease, third site today

After wiring the NOTE, the hook printed both:

```
pre-commit: NOTE - rule 2 is NOT APPLYING on branch 'track/infra-foundation' ...
pre-commit: OK - rule 2: no other seat's files were taken by a glob
```

**The OK contradicted the NOTE printed one line above it.** The rule had just been declared inapplicable and then declared passed. That is the same defect I fixed at 15:00 in `verify-docs.sh` and at 16:00 in the hook's card check — **an OK that describes a measurement that never happened** — appearing a third time, in code I had edited minutes earlier.

Fixed: the OK is now inside `if [ -n "$SEAT" ]`, so it can only print when the rule ran.

## 4. And the NOTE itself cried wolf at first

My first `n_other` counter matched `^docs/coordination/tasks/[^/]*/`, which **includes `tasks/aegis/` — my own directory**. So committing my own delivery note produced:

```
NOTE - rule 2 is NOT APPLYING ... 1 staged file(s) are under docs/coordination/tasks/<seat>/
```

A note that fires on every one of my own commits teaches the reader to ignore it, and then the real signal is ignored with it. **That is the exact failure I fixed at 16:00 in `check-unclaimed-work.sh`**, and I walked into it again within the same hour.

Fixed by excluding `tasks/aegis/`, then re-verified both directions.

## 5. Proof, both directions, after the fix

| scenario | result |
|---|---|
| on track, another seat's file staged (`tasks/lyra/…`, well-formed) | **NOTE printed, exit 0** — visible, not blocking |
| on track, only my own `tasks/aegis/` doc staged | **no NOTE**, exit 0 — no false alarm |
| `bash -n` hook | 0 |

Probe files removed; `git status` clean except the known untracked `scripts/bind-seatloom-seats.sh`.

## 6. What I did not do

- Did not add the strict rule-2 variant (needs the operator's ruling).
- Did not touch `DELETION_EXEMPTIONS`, `request_acceptance`'s `BranchNotSynced`, or the clickability criterion.
- Did not merge `6e04dae`, did not open cards, did not write to the ledger.
- Did not grandfather the 27 non-conforming docs.

## 7. Checks

| check | exit |
|---|---|
| `check-ledger-exists` (35 tables, proven non-empty) | 0 |
| `fleet-status.sh` | 0 |
| `check-salvage-fresh` / `check-cards-clickable` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable` / `test-clickability-criterion` / `check-unclaimed-work` | 0 each |
| protocol 2b / 2c | 0 rows each |
| hook: another seat's file on track | NOTE, exit 0 |
| hook: only my own doc on track | no NOTE, exit 0 |
| hook syntax | 0 |
| working tree | clean except known untracked |

## 8. For the operator — three lines

**变了什么**：上轮报告的那个洞，我把安全那一半补了——守卫在 track 上不再静默跳过，它会说清「这条规则没在跑、为什么、后果是什么」；同时它不再打印一句它没挣到的 OK。严格那一半（真的去拦）我没做，那可能拦死所有合并，是你才能定的。

**你会看到什么**：`fleet-status.sh` 无变化。以后在集成分支上提交时，如果暂存区里有别的席位目录的文件，你会看到一条 NOTE（不拦，只提醒）。

**需要你做什么**：三件——① 严格版要不要（在 track 上拒别席位目录的文件，代价是可能拦死合并）；② 那 27 篇存量不合规文档；③ Nimbus 的删除守卫要不要合进 track。

**另外要如实说一句**：这一轮我在自己刚改的代码里又造了一次「未被支撑的 OK」，还让提示音在自己提交时乱响。两类病我今晚都修过别的脚本，转头在自己的新代码里重犯。已连同修法一起记在 hook 注释里。
