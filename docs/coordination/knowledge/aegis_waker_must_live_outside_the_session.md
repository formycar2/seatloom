---
memory_id: aegis_waker_must_live_outside_the_session
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: hot
source_kind: canonical_event
source_ref: disp-aegis-pi-cutover
l2_ref_kind: file
l2_file_path: scripts/aegis-self-wake.sh
l2_line_start: 1
l2_line_end: 20
curation: authored
l0_summary: Aegis 的自醒必须建在会话之外。会话内定时任务随会话死(死过三次),而 pi 根本没有这个工具 —— 2026-10-12 换成系统 crontab + tmux send-keys,和唤醒服务叫别的席位同一条通道。
---

## 发生了什么

Aegis 是唯一没有 waker 的席位:`seatloom-waker` 的日志明说
`NOTIFY for unmapped seat seat-aegis-001 — ignoring`。于是任何经过 Aegis 的环节
(门事件、跨席位裁决、迁移落库)都卡到张先生出现为止。**这发生了三次,都是同一个根因。**

前两次的自醒是 Claude Code 的会话内定时任务(`CronCreate 7d114e8e`,每 :11/:41)。
它随会话死掉 —— 而 AGENTS.md 里早就写着"换会话后第一件事就是重建它",
说明这个教训被记住过,只是**没有任何机制保证它真的被执行**:
换 harness 的那一刻,人的注意力在"pi 起来没有、MCP 通没有",没人去重建 cron。
2026-10-12 从 Claude Code 切到 pi 时,crontab 是空的 —— 自醒确实又死了。

## 为什么会改变未来动作

**自醒不能依赖"某个会话里的某个 agent 记得去建它"。** 必须建在会话之外:
系统 crontab(活在会话、harness、pi 重启之外)+ `tmux send-keys`
(和 `seatloom-waker` 叫醒其他席位是同一条既定通道,不是新发明的机制)。

2026-10-12 的实现:`scripts/aegis-self-wake.sh`,crontab 每 :11/:41 跑一次,
只往 Aegis-Supervisor 的面板发一条**短的**读文件指令,心跳协议本身在
`docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md`。
两处细节是实测出来的,不是设计出来的:

- cron 的 PATH 只有 `/usr/bin:/bin`,而 tmux 在 `/opt/homebrew/bin`。
  不显式定位 tmux,它只会每 30 分钟往日志里写一行失败,自醒看起来"在跑"。
- 长字面量会被 `send-keys` 截断(`scripts/seat-dispatch.sh` 第 4 行,席位收到过半条消息)。
  所以只发一条短指令,payload 走文件。

## 证据

- 切换前 `crontab -l` → `no crontab for jyxc-dz-0100609`
- `scripts/switch-aegis-to-pi.sh` 第 155 行起 pi,交接 prompt 里让新 Aegis 建 cron
- `scripts/aegis-self-wake.sh` 与 `/tmp/seatloom_aegis_waker.log` 的 `POKED` 行

相关:[[waker_idle_agent_looks_identical_to_a_working_one]](同一个"没 waker"病灶的另一层)
