# SeatLoom Interaction Spec v1.0

| 项目 | 内容 |
|------|------|
| 文档 | Interaction Spec v1.0 |
| 状态 | Draft |
| 更新时间 | 2026-04-27 |
| 依赖文档 | PRD v0.4、MVP Scenarios v2.0、UX Spec v1.0 |

---

## 1. 目标

将关键用户流程定义为“可实现、可测试”的交互条款，避免仅有布局而无行为合同。

---

## 2. 全局交互规则

### 2.1 焦点流

- `Tab/Shift+Tab`：Sidebar -> Main -> Detail -> Terminal 循环
- `Esc`：关闭 Detail 或退出当前弹窗
- `Enter`：打开当前选中项

### 2.2 列表规则

- 列表支持键盘上下移动
- 列表项必须支持点击打开详情
- 长列表至少支持分页或懒加载其一

### 2.3 状态反馈

- 所有异步动作必须反馈 `pending/success/error`
- 错误反馈包含“原因 + 下一步建议”

---

## 3. 关键流程定义

## 3.0 多项目切换

触发：

- 标题栏 Project Switcher
- `Cmd/Ctrl+K` 打开项目切换器
- `All Projects` 首页点击目标项目

流程：

1. 打开切换器，显示 Recent Projects + Pinned Projects
2. 用户选择目标项目
3. 系统执行切换保护检查（running session / unsaved input）
4. 通过检查后切换到目标项目工作面
5. 恢复该项目上次 UI 状态（tab/filter/selected object）

失败分支：

- 路径不存在：提示 `Project path not found` + Remove from recent
- 权限不足：提示 `Permission denied` + Open in file picker
- 保护检查未通过：弹窗让用户确认是否继续

---

## 3.1 初始化项目

触发：`Open Project` 选择仓库后无 `.seatloom/`

流程：

1. 显示初始化弹窗
2. 用户确认后创建基础目录
3. 跳转 Inbox 视图并显示空状态引导

失败分支：

- 创建失败 -> Toast：`Failed to initialize project` + Retry

---

## 3.2 添加 Seat

触发：Sidebar `SEATS +`

流程：

1. 打开 AddSeatDialog（name + role）
2. 点击 Create 后写入 profile
3. Sidebar 立即出现新 Seat

校验：

- `name` 必填且唯一
- `role` 必须在允许值内（含 custom）

---

## 3.3 Attach 运行中 Session

触发：Session 区域 `Attach`

流程：

1. 扫描可附着进程并展示列表
2. 选择 `runtime + pid + seat`
3. Attach 成功后出现在 Session 列表，Terminal 可切换查看

失败分支：

- attach 不可用 -> 提示 fallback：`Launch with wrap`

---

## 3.4 Wrap 启动新 Session

触发：Terminal `New Session`

流程：

1. 选择 seat/runtime/workdir
2. 启动新 session
3. 自动进入对应 terminal tab
4. 写入 `session.started` 事件

---

## 3.5 Inbox 行动处理

触发：进入 Inbox 视图或新事项到达

流程：

1. 按优先级排序（Critical > Normal > Low）
2. 选中条目后 Detail 展示详情与动作按钮
3. 点击动作（Accept/Return/Resolve）后，条目状态更新并从列表移除

---

## 3.6 Timeline 回放

触发：`Ctrl+2` 或点击 Timeline tab

流程：

1. 默认加载最近 24h / 200 条
2. 支持过滤 seat/workitem/type/time
3. 点击事件打开 Event Detail
4. 详情支持跳转关联对象

---

## 3.7 创建 WorkItem

触发：`Ctrl+N` 或 WorkItems 区域 `+`

流程：

1. 快速模式：仅 title -> draft
2. 完整模式：title/goal/ac/owner/priority
3. 创建后侧边栏即时显示

gate：

- `ready` 需要 AC
- `active` 需要 owner

---

## 3.8 创建与处理 Handoff

触发：`Ctrl+H` 或 WorkItem Detail `Create Handoff`

流程：

1. 填写 from/to/workitem/purpose/expected_outcome
2. 选择附带 artifacts
3. Send 后进入接收方 Inbox
4. 接收方 Accept/Return

---

## 3.9 跨工具切换（Rehydrate）

触发：Session Detail `Switch Runtime`

流程：

1. 选择目标 runtime
2. 生成 LaunchPack 并展示预览
3. 用户确认后启动新 session

fallback：

- 自动注入失败 -> 文件路径 + 短指令

---

## 3.10 Session 中断恢复

触发：Session 状态为 interrupted

流程：

1. 展示恢复选项（native/rebuild/minimal）
2. 优先 native，失败自动降级
3. 恢复成功后产生新 session 记录

---

## 4. 错误与空状态合同

### 4.1 空状态

- 无项目：显示 `Open Project` + Recent Projects
- 无 Seat：提示创建首个 Seat
- 无 Session：提示 Attach 或 Launch
- Inbox 空：`All clear`
- Timeline 空：`No events match current filters`

### 4.2 错误状态

- 所有失败必须给出可重试按钮
- 不允许只在 console 输出错误而无 UI 反馈

---

## 5. 交互验收要点

- 每个关键流程都有触发、成功路径、失败路径
- 每个动作都能在 UI 看到状态变化
- 每个对象都可从列表跳到详情再跳到关联对象

---

*本文件用于约束 Mira 的交互实现和 Flux 的行为验收。*
