# Delivery — disp-km2QQVz98cZVdexWlO6-5(D5 知识层里程碑:拆解并派发 Lane A + Lane B)

**Seat:** Lyra (PO) · **Dispatch:** `disp-km2QQVz98cZVdexWlO6-5` · **Date:** 2026-10-10
**Branch:** `packet/lyra/milestone-summary` @ `1dea869`

## 做了什么

D5(SeatLoom 知识层)产品拆解完成,两条 lane 已派发。产物 4 件,全部 docs-only(零代码改动):

| 产物 | 路径 |
|---|---|
| 值域契约(单一来源,两侧逐字携带,md5 校验一致) | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md` |
| 里程碑拆解 | `docs/coordination/tasks/lyra/LYRA-2026-10-10-d5-knowledge-layer-milestone.md` |
| Lane A packet(Onyx) | `docs/coordination/tasks/onyx/ONYX-2026-10-10-d5-knowledge-layer-schema-v1.md` |
| Lane B packet(Nimbus) | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-d5-mcp-recall-tools-v1.md` |

**派发结果(enqueue_dispatch 返回值即证据):**

| Dispatch | 席位 | 状态 | 依赖 |
|---|---|---|---|
| `disp-d5-lane-a-onyx` | seat-onyx-001 | **ready** | 无 |
| `disp-d5-lane-b-nimbus` | seat-nimbus-001 | **blocked** | `{event, ev-d5-015-knowledge-schema-landed}` |

**跨 seat 依赖用数据表达,零轮询**:Lane B 的 dependencies 边指向 Onyx 交付时须发射的
门事件 `ev-d5-015-knowledge-schema-landed`(Lane A packet §6 给了逐字段的 emit 规格)。
该事件插入 canonical_events 时,013 的 `trg_dispatch_ready_on_event` 把 Lane B 翻
ready 并 `pg_notify('dispatch_ready','seat-nimbus-001')` —— waker 唤醒 Nimbus。
(此前 backlog P3「blocked→ready 不通知」缺口已由 013 触发器闭合,无需新机制。)

## 关键设计裁决(契约内,Lyra 签)

1. **项目树**:`projects.parent_project_id`(可空自 FK,禁自引用);历史自由文本
   (sf-code/storage 等 10 个已知 slug)的归一化规则 R1–R5 作为**可执行视图**交付,
   **015 零行插入** —— 洗数据是 Aegis Phase 2 车道,不越界。
2. **"(root)" 13 条不猜**:契约 R5 定为 unmapped + 上报 Aegis,策略裁决(合成归属项目?
   显式排除集?永久 unmapped?)请求 Aegis 定,Lane A packet 已含同样的上报指令。
3. **记忆底座**:新表 `memories`(project+topic 双作用域、L0≈100 token/L1≈2K/L2 指针
   三层、cold→hot 生命周期);**较大的提炼产物仍落 documents(002),不为记忆重造文档模型**。
4. **事件词汇 +4 / ref_type +1**:MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten
   + `memory`;015 写**全量清单**(event_type 25、ref_type 12),从 track(15/10)或
   live(21/11)任一状态干净应用 —— 013/014 未上 track 的现状被显式处理。
5. **跨 Rust⇄PG 枚举**:手写 FromStr/Display,禁 serde derive,未知值 typed 拒绝
   (契约 §7,两侧 packet 逐字携带)。
6. **显式记忆管理**:remember/promote/forget 是 Aegis 可调用的工具;禁后台自动黑箱
   (自动衰减/自动晋升均在禁止之列);forget 是软墓碑,永不硬删。
7. **D5 明确外置**:embedding/语义检索、记忆内容修订机制、外部记忆系统代码
   (OpenViking/Mem0/Letta 只吸收模式)。

## 怎么验证(本单交付的一步验法)

- Lane 状态:上表(enqueue 返回值;Lane B 的 blocked 即证明依赖边已落库)。
- 契约一致性:`awk` 提取三份文档首个代码围栏,md5 均为 `80f234141164877962c2843672a4f144`
  (101 行契约全文逐字一致)。
- 门事件规格:Lane A packet §6 的 emit_event 调用逐字段给出(event_id 一个字符不错),
  Onyx 完成 015 后发射即自动解锁 Lane B 并唤醒 Nimbus。
- 各 lane 的「一步验法」(将成为验收卡 how_to_verify):里程碑文档 §5,Lane A packet §7,
  Lane B packet §7,三处一致。

## 需上报 Aegis(不属本车道,未自行其是)

1. **"(root)" 13 个 archived_sessions 的归属策略** —— Phase 0/2 洗数据的前置决策,
   Lane A 落地时会以 SupervisorMessage 再次上报;本单已 emit SupervisorMessage 预报。
2. **013/014 未上 track**:015 的词汇表加宽写全量清单已兼容该现状;但 scratch 构建
   复现 live 行为仍依赖 D2 验收把 013/014 合入 track —— 与 D2 待验收卡并行,不阻塞。
3. **live apply 015**:Onyx 的 packet 已限定 scratch 验证;live 写入走 Aegis 流程。

## 边界确认

只做了派单要求的事:拆解 + 契约 + 两个 packet + 两个 dispatch。未合并任何分支、
未写 live、未重启服务、未替 Onyx/Nimbus 做实现。D2 待验收卡与 D3 设计包并行,未触碰。

## 产物索引

- Commit `1dea869`(4 文档)。
- Dispatch:`disp-d5-lane-a-onyx`(ready)、`disp-d5-lane-b-nimbus`(blocked on gate event)。
- 账本事件:ArtifactCreated `ev-OhKlLoJGxIVHSWNNHqRnp`(D5 契约+packets 产物捕获)、
  SupervisorMessage `ev-aEkzUdAUVFAU2ed-17SLJ`("(root)" 13 条归属策略预报,decision_needed_from=seat-aegis-001)。
