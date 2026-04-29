# SeatLoom UX 规格 v1.0

| 项目 | 内容 |
|------|------|
| 文档 | UX Specification v1.0 |
| 状态 | Draft — 待 Mr. Zhang review |
| 作者 | Aegis (for Mira) |
| 更新时间 | 2026-04-27 |
| 依赖文档 | MVP Scenarios v2.0、Architecture Decisions v1.0、PRD v0.3 |

---

Reference-only during v0.5 consolidation. Start from `docs/PRODUCT_TRUTH.md` for current product work.

## 0. 本文档定位

**为什么需要这份文档**：mvp-scenarios.md 定义了"用户能做什么"，但没有定义"每一屏长什么样、信息怎么排列、状态怎么表达"。这份 UX 规格是 Mira 实现 React 原型组件的直接输入，也是 Mr. Zhang 确认"最终用户看到什么"的 review 对象。

**阅读顺序**：
1. 全局设计语言（§1）— 颜色、字体、图标、间距
2. 布局骨架（§2）— 四个区域怎么划分、怎么伸缩
3. 逐屏规格（§3-§10）— 每一屏的信息架构、交互细节、状态表达
4. 空状态与错误状态（§11）— 新用户第一眼看到什么
5. Mira 交付清单（§12）— 她需要产出哪些 React 组件

---

## 1. 全局设计语言

### 1.1 色彩系统

| 用途 | Light Mode | Dark Mode | 语义 |
|------|-----------|-----------|------|
| Background primary | `#FFFFFF` | `#1A1A2E` | 主面板背景 |
| Background secondary | `#F8F9FA` | `#16213E` | 侧边栏/Detail Pane 背景 |
| Background elevated | `#FFFFFF` | `#1F2B47` | 弹窗、卡片 |
| Text primary | `#1A1A2E` | `#E8E8E8` | 正文 |
| Text secondary | `#6B7280` | `#9CA3AF` | 辅助说明 |
| Text muted | `#9CA3AF` | `#6B7280` | 时间戳、ID |
| Accent | `#3B82F6` | `#60A5FA` | 主交互色（按钮、链接、选中态） |
| Status: active/running | `#22C55E` | `#4ADE80` | 绿 |
| Status: warning/blocked | `#F59E0B` | `#FBBF24` | 黄 |
| Status: error/failed | `#EF4444` | `#F87171` | 红 |
| Status: done/completed | `#6B7280` | `#9CA3AF` | 灰 |
| Status: drifted | `#A855F7` | `#C084FC` | 紫 |
| Border | `#E5E7EB` | `#2A3A5C` | 分割线 |

### 1.2 字体

| 用途 | 字体 | 大小 | 行高 |
|------|------|------|------|
| 标题（视图名） | system sans-serif, semibold | 18px | 24px |
| 小节标题 | system sans-serif, medium | 14px | 20px |
| 正文 | system sans-serif, regular | 13px | 18px |
| 辅助文字 | system sans-serif, regular | 12px | 16px |
| 等宽（ID、代码、终端） | system monospace | 13px | 18px |

### 1.3 状态指示符

所有对象状态用 **颜色圆点 + 文字标签** 双重表达，不依赖单一维度：

| 符号 | 含义 | 用于 |
|------|------|------|
| `●` 绿 | active / running | Seat 有活跃 session、Session running |
| `◐` 蓝 | in review / launching | WorkItem in review、Session launching |
| `●` 黄 | blocked / input required | WorkItem blocked、Session input required |
| `●` 红 | failed / interrupted | Session failed、Pipeline failed |
| `●` 紫 | drifted | WorkItem drifted |
| `○` 灰 | inactive / draft / completed | 无活跃 session、WorkItem draft/done |

### 1.4 Inbox 优先级图标

| 图标 | 级别 | 颜色 |
|------|------|------|
| `‼` | Critical | 红色 |
| `!` | Normal | 黄色 |
| `·` | Low | 灰色 |

### 1.5 间距与圆角

| 元素 | 值 |
|------|-----|
| 基础间距单位 | 4px |
| 列表行内边距 | 8px 12px |
| 列表行间距 | 0（紧凑），hover 用背景色区分 |
| 面板内边距 | 16px |
| 弹窗内边距 | 20px |
| 按钮圆角 | 6px |
| 卡片/弹窗圆角 | 8px |

---

## 2. 布局骨架

### 2.1 四区域划分

```
┌─────────────────────────────────────────────────────────────┐
│  TITLE BAR  (native, 32px height)              ─ □ ✕        │
├────────────┬──────────────────────────────┬──────────────────┤
│  SIDEBAR   │  MAIN PANEL                  │  DETAIL PANE     │
│  220px     │  flex: 1                     │  360px           │
│  min 180px │  min 400px                   │  collapsible     │
│            │                              │                  │
│            │                              │                  │
│            │                              │                  │
│            │                              │                  │
│            │                              │                  │
│            │                              │                  │
│            │                              │                  │
├────────────┴──────────────────────────────┴──────────────────┤
│  TERMINAL PANEL (collapsible, default hidden)                │
│  height: 300px (resizable), min 150px                        │
│  toggle: Ctrl+`                                              │
├──────────────────────────────────────────────────────────────┤
│  STATUS BAR  (24px height)                                    │
│  Left: project path | Center: reconcile status | Right: seat │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 区域职责

| 区域 | 固定/可变 | 职责 | 交互 |
|------|---------|------|------|
| **Title Bar** | 固定 32px | 应用名 + 窗口控制 | 原生拖拽移动窗口 |
| **Sidebar** | 宽度 220px，可拖拽调整 | 导航入口：Inbox badge、Seats、Sessions、WorkItems | 点选 → Main Panel / Detail Pane 联动 |
| **Main Panel** | 弹性宽度 | 主视图区域：Tab 切换 Inbox/Timeline/WorkItems | Tab 键 / Ctrl+1/2/3 切换 |
| **Detail Pane** | 360px，可折叠 | 选中对象详情 + 操作按钮 | 点击 Main Panel 条目打开，Esc 关闭 |
| **Terminal Panel** | 高度 300px，可折叠 | 内嵌 agent session 终端 | Ctrl+\` 切换，可多标签页 |
| **Status Bar** | 固定 24px | 项目路径、reconcile 状态、当前 seat | 只读信息展示 |

### 2.3 响应式规则

| 窗口宽度 | 行为 |
|---------|------|
| ≥ 1440px | Sidebar + Main Panel + Detail Pane 三栏并列 |
| 1024-1439px | Detail Pane 默认折叠，点击后覆盖 Main Panel 右侧 |
| < 1024px | Sidebar 折叠为图标栏（48px），点击弹出浮层 |

### 2.4 焦点流

`Tab` / `Shift+Tab` 在四个区域之间循环：Sidebar → Main Panel → Detail Pane → Terminal Panel

区域内用 `↑/↓` 上下移动选中项，`Enter` 确认/展开，`Esc` 返回上级/关闭面板。

---

## 3. Sidebar 规格

### 3.1 结构

```
SIDEBAR (220px)
┌──────────────────┐
│ ▸ Inbox (3)       │  ← badge 数字 = 未处理条目数
│                    │
│ ─── SEATS ─── [+] │  ← section header + add button
│  ● lyra            │
│  ● nimbus          │
│  ○ flux            │
│                    │
│ ─── SESSIONS ───   │
│  SES-004 ● claude  │  ← id + status dot + runtime
│  SES-005 ● codex   │
│  SES-007 ○ codex   │
│                    │
│ ─── WORKITEMS ─ [+]│
│  WI-012 ● high     │  ← id + status dot + priority
│  WI-011 ◐ medium   │
│  WI-013 ● low      │
└──────────────────┘
```

### 3.2 每一节行为

| 节 | 显示内容 | 点击行为 | 联动 |
|----|---------|---------|------|
| **Inbox** | 名称 + 未处理数 | Main Panel 切到 Inbox 视图 | — |
| **Seats** | name + status dot | Main Panel 不变，Detail Pane 显示 Seat 详情 | Timeline 按该 Seat 过滤 |
| **Sessions** | id (short) + dot + runtime | Detail Pane 显示 Session 详情 | Terminal Panel 可切换到该 session |
| **WorkItems** | id (short) + dot + priority text | Detail Pane 显示 WI 详情 | Timeline 按该 WI 过滤 |

### 3.3 分组与排序

- **Sessions** 按 Seat 分组（组标题 = seat name），组内按 created_at 降序
- **WorkItems** 按 priority 降序 → status（active > blocked > ready > draft > done）
- 每节可折叠（点击 section header 箭头）

### 3.4 Seat 条目格式

```
● nimbus                     ← dot + name
  └ SES-005 ● codex (running) ← 缩进的活跃 session
```

当某个 Seat 有活跃 session 时，在 Seat name 下方缩进显示最近的活跃 session（最多 1 条）。这让用户一眼看到"谁在跑什么"。

---

## 4. Main Panel — Inbox 视图

### 4.1 信息架构

```
┌────────────────────────────────────────────────┐
│  [Inbox]  Timeline  WorkItems         Ctrl+1   │  ← Tab bar
├────────────────────────────────────────────────┤
│ ┌─ Morning Digest ───────────────────────────┐ │  ← 条件出现
│ │ Since last session (yesterday 21:30):       │ │
│ │ · 2 sessions ended · 1 interrupted          │ │
│ │ · Git: 2 commits on feature/oauth-login     │ │
│ │ · 1 WI drifted: WI-010             [Dismiss]│ │
│ └────────────────────────────────────────────┘ │
│                                                │
│ Inbox (5)                                      │  ← 计数
│                                                │
│  ‼ Handoff pending     flux → you   WI-012     │  ← row
│     "fix: callback TypeError"                  │
│  ‼ Review requested    nimbus       WI-011     │
│     "OAuth implementation ready"               │
│  !  Input required     nimbus SES-007          │
│     "NextAuth v4 还是 v5?"                      │
│  !  WorkItem blocked   ──auto── WI-013         │
│     "blocked by WI-012"                        │
│  ·  Drift detected     ──auto── WI-010         │
│     "done 但 branch 未 merge"                   │
└────────────────────────────────────────────────┘
```

### 4.2 每行结构

```
[priority icon]  [type label]  [actor]  [object ref]
                 [summary text, single line, truncated with ...]
```

- 行高：48px（双行：第一行 type+actor+ref，第二行 summary）
- 选中行：浅蓝背景 (`accent` 10% opacity)
- Hover：浅灰背景

### 4.3 Morning Digest 横幅

- 显示条件：本次启动时 reconcile 发现变化（新 commit、中断 session、漂移 WI）
- 位置：Inbox 列表上方，固定高度卡片
- 关闭后本次 session 不再显示

### 4.4 空状态

```
┌────────────────────────────────────────┐
│                                        │
│        ✓ All clear                     │
│        No items need attention         │
│                                        │
└────────────────────────────────────────┘
```

---

## 5. Main Panel — Timeline 视图

### 5.1 信息架构

```
┌────────────────────────────────────────────────┐
│  Inbox  [Timeline]  WorkItems         Ctrl+2   │
├────────────────────────────────────────────────┤
│  Filter: [All seats ▾] [All types ▾] [24h ▾]  │  ← filter bar
│                                        [🔍]   │  ← search
├────────────────────────────────────────────────┤
│  14:32  nimbus  session.started   SES-004      │
│         claude_code                            │
│  14:35  nimbus  artifact.created  AR-012       │
│         design_note "OAuth flow design"        │
│  14:51  lyra    handoff.sent      HO-003       │
│         lyra → nimbus  WI-012                  │
│  15:10  nimbus  handoff.accepted  HO-003       │
│  15:12  nimbus  session.started   SES-005      │
│         codex (rehydrated from SES-004)        │
│                                                │
│  ─── 30 min gap ───                            │
│                                                │
│  16:40  nimbus  artifact.created  AR-015       │
│         diff_summary "+OAuth callback"         │
│  16:41  auto    pipeline.started  verification │
│         WI-012                                 │
│  16:52  flux    artifact.created  AR-016       │
│         test_report "3/4 passed"               │
│  16:52  auto    pipeline.completed FAIL        │
│  16:53  auto    handoff.sent      HO-004       │
│         flux → nimbus "fix callback"           │
└────────────────────────────────────────────────┘
```

### 5.2 每行结构

```
[time]  [actor badge]  [event_type tag]  [object ref link]
        [summary line, contextual]
```

- 行高：40px（双行）
- `actor badge`：Seat name，等宽 8 字符宽度对齐（超长截断）。`auto` 用斜体灰色。
- `event_type tag`：小圆角标签，按类型着色：
  - session.* → 蓝
  - handoff.* → 绿
  - artifact.* → 紫
  - pipeline.* → 橙
  - reconcile.*/drift.* → 灰
- `object ref`：可点击，蓝色链接样式，点击 → Detail Pane
- 时间间隔 > 30 分钟时插入分隔线 `── 30 min gap ──`

### 5.3 Filter Bar

| 过滤器 | 类型 | 选项 |
|--------|------|------|
| Seat | 单选下拉 | All seats / lyra / nimbus / flux / ... |
| Event type | 多选下拉 | session / handoff / artifact / pipeline / reconcile |
| Time range | 单选下拉 | 1h / 6h / 24h / 7d / custom range |

- 过滤器状态变化 → 列表实时更新（无需按钮确认）
- 侧边栏点击 Seat/WI 时自动设置对应过滤器

### 5.4 搜索

搜索框支持文本匹配：匹配 summary、object ID、actor name。输入时实时过滤。

### 5.5 加载策略

- 默认加载最近 200 条事件
- 向上滚动到顶部 → 自动加载更早的 200 条
- 新事件 → 自动追加到底部（如果用户在最底部位置）
- 如果用户向上滚动了 → 底部显示 "↓ New events" 浮动按钮

---

## 6. Main Panel — WorkItems 视图

### 6.1 信息架构

```
┌────────────────────────────────────────────────┐
│  Inbox  Timeline  [WorkItems]         Ctrl+3   │
├────────────────────────────────────────────────┤
│  [+ New]  Group by: [Status ▾]  Sort: [Pri ▾]  │
├────────────────────────────────────────────────┤
│  ▼ Active (2)                                  │
│                                                │
│   ● WI-012  实现 OAuth 登录           high      │
│     nimbus · 3 AC · SES-005 running            │
│                                                │
│   ● WI-013  修复用户注册bug           medium    │
│     nimbus · 1 AC · blocked by WI-012          │
│                                                │
│  ▼ In Review (1)                               │
│                                                │
│   ◐ WI-011  OAuth implementation     medium    │
│     nimbus · review by flux                    │
│                                                │
│  ▶ Draft (0)                                   │
│  ▶ Done (3)                                    │
└────────────────────────────────────────────────┘
```

### 6.2 每行结构

```
[status dot]  [id]  [title]                    [priority]
              [owner] · [AC count] · [context hint]
```

- context hint：最相关的一条信息（running session / blocked by / in review by / last handoff）
- 分组折叠：点击组标题箭头

### 6.3 分组与排序

| 分组方式 | 可选 |
|---------|------|
| 按 Status | Draft → Ready → Active → Blocked → InReview → Drifted → Done |
| 按 Owner | 按 Seat 分组 |
| 无分组 | 平铺列表 |

排序：Priority 降序 → updated_at 降序

---

## 7. Detail Pane 规格

Detail Pane 是右侧 360px 的上下文面板，显示选中对象的完整信息。根据选中对象类型切换模板。

### 7.1 通用结构

```
┌──────────────────┐
│  [×]  TYPE  ID   │  ← header: close button + object type + id
├──────────────────┤
│                  │
│  CONTENT         │  ← 按对象类型不同
│                  │
├──────────────────┤
│  ACTIONS         │  ← 按对象类型不同的操作按钮
├──────────────────┤
│  RELATED         │  ← 关联对象链接列表
└──────────────────┘
```

### 7.2 Seat 详情

```
● nimbus (active)

  Role: architect
  Created: 2026-04-27

  Active Sessions:
    SES-005 ● codex (running)

  Recent WorkItems:
    WI-012 ● 实现 OAuth 登录
    WI-011 ◐ OAuth implementation

  ────────────────
  [Launch Session]  [View Timeline]
```

### 7.3 Session 详情

```
SES-005 ● running

  Seat: nimbus
  Runtime: codex
  Branch: feature/oauth-login
  Started: 15:12 today
  PID: 42950

  Last Checkpoint:
    CP-003 (16:40) "OAuth callback 实现完成"

  LaunchPack:
    📄 .seatloom/sessions/ses-005/launch_pack.md
    Budget: 6200 tokens
    Source: SES-004 (rehydrated)

  Artifacts Produced:
    AR-015 diff_summary "+OAuth callback"

  ────────────────
  [Open Terminal]  [Switch Runtime]  [View LaunchPack]
```

### 7.4 WorkItem 详情

```
WI-012 ● active (high)

  实现 OAuth 登录
  Goal: 支持 Google/GitHub OAuth，使用 NextAuth.js

  Acceptance Criteria:
    ☐ 用户可使用 Google 账号登录
    ☐ 用户可使用 GitHub 账号登录
    ☐ 登录后跳转到 /dashboard

  Owner: nimbus
  Branch: feature/oauth-login

  Sessions:
    SES-004 ○ claude_code (completed)
    SES-005 ● codex (running)

  Handoffs:
    HO-003 lyra → nimbus (accepted)
    HO-004 flux → nimbus (pending)

  Artifacts:
    AR-010 brief · AR-011 AC · AR-012 design_note
    AR-015 diff · AR-016 test_report

  ────────────────
  [Edit]  [Create Handoff]  [Run Pipeline]  [View Timeline]
```

### 7.5 Handoff 详情

```
HO-004 (sent)

  From: flux → To: nimbus
  WorkItem: WI-012 实现 OAuth 登录

  Purpose:
    修复 OAuth callback TypeError

  Expected Outcome:
    callback.test.ts 全部通过

  Artifacts:
    📄 AR-016 test_report "3/4 passed"
    📄 AR-015 diff_summary "+OAuth callback"

  Receipt: required
  Sent: 16:53 today

  ────────────────
  [Accept]  [Return]  [View Artifacts]
```

### 7.6 Artifact 详情

```
AR-016 (test_report)

  Title: "3 passed, 1 failed"
  Created: 16:52 today
  Source Session: SES-008 (flux/codex)
  Source WorkItem: WI-012

  Content Preview:
  ┌──────────────────────────────┐
  │ ✓ auth/login.test.ts         │
  │ ✓ auth/signup.test.ts        │
  │ ✓ auth/logout.test.ts        │
  │ ✗ auth/callback.test.ts      │
  │   TypeError: Cannot read ... │
  └──────────────────────────────┘

  ────────────────
  [Open Full Content]  [Copy Path]
```

### 7.7 Event 详情（Timeline 条目展开）

```
ev-a2  artifact.created

  Time: 16:40:12 today
  Actor: nimbus (seat)

  Objects:
    AR-015 diff_summary
    SES-005 (source session)

  Evidence:
    sessions/ses-005/raw/stdout.log (line 1240-1280)

  Payload:
    kind: DiffSummary
    title: "+OAuth callback handler"
    files_changed: 3
    insertions: 42
    deletions: 8

  ────────────────
  [View Artifact]  [View Session]
```

---

## 8. Terminal Panel 规格

### 8.1 布局

```
┌─ Terminal ──────────────────────────────────────┐
│ [SES-005 ● nimbus/codex] [SES-008 ○ flux/codex]│  ← tab bar
│ [+ New Session]                          [▼ ▲]  │  ← resize handle
├─────────────────────────────────────────────────┤
│                                                 │
│  $ codex                                        │  ← xterm.js terminal
│  > You are an engineer working on OAuth login...│
│  > I'll start by examining the current code...  │
│                                                 │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 8.2 Tab Bar

- 每个活跃 session 一个 tab：`SES-id ● seat/runtime`
- Tab 右侧有关闭按钮（×）→ 关闭 tab 不终止 session（移到后台）
- `+ New Session` 按钮 → 打开 Launch 弹窗（场景 3.3）
- 非活跃 tab 灰色，活跃 tab 有下划线

### 8.3 行为

| 操作 | 行为 |
|------|------|
| 键盘输入 | 直接发送到 PTY stdin |
| 输出显示 | 实时从 PTY stdout 渲染 |
| Ctrl+C | 发送 SIGINT 到 PTY，session 结束 → 自动 Checkpoint |
| 窗口大小变化 | 自动 pty_resize |
| 折叠 | 终端继续运行，只是不可见 |

---

## 9. 弹窗规格

所有弹窗居中显示，带半透明遮罩层（点击遮罩关闭），Esc 关闭。

### 9.1 初始化向导（场景 1）

```
┌─────────────────────────────────────┐
│  Initialize SeatLoom                 │
│                                     │
│  Project: ~/projects/seatloom       │
│  Git status: clean, 3 branches     │
│                                     │
│  This will create a .seatloom/      │
│  directory in your project root.    │
│                                     │
│  [Initialize]  [Cancel]             │
└─────────────────────────────────────┘
```

宽度：480px。最小化信息，不问多余问题。

### 9.2 Add Seat（场景 2）

```
┌───────────────────────────────────┐
│  Add Seat                         │
│                                   │
│  Name:  [               ]         │
│  Role:  [product_owner ▾]         │
│                                   │
│  [Create]  [Cancel]               │
└───────────────────────────────────┘
```

宽度：400px。Role 下拉包含：product_owner / architect / verifier / designer / custom。

### 9.3 Attach Session（场景 3.2）

```
┌────────────────────────────────────────────────┐
│  Attach to Running Session                      │
│                                                │
│  Runtime          PID    Directory    Started   │
│  ● Claude Code    42831  ~/seatloom   10m ago   │
│  ● Codex          42950  ~/other      2h ago    │
│  ○ Cursor CLI     43100  ~/seatloom   5m ago    │
│                                                │
│  Assign to seat: [nimbus ▾]                     │
│                                                │
│  [Attach]  [Cancel]                             │
└────────────────────────────────────────────────┘
```

宽度：560px。列表可滚动，最多显示 10 条，多余出滚动条。

### 9.4 Launch Session（场景 3.3）

```
┌───────────────────────────────────┐
│  Launch New Session               │
│                                   │
│  Seat:     [nimbus ▾]             │
│  Runtime:  [claude ▾]             │
│  Working directory:               │
│  [~/projects/seatloom       ] [📁]│
│                                   │
│  [Launch]  [Cancel]               │
└───────────────────────────────────┘
```

宽度：440px。📁 按钮打开系统文件选择器。

### 9.5 New WorkItem — 快速模式（场景 6）

```
┌─────────────────────────────────────────┐
│  New WorkItem                           │
│                                         │
│  Title: [                             ] │
│                                         │
│  [Create as Draft] [More Options] [Cancel]│
└─────────────────────────────────────────┘
```

宽度：480px。回车等同 "Create as Draft"。

### 9.6 New WorkItem — 完整模式（场景 6）

```
┌──────────────────────────────────────────┐
│  New WorkItem                            │
│                                          │
│  Title:    [                           ] │
│  Goal:     [                           ] │
│                                          │
│  Acceptance Criteria:                    │
│    [+ Add criterion]                     │
│    ☐ [criterion 1                     ] [×]│
│    ☐ [criterion 2                     ] [×]│
│                                          │
│  Assign to: [nimbus ▾]                   │
│  Priority:  ○ high  ● medium  ○ low     │
│                                          │
│  [Create]  [Cancel]                      │
└──────────────────────────────────────────┘
```

宽度：520px。[+ Add criterion] 点击后在下方追加一行输入框。

### 9.7 New Handoff（场景 7）

```
┌──────────────────────────────────────────┐
│  New Handoff                             │
│                                          │
│  From:     [lyra ▾]                      │
│  To:       [nimbus ▾]                    │
│  WorkItem: [WI-012 实现 OAuth 登录 ▾]   │
│                                          │
│  Purpose:                                │
│  [                                     ] │
│  [                                     ] │
│                                          │
│  Expected Outcome:                       │
│  [                                     ] │
│  [                                     ] │
│                                          │
│  Attach Artifacts:                       │
│    ☑ AR-010  brief          "OAuth 需求" │
│    ☑ AR-011  AC             "3 条 AC"    │
│    ☐ AR-012  design_note    "OAuth flow" │
│                                          │
│  Require receipt: [☑]                    │
│                                          │
│  [Save as Draft]  [Send Now]  [Cancel]   │
└──────────────────────────────────────────┘
```

宽度：560px。Artifact 列表自动填充该 WorkItem 关联的所有 Artifact，默认全选。

### 9.8 Switch Runtime（场景 8）

```
┌──────────────────────────────────────────┐
│  Switch Runtime for Nimbus               │
│                                          │
│  Current: SES-004 (claude_code)          │
│  New runtime: [codex ▾]                  │
│  Working directory:                      │
│  [~/projects/seatloom             ] [📁] │
│                                          │
│  [Build LaunchPack & Launch]  [Cancel]   │
└──────────────────────────────────────────┘
```

### 9.9 LaunchPack Preview（场景 8 step 2）

```
┌──────────────────────────────────────────────┐
│  LaunchPack Preview (6200 tokens)            │
│                                              │
│  ✓ WorkItem: WI-012 "实现 OAuth 登录" (3 AC) │
│  ✓ From session: SES-004 (checkpoint)        │
│  ✓ Artifacts: AR-012, AR-010                 │
│  ✓ Branch: feature/oauth-login               │
│  ✓ Recent failure: callback.test.ts          │
│                                              │
│  Saved to:                                   │
│  .seatloom/sessions/ses_005/launch_pack.md   │
│                                              │
│  [Launch Now]  [Edit LaunchPack]  [Cancel]   │
└──────────────────────────────────────────────┘
```

宽度：520px。"Edit LaunchPack" 在文本编辑器中打开 .md 文件。

### 9.10 Pipeline 执行视图（场景 12）

Pipeline 执行不使用弹窗，而是在 Main Panel 中全屏显示（替代当前 Tab 内容）。见 §10。

---

## 10. Pipeline 执行视图

当 Pipeline 开始运行时，Main Panel 切换到 Pipeline 视图。

```
┌────────────────────────────────────────────────┐
│  ← Back to WorkItems                           │
│                                                │
│  Pipeline: verification_loop (WI-012)          │
│  Status: ● running                  [Stop]     │
│                                                │
│  ┌─ Stage 1/3: collect_context ─────── ✓ 2s ─┐│
│  │ Built ContextPack: 4200 tokens             ││
│  └────────────────────────────────────────────┘│
│                                                │
│  ┌─ Stage 2/3: run_verifier ──────── ● 23s ─┐ │
│  │ flux/codex SES-009 launched               │ │
│  │ Running tests...                          │ │
│  │ ████████████░░░░░░░░ 60%                  │ │
│  └───────────────────────────────────────────┘ │
│                                                │
│  ┌─ Stage 3/3: evaluate ──────────── ○ ─────┐ │
│  │ (pending)                                 │ │
│  └───────────────────────────────────────────┘ │
│                                                │
│                                                │
│  ═══════════════ completed ══════════════════  │
│  Result: FAIL (48s)                            │
│  Artifacts: AR-018 (test_report)               │
│  Auto-handoff: HO-005 (flux → nimbus)          │
│                                                │
│  [View Test Report] [View Handoff] [Re-run]    │
└────────────────────────────────────────────────┘
```

每个 Stage 卡片实时更新内容。完成后底部显示结果摘要和操作按钮。

---

## 11. 空状态与错误状态

### 11.1 首次打开（未初始化）

Main Panel 居中显示：

```
  Welcome to SeatLoom

  Open a Git project to get started.

  [Open Project]
```

### 11.2 已初始化但无 Seat

Sidebar SEATS 区域：

```
  No seats yet.
  [+ Add your first seat]
```

### 11.3 已有 Seat 但无 Session

Sidebar SESSIONS 区域：

```
  No sessions yet.
  Attach or launch an agent to start.
```

Terminal Panel 折叠状态下 Status Bar 提示：`Press Ctrl+\` to open terminal`

### 11.4 Inbox 为空

见 §4.4。

### 11.5 Timeline 过滤无结果

```
  No events match your filters.
  [Clear Filters]
```

### 11.6 网络/文件错误

Status Bar 红色背景闪烁：`⚠ Failed to read .seatloom/ledger/events.jsonl`

Detail Pane 中操作失败：红色 toast 通知（右下角，5 秒自动消失）：
```
  ✗ Failed to accept handoff HO-004
  Reason: file lock timeout
  [Retry]  [Dismiss]
```

---

## 12. Mira 交付清单

以下是 Mira 需要产出的 React 组件，按优先级排序。每个组件应当：
- 在 `ui/src/` 目录下
- 使用 mock 数据渲染（不需要真实 IPC）
- 支持 light/dark theme
- 附带 empty state 和至少一个 error state

### Phase 1 交付物（对应实现 Phase 1，Week 3-4）

| # | 组件 | 位置 | 说明 |
|---|------|------|------|
| 1 | `AppShell` | `layouts/AppShell.tsx` | 四区域骨架 + 响应式规则 |
| 2 | `Sidebar` | `layouts/Sidebar.tsx` | Inbox badge + Seats/Sessions/WIs 分节 |
| 3 | `InboxView` | `views/InboxView.tsx` | Inbox 列表 + Morning Digest + 空状态 |
| 4 | `InboxItem` | `components/InboxItem.tsx` | 单行渲染（priority icon + type + actor + summary） |
| 5 | `TimelineView` | `views/TimelineView.tsx` | Timeline 列表 + Filter Bar + 搜索 + 时间间隔线 |
| 6 | `EventRow` | `components/EventRow.tsx` | 单条事件渲染（time + actor + type tag + ref link） |
| 7 | `DetailPane` | `layouts/DetailPane.tsx` | 通用 shell：header + content + actions + related |
| 8 | `SeatDetail` | `components/SeatDetail.tsx` | Seat 详情模板 |
| 9 | `SessionDetail` | `components/SessionDetail.tsx` | Session 详情模板 |
| 10 | `WorkItemDetail` | `components/WorkItemDetail.tsx` | WorkItem 详情模板 |
| 11 | `HandoffDetail` | `components/HandoffDetail.tsx` | Handoff 详情模板 |
| 12 | `StatusBar` | `layouts/StatusBar.tsx` | 底部状态栏 |
| 13 | `InitDialog` | `components/InitDialog.tsx` | 初始化弹窗 |
| 14 | `AddSeatDialog` | `components/AddSeatDialog.tsx` | 添加 Seat 弹窗 |
| 15 | Theme | `styles/theme.ts` + `styles/globals.css` | 色彩系统 + light/dark 切换 |

### Phase 2 交付物（对应实现 Phase 2，Week 5-6）

| # | 组件 | 位置 | 说明 |
|---|------|------|------|
| 16 | `WorkItemForm` | `components/WorkItemForm.tsx` | 快速+完整创建表单 |
| 17 | `HandoffForm` | `components/HandoffForm.tsx` | Handoff 创建表单 + Artifact 勾选 |
| 18 | `LaunchPackPreview` | `components/LaunchPackPreview.tsx` | LaunchPack 预览弹窗 |
| 19 | `AttachDialog` | `components/AttachDialog.tsx` | Attach 进程选择弹窗 |
| 20 | `LaunchDialog` | `components/LaunchDialog.tsx` | Launch Session 弹窗 |
| 21 | `SwitchRuntimeDialog` | `components/SwitchRuntimeDialog.tsx` | 切换 Runtime 弹窗 |
| 22 | `ArtifactDetail` | `components/ArtifactDetail.tsx` | Artifact 详情（含内容预览） |
| 23 | `EventDetail` | `components/EventDetail.tsx` | Event verbose 详情 |
| 24 | `WorkItemsView` | `views/WorkItemsView.tsx` | WorkItems 列表视图 + 分组 |

### Phase 3 交付物（对应实现 Phase 3，Week 7-8）

| # | 组件 | 位置 | 说明 |
|---|------|------|------|
| 25 | `TerminalPanel` | `layouts/TerminalPanel.tsx` | xterm.js 嵌入 + Tab bar |
| 26 | `PipelineView` | `views/PipelineView.tsx` | Pipeline 执行实时进度 |
| 27 | `PipelineStageCard` | `components/PipelineStageCard.tsx` | 单个 Stage 状态卡片 |
| 28 | `ReconcileNotification` | `components/ReconcileNotification.tsx` | Reconcile 结果弹出通知 |
| 29 | `MorningDigest` | `components/MorningDigest.tsx` | 可复用的 Digest 横幅 |
| 30 | `SessionRecoveryPanel` | `components/SessionRecoveryPanel.tsx` | 中断恢复选项面板 |

---

*本文档待 Mr. Zhang review 后生效，作为 Mira React 原型开发的直接输入。*
