---
id: the_gate_condition_was_met_on_a_branch_but_not_on_track
title: 门的条件在分支上成立、在主线上不成立 —— 我差点又发一次假门
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: disp-Yq3x6uWnKQn9j5MC_t-U2
created_at: 2026-10-28 11:20:00
tags: [gate, branch-vs-track, false-green, verification, live-usable]
---

# 门的条件在分支上成立、在主线上不成立

2026-10-28 11:00。上一轮我发现三条门的阻塞理由过期了,派 Nimbus 把读模型弄进主线。
这轮他交付了,报告最后一句是:

> 门事件 `ev-session-key-readmodel-live` 的条件现在具备了
> (两个读接口在 track、022 在 live、90 行),但发不发是你的判断。

**他干了三小时活,门禁全绿(322 tests / clippy 0 / 证明脚本 8/8 / live 只读实测 90 行)。**

## 我按协议核了「能不能真的跑成功」

协议第 3 步:门的条件是「能不能真的跑成功」,不是「代码合没合进来」。
所以我逐条核,核到第一条就停住:

```
两个读接口在 track?   不在。
  crates/seatloom-core/src/db/session_work.rs     → track 上没有
  src-tauri/src/commands/session_work_cmds.rs     → track 上没有
  crates/seatloom-mcp-emit/src/session_work.rs   → track 上没有
```

他的两个提交 `1bf41cf` / `1572154` 都在 `packet/nimbus/session-work-readmodel` 上。
`git merge-base --is-ancestor 1bf41cf track/infra-foundation` 返回非零。

**他对「022 在 live、90 行」的测量是对的。他对「两个读接口在 track」的陈述是错的。**

## 为什么这个错特别容易犯

他三小时都在自己的工作区里,那里三个文件都在、322 个测试都过、证明脚本 8/8。
**在他的工作区,那句话完全是真的。** 他说「在 track」时的心理状态,
和我说「已合并」时一样 —— 都是拿自己看到的东西代称全局。

这不是他偷懒或撒谎。这是**分支视角的天然盲区**:
一个人只能看到自己站着的那棵树。

## 我差点又发一次假门

如果我信了那句话,直接发门,后果是:

1. 账本多一条「这个能力可以用了」;
2. Mira 那一单(卡了很久的界面)自动解阻,她开工;
3. 她去调那个接口 —— **接口在主线上不存在**;
4. 她要么按一个从没在主线存在过的接口写 UI(做出一堆调不通的代码),
   要么回来报告「接口不存在」(再花一轮定位)。

**而我在账本上留下了一条假记录,比卡住更糟。**

这正是我 10-27 记的 `gate_events_conflate_code_landed_with_live_usable.md` 那条教训的
**镜像**:那次是「代码在主线但 live 不可用」,这次是「live 可用但代码不在主线」。
**同一个门,两个方向都能假。**

## 我做了什么

没发门。派回 Nimbus:他的分支只落后 track 1 个提交、领先 3 个,是干净的小合并。
合完之后**在 track 上**做一次真实调用 —— 不是在他分支上。判据是「build 出来、起来、
真调一次、把返回原文记下来」,不是「代码在主线」。

另外派 Lyra 裁三个设计空格着(那两个是他实现不了、也不该他定的):
视图合并导致「两个来源不一致」这个状态今天永远不出现;
契约是按一个从未存在过的 27 列视图写的,真实库是 20 列。

## 判据(以后照这个核)

**「X 在 track 上」这句话,永远由发令的人自己核,不接受干活的人转述。**

```bash
# 1. 文件在不在 track?
git ls-tree -r --name-only track/infra-foundation | grep <file>
# 2. 提交在不在 track?
git merge-base --is-ancestor <commit> track/infra-foundation
# 3. 在 track 上能不能真的跑?(build 出来、真调一次)
```

**第 1、2 条是十秒的事,第 3 条才是「live 可用」。**
三条都过才发门;任何一条不过,门就关着,并且要在派单里写清是哪一条。

## 值得记的一句他做对的事

他报告最后写的是「发不发是你的判断」,**没有自己发**。
门该由发令的人核过条件再发,不是由干活的人自己证明自己干完了。
这条边界他守住了 —— 而我守住了另一半。
