---
title: 两个远端,席位各推各的,于是同一份代码在一个远端上存在、在另一个上不存在
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-YTiXxngPup06yFBLY_cTV
confidence: quoted
tags: [git, remotes, integrity, policy]
---

# 两个远端,而没有任何规矩说该推哪个

这个仓库有两个远端:

- `origin` = github.com/formycar2/seatloom.git
- `gitlab` = gitlab.basemind.com/zhangxiaolong/seatloom.git

**没有任何规矩说分支该推到哪里。** 席位各推各的,于是同一个分支
在一个远端上是最新的、在另一个上是旧的,或者干脆不在。

## 实测(2026-10-31 02:00,只看带卡的分支)

| 分支 | 本地 | origin | gitlab |
|---|---|---|---|
| `packet/nimbus/project-scoped-reads` | `1a7c8f4` | `1a7c8f4` | `019eb9b` **差 2 个提交** |
| `packet/flux/intent-verdict` | `9dd4b7b` | **没有这个分支** | `53aaaff` |
| `packet/mira/knowledge-panel` | `04af484` | `04af484` | `04af484` |
| `packet/onyx/*`(三个) | — | 一致 | 一致 |
| `packet/vela/washing-pipeline` | `dd41974` | **没有这个分支** | `dd41974` |

把范围放宽到所有 `packet/` 分支,不一致的有 **43 个** ——
但绝大多数是没人碰的旧分支,不说明问题。**带卡的那几个才说明问题。**

## 为什么这不是「推一下就好了」

Flux 上一轮抓到 Nimbus 的分支「根本没推出去」,我照此派了单。
Nimbus 推了 —— **推到了 origin**。gitlab 还差两个提交。
他不是没推,是**没有一个地方告诉他该推哪儿**。

Vela 的分支反过来:只在 gitlab,origin 没有。
所以两个席位做了相反的选择,而两个选择都「对」。

## 风险的具体形状

1. **谁从另一个远端取代码,拿到的是旧的。** 现在没有第三个人
   在这个仓库上工作,所以还没造成损害 —— 但张先生的备份、
   将来的协作者、或者换一台机器,都会拿到不同的东西。
2. **「分支已推送」这个说法本身是假的**,除非说清推到哪个远端。
   Flux 上一轮的 finding 写「the branch is unpushed」,
   严格说也不准确 —— 它是「在一个远端上不存在」。
3. **我不能替他们定规矩。** 选哪个远端作为唯一真相源是张先生的事,
   涉及 github 是公开仓库而 gitlab 大概不是 —— 这是他的决定,不是我的。

## 已做的

派了 Nimbus 把 gitlab 补上(`disp-YTiXxngPup06yFBLY_cTV`),
并在派单里明说:如果想让我定规矩,他会说一声,我去问张先生。

## 待张先生决定

**分支该推到哪里?** 三个选项:
- 只推 gitlab(如果 github 那份只是公开镜像)
- 两个都推(费一倍时间,但不会再有这种分裂)
- 明确其中一个为真相源,另一个由某个机制同步

在他定之前,每个席位都在猜。
