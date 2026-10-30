---
title: 操作者的三张待验收卡在产品里找不到,而他的活跃项目是一个测试夹具的 id
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-QgtYChq29ccrCEB5ZCjAH
confidence: quoted
tags: [acceptance-gate, product, operator-visibility, false-clean]
---

# 他要验收的三张卡,在产品里没有任何一处能到达

Mira 写了四条产品发现。第二条是「操作者的卡在产品里找不到」。
我逐条核了,**两条她都对,而且我量到第三条,比她的框架更糟。**

## 她的两条(我逐条复现)

1. **面板默认关,而且从 localStorage 恢复成关。**
   实测他 live 的 `seatloom.supervisor.open = f`。
2. **面板按 `activeProjectId` 过滤。**
   `SupervisorIntent.tsx:176` 和 `AcceptanceSection.tsx:103`
   都调 `api.listAcceptances(activeProjectId)`。
   而三张 pending 卡的 `project_id` **全是 `seatloom`**
   (账本:`seatloom` pending 3 / accepted 68 / cancelled 9,没有别的项目有卡)。
3. 面板外面没有任何计数或徽章。我 grep 了 NavRail / layouts / components,0 命中。

## 我量到的第三条

他保存的项目 id 是 **`p-1`**,而 `p-1` 在数据库里**不是项目**:

- localStorage 里 `seatloom.supervisor.activeProjectId` 是 UTF-16LE
  的 `70 00 2d 00 31 00`,解出来是 `p-1`
- `select count(*) from projects where id='p-1'` → **0**
- `p-1` 来自 `ui/src/stores/useDataStore.ts:100`,
  是 `SEED_PROJECTS_FOR_TESTS` 的第一条 —— **一个测试夹具**
- `select count(*) from acceptances where project_id='p-1'` → 0

所以不是「过滤到了错的项目」,是**「指向了一个不存在的项目」**。

而 `useDataStore.ts:1848-1854` 的水合逻辑会把这个后端不返回的 id
**静默丢掉**、退回 `projects[0]` —— 那是 `(root)`,也不是 `seatloom`。

**两条路都到不了他那三张卡。**

## 为什么这是最要紧的一条

这不是一个按钮不好用。这是:产品里**唯一**能到达验收的路径,
从他的实际状态出发,**必然显示空**。

而「空的没有待验收卡」比「找不到入口」更糟 ——
前者是一个明确的否定回答。他会读成「没有我的事」,
然后去做别的。**一个静默的否定,比一个沉默的缺失更难被发现。**

## 和 finding 4 是同一个病根

Mira 的 finding 4 是「项目不存在时告知操作员,不只静默改数据」。
 finding 2 和它是同一件事的两面:

> 一个指向不存在之物的状态,被静默修正成了另一个东西。

静默修正是这里反复出现的形状 —— 它让「哪儿错了」从界面上消失,
只留在数据库里。我这周已经在账本上抓到过同形状的四次
(迁移没登记、卡没开、verdict 指错、合并搭车),**每一次都是
「界面说没问题,而库里有问题」。**

## 附带验了 finding 1,它修得好

SAMPLE 徽章那条:一个测试**按名字把七个真读 Postgres 的标签排除在外**,
并用一条断言强制执行这个排除。她改了。

我做了变异:`LIVE_TAB_BACKING` 里删掉 `inbox` → **2 个测试红**;
恢复后 13/13 绿、工作区干净。

**那条旧测试不只是在允许撒谎,它在强制撒谎。**
而新测试现在真的在守这件事。
