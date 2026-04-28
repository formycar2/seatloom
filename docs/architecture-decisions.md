# SeatLoom Architecture Decisions

| 项目 | 内容 |
|------|------|
| 文档 | Architecture Decisions v1.0 |
| 状态 | Approved |
| 更新时间 | 2026-04-27 |
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
│  │   ├── context: ContextPack compiler            │
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

## AD-004: ContextPack — MVP 纯规则，不调 LLM

**决定**：MVP 阶段 ContextPack Compiler 完全使用确定性规则，不调用任何 LLM。

**流程**：Selector → Budgeter → Assembler → Verifier，每一步确定性。

### Selector（选什么进包）

必放项（硬编码优先级）：

| 内容 | 来源 | 估算 token |
|------|------|-----------|
| WorkItem title + goal + AC | workitems/wi_xxx.yaml | ~200-500 |
| 当前 branch + 最近 3 条 commit message | git log | ~100 |
| 最近 Handoff 的 purpose + expected_outcome | handoffs/ho_xxx.yaml | ~200 |
| 最近 Checkpoint 的 summary | sessions/ses_xxx/checkpoints/ | ~500 |
| 最近失败的 test（最多 3 条） | 最近 test_report artifact | ~300 |

选放项（按优先级排列，受预算裁剪）：

| 内容 | 来源 | 估算 token |
|------|------|-----------|
| 最近 Artifact 摘要（最多 5 个） | artifacts/ar_xxx/meta.yaml summary | ~500 |
| 上一个 session transcript 尾部（最后 3 轮对话） | raw/transcript.jsonl 尾部截取 | ~1000 |
| 相关文件引用列表（不内联内容） | Checkpoint artifacts_at_checkpoint | ~200 |

### Budgeter（裁剪到预算内）

- 默认预算：8192 tokens（可配置）
- 计算：1 token ≈ 4 字符英文 / 2 字符中文
- 裁剪顺序：先砍选放（低优先级先移除），再压必放（截断 transcript → 截断 summary → 截断 artifact 摘要）
- 极端情况：只保留 WorkItem + branch + 最近 Handoff

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

**LaunchPack 永远是落地的 `.md` 文件**，即使自动注入也先写文件再注入，保证可审计可回溯。

**P1 扩展**：引入 LLM 对超预算内容做有损压缩。

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

## AD-007: Reconciliation — 三触发点，无文件监听

**决定**：MVP Reconciliation 在三个时机触发：

1. **应用启动时**：自动执行
2. **Pipeline 执行前**：自动执行
3. **用户手动触发**：GUI 按钮或 `seatloom reconcile`

MVP 不做文件监听（无 daemon），依赖启动时 + 手动 + Pipeline 前的三个触发点覆盖。

---

*本文档记录所有已确认的架构决定。后续决定追加到本文件。*
