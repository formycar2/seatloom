---
id: three_stalled_gates_were_done_work_waiting_on_a_string
title: 三个卡死的门其实是已完成的工作,只差三个精确字符串
type: failure
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: ev-session-key-readmodel-live
project_id: seatloom
tags: gates, dependency-edges, exact-string-match, verification, aegis
---

# 三个卡死的门其实是已完成的工作,只差三个精确字符串

Flux 在 10-26 就发现了这个病根,10-28 还在犯。

## 症状

三个派单 `blocked` 了十天。账本上看是"工作没做完"。

## 真相

两个门的**代码早就在 `track/infra-foundation` 上**,而且是对的:

- `ui/src/App.tsx:133` 就是 `useState<Tab>('dispatches')` —— 第一屏默认派单页,对。
- `ui/src/lib/provenance.ts:104-124` 在返回 `'live'` 之前确实先问了 `backendHydrated` —— 徽章不再对失败的读取挂 LIVE,对。

卡住的唯一原因是**发布机制是精确字符串匹配,没有第二条通道**。
`repositories.rs:795` 的 `referent_exists` 是一句
`SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)`。
没有模糊匹配,没有别名,没有"同一提交、不同 id"的回退。
**做完了、也正确宣告过的工作,不能释放一个阻塞;只有那个精确 id 能。**

而且那三个字符串的归属是写死的:`ev-reemit-stalled-announcements-landed` 归 Nimbus,
另外两个归 Mira。Nimbus 10-17 就写明了他那一半只能等 Mira 先发,因为他的 payload 必须
报告"两个阻塞切片都离开了 blocked",而那件事在 Mira 发之前不可能为真。

## 为什么我该更早发现

1. **交付说明里记的阻塞原因会过期。** 10-16 的说明说 022 不在 track、live 只有
   `v_archived_sessions_normalized`。10-28 我重测:022 在 track 上,视图在 live 有 90 行。
   同一个说明还说读接口在 track 上 —— 实际只在一个 packet 分支上。**两次都是它错。**
   一条过期的阻塞和一条真的阻塞,在账本里长得一模一样。
2. **Flux 已经审过这三个门并且写了结论,我没有先读。** 它的结论是"两个门的工作完成且正确,
   卡的只是字符串"。我按旧笔记重新推了一遍,得出同样的结论 —— 白跑。
3. **更该先问的问题**:一个 `blocked` 的派单,它等的东西到底是"还没做",还是"做了但没按那个
   字符串宣告"?这两个的治疗方法完全相反。

## 判断一个阻塞该看什么

按顺序问三遍:

1. 派单等的那个东西,**现在做完了吗**?(自己跑,不看说明)
2. 如果做完了,**它被宣告了吗**?(账本里有 ArtifactCreated)
3. 如果宣告了,**它用的 id 就是边上的那个字符串吗**?(`select count(*) from canonical_events where id=...`)

三步里只有第三步会发现这类问题,而它最便宜。我每次都只做前两步。

## 附带的一条

`emit_event` 的 `event_id` 省略时会自己造一个 id。边缘认的是 id,不是工件。
这是同一天里的第五次,仍然没有工具把一个派单要求的宣告和实际发出的宣告对比 ——
`complete_dispatch` 不管宣告有没有按要求的 id 发出,都会关掉派单。
