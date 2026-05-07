# Delivery: app-v2 Supervisor viewMode Orthogonal State (chan-09)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-05-07 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-v1.md`, `docs/architecture-decisions.md` AD-013 v2 §7, `docs/architecture-design.md` §7.1 |
| tags | nimbus, ui, app-v2, supervisor, viewmode, breadcrumb, AD-013-v2, chan-09 |
| owner | Nimbus |
| acceptance owner | Lyra |

## 1. Scope Restated

Implement AD-013 v2 §7 viewMode 正交状态：

- `SupervisorViewMode = 'dashboard' | 'chat'` 新类型
- `viewMode` state 与 `contextMode` 正交，localStorage 持久化 + stale fallback
- `switchContact` 重写：project-channel → dashboard 路径；seat/supervisor → 仅切 viewMode='chat'，不动 contextMode
- 面包屑 4-case 渲染（AD-013 §7 决策表）
- chat+seat 面包屑中间层变成可点击 link（§C affordance）
- `enterProjectFromBreadcrumb` 同步 activeContactId 到 project channel
- 零回归：chan-03 全部 8 个 smoke 场景兼容

## 2. Files Changed

### Modified Files

- `ui/src/app-v2/types.ts` — 新增 `SupervisorViewMode = 'dashboard' | 'chat'`
- `ui/src/app-v2/panel/SupervisorPanel.tsx` — viewMode state + breadcrumb 4-case + switchContact 重写 + middle-link

### Files Explicitly NOT Touched (per scope boundary)

- `ui/src/app-v2/dashboard/ProjectDashboard.tsx` — unchanged
- `ui/src/app-v2/dashboard/GlobalDashboard.tsx` — unchanged
- `ui/src/app-v2/mock-data.ts` — unchanged (每个 project 都有对应 channel contact，无需修改)
- `ui/src/app-v2/views/*.tsx` — unchanged
- `ui/src/stores/useDataStore.ts` — read-only（getState() for name resolution）
- Rust backend / Postgres / infra — untouched
- `ui/src/app-v2/styles/tokens.css` — untouched
- No new third-party dependencies

## 3. State Design

`viewMode` 作为 local `useState` in `SupervisorPanel`，与现有 `contextMode` / `activeProjectId` 并列，遵循 option X 模式（chan-03 §B）。

### 右侧 pane 决策表（实现）

| `viewMode` | 子条件 | 右侧 pane |
|---|---|---|
| `dashboard` | `contextMode === 'global'` | `GlobalDashboard` |
| `dashboard` | `contextMode === 'project'` | `ProjectDashboard` for `activeProjectId` |
| `chat` | 任意 | IM chat pane（按 `activeContact`） |

### 面包屑决策表（实现）

| `viewMode` | 子条件 | 面包屑 |
|---|---|---|
| `dashboard` | `contextMode === 'global'` | `全局`（非可点击） |
| `dashboard` | `contextMode === 'project'` | `全局 › <projectName(activeProjectId)>`（项目名非可点击） |
| `chat` | `activeContact.type === 'supervisor'` | `全局`（非可点击） |
| `chat` | `activeContact.type === 'seat'` | `全局 › <projectName(seat.projectId)> › <seatName>`（中间层可点击 link） |

## 4. Mutator 函数（§D）

```typescript
// 已有，更新：加入 setViewMode('dashboard')
enterGlobal() → contextMode='global', activeProjectId=null, viewMode='dashboard'

// 已有，更新：加入 setViewMode('dashboard')；移除原来的 auto-setActiveContactId
enterProject(projectId) → contextMode='project', activeProjectId=projectId, viewMode='dashboard'

// 新增：面包屑中间层点击 + GlobalDashboard.onSelectProject
enterProjectFromBreadcrumb(projectId) → enterProject(projectId) + setActiveContactId(channelContact.id)

// 重写：按 contact.type 分支
switchContact(id):
  - project-channel → enterProject(channel.projectId)  [sets viewMode='dashboard']
  - seat / supervisor → setViewMode('chat')  [不动 contextMode / activeProjectId]
```

**破坏性变更（v2 唯一）**：`switchContact` 在点击 seat 时不再自动同步 `contextMode='project'`。

## 5. localStorage Keys（§E）

| Key | Type | Notes |
|---|---|---|
| `seatloom.supervisor.viewMode` | `'dashboard' \| 'chat'` | 新增 — viewMode 切换时写入 |
| `seatloom.supervisor.contextMode` | `'global' \| 'project'` | 已有（chan-03），不改名 |
| `seatloom.supervisor.activeProjectId` | `string` (or absent) | 已有（chan-03），不改名 |
| `sl-supervisor-active-contact` / `sl-supervisor-pos` / `sl-supervisor-size` / `sl-supervisor-draft-*` | (existing) | 不变 |

## 6. Stale Fallback（§E）

`loadInitialContext()` 扩展：

```
contextMode / activeProjectId：chan-03 §D 规则不变
viewMode：
  缺失/非法字符串 → 'dashboard'
  'chat' 但 sl-supervisor-active-contact 找不到对应 contact → 降级 'dashboard'
```

## 7. 中间层可点击 link（§C）

```
默认态：color: var(--sl-text-secondary), fontWeight: 500, cursor: pointer
hover 态：color: var(--sl-brand)
```

与 `全局` 按钮视觉风格一致（无边框、无背景、hover 变色）。`maxWidth: 120` + `textOverflow: ellipsis` 防止溢出。

## 8. 零回归保证

- `ProjectDashboard` key 从 `activeContactId` 改为 `activeProjectId`（更精确：只有 project 切换才重置 tab 状态，contact 切换不会）
- `GlobalDashboard` 的 `onSelectProject` 改为调用 `enterProjectFromBreadcrumb`（保留 chan-03 场景 2 的 contact 同步行为）
- chan-03 的 8 个 smoke 场景全部兼容（见下静态追踪）

## 9. Validation Results

```
cd ui && npx tsc --noEmit
=> exit 0, zero errors (PASS)

cd ui && pnpm build
=> tsc + vite build PASS
=> dist/assets/index-*.js  353.04 kB │ gzip 102.35 kB
=> ✓ built in 1.20s (PASS)
```

### chan-03 回归静态追踪（8 场景）

| # | 场景 | chan-09 行为 | 兼容 |
|---|---|---|---|
| 1 | 冷启动（清 localStorage）| loadInitialContext → {global, null, dashboard} → GlobalDashboard | ✓ |
| 2 | GlobalDashboard 点 project 行 | onSelectProject → enterProjectFromBreadcrumb('p-1') → contextMode=project, viewMode=dashboard, ch-p1 高亮 | ✓ |
| 3 | 点面包屑「全局」| enterGlobal → global, null, dashboard | ✓ |
| 4 | 切换不同 project（contact list project-channel）| switchContact(ch-p2) → enterProject('p-2') → dashboard, p-2 | ✓ |
| 5 | reload 恢复 project mode | localStorage {mode=project, activeProjectId=p-1, viewMode=dashboard} → 恢复 | ✓ |
| 6 | stale fallback | {mode=project, activeProjectId=p-bogus} → p-bogus 不在 projects → 降级 global | ✓ |
| 7 | Tab 回归 | viewMode=dashboard → ProjectDashboard key=activeProjectId；内部 tab state 不变 | ✓ |
| 8 | Overview Slice B 回归 | ProjectDashboard untouched | ✓ |

### 新增 6 个 smoke 场景（静态追踪）

| # | 场景 | 静态追踪 |
|---|---|---|
| 9 | 默认打开 → contact 列表点 Lyra（seat） | switchContact('p1-lyra') → setViewMode('chat')；contextMode 维持 global；面包屑 `全局 › SeatLoom 协调核心 › Lyra` |
| 10 | 与 Supervisor 聊天 | switchContact('supervisor') → setViewMode('chat')；activeContact.type='supervisor'；面包屑 `全局`（不伸展） |
| 11 | chat-with-seat 状态下点面包屑中间层 | enterProjectFromBreadcrumb('p-1') → viewMode=dashboard, contextMode=project, activeContactId=ch-p1；面包屑变 `全局 › SeatLoom 协调核心` |
| 12 | chat-with-seat (p-1 Lyra) 点面包屑「全局」| enterGlobal → viewMode=dashboard, contextMode=global, activeProjectId=null |
| 13 | 跨 project 分叉态：ProjectDashboard p-1 → 点 Iris (p-2 seat) | switchContact('p2-iris') → viewMode='chat'；contextMode 仍 'project'；activeProjectId 仍 'p-1'；面包屑 `全局 › ThoughtOnly 叙事引擎 › Iris`（来自 seat.projectId='p-2'） |
| 14 | chan-03 全部 8 场景回归 | 见上表，全部 PASS |

### Browser smoke test（operator-required）

CLI-only seat 无法完成浏览器测试。上述 14 个场景已静态追踪，预期全部通过。Flux verify-only + Mr. Zhang 人工层请完成最终验证。

## 10. Branch and Commit

- Branch: `track/infra-foundation`
- Base commit: `f5b8423` (chan-03 delivery)
- Delivery commit: `cadf36e`

## 11. Reporting

```text
[Nimbus -> Aegis] app-v2 Supervisor viewMode Orthogonal State (chan-09) Delivery
branch:
- track/infra-foundation
commit:
- cadf36e
completed:
- types.ts: SupervisorViewMode = 'dashboard' | 'chat' added
- panel/SupervisorPanel.tsx: viewMode state + breadcrumb 4-case + switchContact rewrite + middle-link (§C affordance)
- localStorage seatloom.supervisor.viewMode key + stale fallback (§E)
- enterGlobal/enterProject: now also set viewMode='dashboard'
- enterProjectFromBreadcrumb: new mutator (enterProject + sync activeContactId to channel); used by GlobalDashboard.onSelectProject + breadcrumb middle click
- switchContact: project-channel → dashboard path; seat/supervisor → chat only, contextMode/activeProjectId untouched (v2 breaking change)
validation:
- cd ui && npx tsc --noEmit => PASS (zero errors)
- cd ui && pnpm build => PASS (tsc + vite build green; 353.04 kB / 102.35 kB gzip)
- chan-03 regression (8 scenarios static review) => all PASS
- new behavior (6 scenarios static trace) => all PASS
blockers:
- none — Docker/browser unavailable on this seat; Flux to perform browser smoke test + Mr. Zhang manual smoke
next action:
- wait for Flux verify-only + Mr. Zhang manual smoke (6 new scenarios + chan-03 regression)
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-delivery-v1.md
```
