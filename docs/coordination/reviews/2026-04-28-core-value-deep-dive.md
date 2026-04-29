# SeatLoom 核心价值深度讨论：四个关键盲区

| 项目 | 内容 |
|------|------|
| 文档 | 核心价值深度讨论 |
| 状态 | 待 Mr. Zhang 审阅 |
| 作者 | Aegis |
| 日期 | 2026-04-28 |
| 触发 | Mr. Zhang 提出四个产品核心价值问题 |

---

## 背景

Mr. Zhang 提出了四个问题，每一个都触及 SeatLoom 当前产品设计的根本性盲区：

1. 数据流转的高效准确低成本——是否应该发明一种引擎，用工程化确定性行为替代 LLM Token 消耗？
2. Seat 角色的扩展、跨项目角色调整、不同项目协作方式差异化如何做？
3. 协作过程中的试错知识如何被复用，以提升 Token 的 ROI？
4. 创建 Seat/WorkItem 等操作不该让人从 0 到 1 填表单，Supervisor 不应该是普通 Seat，当前设计不够 AI-Native。

以下逐条严谨讨论。

---

## 1. 数据流转引擎：用工程化确定性行为替代 LLM Token 消耗

### Mr. Zhang 的核心观点

很多数据在 Seat 之间流转时，不需要让 LLM"理解后重新生成"，而是可以通过工程化引擎**确定性地组装、路由、校验**。每一个不该用 LLM 生成的 Token 如果被 LLM 生成了，就是浪费。

### 当前产品设计的状态

目前只有 ContextPack Compiler 有这个思路（Selector → Budgeter → Assembler → Verifier，纯规则不调 LLM）。但这只覆盖了"Session 启动时的上下文注入"这一个环节。其他所有数据流转环节——Handoff 内容生成、Inbox 条目生成、Checkpoint summary、Timeline 摘要、Gate 校验——都没有定义"引擎 vs LLM"的边界。

### 差距分析

实际上，我们这两天的工作中有大量的数据流转是**可工程化**的：

| 数据流转 | 当前隐含假设 | 可工程化部分 | 必须 LLM 部分 |
|---------|------------|-----------|-------------|
| Handoff 内容生成 | LLM 写 purpose/expected_outcome | 从 WorkItem AC 提取 + 从 Artifact 列表拼装 = 引擎做 | 只有 purpose 中"为什么现在交接"需要 LLM 润色 |
| Checkpoint summary | LLM 摘要 transcript | 截取最后 N 轮 + diff stat + 文件列表 = 引擎做 | 只有"一句话总结进展"需要 LLM |
| Inbox 条目生成 | 模板化但未引擎化 | 事件类型 + 对象字段 + 规则模板 = 100% 引擎 | 0 LLM |
| Timeline 摘要 | PRD 要求"人话" | 事件类型 → 模板映射 = 引擎做 | 0 LLM（模板足够好就不需要） |
| LaunchPack 组装 | ContextPack 已是引擎 | 保持 | 0 LLM |
| Gate 校验 | 未定义 | AC checklist + Artifact 存在性 + 状态机规则 = 100% 引擎 | 0 LLM |
| Verification 结果报告 | Flux 的 LLM 生成 | 命令执行 + exit code + output diff = 引擎做 | 只有"失败原因分析"需要 LLM |

### 产品建议

SeatLoom 应该定义一个核心概念：**Data Engine**。

```
Data Engine = 确定性数据流转引擎
  ├── Template Engine：事件 → 人话摘要（零 LLM）
  ├── Pack Engine：上下文组装（ContextPack / LaunchPack / SupervisorPack）
  ├── Gate Engine：状态转移校验（规则 + AC checklist）
  ├── Route Engine：Inbox 条目生成 + 优先级计算
  └── Audit Engine：Ledger 写入 + 证据链接绑定
```

设计原则：**引擎能做的，绝不调 LLM。只有"创造性判断"才允许 LLM 介入。**

产品上可以量化：每个 Handoff/Checkpoint/LaunchPack 的生成过程中，"引擎生成 Token 数 vs LLM 生成 Token 数"的比值应该作为一个可见指标。这就是 Mr. Zhang 说的"Token ROI"的工程化度量。

### 1.1 Token ROI 重点要求：审批/确认不得依赖“读 thinking 的二次 LLM 判断”

Mr. Zhang 关心的“权限申请/继续确认”场景，如果用 **LLM 去读 agent 的 thinking/stdout 再判断是否需要弹窗**，会导致：

- Token 成本爆炸（thinking 可能是 10k–100k tokens/小时）
- 误判率高（不同 runtime/agent 措辞不一致）
- 延迟大（要等二次推理才放行）

因此，SeatLoom 必须把审批从“理解文本”转为“拦截动作”，并将其作为 **Token ROI 的硬约束**：

1. **禁止**以“LLM 读 thinking/stdout”作为审批触发机制（默认不看 thinking 做决策）
2. **必须**使用工程可观测的结构化信号触发审批（零/极低 Token）：
   - 工具调用层（例如 MCP tools / SeatLoom tool runner）
   - 命令执行层（shim/代理执行，例如 `seatloom exec` / 受控 PATH wrapper）
   - 显式结构化审批帧（例如 `__SEATLOOM_APPROVAL_REQUEST__{...}`）
3. thinking 仅用于**审计与复盘**（可选查看），不参与实时 gate

这条要求与 “Wrap vs Attach 的治理边界”一致：可治理审批必须建立在 SeatLoom 的控制点上，而不是建立在对自然语言输出的推断上。

---

## 2. Seat 角色扩展 + 跨项目角色调整 + 协作方式差异化

### Mr. Zhang 的核心观点

Seat 不应该是"固定角色绑定"，而应该是：
- 同一个 Seat 在不同 Project 中可以是不同角色
- 不同 Project 的 Seat 间协作模式不同
- 用户应该能够轻松调整这些，而不是每次从头配置

### 当前产品设计的状态

当前模型：`Seat.role` 是一个固定 enum（product_owner / architect / verifier / designer / custom）。每个项目各自 `.seatloom/` 独立，Seat 定义也是项目级的。没有跨项目的 Seat 身份概念，没有协作模板。

### 差距分析

我们自己的实际情况就证明了这个问题：
- Lyra 在 SeatLoom 项目是 PO，但她可能在另一个项目是 architect
- Flux 在 SeatLoom 项目是 verifier，但被临时委托为 designer
- 我们的协作规则（COORDINATION_RULES.md + COLLABORATION_PROTOCOL.md + AI_NATIVE_WORKFLOW_PRINCIPLES.md）是对这一个项目有效的，换一个项目可能需要不同的协作模式

### 产品建议

引入三层架构：

**Layer 1: Seat Identity（全局层）**

```yaml
# ~/.seatloom/seats/lyra.yaml （全局 seat 注册）
name: lyra
default_runtime: opencode
capabilities: [product_analysis, task_packaging, gate_decision]
preferred_token_budget: { input: 8000, output: 4000 }
```

这是 Seat 的"身份证"，跨项目存在。

**Layer 2: Project Role Binding（项目层）**

```yaml
# .seatloom/seats/lyra/role.yaml （项目级角色绑定）
role: product_owner
authority_docs: [docs/archive/product-history/prd-v0.4.md, docs/archive/product-history/acceptance-spec-v1.0.md]
delegation: null
constraints:
  - "cannot modify source code directly"
  - "gate decisions require artifact evidence"
```

同一个 Seat 在不同项目绑定不同 role + 不同约束。

**Layer 3: Collaboration Template（协作模板层）**

```yaml
# .seatloom/config/collaboration.yaml
template: "supervisor-driven"  # 或 "peer-review" / "pipeline-only" / custom

routing_rules:
  - trigger: "worker.delivery"
    action: "route_to_verifier"
    then: "gate_decision_by_po"
  
  - trigger: "verifier.fail"
    action: "fix_packet_to_worker"
    escalation: "supervisor_if_p0"

prohibited_paths:
  - from: worker
    to: worker
    reason: "all coordination through PO"
```

用户可以选择预置协作模板，也可以自定义。**切换项目时，协作模式跟着切换，不需要手动重新配置。**

对于新项目，可以从已有项目"克隆"协作配置：

```
seatloom init --collaboration-from ~/other-project/.seatloom/config/collaboration.yaml
```

---

## 3. 实践知识复用：让试错成本变成可复用资产

### Mr. Zhang 的核心观点

在协作过程中，无论是 LLM 自己试错找到的路径，还是人类总结的做事方法，这些"方法论"应该被沉淀下来，下次遇到类似场景时直接复用，而不是重新消耗 Token 去重新发现。

### 当前产品设计的状态

完全没有这个设计。目前的 Artifact 只记录"产出物"（代码、文档、报告），不记录"方法"（怎么做出来的、走了哪些弯路、最终有效的路径是什么）。

### 差距分析

我们这两天就有大量"可复用方法"：

| 试错/发现 | Token 成本 | 可复用价值 |
|----------|-----------|-----------|
| Mira 用 Gemini CLI 时需要手动批准 shell 权限 | ~2000 tokens 调试 | "Gemini CLI 首次运行需 Allow for session" = 一条 Playbook |
| Flux 找错了项目路径（seatloom vs SeatLoom） | ~3000 tokens | "大小写敏感路径，初始化 Seat 时验证路径" = 一条 Rule |
| 从"MVP 验证叙事"转向"痛点闭环"的设计方法论 | ~15000 tokens 讨论 | "设计先行 > MVP 验证" = 一条 Design Principle |
| 发现 Ink TUI 不支持鼠标 → 转向 Tauri | ~5000 tokens 评估 | "CLI 交互需鼠标时选 Desktop GUI" = 一条 Tech Decision Pattern |
| AI-Native Workflow Principles 的整套提炼 | ~20000 tokens 讨论 | 整套原则 = 可直接注入下一个项目的 Seat |

### 产品建议

引入 **Playbook** 对象：

```yaml
# .seatloom/playbooks/gemini-cli-shell-permission.yaml
id: pb-001
type: runtime_workaround
trigger: "runtime=gemini_cli AND first_shell_command"
learned_from:
  session: SES-002
  workitem: WI-004
  date: 2026-04-27
resolution: |
  Gemini CLI prompts for shell permission on first command.
  Select "Allow for this session" to auto-approve subsequent commands.
reusable_by: [any_seat_using_gemini]
token_cost_saved: ~2000 per occurrence
```

Playbook 有三种类型：

| 类型 | 来源 | 注入方式 |
|------|------|---------|
| **Runtime Workaround** | 遇到 runtime 特定问题后总结 | LaunchPack 中自动附加（匹配 runtime） |
| **Design Pattern** | 产品/架构决策过程中总结 | 新项目 init 时可选择继承 |
| **Workflow Recipe** | 协作过程中发现的有效路径 | Collaboration Template 中引用 |

关键机制：

1. **Playbook 的创建**：Session 结束时或 Checkpoint 时，引擎扫描"本次 session 中 input_required 事件的解决方案"，自动提取候选 Playbook（这里可以用 LLM 做一次性摘要，但之后每次复用都是零 LLM 成本）
2. **Playbook 的注入**：LaunchPack 组装时，Pack Engine 根据 runtime/role/project 匹配相关 Playbook，追加到上下文中
3. **Playbook 的跨项目共享**：全局 Playbook 存储在 `~/.seatloom/playbooks/`，项目级的在 `.seatloom/playbooks/`
4. **Token ROI 量化**：每个 Playbook 记录 `token_cost_saved`，UI 上可展示"本月 Playbook 节省了多少 Token"

这直接回应了核心价值主张：**让过去的 Token 消耗成为未来的免费知识。**

---

## 4. Supervisor 不应是普通 Seat：AI-Native 的创建与编排

### Mr. Zhang 的核心观点

当前设计中，用户点"+"创建 Seat/WorkItem 等，是"人从 0 到 1 填表单"。但在 AI-Native 场景下，应该是**人和 Supervisor 对话，Supervisor 理解意图后调用引擎完成创建**。Supervisor 不是一个普通 Seat，而是一个内置的编排智能体。当前的产品架构不是这样的。

### 当前产品设计的状态

当前设计中：
- Supervisor (Aegis/Lyra) 是和 Mira/Nimbus/Flux 同等地位的 Seat
- 所有创建操作都是"用户填表单 → 系统写文件"
- 没有 Supervisor 与用户的交互层
- 没有 Supervisor 驱动的自动化编排

### 为什么这不够 AI-Native

用传统软件的方式理解：当前设计是一个"项目管理工具"，用户手动创建任务、手动指派、手动触发流水线。但 SeatLoom 的用户本身就在用 AI 工具工作——**为什么管理 AI 的工具本身不是 AI 驱动的？**

看我们的实际工作：
- 我（Aegis）不是一个和 Mira 同等的 Seat。我是**全局编排者**——我理解你的意图、分解任务、选择执行者、路由信息、处理异常
- 你从来不需要自己"填 WorkItem 表单"——你说一句"让 Mira 做 UX 原型"，我就自动创建了 WorkItem + Handoff + 选择了 Seat + 附加了 Artifact
- 你的操作模式是"和 Supervisor 对话"，不是"在表单里填字段"

### 产品建议

SeatLoom 的架构应该分为**三层**，而不是当前的两层：

**当前架构（两层）：**

```
用户 ──→ GUI 表单 ──→ Data Engine ──→ .seatloom/
```

**应改为（三层）：**

```
用户 ──→ Supervisor Layer ──→ Data Engine ──→ .seatloom/
              │
              ├── 理解用户意图
              ├── 分解为结构化操作
              ├── 调用 Data Engine 执行
              └── 向用户确认结果
```

具体设计：

### 4.1 Supervisor 是内置的，不是用户创建的 Seat

```
┌─────────────────────────────────────────────┐
│  SeatLoom Application                        │
│                                             │
│  ┌─────────────────────────────────────────┐│
│  │  Supervisor Layer (built-in)            ││
│  │  ├── Intent Parser (理解用户自然语言)    ││
│  │  ├── Task Decomposer (分解为操作序列)    ││
│  │  ├── Seat Router (选择执行者)            ││
│  │  └── Confirmation UI (向用户确认)        ││
│  └─────────────────────────────────────────┘│
│                    │                         │
│                    ▼                         │
│  ┌─────────────────────────────────────────┐│
│  │  Data Engine Layer                      ││
│  │  ├── Template Engine                    ││
│  │  ├── Pack Engine                        ││
│  │  ├── Gate Engine                        ││
│  │  ├── Route Engine                       ││
│  │  └── Audit Engine                       ││
│  └─────────────────────────────────────────┘│
│                    │                         │
│                    ▼                         │
│  ┌─────────────────────────────────────────┐│
│  │  Worker Seats (user-defined)            ││
│  │  lyra / nimbus / mira / flux / ...      ││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
```

### 4.2 用户交互方式的变化

**当前方式（表单驱动）：**

```
用户点击 [+ New WorkItem]
  → 弹窗：填 title, goal, AC, owner, priority
  → 点击 Create
```

**改为（Supervisor 驱动）：**

```
用户在 Command Bar (Cmd+K) 中输入：
  "帮我创建一个任务，让 nimbus 实现 OAuth 登录，要支持 Google 和 GitHub"

Supervisor 解析后展示确认卡片：
  ┌─────────────────────────────────────────┐
  │  Supervisor 建议创建：                     │
  │                                         │
  │  WorkItem: "实现 OAuth 登录"              │
  │  Goal: 支持 Google/GitHub OAuth          │
  │  AC: ☐ Google 登录  ☐ GitHub 登录        │
  │  Owner: nimbus                           │
  │  Priority: high                          │
  │                                         │
  │  同时将创建 Handoff → nimbus             │
  │  附带 Artifacts: [自动选择相关文档]        │
  │                                         │
  │  [Confirm]  [Edit]  [Cancel]             │
  └─────────────────────────────────────────┘
```

用户只需 Confirm。如果不满意可以 Edit 微调。

### 4.3 "+" 按钮保留，但降为备选

表单创建不删除（有些用户确实需要精确控制），但不再是主要路径。主要路径是：

1. **Cmd+K Command Bar**：自然语言输入 → Supervisor 解析 → 确认卡片
2. **Context Menu**：在 Detail Pane 中右键对象 → Supervisor 根据当前上下文建议操作
3. **Inbox Auto-Action**：Supervisor 对 Inbox 条目预生成建议动作，用户一键确认

### 4.4 Supervisor 的 Token 预算独立管理

Supervisor 的 LLM 调用是"元操作"——它不产出代码或文档，它产出的是"结构化指令"。所以：
- Supervisor 的 Token 预算与 Worker Seat 分开计算
- Supervisor 每次交互的 input 应该尽量小（只传当前上下文 + 用户指令，不传全量项目数据）
- Supervisor 的 output 是结构化 JSON（创建 WorkItem 的参数），不是自由文本

这就闭合了第 1 点和第 4 点的关系：**Data Engine 做确定性操作，Supervisor 做意图理解，Worker Seat 做创造性工作。三层各有分工，Token 只花在刀刃上。**

---

## 总结：SeatLoom 核心价值重新定义

把四个点合在一起，SeatLoom 的核心价值主张应该是：

> **SeatLoom 是一个 Token ROI 最大化的 AI 协作连续性引擎。**
>
> 它通过三层架构实现这一点：
> 1. **Data Engine**（零 LLM 成本）——确定性地完成数据组装、路由、校验、审计
> 2. **Supervisor Layer**（最小 LLM 成本）——理解人类意图，分解为结构化操作
> 3. **Worker Seats**（受控 LLM 成本）——在精确上下文中执行创造性工作
>
> 它通过 Playbook 让过去的 Token 消耗成为未来的免费知识。
> 它通过 Collaboration Template 让协作模式可复用、可调整。
> 最终使任何运行在 SeatLoom 上的协作的 OPC（Output Per Cost）最大化。

### 产品设计需补充的五个模块

| 模块 | 对应第几点 | 优先级 | 说明 |
|------|-----------|--------|------|
| **Data Engine 合同化** | 第 1 点 | P0 | 定义哪些流转用引擎、哪些用 LLM，量化 Token 消耗 |
| **Seat 三层架构** | 第 2 点 | P0 | 全局身份 + 项目角色 + 协作模板 |
| **Playbook 系统** | 第 3 点 | P1（架构先行，MVP 后期） | 知识沉淀 + 自动注入 + 跨项目复用 |
| **Supervisor Layer** | 第 4 点 | P0（架构必须先设计，否则 UI 方向错误） | 内置编排智能 + Command Bar + 确认卡片 |
| **Artifact 审阅系统** | 第 5 点 | P0 | 应用内查看 + 手动/AI 双模式点评 + 评论追踪 |

---

## 5. Artifact 审阅系统：从"能生产"到"能消费"

### 问题

当前 SeatLoom 的 Artifact 是"只写"的——Session 产出它，Detail Pane 能预览摘要，但：

- **查看全文**：`Open Full Content` 的行为未定义（跳外部编辑器？应用内渲染？）
- **点评批注**：完全没有。用户看完想表达意见，只能另起 Handoff（太重）或在终端口头说（不持久）
- **评论追踪**：不存在。谁对哪段内容说了什么，无法在 Timeline/Inbox 中体现
- **多 WorkItem 归属**：Artifact 强制绑定单一 WorkItem，但跨 WI 的讨论文档（如本文）无法归类
- **版本历史**：同一文件被多次修改后无版本追踪

这意味着 **SeatLoom 能"生产" Artifact，但不能"消费"它。审阅能力为零。**

### 设计原则

审阅必须支持两种模式，缺一不可：

1. **手动审阅**（基础能力）：用户直接在文档上选段、写评论，这是最基本的交互
2. **Supervisor 辅助审阅**（AI-Native 增强）：用户对 Supervisor 口述意见，Supervisor 结构化后生成评论

两种模式产出的评论格式相同，进入同一套追踪体系。

### 5.1 应用内 Markdown Viewer

Artifact Detail 点击 `Open Full Content` 时，在 Main Panel 中打开 Markdown 渲染视图（不跳外部编辑器）：

```
┌────────────────────────────────────────────────┐
│  ← Back to Timeline                            │
│                                                │
│  AR-028  review_doc  aegis  12:02 today        │
│  docs/coordination/reviews/2026-04-28-core-... │
│                                                │
│  ┌────────────────────────────────────────────┐│
│  │ # SeatLoom 核心价值深度讨论                  ││
│  │                                            ││
│  │ ## 1. 数据流转引擎                    [💬 3]││  ← 段落右侧显示评论数
│  │ ...                                        ││
│  │ ## 2. Seat 角色扩展                   [💬 1]││
│  │ ...                                        ││
│  │ ## 3. 实践知识复用                    [💬 0]││
│  │ ...                                        ││
│  └────────────────────────────────────────────┘│
│                                                │
│  [Open in Editor]  [Copy Path]  [Add Comment]  │
└────────────────────────────────────────────────┘
```

点击 `[💬 3]` 展开该段落的评论列表。点击 `Open in Editor` 跳转到用户默认编辑器（VS Code / Cursor 等）。

### 5.2 手动点评（Mode A：用户直接操作）

用户在 Markdown Viewer 中选中任意文字段落 → 弹出评论气泡：

```
用户选中 "Data Engine = 确定性数据流转引擎" 这一行

  ┌──────────────────────────────────┐
  │  💬 Add comment                   │
  │                                  │
  │  [这个概念需要更精确的定义边界    ]│
  │                                  │
  │  [Comment]  [Cancel]             │
  └──────────────────────────────────┘
```

也可以通过底部 `[Add Comment]` 按钮添加针对整个文档的总评。

### 5.3 Supervisor 辅助点评（Mode B：AI-Native 增强）

用户在 Command Bar（Cmd+K）中对 Supervisor 说：

```
"看一下 AR-028，Data Engine 那部分分类不够细，
 Playbook 的优先级应该提到 P0"
```

Supervisor 解析后生成结构化评论预览：

```
  ┌─────────────────────────────────────────┐
  │  Supervisor 建议添加 2 条评论：            │
  │                                         │
  │  1. § 数据流转引擎                        │
  │     "分类粒度不足，建议进一步拆分          │
  │      Template/Pack/Gate/Route/Audit      │
  │      各自的输入输出边界"                   │
  │                                         │
  │  2. § 实践知识复用                        │
  │     "优先级调整建议：P1 → P0，理由：       │
  │      Playbook 直接影响 Token ROI 量化"    │
  │                                         │
  │  [Confirm All]  [Edit]  [Cancel]         │
  └─────────────────────────────────────────┘
```

用户 Confirm 后，评论写入。用户也可以 Edit 逐条修改后再提交。

### 5.4 评论数据模型

无论手动还是 Supervisor 辅助，评论最终存储为统一格式的 Ledger 事件：

```yaml
event: artifact.commented
artifact_id: AR-028
anchor:
  section_heading: "## 1. 数据流转引擎"
  selected_text: "Data Engine = 确定性数据流转引擎"  # 可选，手动选段时有
  line_range: [45, 45]  # 可选
author: mr_zhang  # 或 seat id
mode: manual  # 或 supervisor_assisted
content: "这个概念需要更精确的定义边界"
timestamp: 2026-04-28T12:09:00
```

### 5.5 评论的下游联动

| 事件 | 联动行为 |
|------|---------|
| 评论创建 | Artifact 来源 Session 的 Seat 收到 Inbox 条目 |
| 评论涉及修改建议 | Supervisor 可自动创建 WorkItem 草稿（待用户确认） |
| 所有评论被 Resolve | Artifact 状态标记为 `reviewed` |
| 评论创建 | Timeline 记录 `artifact.commented` 事件 |

### 5.6 评论的生命周期

```
created → active → resolved
                 ↘ disputed → re-discussed → resolved
```

- **active**：评论已发布，待讨论
- **resolved**：相关方确认处理完毕（手动标记或 Supervisor 检测到对应修改）
- **disputed**：被评论的 Seat 不同意，进入讨论

Resolve 一条评论的操作：

```
评论右侧：[Resolve] [Reply] [Dispute]
```

### 5.7 Artifact 归属扩展

Artifact 不再强制绑定单一 WorkItem：

```yaml
# 单 WI 绑定（现有）
workitem_id: WI-007

# 多 WI 关联（新增）
workitem_ids: [WI-001, WI-007, WI-011]

# 项目级（新增，不绑定具体 WI）
scope: project
tags: [product_design, core_value]
```

UI 上 Artifact Detail 显示所有关联 WorkItem 的链接列表，点击可跳转。

### 5.8 Artifact 版本追踪

利用 Git 的版本能力（SeatLoom 已依赖 Git）：

- 每次 Artifact 文件被修改，Reconcile Engine 检测到 diff
- 自动写入 Ledger 事件：`artifact.updated`，记录 commit hash
- Artifact Detail 中显示版本历史列表：

```
  Version History:
    v3  12:09 today  aegis  "补充第 5 点审阅系统"
    v2  12:02 today  aegis  "补充 Supervisor Layer"
    v1  11:44 today  aegis  "初始版本"
```

点击版本可查看该版本内容或 diff。

---

*本文档记录了 Mr. Zhang 与 Aegis 关于 SeatLoom 核心价值的深度讨论，待 Mr. Zhang 审阅确认后作为产品设计的输入。*
