---
id: i_could_not_reproduce_the_bug_until_i_used_the_command_that_caused_it
title: 我复现不了那个 bug,因为我用的不是导致它的那条命令
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [reproduction, tmux, pipe-pane, mutation-testing, verification]
---

# 我复现不了那个 bug,因为我用的不是导致它的那条命令

2026-10-28 22:00。我想验证 Nimbus 的 pipe 守卫真的挡住了张先生遇到的那个 bug。

我的复现:建一个真 tmux 会话、真 FIFO、真 attach 我们的 writer,
然后"从外部抢 pipe" —— 我跑的是:

```bash
tmux pipe-pane -t <pane> "cat >> /tmp/other.fifo"   # 带另一个 cat
```

**什么都没发生。** 两个 writer 并存,`pane_pipe` 还是 1。

我以为守卫是假的,差点去报"Nimbus 的修复没效果"。

## 真相

原始 bug 是**裸 `pipe-pane`(无参数)** —— 那是 tmux 的 **STOP 命令**,不是查询。
Nimbus 当时以为自己在查询,实际把 pipe 关了。

我带另一个 `cat` 的版本是"再加一个 writer",不是"抢走"。
**两者在 tmux 里是完全不同的操作。**

用对命令后,一次就复现了:

```
pane_pipe: 1 -> 0
ps 里我们的 cat: 还在,状态 S( sleeping ),不是 Z(僵尸)
```

## 顺带测出这个守卫唯一不可替代的部分

STOP 之后,`ps` 里那些 `cat` **还在跑、还在睡、还被列成我们 FIFO 的 writer**。

所以:**只看进程表的守卫会判"镜像健康",而当时一个字节都不在流。**
`pane_pipe` 是唯一说真话的信号。

这也是为什么 `pipe_guard` 把 `pane_pipe == 0` 放在最前面 —— 它不是一个
普通检查,它是这个守卫的**全部意义**。

## 变异测试抓出了我自己测试的假

我写了真实复现测试,跑通了。然后按惯例做变异:

| 变异 | 结果 |
|---|---|
| 把 `pane_pipe` 检查挪到进程表之后 | **存活** ❌ |
| 把 `pane_pipe` 检查整个删掉 | **被杀** ✅ |

第一行让我发现:我的测试**不是 load-bearing 的**。两个分支最后都返回 `Lost`,
所以挪位置不影响结果 —— 我的断言根本没有区分"只看 pane_pipe"和"也看进程表"。

补了一条断言(构造一个"有活 writer + pane_pipe=0"的快照),再变异:

```
left: Owned    right: Lost
test result: FAILED
```

现在它是真的。

## 判据

**复现一个 bug,必须用导致它的那条命令,不是听起来等价的那条。**

"抢 pipe"在我脑子里是"换个 writer",在 tmux 里"关掉 pipe"是另一回事。
我按自己的理解造了一个复现,得到"没复现",然后差点据此报"修复无效" ——
**一个按自己理解造出来的复现,和没复现是同一件事。**

更广的一条:**变异存活的时候,不要改变异,要改测试。**
第一直觉是"这个变异不够强",而真相是"我的断言不够强"。
一个通过了自己测试的错误结论,比没有测试更危险 ——
它读起来像证明。
