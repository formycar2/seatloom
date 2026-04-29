# SeatLoom Acceptance Spec v1.0

| Field | Value |
|---|---|
| Document | Acceptance Spec v1.0 |
| Status | Superseded archived baseline |
| Updated | 2026-04-28 |
| Canonical entrypoint | `docs/PRODUCT_TRUTH.md` |
| Current replacement | `docs/acceptance-spec-v1.1.md` |

---

Reference-only after the v0.5 contract consolidation. Start current acceptance work from `docs/acceptance-spec-v1.1.md` through `docs/PRODUCT_TRUTH.md`.

## 1. 文档目的

定义 SeatLoom 的产品验收条款，确保 Lyra/Mira/Nimbus/Flux 对“完成”的理解一致。

---

## 2. 验收范围

### 2.1 本轮必须覆盖

- 多项目切换与项目状态记忆
- 应用布局：Sidebar / Main / Detail / Terminal / Status
- Inbox 行动队列
- Timeline 过滤与详情
- WorkItem/Handoff 核心流
- Attach/Wrap/Switch/Rebuild 关键连续性路径

### 2.2 本轮不作为阻塞项

- daemon 常驻模式
- file watcher 自动监听
- 完整策略编排引擎

---

## 3. 产品验收清单（PO 视角）

| 编号 | 条目 | 通过标准 | Owner |
|------|------|----------|-------|
| P-00 | 多项目切换可用 | 支持切换器、recent 列表、切换保护、项目级状态记忆 | Mira + Nimbus |
| P-01 | 首页可见核心工作面 | 打开应用后 1 屏看到 Inbox、Session、WorkItem 主信息 | Mira |
| P-02 | Inbox 是行动队列 | 仅出现需用户动作条目；Accept/Return/Resolve 后状态正确变更 | Mira + Nimbus |
| P-03 | Timeline 可回放 | 至少支持 Seat/WorkItem/事件类型/时间范围过滤，点击可看详情 | Mira + Nimbus |
| P-04 | Handoff 闭环完整 | create/send/accept/return/complete 全链路可走通 | Nimbus |
| P-05 | 中断恢复可用 | interrupted session 可通过 fallback 恢复继续工作 | Nimbus |
| P-06 | 早晨收束可用 | 启动后生成 digest，包含中断与漂移信息 | Nimbus |

---

## 4. UI/UED 验收清单（Mira 视角）

| 编号 | 条目 | 通过标准 |
|------|------|----------|
| U-00 | 多项目入口清晰 | 标题栏可见当前项目，切换器可发现 recent/pinned 项目 |
| U-01 | 信息层级清晰 | Sidebar 与 Main/Detail 分工稳定，无语义冲突 |
| U-02 | 状态表达一致 | dot/color/icon 对 status 和 priority 一致映射 |
| U-03 | 空状态可用 | 无 Seat/无 Session/无 Inbox/无 Timeline 结果时有明确引导 |
| U-04 | 错误状态可用 | 操作失败有可理解文案和重试入口 |
| U-05 | 主题一致 | Light/Dark 两套主题视觉一致、可读性达标 |
| U-06 | 键盘可达 | Tab/Shift+Tab/Enter/Esc 与文档一致且可用 |

---

## 5. 工程验收清单（Nimbus 视角）

| 编号 | 条目 | 通过标准 |
|------|------|----------|
| E-00 | 多项目数据隔离 | 项目 A/B 的 `.seatloom` 数据不混用，切换不污染对象状态 |
| E-01 | 类型对齐 | TS 类型字段与 Rust schema 对齐，无关键字段漂移 |
| E-02 | 事件链一致 | Ledger 事件能投影到 Inbox/Timeline 视图 |
| E-03 | fallback 明确 | attach/resume 失败时有固定 fallback 与用户提示 |
| E-04 | 存储可追溯 | 关键对象与 launchpack/checkpoint 文件可落地可回查 |
| E-05 | 运行可见 | 本地可通过 `pnpm dev` 启动并稳定访问原型 |

---

## 6. 测试与证据清单（Flux 视角）

| 编号 | 条目 | 证据要求 |
|------|------|----------|
| Q-00 | 多项目切换可复现 | 提供 A->B->A 切换视频/截图，证明状态恢复与运行会话不丢失 |
| Q-01 | 关键流程走通 | 每条流程提供步骤 + 结果截图/日志 |
| Q-02 | 状态转移正确 | 至少 1 条 WorkItem/Session/Handoff 状态转移证据 |
| Q-03 | 回放可追溯 | Timeline 事件与对象详情互相可跳转 |
| Q-04 | 启动可复现 | 本地启动命令、URL、首屏描述、异常排查步骤 |

---

## 7. Go/Hold 规则

### Go

- 所有 P 类条目通过
- U/E/Q 中无 Critical 问题，High 问题有明确修复计划

### Hold

- 任一 P 类条目失败
- 连续性主链（attach/wrap/rehydrate/rebuild）存在阻断

---

*本文件作为 Lyra 驱动执行与 Flux 验收的统一检查表。*
