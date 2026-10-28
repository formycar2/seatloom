---
id: a_growing_file_does_not_prove_the_pipe_is_plugged_into_the_right_pane
title: 文件在长,不等于管子接对了 pane
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1
created_at: 2026-10-27 19:00:00
tags: [tmux, pipe-pane, live-mirror, verification, false-green]
---

# 文件在长,不等于管子接对了 pane

2026-10-27,张先生报告"我跟 Aegis 说的话显示在 Nimbus 的实时席位里"。
Aegis 一度不信(第 4 次犯「不要怀疑操作者」的错),量到字节级才认。

## 事件

Nimbus 排障"哪个 FIFO 属于哪个面板"时,对着 `Aegis-Supervisor` 执行了
`tmux pipe-pane`(不带参数)。他以为那是**查询**,实际那是**停止** ——
app 正在用的管断了。他发现后"挂回 app 自己的 FIFO",并用
**"转录文件重新增长(5 秒 +71939 字节)"** 作为恢复正常的证据。

**他挂错了。** 他把 Aegis pane 的输出接到了 app 以为属于 Nimbus 的那个 FIFO
(`ses-18e24db955e0ad38.fifo`)。于是 Nimbus 那一格开始显示 Aegis 的全部输出,
包括张先生跟 Aegis 说的每一句话。

同一个 FIFO 的转录文件,180,986,013 字节:

| 字节区间 | 归属 |
|---|---|
| `0 → 6,819,895` (3.8%) | Nimbus 的 pane |
| `6,819,895 → 180,986,013` (96.2%) | Aegis 的 pane |

一次交接,没有重叠。

## 为什么那个检查看起来完全正常

"文件在长"测的是**这一端有数据在写**,完全测不出**写进来的是谁**。
他刚亲手接上那一端,文件当然会长。增长是真的,结论是假的。

这和 `a_directory_listing_cannot_prove_a_compressed_asset_shipped.md` 是同一族:
**检查测的是相邻的东西,不是被测的东西。**

## 判据(以后照这个写)

要证明"这个 FIFO 属于这个 pane",必须让两侧各自独有的指纹在对的位置出现:

```bash
# 只可能由 Aegis pane 打印的字符串
grep -c 'seatloom (track/infra-foundation)' <transcript>
# 只可能由 Nimbus pane 打印的字符串
grep -c 'seatloom-worktrees/nimbus (packet/' <transcript>
```

**增长、大小、mtime、`pane_pipe=1`,全部不足以证明接对了。**
`pane_pipe=1` 只说明"这个 pane 有 pipe",不说明"pipe 通向哪个 FIFO"。

## 顺带:tmux 每个 pane 只有一个 pipe

`pipe-pane` 不是引用计数,是"这个 pane 的输出复制到哪"。谁后写谁赢,
前一个 `cat` 当场死掉。`PtySession::kill()` 的文档早就写明了这条限制
("the pipe is per-pane, not per-client"),但产品对**外部进程抢 pipe** 零防御。

## 同一天 Aegis 自己又踩了一次

为了定位,Aegis 对自己的 pane 执行 `tmux pipe-pane`(不带参数)来关管验证 ——
同一个动作,又踩了一次。**读到教训不等于会照做**,这句话写在这里提醒下一个读的人。

查询 pipe 状态用 `display-message -p '#{pane_pipe}'`;
`pipe-pane` 不带参数是**停止**,永远不要用它来"看一下"。
