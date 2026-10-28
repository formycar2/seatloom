---
id: a_verdict_that_names_a_dispatch_which_does_not_exist_is_not_a_verdict
title: 一条点名了不存在派单的 verdict,不是 verdict —— 而我今天又见到一次
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [verdict, typo, acceptance-gate, nanoid, third-party]
---

# 一条点名了不存在派单的 verdict,不是 verdict

2026-10-28 15:00。我在清理"干完了但没人接"的活,逐件查 verdict。

`disp-B4Y5oAhMQwtXm5Vva1fw` —— 这条 verdict 的 `payload.dispatch_id` 指向它。
查 `dispatches` 表:**0 行。**

正确拼写是 `disp-B4Y5oAohMQwtXm5Vva1fw` —— `o`,不是 `0`。

一个字符。`0`(零)和 `o`(字母 o)在等宽字体里几乎一样,
而派单 id 是 nanoid,字符集里同时有 `0` 和 `o`。

## 为什么这不是无害的

10-27 的裁决把 `flux_verdict_ref` 定为验收的硬条件:
verdict 必须点名**正在被验收的那个 dispatch**。

一条点名不存在派单的 verdict:

- 它**看起来**是验绿依据 —— 事件类型对、verdict=pass、actor 是第三方
- 它**不能**为任何卡提供依据,因为它点名的派单不存在
- 更糟:如果哪天真有一个 id 恰好长这样的派单,这条 verdict 会**突然变成**
  那个不相干派单的验绿依据

这就是"向账本谎称某事已发生"的一个具体形态:不是撒谎,
是**指向空处的指针被当成了证据**。

## 我今天怎么发现的

不是靠读,是靠**join**:

```sql
select e.id, e.payload->>'dispatch_id'
from canonical_events e
where e.event_type='ReviewVerdictIssued'
  and e.payload->>'dispatch_id' like '%B4Y5oA%';
-- 两条,一条拼错一条拼对
```

如果只查"有没有 verdict 事件",会查到 2 条,然后认为有依据。
只有把 `payload.dispatch_id` 拿回 `dispatches` 表 join,才发现其中一条指向 0 行。

## 和 10-27 那条病的关系

10-27 我记录的是「verdict 没点名任何东西」(空指针)。
这次是「verdict 点名了一个不存在的东西」(野指针)。
同一个家族的两种形态,而第二种更难发现 —— 空指针会报错,野指针不会。

## 判据

**verdict 的有效性不能停在"事件存在",要 join 到 `dispatches` 确认那行在。**

```sql
select count(*) from dispatches d
where d.id = (select payload->>'dispatch_id' from canonical_events where id=<verdict>);
```

`0` 就是无效,不管它的 verdict 字段写的是什么。

而根因那条我今天已经写过了:**派单 id 一律从账本复制,绝不手打。**
这条 verdict 是手打的,所以错。规则写了,还是会有人(包括我)手打 ——
所以 join 这一步不能省,它是唯一能抓住手打的那道网。
