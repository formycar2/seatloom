---
id: a_seat_that_cannot_reach_the_model_looks_online_forever
title: 席位连不上模型时,fleet-status 永远显示「在线」—— 我加了一条能报错的检查
curation: authored
confidence: inferred
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1
created_at: 2026-10-28 00:20:00
tags: [monitoring, false-green, model-proxy, stall, self-inflicted]
---

# 席位连不上模型时,fleet-status 永远显示「在线」

2026-10-28 00:00 心跳。协议第 1 步说「进程活着是存活信号,**不是进度信号**。
账本最后一条事件的时间比面板可靠」。我照做了,查到 Flux 和 Nimbus 两件
`in_flight` 已经 120 分钟没动。

## 真因不是它们卡住,是连不上模型

面板里刷着同一句:

```
 Error: All fallback models failed (last: network unreachable).
       Chain: stepplan/step-5-preview → stepplan-wash/step-5-preview
 Error: Retry failed after 3 attempts: Connection error.
```

三个席位中招:Flux(15 行 error)、Nimbus(16)、QL(20)。
另外六个席位 error=0 —— 但**它们不是没报错,是根本没在跑**。

## 我第一反应查错了方向

先怀疑 SeatLoom 自己:`hub.i.basemind.com` ping 不通、`nc 443` 不通。
**但那个域名跟模型链路无关** —— 它是镜像仓库地址,不是推理端点。
在无关的东西上花了两轮。

真正的端点在 `~/.pi/agent/models.json`:

```
stepplan-wash-anthropic   https://api.stepfun.com/step_plan
```

实测那个地址**是通的**:DNS 解析到 14.103.2.83,443 开放,
`/step_plan/v1/models` 返回 401(活着,只是没带 key)。

**所以是间歇性的,不是断网。** 这也解释了为什么我自己的回合一直正常 ——
我用的就是同一个 provider、同一个模型。

## 为什么 fleet-status 看不出来

`scripts/fleet-status.sh:73`:

```bash
proc=""; [ -n "$pane" ] && proc=$(pgrep -P "$pane" 2>/dev/null | head -1)
if [ -n "$proc" ]; then alive=$((alive+1)); up="在线"; else up="❌掉线"; fi
```

**「在线」的判据只有一个:pane 里有没有子进程。**
一个 agent 进程卡在 `Connection error` 重试里,`pgrep -P` 照样返回它,
于是永远「在线」、永远「干活中(02:00 前开始)」。

这正是协议警告的那件事,只是换了个形态:**进程活着 ≠ 在推进**。

## 我做了什么

按协议第 4 步「叫醒它是安全的兜底」poke 了两个席位。
75 秒后:

| 席位 | 前 | 后 |
|---|---|---|
| Nimbus | R303M | R305M(在跑 `sed` 看测试) |
| Flux | R197M | R199M(在跑变异测试,M7 applied → 14 passed) |

**token 计数器在动 = 真的在做模型调用。** 这是这次唯一有效的判据:
不是看进程、不是看面板文字、不是看 `in_flight` 年龄,而是看**上下文读数有没有涨**。

## 判据(以后照这个做)

判断一个席位是「卡住」还是「活着但慢」,看它的 token 计数器:

```bash
tmux capture-pane -p -t <session>:0 -S -4 | tr -d '\r' | grep -oE 'R[0-9.]+[Mk]'
```

隔 60~90 秒读两次。**涨了就是活着**,不管面板上刷的是什么错。
两次读数不变,才是真的卡住,那时才需要人介入。

另外:**报错信息的措辞会把人带偏。** 这次刷的是 "network unreachable",
很容易理解成"公司网断了",进而去 ping 内网域名。
**先量端点,再信措辞。**
