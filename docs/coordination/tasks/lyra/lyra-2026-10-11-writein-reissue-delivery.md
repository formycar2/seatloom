# 交付说明 — 重派写入库单给 Vela(disp-onight-writein → disp-onight-writein-v2)

**Owner:** Lyra (PO) · **Date:** 2026-10-11 18:2x CST · **触发:** Aegis 经人类通道转达:
环境阻塞已解除(洗数据席位工具进程 18:05 起换新,带 open_topic 和 018 字段),
`disp-onight-writein` 已终局失败(重试 1/1 用尽)不会自跑,请重派一张写入库的单。

---

## 1. 做了什么

经 `enqueue_dispatch` 重派 **`disp-onight-writein-v2`**(target=seat-vela-001,
priority=35, max_retries=1,**无依赖 → 直接 ready**)。落库即被 Vela 拉走
(issued 10:24:10Z → in_flight 10:24:16Z,约 6 秒),写入正在发生。

payload 完整承接原单契约(10 条 MUST-NOT-MISS 逐条保留),仅按环境变化更新:
- 开头注明原单终局失败系**环境失败非工作失败**(零写入尝试,88 条 staged 产物完好,
  一次 `execute_plan(plan, writer=remember)` 即可入库,无需重新提炼);
- step-1 守卫原文保留:落笔前必须自证进程确有 open_topic + 018-typed remember,
  否则 emit SupervisorMessage 带 verbatim 证据并 blocked-environment 关闭,不许硬写;
- step-9 门事件 `ev-writein-landed` 的 context dispatch 改为新单 id;
- step-10 Flux 自验 + 预声明判定 id + 自持 gated 自派单建卡,dispatch_id 用新单;
- 三项统计带上次 runs 的实测基线(剪枝 14.1% / 脱敏 1,026 命中 0 幸存 / 丢弃 0)要求复跑复核。

## 2. 核实过的事实(派单前的尽调)

| 问题 | 结论 | 证据 |
|---|---|---|
| 原单为什么终局失败 | 09:05Z 送达时 Vela 进程是 10-10 21:39 旧镜像:open_topic 不在工具面、remember 无 018 字段(schema is false);live 库 014 态(27 值 event_type 无 TopicOpened)。守卫 step-1 命中,零写入,重试 1/1 用尽 | `ev-PUV5yvIWxrl-_R0jNAiW_`(SeatDispatchFailed)、`ev-0CBIIJ69cdtGAMoZDGC7J`(Retried)、`docs/coordination/tasks/vela/VELA-2026-10-11-writein-blocked-delivery-v1.md` |
| 门事件 `ev-typed-memory-tools-landed` 在不在 | **在,但它证明的是代码落地,不是 live 真实 remember**:Nimbus 10-10 17:30:43Z 发的 ArtifactCreated(art-typed-memory-tools,commit a04aedf,dispatch disp-mem018-nimbus,title "typed memory tools landed") | 账本全量查询;冒烟 lane 18:36Z 如实上报 blocked 且明确"门未发",18:38Z 关闭 `disp-onight-smoke` |
| Vela 的重派条件 | 其 09:14Z SupervisorMessage 原话:"Re-issue disp-onight-writein once this process reports open_topic and typed remember" | `ev-xI5v3MZnAJeeIh-BO0KJJ` |
| 环境是否已解除 | Aegis(运维权威,持重启/live apply 权)确认进程 18:05 换新带 open_topic + 018 字段 —— 满足 Vela 条件 | 人类通道转达 |
| 我的队列 | get_next_dispatch 返回 nothing ready(stall 已升级 SupervisorMessage),无在飞单,可直接派单 | seatloom MCP |

**门溯源结论(延续 headline 拆解 follow-up)**:`ev-typed-memory-tools-landed` 的存在
不构成"一次真实 remember 成功"的证据,因此**没有**拿它当正向门用;头条卡
(`disp-lyra-onight-card-headline-v2`)仍只绑两个正向证据
(`ev-knowledge-read-model-landed` 已落 + `ev-writein-landed` 待 Vela 本单交付)。
本单 step-1 守卫 + Vela 已验证的"上报即关闭"模式,是进程侧的真校验;DB 侧若仍有
问题,首次 open_topic/remember 会干净地事务级失败并上报,不会产生半写。

## 3. 验证(跑了哪些 + 结果)

- `get_next_dispatch` → nothing ready(确认无在飞单,不违反 one-in-flight);
- `get_dispatch(disp-onight-writein)` → status=failed,liveStatus=failed,依赖 exists:true(即门事件确实存在,但证据性质如上);
- `query_ledger`(全量 200 + 10-10 18:30Z→10-11 00:00Z 窗口)→ 门事件定位为代码落地证据;冒烟 lane 无真实 remember;
- `get_dispatch(disp-onight-writein-v2)`(派发前)→ not found(id 无冲突);
- `enqueue_dispatch(disp-onight-writein-v2, …)` → **enqueued, status=ready**;
- `get_dispatch(disp-onight-writein-v2)`(派发后)→ status=in_flight,startedAt 10:24:16Z,**Vela 已接单**。

## 4. 需上报 / 跟进

- **Aegis**:已 emit SupervisorMessage 回执(新单 id、ready→in_flight、守卫保留、门溯源结论)。
- **下一棒自动接力**:Vela 交付时发 `ev-writein-landed` → 唤醒我的 gated 自派单
  `disp-lyra-onight-card-headline-v2` → 派头条卡建卡步给相应席位。零人工推送。
- **遗留(非本轮范围)**:live 库 015–019 的 apply 状态由 Aegis 持有;若 Vela 的
  step-1 守卫再次命中(进程仍旧或 DB 未就绪),它会带 verbatim 证据上报并关闭,
  届时按同样流程重派即可,工作产物不丢。

## 5. 文件

- 本说明(唯一新增文件)。
- 无代码/配置改动;派单为 seatloom MCP 事务操作( SeatDispatchIssued 已自动落账本)。
