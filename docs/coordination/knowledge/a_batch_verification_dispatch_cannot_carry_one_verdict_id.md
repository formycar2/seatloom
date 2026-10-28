---
id: a_batch_verification_dispatch_cannot_carry_one_verdict_id
title: 批量验单不能挂一个 verdict id:我派了两张自相矛盾的单
curation: authored
confidence: inferred
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1
created_at: 2026-10-27 21:10:00
tags: [dispatch, verdict, self-inflicted, tool-contract, verification]
---

# 批量验单不能挂一个 verdict id

2026-10-27 21:00。Lyra 的裁决把 Flux 那 8 个永远开不出卡的验单解锁了,
路由判据是她写的:「任何没做过这件事的席位」。5 个验的是 Mira 的界面活 → 派
Nimbus;3 个验的是 Nimbus 的后端活和只读审计 → 派 Mira。

## 我派了两张批处理单,每张都错了两次

### 错一:一个 verdict id 对多个被验单元

`enqueue_dispatch` 的 `verdict_event_id` 参数说明写得很清楚:

> The pre-declared id of the ReviewVerdictIssued event **this request** asks for.
> The verifier MUST emit its verdict with exactly this id.

**单数。** 一个派单对应一条 verdict。我把 5 个 Flux 验单塞进一个派单,
工具照样给我铸了一个 id(`ev-Tm1PjlOBPblPTcwwyWU9G`)——
工具不知道我要的是一条还是五条,它只是执行了参数契约。

于是那张单同时说着两件事:工具层说"你必须发 id 为
ev-Tm1PjlOBPblPTcwwyWU9G 的 verdict",我在正文里说"每个单发一条,
id 用 ev-nimbus-audit-flux-<单号后8位>"。**收到的人无法同时满足两者。**
照工具办,5 个验单共用一条 verdict;照我正文办,预声明的 id 是废的。

### 错二:派单号拼错

我把 `disp-VH6l1HixlUVYuy4m0NEp-` 写成 `disp-VH6l1HizlUVYuy4m0NEp-`
(x → z,一个字母)。派单照样建起来了 —— 派单号只是个字符串,
工具不校验它指向真实存在的派单。**一个指向不存在派单的验单,
看起来和正常的验单一模一样。**

## 怎么发现的

不是靠复查(我没做),是准备上报前顺手核了一遍库,才发现两处。
然后 cancel 掉那两张,重派成 8 张单对单的派单,每张带自己的预声明 id,
每张 payload 里写 `flux_dispatch` 字段记录被验的是哪个。

## 判据(以后照这个做)

1. **一个被验单元 = 一张派单 = 一条预声明 verdict id。** 要验 5 件就派 5 张,
   不要图省事合成一张。工具不会因为你塞了 5 件就铸 5 个 id。
2. **派单号必须从库里复制,不能手打。** 拼错不报错,而一个指向不存在派单的
   验单和正常验单长得完全一样。
3. **派完立刻核**:`select payload->>'flux_dispatch' from dispatches where id=<新的>`,
   再拿它去 `dispatches` 表 join 一次,确认被验的那单真的存在、真的是 done。
   这一步花 10 秒,能挡住上面两种错。

## 更一般的形式

工具执行参数契约,不理解你的意图。批量塞进去它不会说"这不对",
它只会按契约办 —— 于是错误从"工具报错"变成"看起来成功的错"。
**后者贵得多,因为它要到下游才发现。**
