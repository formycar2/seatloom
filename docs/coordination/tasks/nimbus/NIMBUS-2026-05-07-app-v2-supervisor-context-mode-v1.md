# Task: app-v2 Supervisor Context Mode (chan-03)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-05-07-app-v2-supervisor-context-mode-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-08 |
| depends_on | `docs/architecture-decisions.md` AD-013, `docs/architecture-design.md` §7.1, `docs/coordination/reviews/2026-05-07-aegis-next-app-v2-slice-priority-decision.md`, `docs/coordination/tasks/aegis/AEGIS-2026-04-30-pending-changes-register.md` (chan-03), commit `1c7935c` |
| tags | nimbus, ui, app-v2, supervisor, context-mode, global-dashboard, L1, chan-03, pinned-commit |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Do not parallelize with any other Nimbus task. Pause Postgres / Rust infra work during this slice — Aegis already paused L2 UI slices. |

## Cross-Track Note (重要)

Nimbus 主轨为 Rust + Postgres infra。本任务跨入 v2 前端（TypeScript / React / Zustand），是 Copilot 不可用情况下的临时跨轨指派（per Mr. Zhang 2026-05-07）。

为降低跨轨摩擦，本任务包附带：
- **§A** Phase 1 模块结构参考（14 个文件 + 各自职责一行）
- **§B** GlobalDashboard ↔ ProjectDashboard 对照点
- **§C** localStorage 命名约定
- **§D** Stale state fallback 规则
- **§E** MOCK_GLOBAL_SUMMARY 数据形状 sketch
- **§F** "回到全局" UI 入口形态

请按 §A → §F 顺序读完后再动代码。

## Base Commit

Work from commit `1c7935c` on `track/infra-foundation`.

If local HEAD is not at or ahead of `1c7935c` when you start, stop and report drift.

## Objective

实现 AD-013 定义的 Supervisor 两层上下文模型：

1. 引入 `currentContextMode: 'global' | 'project'` 状态机
2. 新增 `GlobalDashboard.tsx` 组件承载 Global context
3. `SupervisorPanel.tsx` 根据 mode 决定渲染 GlobalDashboard 或 ProjectDashboard
4. `mock-data.ts` 新增 `MOCK_GLOBAL_SUMMARY`
5. `types.ts` 新增 `SupervisorContextMode` 类型
6. localStorage 持久化 mode + activeProjectId
7. **零回归**：Project mode 下所有 Phase 1/2 已交付视图保持不变

## Scope Boundary

### In scope

仅以下文件变更：

- `ui/src/app-v2/types.ts` — 新增 `SupervisorContextMode` 类型
- `ui/src/app-v2/mock-data.ts` — 新增 `MOCK_GLOBAL_SUMMARY` 常量
- `ui/src/app-v2/dashboard/GlobalDashboard.tsx` — 新建文件
- `ui/src/app-v2/panel/SupervisorPanel.tsx` — 增加 context mode state + 切换入口 + 条件渲染
- 可选：新建 `ui/src/stores/useSupervisorStore.ts`（Zustand）**或** 把 mode 作为 `SupervisorPanel` 内的 `useState`（任选其一，见 §B 决策点）

### Explicitly out of scope

- 不修改 `ProjectDashboard.tsx`（项目视图保持现状）
- 不修改任何 `views/` 文件（Inbox/WorkItems/Artifacts 视图）
- 不修改 `dashboard/` 下其他 Section 文件
- 不修改 `useDataStore.ts`
- 不修改 Rust 后端、Postgres、infra 脚本
- 不动 styles/tokens.css
- 不重构 Phase 1 已有模块边界
- 不引入新第三方依赖

## Required Read Order

按此顺序读完后再动代码：

1. `docs/architecture-decisions.md` AD-013（互斥规则、切换语义、数据隔离规则、零回归约束）
2. `docs/architecture-design.md` §7.1 supervisorStore 块
3. 本文件 §A → §F
4. `ui/src/app-v2/panel/SupervisorPanel.tsx`（理解现有渲染流）
5. `ui/src/app-v2/dashboard/ProjectDashboard.tsx`（仅作对照，不改）
6. `ui/src/app-v2/types.ts`、`ui/src/app-v2/mock-data.ts`（理解既有类型/数据组织风格）

## §A — Phase 1 模块结构参考

```
ui/src/app-v2/
├── AppV2.tsx                          (110 lines, root + ⌘K + supervisor toggle)
├── types.ts                           (ChatContact, ChatMessage, PlanPhase, TimelineEntry, ProjectChannelData)
├── mock-data.ts                       (MOCK_CONTACTS, MOCK_MESSAGES, MOCK_CHANNEL_DATA)
│
├── components/
│   ├── Avatar.tsx                     (SeatLoomLogo, StatusDot, Avatar)
│   ├── MessageBubble.tsx              (TYPE_BADGES + MessageBubble)
│   ├── ContactRow.tsx                 (ContactRow - left contact list row)
│   └── ChatInput.tsx                  (ChatInput with routing governance guard)
│
├── panel/
│   └── SupervisorPanel.tsx            (draggable IM panel, contact list + dialog area)
│
├── dashboard/
│   ├── ProjectDashboard.tsx           (project channel main view, overview + tabs)
│   ├── BlockersSection.tsx            (blockers panel)
│   ├── ActiveWorkSection.tsx          (active work panel)
│   ├── TimelineSection.tsx            (timeline / activity panel)
│   ├── StageProgressSection.tsx       (stage progress header)
│   └── GoalsSection.tsx               (goals + stages list)
│
└── views/
    ├── InboxView.tsx                  (Phase 2 - decision queue)
    ├── WorkItemsView.tsx              (Phase 2 - workitem lifecycle)
    └── ArtifactsView.tsx              (Phase 2 - document library)
```

新增文件位置：
- `dashboard/GlobalDashboard.tsx`（与 ProjectDashboard 同级，dashboard/ 下）

## §B — GlobalDashboard ↔ ProjectDashboard 对照点

| 维度 | ProjectDashboard | GlobalDashboard（你要新建的） |
|------|-----------------|------------------------------|
| 入口条件 | `currentContextMode === 'project'` && `activeProjectId !== null` | `currentContextMode === 'global'` |
| 数据源 | `useDataStore().projectData[activeProjectId]` + `MOCK_CHANNEL_DATA[contactId]` | `MOCK_GLOBAL_SUMMARY`（本任务新增） |
| Props | `channelId`, `activeContactId` | 无（或仅 `onSelectProject(projectId)` 一个回调） |
| Tab 导航 | 4 个 tab（看板/待办/工作项/文档） | 无 tab — 单一全局视图 |
| 数据范围 | 单 project 的完整 projectData | 所有 project 的聚合 summary（每 project 一行） |
| 点击 project 行 | N/A | 调用 `enterProject(projectId)` 切换到 project mode |

**State location 决策点**：

context mode + activeProjectId 的 state 位置任选其一：

- **(选项 X) 放进 `SupervisorPanel.tsx` 的 `useState`**：最简单，与 Phase 1/2 已有 localStorage 持久化模式一致（参考 `STORAGE_KEY_POS` / `STORAGE_KEY_SIZE`）。
- **(选项 Y) 新建 `ui/src/stores/useSupervisorStore.ts`**：Zustand store，符合 architecture-design.md §7.1 蓝图，未来可被其他组件 subscribe。

推荐**选项 X**（最小变更、最低跨轨风险）。如果你认为选项 Y 更合理可以选 Y，但要在 delivery 中说明理由。

## §C — localStorage 命名约定

新增 key：

- `seatloom.supervisor.contextMode` — 存 `'global' | 'project'`
- `seatloom.supervisor.activeProjectId` — 存 `string | null`（global mode 下序列化为 `null` 字符串或省略 key 都可以）

不得改名既有 key（`sl-supervisor-open`、`STORAGE_KEY_POS`、`STORAGE_KEY_SIZE` 等）。

## §D — Stale state fallback 规则

从 localStorage 恢复时：

1. 读 `seatloom.supervisor.contextMode`，缺失或非法 → 默认 `'global'`
2. 读 `seatloom.supervisor.activeProjectId`，缺失 → `null`
3. **校验**：如果 mode 是 `'project'` 且 `activeProjectId` 在 `useDataStore().projects` 中**找不到**对应 project（可能 project 已删除或 mock 数据更新），**降级回 global mode**（`mode = 'global'`, `activeProjectId = null`）

这条规则的目的：避免 localStorage 残留过期 projectId 导致 ProjectDashboard 渲染白屏或抛错。

## §E — MOCK_GLOBAL_SUMMARY 数据形状 sketch

每个 project 一行卡片，字段如下：

```typescript
interface GlobalProjectSummary {
  projectId: string;          // matches useDataStore().projects[].id
  projectName: string;         // matches useDataStore().projects[].name
  healthStatus: 'healthy' | 'warning' | 'blocked';
  activeBlockerCount: number;  // 当前阻塞数量
  lastActivityTime: string;    // ISO 8601 timestamp
}

export const MOCK_GLOBAL_SUMMARY: GlobalProjectSummary[] = [
  // 每个已存在 project 一行，与 useDataStore.INITIAL_PROJECTS 保持一致
  // 至少覆盖：p-1 (SeatLoom 协调核心), p-2 (ThoughtOnly 叙事引擎), 以及任何其他 mock project
];
```

UI 上每行至少显示：
- project 名称
- health status（颜色 dot：healthy=绿、warning=黄、blocked=红）
- active blocker count（数字 chip）
- last activity time（相对时间，如 "5 分钟前"）

行可点击 → `enterProject(projectId)`。

**注意**：MOCK_GLOBAL_SUMMARY 是**纯 mock**，不要尝试从 `useDataStore()` 实时计算 health status — 那是后续 slice 的工作（reconcile + 真实数据派生）。本 slice 只要求 mock 数据形状正确并能渲染。

## §F — "回到全局" UI 入口形态

AD-013 只指明在 SupervisorPanel 顶部，没说具体形态。本任务包要求：

**面包屑式入口**：
- Global mode 下顶部显示：`全局`（无前缀箭头）
- Project mode 下顶部显示：`全局 › <项目名>`，其中 `全局` 是可点击文本，点击后调用 `enterGlobal()` 返回 global mode
- 当前所在节点（Project mode 下的项目名）显示为非可点击文本，颜色比 `全局` 略深以示当前位置

不允许的形态（避免歧义）：
- 不要用 tab 形态（与 ProjectDashboard 内部已有 tab 冲突）
- 不要用单独按钮浮在右上角（破坏面包屑的"层级"语义）
- 不要叠加返回箭头 `←`（语义重复）

## Implementation Strategy

### Step 1 — 类型 + Mock 数据

- `types.ts` 增加 `SupervisorContextMode` 和 `GlobalProjectSummary`
- `mock-data.ts` 增加 `MOCK_GLOBAL_SUMMARY` 数组，覆盖既有所有 mock project

### Step 2 — GlobalDashboard 组件

- 新建 `dashboard/GlobalDashboard.tsx`
- Props: `{ onSelectProject: (projectId: string) => void }`
- 内部：渲染 `MOCK_GLOBAL_SUMMARY`，每行可点击触发 `onSelectProject`
- 视觉风格：复用 Phase 1/2 的 token + Avatar + StatusDot 组件，不引入新视觉系统

### Step 3 — SupervisorPanel context mode

- 增加 `currentContextMode` 和 `activeProjectId` state（推荐选项 X：local useState）
- localStorage 读写：mount 时读取并按 §D 规则校验，state 变化时持久化
- 顶部增加面包屑（§F）
- 根据 mode 条件渲染：
  - `'global'` → `<GlobalDashboard onSelectProject={handleEnterProject} />`
  - `'project'` → `<ProjectDashboard ... />`（原有渲染逻辑保持）

### Step 4 — 互斥不变量执行

- `enterGlobal()`：`setMode('global')` + `setActiveProjectId(null)`，写入 localStorage
- `enterProject(projectId)`：`setMode('project')` + `setActiveProjectId(projectId)`，写入 localStorage
- 不允许任何其他路径修改这两个 state（强制走 enterGlobal/enterProject 函数）

### Step 5 — 零回归验证

- Project mode 下，ProjectDashboard 的所有 Phase 1/2 视图（overview/inbox/workitems/artifacts）行为完全不变
- 现有 Contact 列表点击 project channel 的行为：等价于 `enterProject(projectId)`，但保持原有点击机制

## Validation (verifier gate)

按 Phase 2 模式：

```bash
cd ui && npx tsc --noEmit       # 必须零错误
cd ui && pnpm build              # 必须 PASS
cd ui && pnpm dev                # 浏览器 smoke test
```

浏览器 smoke test 必须覆盖：

1. 初次打开应用（清空 localStorage）→ 默认进入 global mode，看到所有 project 一览
2. 点击 global 视图中某个 project 行 → 进入 project mode，看到 ProjectDashboard
3. 点击面包屑 `全局` → 返回 global mode，state 干净
4. Project mode 下切换不同 project（通过 Contact 列表）→ ProjectDashboard 重新渲染，数据切换正确
5. 关闭再打开应用 → 恢复上次的 mode + project（如果是 project mode，且 projectId 仍存在）
6. **Stale fallback**：手动在 localStorage 写入不存在的 projectId 再打开应用 → 降级回 global mode，无白屏
7. **回归**：进入 project mode 后切换 inbox/workitems/artifacts tab → 行为与 Phase 2 完全一致
8. **回归**：overview tab 的 Slice B artifact panel 仍渲染正确

## Delivery Required

完成后：

1. 创建 1 个或 2 个 commit（按你的判断；推荐 1 个 commit 涵盖整个 slice）
2. 在 `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-delivery-v1.md` 写交付文档
3. 通过 tmux 用标准格式回报：
   - completed
   - validation
   - blockers
   - commit hash
   - artifact path(s)

## Failure Rule

如果发现本 slice 无法在不动 ProjectDashboard 的前提下完成，**停下来**报告具体冲突，不要扩大 scope。

如果你认为 §B 的选项 Y（独立 Zustand store）比选项 X 更合适，可以采用，但要在 delivery 中说明 ≥3 行理由。

## Reporting Format

```text
[Nimbus -> Aegis] app-v2 Supervisor Context Mode (chan-03) Delivery
branch:
- track/infra-foundation
commit:
- <new commit hash>
completed:
- types.ts: SupervisorContextMode + GlobalProjectSummary added
- mock-data.ts: MOCK_GLOBAL_SUMMARY added (N projects)
- GlobalDashboard.tsx: created
- SupervisorPanel.tsx: context mode state + breadcrumb + conditional render
- localStorage persistence + stale fallback
state location choice:
- option X (local useState) / option Y (Zustand store) — and reason if Y
validation:
- cd ui && npx tsc --noEmit => ...
- cd ui && pnpm build => ...
- browser smoke test (8 scenarios) => ...
blockers:
- none OR ...
next action:
- wait for Flux verify-only re-verification
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-delivery-v1.md
```
