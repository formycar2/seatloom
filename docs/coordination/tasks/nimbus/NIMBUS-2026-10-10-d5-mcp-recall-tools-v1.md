# NIMBUS-2026-10-10-d5-mcp-recall-tools-v1 — Lane B:MCP 召回/记忆管理/账本读工具

**Owner:** Nimbus (seat-nimbus-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-d5-lane-b-nimbus`(**blocked** —— 依赖 event `ev-d5-015-knowledge-schema-landed`,
即 Onyx 的 migration 015 落地门事件;013 触发器会在该事件落账本时把你翻 ready 并唤醒你,**不要轮询**)
· **Milestone:** D5 知识层
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/nimbus/d5-mcp-recall`
**Done-definition:** §4 全部落地 + §5 验收查询/证明全绿 + §6 收尾流程走完(Flux 验绿 → 提请验收)。

---

## 0. 定位(为什么做这个)

seatloom MCP 目前**零读工具**(仅 emit_event/enqueue_dispatch/get_next_dispatch/
complete_dispatch/cancel_dispatch/request_acceptance)—— 这就是各席位被迫用 psql 轮询的根因。
D5 把知识层做成 Aegis 可调用的能力:recall(渐进披露召回)、remember/promote/forget
(显式记忆管理,不是后台黑箱)、get_dispatch/query_ledger(任何席位不必再碰 psql)。
**每次 recall 留痕 canonical_event:账本从「发生了什么事」扩展到「知道了什么」。**

## 1. 前置(你被唤醒时,这些已就绪)

- Onyx 的 `015_knowledge_layer.sql` 已落地并 apply 到 live:`memories` 表 + `projects.parent_project_id`
  + 归一化视图 `v_archived_sessions_normalized` + 词汇表加宽(event_type=25、ref_type=12)均在。
- 契约见 §2(与 Lane A 逐字一致)。memories 的行**只能**经你的工具写,每个工具在同一事务里
  追加事件(012 dispatch 模式)。
- `crates/seatloom-core/src/db/event_domain.rs`(EventType/RefType 手写 FromStr/Display、
  `EventType::ALL` 穷举测试钉死 PG 值域)与 `crates/seatloom-mcp-emit/src/`(protocol.rs 的
  tools/list、emit.rs、dispatch.rs)是既有模式,顺着扩,不要分叉。

## 2. 值域契约(逐字携带,与 Lane A 一致;不得改字)

来源:`docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md`。
以下为契约全文,Lane A packet 携带**同一文本**。任何偏离 = 上报,不是静默调和。

```
KNOWLEDGE LAYER (D5) — value-domain contract, Lyra sign-off 2026-10-10

SCOPE FRAMEWORK (unchanged ruling): project + topic is the ONLY scoping framework.
Every knowledge object carries BOTH ids. Agent harness + LLM are execution entities
serving project+topic, never a scope of their own.

1. PROJECT TREE — projects.parent_project_id (NEW, migration 015)
   parent_project_id TEXT NULL REFERENCES projects(id)
   NULL = root project. CHECK (parent_project_id IS DISTINCT FROM id).
   Rationale: history already shows hierarchy (sf-code/storage, ai-assistant/sf-package,
   fde/field_evaluation); the tree expresses it without inventing a second framework.

2. PROJECT SLUG NORMALIZATION RULE (archived_sessions.project free text -> project tree)
   R1 exact:   slug == projects.id -> that project.
   R2 path:    'a/b' -> leaf project 'b' under parent 'a'; 'a/b/c' nests likewise.
   R3 the rule CREATES NO ROWS. Materializing the tree is the Phase-2 washing pipeline
      (Aegis lane). 015 ships the rule as an executable object + a verification query
      over all 271 archived_sessions rows; it inserts zero project rows.
   R4 unmapped: unknown slug -> map_status 'unmapped', reported, never guessed.
   R5 '(root)' (13 sessions): 'unmapped' + ESCALATE to Aegis. No ruling is assumed
      here; the strategy decision is requested, not taken.

3. TOPICS — UNCHANGED by 015. The 010 topics table is the topic unit; hierarchy is
   ready via parent_topic_id; status vocab stays open|closed|merged. 015 must not
   alter topics' columns or vocab.

4. MEMORIES (NEW table, 015) — small-granularity distilled knowledge.
   Large refined products stay in documents (002): do not fork that model.
   id                 TEXT PRIMARY KEY
   project_id         TEXT NOT NULL REFERENCES projects(id)
   topic_id           TEXT NOT NULL REFERENCES topics(id)
   title              TEXT NOT NULL
   tags               TEXT[] NOT NULL DEFAULT '{}'
   l0_summary         TEXT NOT NULL          -- ~100 tokens; the ONLY default layer
   l1_overview        TEXT                   -- ~2K tokens; fetched on demand
   l2_ref_kind        TEXT NOT NULL          -- document | file
   l2_document_id     TEXT NULL REFERENCES documents(id)
   l2_file_path       TEXT NULL              -- repo-relative, documents.file_path style
   l2_body_digest     TEXT NULL              -- sha256 hex, documents.body_digest style
   CHECK ((l2_ref_kind = 'document' AND l2_document_id IS NOT NULL)
       OR (l2_ref_kind = 'file' AND l2_file_path IS NOT NULL))
   source_kind        TEXT NOT NULL          -- archived_session | canonical_event | document | manual
   source_ref         TEXT NOT NULL          -- '<harness>:<id>' | event id | doc id | actor note
                                             -- polymorphic TEXT, NO FK (010 ref-fabric reasoning)
   heat               INTEGER NOT NULL DEFAULT 0    -- +1 per recall; NO automatic decay, ever
   recall_count       INTEGER NOT NULL DEFAULT 0
   last_recalled_at   TIMESTAMPTZ NULL       -- NULL = never recalled
   tier               TEXT NOT NULL DEFAULT 'cold'  -- cold | warm | hot
   tier_changed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
   status             TEXT NOT NULL DEFAULT 'active' -- active | forgotten
   forgotten_reason   TEXT NULL
   created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
   updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()

   LAYER DOMAIN (3): l0 | l1 | l2 — progressive disclosure. recall returns l0 by
     default; l1/l2 only on explicit depth. The token budget belongs to the caller.
   TIER DOMAIN (3): cold | warm | hot. Transitions ONLY via the promote tool
     (explicit target tier, up or down). No background daemon, no auto-decay.
   STATUS DOMAIN (2): active | forgotten. forget is a SOFT tombstone (reason
     required). Rows are never hard-deleted (ledger discipline).

5. EVENT VOCABULARY +4 (015) — each maps to EXACTLY ONE transition:
   MemoryCreated    -- (none) -> active/cold; actor = the remembering seat
   MemoryRecalled   -- active: heat+1, recall_count+1, last_recalled_at=now
                       (projection UPDATE backed by the event, WorkItemStatusChanged
                       pattern; payload carries the heat delta)
   MemoryPromoted   -- tier change; payload carries from_tier/to_tier (up or down)
   MemoryForgotten  -- active -> forgotten; payload carries reason (soft tombstone)
   Full PG list after 015 = 25 values (the 21 live values + these 4). 015 writes the
   COMPLETE list in its guarded widening (013's pattern), so it applies cleanly from
   either the track state (15 values) or the live state (21 values).

6. REF TYPE VOCABULARY +1 (015): memory — subject of the four memory events (same
   reasoning 010 used for 'delegation', 012 for 'dispatch', 014 for 'acceptance').
   Full PG list after 015 = 12 values (the 11 live values + memory).

7. CROSS-RUST<->PG TEXT ENUM RULE (binding on both packets): every TEXT enum above is
   hand-written FromStr/Display on the Rust side (seatloom-core event_domain.rs
   pattern); NO serde derive across the Rust<->PG boundary; an unknown value is a
   typed ParseValueError, never a silent default. The Rust EventType::ALL
   exhaustiveness test must pin 25; RefType must gain Memory.

8. WRITE PATH: memories rows are written ONLY through the MCP memory tools, each
   appending its event in the SAME transaction (012 dispatch pattern). No second
   write path, no direct SQL by seats, no file-side authority. No second source of
   truth: refined products live in SeatLoom, never in a bypass file.

9. RECALL AUDIT: every recall call appends exactly one MemoryRecalled event — the
   ledger grows from "what happened" to "what is known". refs: subject = the memory,
   actor = the calling seat, context = the dispatch when recall serves a dispatch.

10. READ TOOLS (Lane B): recall / remember / promote / forget / get_dispatch /
    query_ledger — all MCP tools on seatloom-mcp-emit. Read tools mutate nothing
    except recall's audit event. Zero psql for any seat, ever again.

11. OUT OF SCOPE for D5 (do not build): embeddings / semantic retrieval (recall's
    query is a text filter over title + l0_summary via PG FTS, 002's pattern);
    automatic heat decay or auto-promotion (explicit tools only); memory content
    revision machinery (a correction = forget + remember); any external memory
    system (OpenViking/Mem0/Letta patterns may be studied, code never imported —
    SeatLoom is the base; a base inside a base is a contradiction).
```

## 3. 工具规格(6 个,全部 MCP on seatloom-mcp-emit)

### 3.1 recall(project, topic, query, depth="l0") — 渐进披露召回
- 入参:`project_id`、`topic_id`(两者都是必填 —— 作用域框架不允许「全项目捞记忆」)、
  `query`(文本过滤,PG FTS 对 title + l0_summary,002 模式;可空 = 该 topic 下全部 active)、
  `depth` ∈ `l0|l1|l2`,**默认 l0**;未知 depth =  typed 拒绝(手写 FromStr,不用 serde 兜底)。
- 返回:**带父上下文的主题子树**,不是孤立记忆行 —— 用 `topics.parent_topic_id` 递归 CTE
  取 topic 及其祖先链(每层 title/status),避免断章取义;记忆行按 depth 只含请求层:
  l0 只回 id/title/l0_summary/tier/heat;l1 加 l1_overview;l2 加 l2_ref_kind/l2_document_id/
  l2_file_path/l2_body_digest。
- 副作用:**恰一条** `MemoryRecalled` 事件/被击中的记忆行(同事务:UPDATE memories SET
  heat=heat+1, recall_count=recall_count+1, last_recalled_at=now + INSERT canonical_events,
  契约 §9 的 refs 形状)。零命中 = 零事件。
- 分页:`limit`(默认 20,硬上限如 100),按 heat DESC, updated_at DESC。

### 3.2 remember(project, topic, title, l0_summary, l1_overview?, l2?, source_kind, source_ref, tags?) — 建档
- 入参照契约 §4 的列;`l2_ref_kind`='document' 时必填 `l2_document_id`,'file' 时必填
  `l2_file_path`(CHECK 配对,DB 侧也会拒,双保险);`source_kind` ∈
  `archived_session|canonical_event|document|manual`,`source_ref` 对 archived_session 是
  `'<harness>:<id>'`。tier 初始恒为 cold、status 恒为 active(不允许调用方指定)。
- 副作用:INSERT memories + `MemoryCreated` 事件,同一事务。返回 memory_id。

### 3.3 promote(memory_id, to_tier) — 晋升/降级(显式,唯一通道)
- `to_tier` ∈ cold|warm|hot,可上可下;`from_tier` 从当前行读,两者相同 = typed 拒绝(无变迁)。
- 副作用:UPDATE tier + tier_changed_at + `MemoryPromoted` 事件(payload 带 from_tier/to_tier),
  同一事务。**没有自动衰减、没有后台 daemon**;heat 只由 recall +1。

### 3.4 forget(memory_id, reason) — 软遗忘
- reason 必填非空;对已 forgotten 的行 = typed 拒绝(墓碑不可改,ledger 纪律)。
- 副作用:UPDATE status='forgotten' + forgotten_reason + `MemoryForgotten` 事件,同一事务。
  **永不硬删除。**

### 3.5 get_dispatch(dispatch_id) — 派单读模型(堵 psql 洞)
- 返回该 dispatch 行 + **live 状态**(按依赖存在性重算,与 pull 同款查询)+ 依赖列表。
- 纯读:零写入、零事件。(归属不限本席位 —— 协调状态对舰队可见;但不暴露任何写能力。)

### 3.6 query_ledger(filters) — 账本查询(堵 psql 洞)
- 过滤:project_id、event_type(可多选,手写 FromStr 校验)、ref(ref_type+ref_id)、
  时间窗(occurred_at from/to)、limit/offset;返回 canonical_events 行 + 其 event_object_refs。
- 纯读:零写入、零事件。

## 4. Done-definition(逐条可验)

- [ ] `crates/seatloom-core`:RefType +Memory;EventType +4 且 `EventType::ALL` 钉 25;
       `event_domain.rs` 的 exhaustiveness 测试更新并通过;**手写 FromStr/Display,零 serde derive
       跨 Rust⇄PG 边界**;memories 的 model/repository(read + 三写路径,写只经工具)。
- [ ] `crates/seatloom-mcp-emit`:新模块(如 `src/memory.rs`、`src/read.rs`)实现 6 工具;
       `protocol.rs` tools/list 从 6 → 12;每个写工具 = 行变更 + 事件同事务(复用
       `append_event_in_tx`,不第二实现);actor 从 SEATLOOM_ACTOR_REF,不接受调用方伪造。
- [ ] recall 默认 l0、显式 depth 才取 l1/l2;topic 祖先链随返回;每次 recall 恰一条 MemoryRecalled。
- [ ] 门禁全绿:`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
       `cargo test --workspace`(DB 测试用隔离库名 seatloom_ci,既有惯例)。
- [ ] 证明脚本(既有模式 `crates/seatloom-mcp-emit/proof/*.sh`):对 scratch 库实跑 6 工具,
       含负例(未知 depth、forget 已忘、promote 同 tier、l2 配对违反各一个 typed 拒绝)。
- [ ] 交付分支 `packet/nimbus/d5-mcp-recall`,commit-pinned(SHA 写进交付文档);
       未动 `infra/**`(Lane A 的文件),未合并任何分支。
- [ ] 收尾流程 §6 走完。

## 5. 验收查询/证明(必测点,SQL/脚本你定稿)

1. remember → recall(depth=l0):返回行**只有** l0 层字段;账本恰多 1 条 MemoryRecalled,
   该行 heat=1、recall_count=1、last_recalled_at 非空。
2. recall(depth=l1) / recall(depth=l2):分别才出现 l1_overview / l2_* 字段。
3. recall 返回体含 topic 祖先链(parent_topic_id 递归),对嵌套 topic 实跑。
4. remember 后查 `canonical_events`:MemoryCreated 的 subject ref 是 ref_type=memory 且 role=subject。
5. promote cold→warm→hot 与 hot→cold:各一条 MemoryPromoted,payload 的 from_tier/to_tier 正确;
   同 tier promote 被 typed 拒绝且零写入。
6. forget 带 reason:status=forgotten、行仍在(软墓碑)、MemoryForgotten 在账本;再 forget 被拒。
7. get_dispatch / query_ledger 对已知 dispatch/事件返回与 psql 直查一致的行(附对比输出)。

## 6. 收尾流程(本 dispatch 的完成协议)

1. 提交到 `packet/nimbus/d5-mcp-recall`(commit 信息带 dispatch id)。
2. 写交付说明 `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-d5-mcp-recall-tools-delivery-v1.md`:
   改了哪些文件、跑了哪些门禁+结果、§5 证明输出、工具清单(6→12)。
3. **自请 Flux 技术验证**:enqueue_dispatch 到 `seat-flux-001`,payload 指向交付文档 +
   commit-pinned 三元组(branch / SHA / base=track/infra-foundation),COORDINATION_RULES §14。
4. Flux 验绿(ReviewVerdictIssued)后:`request_acceptance(dispatch_id="disp-d5-lane-b-nimbus",
   branch="packet/nimbus/d5-mcp-recall", flux_verdict_ref=<Flux 判定事件 id>,
   how_to_verify=§7 原文)。**席位永不自己合并。**
5. `complete_dispatch(dispatch_id="disp-d5-lane-b-nimbus", outcome="done")`。

## 7. 一步验法(验收卡 how_to_verify,Lyra 出,原样携带)

> 对任一 project+topic `remember` 一条 → `recall(depth=l0)` 只回 ~100 token 摘要、
> 账本恰好多一条 MemoryRecalled 事件;同一 recall 显式 `depth=l1/l2` 才回概览/原文指针;
> `get_dispatch`/`query_ledger` 返回与 psql 一致的行 —— 此后任何席位不必再碰 psql。

## 8. 红线(违反即上报,不要自行其是)

- 不得产生第二个权威源;提炼产物一律落 SeatLoom,旁路文件不成事实源。
- 不引入外部记忆系统代码(OpenViking/Mem0/Letta 只吸收模式);不做 embedding/语义检索(D5 明确外置)。
- 记忆管理必须是显式工具(remember/promote/forget),**不做后台自动黑箱**(自动衰减/自动晋升都在禁止之列)。
- 不整分支合并;合并走 D2 accept 机制(`--no-ff` 进 track,冲突即拒)。
- 事件驱动,零轮询零查库;schema 问题上报 Onyx/Lyra,不自行改 `infra/**`。
- 洗数据(Phase 0/2)、live 运维、张先生接口 = Aegis 车道,不越界。
