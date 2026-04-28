# SeatLoom PRD v0.4

| 项目 | 内容 |
|------|------|
| 产品 | SeatLoom |
| 文档 | PRD v0.4 |
| 状态 | Draft (Design-First Baseline) |
| 更新时间 | 2026-04-27 |
| 文档语言 | 中文 |
| 前版 | v0.3（保留，v0.4 以“产品合同化”为主） |

---

## 0. 本版目标

v0.4 的目标不是“验证可行性”，而是把 SeatLoom 定义成一个可直接开发和验收的产品合同，解决以下现实问题：

- 用户每天在多个 agent 工具间切换，丢上下文、丢决策链、丢交接闭环
- 团队对“哪些行为保证支持、哪些是 best-effort”没有统一答案
- UI/UX 上游语义不足，导致原型只能做框架，无法做到可验收细节

本版以“已知痛点闭环”为优先，不采用“先做 MVP 验证价值”的表述策略。

---

## 1. 产品北极星与成功定义

### 1.1 北极星

> SeatLoom 是一层本地优先的 AI 项目连续性系统。  
> 它让用户在不改变常用 agent 工具的前提下，持续获得“可恢复、可追溯、可交接、可执行”的项目工作面。

### 1.2 成功定义（交付视角）

首次完整交付必须让用户稳定完成以下 6 件事：

1. 在桌面应用中看到所有活跃 Session 与待处理事项（Inbox）
2. 将一个工具中的工作无缝转到另一个工具（Rehydrate）
3. 通过 Timeline 回看“谁在何时做了什么，基于什么证据”
4. 发起并完成结构化 Handoff（带预期结果与材料）
5. 在 Session 中断后可恢复，不需要从头解释背景
6. 启动后 1 分钟内获得“早晨收束”并回到工作主线

---

## 2. 用户与高频任务链

### 2.1 主用户

- **主用户**：重度 AI coding 工具使用者（单人或小团队核心开发）
- **次用户**：协作角色（PO/Architect/Designer/Verifier）在同项目中的并行协作

### 2.2 每日高频任务链（产品主线）

1. 打开 SeatLoom（看到 Morning Digest + Inbox）
2. 选择待办 WorkItem
3. 进入一个已有 Session（attach）或启动新 Session（wrap）
4. 遇到中断/切换时，使用 LaunchPack 继续
5. 产出 Artifact，发起或接收 Handoff
6. 收束结果，留下可回放证据

---

## 3. 产品边界（硬约束）

### 3.1 SeatLoom 做什么

- 统一观察面：Inbox / Timeline / Detail / Terminal
- 连续性能力：Attach / Capture / Rehydrate / Replay
- 轻量治理：Handoff 校验、状态 gate、失败升级到 Inbox

### 3.2 SeatLoom 不做什么

- 不替代 Codex/Claude/Cursor/Gemini 等工具
- 不替代 IDE
- 不做“多 agent 自主编排平台”叙事
- 不把“云端协同”作为首要依赖

---

## 4. 信息架构合同（UI 必须展示什么）

### 4.1 Sidebar 必显字段

- Inbox 未处理数量
- Seat 列表（name/status）
- Session 列表（id/status/runtime/所属 seat）
- WorkItem 列表（id/status/priority/title 摘要）

### 4.2 Main Panel 必显视图

- Inbox：仅展示“需要用户动作”的事项
- Timeline：事件列表 + 过滤器 + 搜索
- WorkItems：任务分组与优先级视图

### 4.3 Detail Pane 必显字段

- 对象统一头部：`type + id + status`
- 对象关键上下文
- 可执行动作按钮（Accept / Return / Run Pipeline / Switch Runtime 等）
- 关联对象跳转（Session ↔ WorkItem ↔ Handoff ↔ Artifact）

### 4.4 Terminal Panel 必显能力

- 多 Session 标签页
- 输入/输出实时同步
- session 状态标识（running/interrupted/completed）
- 折叠不丢会话状态

---

## 5. 三个关键对象的状态机（产品合同）

### 5.1 WorkItem

状态：`draft -> ready -> active -> in_review -> verified -> done`

异常/支路：`blocked`、`reopened`、`drifted`

关键 gate：

- `draft -> ready`：必须有 `title + acceptance_criteria`
- `ready -> active`：必须指定 owner
- `active -> done`：必须满足验收证据（至少 1 个相关 Artifact）

### 5.2 Session

状态：`launching -> running -> input_required -> completed`

异常：`failed`、`interrupted`、`suspended`

恢复顺序（固定）：

1. native resume（若可用）
2. checkpoint + launchpack rebuild
3. minimal launchpack（WorkItem + Artifact refs）

### 5.3 Handoff

状态：`drafted -> sent -> accepted -> completed`

异常：`returned`、`expired`

发送 gate（必须全部满足）：

- `from_ref`、`to_ref`、`workitem_id`
- `purpose`
- `expected_outcome`

---

## 6. Inbox 规则引擎（从“列表”到“行动队列”）

### 6.1 进入 Inbox 的条件

1. Handoff `sent` 且等待接收方动作
2. Session 状态为 `input_required`
3. WorkItem 状态变为 `blocked` 或 `drifted`
4. Pipeline 达到重试上限并失败

### 6.2 优先级规则

- **Critical (`‼`)**：handoff.pending / review.requested / P0 pipeline fail
- **Normal (`!`)**：input.required / blocked
- **Low (`·`)**：drift.detected / info 类提示

### 6.3 离开 Inbox 的条件

- 用户完成明确动作（accept/return/resolve/dismiss）
- 对应对象状态变更到非待办态

---

## 7. Timeline 语义合同（不是原始事件倾倒）

Timeline 每一行必须包含 5 列：

- 时间
- actor
- 事件类型（可读标签）
- 对象引用
- 摘要文案（人话）

禁止只展示内部事件名（如 `handoff.sent`）而无可读摘要。

---

## 8. Attach / Wrap / Switch Runtime 合同

### 8.1 保证能力（Guaranteed）

- `wrap` 启动新 session 并捕获输出
- `switch runtime` 生成 LaunchPack 并落地 `.md` 文件
- session 中断后的 rebuild 恢复链路

### 8.2 最佳努力能力（Best-Effort）

- attach 已运行的外部工具 session（依赖工具能力和环境权限）
- native session id 级别恢复

### 8.3 失败时产品行为

- 明确展示“失败原因 + fallback 操作”
- 自动给出下一步（例如“使用 LaunchPack 启动新 session”）
- 不允许静默失败

---

## 9. LaunchPack 合同

### 9.1 必含项

- 当前 WorkItem（title/goal/AC/status）
- 当前 branch 与最近提交摘要
- 最近 Handoff 的目的与预期结果
- 最近失败信号（如 test failure）

### 9.2 禁止行为

- 禁止让用户手动复制粘贴大段上下文作为主流程
- 禁止仅在内存中存在 LaunchPack（必须先落地文件）

### 9.3 fallback

- L1: 自动注入
- L2: 文件路径 + 短指令
- 不再保留“整包复制到剪贴板”作为标准路径

---

## 10. 可验收标准（产品层）

### 10.1 观察面验收

- 用户可在 10 秒内定位“当前最重要待办”
- Timeline 支持按 Seat/WorkItem/事件类型/时间范围过滤
- 任一列表条目可一跳进入详情

### 10.2 连续性验收

- 从工具 A 切换到工具 B 后，首轮响应不要求用户重新解释核心背景
- Session 中断后，5 分钟内可恢复到可继续状态

### 10.3 交接验收

- Handoff 的发送、接收、退回、完成状态完整可追踪
- 每次交接都可追溯到至少一个 WorkItem 与 Artifact

---

## 11. 版本策略（设计先行，不走 MVP 验证叙事）

### 11.1 当前策略

- 目标不是“证明产品是否有价值”，而是“完成已知痛点闭环”
- 所有开发以产品合同为准，原型不是概念演示，而是交付前置物

### 11.2 里程碑

1. **Baseline Freeze**：PRD/Scenarios/UX/Architecture 四文档无 P0 冲突
2. **Phase 1 UX Baseline**：关键界面字段、状态、交互可对照验收
3. **Implementation Baseline**：Nimbus 按合同进入实现

---

## 12. 与 v0.3 的关键差异

- 从“方向性 PRD”升级为“合同化 PRD”
- 明确保证能力 vs 最佳努力能力
- 加入对象状态机与 Inbox 规则引擎
- 明确 Timeline 必须输出人话摘要
- 明确设计先行的里程碑和验收口径

---

## 13. 多项目切换能力（P0）

多项目切换是首批交付的核心能力，不作为后置项。

### 13.1 目标

- 用户可以在一个桌面应用内快速切换不同项目，不丢失每个项目的上下文
- 每个项目的数据边界保持独立（每个项目各自 `.seatloom/`）
- 切换过程可控，不因误切换导致运行中的工作丢失

### 13.2 必须提供的能力

1. **Project Switcher**：标题栏可切换项目；支持快捷打开（`Cmd/Ctrl+K`）
2. **Recent Projects**：显示最近项目列表，支持 pin/unpin
3. **项目级状态记忆**：记住每个项目上次的 tab/filter/选中对象
4. **切换保护**：切换前检查未保存输入或 running session，给出确认弹窗
5. **跨项目首页**：提供 “All Projects” 入口，显示各项目关键待办摘要

### 13.3 数据与边界规则

- 项目 A 和项目 B 的 Seat/Session/WorkItem/Handoff 不混合
- Timeline/InBox 默认仅显示当前项目数据
- “All Projects” 只做摘要聚合，不支持跨项目对象合并编辑

### 13.4 切换保护规则

- 若当前项目存在 `running` session，切换时提示：
  - `Keep running in background and switch`
  - `Cancel`
- 若存在未提交的表单输入，切换时提示：
  - `Discard and switch`
  - `Stay`

### 13.5 验收标准（多项目）

- 5 秒内完成项目切换并显示目标项目工作面
- 切回原项目后，恢复上次 UI 状态（tab/filter/selection）
- 运行中的 session 不因项目切换被意外终止
- `All Projects` 视图可显示每个项目的 Inbox 数量与最近事件时间

---

*本版本为设计先行基线草案。经 Mr. Zhang 审核后，成为 Lyra/Mira/Nimbus/Flux 的统一执行合同。*
