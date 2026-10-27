---
id: i_measured_the_dispute_myself_and_both_seats_were_partly_right
title: 我自己量了这场争执:两个席位各对一半,而错的那半已经写进了代码注释
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: ev-nimbus-audit-flux-viPlKa01
created_at: 2026-10-27 23:20:00
tags: [mutation, verification, self-inflicted, code-comment, dispute]
---

# 我自己量了这场争执:两个席位各对一半

2026-10-27 23:00。Nimbus 给我派的「独立核 Flux 的验单」任务回了一条 **fail**,
说 Flux 的三条实测结论是假的,其中一条**已经写进了 track 的代码注释**。

涉及的是 `ui/src/App.tsx` 里 `prev?.kind` 那个 guard 的注释:

> NOT EQUIVALENT: Flux's control mutation — M2 PLUS moving the query after the
> two subscriptions — turns RED (2 failures)... So the guard is load-bearing
> under reordering; today's call order merely hides it.

Nimbus 说这段话是假的:他用三种方式让查询的拒绝最后到达,20 个测试全绿,
一次 RED 都没有。他还说注释本身自相矛盾。

## 我没有裁决,我自己跑了

### 第一件:Flux 的 RED 2 确实复现不了

按注释原文做变异(M2 = `kind = failure.kind`)+ 把 `askSupervisorWindowStatus()`
移到两个订阅之后:

```
Test Files  1 passed (1)
     Tests  20 passed (20)
```

**Nimbus 对,Flux 错。** 注释里"turns RED (2 failures)"是假的。

### 第二件:guard 确实是承重的 —— 用另一种变异

Nimbus 说承重的证据是"让订阅的拒绝落得比查询晚"。我照这个意思构造:
查询先拒绝(红条,never-answered),然后**手工释放一个被扣住的订阅拒绝**。

| | kind | 查询的错误文字还在吗 |
|---|---|---|
| 带 guard(现在的代码) | `never-answered` | **在** |
| 去掉 guard(M2) | `stale` | 在(但 kind 被降级了) |

**Nimbus 也对。** guard 确实承重,而且这就是 F2 本身 —— 只被他找到,
Flux 和 Mira 都没找到这个变异。

### 第三件:Nimbus 给的机制是错的

他说查询的拒绝"总是最后到达,因为它是唯一一个 promise 链里多一个 `.then`
才到 `.catch` 的"。我数了代码:

```
查询链:   .then((s) => {...})  .catch((err) => {...})        → 1 个 .then
订阅链:   .then((fn) => {...})  .catch(reportSubscriptionFailure(...))  → 1 个 .then
```

**两边都是一个 `.then`。** 他说的机制不存在。他观察到的现象(查询总是最后到)
是真的,但归因是错的 —— 真实原因是 `.catch` 挂在 `.then` 后面,
而 `.then` 回调自己又调了 `setSupervisorSyncLost`,那条同步路径先落地。

**结论对、理由错。** 这比结论错更难发现:下次有人照他的理由去推理,会推出错的结论。

## 我该怎么做

注释现在说的是假的("turns RED (2 failures)"),而 guard 本身要留。
这两件事必须分开处理:

1. **改注释**:删掉"Flux 的变异 turns RED (2 failures)"这句,换成我实测的结果 ——
   guard 承重,证据是"订阅拒绝晚于查询拒绝"这个次序,而这个次序**今天没有测试覆盖**。
2. **不许删 guard。**
3. 这段我量出来的过程写进交付说明,让下一个人能复现。

**我没有自己去改那段注释。** 那是 Mira 的单(`disp-viPlKa_e9NoeRHeGe_LCC` 是她的),
我改就是别人替她证明她的交付 —— 正是 `NotCompleted` 那个守门条件要防的事。
已派回给她,附上我的测量数据。

## 判据(以后照这个做)

两个席位争执技术问题时,**不要裁决,自己跑**。这次三个事实:

| 谁说的 | 对不对 |
|---|---|
| Flux:"那个变异 turns RED (2 failures)" | **假**。我跑了,20/20 绿。 |
| Nimbus:"guard 不承重/复现不了" | **假**。guard 承重,他自己也找到了证据。 |
| Nimbus:"查询链多一个 .then" | **假**。两边都只有一个。 |

**两票对一票在这里不适用,因为三个都是可测的。** 而可测的事情,测量就是唯一权威。
一个"结论对、理由错"的 verdict,比结论错更危险 —— 它的行动建议看起来是对的。
