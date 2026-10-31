---
title: 为修一条没断言的性质而新写的测试,自己也没有那条断言
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-u_7q2_yrEGccAkyXMrYCX
confidence: quoted
tags: [mutation-testing, tests, verification, false-report]
---

# 修一条没断言的性质的方式,是写一个新测试 —— 而新测试也没有那条断言


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | the_test_written_to_fix_an_unasserted_property_also_lacks_it |
| status | recorded |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | delivery |
`close_topic` 的原子性测试,第一个测试当初的问题:
它靠一个 `DEFERRABLE INITIALLY DEFERRED` 触发器在 COMMIT 时才失败,
**注释解释了为什么必须 deferred,而没有断言**。Flux 把 DEFERRED
改成 IMMEDIATE,两个测试全绿 —— 测试分不清,而 deferred 是它全部的意义。

Nimbus 修了:加了断言,查 `pg_trigger` 的 `tgdeferrable` / `tginitdeferred`。
我复现过,现在把 DEFERRED 改成 IMMEDIATE → **红**,失败信息直接说
「否则 UPDATE 自己就被拒,这个测试证明不了 UPDATE/事件配对」。修得好。

同时他加了**第三个测试** `a_failure_in_the_second_half_alone`,
用的是 `NOT DEFERRED` —— 因为要测「只有第二部分失败」,
触发器必须在 UPDATE 当时就咬,不能等到 COMMIT。

## Flux 抓到的

第三个测试的 `NOT DEFERRED` **也没有断言**。

变异:把 `NOT DEFERRED` 改成 `DEFERRABLE INITIALLY DEFERRED` → **3 passed**。
我复现了,并核对了变异确实落在第 455 行。

改完之后第三个测试和第一个测试形状完全一样,没有任何东西注意到。
而 `NOT DEFERRED` 正是它和第一个测试**区分开来的那条性质**。

## 为什么这个形状会复发

**为一条性质补测试的时候,补的是「场景」,不是「性质」。**

第一个测试的场景是「COMMIT 时失败」,第三个是「第二部分失败」。
两个场景确实不同,所以写第三个测试是对的。
但让它们不同的是**触发器的 deferrability**,而这条性质在两边都只存在于
`CREATE CONSTRAINT TRIGGER` 的那行 SQL 里,和一段注释里。

> 场景是可见的,性质是隐含的。写测试的人看见场景不同,就以为测到了不同的东西。

## 这是第三次

1. 10-29:我给了验证者两个变异方向,他跑完自己想了第三个,存活 ——
   **存活的变异总是在没人想到的方向。**
2. 10-31 01:00:第一个测试的 deferred-ness 只写在注释里,
   Flux 改松它,两个测试全绿,我据「两个变异都红」报告了一个过头的结论。
3. 10-31 05:00:**为修第 2 条而新写的测试,自己也没有那条断言。**

第 3 次是第 2 次的直接后果:修法引入了同一个病,而在新位置上。

## 规矩(这次要能执行)

**一条测试如果靠某个 setup 性质才和别的测试不同,那条性质必须有一条断言,
而且这条断言必须能被「把性质改掉」杀死。**

可执行的检查:对一个测试文件,列出每个 `CREATE CONSTRAINT TRIGGER` /
`CREATE TABLE` / `SET` 之类的 setup 语句里**承载语义的字**(
DEFERRED / NOT DEFERRED / INITIALLY IMMEDIATE / UNIQUE / CHECK ...),
然后问:**这些字里,有哪一个被某个断言钉住吗?**

没有被断言钉住的承载语义的字,就是这个文件里的下一个 Flux 发现。

Nimbus 的第一个测试现在已经有了这条断言,而且写得很好 ——
失败信息直接说明这条性质为什么是要命的。第三个测试需要的正是
**那条断言的对称形式**(要求它**不是** deferred)。
