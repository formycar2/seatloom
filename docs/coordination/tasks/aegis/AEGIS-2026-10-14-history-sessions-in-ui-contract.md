# 在 SeatLoom UI 里按 project 看历史会话(三层契约待定)

**派单方**:Aegis · **执行**:产品负责人 seat-lyra-001
**派单 id**:`disp-history-sessions-in-ui-contract`

## 张先生的原话(2026-10-14)

> 我希望在 Seatloom UI 中的 project 中看到很多个 project,project 是之前各 agent harness
> 启动的各 session 的 home path,每一个 homepath 下面有多个 session,每个 session 是一个 seat,
> 按照这个方式来整理元数据到 seatloom UI 中使用。

## 他的三处纠正(都影响设计)

1. **「183 条真实会话」描述不对** —— 应是「183 个会话,每个会话中有很多条对话消息」。
2. **他质疑 OfficeBench 为什么有 49 个会话** —— 查证后:那不是 49 次手动启动,
   是 1 条 benchmark 运维线被切成很多个短命令会话(每个约 5 条消息,模式完全相同)。
3. **不要按 harness 分类**,同一 home path 下 codex/cc/stepcode/cursor 平铺;
   且 **stepcode 是壳子**(包着 cc 或 opencode),要区分。

## 数据已备好(都在 resume-contexts/)

| 产物 | 内容 |
|---|---|
| `SESSIONS-FLAT.csv` | 183 个会话,各带 home path / harness / 起止 / 来源文件 |
| `by-session/` | 183 个文件,一会话一份,36680 条消息,每条带原文件行号 |

精确范围(逐个核过):

- codex **150** 条(按根 session_id 归并;70 个子代理不单算,用 parent_thread_id 分层)
- claude-code **18** 条(一文件一会话,无子代理机制)
- stepcode **1** 条(另 19 条包着 claude-code,跨 harness 去重)
- cursor **13** 条(按内容收工判据;其中 1 个目录含 2 条会话)
- gemini **0** 条(只有 `.project_root` 标记,无对话内容)

38 个 project(home path)。

## 数据模型硬约束(我查过的,别撞)

- `sessions` 表:`seat_id` **NOT NULL** + 外键 → `seats.id`。
  **所以「每个 session 是一个 seat」在现有结构里不成立** —— 会话属于席位,不等于席位。
- `seats` 表被 **8 张表**引用,含 `dispatches.target_seat_id`。
  往 seats 塞 183 行,它们会变成派单目标(唤醒服务按 dispatch-seats.tsv 映射,
  不在表里的记 unmapped 不会真捅,但会出现在席位注册表和派单下拉里)。
- `projects` 表现有 1 行(`seatloom`),有 `parent_project_id` 支持层级。

## 我建议的模型(可反对,要说明理由)

- 38 个 home path → 38 个 `projects` 行
- 183 个会话 → 183 个 `sessions` 行,`seat_id` 指向一个归档席位(如 `seat-archive-001`)
- **不动 `seats` 语义**(不加 183 个假席位)

## 你要交付的(PRD 硬规则:三层缺一不许进实现)

1. **用户故事**:谁、想干什么、现在怎么被卡住
2. **交互流程**:从打开 UI 到看到某个历史会话的完整上下文,含失败路径
3. **屏幕定义**:project 列表 / 会话列表 / 消息视图各长什么样
4. **三个必答**:解决什么痛点 / 不做会怎样 / 上线后行为怎么变
5. **验收标准** + 拆成可派子单(目标席位、依赖边挂事件 id)

## 边界

- **不重做界面骨架**(张先生明确要求过)
- **空状态要有意义**:没有会话的项目说清"为什么空、下一步是什么",不画空框
- **验收卡的「怎么验」必须是人话**。张先生原话:「这些都没办法验收,我看不懂,不清不楚」。
  判据:全中文;不出现 S0/L2/typed record 这类词;每步写清看什么 → 什么算对 → 什么算打回。
