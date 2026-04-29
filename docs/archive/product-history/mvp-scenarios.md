# SeatLoom MVP 用户场景规格

| 项目 | 内容 |
|------|------|
| 文档 | MVP User Scenarios v2.0 |
| 状态 | Approved |
| 更新时间 | 2026-04-27 |
| 配套文档 | PRD v0.3、MVP Validation Plan v1.0、Architecture Decisions v1.0 |
| 审批人 | 张小龙 |

---

Reference-only during v0.5 consolidation. Start from `docs/PRODUCT_TRUTH.md` for current product work.

## 文档定位

本文档定义 MVP 首次交付后，用户视角支持的全部场景。

SeatLoom 的主交互面是**桌面 GUI 应用**（Tauri 2 + React），辅以 CLI 子命令用于脚本和自动化。本文档以桌面 GUI 为主描述交互，CLI 等价命令在附录中列出。

- PRD v0.3 回答"做什么、为什么做、对象模型是什么"
- 本文档回答"用户拿到 MVP 后，具体能做什么、每一步看到什么"
- Architecture Decisions 回答"用什么技术、怎么分层、关键设计怎么做"
- MVP Validation Plan 回答"怎么验证做没做到"

Lyra 验收、Nimbus 实现、Mira 设计、Flux 测试均以本文档为用户行为基准。

---

## 应用布局

```
┌─────────────────────────────────────────────────────────────┐
│  SeatLoom                                        ─ □ ✕     │
├────────────┬────────────────────────────┬───────────────────┤
│  SIDEBAR   │  MAIN PANEL                │  DETAIL PANE      │
│            │                            │  (click to open)  │
│  ▸ Inbox(3)│  [Inbox] [Timeline] [Work] │                   │
│            │                            │  HO-003           │
│  SEATS     │  !! handoff.pending  flux  │  From: lyra       │
│  ● lyra    │     WI-012 "fix callback"  │  To: nimbus       │
│  ● nimbus  │  !! review.requested nimbus│  Purpose: ...     │
│  ○ flux    │     WI-011 "OAuth ready"   │  Expected: ...    │
│            │  !  input.required  nimbus  │  Artifacts:       │
│  SESSIONS  │     SES-007 "NextAuth v?"  │   AR-010 brief    │
│  SES-004 ● │  !  workitem.blocked       │   AR-011 AC       │
│  SES-005 ● │     WI-013 "blocked"       │                   │
│  SES-007 ○ │  ·  drift.detected         │  [Accept] [Return]│
│            │     WI-010 "unmerged"      │  [View Artifacts] │
│  WORKITEMS │                            │                   │
│  WI-012 ● │                            │                   │
│  WI-011 ◐ │                            │                   │
│  WI-013 ○ │                            │                   │
├────────────┴────────────────────────────┴───────────────────┤
│  TERMINAL PANEL (toggle with Ctrl+`)                        │
│  nimbus/codex (SES-005) ● running                           │
│  $ ...                                                      │
└─────────────────────────────────────────────────────────────┘
```

### 布局说明

| 区域 | 内容 | 交互 |
|------|------|------|
| **Sidebar** | Inbox badge、Seat 列表、Session 列表、WorkItem 列表 | 点选切换焦点，展开/折叠分组 |
| **Main Panel** | Tab 切换视图：Inbox / Timeline / WorkItems | Tab 键或点击切换，列表可滚动/翻页 |
| **Detail Pane** | 选中对象的详细信息 + 操作按钮 | 点击 Main Panel 条目打开，Esc 关闭 |
| **Terminal Panel** | 内嵌终端，显示 wrap/attach 的 agent session | Ctrl+\` 切换显示/隐藏，可调高度 |

### 快捷键

| 快捷键 | 操作 |
|--------|------|
| `Ctrl+1/2/3` | 切换 Main Panel Tab（Inbox/Timeline/WorkItems） |
| `Ctrl+`` ` | 切换 Terminal Panel 显示/隐藏 |
| `↑/↓` | 在列表中上下移动选中项 |
| `Enter` | 打开选中项的 Detail Pane |
| `Esc` | 关闭 Detail Pane / 返回上一层 |
| `Tab/Shift+Tab` | 在 Sidebar → Main Panel → Detail Pane 之间切换焦点 |
| `PageUp/PageDown` | 翻页浏览长列表 |
| `Ctrl+N` | 快速创建 WorkItem |
| `Ctrl+H` | 创建 Handoff |
| `Ctrl+R` | 手动 Reconcile |

---

## 场景总览

| # | 场景 | 对应铲子 | MVP 阶段 |
|---|------|---------|---------|
| 1 | 初始化项目 | — | Phase 1 |
| 2 | 定义 Seat | — | Phase 1 |
| 3 | 接住 agent session | Attach + Capture | Phase 0-1 |
| 4 | 看 Timeline | Replay | Phase 1 |
| 5 | 看 Inbox | Replay | Phase 1 |
| 6 | 创建 WorkItem | — | Phase 1 |
| 7 | 创建 Handoff | Rehydrate | Phase 3 |
| 8 | 跨工具切换 | Rehydrate | Phase 2 |
| 9 | Session 中断恢复 | Rehydrate | Phase 2 |
| 10 | 决策回溯 | Replay | Phase 4 |
| 11 | 早晨收束 | Replay | Phase 1 |
| 12 | 触发 Pipeline | Govern (轻量) | Phase 3 |
| 13 | Reconcile | Replay | Phase 1 |

---

## 场景 1：初始化项目

**触发**：用户在桌面应用中点击 "Open Project" → 选择一个 Git 仓库 → 如果无 `.seatloom/`，显示初始化向导。

**用户看到什么**：

初始化向导弹窗：
- "Initialize SeatLoom for this project?"
- 显示项目路径和 Git 状态
- [Initialize] [Cancel]

点击 Initialize 后：
- `.seatloom/` 目录创建完成
- 应用跳转到空白的 Inbox 视图，侧边栏显示 "No seats yet. Add one?"

**CLI 等价**：`seatloom init`

**闭合条件**：`.seatloom/config/project.yaml` 存在且 schema 合法。

---

## 场景 2：定义 Seat

**触发**：侧边栏 SEATS 区域点击 "+" 按钮，或快捷键打开添加对话框。

**用户看到什么**：

添加 Seat 对话框：
- Name: [lyra]
- Role: [product_owner ▾] (下拉选择：product_owner / architect / verifier / designer / custom)
- [Create] [Cancel]

创建后侧边栏 SEATS 区域立即出现新 Seat，带状态指示灯（灰色 = 无活跃 session）。

**CLI 等价**：`seatloom seat add lyra --role product_owner`

**闭合条件**：`.seatloom/seats/<name>/profile.yaml` 落地，侧边栏正确显示。

---

## 场景 3：接住 agent session（Mode 0 核心入口）

### 3.1 概念区分

| 操作 | 语义 | 入口 |
|------|------|------|
| **Attach** | 附着到已经在跑的 agent session | 应用内按钮或 CLI |
| **Wrap** | 由 SeatLoom 包裹启动新 agent 进程 | 应用内 Terminal Panel 或 CLI |

MVP 不做 `seatloom run "task"`（Mode 1），后置。

### 3.2 Attach 交互

侧边栏 SESSIONS 区域点击 "Attach" 按钮：

弹窗列出系统中正在运行的 agent 进程：
```
Attach to Running Session

  Runtime           PID     Directory                Started
  ● Claude Code     42831   ~/projects/seatloom      10m ago
  ● Codex           42950   ~/projects/other         2h ago
  ○ Cursor CLI      43100   ~/projects/seatloom      5m ago

  Assign to seat: [nimbus ▾]

  [Attach] [Cancel]
```

Attach 后：
- 侧边栏 SESSIONS 出现新条目，绿色指示灯
- Terminal Panel 可切换到该 session 查看实时输出
- 后台自动捕获 stdout/stderr/transcript

### 3.3 Wrap 交互

Terminal Panel 底部有 "New Session" 按钮：

弹窗：
```
Launch New Session

  Seat: [nimbus ▾]
  Runtime: [claude ▾] (claude / codex / cursor / gemini / custom)
  Working directory: [~/projects/seatloom]

  [Launch] [Cancel]
```

Launch 后：
- Terminal Panel 打开新标签页，agent 在其中运行
- 用户直接在 Terminal Panel 中与 agent 交互，体验与原生终端一致
- Ctrl+C 退出时自动创建 Checkpoint

### 3.4 别名模式（终端用户）

```bash
alias claude="seatloom wrap --seat nimbus claude"
```

**CLI 等价**：`seatloom attach --seat nimbus --runtime claude_code` / `seatloom wrap --seat nimbus claude`

**闭合条件**：Session 目录创建，stdout/stderr 实时捕获，Ledger 写入 `session.started`，agent 交互无劣化。

---

## 场景 4：看 Timeline

**触发**：Main Panel 点击 "Timeline" Tab，或 `Ctrl+2`。

**用户看到什么**：

时间线视图，每行一条事件，按时间正序排列：

```
Timeline                              [Filters ▾] [Search]

  14:32  nimbus   session.started    SES-004 claude_code
  14:35  nimbus   artifact.created   AR-012 design_note "OAuth flow design"
  14:51  lyra     handoff.sent       HO-003 lyra → nimbus  WI-012
  15:10  nimbus   handoff.accepted   HO-003
  15:12  nimbus   session.started    SES-005 codex (rehydrated from SES-004)
  16:40  nimbus   artifact.created   AR-015 diff_summary "+OAuth callback"
  16:41  ──auto── pipeline.started   verification_loop WI-012
  16:52  flux     artifact.created   AR-016 test_report "3/4 passed"
  16:52  ──auto── pipeline.completed verification_loop FAIL
  16:53  ──auto── handoff.sent       HO-004 flux → nimbus "fix callback"
```

**每行 5 列**：时间 / actor / event_type / 对象引用 / 摘要

**过滤器**（点击 "Filters" 下拉或侧边栏联动）：

| 过滤维度 | 方式 |
|---------|------|
| 按 Seat | 侧边栏点击某个 Seat，Timeline 自动过滤 |
| 按 WorkItem | 侧边栏点击某个 WorkItem，Timeline 自动过滤 |
| 按事件类型 | Filter 下拉多选：session / handoff / artifact / pipeline |
| 按时间范围 | Filter 下拉：last 1h / 6h / 24h / 7d / custom |

**点击某条事件**：右侧 Detail Pane 展开该事件的详细信息（verbose payload、evidence_refs、关联对象链接）。

**默认行为**：最近 24 小时，最多 200 条，实时追加新事件。

**CLI 等价**：`seatloom timeline [--seat] [--workitem] [--since] [--type] [--limit] [--verbose]`

**闭合条件**：所有被 capture 的事件出现在 Timeline 中，过滤准确，点选能打开详情。

---

## 场景 5：看 Inbox

**触发**：Main Panel 点击 "Inbox" Tab，或 `Ctrl+1`。Inbox 是应用启动后的默认视图。

**设计原则**：Inbox 是行动队列——只放需要人做决定或动手的事。

**用户看到什么**：

```
Inbox (5)

  ‼  Handoff pending      flux → you     WI-012  "fix: callback TypeError"
  ‼  Review requested     nimbus         WI-011  "OAuth implementation ready"
  !  Input required       nimbus SES-007          "NextAuth v4 还是 v5?"
  !  WorkItem blocked     ──auto── WI-013         "blocked by WI-012"
  ·  Drift detected       ──auto── WI-010         "done 但 branch 未 merge"
```

### 条目来源（4 类）

| 来源 | 进入 Inbox 的条件 |
|------|------------------|
| Handoff | 状态为 `sent` 且 `to_ref` 是人类或待人工分派 |
| Session input_required | agent 明确请求人工输入 |
| WorkItem blocked/drifted | 自动检测到阻塞或状态漂移 |
| Pipeline 失败 | 达到重试上限，标记为 `input_required` |

### 优先级（自动计算，显示为图标）

| 图标 | 级别 | 规则 |
|------|------|------|
| ‼ | Critical/High | Handoff 待处理、review 待批、Pipeline 失败影响 P0 |
| ! | Normal | input_required、blocked |
| · | Low | drift detected、info-level |

### 交互

点击某条 Inbox 条目 → Detail Pane 展开：

```
Handoff HO-004

  From: flux → To: you
  WorkItem: WI-012 "实现 OAuth 登录"
  Purpose: 修复 OAuth callback TypeError
  Expected Outcome: callback.test.ts 全部通过

  Artifacts:
    📄 AR-016  test_report    "3 passed, 1 failed"
    📄 AR-015  diff_summary   "+OAuth callback handler"

  ┌─────────────────────────────────┐
  │ [Accept]  [Return]  [Dismiss]   │
  └─────────────────────────────────┘
```

点击 Accept → Handoff 状态更新，从 Inbox 中移除，Ledger 写入事件。
点击 Artifact 链接 → Detail Pane 切换到 Artifact 详情。

**CLI 等价**：`seatloom inbox`

**闭合条件**：所有符合条件的事项出现在 Inbox，优先级排序正确，操作正确更新状态和 Ledger。

---

## 场景 6：创建 WorkItem

**触发**：`Ctrl+N`，或侧边栏 WORKITEMS 区域 "+" 按钮。

### 快速创建

弹窗只显示 title 输入框：
```
New WorkItem
  Title: [实现 OAuth 登录              ]
  [Create as Draft]  [More Options...]  [Cancel]
```

点击 "Create as Draft" → WI 创建为 draft 状态，出现在侧边栏。

### 完整创建

点击 "More Options..." 展开完整表单：
```
New WorkItem

  Title:    [实现 OAuth 登录                              ]
  Goal:     [支持 Google/GitHub OAuth，使用 NextAuth.js    ]

  Acceptance Criteria:
    [+ Add criterion]
    ☐ 用户可使用 Google 账号登录
    ☐ 用户可使用 GitHub 账号登录
    ☐ 登录后跳转到 /dashboard

  Assign to: [nimbus ▾]
  Priority:  [● high  ○ medium  ○ low]

  [Create]  [Cancel]
```

### 字段策略：创建门槛低、推进门槛高

| 字段 | 创建时 | → ready | → active |
|------|--------|---------|----------|
| title | **必填** | 必填 | 必填 |
| goal | 可选 | 建议 | 建议 |
| acceptance_criteria | 可选 | **必填** (gate) | 必填 |
| owner_seat_id | 可选 | 可选 | **必填** (gate) |
| priority | 默认 medium | — | — |

Gate 违反时在 Detail Pane 中显示红色提示：`⚠ Cannot activate: acceptance criteria required`

**CLI 等价**：`seatloom workitem create "title"` / `seatloom workitem create -i`

**闭合条件**：WorkItem 文件落地，Gate 校验正确拦截，侧边栏实时更新。

---

## 场景 7：创建 Handoff

**触发**：`Ctrl+H`，或 Detail Pane 中 WorkItem 详情里的 "Create Handoff" 按钮。

**用户看到什么**：

```
New Handoff

  From:     [lyra ▾]
  To:       [nimbus ▾]
  WorkItem: [WI-012 实现 OAuth 登录 ▾]

  Purpose:
  [需求确认完成，请开始实现 OAuth 登录           ]

  Expected Outcome:
  [可运行的 OAuth 登录功能 + 通过测试            ]

  Attach Artifacts:
    ☑ AR-010  brief              "OAuth 需求简报"
    ☑ AR-011  acceptance_criteria "3 条 AC"
    ☐ AR-012  design_note        "OAuth flow design"

  Require receipt: [☑]

  [Save as Draft]  [Send Now]  [Cancel]
```

### 必填校验

| 字段 | 校验 |
|------|------|
| from_ref | 必填，已注册 Seat 或 human |
| to_ref | 必填，已注册 Seat 或 human |
| workitem | 必填 |
| purpose | 必填 |
| expected_outcome | 必填 |

缺少字段时，对应输入框红色高亮 + 提示文字。

### 自动草稿（MVP 边界）

`requirement_handoff` Pipeline 生成的自动草稿是**纯规则拼装，不调 LLM**：
- 从 WorkItem 取 title + goal + AC
- 从最近 Session 取最后一个 checkpoint 摘要
- 从最近 Artifact 取引用列表
- 拼装成 Handoff 草稿（`drafted`），出现在 Inbox 等用户确认后发送

素材不够时草稿标记 `incomplete`，Inbox 中提示用户补充。

**CLI 等价**：`seatloom handoff create --from lyra --to nimbus --workitem WI-012 ...`

**闭合条件**：Handoff 文件落地，校验拦截，发送后接收方 Inbox 出现，Receipt 机制正确。

---

## 场景 8：跨工具切换（Rehydrate）

**触发**：在 Session 详情的 Detail Pane 中点击 "Switch Runtime" 按钮，或右键 Session → "Launch in different runtime"。

**用户看到什么**：

Step 1 — 选择目标 runtime：
```
Switch Runtime for Nimbus

  Current: SES-004 (claude_code)
  New runtime: [codex ▾]
  Working directory: [~/projects/seatloom]

  [Build LaunchPack & Launch]  [Cancel]
```

Step 2 — LaunchPack 预览（点击 "Build LaunchPack & Launch" 后）：
```
LaunchPack Preview (6200 tokens)

  ✓ WorkItem: WI-012 "实现 OAuth 登录" (3 AC)
  ✓ From session: SES-004 (claude_code) checkpoint
  ✓ Artifacts: AR-012 (design_note), AR-010 (brief)
  ✓ Branch: feature/oauth-login
  ✓ Recent failure: auth/callback.test.ts TypeError

  LaunchPack saved to: .seatloom/sessions/ses_005/launch_pack.md

  [Launch Now]  [Edit LaunchPack]  [Cancel]
```

Step 3 — Launch 后：
- Terminal Panel 打开新标签页，Codex 启动并自动接收 LaunchPack
- 侧边栏 SESSIONS 新增 SES-005，绿色指示灯
- Timeline 记录 `session.started` 事件（含 `rehydrated from SES-004`）

### LaunchPack 注入策略：三级降级

| 级别 | 方式 | 条件 |
|------|------|------|
| **L1** | wrap 启动时自动注入 stdin | 应用内 Terminal Panel 启动 |
| **L2** | 文件 + 短指令复制到剪贴板 | 用户在外部终端手动启动 agent |
| **L3** | LaunchPack 内容复制到剪贴板 | 上述都不可用 |

LaunchPack 永远先写为 `.md` 文件，保证可审计可回溯。

**CLI 等价**：`seatloom session launch --seat nimbus --runtime codex --from-session SES-004`

**闭合条件**：LaunchPack 正确生成（含必放项），新 session 第一个响应理解上下文，Ledger 记录完整。

---

## 场景 9：Session 中断恢复

**触发**：侧边栏某个 Session 显示为红色/灰色（中断/失联）。用户点击该 Session → Detail Pane 显示恢复选项。

**用户看到什么**：

```
Session SES-007 (interrupted)

  Seat: nimbus | Runtime: codex
  Branch: feature/oauth-login
  Last checkpoint: CP-004 (16:52, "测试失败分析完成")
  Interrupted at: 17:01

  Recovery options:
  ┌─────────────────────────────────────────────┐
  │ [Resume Native]  尝试使用原生 session 恢复    │
  │ [Rebuild]        从 CP-004 构建 LaunchPack   │
  │ [Minimal]        仅注入 WorkItem + Artifacts │
  └─────────────────────────────────────────────┘
```

点击 "Resume Native" → 尝试原生恢复，如果失败自动 fallback 到 "Rebuild"。
点击 "Rebuild" → 走跨工具切换流程（场景 8 的 Step 2-3）。

### Checkpoint 创建时机

| 触发 | 内容 |
|------|------|
| Session 结束（正常退出/Ctrl+C） | 完整：transcript 摘要 + 最终 diff + 状态 |
| Artifact 产出（检测到新文件/diff/test） | 增量：截至此刻的进展 |

定时快照（每 15 分钟）→ P1。

### Summary 生成

纯规则：从 transcript 尾部提取最后一轮 agent 回复，截断到 500 tokens。不调 LLM。

**CLI 等价**：`seatloom session resume SES-007`

**闭合条件**：中断 session 正确标记，恢复按三级降级执行，不丢超 5 分钟产物。

---

## 场景 10：决策回溯

**触发**：侧边栏点击某个 WorkItem → Main Panel 自动切到 Timeline 并按该 WorkItem 过滤。

**用户看到什么**：

Timeline 自动过滤为 WI-012 的完整事件链。点击任何事件 → Detail Pane 展开。

Detail Pane 中每个对象都有超链接指向关联对象：
- Session → 可查看 LaunchPack、Checkpoint、Transcript 引用
- Handoff → 可查看 from/to、purpose、artifacts、receipt
- Artifact → 可查看内容摘要、source session、commit

用户通过点击链接在对象之间跳转，像浏览一本链式索引。

**通用详情视图**（`seatloom show` 的 GUI 等价）：

点击任何对象 ID（WI-xxx、SES-xxx、HO-xxx、AR-xxx）→ Detail Pane 展示完整状态 + 关联引用列表。

**CLI 等价**：`seatloom timeline --workitem WI-12 --verbose` + `seatloom show <id>`

**闭合条件**：每个 WorkItem 的完整事件链可还原，每个对象可通过点击跳转查看，能回答"谁→哪个 session→什么上下文→什么交接→什么产出"。

---

## 场景 11：早晨收束

### 背景约束

MVP 没有 daemon。用户关机后不会有夜间自动推进。SeatLoom 保证的是：关机前的工作不丢，早上能快速回到上下文。

### 方案：启动时自动 reconcile + morning digest

用户早上打开 SeatLoom 桌面应用时，系统自动执行：

1. **Reconcile**：扫描 Git 状态，与 Ledger 对比
2. **Session 状态更新**：标记未正常结束的 session 为 `interrupted`
3. **生成 morning digest**：显示在 Inbox 视图顶部

**用户看到什么**：

Inbox 顶部显示 digest 横幅：
```
┌─ Morning Digest ──────────────────────────────────────────┐
│ Since your last session (yesterday 21:30):                 │
│  · 2 sessions ended normally (SES-005, SES-006)            │
│  · 1 session interrupted (SES-007, nimbus/codex)           │
│  · Git: 2 new commits on feature/oauth-login               │
│  · 1 WorkItem drifted: WI-010 done but unmerged            │
│                                              [Dismiss]     │
└────────────────────────────────────────────────────────────┘
```

横幅下方是正常的 Inbox 列表，中断的 session 和漂移的 WorkItem 已经作为条目出现。

**闭合条件**：首次启动自动 reconcile，digest 准确反映变化，中断 session 可从 Inbox 直接恢复。

---

## 场景 12：触发 Pipeline

**触发**：WorkItem Detail Pane 中点击 "Run Pipeline" 按钮 → 选择 Pipeline（verification_loop / requirement_handoff）。

**用户看到什么**：

Main Panel 切换到 Pipeline 执行视图：

```
Pipeline: verification_loop (WI-012)

  [1/3] collect_context .............. ✓ done (2s)
        Built ContextPack: 4200 tokens

  [2/3] run_verifier ................. ● running
        flux/codex SES-009 launched
        Running tests...
        ████████████░░░░░░░░ 60%

  [3/3] evaluate ..................... ○ pending
```

执行完成后：

```
  [2/3] run_verifier ................. ✓ done (45s)
        3 passed, 1 failed

  [3/3] evaluate ..................... ✓ done (1s)
        Result: FAIL

  ──────────────────────────────────────────
  Pipeline completed: FAIL (48s)
  Artifacts: AR-018 (test_report)
  Auto-handoff: HO-005 (flux → nimbus)
  ──────────────────────────────────────────

  [View Test Report]  [View Handoff]  [Re-run]
```

### 行为规格

| 行为 | 规格 |
|------|------|
| 执行方式 | 前台执行，GUI 实时更新每个 stage 进度 |
| 超时 | 每个 stage 默认 5 分钟，整体 15 分钟，可配置 |
| 失败重试 | 最多 2 次，之后标记 input_required 进入 Inbox |
| 并发保护 | 文件锁防止重复触发 |
| 中断 | 用户可点击 "Stop" 按钮，记录已完成 stage 快照 |
| 产物 | 写入 `.seatloom/pipelines/runs/` 和 Ledger |

**CLI 等价**：`seatloom pipeline run verification_loop --workitem WI-012`

**闭合条件**：每个 stage 输入输出完整记录，失败时自动创建 handoff，Inbox 正确更新。

---

## 场景 13：Reconcile

**触发**：`Ctrl+R`，或应用启动时自动触发，或 Pipeline 执行前自动触发。

**用户看到什么**：

状态栏短暂显示 "Reconciling..." → 完成后如果有问题，弹出通知：

```
Reconciliation: 2 issues found

  ⚠ WI-010: marked done but branch not merged → drifted
  ⚠ SES-007: process not running → interrupted

  [View in Inbox]  [Dismiss]
```

无问题时只在状态栏显示 "✓ Reconciled (0 issues)"。

**CLI 等价**：`seatloom reconcile`

**闭合条件**：状态不一致正确检测，异常进入 Inbox，Ledger 记录 `reconcile.completed`。

---

## 附录 A：MVP CLI 命令面

桌面 GUI 和 CLI 共享同一个 `.seatloom/` 数据层。CLI 用于脚本、CI、别名和不想打开 GUI 的场景。

```bash
# 初始化
seatloom init

# Seat 管理
seatloom seat add <name> --role <role>
seatloom seat list

# Session 管理
seatloom attach --seat <name> --runtime <runtime>
seatloom wrap --seat <name> <agent-cmd>
seatloom session launch --seat <name> --runtime <runtime> [--from-session <id>]
seatloom session resume <session-id>
seatloom session list

# WorkItem 管理
seatloom workitem create "<title>"
seatloom workitem create -i
seatloom workitem edit <id>
seatloom workitem list

# Handoff 管理
seatloom handoff create --from <seat> --to <seat> --workitem <id> \
    --purpose "..." --expected-outcome "..." [--artifacts AR-xxx,AR-yyy] [--send]
seatloom handoff list

# 观察面
seatloom timeline [--seat] [--workitem] [--since] [--type] [--limit] [--verbose]
seatloom inbox
seatloom show <object-id>

# Pipeline
seatloom pipeline run <pipeline-id> --workitem <id>

# 维护
seatloom reconcile
```

## 附录 B：场景与 Phase 映射

| Phase | 周数 | 覆盖场景 |
|-------|------|---------|
| Phase 0 | 1-2 | 场景 3（attach/wrap 适配器验证） |
| Phase 1 | 3-4 | 场景 1、2、4、5、6、11、13 + 应用 shell 布局 |
| Phase 2 | 5-6 | 场景 8、9 |
| Phase 3 | 7-8 | 场景 7、12 |
| Phase 4 | 9-10 | 场景 10（回溯验收），全部生命体征测试 |

## 附录 C：v2.0 变更说明

本文档从 v1.0 升级到 v2.0 的核心变更：

- 产品形态从"纯 CLI"调整为"桌面 GUI 应用 + CLI 子命令"
- 所有场景交互从命令行文本输出改为窗口化 GUI 交互（侧边栏、面板、弹窗、按钮）
- 新增应用布局定义（Sidebar + Main Panel + Detail Pane + Terminal Panel）
- 新增快捷键映射
- CLI 命令保留为附录，用于脚本和自动化场景

---

*本文档是 PRD v0.3 的配套文档。所有用户行为实现应以本文档为准。*
