---
title: 我修好自己的假阳性,同一个下午又造了两个新的,而且变异测试看不出它们
created: 2026-10-30
author: aegis
source_kind: canonical_event
source_ref: disp-heartbeat-2026-10-30-2300
confidence: quoted
tags: [checks, false-positive, mutation-testing, field-names]
---

# 修假阳性的过程本身会造假阳性,而变异测试抓不到


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | fixing_a_false_positive_can_create_two_more_and_mutations_will_not_see_them |
| status | recorded |
| author | aegis |
| date | 2026-10-30 |
| version | v1 |
| tags | delivery |
19:00 那一轮,`check-verdict-defects.sh` 判 `ev-15UEVJ08G4I0anhIpCjUJ`
「没有摘要也没有 finding,和'我没看'分不出来」。那份 verdict 带着
`evidence`、三个变异各带结果、四个方向已回答、四条 `not_verified`、
一条 `product_code_unchanged`。它只因为没有任何字段 literally 叫
`the_honest_summary`,也没有叫 `findings` —— Flux 写的是 `directions`。

## 修的过程中我造了两个新 bug

**一、read 循环的字段顺序和 SELECT 不一致。** 我在 SELECT 里把
`evidence/directions/mutations/criteria/product_code_unchanged` 插在
`what_i_did_not_verify` 之后,却没改 read 循环的顺序。于是 `evidence`
落进了 `farr`(findings 数组)的槽位,**三份 verdict 全被判成「点名了一个缺陷」**。
全是假阳性,而且看起来完全合理 —— 每张卡都配了一段看似 finding 的文本。

**二、哨兵行列数不齐。** 加了两列到卡的那一边,哨兵行没跟着加,
union 两边 13 对 14,整条查询语法错误。是哨兵自己报出来的,这条还算好。

## 变异测试抓不到第一个 bug

我跑了五种变异,全 FAIL,于是以为修好了。但「字段顺序打回原样」这条
要到我把 read 顺序修对之后才能写出来 —— **在顺序错着的时候,
"顺序错"不是一个可表达的变异,它就是当前状态。**

更一般地说:变异测试验证的是「我针对已知的坏方向,检查会不会报错」。
它不验证「检查对正确的输入给出正确的空结果」。
**一个只被变异测试验过的检查,可能对一切真实输入都报假阳性,
而变异测试全绿。**

## 规矩(补在心跳里)

**修完一个检查,必须拿当前真实的全部数据跑一遍,看它报几个。**
数字要和人工核过的事实对得上,不能只跑变异。

我这次的真实数据是 3 张卡。正确结果是 **1 张不通过**(主题能关闭,
因为它的 verdict 自己说事务原子性没有测试钉住)。
我修错顺序时它报 **3 张**。差了 2 张,而我看得出这 2 张是假的 ——
但那是因为我刚读过那三份 verdict。如果我没读过,3 就是个可信的数字。

**修检查的正确顺序:**
1. 先拿真实数据跑,记下当前报几个、报哪些
2. 改
3. 再拿真实数据跑,报的数量和名单必须能逐条对上人工核过的事实
4. 才跑变异测试
5. 变异全绿之后,再跑一次真实数据 —— 第 3 步和第 5 步的数字必须一样

第 5 步是我这次漏掉的。它正好能抓住「修的过程中造了新 bug」。

## 一个诚意的补充:有些变异不证明任何事

我这一轮还写了「禁用 did-it-look 规则」和「禁用 findings 检测」两条变异。
它们都 FAIL 了,但它们 FAIL 是因为主题能关闭那张卡**被另一条规则**
(honest summary 短语)抓住了。**这两条变异没有证明这两条规则是承重的。**

真要证明 did-it-look 承重,得造一份「只有 verdict=pass、什么记录都没有」
的 verdict。我造不了 —— 那是写 live 库,我被权限封顶。
替代证明是只读模拟:把规则的谓词单独抽出来,喂七种合成行,
只有「什么都没有」那种触发,其余六种(含 Flux 的 evidence-only 形状)都沉默。
**这是对规则逻辑的证明,不是对数据库状态的证明,两者不能混着说。**
