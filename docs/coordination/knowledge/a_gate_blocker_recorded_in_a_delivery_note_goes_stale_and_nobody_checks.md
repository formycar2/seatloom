---
id: a_gate_blocker_recorded_in_a_delivery_note_goes_stale_and_nobody_checks
title: 门事件的阻塞原因写在交付说明里,过期三个月没人核 —— 我核了,三条里两条早就不成立
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-session-work-readmodel-gate
created_at: 2026-10-28 08:20:00
tags: [gate, stale-reason, blocked-dispatch, verification, unblock]
---

# 门事件的阻塞原因写在交付说明里,过期没人核

2026-10-28 08:00 心跳。队列空、23 张卡在等操作者、无事可做。
按协议「空闲是免费的,堵住不是」,我去查那三条卡了很久的活。

## 三条门的真实状态

| 门 | 账本记录 | 我的实测 |
|---|---|---|
| `ev-session-key-readmodel-live` | 0 行 | **阻塞原因已过期**,只差动手 |
| `ev-first-screen-default-tab-landed` | 0 行 | 无记录,不能发 |
| `ev-reemit-stalled-announcements-landed` | 0 行 | 无记录,不能发 |

## 第一条门的阻塞原因是假的

Nimbus 10-16 的交付说明第 8 节把门押后,写了三条理由:

> **The bridge view (022) is not applied to the live database.** Measured: live's
> `information_schema.views` holds only `v_archived_sessions_normalized`.

我现在实测:

| 它当时测的 | 我现在测的 |
|---|---|
| 022 不在 track 上 | **022 在 track 上**(`f7bfe23`,`git branch --contains` 列出) |
| live 只有 `v_archived_sessions_normalized` | **live 有 `v_memory_session_join`,90 行** |
| "A real call against live would fail with ViewMissing" | 视图在,能查,90 行 |

它自己列的解阻顺序是:022 上 track → 操作者跑 apply → 它做一次真实调用再发门。
**前两条早就发生了,第三条没人做,而门还挂着 fail。**

## 为什么没人发现

因为阻塞原因**写在交付说明的散文里**,不在任何会自动检查的地方。
门事件本身只有一行「没发」,没有「为什么没发」的字段。
所以每轮心跳看到的就是一个门关着,没有理由,也就没有"理由是否还成立"这一步。

**我前面十几轮心跳,每一轮都看到这三件 blocked,每一轮都当它们是"等前置条件"。**
没有人去核前置条件是否已经满足了。**一个过期的阻塞原因和一个真实的阻塞原因,
在账本上长得一模一样。**

## 但门现在还是不能发 —— 有新的真原因

我顺带查了那两个读取界面(一个给界面、一个给工具):

```
grep -rn session_work  五个席位工作区 + track   →  一个文件都没有
```

它们只存在于 10-16 那条分支的 pin `703c863` 上,那条分支**落后 track 367 个提交、
从来没合过**。所以现在是:**数据表 live 有 90 行,程序里没有任何东西能读它。**

门继续关着是对的 —— 但阻塞原因换了。已派 Nimbus 把界面弄进 track
(路径限定,附上我合并漏文件的教训),修 Flux 10-16 找到的 panic,
然后对 live 做一次真实调用,发门时附原文。

## 判据(以后照这个查)

查 blocked 派单,不能只念「等前置条件」,要**把前置条件逐条实测**:

```bash
# 1. 门事件在账本里吗?
select count(*) from canonical_events where id='<gate-id>';
# 2. 不在。那它依赖的东西现在 live 吗?(表的查 information_schema,
#    工具的查 tools/list,接口的 grep 命令注册表)
# 3. 交付说明里写的阻塞理由,逐条复测 —— 它是三个月前的一句话,不是事实。
```

**第 3 步是这次唯一有新发现的一步。** 前两步我每轮都做,第三步从来没做过。

## 顺带:一个"done"了但没有代码的单

`disp-session-key-readmodel` 在账本上 status=done,完成事件只有
`{"dispatch_id": "..."}` 一个空载荷。**而它的代码不在 track 上,也不在任何一个工作区。**
Flux 10-16 还在 Nimbus 的工作区验过它、给了 fail(F1 panic)。

所以它是「在分支上做过、验过、没合进来、账本上记 done」。
这又是 `a_verdict_written_in_the_pane_is_not_a_verdict` 那个病的变体:
**账本上的 done 不等于代码在主线。**
