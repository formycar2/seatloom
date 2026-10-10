# ONYX-2026-10-10-acceptance-closeout-v1 — 017 作废加宽 + 015/016/017 三卡收尾

**Owner:** Onyx (seat-onyx-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-acclose-onyx` · **Milestone:** 验收卡 closeout(Aegis 阻塞裁决驱动)
**Base branch:** `track/infra-foundation`(@ b6751a1) · **Delivery branch:** `packet/onyx/acceptance-cancel-schema`(017 用)
**Done-definition:** §3(017)落地验绿 + §4(三卡)全部建出 + §5 收尾流程走完。

---

## 0. 定位

Aegis 阻塞裁决:5 条已验绿 lane 没有变成验收卡(断点在 PO 已补:验收方案由 Lyra 供稿),
同时发现「点接受会删掉已合并工作」的结构缺陷(Nimbus 做产品防线)。**你那两个分支
(d5-knowledge-schema @ 4520758 / acceptance-plan-schema @ a6f2ac8)已含 track b6751a1、
Flux 已 commit-pinned 验绿 —— 步骤 1(同步)在你侧已完成**,本单只剩:① 落地一个 surfaced
缺口 migration(017);② 用 Lyra 供稿的验收方案把 015/016/017 三张卡建出来。

## 1. 现状(实测,不必重查)

- `packet/onyx/d5-knowledge-schema` @ 4520758:**含 track b6751a1**;Flux PASS(pin 4520758)。
- `packet/onyx/acceptance-plan-schema` @ a6f2ac8:**含 track b6751a1**;Flux PASS(pin a6f2ac8)。
- track 的 event_type 词汇表 = 21 值(014 态);**live = 25 值**(015/016 已 apply)。
- **017 缺口(本单核心)**:cancel_acceptance 正规路径已由 Nimbus 交付且在二进制里,但支撑
  它的 migration **不存在于任何分支**:`acceptances_status_vocab` 无 'cancelled'、
  `canonical_events_event_type_vocab` 无 'AcceptanceCancelled'。Aegis 结清旧空卡
  `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 会被 DB CHECK 拒。Lyra 已上报 Aegis,017 由本单驱动。

## 2. 值域与形状(不得改字)

- 验收方案 shape v1:`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` §1(六份方案的 JSON
  已由 Lyra 写好,逐字入参,见各方案文档 §2)。
- 017 的两处加宽是**全量清单**写法(013/015 模式):event_type 的完整 26 值 = Nimbus Rust
  `EventType::ALL` 钉的集合(逐字):
  `SessionStarted, WorkItemCreated, WorkItemStatusChanged, ArtifactCreated, HandoffSent,
  HandoffAccepted, HandoffCompleted, ReviewVerdictIssued, SeatDelegationIssued,
  SeatDelegationClosed, SupervisorMessage, SeatResponse, SeatDispatchIssued,
  SeatDispatchDelivered, SeatDispatchCompleted, SeatDispatchFailed, SeatDispatchRetried,
  SeatDispatchCancelled, AcceptanceRequested, AcceptanceGranted, AcceptanceRejected,
  AcceptanceCancelled, MemoryCreated, MemoryRecalled, MemoryPromoted, MemoryForgotten`。
  (22 = 014 的 21 + AcceptanceCancelled;+4 memory = 26。写全量 → 从 track 的 21 值态或 live
  的 25 值态任一状态干净应用。)
- acceptances status 值域:`pending | accepted | rejected | cancelled`(cancelled 终态不可变,
  与 accepted/rejected 同纪律)。

## 3. 迁移 017 规格(新分支 `packet/onyx/acceptance-cancel-schema`,基于 track/infra-foundation)

**文件:** `infra/postgres/schema/017_acceptance_cancel.sql`。additive + 幂等
(guarded DO-block;第二遍 apply 零 catalog 写入)。

1. `acceptances_status_vocab` 加宽:guarded 块 DROP+ADD 全量 4 值(guard 检查 'cancelled')。
2. `canonical_events_event_type_vocab` 加宽:guarded 块 DROP+ADD 全量 26 值(guard 检查
   'AcceptanceCancelled')。
3. `COMMENT` 说明:cancelled 是「请求方撤回自己的 pending 请求」的终态(actor=请求席位,
  与 Acceptances 的 operator 决策语义区分);AcceptanceCancelled 一次变迁一事件。
4. `scripts/apply-db.sh` POST_SEED 注册(带理由注释)。
5. **禁止**:不动 acceptances 既有列、不插/改数据行、不重编号历史迁移、不碰 crates/**。
6. 与 Nimbus 对表:他的 Rust 在 `packet/nimbus/d5-mcp-recall` 钉 26(§2 逐字清单即取自该分支);
  若他的 closeout 期间枚举有变,以他交付文档的最终枚举为准并在此说明 —— 值域必须逐字一致。

## 4. 三卡收尾(方案由 Lyra 供稿,你执行调用;身份守卫:只能建自己的卡)

卡 1 — 015:
```
request_acceptance(dispatch_id="disp-d5-lane-a-onyx",
  branch="packet/onyx/d5-knowledge-schema", base="track/infra-foundation",
  how_to_verify="scratch 两遍 apply 第二遍零 catalog 写入;271 行归一化 10 mapped + '(root)' 13 unmapped 已上报;memories 三层与 cold→hot 字段与值域 CHECK 就位",
  flux_verdict_ref=<Flux 对 4520758 的 ReviewVerdictIssued 事件 id(你的 Flux 交付文档里有)>,
  acceptance_plan=<docs/coordination/acceptance/2026-10-10-lyra-onyx-015-knowledge-layer-acceptance-plan-v1.md §2 JSON 逐字>)
```
卡 2 — 016:同式,`dispatch_id="disp-accplan-lane-c-onyx"`,
`branch="packet/onyx/acceptance-plan-schema"`,`flux_verdict_ref=<对 a6f2ac8 的判定>`,
`acceptance_plan=<…-016-plan-carrier-acceptance-plan-v1.md §2>`。
卡 3 — 017(本单交付本体):017 落地 → 自请 Flux 验绿(commit-pinned)后,
`dispatch_id="disp-acclose-onyx"`,`branch="packet/onyx/acceptance-cancel-schema"`,
`flux_verdict_ref=<017 的判定>`,`acceptance_plan=<…-017-cancel-widening-acceptance-plan-v1.md §2>`。

**注意**:若 Nimbus 的 containment 防线已部署,你的分支含 base,会正常通过 —— 那就是防线的
第一个正面实证。三卡建完再 complete 本单(卡 3 的 request_acceptance 在本单 complete 之前)。

## 5. 收尾流程

1. 017 提交到 `packet/onyx/acceptance-cancel-schema`(commit 带 dispatch id)。
2. 自请 Flux 验 017(commit-pinned:branch/SHA/base);015/016 的既有判定直接引用。
3. 按 §4 建三卡(顺序:卡 1、卡 2 可先;卡 3 在 Flux 验绿后)。
4. 写交付说明 `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-closeout-delivery-v1.md`
   (017 改动与幂等证据、三卡的 acceptance_id、分支状态)。
5. `complete_dispatch(dispatch_id="disp-acclose-onyx", outcome="done")`。
6. **席位永不自己合并**;三卡的合并由张先生在 UI 接受后自动 `--no-ff` 进 track。

## 6. 一步验法(卡 3 的 how_to_verify 已在 §4;卡 1/2 见方案文档 §3)

## 7. 红线

- additive + 幂等;零数据行插入/修改(live 的 apply 与旧卡作废是 Aegis 动作)。
- 值域逐字与 Nimbus Rust 一致;不第二套实现;不整分支合并。
- 事件驱动零轮询;schema 之外的问题(防线/UI)上报对应席位,不越界改 crates/ui。
