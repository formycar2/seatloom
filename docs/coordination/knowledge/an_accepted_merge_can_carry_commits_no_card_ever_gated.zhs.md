---
title: 一次接受的合并把 15 个提交带上 track,而只有 2 张卡看着它
created: 2026-10-30
author: aegis
source_kind: canonical_event
source_ref: disp-heartbeat-2026-10-30-1700
confidence: inferred
tags: [acceptance-gate, false-clean, integrity, heartbeat-2c]
---

# 一次「接受」把 15 个提交带上 track,而只有 2 张卡看着它


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | an_accepted_merge_can_carry_commits_no_card_ever_gated |
| status | recorded |
| author | aegis |
| date | 2026-10-30 |
| version | v1 |
| tags | track |
心跳第 2c 步(查"verdict pass 但没人验收")抓到四件**做完了、验绿了、
从没被要求操作者验收**的活。其中一件查下去,发现比"没开卡"更严重:

## 实测

`git log --oneline 980e490^1..980e490^2` → **15 个提交**被带进 track。

`980e490` 是 10-30 10:29 的合并,账本上 `merge_commit` 指着它的卡只有两张:
`acc-disp-H7pVqxTHO077Rlp6GrmzX`(镜像滚动历史)和
`acc-disp-pQ6kiHvOb-IrrfFW1m1Jx`(去重测试自己匹配自己)。

而另外 13 个提交里,至少这五个是**独立的、用户可感知的产品改动**,
每一件都该有它自己的一张卡:

| 提交 | 内容 | 有卡吗 |
|---|---|---|
| `57bda95` | Topics can be closed:主题能关闭了 | **没有** |
| `b6ef3aa` | 镜像滚动历史报真实深度,并说明自己是回填 | **没有** |
| `ee2d03b` | remember 拒绝重复写入 | **没有** |
| `01517af` | 去重测试不再匹配自己的断言文本 | **没有** |
| `e011c0f` | clamp 真的被调用了,unknown 不再显示成 0 行 | **没有** |

`57bda95` 尤其清楚:它的派单 `disp-TbGRGr5cvpJFgCeaayu7S` 有 pass verdict
(`ev-U9LHdjblV_bkLtnfjuNzh`),`acceptances` 表里查它 **0 行**。
我用"沿 first-parent 链倒推、找出 57bda95 成为祖先的最早那个 track 提交"
定位入口,而不是靠分支包含关系猜 —— 否则会得出"它 10-29 05:11 就在 track 上"
的错答案(它 10-29 05:11 只是**在 Nimbus 的分支上**,不在 track 上)。

## 这和"绿灯 = 健康"是同一个病的更上游一格

我此前的记录覆盖了三格:
1. 派单 done → 有活没人接
2. verdict pass 但没被卡引用 → 验绿了没人验收
3. 卡 pending → 卡点不点得动

这一格是: **卡 accepted,但那次接受顺手带上了别没被验收的提交。**
四个机械检查全绿 —— 因为 gate 的判据是"分支包含 base",一次 merge 满足它,
至于这次 merge 还带进了什么,**没有任何机制在看**。

## 规矩(补进心跳第 2c 步)

**"merge_commit 指向我"的卡有 N 张,不等于这次合并的 M 个提交都过过门。**
`M > N` 时,差值就是搭车进来的东西。

```bash
# 一次合并带进了多少提交
git rev-list --count <merge>^1..<merge>^2
# 其中哪些有自己的卡(用提交主题里的派单号或交付说明人工核对)
git log --oneline <merge>^1..<merge>^2
```

发现搭车,不是要撤回合并 —— 已经合了,撤回更伤。
正确动作是**给每件没卡的工作补派开卡单**,让操作者知道
track 上有这些改动、它们是为什么进来的、他是否接受。
隐瞒"它已经在 track 上了"比合并不好更糟。

## 还没解决的

- `packet/flux/intent-verdict` 落后 track **247** 个提交,它有 8 张已接受的卡,
  分支落后会让**未来**开在这分支上的卡点不动。这是结构后果,不是 Flux 的错。
- 四件没卡的活里,`disp-tVzSMjxpWpBQrW3dToQej`(Flux 分支同步)是杂务不是产品活,
  **不该开卡** —— 开卡等于要求操作者验收一次合并。
- Mira 的 `packet/mira/knowledge-panel` 落后 42,她有两件活没卡
  (`disp-faEWBt8kSGgv-6pJs_Rqw` 写盘路径测试、`disp-E06O6IAaWefF-BOW-CNJd`
  首屏默认页),开卡前必须先同步,否则开出来也点不动。
