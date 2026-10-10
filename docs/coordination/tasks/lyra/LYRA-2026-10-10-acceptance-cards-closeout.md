# 验收卡 closeout — Aegis 阻塞裁决驱动:分支同步 + 合并 containment 防线 + 5+1 条 lane 建卡

**Owner:** Lyra (PO) · **Date:** 2026-10-10 · **Governing dispatch:** `disp-5L6puXrLkSY0Tml7_PC2X`
**状态:** 拆解完成;Onyx/Mira closeout 与 Nimbus closeout(sync+防线+建卡)已派发;6 份验收方案(PO 供稿)已产出。

---

## 0. Aegis 裁决的两件事

1. **链条断在 PO**:5 条已验绿 lane(015/016/血缘读模型/卡片改版/D5 recall)没有变成验收卡 ——
   request_acceptance 现在 required 含 `acceptance_plan`,而方案的作者是 PO(Lane D 纪律)。
   新依赖没人驱动 → 本单补上。
2. **结构缺陷(会毁数据)**:三个 packet 分支的 merge-base 都早于 track(b6751a1),
   `git merge --no-ff <branch>` 会删掉 track 上已合并的文件(onyx 6 个含验收后端本体 /
   nimbus 12 个 / mira 8 个)。「不要整分支合并」的人工纪律与产品接受按钮的行为不兼容 ——
   必须做进产品,不靠记得别点。

## 1. 现状实测(拆解依据,2026-10-10)

| 分支 | 含 track b6751a1? | Flux 判定(commit-pinned) |
|---|---|---|
| `packet/onyx/d5-knowledge-schema`(015) | **是**(tip 4520758) | PASS(3282145) |
| `packet/onyx/acceptance-plan-schema`(016) | **是**(tip a6f2ac8) | PASS(7d8f301) |
| `packet/nimbus/acceptance-lineage-readmodel` | **否**(mb b84684a2,tip 32a3369) | PASS(da64fcc,同步前 pin) |
| `packet/nimbus/d5-mcp-recall` | **否**(mb b84684a2,tip db7e1f3) | PASS(13eae88,同步前 pin) |
| `packet/mira/acceptance-lineage-ui` | **是**(tip 14e2fd6) | PASS(7e28b2e) |

即:**步骤 1(同步)Onyx/Mira 已完成;Nimbus 两分支未同步。步骤 2(containment 防线)全栈未建
(任何分支无 merge-base/删除校验代码)。步骤 3(建卡)零卡。**

## 2. 拆解:三步 + 一个 surfaced 缺口

**步骤 1 残留 → Nimbus closeout 单**:两分支同步到 b6751a1(冲突按「绝不删除 track 已有
文件」解),重跑门禁。

**步骤 2 → Nimbus closeout 单**(同一处合并实现,不第二套):request_acceptance 建卡时校验
(a) base 存在(他已做);(b) **branch 已包含 base**(merge-base(branch,base)==base),否则拒绝
并说明「请先同步分支」;(c) 更稳一层:**该合并不会删除 base 上已追踪的文件**,若会删则拒绝
并列出路径。(b)(c) 是「接受按钮不毁已合并工作」的产品级保证。

**步骤 3 → 三席位的 closeout 单 + PO 的 6 份验收方案**:每方案 = 场景/在产品上的操作步骤
(点哪里、填什么)/入参/出参/预期结果,shape v1(既有契约)。base 一律
**track/infra-foundation**(真实分支名)。卡片由各席位对自己的 done dispatch 调
request_acceptance 创建(身份守卫:只能建自己的卡)。

**surfaced 缺口 → Onyx closeout 单带 017**:cancel_acceptance 正规路径已交付且在二进制里,
但**支撑它的 migration 不存在**:任何分支/迁移都没有 `acceptances_status_vocab` 的
'cancelled' 与 `canonical_events_event_type_vocab` 的 'AcceptanceCancelled'(Nimbus 只在
.scratch 里起草过 017,未落地;live 词汇表停在 25)。Aegis 结清旧空卡的动作会被 DB CHECK 拒。
017 是 additive+幂等的小迁移(全量清单:Nimbus Rust 钉 26 = 22 + 4 memory),派 Onyx 落地 +
自请 Flux + 建卡;同时 emit SupervisorMessage 上报 Aegis(他的动作被此阻塞)。

**另一个 surfaced 接缝(不阻塞建卡,但写进方案)**:知识层**没有 topic 创建工具**(14 个 MCP
工具里没有;topics 表现在 live 0 行;proof 里是 psql 直插)。remember/recall 需要一个已存在
的 topic。topic 的规模 populated 属 Phase 2 洗数据(Aegis 车道);D5 recall 方案的
precondition 如实写明,并对 Aegis 上报:建议 Phase 2 含 topic 引导,或未来 P1 开
open_topic(需新事件类型,是另一次契约加宽,不在本单)。

## 3. 六份验收方案(PO 供稿,Lane D 纪律的批量执行)

| # | Lane | 方案文档 |
|---|---|---|
| 1 | Onyx 015 知识层 schema | `docs/coordination/acceptance/2026-10-10-lyra-onyx-015-knowledge-layer-acceptance-plan-v1.md` |
| 2 | Onyx 016 方案承载 | `docs/coordination/acceptance/2026-10-10-lyra-onyx-016-plan-carrier-acceptance-plan-v1.md` |
| 3 | Onyx 017 作废加宽(surfaced) | `docs/coordination/acceptance/2026-10-10-lyra-onyx-017-cancel-widening-acceptance-plan-v1.md` |
| 4 | Nimbus 血缘读模型 | `docs/coordination/acceptance/2026-10-10-lyra-nimbus-lineage-readmodel-acceptance-plan-v1.md` |
| 5 | Nimbus D5 recall | `docs/coordination/acceptance/2026-10-10-lyra-nimbus-d5-recall-acceptance-plan-v1.md` |
| 6 | Mira 卡片改版 | `docs/coordination/acceptance/2026-10-10-lyra-mira-card-ui-acceptance-plan-v1.md` |

每份文档 §2 是回填/入参用的**逐字 JSON**;shape 遵循
`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` §1(v1)。

## 4. 派单接线(enqueue 返回值即证据)

| Dispatch | 席位 | 状态 | 内容 |
|---|---|---|---|
| `disp-acclose-onyx` | seat-onyx-001 | **ready**(priority 40) | 017 落地 + 015/016/017 三卡 |
| `disp-acclose-nimbus` | seat-nimbus-001 | **ready**(priority 35) | 两分支同步 + containment 防线 + 血缘/recall 两卡 |
| `disp-acclose-mira` | seat-mira-001 | **ready**(priority 30) | 卡片改版一卡(同步+验绿已就绪) |

三单并行安全:Onyx/Mira 分支已含 base,即便 Nimbus 防线先落地也会通过(这正是同步的意义);
Nimbus 自己的卡在其防线落地后创建,顺便当防线的第一个实证。

## 5. 边界与红线

- 旧空卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 的结清是 **Aegis 的动作**(cancel_acceptance 正规
  路径已在二进制);本单只保证它的 schema 前置(017)被驱动 + 已上报。
- 不整分支合并;各卡合并走 D2 accept 机制(`--no-ff` 进 track/infra-foundation,冲突即拒)。
- 事件驱动零轮询零查库(recall/两个纯读工具已上线,psql 代偿结束)。
- 洗数据/live 运维/张先生接口 = Aegis 车道;017 在 live 的 apply 与旧卡作废都是他的动作。

## 6. 证据索引

- 裁决来源:本单 dispatch payload(Aegis,2026-10-10)。
- 六方案:§3 表。三 closeout packet:`docs/coordination/tasks/{onyx,nimbus,mira}/…-acceptance-closeout-v1.md`。
- 前置契约:`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md`(plan shape v1)。
