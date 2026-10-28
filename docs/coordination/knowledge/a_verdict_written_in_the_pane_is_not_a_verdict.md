---
id: a_verdict_written_in_the_pane_is_not_a_verdict
title: 写在面板里的 verdict 不是 verdict —— Flux 干了活,账本里一件都没有
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: disp-06a95xPk7h6E2pj6FDo4O
created_at: 2026-10-28 02:20:00
tags: [ledger, verdict, pane-prose, silent-loss, verification]
---

# 写在面板里的 verdict 不是 verdict

2026-10-28 02:00 心跳。队列空了 —— 没有 in_flight、没有 ready。
我上一轮派了三张"开验收卡"的单,三张都 done 了,18 张卡确实开出来了。

但顺手一查,发现一件比"队列空"重要得多的事。

## Flux 干了活,但结论只活在面板里

派单 `disp-06a95xPk7h6E2pj6FDo4O` 要它对三件 Nimbus 的活各出一条独立 verdict。
它面板的滚动记录里写着:

```
 disp-XnCxvGI88oAbdGcyeefhl PASS — 我自己的 10-26 fail 重新判定,
   三个失败条件都关闭了
 disp-98QeI7oCe_posblRFa_nf PASS — 双渲染席位流
 disp-lu24NPShI-80fqWuvNZeq PASS — 两张卡都存在且已被接受
```

结论明确、有依据、有过程。**然后它去干别的了。**

账本里:

```sql
select id, payload->>'dispatch_id', payload->>'verdict'
  from canonical_events
 where event_type='ReviewVerdictIssued'
   and payload->>'dispatch_id' in ('disp-XnCxvGI88oAbdGcyeefhl',
                                   'disp-98QeI7oCe_posblRFa_nf',
                                   'disp-lu24NPShI-80fqWuvNZeq');
```

只返回 10-26 那条老的 **fail**。三条 pass 一条都不在。
我另外扫了它 400 行滚动记录找 `emit_event`:**0 次调用。**

## 后果不是"晚点到",是"对下游不存在"

| 事实 | 影响 |
|---|---|
| 三件活 pass verdict 数 = 0 | 开卡会被 `VerdictNotGreen` 拦(10-26 那条 fail 还挂着) |
| `disp-XnCxvGI88oAbdGcyeefhl` 的 fail 没被翻 | 它已阻塞 35 小时,而且阻塞原因早就解除了 |
| 它 measured 的工作 | **没有消失,但账本里查不到,所以等于不存在** |

它自己在面板末尾写"Queue is now empty, so I'm idle." ——
它认为自己做完了。**从它的视角看确实做完了**,结论都写在那儿。

## 这和协议警告的是同一件事,换了个位置

心跳协议第 1 步写着:

> 进程活着是存活信号,**不是进度信号**。账本最后一条事件的时间比面板可靠:
> `canonical_events` 空白 17 小时而面板显示"6/6 在线"真的发生过。

那次是"面板说有、账本说无"。**这次是"面板写了结论、账本没记录"。**
同一个病:面板是草稿纸,账本才是事实。写在草稿纸上的结论,
下游拿不到,也验不了。

## 我做了什么,没做什么

**做了**:派 Flux 自己去发那三条 verdict,给了三个预声明的事件 id,
并要求它发完贴查询结果。

**没做**:**没有替它发。** 那三件我一件都没验过 —— 我读的是它自己的结论。
我发就是假证据,而假证据比没有证据更糟(下游会以为有人核过了)。
这正是 `Aegis 不拥有那些单` 那条边界,和 `NotCompleted` 守门条件是同一个道理。

## 判据(以后照这个查)

一个席位报"做完了",要确认它做完的东西**在账本里**,不在面板里:

```bash
# 1. 它声称发了 verdict —— 查账本,别信滚动记录
select id, payload->>'dispatch_id', payload->>'verdict'
  from canonical_events
 where event_type='ReviewVerdictIssued' and payload->>'dispatch_id' = '<它验的那一件>';
# 2. 面板里的 ev- 编号不算数。滚动记录里出现 ev-xxx 只说明它提过这个字符串。
# 3. 队列空 ≠ 没事了。队列空只说明没有待领的活,不说明已完成的活有产物。
```

**"队列空"是一个特别危险的信号**:它看起来像"一切就绪",
实际上可能只是"有人把结论写在了没人读的地方"。
