---
title: 我读了测试自己的注释就相信了它,而那条性质没有任何断言钉住
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-oE0BHRvXSthXytwT_jnlt
confidence: quoted
tags: [mutation-testing, verification, tests, false-report]
---

# 我读了测试自己的注释就相信了它,于是向操作者报告了一个过头的结论


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | i_read_the_tests_own_comment_and_believed_it |
| status | recorded |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | delivery |
Nimbus 给 `close_topic` 补了原子性测试。我跑了、做了两个变异、都红、
恢复后干净,然后向张先生报告:**「那个核心主张现在真的被钉住了。」**

那句话说过头了。

## Flux 的第三个变异

测试用一个 `DEFERRABLE INITIALLY DEFERRED` 约束触发器在 COMMIT 时才失败。
这是对的,而且**测试文件自己的注释解释得很清楚**为什么必须是这样:

> A constraint that only bites at COMMIT is what makes the first statement
> visible-then-rolled-back.

Flux 把这个性质改松:

```
DEFERRABLE INITIALLY DEFERRED  →  DEFERRABLE INITIALLY IMMEDIATE
```

**两个测试全绿。**

改成 IMMEDIATE 之后,UPDATE 自己就被拒绝,根本不存在部分状态,
测试是因为一个**更弱的理由**过的。而「只在 COMMIT 时咬」正是这个测试
全部的意义所在 —— 它没有任何断言钉住。

我自己复现了:2 passed。恢复后 `git diff --stat` 空。

## 我为什么漏了它

我给 Nimbus 的两个变异是「删 UPDATE」和「删事件 append」——
都是**删代码**,都是我能从测试的断言里直接读出来的方向。

DEFERRED 这条不在任何断言里,**它只存在于一段注释里**。我读了那段注释,
觉得它写得对,就把它当成已经成立的事实。

**注释不是断言。** 一段解释「为什么必须这样」的注释,
和一条「这样被改变了就会红」的断言,是两回事。
前者可以被后人改松而悄无声息;后者不能。

## 这是同一条规律的第七个形状

「存活的变异总是在没人想到要测的方向」—— 我 10-29 就写了这条,
还专门派 Flux「不要只重跑作者的变异」。**结果我这次就是那个作者:**
我定了两个变异方向,验证者按我的方向跑完,又自己想了第三个,
而我给的两个他都跑过了。

我给的两个变异是「我能从断言里读出的方向」。
**断言里读不出的方向,就是没人测的方向。**

## 附带抓到的一件更普适的事

Flux 头两次跑这个测试,报的是 `test result: ok`,耗时 0.04s。
真相:scratch 库没有 026 的 TopicClosed,guard `return None`,
**两个测试跳过了**。他发现只是因为耗时不可能。

`return None` 让测试直接返回,在汇总行里和通过一模一样。
**一个会静默跳过的测试,比没有测试更糟** ——
它把「我没跑」显示成「我过了」。

这条不是 Nimbus 的缺陷(Flux 把它记成自己的测量笔记,没算他的),
但它属于同一个形状:**输出一样,事实相反。**

## 规矩

**向操作者报告「X 被钉住了」之前,必须先回答:X 的哪一条性质
是我从断言里读出来的,哪一条是我从注释里读出来的?**

后者一条都不能算钉住。要幺补断言,要幺在报告里写明
「这条目前只有注释,没有断言」——**不能因为它写在注释里就当它成立。**

**一个「删除型」变异只能证明「删了会红」。** 要证明一条性质被钉住,
变异必须**改松那条性质本身**,而不是删掉它周围的东西。
