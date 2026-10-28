---
id: the_blocked_dispatch_was_asking_for_work_that_had_already_succeeded
title: 那件 blocked 派单要求的工作,本身已经成功过了 —— 重做会产生重复
curation: authored
confidence: inferred
status: active
created_at: 2026-10-29
source_kind: canonical_event
source_ref: AEGIS-2026-10-29-hourly-heartbeat
tags: [failed-dispatch, stale-blocker, vela, writein, idempotency]
---

# 那件 blocked 派单要求的工作,本身已经成功过了

2026-10-29 01:00。我发现两件 `disp-onight-writein` 失败原因是
"open_topic 工具不可用",量了三个只读判据(迁移在 track、vocab 里有 TopicOpened、
Mira 昨天成功发过 3 条),确认阻塞已消失,派 Vela 重做。

Vela 的回复:**她没有写任何东西,并解释了为什么。**

> 写库已经 live 了。它发生在 **2026-10-11**。那两件报 `BLOCKED-ENVIRONMENT` 的派单,
> 是在尝试做已经成功的工作。再写一次会产生重复。

## 她量到的

| 检查 | 结果 |
|---|---|
| `topics` where id like `wash-%` | **24** —— 正好是 manifest 的 1 root + 10 session + 13 segment |
| `memories` where `source_kind='archived_session'` | **90** 条,89 active + 1 forgotten |
| 逐 topic 比对 manifest vs live | **87 vs 88** |

那个 +1 不是漏记,是她 10-11 做插入后验证时**自己跑的幂等性探针**:
它把记录 `0198f196` 用改过的标题重写一遍,而 `remember` 没有服务端内容摘要去重,
于是产生了第二行。那条记录的标题 literally 写着
"(idempotency probe — expect refusal/duplicate, not a second row)"。

**她自己承认了这个探针造成的多余行,并且找到了它。** 这比一个干净的 87=87 更有价值。

## 我错在哪

我看到"阻塞消失"就派"重做"。但**阻塞消失不等于工作没做过** ——
那是两个独立的问题,而我默认了后者。

今天这已经是第四次遇到"失败原因过期":
- lineage 能力不在树里 → 现在在
- 读取界面不在 track → 现在在
- 016/017 没落地 → 现在在
- **open_topic 不可用 → 可用,但工作本身 10-11 就成功了**

前三次我重派了,第四次重派是错的。
区别不在阻塞的真假,在**工作是否已经完成**。

## 判据

**重派一件失败派单之前,除了问"阻塞还在吗",还要问"这件事做成过吗"。**

后者该怎么问:不要问席位"你做完了吗" —— 今天我已经见过一次,
用"有没有关联事件"查会得到 10 件全部非零(那些是派单生命周期事件)。
要问**产物在不在**:

```sql
select count(*) from memories where source_kind='archived_session';
select count(*) from topics where id like 'wash-%';
```

产物在,就别派。产物不在,才重派。

## 顺带:那条探针留下的问题

`remember` 没有服务端内容摘要去重,所以同样内容写两次会得两行。
这是 Vela 用一次探针暴露出来的真缺陷 —— 她因此多了一行,
而这个缺陷对任何重试路径都成立(一次网络超时后的重试就会产生重复)。

这条该修,但它不是她这件活的范围。已记下,没派。
