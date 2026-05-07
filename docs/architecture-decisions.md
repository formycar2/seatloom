# SeatLoom Architecture Decisions

| 项目 | 内容 |
|------|------|
| 文档 | Architecture Decisions v1.0 |
| 状态 | Approved |
| 更新时间 | 2026-05-07 |
| 对齐版本 | v0.5 contract set (AD-008–AD-012 added); v2 UI L1 surface (AD-013 v2 — viewMode orthogonal state, chan-03 + chan-09) |
| 审批人 | 张小龙 |

---

## AD-001: 产品形态 — 桌面应用 + CLI

**决定**：SeatLoom 的主交互面是桌面 GUI 应用（窗口化，鼠标+键盘操作），辅以 CLI 子命令用于脚本和自动化。

**理由**：
- 用户需要同时观察多个 Seat/Session/WorkItem 的状态，侧边栏 + 面板布局比逐条敲命令效率高
- 参照产品：Claude Code Desktop（侧边栏 + 并行 session）、Cherry Studio（面板式交互）、Cursor Desktop
- 开发者是常驻使用，需要点选、Tab 切换、滚动翻页、快捷键等原生桌面交互

**产品形态**：

```
SeatLoom Desktop
├── 桌面主应用（窗口化 GUI）
│   ├── 侧边栏：Seat 列表、Session 列表、WorkItem 列表
│   ├── 主面板：Timeline / Inbox / Detail 视图切换
│   ├── 内嵌终端：wrap/attach 的 agent session 直接在应用内交互
│   └── 操作方式：鼠标点选、Tab 切换、上下滚动、快捷键、翻页
│
├── CLI（无 GUI，脚本友好）
│   ├── seatloom init / attach / wrap / reconcile
│   ├── seatloom workitem create / handoff create
│   └── seatloom pipeline run
│
└── 启动方式
    ├── 双击图标 / Dock → 桌面 GUI
    └── 终端敲 seatloom <cmd> → CLI 模式
```

**否决方案**：
- 纯 CLI（敲命令 → 看输出）：交互效率低，不支持鼠标
- TUI（终端内全屏应用如 Bubbletea）：受终端能力限制，无法做到桌面级交互体验

---

## AD-002: 技术栈 — Tauri 2 + React/TypeScript + Rust

**决定**：

| 层 | 技术选型 | 说明 |
|----|---------|------|
| 前端 | React 19 + TypeScript + Zustand | 桌面 GUI，运行在 Tauri webview 中 |
| 后端 | Rust (Tauri 2 backend) | .seatloom/ 读写、Ledger、PTY、Git、Pipeline |
| CLI | Rust (clap)，共享后端 core | 无 GUI 的命令行接口 |
| 内嵌终端 | xterm.js (前端) + Rust PTY (后端) | agent session 在应用内交互 |
| 构建 | Vite (前端) + Cargo (后端) | Tauri 标准工具链 |
| 包管理 | pnpm (前端) + Cargo (后端) | — |

**为什么选 Tauri 2 而不是 Electron**：

| 维度 | Tauri 2 | Electron |
|------|---------|----------|
| 包体积 | 3-10 MB | 80-150 MB |
| 内存占用（空闲） | 40-80 MB | 200-400 MB |
| 启动速度 | 200-500 ms | 1-2 秒 |
| CPU 空闲 | <1% | 1-5% |
| 安全模型 | Capability-based，前端沙箱 | 需手动配置 contextIsolation |
| 单二进制分发 | 原生支持 | 需额外打包 |

SeatLoom 是开发者常驻工具，用户同时跑着 Codex、Claude Code、Cursor。不能再吃 200MB 内存。

**先例验证**：Claudette（Claude Code companion app）使用 Tauri 2 + React/TS + Rust 后端，架构与 SeatLoom 高度相似。

**为什么前端用 React/TypeScript**：
- AI agent 团队（Nimbus/Mira）对 React/TS 编码效率最高
- Cherry Studio、Claudette 等同类产品均使用 React
- Mira 可以直接产出 browser-renderable 的 React 组件原型

**Rust 的代价和应对**：
- AI agent 写 Rust 比 TS 慢 → Tauri 架构前后端分离，前端 React/TS 占 70%+ 工作量，Rust 只写 IPC handler 和核心数据层
- Rust 初次编译慢 (~48s) → 增量编译 3.5s，前端 hot reload 不受影响

---

## AD-003: 架构分层

```
┌──────────────────────────────────────────────────┐
│  Frontend (React / TypeScript / Vite)             │
│  ├── Layout: Sidebar + Main Panel + Detail Pane   │
│  ├── Views: Inbox, Timeline, WorkItem, Session    │
│  ├── Embedded Terminal (xterm.js)                  │
│  ├── State: Zustand                               │
│  └── IPC: @tauri-apps/api → Rust backend          │
├──────────────────────────────────────────────────┤
│  Backend (Rust / Tauri 2)                         │
│  ├── seatloom-core (library crate)                │
│  │   ├── ledger: append-only event store          │
│  │   ├── objects: Seat, Session, WorkItem, etc.   │
│  │   ├── data_engine: Data Engine (Pack Engine,   │
│  │   │               Retrieval, Gate, Route)      │
│  │   ├── adapter: wrapper_capture, native_attach  │
│  │   ├── pipeline: runner + stage executor        │
│  │   ├── reconcile: Git/FS ↔ Ledger sync          │
│  │   └── storage: .seatloom/ YAML/JSONL IO        │
│  ├── src-tauri (Tauri binary)                     │
│  │   ├── commands: #[tauri::command] IPC handlers │
│  │   ├── pty: PTY manager for wrap/attach         │
│  │   └── state: app state management              │
│  └── src-cli (CLI binary, reuses seatloom-core)   │
│      └── clap-based CLI subcommands               │
├──────────────────────────────────────────────────┤
│  Shared Data Layer                                │
│  └── .seatloom/ (project-local directory)          │
│      ├── config/, ledger/, seats/, sessions/       │
│      ├── workitems/, artifacts/, handoffs/          │
│      └── pipelines/, views/                        │
└──────────────────────────────────────────────────┘
```

桌面应用和 CLI 共享 `seatloom-core` crate，确保数据操作一致。

---

## AD-004: Pack Engine — Worker/Supervisor Pack Types, Tiered Context, Deterministic Assembly

**决定**：MVP 阶段 Pack Engine（Data Engine 的核心子系统，原名 ContextPack Compiler）完全使用确定性规则，不调用任何 LLM。Pack Engine 负责为每次 session 启动或恢复组装结构化 Continuity Pack 文件。

### Worker Pack vs Supervisor Pack

| 维度 | Worker Continuity Pack (P0) | Supervisor Continuity Pack (P1) |
|------|-----------------------------|----------------------------------|
| 目标 | 执行任务的 agent seat | 跨任务协调的 seat（Lyra 等） |
| 内容范围 | 当前 WorkItem + Tier 0/1/2 | 所有活跃 WorkItem + 跨工作面板 |
| 默认预算 | 8192 tokens（可配置） | 32768 tokens（P1，可配置） |
| 落地文件 | `launch_pack.md` | `supervisor_pack.md`（P1） |

### Context Tiers（Worker Pack）

| Tier | 内容 | 是否必放 |
|------|------|---------|
| Tier 0 | SeatIdentity + ProjectRoleBind + 协作模式 | 是 |
| Tier 1 | 当前 WorkItem + AC、当前 branch + 最近 3 条 commit、最近 Checkpoint summary | 是 |
| Tier 2 | 最近 Handoff purpose/outcome、最近失败 test（≤3 条） | 是 |
| Optional | 最近 Artifact 摘要（≤5）、上一 session transcript 尾部（最后 3 轮）、文件引用列表 | 受预算裁剪 |

### Selector（选什么进包）

Tier 0–2 必放项来源：Seat identity.yaml + workitems/wi_xxx.yaml + git log + sessions/checkpoints + artifact meta.yaml。

选放项（按优先级排列，受预算裁剪）：

| 内容 | 来源 | 估算 token |
|------|------|-----------|
| 最近 Artifact 摘要（最多 5 个） | artifacts/ar_xxx/meta.yaml summary | ~500 |
| 上一个 session transcript 尾部（最后 3 轮对话） | raw/transcript.jsonl 尾部截取 | ~1000 |
| 相关文件引用列表（不内联内容） | Checkpoint artifacts_at_checkpoint | ~200 |

### Budgeter（裁剪到预算内）

- Worker 默认预算：8192 tokens（可配置，per-project）
- 计算：1 token ≈ 4 字符英文 / 2 字符中文
- 裁剪顺序：先砍 Optional（低优先级先移除），再压 Tier 2，极端情况仅保留 Tier 0 + Tier 1
- 超预算时 Verifier 记录 truncation 警告，不阻断输出

### Assembler（组装成 Markdown）

固定模板，不做改写：

```markdown
# LaunchPack for {seat_name} / {runtime}

## Your Task
- **WorkItem**: {wi_id} — {title}
- **Goal**: {goal}
- **Acceptance Criteria**:
  - {ac_1}
  - ...

## Current State
- **Branch**: {branch}
- **Recent commits**: {commit_list}

## Context from Previous Session
{checkpoint_summary}

## Handoff
- **From**: {from_seat}
- **Purpose**: {purpose}
- **Expected Outcome**: {expected_outcome}

## Known Issues
- {test_failures}

## Artifacts (reference, do not inline)
- {artifact_path} — {summary}

## Instructions
Continue working on this task. Read referenced files as needed.
Do not re-do work that is already completed.
```

### Verifier（完整性检查）

| 检查 | 失败时处理 |
|------|----------|
| WorkItem 存在且有 AC | 无 AC 则标记警告 |
| 引用文件路径存在 | 不存在则移除并标记 |
| 总 token 在预算内 | 超预算则继续裁剪 |
| Seat identity.yaml 存在（Tier 0） | 缺失则标记 tier0_missing，仅用 name fallback |

**LaunchPack 永远是落地的 `.md` 文件**，即使自动注入也先写文件再注入，保证可审计可回溯。

### 降级顺序

1. 原生 session resume（工具自带，如 Codex `--resume`）
2. Tier 0–2 Worker Pack from structured Checkpoint + Ledger
3. 最小恢复：仅 WorkItem + AC + 最近 Handoff

**P1 扩展**：引入 LLM 对超预算内容做有损压缩；Supervisor Pack 组装；定时 Checkpoint（每 15 分钟）。

---

## AD-005: Checkpoint — 两触发点，纯规则摘要

**决定**：MVP Checkpoint 在两个时机自动创建：

1. **Session 结束**（正常退出或 Ctrl+C）：最完整
2. **Artifact 产出**（检测到新文件/diff/test report）：增量

定时快照（每 15 分钟）作为 P1。

**Summary 生成方式**：从 transcript 尾部提取最后一轮 agent 回复，截断到 500 tokens。不调 LLM。transcript 不可用时 summary 为空，标记 `summary_quality: minimal`。

**恢复优先级**：原生 resume > SeatLoom checkpoint + LaunchPack > 最小恢复（仅 WorkItem + Artifact 引用）

---

## AD-006: MVP Pipeline — 前台同步执行

**决定**：MVP Pipeline 为前台同步执行。用户通过 GUI 点击触发或 CLI 显式调用，应用内实时显示进度。

- 每个 stage 默认超时 5 分钟，整体 15 分钟
- 最多重试 2 次，之后标记 input_required 进入 Inbox
- 文件锁防止重复触发
- 中断后记录已完成 stage 快照，下次可断点续跑

后台 daemon 模式作为 P1。

---

## AD-007: Reconciliation — 三触发点，无文件监听（已扩展：PostgreSQL 写入路径合同）

**决定**：MVP Reconciliation 在三个时机触发：

1. **应用启动时**：自动执行
2. **Pipeline 执行前**：自动执行
3. **用户手动触发**：GUI 按钮或 `seatloom reconcile`

MVP 不做文件监听（无 daemon），依赖启动时 + 手动 + Pipeline 前的三个触发点覆盖。

**PostgreSQL 写入路径合同（新增）**：

Reconcile 是 Repo markdown → PostgreSQL 权威存储的唯一有界导入操作。稳态写入路径定义如下：

| 层 | 角色 |
|----|------|
| Repo markdown 文件 | 外部创作输入面和可选导出面（不是权威存储） |
| `seatloom reconcile` | 有界导入/更新操作：扫描 → 解析 → upsert → 变更检测 |
| PostgreSQL `documents` 表 | 结构化权威存储：文档正文、全局头字段、sections、associations、version history |

**Reconcile 行为合同（MVP）**：

- 扫描范围：仅 `docs/*.md` 和 `docs/coordination/**/*.md`，且需能解析为已知 `template+subtype` 对
- 幂等性：相同文件内容不创建重复行/版本快照（通过 `body_digest` 比较）
- 变更检测：内容变更则增加 `revision`，记录 `document_versions` 快照
- 冲突处理：同一 `doc_id` 映射到不同 `file_path` 时记录为 conflict，不静默覆盖
- 解析失败：记录为 failed item，不静默丢弃
- 可观测性：每次运行写入 `reconcile_runs` + `reconcile_items`，扫描/插入/更新/未变更/失败/冲突计数均可查

**不做的事（MVP）**：

- 无文件监听 daemon
- 无后台同步
- 无 API 端点触发
- 无语义检索嵌入（P1）

---

*本文档记录所有已确认的架构决定。后续决定追加到本文件。*

---

## AD-008: Dual-Key Artifact Typing — template + subtype

**决定**：Artifact 的分类模型从平坦的 `ArtifactKind` 枚举改为双键模型（`template` + `subtype`），与协调文档系统的 T1-T7 分类法对齐（DOCUMENT_TEMPLATES.md §11.1）。

**双键模型**：
- `template: Option<ArtifactTemplate>` — 文件族（T1-T7），从 markdown header 字段提取
- `subtype: Option<String>` — 具体子类型，经 DOCUMENT_TEMPLATES.md §11.1 allow-list 验证
- `subtype_valid: Option<bool>` — `None`=待验证；`true`=有效；`false`=降级模式
- `system_kind: Option<SystemArtifactKind>` — 系统生成类型（DiffSummary, TestReport 等），不携带协调文档结构

**Template 枚举**：

| 枚举值 | 文件族 |
|--------|--------|
| T1AuthorityDoc | 权威文档（产品文档、ADR） |
| T2RoleProfile | 角色档案 |
| T3TaskPacket | 任务包（WorkItem packet, handoff） |
| T4Review | 评审文档 |
| T5Acceptance | 验收文档 |
| T6DailyMemory | 日常记忆、session summary |
| T7GovernanceDoc | 治理文档 |

**降级行为**：`template` 或 `subtype` 缺失/无效时，使用通用 markdown reader + UI 显示可见警告（不阻断展示）。

**理由**：v0.5 引入了丰富的协调文档系统（Inbox routing、Detail Pane layout、Gate automation），需要统一的类型键来驱动渲染逻辑和路由规则。系统生成 artifact（DiffSummary 等）不携带文档结构，用 `system_kind` 单独表达。

---

## AD-009: Seat Three-Layer Model and Delegation Overlay

**决定**：Seat 对象从平坦结构改为三层模型，分离全局身份、项目角色绑定和协作模板引用。委托操作通过 `SeatDelegation` overlay 实现，不修改原始所有权历史。

**三层模型**：

| 层 | 结构 | 作用域 | 存储路径 |
|----|------|--------|---------|
| Layer 1 | `SeatIdentity` | 全局 | `seats/<name>/identity.yaml` |
| Layer 2 | `ProjectRoleBind` | 每项目 | `seats/<name>/role-bindings/<project>.yaml` |
| Layer 3 | CollaborationTemplate 引用 | 每项目（引用） | `seats/<name>/role-bindings/<project>.yaml` 内的 `collaboration_template_ref` 字段 |

**SeatDelegation**：
- 字段：`id`, `issuer_seat_id`, `from_seat_id`（原始所有者）, `to_seat_id`（代理执行者）, `workitem_id?`, `scope_description`, `issued_at`, `expires_at?`, `status`
- 状态机：`Active → Closed | Expired`
- 不修改 `from_seat` 的历史记录；Timeline 展示格式：`{to_seat} acting for {from_seat} on {workitem_id}`
- Ledger 事件：`SeatDelegationIssued`, `SeatDelegationClosed`
- 存储：`delegations/del_xxxxxxxx.yaml`

**理由**：三层分离使 Seat 身份在项目间可复用（全局 identity）。委托是高频协作需求（US-P0-04），overlay 模式比修改所有权字段更能保证可审计性，原始 from_seat 的 Ledger 历史不受污染。

---

## AD-010: WorkItem Review Failure — Event-First Contract

**决定**：评审失败和范围重定义通过 Ledger 事件链记录，不新增持久 WorkItem 状态。WorkItem 经评审后通过现有生命周期重新入队。

**事件链（INT-05）**：

| 事件 | Payload | 触发时机 |
|------|---------|---------|
| `ReviewVerdictIssued` | `verdict`（reject/pass）、`reason`、`linked_evidence_artifact_id` | 评审人选择 Reject delivery / Return |
| `WorkItemRescoped` | `scope_change_summary`、`new_ac_refs` | 负责人修订 AC / 范围后重新发起 |

**生命周期重入路径**：
- `InReview → (ReviewVerdictIssued: reject) → Blocked`（工作项重新进入 Blocked 状态，等待修订）
- 负责人修订 AC → `WorkItemRescoped` 事件 → `Blocked → Ready → Active`（通过标准重发/再分配路径）

**不新增**：`Rejected` 和 `Rescoped` 不作为持久 `WorkItemStatus` 变体。评审决定的可溯源性通过 `ReviewVerdictIssued` 事件 payload 和 `linked_evidence_artifact_id` 保证。

**理由**：INT-05 明确要求 WorkItem 通过现有生命周期规则和新的 Handoff/分配重新进入活跃路径，而非停留在孤立的拒绝状态。事件优先模型保证了责任链可审计，同时不扩展 WorkItem 状态机的合约表面。

---

## AD-011: Data Engine Retrieval Order — Fixed Layer Sequence

**决定**：所有 SeatLoom 内的证据检索必须严格按固定顺序执行，每一层未满足时才进入下一层。

**层次顺序（不可逆乱）**：

| Layer | 方式 | P0/P1 |
|-------|------|-------|
| L1 | 结构化索引（template, subtype, id, status, workitem_id, object refs 精确匹配） | P0 |
| L2 | 全文检索（Artifact 正文 + Ledger payload 文本） | P0 |
| L3 | 语义检索（embedding 相似度） | P1 |
| L4 | LLM 解释（基于前三层已检索证据做总结） | P1 |

**P0 要求**：在不启用 L3/L4 的前提下，INT-13（精确证据搜索）必须能返回可用的结构化和全文检索命中（acceptance-spec E-02）。

**P0 存储权威（已更新）**：L1/L2 使用 **PostgreSQL** 作为持久化权威存储。

- PostgreSQL 是 SeatLoom 结构化项目真相的唯一权威存储引擎。
- `.seatloom/` 文件目录作为迁移输入、证据有效载荷、缓存和兼容层，不再是权威存储。
- 早期设计文件中提到的 SQLite FTS5 方向已被本更新正式取代。
- PostgreSQL 内置的 `tsvector` / `GIN` 索引满足 L2 全文检索的 P0 要求。
- L3 语义检索：PostgreSQL + `pgvector` 扩展（P1 时评估）。

| 层 | 后端 | 角色 |
|----|------|------|
| L1 结构化索引 | PostgreSQL（`documents`, `artifacts`, `workitems` 等精确字段查询） | 持久化权威存储 |
| L2 全文检索 | PostgreSQL `tsvector` / GIN 索引（文档正文 + Ledger payload） | 持久化权威存储 |
| 内存缓存 | 启动时从 PostgreSQL 重建的热投影 | 缓存加速，不写入 |
| L3 语义检索 | P1，PostgreSQL + pgvector（TBD） | — |
| L4 LLM 解释 | P1，基于 L1-L3 证据 | — |

**理由（更新）**：PostgreSQL 已作为 SeatLoom 的真实结构化存储实装并通过 commit-pinned 验证。它满足本地优先（Docker Compose 单机部署）、无外部服务依赖（嵌入式 Docker）、重启持久化三个 P0 要求，同时通过 `tsvector` 原生支持 L2 全文检索，并为 P1 语义检索提供 pgvector 升级路径。原 BLOCKER-001（检索存储后端决策）已由此更新关闭并重新对齐到 PostgreSQL 方向。

---

## AD-012: Interactive Prompt Architecture — Classification, Policy, Bounded Evidence, Audit

**决定**：包裹会话（wrapped session）中的交互式 prompt 作为一等架构状态建模，覆盖分类、策略、有界证据窗口、辅助预算和审计事件链。

### Prompt 状态扩展（SessionStatus）

`SessionStatus` 新增 `PromptBlocked` 变体，表示会话因等待交互式输入而暂停。与 `InputRequired`（SeatLoom 系统级请求）区分：`PromptBlocked` 特指 wrapped runtime 自身发出的 stdin 阻塞。

### Prompt 分类模型（PromptKind）

| 分类 | 语义 | 示例 |
|------|------|------|
| `Deterministic` | 固定选项，可确定性注入 | y/n、数字选项 |
| `WizardMenu` | 多步骤向导或菜单 | 安装选项序列 |
| `Freeform` | 任意文本输入 | 文件路径、名称 |
| `Sensitive` | 凭证、密钥、OTP、sudo | 密码、token |

### Prompt 策略模型（PromptPolicy）

| 策略 | 语义 | 处理路径 |
|------|------|---------|
| `AutoAllowed` | SeatLoom 可直接注入，无需确认 | 仅限 `Deterministic` 且策略配置允许 |
| `NeedsApproval` | 注入前需用户确认 | `WizardMenu` / `Freeform` |
| `HumanRequired` | 必须人工直接输入 | `Sensitive`（Supervisor assist 禁用） |

### 有界证据窗口（BoundedPromptWindow）

Prompt 检测和 Supervisor assist 必须基于有界窗口（最后 10-20 行终端输出 + Tier 0-2 结构化上下文），不得重读完整终端历史。

### 用户动作

| 动作 | 适用场景 | 语义 |
|------|---------|------|
| `Approve` | `AutoAllowed` / `NeedsApproval` | 确认并注入 |
| `HumanTakeover` | 任意场景 | 用户直接接管终端输入 |
| `SupervisorAssist` | 非 `Sensitive`，预算内 | Supervisor 基于有界窗口提议输入，用户确认 |
| `Stop` | 任意场景 | 停止会话，记录 prompt 事件 |

### Assist 预算约束

| 约束 | 值（默认，可配） | 超限处理 |
|------|-----------------|---------|
| `max_assist_steps` | 5 步 | 超限强制 HumanTakeover |
| `max_assist_tokens` | 2048 tokens | 超限强制 HumanTakeover |
| Sensitive prompt | 禁用 SupervisorAssist | 永远 HumanRequired |

下一步提议（expected_next_prompt）不匹配时，强制 HumanTakeover 并记录偏差事件。

### 审计事件

| 事件 | Payload 关键字段 |
|------|----------------|
| `PromptDetected` | `kind`, `policy`, `bounded_window_ref`（指向有界快照路径）, `risk` |
| `PromptInputInjected` | `operator`（`user` / `supervisor`）, `scope`（`once` / `session` / `project`）, `result`, `assist_steps_used?`, `assist_tokens_used?` |

**理由**：INT-16 / US-P0-11 / UX-12 将交互式 prompt 处理定义为 P0 功能。无架构建模会导致：实现时将 prompt 处理散落在 adapter 层、无审计事件、Supervisor assist 无预算约束、sensitive prompt 保护缺失。

---

## AD-013: Supervisor 两层上下文模型 — Global vs Project Context

**决定**：Supervisor 在单个 Person 的使用会话中只处于一个 context 下工作，分为 `Global` 和 `Project` 两种互斥模式。Global 提供跨项目摘要视图，Project 提供单项目细节视图。切换是显式用户动作，同一 Supervisor 会话不并发承载两种 context。

> 本决定对应 `AEGIS-2026-04-30-pending-changes-register` 中 **chan-03** 的 AD-AEGIS-01 占位，正式编号落为 AD-013。

### Person-Supervisor 绑定（Q4 对齐）

- 一个 Person 对应一个 Supervisor 实例（1:1）
- Person 不是实体（不引入 Person struct）
- Supervisor 的 `currentContextMode` 是运行时状态，不是数据模型字段
- Person-Supervisor 绑定关系不跨 Supervisor 会话（但 localStorage 可以持久化上一次 context）

### Context mode 定义

| Mode | 数据范围 | 决策目的 | 展示形态 |
|------|---------|---------|---------|
| `global` | 所有 project 的聚合 summary（不含具体 workitems/artifacts 详情） | Supervisor 要做"现在哪个 project 需要我关注"的决策 | GlobalDashboard 组件（新增） |
| `project` | 单个 `activeProjectId` 的完整 `projectData` | Supervisor 要做"这个 project 内部的 gate/routing/approve"决策 | ProjectDashboard 组件（已存在） |

**互斥规则**：同一时刻 `currentContextMode` 取 `global` 或 `project` 中恰好一个。

- Project mode 必须伴随**非空** `activeProjectId`
- Global mode 下 `activeProjectId` **必须为 `null`**（不得用于 UI 高亮或其他目的；高亮由 Contact 列表自身的 hover/selection 机制处理）

### 切换语义

| 动作 | 前状态 | 后状态 | 实现锚点 |
|------|-------|-------|---------|
| 初次打开 | 无 | `global`（默认）或 localStorage 保存的上一次 context | SupervisorPanel `useState` 初始化 |
| 从 Global 进入某 Project | `global`, `activeProjectId = null` | `project`, `activeProjectId = <selected>` | GlobalDashboard 中 project 行的点击 handler |
| 从 Project 返回 Global | `project`, `activeProjectId = <X>` | `global`, `activeProjectId = null`（**清空**） | SupervisorPanel 顶部的 "全局" 切换入口 |
| 切换不同 Project | `project`, `activeProjectId = A` | `project`, `activeProjectId = B` | Contact 列表中的 project channel 点击（已有机制） |

切换是**显式用户动作**。不自动推断 context（避免 Supervisor 在用户不知情时跳出当前 project）。

### 数据隔离规则

- Global 视图**不得**渲染任何单一 project 的 workitems/artifacts/handoffs 详情（避免"看似 global 实则 project 碎片拼接"）
- Project 视图**不得**在左侧 Contact 列表之外展示其他 project 的数据（避免跨 project context 污染）
- Contact 列表（左侧）跨 context 保持不变：它是**导航通道**，不是 context 内数据

### 实现锚点（v2 前端）

| 组件 | 职责 |
|------|------|
| `panel/SupervisorPanel.tsx` | 承载 `currentContextMode` state 和切换入口；根据 mode 决定渲染 GlobalDashboard 或 ProjectDashboard |
| `dashboard/GlobalDashboard.tsx`（新增） | Global context 视图：所有 project 的健康度卡片、阻塞汇总、最近活动 |
| `dashboard/ProjectDashboard.tsx`（已存在） | Project context 视图：保持 Phase 1/2 已交付的 tab 和 section 结构不变 |
| `mock-data.ts` | 新增 `MOCK_GLOBAL_SUMMARY`（所有 project 的聚合摘要） |
| `types.ts` | 新增 `SupervisorContextMode = 'global' \| 'project'` |

### 零回归约束

- Project mode 下的所有 Phase 1/2 已交付视图（Inbox / WorkItems / Artifacts / Overview 各 section）保持行为不变
- `projectId` prop 在 Project mode 下的来源、传递、默认值均不变
- 现有 localStorage key 不改名（新增 key 可以）

**理由**：

1. **避免跨 project context 污染** — Mr. Zhang 在 Q4-Q5 对齐中明确要求 Supervisor 在单 project 内不混入其他 project 的数据（chan-03 核心问题）。
2. **支持全局健康监控** — Aegis/Supervisor 的实际工作流需要先看"哪个 project 需要关注"才能决定进入哪个 project 细节（`supervisor-im-as-l1-insight.md`）。
3. **与 L1 Supervisor IM 定位一致（chan-06）** — Supervisor 作为 L1 高频入口，必须提供"全局→细节"的双层跳转，而不是强迫用户从一个 project 跳到另一个 project 才能看到全局。
4. **用户决策路径最短化** — 对比需要打开多个 project 频道才能拼凑全局视图的旧模式，两层 context 让"看全局"和"做项目级决策"各自成为一个单独的、命名明确的 Supervisor 状态。

### 修订历史

| 日期 | 版本 | 变更 | 触发 |
|------|------|------|------|
| 2026-05-07 | v1 | 初版（chan-03） | Aegis 设计，Lyra 验收 |
| 2026-05-07 | v1.1 | 互斥规则收紧：global mode ⇒ activeProjectId 必须为 null | Lyra review 反馈 |
| 2026-05-07 | v2 | 新增 §7 viewMode 正交状态（chan-09） | Mr. Zhang + Lyra design discussion |

### §7 viewMode 正交状态（v2，chan-09）

**问题陈述**：v1 把 `contextMode` 设计成同时承担两个职责 — (a) dashboard 数据隔离、(b) 右侧 pane 路由。这两个职责在 dashboard-only 场景下重合，但当用户在右侧 pane 与某个 seat **chat** 时出现歧义：dashboard 数据隔离仍然有意义，但右侧 pane 既不是 GlobalDashboard 也不是 ProjectDashboard，contextMode 不再能完整描述 UI 状态。

**决定**：引入与 `contextMode` 正交的 `viewMode` 状态。

```typescript
type SupervisorContextMode = 'global' | 'project';   // 不变（§6 不变量保留）
type SupervisorViewMode    = 'dashboard' | 'chat';   // 新增
```

#### 职责重新分配

| 字段 | 职责 |
|------|------|
| `contextMode` | dashboard 数据隔离（决定 GlobalDashboard 还是 ProjectDashboard 的数据范围） |
| `activeProjectId` | dashboard 互斥不变量的载体（与 contextMode 配对） |
| `viewMode` | 右侧 pane 路由（决定渲染 dashboard 还是 chat） |
| `activeContactId` | chat 当前选中的 contact（仅在 viewMode='chat' 下有渲染意义） |

#### 右侧 pane 决策表

| `viewMode` | 子条件 | 右侧 pane |
|------------|-------|-----------|
| `dashboard` | `contextMode === 'global'` | GlobalDashboard |
| `dashboard` | `contextMode === 'project'` | ProjectDashboard for `activeProjectId` |
| `chat` | `activeContact.type === 'supervisor'` | Chat（Supervisor） |
| `chat` | `activeContact.type === 'seat'` | Chat（Seat） |
| `chat` | `activeContact.type === 'project-channel'` | ProjectDashboard for `activeContact.projectId`（project channel 是 dashboard 的 alias，viewMode 一律走 dashboard 视觉） |

> 注：实际实现可以把 project channel 点击映射为「viewMode=dashboard + enterProject(channel.projectId)」，从而让该行不存在；两种实现等价。NIMBUS 包推荐前者（路由分支收敛）。

#### 面包屑决策表

| `viewMode` | 子条件 | 面包屑 |
|------------|-------|--------|
| `dashboard` | `contextMode === 'global'` | `全局` |
| `dashboard` | `contextMode === 'project'` | `全局 › <projectName(activeProjectId)>` |
| `chat` | `activeContact.type === 'supervisor'` | `全局` |
| `chat` | `activeContact.type === 'seat'` | `全局 › <projectName(seat.projectId)> › <seatName>` |

**关键**：chat-with-seat 模式下面包屑显示的 `projectName` 来自 **seat contact 的 `projectId` 字段**，与 `activeProjectId` 解耦。这意味着用户可以处于「dashboard 看 p-1，chat 与 p-2 的某 seat」的分叉态——这是 v2 的合法状态，**不强制同步**。

#### 触发器表

| 用户动作 | 副作用 |
|---------|-------|
| 点面包屑的 `全局` | `viewMode='dashboard'` + `enterGlobal()` |
| 点面包屑中间层 `<projectName>` | `viewMode='dashboard'` + `enterProject(projectId)` + `setActiveContactId(<该 project 的 channel contact>)`（同步 contact 高亮，避免 contact 列表 vs 右侧 pane 视觉不一致） |
| 点 contact 列表中的 project channel | `viewMode='dashboard'` + `enterProject(contact.projectId)` + `setActiveContactId(contact.id)` |
| 点 contact 列表中的 seat | `viewMode='chat'` + `setActiveContactId(seat.id)`；**不动** `contextMode` 与 `activeProjectId`（与 v1 的 switchContact 自动同步行为不同） |
| 点 contact 列表中的 supervisor | `viewMode='chat'` + `setActiveContactId('supervisor')`；**不动** `contextMode` |

#### 不变量

**v1 不变量保留（不放松）**：
- `contextMode === 'global'` ⇔ `activeProjectId === null`
- 仅 `enterGlobal()` 和 `enterProject(projectId)` 可修改 `contextMode` / `activeProjectId`
- Person-Supervisor 1:1 绑定
- 切换是显式用户动作
- 数据隔离规则：Global 不渲染单 project 详情，Project 不渲染其他 project 数据

**v2 新增不变量**：
- `viewMode` 与 `contextMode` 正交，互不蕴含
- `viewMode` 由 contact 列表点击与面包屑点击驱动；不由 dashboard 内部交互驱动
- chat-with-seat 模式下面包屑的 `projectName` 来源是 `activeContact.projectId`，**不**是 `activeProjectId`

#### 视觉 affordance

面包屑中间层 `<projectName>` 在 v2 下变成可点击 link，需要在视觉上明确：
- 默认态：与 v1 中间层文本同色（`var(--sl-text-secondary)` 或 `var(--sl-text-primary)`）
- hover 态：颜色切到 `var(--sl-brand)`，cursor: pointer
- 与 `全局` 入口的视觉一致（同为 button 样式，无下划线）

NIMBUS 包负责实现细节。

#### localStorage key 清单（含本次新增）

| key | 写入时机 | 读取规则 | stale fallback |
|-----|---------|---------|---------------|
| `seatloom.supervisor.contextMode` | enterGlobal / enterProject | 缺失或非法 → `'global'` | — |
| `seatloom.supervisor.activeProjectId` | enterGlobal / enterProject | 缺失 → null；非法 projectId → 降级回 global mode | §D rule (chan-03) |
| `seatloom.supervisor.viewMode`（新） | viewMode 切换时 | 缺失或非法 → `'dashboard'` | 同左 |
| `sl-supervisor-active-contact`（已有，不改名） | switchContact | 既有 | 既有 |
| `sl-supervisor-open` / `sl-supervisor-pos` / `sl-supervisor-size` / `sl-supervisor-draft-*`（已有） | 既有 | 既有 | 既有 |

不得改名既有 key。

#### 零回归约束（v2 扩展）

- chan-03 已验收的 8 个 smoke 场景全部兼容
- v2 新增 6 个 smoke 场景（chat 面包屑三级、chat 面包屑一级、面包屑中间层点击、chat→面包屑全局回退、跨 project 分叉态、回归 chan-03 全集）
- v1 的 `switchContact` 在点击 seat 时自动同步 contextMode 的行为**被废弃**（v2 改为不动 contextMode）— 这是 v2 唯一的破坏性行为变更，必须通过 NIMBUS 包显式实现并通过 verify

#### 理由

1. **职责单一**：把 dashboard 数据隔离与右侧 pane 路由拆开两个状态字段，每个字段含义单一，避免 v1 的 contextMode 在 chat 模式下的歧义
2. **保留分叉态作为合法 UI 状态**：用户可能希望"dashboard 维持对 p-1 的关注，同时与 p-2 的 seat 沟通"——v2 把这种状态显式化、合法化，而不是强制同步
3. **chan-06 一致性**：Supervisor 作为 L1 高频入口，面包屑必须如实反映"我现在在跟谁说话 / 看谁的数据"，三级面包屑提供这种透明度
4. **与 contact 列表视觉一致**：通过中间层点击同步 `activeContactId`，避免 contact 列表高亮与右侧 pane 内容错位

---

*本文档记录所有已确认的架构决定。后续决定追加到本文件。*
