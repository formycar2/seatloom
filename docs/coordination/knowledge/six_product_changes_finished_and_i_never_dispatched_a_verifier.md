---
title: "Six real product changes finished, and I never dispatched Flux nor had anyone open a card"
category: knowledge
topic: acceptance-integrity
tags: [done-without-card, verification, backlog, self-inflicted, aegis-neglect]
created: 2026-10-29T07:10:00+08:00
source_kind: canonical_event
source_ref: ev-six-product-changes-never-reached-the-operator
confidence: inferred
curation: authored
---

# 六件真产品改动做完了,而我没派 Flux 验,也没让任何人开卡

## 12:00 心跳量到的

全队空闲、队列空、四项检查全绿。看起来健康。

然后我按心跳协议第 2c 步之外多问了一句:**最近 done 的 implementation 派单,
哪些没有验收卡?** 结果 66 件。

大部分是我自己派的同步单(不该有卡,正确)。剔掉之后剩 6 件真工作:

| 派单 | 席位 | 内容 |
|---|---|---|
| `disp-H7pVqxTHO077Rlp6GrmzX` | nimbus | 镜像滚动条报真实深度并标注是回填(`b6ef3aa`) |
| `disp-TbGRGr5cvpJFgCeaayu7S` | nimbus | 主题能关闭了(`57bda95`) |
| `disp-rKdMdlyZ_LSFpQeJwv8XU` | nimbus | 去重测试不再匹配自己的断言文本(`01517af`) |
| `disp-TRxqo8RDIyg1ycBj36NXb` | onyx | 验收方案里的 SQL 换成应用内操作 |
| `disp-E06O6IAaWefF-BOW-CNJd` | mira | 默认落地页被持久化覆盖(那张卡断言是假的) |
| `disp-xoqVoiMs4HnL_EZn7-R5S` | vela | saih 归档裁定 |

**6 件,0 个 verdict,0 张卡。** 全都没到过张先生手上。

## 这是我的失职,而且不是新错

我在 10-28 专门建了 `check-verdict-refs.sh` 和心跳第 2c 步,抓的正是
"有 pass verdict 但没有卡引用它"。**这次的形态是上游一格:连 verdict 都没有。**

四项机械检查全绿,因为它们各自只问一个问题:

- `check-cards-clickable.sh` 问"已开的卡点得动吗"
- `check-verdict-refs.sh` 问"已开的卡的 verdict 指对了吗"
- `check-card-verifications.sh` 问"已开的卡的验法是不是废的"

**三个检查都只覆盖"已经开到张先生面前的那 7 张"。** 一个还没开到那儿的活,
三个检查都不看不见它。而"全绿"读起来就是"没问题"。

**和 10-28 那次 `fleet-status.sh` 只查 base 分支存在、于是 23 张点不动的卡被打成
"✅ 可以点"是同一个病:检查的覆盖范围比它声称的小,而它不报错。**

## 为什么我会漏

我派活的时候心里想的是"这个问题有解了",没有想"解完了怎么到张先生手上"。
**派单即进展,是我的错觉。** 对张先生来说,一件活在他点 Accept 之前不存在。

## 修法

### 1. 心跳协议加一步:done 无卡清单

不只看"有 verdict 无卡",要看**"有 done 无 verdict 无卡"** —— 更上游那一格。
而且要能区分同步单(我派的,不该有卡)和真工作,否则 66 行噪音淹掉 6 行信号。

### 2. 派 implementation 派单时,同时记下它需要一个 verdict 和一张卡

我在派单 payload 里写清楚这是产品改动、需要 Flux 验、需要开卡。
**当时不写,后面就会忘,而忘了不报错。**

### 3. 全绿不等于没问题

四项检查全绿的那一分钟,有 6 件真改动在等一个从没被派出的验绿单。
**绿灯的数量是已知问题的数量,不是健康程度的度量。**

## 现在补上的

按"一个验绿单位 = 一件派单 = 一个预先声明的 verdict id"逐件派 Flux,没有批量:

| verdict id | 验什么 |
|---|---|
| `ev-sitJrfiH1K8-lLl0hmCbP` | 镜像滚动条报真实深度并标注是回填 |
| `ev-U9LHdjblV_bkLtnfjuNzh` | 主题能关闭了 |
| `ev-GcK0xPj526FQyp918Wdlc` | 去重测试不再自证 |
| `ev-jrxSQVGlifCj-0RZuwtbV` | 验收方案里的 SQL 换成应用内操作 |
| `ev-EUiQEB8IvQb3nxruwvI-k` | 默认落地页(那张卡断言是假的) |

第六件是裁定不是产品改动,Vela 已交付裁定文档,直接转成操作者裁定请求
`ev-op-decision-saih-in-archive`,不走验绿。

## Vela 那份裁定值得单独记一笔

她回答"saih 该不该进归档"的方式,是这个舰队该有的样子:

- 扫 40 个文件 51,748 条记录,按角色分类,而不是说"格式看起来差不多"
- **在她自己的会话文件上跑密钥扫描**,找到 38 内网 + 21 basic-auth,
  然后去量已归档语料的同形状密度(62MB codex 文件 32+26,15MB claude-code 9+79)
  —— 用对照证明"不是新风险类",而不是断言
- 对她想选的答案所依赖的那个反对意见(这些是活的工作状态,不该归档),
  **认真对待并给出拒绝的理由**,而不是忽略它
- 最后指出:**当前状态是三种选择里最坏的那个** —— 功能在、覆盖五个 harness、
  唯独漏掉每一个席位都在跑的那个,而产品不告诉你

我独立复验了她两条承重主张:归档表已有 413,703,196 字节 / 148,387 行的文件
(saih 最大 193MB,不是新量级);saih 行带 `type/version/id/timestamp/cwd`,
和现有适配器读的同形。两条都过。
