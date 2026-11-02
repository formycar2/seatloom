---
title: 出处真实存在不等于出处有效 —— 我引用了一份比张先生的要求更早的计划,还写着「出处」
created: 2026-11-02
author: aegis
source_kind: canonical_event
source_ref: AEGIS-2026-11-02-1130-heartbeat-the-fleet-is-alive
confidence: quoted
tags: [provenance, staleness, seats, model]
---

# 我给五个核心席位登记的模型,出处是真的,但出处过期了


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | a_dated_source_is_not_a_valid_source.zhs |
| status | recorded |
| author | aegis |
| date | 2026-11-02 |
| version | v1 |
| tags | delivery |
`register-fleet-seats.sh` 里我写着 `stepplan/step-5-preview`,
并且附了出处:10-08 的迁移计划 §4「Default workhorse: stepplan/step-5-preview (unlimited)」。
我还在注释里特意说明「不是我猜的」。

**出处是真的。那个文件真的那么写。** 但它比张先生 10-16 的要求更早:

| 日期 | 来源 | 值 |
|---|---|---|
| 10-08 | 我的迁移计划 | `stepplan/step-5-preview` |
| 10-16 | **张先生 的要求** | `modelproxy-anthropic/step-5-preview-sys` |
| 11-02 02:43 | 启动记录实测 | `modelproxy-anthropic/step-5-preview-sys` |

三个证据里两个指向 `modelproxy-anthropic`,只有我引的那份指向 `stepplan`。
**我把一份被后来的要求覆盖掉的计划当成权威来源,还标着「出处」。**

## 为什么这条比「出处是假的」更危险

出处假,别人会去查,然后发现它假。
**出处真但过期,别人查完会确认「确实有出处」,然后相信结论。**
我加的「不是我猜的」这句话,让这条错误的可信度比它应得的高得多。

## 现在造成的后果(不阻断,但要记账)

两个值都是合法的 `provider/model` 格式,两个 provider 都配着,
`seat-launch.sh` 都会接受,席位起得来。
但下一次启动,五个核心席位会从 `modelproxy-anthropic` **静默落到 `stepplan`** ——
换计费、换行为,而账本、启动记录、席位的实际模型三者会再次对不上。

## 规矩

**引用出处时必须同时给出它的日期,并说明它之后有没有更新的指令。**
一份没有日期的出处无法被判断有效性;
一份有日期但更早的出处,记录的是**当时的决定**,不是现在的决定。

出处是必要不充分条件。**它证明我没编,不证明我对。**
