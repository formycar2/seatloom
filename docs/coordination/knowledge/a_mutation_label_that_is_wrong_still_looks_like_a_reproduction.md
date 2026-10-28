---
id: a_mutation_label_that_is_wrong_still_looks_like_a_reproduction
source_kind: canonical_event
source_ref: ev-SNtpi-dbYOEjmkdz4fK7e
session_scope: seatloom
title: 变异标签写错了,复现结果对得上也照样是错的 —— Flux 抓到我这个
domain: failure
modality: failure
status: active
curation: authored
confidence: quoted
owner: aegis
created_by: aegis
created_at: 2026-10-27
evidence_refs:
  - disp-yqYB1iZlPYe7HceKux71D (Flux 的 verdict,含 q4_mutations 与 on_aegis_m1)
  - disp-6xJ-8eWA7-LtBDeW9G5Yj (Mira 的交付,M1 原文是「贴回 .catch(() => {})」)
---

## 发生了什么

Aegis 派 Flux 验 Mira 的 never-answered 修复,派单里写:
「她 4 个变异:M1 贴回 `.catch(() => {})`(F1 本身)……」

**这一句是错的。** 我在派单的别处把 M1 描述成「合并两条失败路径」,
然后照着自己那个错误描述去"复现"。

Flux 做了 Aegis 没做的事:他试了**五种**不同的合并方式
(状态侧、渲染侧、两侧、把查询失败路由到 stale reporter、去掉重试),
**每次都是 RED 2,从来不是 3**。

能给 RED 3 的变异是**恢复 `.catch(() => {})`**,也就是原 F1 本身。

## 为什么这件事危险

**我复现的结果(3 failed)和 Mira 报的(RED 3)对上了,所以我以为复现成功。**

但我实际注入的确实是"贴回 `.catch(() => {})`"—— **动作对,标签错。**
如果 Mira 的 M1 真是"合并两条路径",我的复现本该对不上,
而我大概会把对不上当成"她的声明夸大",而不是"我的标签写错了"。

**一个错的标签 + 一个对的结果,比一个错的标签 + 错的结果更难发现。**
它看起来完全像一次成功的复现。

## 判据(照这个做)

复现别人的变异时:

1. **变异要自己从原文构造,不从转述构造。** Mira 的交付里 M1 写的是
   「贴回 `.catch(() => {})`」,账本 verdict 的 `mutation_table` /
   `q4_mutations` 里有全表 —— 两处都有原文,我两处都没先读。
2. **报数字时要报变异内容**,不能只报 "RED 3"。
   Flux 报的是 `restore the query's .catch(() => {}) (the original F1) -> RED 3`
   —— 变异和结果绑在一起,标签错了当场就能看见。
3. **复现不出时,先怀疑自己的转述**,再怀疑对方的声明。
   我这次的次序恰好反了,只是运气好结果对上了。

## 顺带:Flux 这轮还示范了什么叫独立验

- 四种组合全跑(不是读交付说明里那张表,是自己驱动真实 App 测的)
- 确认我 M1 复现对的同时,指出我给它贴的标签错
- 抓出 F2(单槽位覆盖,后写的赢),并实测四种组合里最后那行的后果
- 给出修复方向时明确说"照她自己已经为两个订阅通道做的那套" ——
  不是另起一套设计
- 明确不动手改(派单邀请的是 finding 不是 edit)

## 一句话

**复现要复现"那个变异",不是复现"你印象里的那个变异"。**
数字对上了,不等于你做的是同一件事。
