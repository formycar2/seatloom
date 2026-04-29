# Playbook vs Agent Skills，以及 MCP / A2A / Harness 对 SeatLoom 的借鉴

| 项目 | 内容 |
|------|------|
| 文档 | 生态对标分析 |
| 状态 | 待 Mr. Zhang 审阅 |
| 作者 | Aegis |
| 日期 | 2026-04-28 |

---

## 一、SeatLoom Playbook vs Agent Skills：本质区别

### 1.1 Agent Skills 是什么

Agent Skills（由 Anthropic 发起，2025 年底开源，现已被 Cursor、Claude Code、Codex CLI、Gemini CLI、GitHub Copilot 等 26+ 平台采纳）是一个开放标准，用于给 AI Agent 注入**能力和专业知识**。

核心结构：

```
.cursor/skills/deploy-staging/
  ├── SKILL.md          ← YAML 元数据 + Markdown 指令
  ├── scripts/          ← 可执行脚本
  ├── references/       ← 参考文档
  └── assets/           ← 模板、数据文件
```

工作机制（三阶段渐进加载）：
1. **Discovery**（~100 tokens）：启动时只读 name + description，判断何时相关
2. **Activation**（<5000 tokens）：匹配到任务时，加载完整 SKILL.md 指令
3. **Execution**：执行指令，按需加载 scripts/references/assets

关键特征：
- **教 Agent "怎么做事"**：比如"我们团队的部署流程是…"、"我们的 API 路由规范是…"
- **便携跨平台**：同一个 Skill 在 Cursor / Claude Code / Codex 都能用
- **渐进加载**：不预加载全部内容，按需注入，节省 Token
- **项目级 + 用户级**：`.cursor/skills/`（项目）和 `~/.cursor/skills/`（全局）

### 1.2 SeatLoom Playbook 是什么

Playbook 是我们在 `core-value-deep-dive.md` 中提出的概念，用于沉淀**试错经验和最佳实践**，使其可被自动复用以节省 Token。

### 1.3 核心区别

| 维度 | Agent Skills | SeatLoom Playbook |
|------|-------------|------------------|
| **本质** | 教 Agent 做事的指令包（How to） | 从实践中沉淀的经验知识（Learned from） |
| **创建方式** | 人工编写 SKILL.md | 从 Session 试错中自动/半自动提取 |
| **触发方式** | Agent 根据 description 自动匹配，或用户 `/skill-name` 手动调用 | Pack Engine 根据 runtime/role/context 自动匹配注入 |
| **消费者** | 单个 Agent（在一个 Session 内） | 跨 Seat 跨 Session（任何遇到类似场景的 Seat） |
| **生命周期** | 静态文件，手动维护 | 活的知识：有来源 Session、有节省统计、可被验证/废弃 |
| **跨项目** | 全局 Skills（`~/.cursor/skills/`） | 全局 Playbook（`~/.seatloom/playbooks/`） |
| **与协作的关系** | 无——Skills 不知道有其他 Seat | 深度绑定——Playbook 知道哪个 Seat 在哪个 runtime 遇到了什么问题 |
| **Token ROI 量化** | 无 | 每个 Playbook 记录 token_cost_saved |

### 1.4 关键洞察：Playbook 和 Skills 不是替代关系，而是互补关系

```
Agent Skills = 教 Agent "做事方法"（预设的、通用的）
SeatLoom Playbook = 从实践中"学到的经验"（后验的、具体的）
```

**类比**：Skills 是"新员工入职手册"，Playbook 是"老员工的踩坑笔记"。两者都需要。

### 1.5 SeatLoom 应该怎么做

**借鉴 Skills 的地方**：
- 渐进加载机制（Discovery → Activation → Execution）—— 直接采用，Playbook 也应该只在匹配时注入
- 标准化文件格式 —— Playbook 可以采用类 SKILL.md 的 YAML 元数据 + Markdown 结构
- 项目级 + 全局级存储 —— 已在设计中

**不应照搬的地方**：
- Skills 是静态手写的；Playbook 的核心价值在于**从 Session 中自动提取**
- Skills 不追踪"这个 Skill 节省了多少 Token"；Playbook 必须量化 ROI
- Skills 不知道协作上下文；Playbook 必须绑定 Seat/Runtime/WorkItem

**最终产品形态**：SeatLoom 中应该同时存在两个概念：

| 概念 | 用途 | 谁写 | 何时注入 |
|------|------|------|---------|
| **Seat Skill** | 定义 Seat 的能力和做事方式（等价于 Agent Skill） | 人工编写 | Session 启动时，由 Pack Engine 注入 LaunchPack |
| **Playbook** | 沉淀试错经验和运行时最佳实践 | 自动/半自动从 Session 提取 | Session 启动时，由 Pack Engine 根据 runtime/role 匹配注入 |

---

## 二、MCP 对 SeatLoom 的借鉴

### 2.1 MCP 是什么

Model Context Protocol（Anthropic 发起，2024 年 11 月发布，被称为"AI 的 USB-C"）是一个开放的 JSON-RPC 协议，标准化 AI Agent 与外部工具/数据源的连接方式。

架构：Host → Client → Server（三层）

五个原语：
1. **Tools**：Agent 可调用的函数（查询数据库、调 API 等）
2. **Resources**：Agent 可读取的数据源（文件内容、数据库记录等）
3. **Prompts**：预定义的工作流模板
4. **Sampling**：Server 反向请求 Host 进行 LLM 推理
5. **Roots**：向 Server 暴露 Host 的文件系统根目录

传输层：stdio（本地）/ Streamable HTTP（远程）/ WebSocket（起草中）

### 2.2 MCP 与 SeatLoom 的关系

SeatLoom 不是 MCP 的替代品，也不是 MCP 的竞争者。**MCP 解决 Agent 如何连接工具，SeatLoom 解决多个 Agent 如何协作和保持连续性。**

但 MCP 有三个设计理念值得 SeatLoom 深度借鉴：

#### 借鉴 1：能力协商机制（Capability Negotiation）

MCP 中 Client 和 Server 在初始化时明确声明各自支持的能力。SeatLoom 的 Seat 目前没有这个机制——Seat 的 role 是一个粗粒度 enum，没有声明"我能做什么、不能做什么"。

**产品建议**：Seat 注册时应声明 capabilities：

```yaml
seat: nimbus
capabilities:
  tools: [code_generation, architecture_design, code_review]
  input_formats: [markdown, yaml, typescript]
  output_formats: [code, markdown, diff]
  max_context_tokens: 128000
  supports_streaming: true
```

这使得 Supervisor 在路由任务时可以根据 capability 匹配，而不是硬编码"nimbus 是 architect 所以给他技术任务"。

#### 借鉴 2：渐进加载 / 按需获取

MCP 的 Resource 是按需读取的——Agent 不会预加载所有数据，而是在需要时 `resources/read`。SeatLoom 的 LaunchPack 目前是"一次组装全部注入"，没有按需获取机制。

**产品建议**：LaunchPack 应分层：
- L0（必注入）：WorkItem title + AC + 当前状态（~500 tokens）
- L1（默认注入）：最近 Handoff + 最近 Checkpoint（~2000 tokens）
- L2（按需获取）：完整 Artifact 内容、历史 Session 摘要（Seat 在 Session 中主动请求）

这直接呼应了第 1 点讨论中 Data Engine 的设计。

#### 借鉴 3：Server 隔离原则

MCP 的核心安全设计：Server 看不到完整对话历史，Server 之间互相隔离，一切由 Host 编排。

**这恰好就是 SeatLoom 的 AI-Native Workflow Principles 中的"need-to-know by default"和"supervisor-only global view"。** MCP 在协议层面用工程手段实现了这个原则——SeatLoom 目前只是用文档约定了这个原则，没有工程化强制执行。

**产品建议**：SeatLoom 的 Data Engine 应在架构层面强制执行 Seat 隔离——Worker Seat 在 Session 中只能看到 Pack Engine 注入的内容，不能自主访问其他 Seat 的 Session 数据或全局 Ledger。

### 2.3 SeatLoom 是否应该实现为 MCP Server？

**值得认真考虑。** 如果 SeatLoom 的 Data Engine 暴露为 MCP Server：

```
Agent (Claude/Codex/Cursor)
  └── MCP Client
        └── SeatLoom MCP Server
              ├── Tool: create_workitem(title, goal, ac)
              ├── Tool: send_handoff(from, to, workitem, purpose)
              ├── Tool: get_inbox()
              ├── Resource: workitem/{id}
              ├── Resource: session/{id}/checkpoint
              └── Resource: timeline?seat=nimbus&since=24h
```

这意味着：任何支持 MCP 的 Agent 工具都可以直接与 SeatLoom 交互——不需要 SeatLoom 自己实现 Terminal Panel 来嵌入 Agent，而是让 Agent 工具（Cursor/Claude Code/Codex）通过 MCP 连接 SeatLoom。

**这是一个可能改变产品形态的架构选择，值得单独讨论。**

---

## 三、A2A 对 SeatLoom 的借鉴

### 3.1 A2A 是什么

Agent-to-Agent Protocol（Google DeepMind 发起，2025 年 4 月发布，2026 年 3 月发布 v1.0，Linux Foundation 托管），标准化 AI Agent 之间的通信。

核心概念：
- **Agent Card**：Agent 的自我描述（能力、端点、认证方式），发布在 `/.well-known/agent.json`
- **Task**：工作单元，有状态生命周期（submitted → working → completed / failed）
- **Message**：Agent 之间的结构化通信
- **Artifact**：Agent 产出的结果物

MCP 是"垂直"的（Agent → Tool），A2A 是"水平"的（Agent ↔ Agent）。

### 3.2 A2A 与 SeatLoom 的关系

**SeatLoom 的 Seat 间协作本质上就是 A2A 要解决的问题。** 但有一个关键差异：

| 维度 | A2A | SeatLoom |
|------|-----|---------|
| Agent 部署 | 独立服务，可能跨组织 | 本地进程，同一台机器 |
| 通信方式 | HTTP + SSE | 本地文件 + tmux / PTY |
| Task 状态 | 协议级状态管理 | Ledger + 文件系统 |
| 发现机制 | Agent Card（HTTP 端点） | Seat Profile（YAML 文件） |

### 3.3 值得借鉴的地方

#### 借鉴 1：Agent Card → Seat Card

A2A 的 Agent Card 是一个结构化的自描述文件，声明 Agent 能做什么、接受什么输入、用什么认证。SeatLoom 的 Seat Profile 目前只有 name + role，太弱了。

**产品建议**：Seat Card 应该是 Seat 的完整"能力简历"：

```yaml
# .seatloom/seats/nimbus/card.yaml
name: nimbus
role: architect
capabilities:
  - code_generation
  - architecture_design
  - code_review
accepted_input_types:
  - task_packet
  - fix_packet
  - integration_packet
output_types:
  - code_change
  - implementation_report
  - blocker_report
preferred_runtime: codex
token_budget:
  input: 32000
  output: 16000
constraints:
  - "cannot redefine product scope"
  - "must report exact commits and files"
```

Supervisor 和 Route Engine 根据 Seat Card 做任务分配，而不是靠硬编码。

#### 借鉴 2：Task 生命周期标准化

A2A 的 Task 有标准状态机：submitted → working → completed / failed / canceled。SeatLoom 的 WorkItem 状态机更复杂（draft → ready → active → in_review → done + 异常分支），但**Handoff 缺少明确的"执行进度"表达**。

**产品建议**：在 Handoff 状态中增加 working 状态，允许接收方反馈进度：

```
drafted → sent → accepted → working (进度可见) → completed
                           ↘ returned
```

#### 借鉴 3：Streaming 更新

A2A 支持 Task 的 SSE streaming——发起方可以实时看到执行方的进度。SeatLoom 目前没有这个——Handoff 发出后，发送方只能等 Inbox 中出现"completed"通知。

**产品建议**：对于 Pipeline 执行和长时间 Session，在 Timeline 中支持实时事件流（不需要 SSE 协议，利用本地文件系统 watch 即可）。

---

## 四、Agent Harness 对 SeatLoom 的借鉴

### 4.1 Agent Harness 是什么

Agent Harness 是 2026 年最热门的工程概念之一。核心观点：**模型是可替换的商品，Harness（运行时基础设施）才是决定 Agent 可靠性的关键。**

Anthropic 的定义："让 AI 模型能够跨多个上下文窗口处理长任务的操作结构。"

五层架构：

| 层 | 职责 | 缺失时的后果 |
|----|------|------------|
| 1. Orchestration | 控制执行流程 | Agent 无方向地运行或不终止 |
| 2. Context Management | 管理模型看到什么 | 幻觉、上下文腐烂、状态丢失 |
| 3. Tool Integration | 连接外部系统 | 工具调用失败无声传播 |
| 4. Verification | 验证每一步输出 | 错误结果被自信地交付 |
| 5. Operations | 监控、成本控制、故障处理 | 成本失控、静默退化、无法调试 |

关键洞察："Harness 在每个任务步骤上做的工作比模型本身还多。模型替换影响输出质量 10-15%。Harness 设计是 80% 的决定因素。"

### 4.2 Harness 五层与 SeatLoom 的映射

| Harness 层 | SeatLoom 对应 | 当前状态 | 差距 |
|-----------|-------------|---------|------|
| **Orchestration** | Supervisor Layer + Collaboration Protocol | 只有文档约定，无工程实现 | 需要变成 Data Engine 中的 Route Engine |
| **Context Management** | ContextPack Compiler + LaunchPack | 有设计，但只覆盖 Session 启动 | 需要扩展到每一步（Pack Engine 渐进加载） |
| **Tool Integration** | Adapter（wrapper_capture, native_attach） | 有设计 | 需要增加 MCP Server 集成路径 |
| **Verification** | Pipeline (verification_loop) + Flux | 有设计但太重（整个 Pipeline 才能验证） | 需要增加轻量级步骤验证（Gate Engine） |
| **Operations** | Ledger + Timeline + Reconcile | 有审计和回放 | 缺少成本监控（Token 消耗追踪）和预算强制执行 |

### 4.3 最关键的借鉴：Token Budget 的工程化强制执行

Harness 架构中，Token 预算不是建议，而是**硬约束**：

> "Token budget enforcement caps cumulative token spend per task. The harness tracks tokens consumed across all LLM calls in a session and terminates gracefully when the budget is exhausted."

SeatLoom 的 AI-Native Workflow Principles 中提到了"token budget as policy"（P4），但只是原则声明，没有工程实现。

**产品建议**：Data Engine 必须包含 Budget Enforcer：

- 每个 Session 有 token budget（input + output 分别计）
- 每个 Handoff 有 token budget（Pack Engine 组装时不超过预算）
- 每个 Pipeline 有 token budget（stage 级别 + 整体级别）
- 预算耗尽时结构化终止：给 Seat 一次机会产出部分结果或升级，然后关闭 Session
- UI 上实时显示当前 Session 的 Token 消耗进度条

这就把我们在第 1 点讨论的"Token ROI"从理念变成了可执行的产品功能。

### 4.4 Harness 的 Verification Loop 启示

Harness 最佳实践：**每一步工具调用之后都有验证**，不是等整个任务做完才验证。

SeatLoom 当前的验证是"Pipeline 级别"的——只有触发 verification_loop Pipeline 才会验证。但更高效的方式是：

- Gate Engine 在每次状态转移时做轻量校验（AC 存在性、必填字段、关联完整性）
- 只有需要运行测试/执行代码的验证才走 Pipeline
- 轻量验证零 LLM 成本，重量验证才消耗 Token

---

## 五、Harness（Harness.io 产品）对 SeatLoom 的借鉴

### 5.1 Harness Agents 产品设计

Harness.io 的 Harness Agents 是另一个具体产品（不同于 Harness Engineering 的概念）。它将 AI Agent 嵌入 CI/CD Pipeline：

- Agent 作为 Pipeline Step 执行（继承 Pipeline 的权限、密钥、治理）
- Agent 是 Pipeline-native——不是独立运行的，而是在 Pipeline 编排下运行
- 每个 Agent 底层都是一个 Pipeline 定义（可 fork、可编辑 YAML、可版本管理）
- 用 MCP 连接工具层

### 5.2 可借鉴的设计

#### "Agent 即 Pipeline" 的透明化

Harness 的核心理念：Agent 不是黑盒。每个 Agent 的行为都是可检查的 Pipeline 定义。

SeatLoom 的 Seat 目前是半黑盒——用户知道 Seat 的 role 和 capabilities，但不知道它具体怎么执行任务。Seat 内部的行为完全由 LLM 的 prompt 决定，不可审计。

**产品建议**：SeatLoom 中每个 Seat 的执行行为应该有一个可检查的"Execution Template"：

```yaml
# .seatloom/seats/flux/execution_template.yaml
name: verification
steps:
  - name: setup_environment
    type: engine  # 零 LLM
    action: checkout_branch + install_deps
  - name: run_tests
    type: engine
    action: execute_test_command
  - name: analyze_results
    type: llm  # 需要 LLM
    action: interpret_test_output
    token_budget: 4000
  - name: generate_report
    type: engine  # 零 LLM
    action: template_fill(verification_report)
```

这实现了：
1. 透明性：用户可以看到 Seat 的执行步骤
2. Token 精确控制：只有 `type: llm` 的步骤消耗 Token
3. 可审计：Ledger 记录每一步的输入输出

---

## 六、总结：SeatLoom 应该借鉴什么

| 来源 | 借鉴内容 | 对应 SeatLoom 模块 | 优先级 |
|------|---------|-------------------|--------|
| **Agent Skills** | 渐进加载三阶段（Discovery → Activation → Execution） | Pack Engine | P0 |
| **Agent Skills** | 标准化文件格式（YAML 元数据 + Markdown） | Playbook + Seat Skill | P0 |
| **Agent Skills** | 项目级 + 全局级双层存储 | Playbook | P1 |
| **MCP** | 能力协商机制 | Seat Card（capabilities 声明） | P0 |
| **MCP** | 按需获取 / 分层加载 | LaunchPack L0/L1/L2 分层 | P0 |
| **MCP** | Server 隔离原则的工程化实现 | Data Engine 隔离层 | P0 |
| **MCP** | SeatLoom 暴露为 MCP Server | 架构选择（需单独讨论） | P1 |
| **A2A** | Agent Card → Seat Card 结构化描述 | Seat Profile 升级 | P0 |
| **A2A** | Task 生命周期 + streaming 进度 | Handoff working 状态 + Timeline 实时流 | P1 |
| **Harness Eng** | Token Budget 工程化强制执行 | Data Engine Budget Enforcer | P0 |
| **Harness Eng** | 每步验证（不是只在最后验证） | Gate Engine 轻量校验 | P0 |
| **Harness Eng** | 五层架构对齐 | SeatLoom 架构层检查 | 设计参考 |
| **Harness.io** | "Agent 即 Pipeline" 透明化 | Seat Execution Template | P1 |
| **Harness.io** | Agent 行为可 fork 可编辑 | Seat 行为的版本管理 | P2 |

### 一句话总结

> SeatLoom 不需要重新发明 Agent Skills / MCP / A2A / Harness 中的任何一个。
> 它需要的是：**用 MCP 的隔离和按需加载做 Data Engine，用 Agent Skills 的格式做 Playbook/Skill，用 A2A 的 Agent Card 做 Seat Card，用 Harness 的预算强制和步骤验证做 Operations。**
> 
> SeatLoom 自己独有的价值是把这些全部串起来的**协作连续性层**——这是以上任何一个协议/产品都没有覆盖的。

---

*本文档待 Mr. Zhang 审阅。结论应反馈给 Lyra 纳入 PRD v0.5 设计。*
