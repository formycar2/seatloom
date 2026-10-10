# 张先生阻塞反馈拆解 — 验收卡必须展示血缘 + 结构化验收方案

**Owner:** Lyra (PO) · **Date:** 2026-10-10 · **Governing dispatch:** `disp-rJddT4S5QhXjuv4aQMu8j`
**状态:** 拆解完成,Lane A/B/C 已派发(依赖边串联),Lane D(本人 authorship 纪律)已立规 + 首个样例已产出。

---

## 0. 他的原话与诊断

> 「待验收给的信息太少了,前因后果都没了,任务的血缘关系看不出来,验收方案(按照场景在产品上
> 操作功能的步骤、入参、出参、预期结果)」

**分两类,不混(Aegis 已实证):**

- **A. 血缘 = 纯展示缺口**:拿现有卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 实测,仅凭卡上已有的
  dispatch_id + flux_verdict_ref 就挖出了上游派单(标题/issuer/时间/1253 字符原始任务
  说明)、10 条完整事件链、Flux 判定详情(verdict/evidence/evidence_detail/target_branch)、
  依赖边机制。**数据全在账本里,UI 没取** —— 不需要新存数据,需要组装查询 + 卡片展示。
- **B. 验收方案 = 真数据缺口**:how_to_verify 是单行自由文本,装不下「场景/步骤/入参/
  出参/预期结果」五件事。需要结构化承载,且**由 request_acceptance 强制要求**——
  缺验收方案就不许建卡。

## 1. Lane 划分(依赖边串联,事件驱动零轮询)

| Lane | 席位 | 内容 | 状态 |
|---|---|---|---|
| C | Onyx | migration 016:acceptances.acceptance_plan JSONB + 非空 CHECK(NOT VALID);additive+幂等 | **ready** |
| B | Nimbus | 血缘组装:seatloom-core lineage 模块 + `cmd_get_acceptance_lineage` + MCP 读工具;request_acceptance 计划强制(typed 拒绝) | blocked → `ev-016-acceptance-plan-landed` |
| A | Mira | 卡片改版:血缘 + 结构化验收方案,L0/L1/L2 渐进披露(与 D5 同一思路) | blocked → `ev-acceptance-lineage-readmodel-landed` |
| D | Lyra(本人) | **验收方案的作者是 PO**:此后每个验收包必带「照着做就能验」的产品操作脚本;首个样例已为在候卡产出 | 进行中(纪律已立) |

**串联理由**:Lane B 的读模型要 select `acceptance_plan` 列(016 落地后才存在),Lane A 的
UI 要调 Lane B 的命令 —— 顺序依赖用 dependencies 边表达,013 触发器负责翻转 + 唤醒,
任何席位不轮询不查库。关键路径 deliberately 串行:每一 lane 消费上一 lane 落地的契约面,
避免 D2 式「合并一半绿、代码一半阻塞」的跨车道破损。

## 2. 契约(四路逐字一致)

`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` —— 单一契约,Lane A/B/C 三个 packet
**逐字携带**,Lane D 按它生产。要点:验收方案 shape v1(plan_version/scenario/steps/action+
input/expected/output+expect/preconditions);双层强制(tool typed 拒绝 + DB NOT VALID
CHECK);血缘 = 纯读组装零新存储;卡片三档披露 + 固定信息序(他的原话顺序);
authorship 归 PO。

## 3. Lane D — 本人的新纪律(立即生效)

1. 每个交付的验收包 = how_to_verify(一行判据)**+ 结构化验收方案**(照着做的产品操作脚本)。
   只有一句话的验收包不再可接受。
2. 首个样例(在候卡 D2/Mira UI):`docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md`。
3. 该样例的落库(backfill 进行内卡 + VALIDATE CONSTRAINT)是 live-DB 动作 = Aegis 车道,
   已在契约与本拆解中路由,不自行其是。

## 4. 边界与已知并行项

- **不重复派**:Nimbus 手上的 `disp-E8BP2vKlOwMHLomm0eX8J`(acceptance base 记成 'track'
  而真实分支是 'track/infra-foundation' + 无被认可的作废卡片路径)已派,本单 Lane B/C
  的 packet 已写明与其的文件排序(同席串行,先缺陷后本单,或同分支续作),避免两分支
  同时改 `dispatch.rs` 合并打架。
- **track 已变**:D2 已由 Aegis 自举合入(merge `6aa705a`)—— acceptances 表、待验收区
  UI、013/014 迁移现都在 `track/infra-foundation` 上。三 lane 的 base 分支都是它。
  注意:track 的 mcp-emit **没有** request_acceptance 工具(该工具在
  `packet/nimbus/waker-hardening`;live 在跑的是含它的构建)—— Lane B packet 已点明这个
  分裂现状及处理方式。
- **不整分支合并**:各分支相对 track 已 stale,合并走 D2 accept 机制(路径限定/`--no-ff`)。
- **不归本单**:Phase 0/2 洗数据、live 运维、张先生接口 = Aegis 车道;D5 知识层两 lane
  (015/recall)并行推进,互不阻塞。

## 5. 一步验法(各 lane 验收卡 how_to_verify,Lyra 出)

**Lane C(Onyx):**
> scratch 库跑两遍 apply-db.sh(含 016)→ 第二遍零 catalog 写入;`acceptances` 多出
> `acceptance_plan` JSONB 列 + 非空 CHECK(NOT VALID,在 `pg_constraint` 可见
> convalidated=false);插入一张 acceptance_plan='{}' 的行被 DB 拒绝。

**Lane B(Nimbus):**
> 对在候卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 调 `cmd_get_acceptance_lineage` → 一次返回
> 上游派单(标题/issuer/时间/原始 payload)、完整事件链、Flux 判定与证据、产物/交付文档
> 入口、依赖边;MCP `request_acceptance` 缺验收方案时 typed 拒绝且零写入,带合法 plan 才建卡。

**Lane A(Mira):**
> 打开 SeatLoom 待验收区 → 卡片默认展示里程碑/lane+任务标题、下单人+时间、Flux 验绿
> 徽章、验收方案场景+步骤数+首条预期,以及接受/打回;展开后可见原始任务 payload、完整
> 事件链、产物/交付说明入口、完整验收方案 —— 不翻代码即可判断价值。

## 6. 证据索引

- 契约:`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md`
- Lane C packet:`docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-plan-schema-v1.md`
- Lane B packet:`docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1.md`
- Lane A packet:`docs/coordination/tasks/mira/MIRA-2026-10-10-acceptance-lineage-ui-v1.md`
- Lane D 首个样例:`docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md`
