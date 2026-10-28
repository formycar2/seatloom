---
id: i_handed_over_verdict_ids_without_checking_which_dispatch_each_one_names
title: 我把 verdict id 从查询结果里拿出来交给席位,没查它点名的是谁 —— 我自己的检查 40 分钟后抓住了
curation: authored
confidence: inferred
status: active
created_at: 2026-10-29
source_kind: canonical_event
source_ref: AEGIS-2026-10-29-hourly-heartbeat
tags: [verdict, acceptance-card, handoff, my-own-check, join-state]
---

# 我把 verdict id 从查询结果里拿出来交给席位,没查它点名的是谁

2026-10-29 02:00。我上一轮(01:00)发现"10 件 implementation 有 pass verdict、没有卡",
然后派席位开卡。我在派单里甚至写了这句:

> 其中两件共用同一条 verdict(`ev-pnBnPX1pdaRysj8UYW077`)——
> 如果它真的是关于两件的,两张卡都能指它;如果只关于一件,说出来。

**我知道有风险,写在纸上了,然后没有自己去查。**

40 分钟后,我 01:00 刚写进心跳的第 2c 步配的 `check-verdict-refs.sh` 抓住了:

```
BAD  acc-disp-b5VbRor1fHhP3nN4YZI_l
     the verdict does not name THIS dispatch - it names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-ibABeNk-nkEDAkI6Qj2sG
     the verdict does not name THIS dispatch - it names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-reemit-stalled-announcements
     the verdict does not name THIS dispatch - it names disp-integration-round-verify-v2
```

**三张卡,全部指错。** 而其中两张是我自己预警过的那两件。

## 真相

`ev-pnBnPX1pdaRysj8UYW077` 的 `payload.dispatch_id` 是 `disp-ySsxkvHxKurEgKpt7Q_Hm` ——
**Flux 验那两个洞的验活派单**。而我要 Nimbus 开卡的两件是
**Nimbus 修那两个洞的实施派单**。

它们是两件不同的活,在账本里是两个 id。verdict 关于前者,卡挂在后者。

我为什么会拿错:我的查询是

```sql
select e.id, e.payload->>'verdict'
from canonical_events e join event_object_refs r on r.event_id=e.id
where e.event_type='ReviewVerdictIssued' and r.ref_id='$d'
```

`event_object_refs` 把 verdict 和"它提到的任何东西"连起来,**不是只和它的 subject 连**。
Flux 的 verdict 在 refs 里提到了 Nimbus 的实施派单(作为 context),所以我的 join 抓到了它,
而它的 `payload.dispatch_id` 是它自己的派单。

**join 到 refs 表不等于 join 到 subject。** 我查了"有关联",没查"是关于谁"。

## 我不该自己撤卡

我第一反应是自己撤那两张卡,工具拒了:

> not a pending acceptance of a dispatch owned by "seat-aegis-001"
> (only the requesting seat may withdraw its own request)

**拒得对。** 那是 Nimbus 的卡。我不能替 seat 撤卡,和我不能替他们开卡是同一条规则。
我甚至刚在别处写过这条,然后差点绕过它。已派 Nimbus 自己撤。

## 那个检查为什么能抓住

因为我 01:00 把它写进了心跳协议,并且它 join 的是**两半边**:

```sql
exists(select 1 from canonical_events e
       where e.id=a.flux_verdict_ref and e.event_type='ReviewVerdictIssued')  -- 事件存在
exists(select 1 from canonical_events e
       join dispatches d on d.id = e.payload->>'dispatch_id'
       where e.id=a.flux_verdict_ref and d.id = a.dispatch_id)                -- 且点名这张卡的派单
```

第一条我跑过(所以我以为 verdict 有效),**第二条我今天之前从没跑过**。
一条 40 分钟前刚写下的检查,抓住了我 40 分钟前刚犯的错 —— 这不是运气,
是因为我把判据从"事件存在"换成了"事件是关于这张卡的"。

## 判据

**交出去之前,先跑一遍你准备给别人的那个判据。**

我给 Nimbus 的派单里写着"verdict 用 pass 那条",而我手上就有能验这件事的脚本。
我没跑,因为那脚本是 40 分钟前才写的、还没成习惯。

更直接的一条:**`event_object_refs` 上的 join 不是 subject 上的 join。**
想知道"这条 verdict 是关于哪件派单的",只能读 `payload.dispatch_id`。
refs 表里它会提到一堆东西 —— 那正是这个 bug 的形状。
