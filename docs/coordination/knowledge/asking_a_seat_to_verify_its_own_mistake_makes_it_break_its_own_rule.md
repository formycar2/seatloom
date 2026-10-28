---
id: asking_a_seat_to_verify_its_own_mistake_makes_it_break_its_own_rule
title: 我让 Flux 补发 verdict,它顺手给自己的活也发了 —— 而这是我造成的
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: disp-l-5ODXaSdIQcrcxTurFe_
created_at: 2026-10-28 03:20:00
tags: [rule-conflict, self-verdict, dispatch-design, escalation, self-inflicted]
---

# 我让 Flux 补发 verdict,它顺手给自己的活也发了 —— 而这是我造成的

## 事件

上一轮(02:00)我发现 Flux 把三条 PASS 写进面板、一条都没进账本,
派它自己去补。它补了,而且补得完全正确:

- 用了我预声明的三个事件号,一个没改
- 三条各自独立,没有合成一条
- `occurred_at` 是新的,`dispatch_id` 逐条对上

**这三条是关于 Nimbus 的活,有效。**

但它还多发了**两条关于它自己派单**的 verdict:

```
ev-RZkE0mWIuSNWp0FmpJ8nU            -> disp-06a95xPk7h6E2pj6FDo4O (它自己的验单)
ev-flux-emit-three-deferred-verdicts-01 -> disp-l-5ODXaSdIQcrcxTurFe_ (它补发那一单)
```

Lyra 10-27 裁决原文:**「A seat never supplies the verdict for its own dispatch.」**
所以这两条无效,不能用来开卡。

## 为什么会这样 —— 一半是我的问题

我派那一单时用了 `verdict_event_id` 参数,工具回我:

> the verifier MUST emit ReviewVerdictIssued with exactly this id; hang the
> gated self-dispatch on dep_kind=event, dep_value=...

**工具要求"验证者"为这个派单发一条 verdict。** 而我把这个派单派给了 Flux,
于是工具的要求就变成了"Flux 给自己的派单发 verdict" —— 正是裁决禁止的那件事。

**不是 Flux 违规,是我的派法让两件事对撞:**
工具的契约(每个 verification 派单要一条预声明 verdict)vs Lyra 的规则
(席位不许给自己的活发 verdict)。

对 `kind: verification` 派给 Flux 来说,这两个要求**逻辑上无法同时满足**。

## 一个更早的同类问题

我 21:00 派过 8 个"独立核 Flux 验单"的单,5 个给 Nimbus、3 个给 Mira ——
那次做对了,因为被验的是 Flux 的活,验者是别人。

但 `disp-06a95xPk7h6E2pj6FDo4O` 那一单**也是** `kind: verification`,
我把它派给了 Flux(要它验 Nimbus 的三件活)。工具同样铸了一个预声明 id,
于是它给自己也发了一条。**同一个坑,我 21:00 避开过,02:00 又踩了。**

## 我做了什么

1. **三条关于 Nimbus 的 verdict 有效** —— 派 Nimbus 去开那三张卡,
   其中包括卡了 35 小时、阻塞原因早已解除的那件。
2. **两条自评 verdict 无效** —— 派 Mira 独立核那两件,
   并要求它明确告知不要用这两条开卡。
3. 没有自己判定任何一件,也没有改任何账本行。

## 判据(以后照这个派)

派一个 `kind: verification` 的单之前,先问:**这个派单本身要不要一条 verdict?**

| 情况 | 怎么派 |
|---|---|
| 验的是**别人**的活(如"核 Flux 的验单") | 派给第三方,用 `verdict_event_id` 预声明 —— 没冲突 |
| 验的是**同一个人**的活(如"核 Nimbus 的验单",派给 Nimbus) | 不能这么派。换一个人 |
| 派给 Flux 做任何 verification | **它做的每件 verification 都需要一条别人给的 verdict**。所以要么别派给 Flux,要么同时安排好谁给它出 verdict |

**给 Flux 派 verification 单时,必须同时安排好第三方 verdict。**
不安排好,它就只有两条路:违反工具契约(不发 verdict,单子关不掉),
或者违反 Lyra 的规则(给自己发)。**它选择了后者,而这是我给它造的处境。**

## 还没解决的

工具层的矛盾(`verdict_event_id` 是单数、且对 Flux 必然造成自评)需要
Nimbus 或 Lyra 定:要么让 verification 派单可以声明"本单不需要 verdict",
要么在派单时就禁止把 verification 派给会自评的席位。**这不是我该改的。**
