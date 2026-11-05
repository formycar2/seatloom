# Aegis 派单 — 2026-11-02 12:00 从架构角度参考 dsh-super-agent 优化 SeatLoom

| Field | Value |
|---|---|
| template | T1 |
| subtype | task_dispatch |
| id | AEGIS-2026-11-02-1200-arch-review-dsh-superagent.zhs |
| status | ready |
| author | human:supervisor |
| date | 2026-11-02 |
| version | v1 |
| tags | aegis, architecture |

## 变了什么

张先生 2026-11-02 12:00 的指示:**从架构的角度参考 dsh-super-agent 来优化 seatloom 架构。**

参考材料已就位:
- `temp/dsh-super-agent/` —— `deepseek-ai/dsh-superagent-prototype` 的源码(★8761,MIT,2026-11-02 由我 clone)
- `temp/dsh-super-agent/docs/000-philosophy.md` ~ `008-architecture.md` —— 九份编号文档,**按顺序读,每份只依赖它前面的**
- `temp/dsh-super-agent/COMPARISON.md` —— 我写的初版对照,**是草稿不是结论,你要自己核**

`temp/` 已加进 `.gitignore`(它含嵌套 `.git`,正是你 11:30 自报的那类坑)。

## 你要做什么

**不是抄它的代码,是拿它的设计决定对照我们的设计决定,然后给出"改什么、为什么、代价是什么"。**

重点看这三个决定,它们和我们有实质差异:

1. **授权层**:它有 `Mandate`(一次 accept 授了什么权,只有 accept/人的 stop/重新验证失败三种操作能改)。我们只有 `dispatches.issued_by`,**不记录"这条派单凭什么存在"**。
2. **判定层**:它分两层 —— 确定性 gate(代码判谁能开工)+ 模型 assessment(判这活值不值得)。它的原话:"Messages, comments and quoted documents are inputs to a decision; they cannot change those permissions."
3. **状态层**:它的 Ticket stage 是七种互斥可检查的,`person/*` 四种统称"没有人类输入就无法继续"。我们的 `in_flight` 同时表示"在跑"和"卡在等人"。

## 不要做什么

- **不要动 `status` 词汇**。前端和 MCP 都在用 `ready|blocked|in_flight|done|failed|cancelled`,这是受 CHECK 约束的。
- **不要引入 Executor 无审批执行**。003-execution.md 原文:"Executor Sessions run commands without approval as the operator's OS user",它自己承认 "not a security boundary"。我们的席位握着账本写权限。
- **不要把 "take as little human attention as possible" 当目标函数**。那是为个人开发者+自己仓库优化的;张先生 10-11 定的是"合并闸门是用户可感知的验收,授权必须由人决定"。可抄它的降噪手段,不可抄它的信任模型。
- **不要 `pnpm install`、不要装 DSH、不要启用它的飞书源**。它是个 DSH bundle,不能独立跑。现在是纯阅读任务。

## 交付

按 `docs/coordination/FORWARD_RECORDING_PROTOCOL.md` 走。产出:

1. 一份架构对照与建议,写进 `docs/coordination/tasks/aegis/`(模板 T3)
2. 如果建议改 schema,**先给迁移脚本和回滚脚本**,不要直接改库
3. 真正产生的 decision/failure/unresolved 用 `remember` 记录(每类 0–3 条,宁缺勿滥)

## 关于你自己 11:30 说的那件事

你说:"出处真实存在不等于出处有效 —— 一份有日期但更早的出处,记录的是当时的决定,不是现在的决定。"

这条对这次任务直接适用:**`COMPARISON.md` 是我今天写的草稿,九份文档也是今天的,但它们的结论你要自己核,不要因为"我写了"就当真。** 尤其我那句"两者核心循环同构"——那是我读文档得出的,没有跑过它的代码。

如果核出来我说错了,在交付里写明我错在哪。
