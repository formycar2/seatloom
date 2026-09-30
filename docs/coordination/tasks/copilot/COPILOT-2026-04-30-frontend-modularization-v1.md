# Task: v2 前端模块化拆分

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | COPILOT-2026-04-30-frontend-modularization-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-30 |
| to | copilot |
| priority | P0 |
| owner | Copilot |
| acceptance owner | Aegis |

## 背景

`ui/src/app-v2/AppV2.tsx` 当前有 **2093 行**，包含了 v2 前端的全部内容：类型定义、Mock 数据、所有组件逻辑、所有 UI 渲染。这造成了两个问题：

1. **文件过大**：Agent 每次修改任何功能都必须读完整个文件，效率极低
2. **职责混乱**：Mock 数据和组件混在一起，没有清晰的模块边界

本任务的目标是将 `AppV2.tsx` 拆分为多个职责单一的文件，**零行为变更**。

## 核心约束

- **零行为变更**：不修改任何组件的功能、样式、交互逻辑，只做物理拆分和 import 重组
- **不引入新库**：不引入任何第三方依赖
- **不改变 tokens**：不修改 `ui/src/app-v2/styles/tokens.css`
- **不改变 DAG 相关文件**：`dag-model.ts` 和 `DagWorkflow.tsx` 已经是独立文件，不需要动
- **验证必须通过**：完成后执行 `cd ui && npx tsc --noEmit`，必须零错误

## 当前文件结构

```
ui/src/app-v2/
├── AppV2.tsx           ← 2093 行，所有东西都在这里（需要拆分）
├── dag-model.ts        ← 已独立（不动）
├── DagWorkflow.tsx     ← 已独立（不动）
└── styles/
    └── tokens.css      ← CSS tokens（不动）
```

## 目标文件结构

```
ui/src/app-v2/
├── AppV2.tsx                          ← 精简为 ~80 行，只负责根组件和键盘快捷键
├── dag-model.ts                       ← 不动
├── DagWorkflow.tsx                    ← 不动
├── styles/
│   └── tokens.css                    ← 不动
│
├── types.ts                          ← 所有类型定义
├── mock-data.ts                       ← 所有 Mock 数据
│
├── components/
│   ├── Avatar.tsx                    ← Avatar、StatusDot、SeatLoomLogo
│   ├── MessageBubble.tsx             ← TYPE_BADGES、MessageBubble
│   ├── ContactRow.tsx                ← ContactRow
│   └── ChatInput.tsx                 ← ChatInput（含 routing governance 逻辑）
│
├── panel/
│   └── SupervisorPanel.tsx           ← SupervisorPanel（可拖拽浮层，含联系人列表和对话区）
│
└── dashboard/
    ├── ProjectDashboard.tsx          ← 主容器（数据加载、状态、渲染顺序）
    ├── BlockersSection.tsx           ← 阻塞项区块
    ├── ActiveWorkSection.tsx         ← 正在发生区块
    ├── TimelineSection.tsx           ← 时间线区块
    ├── StageProgressSection.tsx      ← 当前阶段进度区块
    └── GoalsSection.tsx              ← 目标和阶段区块（可折叠）
```

## 详细迁移说明

### 1. `types.ts` — 类型定义

**从 `AppV2.tsx` 中提取** 以下 interface/type，**原封不动**地移到这个文件：

```typescript
// types.ts 顶部注释：
// 类型定义：v2 前端使用的所有数据类型
// 包括：ChatContact, ChatMessage, PlanPhase, TimelineEntry, ProjectChannelData
```

要迁移的类型：
- `ChatContact`（包含 `seatType` 字段：`'po' | 'worker' | 'verifier'`）
- `ChatMessage`（包含 `type` 字段和 `card` 结构）
- `PlanPhase`
- `TimelineEntry`
- `ProjectChannelData`（包含嵌套的 `goals`、`currentGoal`、`currentStage`、`justNow`、`earlierToday`、`yesterday`）

这些类型要从 `AppV2.tsx` 删除，在 `types.ts` 中 `export`，其他文件通过 `import { ... } from '../types'` 引用。

---

### 2. `mock-data.ts` — Mock 数据

**从 `AppV2.tsx` 中提取** 以下常量，**原封不动**地移到这个文件：

```typescript
// mock-data.ts 顶部注释：
// Mock 数据：用于 v2 前端开发阶段的静态数据
// 包括：MOCK_CONTACTS（联系人列表）、MOCK_MESSAGES（每个联系人的消息）、MOCK_CHANNEL_DATA（项目频道工作流数据）
```

要迁移的常量：
- `MOCK_CONTACTS: ChatContact[]`（约 16 个联系人，含 3 个项目）
- `MOCK_MESSAGES: Record<string, ChatMessage[]>`（每个 contactId 的消息）
- `MOCK_CHANNEL_DATA: Record<string, ProjectChannelData>`（3 个项目频道的工作流数据）

这些常量要从 `AppV2.tsx` 删除，在 `mock-data.ts` 中 `export`，其他文件通过 `import { ... } from '../mock-data'` 引用（相对路径根据调用方位置调整）。

---

### 3. `components/Avatar.tsx`

```typescript
// Avatar.tsx 顶部注释：
// 通用小组件：SeatLoomLogo（Logo SVG）、StatusDot（带颜色点和标签的状态指示器）、Avatar（单字母圆形头像，带在线状态）
```

从 `AppV2.tsx` 迁移：
- `SeatLoomLogo` 组件
- `StatusDot` 组件
- `Avatar` 组件

全部 `export`（具名导出）。

---

### 4. `components/MessageBubble.tsx`

```typescript
// MessageBubble.tsx 顶部注释：
// 消息气泡：渲染单条聊天消息，支持 text / delivery / verification / gate / blocker / progress / alert / info 等卡片类型
```

从 `AppV2.tsx` 迁移：
- `TYPE_BADGES` 常量（消息类型的标签和颜色映射）
- `MessageBubble` 组件

`MessageBubble` 具名导出。

---

### 5. `components/ContactRow.tsx`

```typescript
// ContactRow.tsx 顶部注释：
// 联系人列表行：渲染单个联系人条目（头像、名字、角色、未读数、最新消息预览）
```

从 `AppV2.tsx` 迁移：
- `ContactRow` 组件（依赖 `Avatar`，通过 import 引用）

具名导出。

---

### 6. `components/ChatInput.tsx`

```typescript
// ChatInput.tsx 顶部注释：
// 消息输入框：带 routing governance 逻辑（向 worker/verifier 直发时弹出路由守卫对话框，建议通过 PO 中转）
```

从 `AppV2.tsx` 迁移：
- `ChatInput` 组件（含 `showRoutingGuard` 状态、PO 查找、路由守卫弹窗的完整逻辑）

具名导出。

---

### 7. `panel/SupervisorPanel.tsx`

```typescript
// SupervisorPanel.tsx 顶部注释：
// Supervisor 浮层面板：可拖拽、可调整大小的 IM 风格浮窗，包含左侧联系人列表和右侧对话区（支持项目频道和 seat 对话两种视图）
// 状态通过 localStorage 持久化：面板位置、尺寸、当前联系人、各联系人草稿
```

从 `AppV2.tsx` 迁移：
- `STORAGE_KEY_POS`、`STORAGE_KEY_SIZE` 常量
- `loadSaved()` 函数
- `SupervisorPanel` 组件（含拖拽逻辑、resize 逻辑、联系人列表渲染、对话区渲染、`ProjectDashboard` 嵌入、`ChatInput` 嵌入）

具名导出。

---

### 8. `dashboard/` — ProjectDashboard 拆分

`ProjectDashboard` 目前约 **900 行**（`AppV2.tsx` 第 577–1479 行），是最大的单个组件，需要拆分为 6 个文件。

#### 8a. `dashboard/ProjectDashboard.tsx`（主容器）

```typescript
// ProjectDashboard.tsx 顶部注释：
// 项目频道主视图：根据 channelId 加载频道数据，管理 UI 状态（展开/折叠），按优先级顺序渲染各区块
// 渲染顺序（从细节到整体）：BlockersSection → ActiveWorkSection → DagWorkflow → 下一步 → TimelineSection → StageProgressSection → GoalsSection
```

负责：
- `MOCK_CHANNEL_DATA[channelId]` 数据加载
- `useDataStore()` 混合数据源
- scroll lock 逻辑（`scrollLockRef`、`scrollTimerRef`、`useEffect`）
- `getSeatName()` helper
- `formatClock()`、`formatSince()`、`formatInboxObjectRef()` helper
- 顶层滚动容器和区块渲染顺序
- 将数据作为 props 传给各 Section 组件

#### 8b. `dashboard/BlockersSection.tsx`

```typescript
// BlockersSection.tsx 顶部注释：
// 阻塞项区块：显示当前阶段的阻塞工作项（红色高亮），没有阻塞时不渲染
```

Props：
```typescript
interface BlockersSectionProps {
  blockers: { text: string; owner: string; since: string }[];
  formatSince: (iso: string) => string;
}
```

负责渲染阻塞项列表（红色背景区块、每项 owner+since）。无阻塞时返回 `null`。

#### 8c. `dashboard/ActiveWorkSection.tsx`

```typescript
// ActiveWorkSection.tsx 顶部注释：
// 正在发生区块：显示当前 DAG 中状态为 active 的工作节点及其上下游依赖关系
```

Props：
```typescript
interface ActiveWorkSectionProps {
  workflow: StageWorkflow;
  formatSince: (iso: string) => string;
}
```

负责渲染活跃节点卡片和上下游依赖视图。

#### 8d. `dashboard/TimelineSection.tsx`

```typescript
// TimelineSection.tsx 顶部注释：
// 时间线区块：渲染最近发生的事件（展开）和更早的事件（折叠/单行摘要）
```

Props：
```typescript
interface TimelineSectionProps {
  justNow: TimelineEntry[];
  earlierToday: string;
  yesterday: string;
  showEarlier: boolean;
  onToggleEarlier: () => void;
  formatClock: (iso: string) => string;
}
```

#### 8e. `dashboard/StageProgressSection.tsx`

```typescript
// StageProgressSection.tsx 顶部注释：
// 当前阶段进度区块：显示阶段名称、完成工作项数量和进度条
```

Props：
```typescript
interface StageProgressSectionProps {
  stageName: string;
  workItemsDone: number;
  workItemsTotal: number;
}
```

#### 8f. `dashboard/GoalsSection.tsx`

```typescript
// GoalsSection.tsx 顶部注释：
// 目标和阶段区块（默认折叠）：显示项目级目标列表和当前目标下的阶段列表，提供展开/折叠按钮
```

Props：
```typescript
interface GoalsSectionProps {
  goals: ProjectChannelData['goals'];
  currentGoal: ProjectChannelData['currentGoal'];
}
```

---

### 9. `AppV2.tsx` — 精简后的根组件

拆分完成后，`AppV2.tsx` 应该只剩下：

```typescript
// AppV2.tsx 顶部注释：
// v2 前端入口：负责全局键盘快捷键（⌘K 唤出 Supervisor 面板）和面板开关状态持久化
```

包含：
- `AppV2` 根组件（约 80 行）
- `⌘K` 键盘监听逻辑
- `showSupervisor` 状态（含 localStorage 持久化）
- 顶部 Bar 渲染（SeatLoomLogo、"打开 Supervisor" 按钮）
- `SupervisorPanel` 条件渲染

---

## Import 路径规则

各文件的 import 关系（用相对路径）：

```
AppV2.tsx
  → ./styles/tokens.css
  → ./components/Avatar (SeatLoomLogo)
  → ./panel/SupervisorPanel

SupervisorPanel.tsx
  → ../types
  → ../mock-data
  → ../components/Avatar
  → ../components/ContactRow
  → ../components/MessageBubble
  → ../components/ChatInput
  → ../dashboard/ProjectDashboard
  → ../../stores/useDataStore （如果用到）

ProjectDashboard.tsx
  → ../types
  → ../mock-data
  → ../../stores/useDataStore
  → ../../dag-model（注意：dag-model 在 app-v2/ 根目录，不在 dashboard/ 下）
  → ../../DagWorkflow
  → ./BlockersSection
  → ./ActiveWorkSection
  → ./TimelineSection
  → ./StageProgressSection
  → ./GoalsSection

各 Section 组件只 import types 和必要的 React，不相互依赖。
```

> **注意**：`dag-model.ts` 和 `DagWorkflow.tsx` 在 `ui/src/app-v2/` 根目录，`dashboard/` 的组件引用它们时要用 `../../dag-model` 和 `../../DagWorkflow`（相对于 `dashboard/` 文件夹的两级上行）。实际上 `dag-model.ts` 是 `ui/src/app-v2/dag-model.ts`，而 `dashboard/` 是 `ui/src/app-v2/dashboard/`，所以从 `dashboard/` 引用是 `../dag-model`（一级上行即可）。请在写 import 时仔细计算路径层级。

---

## 执行步骤

1. 先读取 `ui/src/app-v2/AppV2.tsx` 全文（2093 行）
2. 按照上方说明，**逐文件创建**新文件，将对应代码段粘贴进去并添加导出
3. 每创建一个新文件，同时从 `AppV2.tsx` 删除对应代码段，添加对应 import
4. 最后检查 `AppV2.tsx` 是否已精简至约 80 行
5. 执行 `cd ui && npx tsc --noEmit` 验证，修复所有类型错误

---

## 文件头注释格式

每个新文件必须以如下格式开头（单行注释，说明文件职责）：

```typescript
/**
 * [文件名]
 * [一句话说明：这个文件是什么，包含哪些组件/常量/类型，供谁使用]
 */
```

---

## Done Definition

- [ ] `types.ts` 创建，包含全部类型定义，均已导出
- [ ] `mock-data.ts` 创建，包含三个 Mock 数据常量，均已导出
- [ ] `components/Avatar.tsx` 创建，包含 `SeatLoomLogo`、`StatusDot`、`Avatar`
- [ ] `components/MessageBubble.tsx` 创建，包含 `TYPE_BADGES`、`MessageBubble`
- [ ] `components/ContactRow.tsx` 创建，包含 `ContactRow`
- [ ] `components/ChatInput.tsx` 创建，包含 `ChatInput`（含 routing governance）
- [ ] `panel/SupervisorPanel.tsx` 创建，包含完整的 `SupervisorPanel`
- [ ] `dashboard/ProjectDashboard.tsx` 创建，主容器
- [ ] `dashboard/BlockersSection.tsx` 创建
- [ ] `dashboard/ActiveWorkSection.tsx` 创建
- [ ] `dashboard/TimelineSection.tsx` 创建
- [ ] `dashboard/StageProgressSection.tsx` 创建
- [ ] `dashboard/GoalsSection.tsx` 创建
- [ ] `AppV2.tsx` 精简至 ~80 行
- [ ] `npx tsc --noEmit` 零错误
- [ ] 浏览器中行为与拆分前完全一致
