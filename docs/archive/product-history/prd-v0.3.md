# SeatLoom PRD v0.3

| 项目 | 内容 |
|------|------|
| 产品 | SeatLoom |
| 文档 | PRD v0.3 |
| 状态 | Draft |
| 更新时间 | 2026-04-24 |
| 文档语言 | 中文 |
| 前版 | v0.2（已归档。v0.3 是一次收敛性重写，而非增量更新） |

---

Reference-only during v0.5 consolidation. Start from `docs/PRODUCT_TRUTH.md` for current product work.

## 0. 定位声明

> SeatLoom 不是另一个 agent，也不是另一个 IDE；它是一层本地优先的 AI 项目连续性系统，用来把异构 coding agent 工具中的 session、证据、交接与状态，收束成一个可恢复、可审计、可持续推进的项目工作面。

这一句话锁定四个约束：
- **"一层"而非"一个平台"**：不做替代，只做收敛。
- **"本地优先"**：项目数据随项目目录落地，不依赖云端即可工作。
- **"连续性系统"**：品类定义。解决的根本问题是跨工具、跨 session、跨时间的项目连续性。
- **"收束成工作面"**：用户不再在多个终端窗口间搬运上下文，所有关键信息收束到 inbox + timeline + 结构化 handoff。

在继续阅读之前，请确认你理解并接受以下前提：SeatLoom 不生成代码，不替代 IDE，不充当 runtime orchestrator，也不以"管理大量 agent"为第一阶段目标。

---

## 1. 为什么 SeatLoom 独立成立

### 1.1 存在前提

SeatLoom 的价值建立在以下四个假设之上。如果它们被证伪，产品需要重新审视。

**假设 1：AI 编程工具将长期异构共存。** Codex、Claude Code、Gemini CLI、Cursor CLI 各有优势场景，没有单一工具能在所有任务上胜出。用户持续地在不同工具间切换是常态，不是过渡态。

**假设 2：项目级连续性无法由任一工具单独提供。** 每个工具的原生 session 记忆是私有的。Claude Code 不知道 Codex 做了什么。这种隔阂是架构性的，不会因单个工具增强 memory 而消失。

**假设 3：重度用户确实有上下文搬运的痛点。** 在多个终端窗口之间复制粘贴代码、错误信息、需求描述，是一种可观察、可统计的真实消耗。

**假设 4：用户愿意为本地数据主权和控制力付费。** 本地优先意味着对项目数据的完全控制，对独立开发者、小型技术团队、隐私敏感项目具有真实价值。

### 1.2 SeatLoom 不做什么

| 不是这个 | 原因 |
|----------|------|
| 不是另一个 coding agent | 写代码是 Codex、Claude Code、Cursor CLI 的事 |
| 不是另一个 IDE | 不与 VSCode、JetBrains 竞争工作台 |
| 不是 runtime orchestrator | 不接管 agent 进程调度、资源分配 |
| 不是 swarm 框架 | 不以"100+ agent 自主进化"为卖点 |
| 不是云端 SaaS 平台 | 核心功能离线可用，云端能力是增强项 |
| 不是 Agent 中台 | 不定定义"跨组织 agent 统一管理"的野心 |

### 1.3 与底层工具的关系：为什么上游增强后 SeatLoom 仍然成立

SeatLoom 不赌上游工具永远不进化。相反，假设工具会越来越强。但以下价值独立于任一工具的进化：

| 上游能力增强 | SeatLoom 独立保留的价值 |
|-------------|----------------------|
| Claude Code 支持 project memory | 跨工具 continuity（Claude 记不住 Cursor 做了什么） |
| Codex 支持原生 session resume | 项目级证据链独立于工具（两周后追查不依赖"那个 session 还在不在"） |
| 所有工具都支持更好的上下文管理 | 治理规则需要工具无关（代码评审标准不应绑死在某个工具的实现上） |
| 某个工具成为绝对主导 | 历史数据跨工具可迁移 |

**最差情况下的退化路径**：即使某个工具一统天下，SeatLoom 仍可退化为跨工具的 Ledger + Timeline + 工具无关的 Handoff/Governance 薄层。

---

## 2. 五个铲子：核心能力主轴

SeatLoom 靠五个具体工程能力创造价值，而非靠"比 agent 更聪明"。

| 铲子 | 名称 | 回答问题 | MVP 优先级 |
|------|------|---------|-----------|
| 1 | **Attach** | "我有 4 个在跑的终端窗口，能不重启就把它们接进 SeatLoom 吗？" | P0 必须 |
| 2 | **Capture** | "刚才那个 agent 干了什么？输出在哪？" | P0 必须 |
| 3 | **Rehydrate** | "我要让 Codex 接手 Claude 的工作，但不想从头解释背景。" | P0 必须 |
| 4 | **Replay** | "两周前那次 bug 是谁引入的？当时为什么做这个决定？" | P0 必须 |
| 5 | **Govern** | "我怎么确保每个 Handoff 都有验收标准？" | P1 轻量 |

### 2.1 铲子优先级说明

Govern 后置于前四个铲子，原因：治理的有效性依赖于前四个能力的成熟度。没有 capture 就没有可治理的素材；没有 rehydrate 治理规则无法跨 session 执行；没有 replay 治理效果无法验证。MVP 阶段仅做 Handoff 必填校验 + 状态 gate + 最小权限声明，不做完整 PolicyPack 体系。

### 2.2 五个铲子与渐进采用模型的映射

| 采用档位 | 用户行为 | 核心依赖铲子 | 用户需要理解的概念 |
|---------|---------|-------------|-------------------|
| **Observe** | 附着 Session，看 Timeline/Inbox | attach + capture + replay | 无（自动生成） |
| **Assist** | 系统自动生成交接草稿 | attach + capture + rehydrate | Handoff 草稿（浅层） |
| **Structure** | 用户显式创建 WorkItem/Handoff | 全部五个 | WorkItem、Handoff、Artifact |
| **Automate** | 开启 daemon/Pipeline/Gate | 全部五个，govern 加重 | Pipeline、Gate、PolicyPack |

---

## 3. 渐进采用模型

SeatLoom 的用户入口不能是对象模型，而应是可感知的价值。

| 模式 | 入口命令 / 行为 | 用户看到什么 | 对用户隐藏什么 |
|------|----------------|-------------|---------------|
| **Mode 0: Attach** | `seatloom wrap <agent-cmd>` 或配置 shell 别名，如 `alias claude="seatloom wrap claude"` | Timeline + Inbox（自动生成）。用户仍用原生命令与 agent 交互，完全无感。 | Seat、Session、WorkItem 全部隐式创建 |
| **Mode 1: Run** | `seatloom run "fix bug"` | 单个命令启动，后台自动留痕 | Session、WorkItem、Ledger 均隐式 |
| **Mode 2: Collaborate** | 显式 `seatloom handoff create` | Handoff 草稿、Review Queue、WorkItem 面板 | Pipeline 定义细节 |
| **Mode 3: Govern** | 配置 Pipeline、PolicyPack | 完整的治理视图和自动化 | 无（全部暴露） |

**核心原则**：顶层概念可以埋在架构里，但不能一开始就怼在用户脸上。Mode 0 的用户第一眼必须看到 timeline 和 inbox，而非 Seat/Session/Handoff 的定义。

**实施要点**：Mode 0 的真实入口必须是**别名拦截**而非“先启动 agent 再 attach”。用户配置好别名后，所有 agent 命令被 SeatLoom 包裹启动，原始交互体验不变，Ledger 在后台自动生长。这保证用户体验零割裂。

---

## 4. Adapter 策略与降级

### 4.1 三档接入策略

| 接入档位 | 方式 | 适用工具 | 优势 | 局限 | MVP 角色 |
|---------|------|---------|------|------|---------|
| `wrapper_capture` | 包一层启动命令，捕获 IO | 大多数 CLI | 最快落地、统一性高 | 对工具内部语义感知有限 | **基本盘** |
| `native_attach` | 读取原生 session id、日志目录、resume 能力 | Codex、Claude Code 等 | 保留工具原生连续性更好 | 依赖工具暴露能力 | **增强项** |
| `managed_launcher` | SeatLoom 直接负责启动、隔离、停止 | Shell/CI/自有适配器 | 生命周期控制最强 | 对接入要求最高 | **预留** |

### 4.2 MVP 适配器承诺

| 承诺项 | 内容 |
|--------|------|
| **深适配工具** | Codex CLI + Claude Code（2 个，各做 wrapper_capture + native_attach） |
| **浅适配工具** | Cursor CLI / Gemini CLI（至少 1 个，仅 wrapper_capture） |
| **不承诺** | 原生私有 session 双向迁移。不把"篡改底层工具内部状态"作为 MVP 目标。 |
| **MVP 成功标准** | 两个深适配工具的 attach + capture 成功率 ≥ 95% |

### 4.3 Adapter 能力声明矩阵

每个 adapter 必须声明：

| 能力 | 说明 | 是否必达 |
|------|------|---------|
| `can_launch` | 能否由 SeatLoom 启动新 Session | 可选 |
| `can_attach` | 能否附着到现有原生 Session | P0 |
| `can_resume` | 能否使用原生 session id 恢复 | P1 |
| `can_capture_transcript` | 能否拿到结构化对话 transcript | P0 |
| `can_capture_tool_calls` | 能否拿到工具调用摘要 | P1 |
| `can_inject_input` | 能否程序化发送输入 | P1 |
| `workspace_modes` | 支持 local / worktree / docker 中哪些模式 | local 必达 |

### 4.4 Fallback 策略

当原生能力不可用时，不崩溃，而是降级：
- `can_resume` 不可用 → 退回到 `Checkpoint + ContextPack` 重建
- `can_capture_transcript` 不可用 → 退回到 stdout/stderr 原始捕获
- `can_inject_input` 不可用 → 将 LaunchPack 写入文件 `.seatloom/sessions/<id>/launch_pack.md`，并生成一条极短的复制指令（如 `claude --load-context .seatloom/sessions/<id>/launch_pack.md` 或直接提示用户在新 session 中输入“请读取 .seatloom/.../launch_pack.md 并基于此开始工作”）。**禁止让用户手动复制粘贴几千 Token 的上下文**。

---

## 5. ContextPack Compiler 子系统

ContextPack Compiler 是 SeatLoom 的"智能层"——它决定从历史中提取什么、如何压缩、如何注入新 session。这是跨工具连续性的核心技术面。

### 5.1 子系统组成

| 组件 | 职责 | 实现策略 |
|------|------|---------|
| **Selector** | 挑哪些内容值得进包 | 规则优先：当前 WorkItem、AC、最近 Handoff、当前 branch、最近 failed test **必放** |
| **Budgeter** | 按 token 预算硬裁剪 | 默认 8K token 上限（可配置）。先裁剪选放项，再压缩必放项 |
| **Compressor** | 摘要、分层压缩、指针化引用 | 规则优先：大文件不内联，只放摘要 + 路径引用。LLM 仅用于超预算时的有损压缩 |
| **Assembler** | 组出 LaunchPack / HandoffPack | 按固定模板组装，确保结构一致性 |
| **Verifier** | 检查包内最低上下文完整性 | 验证必放项是否齐全、token 是否超限、引用是否有效 |

### 5.2 LaunchPack 结构

```yaml
launch_pack:
  seat: nimbus
  target_runtime: codex
  created_at: 2026-04-24T10:30:00Z
  budget_used: 6200  # tokens

  # 必放项
  mandatory:
    workitem:
      id: WI-12
      title: "实现 OAuth 登录"
      goal: "支持 Google/GitHub OAuth，使用 NextAuth.js"
      acceptance_criteria:
        - "用户可使用 Google 账号登录"
        - "用户可使用 GitHub 账号登录"
        - "登录后跳转到 /dashboard"
      status: active
    current_branch: "feature/oauth-login"
    last_handoff:
      from: lyra
      purpose: "需求确认完成，请开始实现"
      expected_outcome: "可运行的 OAuth 登录功能 + 测试"
    recent_failures:
      - test: "auth/callback.test.ts"
        error: "TypeError: Cannot read properties of undefined"
        timestamp: 2026-04-24T10:15:00Z

  # 选放项（按相关性排列，受 budget 裁剪）
  optional:
    recent_artifacts:
      - id: AR-45
        kind: design_note
        title: "OAuth 流程设计"
        summary: "使用 NextAuth.js v5，支持 Google 和 GitHub provider..."
    related_sessions:
      - seat: lyra
        runtime: claude_code
        summary: "需求讨论 session，产出 Acceptance Criteria"
    policies:
      - coding_standard: "使用 TypeScript 严格模式"
      - testing_standard: "覆盖所有 OAuth callback 路径"

  # 引用（不内联）
  references:
    - path: "docs/auth-design.md"
      summary: "完整 OAuth 架构设计文档"
    - path: "prisma/schema.prisma"
      summary: "数据库 schema，含 User/Account 模型"

### 5.3 设计原则

1. **规则优先 + LLM 辅助**：不把 ContextPack 的正确性建立在另一个 LLM 摘要节点上。硬指针（branch、AC、failed test、Handoff）由规则保证必放。
2. **大文件不内联**：只放摘要 + 路径引用，由 agent 工具自行按需读取。
3. **超预算时先裁剪选放项**：必放项被压缩是最后手段。
4. **可追溯**：每个 LaunchPack 记录用了哪些源 Artifact/Session，可在 Ledger 中回溯。

---

## 6. Source of Truth 与 Reconciliation

### 6.1 分层事实模型

SeatLoom 不假装自己是唯一真相源，而采用分层事实模型：

| 层级 | 名称 | 角色 | 权威性 |
|------|------|------|--------|
| L0 | **Git / Filesystem** | 代码事实源 | 最高。代码的真实状态以此为准。 |
| L1 | **Raw Evidence** | 过程事实源 | 原始捕获的 stdout、transcript、diff。不可变。 |
| L2 | **Ledger** | 标准化事件事实源 | append-only 事件流。尽量不可变，schema 可迁移。 |
| L3 | **Object Projection** | 产品语义层 | 从 Ledger 投影出的 Seat、Session、WorkItem、Handoff。可重建。 |
| L4 | **Views** | 派生视图 | Inbox、Timeline、Review Queue。纯派生，可删除重建。 |

### 6.2 Reconciliation 策略

当 L3（对象状态）与 L0（Git/FS 真实状态）冲突时：

| 冲突类型 | 处理方式 |
|---------|---------|
| WorkItem 标记为 done，但对应 branch 未 merge | 标记 WorkItem 为 `drifted`，推送至 Inbox |
| Session 记录"文件已修改"，但 Git 无变更 | 补录 `reconcile:file_no_change` 事件，标记该 Session Run 为 `stale` |
| Pipeline 基于过时状态触发 | 任何自动化动作执行前，先执行 reconciliation check |

### 6.3 Reconciliation 触发时机

- **启动时**：扫描 Git/branch/worktree，与 Ledger 最新状态对比
- **文件监听**：外部变更（用户手动修改、其他工具修改）被检测到时
- **Pipeline 执行前**：每个 Stage 启动前验证输入假设仍成立
- **手动触发**：`seatloom reconcile` 命令

### 6.4 设计约束

- Git/FS 是绝对第一真理。代码仓库的真实文件状态比任何对象状态都更底层。
- 如果冲突无法自动解决，标记为 `needs_human` 并进入 Inbox，而不是静默覆盖任一方的状态。
- 文件监听（watcher/hook）是加速器，不是 Reconciliation 的唯一生命线。必须支持 wake-up reconciliation 和 manual sync。

---

## 7. 核心对象模型

以下对象模型覆盖 SeatLoom 需要理解的核心概念。注意：这些概念是内部数据语义，不是用户的首次接触面。

### 7.1 顶层对象

| 对象 | 含义 | 为什么是一等对象 |
|------|------|-----------------|
| `Seat` | 稳定协作身份 | 保住角色连续性，不被工具切换打断 |
| `Session` | 某个工具中的连续工作上下文 | 保住 Codex、Claude Code 等工具内的脑状态 |
| `WorkItem` | 有目标、有边界、有完成标准的工作单元 | 把项目推进主线固定下来 |
| `Artifact` | 值得长期保存和复用的材料 | 避免顶层概念无限膨胀 |
| `Handoff` | 一次正式责任交接 | 替代模糊回复链和聊天片段 |
| `Pipeline` | 可复用的多阶段自动化定义 | 编排跨阶段、跨 Seat 的流程逻辑 |

### 7.2 Supporting Entities

| 实体 | 作用 |
|------|------|
| `Run` | 记录某次 Session 执行片段或 Pipeline 执行实例 |
| `Checkpoint` | 从 Session 中提取的可恢复快照 |
| `ContextPack` | 给新 Session 注入的显式上下文包 |
| `SeatProfile` | Seat 的 runtime 配置、权限、默认工具 |
| `Ledger` | Append-only 事件账本，所有事实的标准化记录 |
| `Receipt` | Handoff 的接收、签收、退回、完成记录 |

### 7.3 对象关系

```mermaid
flowchart LR
    Seat --> Session
    Seat --> WorkItem
    Seat --> Handoff
    Session --> Artifact
    Session --> Checkpoint
    Session --> Run
    WorkItem --> Artifact
    WorkItem --> Handoff
    Handoff --> Receipt
    Pipeline --> Run
    Checkpoint --> ContextPack
    Artifact --> ContextPack

## 8. Ledger 与事件模型

### 8.1 为什么需要 Ledger

- 原始输出不一定稳定，但事件序列是追踪和回放的基础。
- 结构化对象可能随规则迭代重建，但 Ledger 保持为事实层。
- 把人工触发、自动触发、外部触发和 Seat 动作统一到一条时间线上。

### 8.2 Canonical Event 最小字段

| 字段 | 说明 |
|------|------|
| `event_id` | 稳定事件主键 |
| `event_type` | 如 `session.started`、`handoff.sent` |
| `occurred_at` | 事件发生时间 |
| `actor_ref` | 触发者（human / seat / automation / external） |
| `object_refs` | 关联对象列表 |
| `evidence_refs` | 原始日志、transcript、diff 的引用 |
| `payload_ref` | 结构化详情位置 |

### 8.3 证据链

```mermaid
flowchart LR
    A[Raw Evidence] --> B[Canonical Event]
    B --> C[Object Projection]
    C --> D[Views]
    C --> E[Context Engine]
    E --> F[LaunchPack / ContextPack]

## 9. 本地存储结构

项目根目录下 `.seatloom/`：

```text
.seatloom/
  config/
    project.yaml
    seats.yaml
  ledger/
    events.jsonl
  seats/
    lyra/profile.yaml
    nimbus/profile.yaml
    flux/profile.yaml
  sessions/
    ses_01/
      meta.yaml
      raw/stdout.log stderr.log transcript.jsonl
      checkpoints/cp_01.yaml
      context/launch_pack.md
      outputs/diff.patch
  workitems/wi_001.yaml
  artifacts/ar_001/meta.yaml payload.md
  handoffs/ho_001.yaml
  pipelines/definitions/ runs/
  views/  # 可重建


## 10. Pipeline（最小集）

### 10.1 MVP 仅做两个 Pipeline（显式触发，无后台 Daemon）

MVP 阶段**不实现常驻后台的 daemon/事件监听**。所有 Pipeline 通过以下方式触发：

- **同步钩子**：在 `handoff create`、`artifact promote` 等命令内部，完成后自动调用关联 Pipeline。
- **显式命令**：`seatloom pipeline run <pipeline-id>`。

| Pipeline | 触发方式 | 做什么 |
|----------|---------|--------|
| `requirement_handoff` | `seatloom handoff create` 成功后自动触发 | 生成 Handoff 草稿，从 Lyra 到 Nimbus |
| `verification_loop` | `seatloom pipeline run verification_loop` 或 `artifact promote` 时附带触发 | 触发 Flux 的测试 session，产出 test_report |

### 10.2 Pipeline 最小定义（清晰标注触发为显式调用）

```yaml
id: verification_loop
trigger_mode: explicit  # 而非事件驱动。可由 CLI 命令调用，或作为其他动作的同步后置步骤
stages:
  - id: collect_context
    action: build_context_pack
  - id: run_verifier
    seat: flux
    runtime: codex
    action: launch_session
  - id: evaluate
    action: summarize_test_results
gates:
  - id: acceptance_exists
    rule: workitem.acceptance_criteria_present == true
outputs:
  - kind: test_report
  - kind: evaluation
retry:
  max_attempts: 2
  backoff: exponential
idempotency:
  key: "{{workitem_id}}:verification_loop:{{artifact_id}}"

### 10.3 失败恢复

- **同步运行保证**：每个 Pipeline 作为单次进程执行，失败即退出，不残留后台任务。
- **阶段快照**：每个 Stage 完成后记录快照，重试可从安全点继续。
- **人工接管**：达到重试上限后，将现状写回 WorkItem 并标记为` input_required` ，推送至 Inbox。
- **Project Lock 说明**：由于 MVP 无 daemon，不存在多进程并发调度。Project Lock 仅作为并发执行` pipeline run` 时的文件级互斥锁，防止用户手动重复启动。

---

## 11. MVP 范围与止损

### 11.1 MVP 必须实现

- 定义命名 `Seat`（Lyra、Nimbus、Flux）
- 支持 Codex CLI + Claude Code 的 wrapper_capture + native_attach
- 支持至少 1 个辅助工具的 wrapper_capture
- 实现 append-only Ledger + Timeline + Inbox
- 实现 Checkpoint + ContextPack 跨工具重建（Rehydrate）
- 支持 `requirement_handoff` 和 `verification_loop` 两个最小 Pipeline
- 支持最小 Instruction Stack（3 条 gate）
- 支持 local workspace（worktree 和 docker 为 P1）

### 11.2 MVP 明确不做

- 完整 PolicyPack 治理体系
- daemon / overnight 自治模式
- Memory / Skill Growth 闭环
- ORCH / CAO 等第三方编排框架适配
- 多用户协作
- 成本统计与预算控制
- Docker workspace 隔离
- 完整评估体系（仅保留基本 check）

### 11.3 MVP 验证指标

| 指标 | 目标 | 优先级 |
|------|------|--------|
| 上下文搬运减少 | ≥ 50% | P0 |
| Mode 0 上手时间 | ≤ 10 分钟 | P0 |
| 跨工具切换恢复 | ≤ 5 分钟 | P0 |
| Attach + Capture 成功率 | ≥ 95% | P0 |
| Session 中断恢复成功率 | ≥ 80% | P1 |
| 两周决策回溯覆盖率 | ≥ 80% | P1 |

### 11.4 止损条件

以下任一触发，MVP 视为失败：

| 编号 | 条件 |
|------|------|
| STOP-1 | 两个主力工具的 attach + capture 成功率 < 90% |
| STOP-2 | 上下文搬运减少 < 30% |
| STOP-3 | 跨工具 rehydrate 后的效率低于手动搬运 |
| STOP-4 | Mode 0 用户在 30 分钟内放弃 |

---

## 12. 四个生命体征（终极验收标准）

1. **跨工具切换不需重写上下文**：Nimbus 从 Claude Code 切到 Codex，系统自动注入足够上下文继续干活。
2. **Session 中断后项目不断**：从 checkpoint + context pack + ledger 恢复推进，不丢关键产物。
3. **早晨一眼看全局**：不看 4 个窗口，只看 1 个 inbox + 1 条 timeline，就知道卡住、待批、已完成。
4. **两周后可回溯决策链**：能回答“是谁、在什么上下文下、依据什么证据、经过什么 gate 做了这个决定”。

如果这四件事做不到，SeatLoom 就只是一个结构良好的管理外壳。

---

## 13. 风险与开放问题

### 13.1 四大 P0 风险

| 风险 | 严重度 | 应对 |
|------|--------|------|
| 产品摩擦过高，早期价值不显著 | P0 致命 | 渐进采用模型 + Mode 0 极简体验 |
| 适配器不稳，attach/rehydrate 不可靠 | P0 致命 | wrapper_capture 为基本盘，不赌深度 native attach |
| ContextPack 失控，跨工具 continuity 变口号 | P0 致命 | 规则优先 + 必放项硬约束 + budget 硬裁剪 |
| 账本与真实代码状态漂移 | P0 致命 | Reconciliation Engine + Git/FS 为最高事实源 |

### 13.2 次级风险

- 被底层 agent 工具逐步侵蚀价值（应对：明确退化路径）
- 标准化缺乏网络效应（应对：第一阶段不讲标准，只讲项目内 canonical model）
- 长期 adapter 维护债（应对：三档分层，只深接 2 个工具）

---

## 14. 最小 CLI 命令面

```bash
seatloom init
seatloom attach                        # Mode 0 入口
seatloom run "fix login bug"           # Mode 1 入口
seatloom seat add lyra --role product_owner
seatloom handoff create --from lyra --to nimbus --workitem WI-12
seatloom inbox
seatloom timeline
seatloom reconcile

