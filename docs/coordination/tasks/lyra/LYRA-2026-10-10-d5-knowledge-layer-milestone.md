# D5 产品拆解 — 知识层里程碑:从「管 work」扩展到「管 knowledge」

**Owner:** Lyra (PO) · **Date:** 2026-10-10 · **Governing dispatch:** `disp-km2QQVz98cZVdexWlO6-5`
**状态:** 拆解完成,Lane A / Lane B 已派发(见 §5)。

---

## 0. 北极星与框架(张先生定的,不改)

project + topic 是唯一作用域框架;agent harness + LLM 只是服务于 project+topic 的执行实体;
data 经提炼成 context → memory,其 cold→hot 生命周期也在该框架下。Aegis 是所有 project+topic
的助理,决定这些 data/context/memory 怎么管、怎么召回、怎么把活分给 harness。SeatLoom 最终是
「数据+AI 驱动的 AI Assistant 基座」,历史 Agent Harness session 都归它管。

D5 = 把这个框架落成系统能力:**知识层**(topics + memory 底座 + 召回读工具)。

## 1. 已实测的现状(拆解依据,非推断)

- work 账本是活的:canonical_events=178,dispatches=32,artifacts=15,acceptances=1
- knowledge 层全空:topics=0,documents=0,零 memory/context/embedding 表
- 历史原料:archived_sessions=271 行,14 GB / 481 万行 / 2025-11-20→2026-10-08 / 6 种 harness
  (claude-code, codex, cursor, gemini, opencode, stepcode),只有索引(raw_path+sha256+lines),没有提炼
- projects 仅 1 行且无父子字段;但历史数据里已自发出现层级:sf-code/storage, sf-code/brainpp,
  sf-code/stepatlas, ai-assistant/android-agent, ai-assistant/sf-package, fde/field_evaluation,
  OfficeBench, SteptronOss, cn-gpu-infer-fabric, seatloom, "(root)"(13 个无归属)
- topics.parent_topic_id 已存在(010,层级就绪,只是 0 行)
- seatloom MCP 零读工具(仅 emit_event/enqueue_dispatch/get_next_dispatch/complete_dispatch/
  cancel_dispatch/request_acceptance)—— 各席位被迫用 psql 轮询的根因
- 迁移现状:track/infra-foundation 有 001-005,008-012;013/014 在 Onyx 分支且已 out-of-band 应用到 live,
  待 D2 验收合入;**015 是下一个迁移号**(006/007 是历史空号,不重编号)

## 2. 拆解:两条 lane(文件不相交,payload 自足)

| Lane | 席位 | 内容 | 文件 | 依赖 |
|---|---|---|---|---|
| A | Onyx | migration 015:projects 父子 + 记忆底座 memories(L0/L1/L2 + cold→hot)+ 归一化规则(可执行)+ 词汇表加宽 | `infra/postgres/schema/015_knowledge_layer.sql`、`scripts/apply-db.sh` | 无(就绪) |
| B | Nimbus | MCP 读/管工具:recall/remember/promote/forget + get_dispatch/query_ledger;Rust 侧事件域 +4 类型 +1 ref_type | `crates/seatloom-core/**`、`crates/seatloom-mcp-emit/**` | **event `ev-d5-015-knowledge-schema-landed`**(Lane A 落地时发出) |

**依赖用数据表达,不用轮询**:Lane B 的 dispatch 在 enqueue 时带 dependencies 边
`{dep_kind:"event", dep_value:"ev-d5-015-knowledge-schema-landed"}`。013 的
`trg_dispatch_ready_on_event` 在该事件插入时把 Lane B 翻 ready 并
`pg_notify('dispatch_ready','seat-nimbus-001')` —— waker 唤醒 Nimbus,零轮询。
(此前 backlog P3「blocked→ready 不通知」的缺口已由 013 的触发器闭合。)

## 3. 契约(两侧逐字一致)

`VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md` —— 单一契约,Lane A / Lane B 两个 packet
**逐字携带**,任何偏离都是上报,不是静默调和。要点:项目树父子字段;归一化规则 R1-R5
(「(root)」13 条不猜、上报 Aegis);memories 表三层 L0/L1/L2 + cold→hot 生命周期;
事件词汇 +4(MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten);
ref_type +1(memory);跨 Rust⇄PG 的 TEXT 枚联手写 FromStr/Display,不靠 serde derive。

## 4. 边界(红线,两包共有)

1. **不得产生第二个权威源**:提炼产物一律落 SeatLoom,不允许旁路文件成为事实源。
2. **不引入外部记忆系统代码**(OpenViking/Mem0/Letta 等一律只吸收模式)。张先生要 SeatLoom
   做基座,基座里套基座自相矛盾;且它们多以 agent/user 为轴,与我们 project+topic 为轴冲突。
3. **不要整分支合并**:各 seat 分支相对 track 已 stale(删了 track 文档、重写了 DispatchInboxView),
   合并须路径限定;验收合并走 D2 既有 accept 机制(`--no-ff` 进 track,冲突即拒)。
4. **事件驱动,零轮询零查库**;跨seat依赖只用 enqueue_dispatch 的 dependencies 边。
5. **不归本里程碑**:Phase 0 洗数据 key 配置、Phase 2 洗数据流水线与专职新 seat、Phase 4 飞书
   Aegis 附体接入 —— 都是 Aegis 的车道。D2 待验收卡与 D3 设计包并行,不互相阻塞。

## 5. 一步验法(验收卡的 how_to_verify,Lyra 出)

**Lane A(Onyx):**
> 在 scratch 库跑两遍 `apply-db.sh`(含 015)→ 第二遍零 catalog 写入(幂等);
> 对 271 行 archived_sessions 跑归一化视图 → 10 个已知 slug 全部落进项目树、
> 13 行 "(root)" 报 unmapped 且已上报 Aegis(不猜);`\d memories` 可见三层 L0/L1/L2 与
> cold→hot 生命周期字段,四个值域 CHECK 均在。

**Lane B(Nimbus):**
> 对任一 project+topic `remember` 一条 → `recall(depth=l0)` 只回 ~100 token 摘要、
> 账本恰好多一条 MemoryRecalled 事件;同一 recall 显式 `depth=l1/l2` 才回概览/原文指针;
> `get_dispatch`/`query_ledger` 返回与 psql 一致的行 —— 此后任何席位不必再碰 psql。

## 6. 流程(两 lane 同一收尾)

交付 → 自请 Flux 技术验证(commit-pinned:分支 + SHA,COORDINATION_RULES §14)→ Flux 验绿
(ReviewVerdictIssued)→ `request_acceptance`(branch + 上面的一步验法 + flux_verdict_ref)
生成待验收卡 → Aegis 呈张先生。**席位永不自己合并。**

## 7. 证据索引

- 契约:`VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md`
- Lane A packet:`docs/coordination/tasks/onyx/ONYX-2026-10-10-d5-knowledge-layer-schema-v1.md`
- Lane B packet:`docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-d5-mcp-recall-tools-v1.md`
- 前置契约(仍生效):`VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` + D2 addendum
