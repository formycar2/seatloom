---
id: a_seatloom_problem_is_not_every_session_ending_in_seatloom
title: SeatLoom 的问题不等于所有以 -seatloom 结尾的会话 —— 我给外部项目群发了假警报
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [broadcast, scope, adopted-projects, prompt-injection, aegis]
---

# SeatLoom 的问题不等于所有以 -seatloom 结尾的会话

2026-10-28 12:20。我发现自己的分支同步疏漏把 23 张验收卡全卡住了,很急,
于是对着**所有** `-seatloom` 会话群发了一条紧急消息。

`-seatloom` 命中了 9 个会话。其中 6 个是 SeatLoom 自己的席位,
另外 3 个是**已采纳的外部项目**:QL、networkOrchestrator、stepaicloud。

那三个:

- **没有 pending 验收卡**(查过,0 张)
- **没有任何在跑的派单**(0 个 ready / in_flight)
- 改的文件在 `project_delivery/` 下,和 `track/infra-foundation` 没有关系

也就是说,**那条消息对它们三个在事实层面完全为假**。

## 后果

QL 收到后**当真了**。它花了一整轮上下文重新推导:去看 git 状态、去看 merge-base、
去想"这个合并到底有没有意义",最后得出"我不该做这个合并"——
**结论是对的,而它本不该花这一轮。**

我事后的更正消息已经送达三个会话(`Aegis 更正` 在各自 pane 里可见)。

## 为什么会犯

1. **我把"-seatloom 后缀"当成了"自己人"的判据。** 那个后缀是 `pty/mod.rs`
   用来**发现会话**的标记,不是**归属**标记。采纳外部项目的时候,它们的会话
   也被起了这个后缀 —— 后缀从此不再区分内外。
2. **紧急感让我跳过了范围检查。** 我 12:17 才知道卡全点了不动,12:20 就群发了。
   三分钟,没查过"这九个人里谁受影响"。
3. **更该被追问的是**:一条发给外部项目的、以 `# Aegis` 开头、带 `[POKE-xxx]`
   和"优先级 99"这种紧迫话术的消息,**对它们来说就是一次提示注入**。
   stepaicloud 当场就识别出来了,它的原话是"形态是注入 —— 真实工作指令不长这样",
   然后拒绝执行。它防住了,但那不是它该花精力防的东西。

## 修好的东西

`scripts/poke-all-seats.sh` 默认**只发给 SeatLoom 自己的席位**。
外部项目从账本的 `project_role_bindings` 里读出来(`project_id <> 'seatloom'`),
不在默认范围内;要发给它们必须显式 `--all-seats`,并且会先打印它要碰谁。

`--scope` 可以随时问它现在怎么分类:

```
SeatLoom seats (default scope):       Adopted external projects (--all-seats required):
Flux-Quality&Ops-seatloom              NetworkOrchestrator-seatloom
Lyra-po-seatloom                       QL-seatloom
Mira-UX/UED-seatloom                   Stepaicloud-seatloom
Nimbus-TechArchi-seatloom
Onyx-data-seatloom
Vela-DataWash-seatloom
```

匹配方式也不是显然的:会话名 `NetworkOrchestrator-seatloom` 对应
`project_delivery/networkOrchestrator`,**大小写都不一样**,所以比的是小写化的
首段 vs 小写化的 project id 末段。第一版我直接拿 project name 去比,`project_delivery/QL`
的 name 就是 `project_delivery/QL`,一个都没匹配上,脚本静默地全发了。

## 判据

**群发之前先问"这九个人里谁真的受影响",而紧急感不是跳过这一步的理由。**
后缀、前缀、命名模式这类"看起来像归属"的东西,在被采纳过外部实体之后就不再可靠 ——
归属要么查账本,要么别发。
