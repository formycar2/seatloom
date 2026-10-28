---
id: symptom_gone_is_not_root_cause_gone
title: 症状消失,不等于病因消失
curation: authored
confidence: inferred
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1
created_at: 2026-10-27 19:05:00
tags: [incident, root-cause, honesty, false-green]
---

# 症状消失,不等于病因消失

2026-10-27 实时席位交叉串线,定位过程中症状自己消失了:Aegis 关掉自己 pane 的 pipe
之后,那个乱接的 `cat` 跟着死掉,app 的 reader 命中 EOF 把 FIFO 丢掉。

看起来"好了"。实际上:

- `Aegis pane_pipe=0`、`Nimbus pane_pipe=0`;
- app 从 8 个 FIFO 掉到 7 个;
- Nimbus 那一格现在是**死的**,不是正常的。

**两个 pipe 都断了,所以看不见串线了。** 这是"症状消失但病因还在"的典型形态。

## 规则

报"已修复"之前,必须回答:

1. 是什么让症状消失的?是修复,还是让承载症状的那条路断了?
2. 那条路断掉之后,**本来该有的东西还在不在**?(这里:Nimbus 的实时输出)

如果答案是"因为坏了所以看不见了",那要报的是**现在哪里是坏的**,
不是"已修复"。

## 更一般的形式

一个能自愈的症状比一个持续的症状更危险:持续的症状会一直催人去修,
自愈的症状会让人合入一个没修的 bug,然后在某个不凑巧的时刻复现,
而那时已经没有人盯着了。
