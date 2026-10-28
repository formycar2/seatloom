---
id: the_waker_was_never_running_and_i_only_found_it_because_a_status_line_said_idle
title: 唤醒服务根本没在跑,而我是从一行"全队空闲"里才发现
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [waker, daemon, silent-failure, fleet-status, discovery]
---

# 唤醒服务根本没在跑,而我是从一行"全队空闲"里才发现

2026-10-28 16:00 心跳。`fleet-status.sh` 报:

```
⏸ 有活排队但没人在做 —— 可能需要唤醒(见下方「唤醒服务」)。
```

我去查唤醒服务,**发现它从来没在跑**:
`seatloom-waker` 二进制没有编译产物,没有进程,没有任何日志。

而我今天早些时候(以及之前很多轮)一直以为它在跑。

## 为什么一直没有发现

1. **`fleet-status.sh` 的判据是 `tmux has-session -t seatloom-waker`。**
   而 waker 是一个 **Rust 常驻守护进程**(`crates/seatloom-mcp-emit/src/bin/waker.rs`,
   1420 行,`main` 里直接 `loop {}`),不是一个 tmux 会话。
   我按它的建议建了一个同名 tmux 会话,`fleet-status.sh` 立刻报"在跑" ——
   **而那时二进制依然没跑。**
2. **我自己在手动 poke 席位。** 我每一轮都用 `poke-all-seats.sh` 叫人,
   所以活一直在推进,也就看不出"没人叫醒"这件事。
   **我手动做的事情,掩盖了自动机制失效的事实。**
3. **席位"在线"和席位"被叫醒"是两件事。** `fleet-status.sh` 报 6/6 在线,
   那是进程存活;活有没有被派下去,是另一件事。

## 我试 `--help` 时挂住了,而那是正确行为

`target/debug/seatloom-waker --help` 挂住不返回。
我第一反应是"卡住了",查了才发现:那个二进制**不处理任何命令行参数**,
进 `main` 就 `Config::from_env()` + `loop { serve_once(...) }`,是常驻守护进程。

给它 `--help` 不是问帮助,是**启动它**。挂住是对的。

**一个没有 `--help` 的守护进程,`--help` 就是启动它。** 这条我记下来,
因为下次还会想用 `--help` 探路。

## 现在的状态

已用 tmux 会话起起来(和其他服务一致的方式),日志:

```
seatloom-waker: 6 seat(s) mapped from .seatloom/dispatch-seats.tsv;
LISTEN dispatch_ready on localhost:5432/seatloom;
stale-after 900s, hard-floor 3600s, sweep every 90s, poke cooldown 120s
seatloom-waker: armed.
seatloom-waker: catch-up scan done (0 seat(s) nudged)
seatloom-waker: sweep: idle seat seat-lyra-001 has no ready dispatch and a
                    standing-duty document -> Lyra-po-seatloom
```

它真的在 poke:Lyra 的 pane 里收到了站岗职责提醒。

seat map 的 6 个会话名**全部正确**(正好是 SeatLoom 自己的席位,不含三个外部项目)。

## 判据

**"看起来正常"和"自动机制在跑"是两件事,而手动兜底会把前者的假象维持很久。**

我每一轮都手动 poke,所以从没见到"派了单但没人动"的症状 ——
不是没发生,是被我挡住了。**我做的事掩盖了系统的失效。**

反过来说:一个需要我每小时手动叫人的系统,不是"我在帮忙",是"自动叫醒没工作"。
这次是撞上"全队空闲"这行字才去查的,而那已经是 16:00 —— 它可能已经停了很久。
