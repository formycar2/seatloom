---
memory_id: pi_upgrade_changes_disk_not_running_processes
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: fact
confidence: inferred
status: active
tier: warm
source_kind: file
source_ref: scripts/restart-seats-for-new-tools.sh
l2_ref_kind: file
l2_file_path: scripts/restart-seats-for-new-tools.sh
l2_line_start: 102
l2_line_end: 130
curation: authored
l0_summary: pi update 只改磁盘:每个已经在跑的 pi 进程(含全部席位)继续执行旧 release 的代码。而 restart-seats-for-new-tools.sh 比的是 MCP 子进程和 seatloom-mcp-emit 的构建时间,不是 pi —— 它永远不会为 pi 升级而重启席位。要用 seat-launch.sh --all。
---

## 发生了什么

2026-10-16 11:24 跑 `pi update`,输出 `Updated pi from 1.1.0 to 1.3.0`,
`~/.pi/agent/install/current-version` 从 `1.1.0` 变成 `1.3.0`,新 release 目录建好。

但**没有任何一个正在跑的进程换上新版**:
- 6 个席位的 pi 进程全部起于 10-10(晚于它们的工具子进程,早于本次升级)
- Aegis 自己的 pi 进程起于 11:24:00,升级命令是在会话开始之后才跑的

原因在启动器 `~/.pi/agent/bin/pi` 里:它**在 exec 那一刻**读一次 `current-version`,
然后 `exec releases/<version>/node_modules/.bin/pi`。进程起来之后代码就在内存里了,
磁盘上换成 1.3.0 对它没有任何影响。这就是 AGENTS.md 第 8 条那条教训的 pi 版本变体。

## 为什么会改变未来动作

- **`pi update` 不等于舰队升级。** 升级完成后必须显式重启席位,否则会得到一个
  "二进制已是新版、全队还在跑旧版"的中间态,而且**面板上看不出来**。
- **`scripts/restart-seats-for-new-tools.sh` 干不了这件事。** 它的判据从头到尾是
  `seatloom-mcp-emit` 的 mtime(第 22 行 `BIN=`、第 102 行 `binepoch=`、第 126 行比较),
  比的是**席位的 MCP 工具子进程**和**那个 Rust 程序**,跟 pi 半点关系没有。
  跑它不会重启任何一个席位,而它的输出是"✅ 全部换上新程序" —— 一个成功的假象。
- 正确的工具是 `scripts/seat-launch.sh --all`(第 28 行 `PI=$HOME/.local/bin/pi`,
  解到 managed 启动器 → 读到新的 `current-version`)。它自带 mid-turn 守卫:
  正在干活的席位会被打上 `mid-turn, refusing`,不会被杀掉。
- **先确认重启是安全的**:6 个 worktree 的已跟踪文件当时全部干净(未跟踪的 `.scratch/`
  草稿不影响,杀掉进程不会让盘上的文件消失)。

## 证据

- `pi update` 输出:`added 119 packages in 5s` / `Updated pi from 1.1.0 to 1.3.0`
- `~/.pi/agent/install/current-version` → `1.3.0`;`releases/` 下 `1.1.0`(10-08)和 `1.3.0`(10-16 11:24)并存
- 各席位 pi 进程起于 10-10(见 `scripts/fleet-process-census.sh` 同类方法),Aegis 自己的 pid 3870 起于 `Fri Oct 16 11:24:00 2026`
- 重启后实跑验证(用 1.3.0):`pi mcp list` → `seatloom: connected, 19 tools`

**已执行(2026-10-16 11:31,`bash scripts/seat-launch.sh --all`)**:6 个席位全部换上新版。
重启前的守卫全部通过:6 个 worktree 已跟踪文件干净、6/6 在线且全空闲。
重启后的核验:6 个新 pi 进程起于 11:31:51–11:32:22(全部晚于 11:24 的升级),
6 个 MCP 工具子进程全部存活,面板都起来了、没有一个停在信任弹窗上,
而且洗数据席位拿到的仍是 `stepplan-wash-anthropic`(没有被静默降级成普通额度)。

**还没证明的一件事**:单个席位真正调通一次工具。按仓库的规矩,
唯一的硬证据是派单从 ready 翻成 in_flight —— 那要等席位自己拉到活才有。
我能证明的是 1.3.0 配这套工具服务能连通(上面那条 `pi mcp list`),
以及每个席位的新进程都挂着自己的工具子进程;这两条加起来是必要不充分。
- 1.2.0 的破坏性改动逐条对过:主题 schema 校验(没有任何席位配主题,不适用)、
  `before_agent_start` 改成 `messageType` 联合、`sendUserMessage` 要求 `deliverAs`
  —— 唯一装着的扩展 `agentd-probe.js` 只挂 `tool_call`/`tool_result` 两个事件,
  这两个事件名在 1.3.0 的 `dist/` 里都还在(169 / 48 处引用,1.1.0 是 167 / 48)
