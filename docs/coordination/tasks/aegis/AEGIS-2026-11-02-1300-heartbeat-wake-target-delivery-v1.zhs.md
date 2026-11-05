# Aegis 13:00 心跳交付：自醒送达了不存在的协议路径

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-1300-heartbeat-wake-target-delivery |
| status | delivered |
| author | aegis |
| date | 2026-11-02 |
| version | v1 |
| tags | heartbeat, runtime, wake-target |

## 变了什么

本轮初始代码 `145cede114b4f1086458d5e2a585543c18188e61`，分支 `track/infra-foundation`。
心跳指定的 `AEGIS-HEARTBEAT.md` 读取失败（ENOENT）；确认当前实际文件为
`docs/coordination/tasks/aegis/AEGIS-HEARTBEAT-v1.zhs.md`，已全文读取并执行检查。

根因是协议改名后，自醒消息和安装器提示没有更新。13:00 cron 有 POKED，
旧安装器 `--check` 也返回 0，但它只验脚本副本/cron，没有验消息中引用的文件。
因此“消息已入队”被误当成“自醒所需输入有效”。

本轮修改：

- `scripts/aegis-self-wake.sh`：消息和注释改用现存路径，保留 shell 注释前缀。
- `scripts/check-aegis-wake-target.py`：只读解析字面量 MSG，要求唯一消息、注释前缀、绝对路径、可读取非空文件；不执行被检查的脚本。
- `scripts/install-aegis-self-wake.sh`：安装前检查原本的目标；`--check` 同时检查原本和实际副本，失败返回非零。检查在 agent/安装器上下文执行，不让 cron 读取 Documents（保留 TCC 约束）。
- `scripts/test-aegis-wake-target.py`：离线测试有效目标、不存在目标、空文件、目录，以及仓库消息必须指向现存协议。

已实际重跑既有安装器，不动 boot daemon、不改库或权限配置。安装副本和 cron 读回一致，
裸 cron 环境实跑在 13:01:50 长出 POKED；本会话收到新版路径消息，是额外的送达证据。
下一次整点自然唤醒尚未到，不能提前声称已验证。

## 全局检查结果（13:00，数据库自己的时间）

- public 表数 35；`check-ledger-exists.sh` 返回 0（1 条 canonical event）。
- 使用 psql stdin、ON_ERROR_STOP 和常量证明行：数据库时间 `2026-11-02 05:00:56+00`，seats=9、dispatches=0、acceptances=0、operator_decisions=0。
- ready/blocked/in_flight 队列、48 小时内 done 无卡、pass verdict 无卡、pending 人工决策分别查询成功，都是 0 行。没有可领的 Aegis 派单或可核的门依赖。
- 6 个核心席位 pane 子进程由 ps 的父子关系确认均为 saih；Aegis pane 同样为 saih。进程在线不代表有进展，更不证明 MCP ready→in_flight 已验证。
- waker pid 94069；fleet-status 报在跑；日志有 Lyra 常规职责唤醒。
- `check-salvage-fresh.sh` 返回 0：2 份有效补丁对得上 Lyra/Nimbus；各 3/2 个未提交文件有覆盖。
- `check-cards-clickable.sh` 返回 0：应用门能看见正确仓库，pending 卡 0。
- verdict refs/defects/card-verifications 返回 0，但检查对象为空；不能由此推导实现已验绿或工作已完成。
- 分支备份脚本返回 0，生成 `backups/git-20261102-130202.bundle`，bundle verify 通过。脚本还报告孤立对象并加本地标签；本轮未独立核实其“独有内容”分类，也未做恢复比对，故只声称包结构验证通过。

## 验证和限制

- `python3 scripts/test-aegis-wake-target.py`：2 个测试方法通过，覆盖 4 个目标样本；坏目标 CLI 返回 1，好目标返回 0。
- `bash -n scripts/aegis-self-wake.sh scripts/install-aegis-self-wake.sh` 返回 0。
- 安装前 `--check`：原本目标有效、旧副本目标 ENOENT，返回 1；证明新检查能对实际坏副本报错。
- 安装后 `--check`：原本和副本目标均有效、脚本内容一致、cron 一致，返回 0。
- `git diff --check` 返回 0；提交只包含上述自有文件和本交付说明。

已有的用户 `.gitignore` 修改和 12:00 派单原文保持不动，不代为暂存或提交。
12:00 架构报告已本地提交 `145cede`；本轮未推远端、未开卡或代替其他席位验收。

本会话没有可用 SeatLoom MCP，也无本派单生命周期事件。没有调用 remember、ArtifactCreated、complete_dispatch；
未改用文件或 SQL 冒充前向记忆，不把本说明当成账本完结。

## 你会看到什么 / 需要你做什么

下一轮自醒的指令将引用真实存在的协议；安装器不再把“引用已删除文件”的副本判为健康。
这项修复已经安装，本轮不需要张先生重复跑命令。

全队仍没有新派单/验收记录，注册修好不是交付链已恢复的证据；架构建议与旧交付重新验收仍未在账本形成。
没有新的人类决定需要本轮追加询问，既有问题不因数据库 pending=0 而被视为已解决。
