# Design Proposal: viewMode Orthogonal State for Supervisor Panel (chan-03 follow-up)

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | AEGIS-2026-05-07-supervisor-viewmode-orthogonal-state-v1 |
| status | issued (review pending) |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| reviewer | Lyra |
| basis | `docs/architecture-decisions.md` AD-013, `docs/coordination/tasks/aegis/AEGIS-2026-04-30-pending-changes-register.md` chan-03, Mr. Zhang 2026-05-07 design discussion |
| tags | design, supervisor, breadcrumb, viewmode, AD-013-extension, chan-03b |

## Origin

Mr. Zhang on 2026-05-07 raised a design point: **在发送消息模式下，面包屑是否应该显示 `全局 › 项目名`？**

当前实现（chan-03 验收 commit `f5b8423`）的行为：

| 用户在做什么 | `contextMode` | 面包屑 | 是否合理 |
|---|---|---|---|
| 看 GlobalDashboard | `global` | `全局` | ✓ |
| 看某 project dashboard | `project` | `全局 › <项目名>` | ✓ |
| **给某 seat 发消息（chat）** | `project`（自动同步） | `全局 › <项目名>` | ⚠️ 不完整 |

Mr. Zhang 的方向：**面包屑应反映右侧 pane 内容，而非 dashboard context state。** Chat with seat 时面包屑应是 `全局 › <项目名> › <seat 名>`，chat with Supervisor 时应是 `全局`。

## Root Issue

AD-013 把 `contextMode` 设计成同时承担两个职责：

1. **数据隔离**：决定 dashboard 渲染哪个数据集（global summary vs single project data）— **AD-013 的原始意图**
2. **右侧 pane 路由**：决定渲染 GlobalDashboard 还是 ProjectDashboard — **chan-03 实现时引入的耦合**

这两个职责在「dashboard-only」场景下重合（看全局 → 看 global summary；看 project → 看 project dashboard），所以 chan-03 实现把它们合并成一个状态字段没有出问题。

但当用户在右侧 pane 与某个 seat **chat** 时，dashboard 数据隔离仍然有意义（消息发出时归属哪个 project），但右侧 pane 既不是 GlobalDashboard 也不是 ProjectDashboard——是个完全独立的第三种 pane。这时强行把 `contextMode` 当作右侧 pane 路由器就出现歧义。

## Proposal

引入与 `contextMode` **正交**的 `viewMode` 状态：

```typescript
type SupervisorContextMode = 'global' | 'project';   // 不变（AD-013 §6 不变量保留）
type SupervisorViewMode    = 'dashboard' | 'chat';   // 新增

interface SupervisorState {
  contextMode: SupervisorContextMode;        // dashboard 数据隔离（AD-013）
  activeProjectId: string | null;            // dashboard 互斥不变量（AD-013）
  viewMode: SupervisorViewMode;              // 右侧 pane 路由（new）
  activeContactId: string;                   // 现有
}
```

### 右侧 pane 决策表

| `viewMode` | 关键条件 | 右侧 pane 渲染 |
|---|---|---|
| `dashboard` | `contextMode === 'global'` | GlobalDashboard |
| `dashboard` | `contextMode === 'project'` | ProjectDashboard for `activeProjectId` |
| `chat` | `activeContact.type === 'supervisor'` | Chat (Supervisor) |
| `chat` | `activeContact.type === 'seat'` | Chat (Seat) |

### 面包屑决策表

| `viewMode` | 子条件 | 面包屑 |
|---|---|---|
| `dashboard` | `contextMode === 'global'` | `全局` |
| `dashboard` | `contextMode === 'project'` | `全局 › <projectName(activeProjectId)>` |
| `chat` | `activeContact.type === 'supervisor'` | `全局` |
| `chat` | `activeContact.type === 'seat'` | `全局 › <projectName(seat.projectId)> › <seatName>` |

注意：在 chat-with-seat 模式下，面包屑显示的 `projectName` 来自 **seat contact 的 `projectId` 字段**，不依赖 `activeProjectId`。这意味着：

- 用户可以在 ProjectDashboard for p-1 状态下点击 p-2 的某个 seat 发消息
- 面包屑会显示 `全局 › <p-2 name> › <seat name>`，但底层 `contextMode` 仍是 `'project'` + `activeProjectId='p-1'`
- 这不破坏 AD-013 不变量（contextMode 只描述 dashboard 数据隔离，与 chat 路由无关）

### 点击触发器

| 用户动作 | 副作用 |
|---|---|
| 点面包屑的 `全局` | `viewMode='dashboard'` + `enterGlobal()` |
| 点面包屑中间层 `<projectName>` | `viewMode='dashboard'` + `enterProject(projectId)` |
| 点 contact 列表中的 project channel | `viewMode='dashboard'` + `enterProject(contact.projectId)` + `setActiveContactId(contact.id)` |
| 点 contact 列表中的 seat | `viewMode='chat'` + `setActiveContactId(seat.id)`；**不动** `contextMode` 与 `activeProjectId` |
| 点 contact 列表中的 supervisor | `viewMode='chat'` + `setActiveContactId('supervisor')`；**不动** `contextMode` |

## AD-013 状态机变化

### 不变量保留

- `contextMode === 'global'` ⇔ `activeProjectId === null`（**严格保留**）
- 只有 `enterGlobal()` 和 `enterProject(projectId)` 能修改 `contextMode` / `activeProjectId`（**严格保留**）
- Person-Supervisor 1:1 绑定（**保留**）
- 切换是显式用户动作（**保留**）
- 数据隔离规则：Global 不渲染单 project 详情，Project 不渲染其他 project 数据（**保留**）

### 新增不变量

- `viewMode` 是与 `contextMode` 正交的独立状态字段
- `viewMode` 由 contact 列表点击驱动（点 project channel → dashboard，点 seat/supervisor → chat），由面包屑点击重置为 dashboard
- `viewMode === 'chat'` 时，dashboard 数据隔离不被使用（但 contextMode 仍维持一个"上次 dashboard 状态"，便于点击面包屑回到 dashboard 时不跳变）

### Project 入口变化

当前 chan-03 实现的 `switchContact`（line 141-148）：在点击 seat contact 时**自动**把 `contextMode` 设为 `'project'` + 写入 `activeProjectId`。

新设计：`switchContact` 在点击 seat 时**仅**设置 `viewMode='chat'` + `setActiveContactId(seat.id)`，**不动** `contextMode`。Dashboard 数据隔离与 chat 路由解耦。

## 行为变化对照

| 用户路径 | 当前面包屑（chan-03） | 新面包屑（viewMode） |
|---|---|---|
| 默认打开（清空 localStorage） | `全局` | `全局` |
| 选项目 channel ch-p1 | `全局 › SeatLoom 主项目` | `全局 › SeatLoom 主项目` |
| 在 ch-p1 dashboard 上点 Lyra（seat） | `全局 › SeatLoom 主项目` | `全局 › SeatLoom 主项目 › Lyra` |
| 直接从 contact 列表点 Lyra | `全局 › SeatLoom 主项目` | `全局 › SeatLoom 主项目 › Lyra` |
| 与 Supervisor 聊天 | `全局`（或残留的 `全局 › <project>`，不一致） | `全局` |
| 在 chat with Lyra 状态下点面包屑 `全局` | 无三级面包屑可点中间层 | 同样回 GlobalDashboard |
| 在 chat with Lyra 状态下点中间层 `SeatLoom 主项目` | 无此入口 | 进 SeatLoom 的 ProjectDashboard |

## 是否破坏 chan-03 已验收行为

不破坏。8 个 chan-03 smoke 场景兼容性分析：

| # | chan-03 场景 | viewMode 设计下行为 | 兼容 |
|---|---|---|---|
| 1 | 清 localStorage 默认 global | viewMode 默认 dashboard，contextMode 默认 global → 渲染 GlobalDashboard | ✓ |
| 2 | global 中点 project 行 | enterProject + viewMode='dashboard' → ProjectDashboard | ✓ |
| 3 | 点面包屑 `全局` | enterGlobal + viewMode='dashboard' | ✓ |
| 4 | 切换不同 project | enterProject(B) + viewMode 保持 dashboard | ✓ |
| 5 | 关闭重开恢复 | localStorage 多读一个 viewMode key | ✓（向后兼容：缺失视为 dashboard） |
| 6 | Stale fallback | 同样规则降级回 global mode | ✓ |
| 7 | Tab 回归 | viewMode='dashboard' 进 ProjectDashboard，tab 行为不变 | ✓ |
| 8 | Slice B artifact panel 回归 | 同 7 | ✓ |

## 工作量与风险评估

**Aegis（设计）**：
- AD-013 修订（新增 §7 viewMode 正交状态部分），约 60 行
- 决策表 + 触发器表 + 不变量更新

**Nimbus（实现）**：
- `types.ts`：新增 `SupervisorViewMode` 类型
- `panel/SupervisorPanel.tsx`：
  - 新增 `viewMode` state + localStorage key
  - 重写 `switchContact` 路由分支（去掉自动 contextMode 切换）
  - 重写面包屑渲染逻辑（4 种情况）
  - 新增中间层 `<projectName>` 的 click handler（调用 `enterProject` 并切回 dashboard）
- `mock-data.ts`：无需修改
- 估计 1 个 commit，约 80 行净增

**Flux verify-only（自动层）**：
- tsc / pnpm build / scope check / 不变量静态检查

**Mr. Zhang 人工层（新 smoke 场景）**：
1. chat with seat → 面包屑显示三级
2. chat with supervisor → 面包屑显示一级
3. chat 状态下点面包屑中间层 → 进 ProjectDashboard
4. chat with seat (project A) 状态下点面包屑 `全局` → 回 GlobalDashboard，contextMode 与 activeProjectId 同步重置
5. 在 ProjectDashboard for p-1 状态下点 contact 列表 p-2 的 seat → 面包屑切到 p-2 链路，但回退面包屑能正确选择目标
6. 全部 chan-03 8 场景的回归检查

## Open questions for Lyra

请 Lyra 在 review 时确认或质询：

1. **viewMode 的命名**：`'dashboard'` vs `'chat'` 是否清晰？候选：`'state-view'` vs `'message-view'`、`'context-view'` vs `'conversation'`
2. **Stale fallback 扩展**：localStorage 中 `seatloom.supervisor.viewMode` 缺失或非法值是否一律降级为 `'dashboard'`？
3. **跨 project chat 的歧义**：用户从 ProjectDashboard for p-1 状态点击 p-2 的 seat（让 dashboard 维持 p-1，chat 进入 p-2 上下文）— 这种"分叉态"是否符合产品意图？还是应该强制 dashboard 跟随 chat？
4. **面包屑中间层是否带 affordance**：当前 chan-03 实现中间层不可点击（只是文本），新设计中间层在 chat mode 下变成 active link。这个交互升级是否需要在视觉上明确（如下划线、hover 颜色）？
5. **chan 编号**：本 follow-up 是叫 `chan-03b` 还是独立 `chan-XX`？影响 pending changes register 的归档。

## Decision needed before next step

- ✅ Lyra review viewMode 设计（含上述 5 个 open questions 的处置）
- ⏸ Aegis 修订 AD-013（在 Lyra 确认 viewMode 设计后）
- ⏸ Aegis 发 NIMBUS 包
- ⏸ Nimbus 实现
- ⏸ Flux + Mr. Zhang verify
- ⏸ Lyra 验收 + chan 关闭

---

*Aegis · 2026-05-07 · status: review pending*
