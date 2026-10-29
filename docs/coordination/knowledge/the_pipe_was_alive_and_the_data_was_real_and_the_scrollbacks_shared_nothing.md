---
title: "The pipe was alive and the data was real, and the two scrollbacks still shared nothing"
category: knowledge
topic: live-mirror
tags: [mirror, tmux, pipe, scrollback, history-limit, zero-overlap, false-cause]
created: 2026-10-29T05:10:00+08:00
source_kind: canonical_event
source_ref: ev-op-decision-ql-mirror-content-mismatch
confidence: inferred
curation: authored
---

# 管道是通的、数据是真的,而两边的滚动历史一点重合都没有

## 症状(张先生 2026-10-29 报的)

> QL 这个 Seat,虽然看上去能向上翻了,但实际上翻到的内容和 tmux 翻到的内容完全不一样

注意"**看上去能向上翻了**"——滚动条是活的,有内容,不是空白。所以不是断线。

## 我量到的硬事实

| 事实 | 数值 |
|---|---|
| QL 的 tmux 真实历史 | **1949 行**,内容「停 poison 看 after」「gang-t1/gang-t3 遗留任务」「A800 上 RUNNING 的 replica」 |
| QL 的 pipe 目标 | `ses-18e2b934c3113710.raw.log`,37 MB,10-29 01:47 建,还在写 |
| 那个文件确实是 QL | 2540 次出现 `project_delivery/QL` |
| **两边重叠** | 取 tmux 的 40 个实质句子去撞文件,**命中 0** |
| 文件开头内容 | `method = str(s.get("method","PUT")).upper()` —— 代码讨论 |
| QL 会话创建 | 10-26 10:58 |
| 该文件创建 | 10-29 01:47 |

**会话比文件早 2 天 15 小时。**

## 我排除的两个错因(都差点报出去)

### 错因一:"pipe 接到一个停写的死文件"

我一开始找到一个 900MB 的 `ses-18e24dac31302458`,里面 41 次命中 QL 独有内容,
最后写入 10-28 22:25(应用重启时刻),`cat` writer 起于 10-27 19:34。
看起来完美:QL 的 pipe 指向一个死文件,所以应用翻到的是旧历史。

**错。** 那个文件横跨 381MB–836MB 都有 QL 内容,是一个长期累积的旧转录。
真正活跃的是另一个文件,而 QL 的实时屏幕内容在里面能命中。

教训:**"停写"是文件的属性,不是 pipe 的归属。** 一个已停写的文件只能证明那个
session 结束了,不能证明当前 pipe 连的是它。要证明 pipe 归属,得用 pane 当前
屏幕的实时内容去撞文件 —— 我后面正是这么做的,才翻过来。

### 错因二:"跨线 bug 复发,pipe 接错了 pane"

我一度以为 `ses-18e2b92c0cab6368` 同时含 `project_delivery/QL` 和
`seatloom-worktrees/lyra` 两个 cwd 指纹,判定这是 10-27 那个跨线 bug 复发。

**也错。** 那是 saih TUI 的**单帧**里同时出现两个路径(可能是一次 grep 或一次
diff 的输出),不是两个 pane 的输出混在一起。后续按 QL 独有内容(`gang 任务`)
去数,该文件命中 **0 次**,而 QL 真实历史里是 22 次。

教训:**cwd 指纹按帧出现,一帧里可以有两个路径。** 判断跨线必须用
**只在某一个 pane 出现过的实质句子**,并且要给出命中次数。单次命中什么都
不证明,零命中才是否定。

## 真正确认的两个缺陷

### 1. tmux 侧只能翻几分钟(已确认)

```
QL-seatloom                 history_size=1949  history_limit=2000
Flux-Quality&Ops-seatloom   3033  2000
Nimbus-TechArchi-seatloom   3325  2000
... 九个席位全是 2000
seatloom-waker              188   50000     <- 只有它是 50000
```

我 10-28 把全局 `history-limit` 从 2000 改成 50000,以为修好了。
**但 pane 是更早创建的,window/pane 级的 `history-limit` 在创建时固化,
改全局对已存在的 pane 无效。** 九个席位一个都没享受到。

而应用侧是 9.5 小时 / 37 MB。**即使 pipe 完美,两边滚动条覆盖的窗口也差着近两个数量级。**

### 2. 两边内容零重叠(未定因,已问张先生)

pipe 通、数据真、文件确实是 QL,但和 tmux 现存历史**没有任何共同句子**。
两个可能:

- **A**:QL 的 pane 在 01:47 前后换过 agent 或断线重连,于是"同一个席位有两段
  历史,两边各显示一段"。这种情况下产品没坏,是不诚实 —— 它该说出来。
- **B**:pipe 接错 pane,10-27 跨线 bug 换了个位置复发。这个要立刻修。

**我在问清之前不动任何 pipe。** 动一下就把要比对现场毁掉了 ——
这和 10-27 我因为"随手跑了个 pipe-pane"而差点复现不出 bug 是同一个道理。

## 规矩

### 1. "停写"是文件属性,不是 pipe 归属

要证明某个 pipe 属于哪个 pane,拿那个 pane **当前屏幕的实时内容**去撞文件。
已停写的大文件往往是上一个 session 的遗产,不是当前的连接。

### 2. 跨线判定要用独有句子 + 命中次数,不能用 cwd 指纹

cwd 指纹按帧出现,一帧可以有多个路径。跨线的证据是:
**只在某一个 pane 出现过的实质句子,在可疑文件里命中 N 次,N > 0。**
单帧命中不证明任何事;要否定,给零命中。

### 3. 改全局 tmux 选项不等于改已存在的 pane

`history-limit` 是 window 级,创建时固化。要让已存在的 pane 生效,
得对每个 pane/window 单独 set,或者重建 session。
**验证方式:逐个 pane 读 `#{history_limit}`,不要读全局 `show-options -g`。**

### 4. "看上去能翻"不等于"翻到的是同一个东西"

滚动条能动、有内容、不是空白 —— 这三件事都成立,而用户看到的仍然可以是
完全另一段历史。**内容的同一性必须用句子命中去证,不能用"有内容"去证。**
