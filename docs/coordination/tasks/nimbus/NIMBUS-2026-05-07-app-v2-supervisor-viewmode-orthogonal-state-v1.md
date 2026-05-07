# Task: app-v2 Supervisor viewMode Orthogonal State (chan-09)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-08 |
| depends_on | `docs/architecture-decisions.md` AD-013 v2 §7, `docs/architecture-design.md` §7.1 supervisorStore (updated), `docs/coordination/reviews/2026-05-07-aegis-supervisor-viewmode-orthogonal-state-design.md`, commits `f5b8423`, `f579c25`, branch HEAD at AD-013 v2 commit |
| tags | nimbus, ui, app-v2, supervisor, viewmode, breadcrumb, AD-013-v2, chan-09, pinned-commit |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Pause Postgres / Rust infra work. |

## Cross-Track Note

Same as chan-03: Nimbus 主轨为 Rust + Postgres infra，本任务跨入 v2 前端（TypeScript / React）。本任务建立在你刚交付的 chan-03 实现之上（commit `f5b8423`），是它的扩展，不是替换。

## Base Commit

Work from the latest `track/infra-foundation` HEAD that contains AD-013 v2 (will be the AD revision commit issued together with this packet).

If local HEAD is older than that commit, stop and report drift.

## Objective

实现 AD-013 v2 §7 定义的 viewMode 正交状态：

1. 引入 `viewMode: 'dashboard' | 'chat'` 与 `contextMode` 正交
2. 重写面包屑：4 种渲染情况（详见 AD-013 v2 §7 决策表）
3. 重写 `switchContact`：按 contact.type 分支（seat → 仅切 viewMode，不动 contextMode；project-channel → 走 enterProject 路径）
4. 面包屑中间层 `<projectName>` 变成可点击 link（带视觉 affordance）
5. 中间层点击同步 `activeContactId` 到该 project 的 channel contact
6. localStorage 增加 `seatloom.supervisor.viewMode` key，含 stale fallback
7. **零回归**：chan-03 已验收的 8 个 smoke 场景全部兼容

## Scope Boundary

### In scope

- `ui/src/app-v2/types.ts` — 新增 `SupervisorViewMode` 类型
- `ui/src/app-v2/panel/SupervisorPanel.tsx`：
  - 新增 `viewMode` state + localStorage 持久化 + stale fallback
  - 重写 `switchContact` 分支逻辑（去掉 v1 中点击 seat 自动同步 contextMode 的行为）
  - 重写面包屑渲染（4 种情况，含中间层可点击 link 和视觉 affordance）
  - 重写右侧 pane 渲染分支（dashboard / chat 二分）
- 可选：`ui/src/app-v2/mock-data.ts` — 仅在确认每个 project 都有对应 channel contact 时不需要修改

### Explicitly out of scope

- 不修改 `ProjectDashboard.tsx`（项目视图保持现状）
- 不修改 `GlobalDashboard.tsx`（全局视图保持现状）
- 不修改任何 `views/` 文件
- 不修改 `dashboard/` 下其他 Section 文件
- 不修改 `useDataStore.ts`
- 不修改后端 / Postgres / infra
- 不动 styles/tokens.css
- 不引入新第三方依赖

## Required Read Order

1. `docs/architecture-decisions.md` AD-013 v2 §7（决策表 + 触发器表 + 不变量）
2. `docs/architecture-design.md` §7.1 supervisorStore（更新后的接口）
3. `docs/coordination/reviews/2026-05-07-aegis-supervisor-viewmode-orthogonal-state-design.md`（设计上下文）
4. 本文件
5. `ui/src/app-v2/panel/SupervisorPanel.tsx` 当前实现（你刚交付的 chan-03）

## §A — chan-03 → chan-09 行为变化总结

| 路径 | chan-03 行为 | chan-09 新行为 |
|------|-------------|---------------|
| 点 contact 列表中的 seat | 自动同步 `contextMode='project'` + `activeProjectId=seat.projectId`，右侧 pane 渲染 chat | **不动** contextMode；`viewMode='chat'`；右侧 pane 渲染 chat |
| 点 contact 列表中的 supervisor | viewMode 概念不存在；右侧 pane 渲染 chat（与 dashboard 共存的 conditional） | `viewMode='chat'`；右侧 pane 渲染 chat |
| 点 contact 列表中的 project channel | 自动同步 contextMode + activeProjectId；右侧 pane 渲染 ProjectDashboard | `viewMode='dashboard'` + `enterProject(channel.projectId)` + `setActiveContactId(channel.id)` |
| 看面包屑 | global mode 显示`全局`；project mode 显示`全局 › <项目名>` | dashboard mode 同 chan-03；chat-with-seat 显示三级 `全局 › <项目名> › <seat 名>`；chat-with-supervisor 显示一级 `全局` |
| 点面包屑 `全局` | enterGlobal | `viewMode='dashboard'` + enterGlobal |
| 点面包屑中间层 | 中间层不可点击 | **可点击** + 视觉 affordance；触发 `viewMode='dashboard'` + `enterProject(projectId)` + `setActiveContactId(<该 project 的 channel contact id>)` |

## §B — 面包屑 4 种渲染情况

```typescript
function renderBreadcrumb({ viewMode, contextMode, activeProjectId, activeContact }) {
  // Case 1: dashboard + global → 仅显示「全局」
  if (viewMode === 'dashboard' && contextMode === 'global') {
    return <GlobalLabel />;  // 非可点击文本
  }

  // Case 2: dashboard + project → 全局 › 项目名
  if (viewMode === 'dashboard' && contextMode === 'project') {
    return <>
      <GlobalButton onClick={enterGlobalAndSwitchToDashboard} />
      <Sep />
      <ProjectLabel name={lookup(activeProjectId)} />  {/* 当前位置，非可点击 */}
    </>;
  }

  // Case 3: chat + supervisor → 仅显示「全局」（不伸展）
  if (viewMode === 'chat' && activeContact.type === 'supervisor') {
    return <GlobalLabel />;  // 非可点击文本
  }

  // Case 4: chat + seat → 全局 › 项目名 › seat 名
  if (viewMode === 'chat' && activeContact.type === 'seat') {
    return <>
      <GlobalButton onClick={enterGlobalAndSwitchToDashboard} />
      <Sep />
      <ProjectLink
        name={lookup(activeContact.projectId)}
        onClick={() => enterProjectAndSwitchToDashboard(activeContact.projectId)}
      />  {/* 注意：可点击 */}
      <Sep />
      <SeatLabel name={activeContact.name} />  {/* 当前位置，非可点击 */}
    </>;
  }
}
```

注意 Case 2 vs Case 4 的关键差异：
- Case 2：中间层 `<projectName>` 是**当前位置**，非可点击
- Case 4：中间层 `<projectName>` 是**可点击 link**（点击切回 dashboard for that project）

## §C — 中间层可点击 link 的视觉 affordance

```css
默认态：
  color: var(--sl-text-secondary)
  font-weight: 500
  cursor: pointer

hover 态：
  color: var(--sl-brand)

active 态（pressed）：
  color: var(--sl-brand-hover) 或 var(--sl-brand) 暗一档
```

实现风格参考 chan-03 中已有的 `全局` button（line 333-342），保持视觉一致。

## §D — 触发器实现

实现 5 个 mutator 函数（建议命名）：

```typescript
// 已有，不变
const enterGlobal = () => {
  setContextMode('global');
  setActiveProjectIdState(null);
  setViewMode('dashboard');  // 新增：返回 global 一定是 dashboard
};

// 已有，不变
const enterProject = (projectId: string) => {
  setContextMode('project');
  setActiveProjectIdState(projectId);
  setViewMode('dashboard');  // 新增：进入 project 一定是 dashboard
};

// 新增：面包屑中间层点击触发
const enterProjectFromBreadcrumb = (projectId: string) => {
  enterProject(projectId);
  // 同步 activeContactId 到该 project 的 channel contact，避免 contact 列表与右侧 pane 视觉不一致
  const channelContact = MOCK_CONTACTS.find(c => c.type === 'project-channel' && c.projectId === projectId);
  if (channelContact) setActiveContactId(channelContact.id);
};

// 重写：switchContact 按 contact.type 分支
const switchContact = (id: string) => {
  // 保留现有的 draft 保存/恢复逻辑（chan-03 line 132-139）
  // ...
  setActiveContactId(id);
  const contact = MOCK_CONTACTS.find(c => c.id === id);
  if (!contact) return;
  if (contact.type === 'project-channel' && contact.projectId) {
    // project channel：走 dashboard 路径
    enterProject(contact.projectId);
    setViewMode('dashboard');
  } else {
    // seat 或 supervisor：仅切 viewMode 到 chat，不动 contextMode
    setViewMode('chat');
  }
};
```

注意 `switchContact` 中的 seat 分支：**不再**自动设置 `contextMode='project'`。这是 v2 唯一的破坏性行为变更。

## §E — localStorage stale fallback 规则

新增 key：`seatloom.supervisor.viewMode`

读取规则：

1. 缺失 → `'dashboard'`
2. 非 `'dashboard'` 也非 `'chat'` 字符串 → `'dashboard'`
3. **协调性 fallback**：如果 viewMode='chat' 但 activeContactId 找不到对应 contact，降级 viewMode 为 `'dashboard'`（可与 chan-03 §D 的 stale fallback 同时生效）

## §F — 6 个新 smoke 场景（人工层）

Mr. Zhang 验证（Flux 自动层先跑 tsc/build/scope/不变量静态检查）：

| # | 场景 | 通过判定 |
|---|---|---|
| 9 | 默认打开 → contact 列表点 Lyra（seat） | viewMode='chat'；面包屑 `全局 › SeatLoom 主项目 › Lyra`；contextMode 维持 global（DevTools 检查） |
| 10 | 与 Supervisor 聊天 | viewMode='chat'；面包屑 `全局`（不伸展） |
| 11 | chat-with-seat 状态下点面包屑中间层 `SeatLoom 主项目` | 切到 ProjectDashboard for p-1；contact 列表高亮自动切到 ch-p1；面包屑变 `全局 › SeatLoom 主项目` |
| 12 | chat-with-seat (p-1 Lyra) 状态下点面包屑 `全局` | viewMode='dashboard'，contextMode='global'，activeProjectId=null（DevTools 检查） |
| 13 | **跨 project 分叉态**：进 ProjectDashboard for p-1 → 点 contact 列表 p-2 的 seat (Iris) | viewMode='chat'；contextMode 仍是 'project'；activeProjectId 仍是 'p-1'；面包屑显示 `全局 › DataForge 数据平台 › Iris`（projectName 来自 seat.projectId='p-2'，与 activeProjectId 解耦） |
| 14 | chan-03 全部 8 场景回归 | 全部 PASS |

## Validation

```bash
cd ui && npx tsc --noEmit       # 必须零错误
cd ui && pnpm build              # 必须 PASS
cd ui && pnpm dev                # 浏览器 smoke test (Flux 自动 + Mr. Zhang 人工)
```

## Delivery Required

完成后：

1. 创建 1 个 commit
2. 在 `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-delivery-v1.md` 写交付文档
3. 通过 tmux 用标准格式回报 commit hash + validation + blockers

## Failure Rule

如果发现 §D 触发器有边界情况无法在不动 ProjectDashboard / GlobalDashboard 的前提下解决，停下来报告具体冲突，不要扩大 scope。

## Reporting Format

```text
[Nimbus -> Aegis] app-v2 Supervisor viewMode Orthogonal State (chan-09) Delivery
branch:
- track/infra-foundation
commit:
- <new commit hash>
completed:
- types.ts: SupervisorViewMode added
- panel/SupervisorPanel.tsx: viewMode state + breadcrumb 4-case + switchContact rewrite + middle-link
- localStorage seatloom.supervisor.viewMode key + stale fallback
validation:
- cd ui && npx tsc --noEmit => ...
- cd ui && pnpm build => ...
- chan-03 regression (8 scenarios static review) => ...
- new behavior (6 scenarios static trace) => ...
blockers:
- none OR ...
next action:
- wait for Flux verify-only + Mr. Zhang manual smoke
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-delivery-v1.md
```
