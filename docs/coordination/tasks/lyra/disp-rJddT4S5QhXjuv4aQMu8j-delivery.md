# Delivery — disp-rJddT4S5QhXjuv4aQMu8j(验收卡血缘 + 结构化验收方案 — 张先生阻塞反馈拆解)

**Seat:** Lyra (PO) · **Dispatch:** `disp-rJddT4S5QhXjuv4aQMu8j` · **Date:** 2026-10-10
**Branch:** `packet/lyra/milestone-summary` @ `a6bf160`

## 做了什么

张先生阻塞反馈拆解完毕,四条 lane 三条已派发 + Lane D(本人)纪律已立。产物 6 文档,零代码:

| 产物 | 路径 |
|---|---|
| 值域契约(四路逐字一致,md5 `c33e3abf…`,71 行) | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` |
| 拆解总览 | `docs/coordination/tasks/lyra/LYRA-2026-10-10-acceptance-card-lineage-milestone.md` |
| Lane C packet(Onyx) | `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-plan-schema-v1.md` |
| Lane B packet(Nimbus) | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1.md` |
| Lane A packet(Mira) | `docs/coordination/tasks/mira/MIRA-2026-10-10-acceptance-lineage-ui-v1.md` |
| **Lane D 首个结构化验收方案样例**(在候卡 D2/Mira UI,JSON 已校验合法) | `docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md` |

**派发结果(enqueue 返回值即证据):**

| Dispatch | 席位 | 状态 | 依赖 |
|---|---|---|---|
| `disp-accplan-lane-c-onyx` | seat-onyx-001 | **ready**(priority 30,高于其队列里的 D5 015) | 无 |
| `disp-accplan-lane-b-nimbus` | seat-nimbus-001 | **blocked** | `{event, ev-016-acceptance-plan-landed}` |
| `disp-accplan-lane-a-mira` | seat-mira-001 | **blocked** | `{event, ev-acceptance-lineage-readmodel-landed}` |

## 诊断与拆解(他的原话 → 两类缺口)

- **血缘 = 纯展示缺口**:Aegis 已实证仅凭卡上 dispatch_id + flux_verdict_ref 就挖出全链;
  零新存储,Lane B 做成组装查询/命令(seatloom-core lineage 模块 + Tauri 命令 + MCP 读工具,
  与 D5 recall 同形合并设计)。
- **验收方案 = 真数据缺口**:Lane C 迁移 016(`acceptance_plan` JSONB + 非空 CHECK NOT VALID),
  Lane B 在 request_acceptance 做 typed 强制(缺方案不许建卡),Lane A 在卡片按他的信息序
  渲染 + L0/L1/L2 渐进披露。
- **Lane D(本人,最关键)**:验收方案的作者是 PO。此后每个验收包 = 一行判据 + 「照着做就能验」
  的产品操作脚本;首个样例已为在候卡产出(含 base 异常的诚实预期,不瞒报)。

## 关键排雷(拆解时发现并处理)

1. **track 已变**:D2 由 Aegis 自举合入(6aa705a),013/014/acceptances/待验收区 UI 现都在
   `track/infra-foundation`;但 **track 的 mcp-emit 没有 request_acceptance 工具**(在
   `packet/nimbus/waker-hardening`,live 跑含它的构建)—— Lane B packet 已点破该分裂并
   要求声明分支血缘,禁止分叉出第二个实现。
2. **Nimbus 文件撞车**:其在飞缺陷单 `disp-E8BP2vKlOwMHLomm0eX8J`(base 记录错误 + 作废
   卡片路径)与本单 Lane B 同碰 `dispatch.rs`/acceptance 写入路径 —— packet 要求串行消化
   (先缺陷后本单或同分支续作)并声明顺序,未重复派单。
3. **在候卡的 base 现实**:base 记为不存在的 'track',`cmd_accept_delivery` 会明确拒绝
   (已读代码确认 checkout 失败即拒,不静默);且 D2 内容已路径限定进 track。Lane D 样例的
   预期结果按此现实写(缺陷单落地前=拒绝即正确;落地后=合并 sha),回填 + VALIDATE 路由给
   Aegis(live 动作),不自行其是。

## 怎么验证

- Lane 状态:上表(enqueue 返回值;两个 blocked 即证明依赖边已落库)。
- 契约一致性:四份文档首个代码围栏 md5 均为 `c33e3abf246cc423851f750b6c9393a4`(71 行逐字一致)。
- Lane D 样例 JSON:`plan_version=1 / scenario / 4 steps / 4 expected / preconditions`,
  python json 解析 + 形状断言通过。
- 各 lane 一步验法:里程碑文档 §5 与各 packet §7 一致(验收卡 how_to_verify 原样携带)。

## 需 Aegis 执行(已路由,不越界)

1. 016 落地后:把 Lane D 样例 §2 的 JSON 回填进行内卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY`
   并 `VALIDATE CONSTRAINT acceptances_plan_required`(live-DB 动作)。
2. Nimbus 缺陷单落地后:该卡 base 记录纠正为 `track/infra-foundation`,接受路径方可实跑合并。
3. 三 lane 各自 Flux 验绿 + 提请验收后,由你呈张先生;关键路径 Onyx→Nimbus→Mira 串行
   (依赖边驱动,零轮询)。

## 边界确认

只做派单要求的事:诊断 + 契约 + 三 packet + Lane D 样例与纪律。未合并任何分支、未写 live、
未重启服务、未替任何席位做实现。D5 两 lane 与在飞缺陷单均未重复派。

## 产物索引

- Commit `a6bf160`(6 文档)。
- Dispatch:`disp-accplan-lane-c-onyx`(ready)、`disp-accplan-lane-b-nimbus` /
  `disp-accplan-lane-a-mira`(blocked on gate events)。
- 账本事件:ArtifactCreated `ev-11D8vxsq89MY4cL6JfKdG`(本单 6 产物捕获)。
