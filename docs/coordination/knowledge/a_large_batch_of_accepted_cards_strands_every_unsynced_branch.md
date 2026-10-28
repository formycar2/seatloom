---
id: a_large_batch_of_accepted_cards_strands_every_unsynced_branch
title: 一大批卡被同时接受,会让所有未同步的分支落后整个批量
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [branch-sync, acceptance-gate, batching, operator-accept, vela]
---

# 一大批卡被同时接受,会让所有未同步的分支落后整个批量

2026-10-28 19:00。张先生在 15:15 到 18:18 之间接受了 **27 张卡**。

三小时前,Vela 的分支落后 0,她的卡点得动。
三小时后,它落后 **173 个提交**,她的卡点不动。

不是她漂移了。她 15:15 同步过,而且是对的。
**是 track 在一次批量接受里前进了 173 个提交。**

## 这个结构性后果此前没被记录

我记录过"我提交会让分支落后 1 个",也记录过"守卫要测提交后的状态"。
但这次的原因完全不同:

- 落后 1 个提交 → 原因是**我**
- 落后 173 个提交 → 原因是**接受门的工作方式**:27 张卡各自合并,
  每个合并都是若干个提交,而未同步的分支一次性落后整个批量

而我此前派的同步单都是"你又落后 1 个",那种话术在这里是错的 ——
它暗示席位做错了什么。Vela 什么都没做错。

## 各分支的实际落后数(19:00 实测)

| 分支 | 落后 |
|---|---|
| `packet/vela/washing-pipeline` | 173 |
| `packet/lyra/milestone-summary` | 179 |
| `packet/nimbus/project-scoped-reads` | 157 |
| `packet/mira/knowledge-panel` | 121 |
| `packet/flux/intent-verdict` | 71 |

## 为什么只有 Vela 紧急

其他四个分支没有 pending 卡,落后不卡任何事。
**落后的严重程度 = 那个分支上有几张 pending 卡**,不是落后数本身。

所以我只派了 Vela 一张优先级 99 单。给其他分支派同步单是浪费它们的上下文。

## 派单时要说清的话

不能再说"你又落后 1 个提交"。这次的说法是:

> 不是你漂移了 —— 你 15:15 同步过、那时卡点得动。
> 之后操作者接受了 27 张卡,合并进 track 共 173 个提交,你又落后了。
> merge 一次(不要 rebase)。

**rebase 在这里尤其危险**:173 个提交的 rebase 又慢又会重写卡指向的 id。

## 判据

**看到"分支落后 N 个提交"时,先问 N 是哪儿来的,再决定要不要派人。**

- N=1~2,而且提交是我的 → 我造成的,我派同步、我在同一回合承担
- N=100+,而且中间有一批卡被接受 → 结构性后果,不是席位的错,
  话术要不同,而且只派有 pending 卡的那个席位

一个"落后数"数字本身不说明责任在谁。我把这当成席位的漂移去措辞,
会让一个什么都没做错的席位以为自己要挨说。
