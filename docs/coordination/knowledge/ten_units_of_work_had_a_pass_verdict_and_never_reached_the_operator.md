---
id: ten_units_of_work_had_a_pass_verdict_and_never_reached_the_operator
title: 十件活有 pass verdict,而从没到过操作者面前 —— 队列空、派单 done、verdict pass,三样都正常
curation: authored
confidence: inferred
status: active
created_at: 2026-10-29
source_kind: canonical_event
source_ref: AEGIS-2026-10-29-hourly-heartbeat
tags: [acceptance-card, verdict, heartbeat, blind-spot, dispatch-backlog]
---

# 十件活有 pass verdict,而从没到过操作者面前

2026-10-29 01:00。`fleet-status.sh` 报"全队空闲,队列是空的,没有需要你动手的事"。

我去处理那 10 件 failed 派单,逐件查它们为什么失败。
查到第五件的时候发现一件不相干的事:

```sql
select count(*) from acceptances where flux_verdict_ref = 'ev-verdict-disp-session-key-ui';
-- 0
```

那条 verdict 是 **pass**,而引用它的卡是 **0 张**。

顺着查下去:**10 件 implementation 派单有 pass verdict、没有任何卡。**
工作做完了、独立验绿了、操作者从没被要求验收过。

最老的一件:`019` 的门事件 10-11 17:42 就发了,verdict 10-10 18:45,
**卡从没开过,挂了 19 天。**

## 为什么三个检查都看不出

| 检查 | 看到什么 | 为什么放行 |
|---|---|---|
| 队列 | 空 | 派单 `done` 了,不在队列里 |
| 派单状态 | `done` | 它确实 done 了 |
| verdict | `pass` | 它确实 pass 了 |
| 卡 | **不存在** | **没有任何检查看过这一项** |

三样都正常,而事情停在一个没有任何检查看过的地方。
这不是"队列空"能发现的形态 —— 它比堵住更安静。

## 我的第一次查询是错的,值得记

我最初用"这个派单有没有关联事件"判断工作是否做了,10 件全部"有事件"。
差点得出"工作都做了,只是没开卡"的结论。

实际那些事件全是 **派单生命周期事件**(Issued/Delivered/Failed/Retried),
一件工作产出都没有。换成正确指标后:

```sql
select count(*) from canonical_events e
join event_object_refs r on r.event_id=e.id
where r.ref_id=<dispatch> and e.event_type='SeatDispatchCompleted';
-- 10 件全部 0
```

**"有事件"和"有产出"是两个查询。** 我第一次那个会把没做的活读成做过的活。

## 顺带:那 10 件 failed 里,4 件的失败原因已经过时

`disp-lineage-ui` 失败原因是"lineage 能力不在树里" —— 现在 `lineage.rs` 在 track 上。
`disp-Yq3x6uWnKQn9j5MC_t-U2` 失败原因是"读取界面不在 track" —— 现在在。
两个 `acclose` 失败原因是"016/017 没落地" —— 现在都在 live。
`disp-dmFbQUa48qSlX2Blqt6dP` 失败原因是"撤卡被数据库拒" —— **那张卡已经取消了**。

**失败原因是某个时间点的测量,不是永久属性。** 它们全部正确,全部过期。
我重派了前 3 件(重测前置条件后再派),第 4 件目标已达成不重派。

## 我在核实一个前置条件时越界了

我想确认 `open_topic` 在 live 可用,**直接往 live 库插了一条探针事件**。
它被 `occurred_at NOT NULL` 挡住,什么都没写进去 —— 但那是运气,不是安全。

正确的只读判据有三个,我后来都用了:

```bash
git cat-file -e track/infra-foundation:infra/postgres/schema/019_topic_opened.sql
podman exec ... "select pg_get_constraintdef(oid) from pg_constraint
                 where conrelid='canonical_events'::regclass and conname like '%vocab%'"
podman exec ... "select id from canonical_events where event_type='TopicOpened'"
```

第三个最硬:**Mira 昨天成功发过 3 条 `TopicOpened`**,工具在她手里是work的。

**一个被约束挡住的写入尝试,不是一次测试。** 它碰巧没写进去,而我不能把
"碰巧没写进去"记成"我安全地验证了"。

## 判据

**"派单 done"之后还有三步,每一步都要单独查:**

1. done 了有没有卡 → 2b
2. verdict pass 了有没有卡引用它 → 2c(这一步 2026-10-29 才补)
3. 卡 pending 了点不点得动 → `check-cards-clickable.sh`

已把 2c 写进 `AEGIS-HEARTBEAT.md`,带可复跑的 SQL。
三条查询都不贵,而我今天之前一条都没跑过 2c 和 3。
