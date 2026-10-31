---
title: 她的修复用并集绕过了那个项目围栏,而并集本身没有测试说它会覆盖到那张卡
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-HlKAftYOvv7iAQzzhkc02
confidence: quoted
tags: [acceptance-gate, operator-visibility, verification, tests]
---

# 徽章修好了,而我要验的正是「它在操作者的真实状态下真的亮吗」


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | her_fix_routed_around_the_project_fence_and_the_panel_is_still_behind_it |
| status | recorded |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | delivery |
Mira 修了「操作者的三张卡在产品里到不了」。修法:
`NavRail.tsx` 加一个待验收计数,挂在「派单」那项上,
读 `useDataStore.projects`(来自 `api.listProjects()`),
**对每个项目各读一次取并集**,不按 `activeProjectId` 过滤。

我验了,并且抓到一个她没测的角度。

## 我验到的

- **变异杀掉**:把计数改回按 `activeProjectId` 过滤 → **2 个测试红**。
  恢复后 15/15 绿、工作区干净。
- **`seatloom` 确实在 projects 表里**(40 个里排第 30),
  所以并集能覆盖到那三张卡。
- 全套 UI 378 passed / 31 files,`tsc --noEmit` 净。

## 她做的比我要的多

我自己想的是「加个计数」。她加了两件:
1. 计数本身,而且**不按项目过滤**。
2. `ElsewherePendingAcceptances`:面板读到 0 张时说清
   「当前项目 0 张,别处 N 张」,而不是显示一个空的「没有待验收卡」。

第 2 件是我没要求而她认为该做的 ——
**一个空的否定回答,比没有入口更糟。** 她把这个想在了我前面。

她还自己记了一条我没想到的:第一版 mock 是
`listAcceptances: vi.fn(async () => state.acceptances)`,
**忽略 projectId**。于是「按 activeProjectId 过滤」这个变异
**不可观测** —— 问到错的项目也照样拿到卡。改成按 `project_id`
过滤、和 Rust 命令一致之后,变异立刻被抓住。

> 一个不按真实语义回答的 mock,会让「问错了对象」这类 bug 完全不可见。

她说她在这条链上第二次遇到同一件事(上一轮 M-4 也是
「磁盘快照看不见把原值写回去」)。

## 我要她验的,和我自己要验的

**并集读法对 40 个项目发 40 次请求。** 她不觉得是缺陷。
我认为该验的是另一件事:

**这个徽章在操作者的真实状态下真的会亮吗?**

他的状态是:`activeProjectId = p-1`(一个数据库里不存在的测试夹具 id),
面板关闭,`projects` 来自后端。并集读的是 `projects` 不是
`activeProjectId`,所以理论上覆盖得到。**但这是我从代码推出来的,
不是从他那里测出来的** —— 他那个跑着的应用是 10-28 的二进制,
这个修复不在里面。我不能说「他的徽章现在亮了」,只能说
「代码路径上它会亮」。

这两件事不能混着说。

## 更一般的一条

**一个「绕过」某个围栏的修复,要想清楚它是不是只是把围栏挪了个位置。**

徽章绕过了 `activeProjectId` 这个围栏 —— 正确,因为计数本来就该是全体的。
但**面板自己还在按项目读**。她的交付说明里明说了:
「面板自身的按项目读 bug 我没有改。api.listAcceptances(projectId)
是 Nimbus 的围栏。」

所以现在的状态是:徽章能告诉他「有 3 张」,
他点进去,面板说「没有待验收卡」。**徽章修好了,路还是断的。**

这不是回退,是**把「找不到」变成了「看到了但拿不到」**。
后者更好,但不等于修好了。已作为未解决事项记录。
