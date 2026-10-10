# ONYX-2026-10-10-d5-knowledge-layer-schema-v1 — Lane A:migration 015 知识层 schema

**Owner:** Onyx (seat-onyx-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-d5-lane-a-onyx`(就绪,无依赖)· **Milestone:** D5 知识层
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/onyx/d5-knowledge-schema`
**Done-definition:** §4 全部落地 + §5 验收查询全绿 + §6 收尾流程走完(Flux 验绿 → 提请验收)。

---

## 0. 定位(为什么做这个)

SeatLoom 从「管 work」扩展到「管 knowledge」。project + topic 是唯一作用域框架
(张先生定的北极星);015 给这个框架落 schema:项目树能表达层级、记忆有底座、历史
自由文本有归一化规则。**Aegis 决定 data/context/memory 怎么管;你负责让「能管」
在结构上成立。**

## 1. 已实测现状(直接可用,不必重新查)

- `projects`(001):id/name/created_at/两个 budget 列,**无父子字段**,仅 1 行。
- `topics`(010):id/project_id/title/status(open|closed|merged)/opened_by/opened_at/
  closed_at/parent_topic_id —— **层级已就绪,0 行。015 不许动它的列与值域。**
- `documents`(002):template/subtype/version/supersedes/depends_on/tags/file_path/
  body_text/body_digest/parse_status 这套文档权威模型 —— **较大的提炼产物落它,
  不要为记忆重造一套。**
- `archived_sessions`(011):271 行,只有索引;project 是自由文本,无 FK(011 设计如此)。
- 词汇表现状(track 与 live 不同,015 必须从任一状态都能干净应用):
  - `canonical_events_event_type_vocab`:track=15 值,live=21 值(013/014 已加宽,
    但 013/014 文件只在 Onyx 分支、未上 track);
  - `event_object_refs_ref_type_vocab`:track=10 值,live=11 值。
- `scripts/apply-db.sh`:POST_SEED_SCHEMA 显式清单 + 每条带理由注释;015 属 POST-SEED
  (新表 + ADD COLUMN + 值域加宽,seed 不写这些列)。

## 2. 值域契约(逐字携带,与 Lane B 一致;不得改字)

来源:`docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md`。
以下为契约全文,Lane B packet 携带**同一文本**。任何偏离 = 上报,不是静默调和。

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

## 3. 迁移 015 规格

**文件:** `infra/postgres/schema/015_knowledge_layer.sql`(015 是下一个号;006/007 是历史空号,不重编号)。

1. **additive + 幂等**:全部 `CREATE TABLE/INDEX IF NOT EXISTS`、`ADD COLUMN IF NOT EXISTS`、
   catalog-guarded DO-block(013 模式);**第二遍 apply 零 catalog 写入**(用 §5 的幂等查询实测)。
2. **projects.parent_project_id**:按契约 §1。索引 `idx_projects_parent ON projects(parent_project_id)`
   (WHERE parent_project_id IS NOT NULL 的部分索引即可,你定)。
3. **memories 表**:按契约 §4 逐字(列、类型、可空性、默认值、CHECK 一个不少)。
   索引:`(project_id, topic_id, status)`、`(project_id, topic_id, tier)`、
   FTS `idx_memories_fts USING gin(to_tsvector('english', coalesce(title,'') || ' ' || coalesce(l0_summary,'')))`
   (002 的 idx_documents_fts 同款)。`COMMENT ON TABLE/COLUMN` 按 010/013 的密度写
   (为什么这么设计,尤其是 heat 无自动衰减、forget 是软墓碑)。
4. **词汇表加宽**(catalog-guarded DO-block,**写全量清单**,013 模式):
   - `canonical_events_event_type_vocab`:12→15(012)→18(013)→21(014)→**25(+4 memory)**。
     全量 25 值 = live 的 21 值 + MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten。
     guard 检查新值存在性(如 `LIKE '%MemoryCreated%'`),不存在才 DROP+ADD 全量。
   - `event_object_refs_ref_type_vocab`:9→10(012)→11(014)→**12(+memory)**。全量 12 值。
   - 注意:track 状态(15/10)与 live 状态(21/11)不同,但全量清单是两者超集,从任一状态干净应用。
5. **归一化规则(可执行)**:view `v_archived_sessions_normalized`(catalog-guarded 创建,
   定义变更走后续迁移,不在 015 里 DROP+CREATE),每行输出:
   `harness, id, raw_slug, normalized_parent, normalized_leaf, map_status(mapped|unmapped)`。
   实现契约 §2 的 R1/R2/R4/R5;R3 —— **零行插入**。验证查询见 §5。
6. **apply-db.sh 注册**:POST_SEED_SCHEMA 加一行 `"015_knowledge_layer.sql|<理由>"`,
   理由按该文件的既有风格写(为什么 POST-SEED:新表 + ADD COLUMN + 值域加宽,seed 不写这些列;
   且值域 CHECK 只有 010 之后才存在,所以 015 不能排在自己的前置之前)。
7. **禁止**:不许插入任何 project/topic/memory 行(洗数据是 Aegis Phase 2 车道);
   不许改 topics 列或值域;不许碰 documents 模型;不许重编号历史迁移。

## 4. Done-definition(逐条可验)

- [ ] `015_knowledge_layer.sql` 存在且通过 §5 全部验收查询(scratch 库)。
- [ ] 幂等:第二遍 `apply-db.sh` 零 catalog 写入(照 013 的度量方式给出前后对比)。
- [ ] 归一化视图对 271 行给出确定映射:10 个已知 slug 全 mapped、"(root)" 13 行 unmapped。
- [ ] 词汇表终态:event_type=25 值、ref_type=12 值(查询 `pg_get_constraintdef`)。
- [ ] `\d memories` 与契约 §4 逐字一致(列/类型/可空/默认/CHECK)。
- [ ] apply-db.sh 注册行就位;从**干净的 track 状态**与**当前 live 状态(21/11)** 都能干净应用。
- [ ] 交付分支 `packet/onyx/d5-knowledge-schema`,commit-pinned(SHA 写进交付文档)。
- [ ] 未动任何 Lane B 的文件(`crates/**`);未合并任何分支;未写 live(除按既有流程 apply 验证外,
       live 写入须走 Aegis 流程,不在本包范围)。

## 5. 验收查询(scratch 库,SQL 你定稿,这些是必测点)

1. 幂等:第二遍 apply 前后对 `pg_class`/`pg_constraint`/`pg_attribute` 计数 + `\d memories` 对比,零变化。
2. 词汇表:`SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname IN ('canonical_events_event_type_vocab','event_object_refs_ref_type_vocab');` —— 25/12。
3. 归一化:`SELECT map_status, count(*) FROM v_archived_sessions_normalized GROUP BY 1;`
   —— mapped 覆盖 10 个已知 slug(含 sf-code/storage 这类父子形态的 parent/leaf 分解),
   unmapped 恰为 "(root)" 13 行;附每个已知 slug 的落点明细。
4. CHECK 存在性:memories 的三个值域 CHECK(l2_ref_kind 配对、tier、status)均可从
   `pg_constraint` 查到,且插入违反值被拒(给一个负例)。

## 6. 收尾流程(本 dispatch 的完成协议)

1. 提交到 `packet/onyx/d5-knowledge-schema`(commit 信息带 dispatch id)。
2. 写交付说明 `docs/coordination/tasks/onyx/ONYX-2026-10-10-d5-knowledge-layer-schema-delivery-v1.md`:
   改了哪些文件、§5 查询实测输出、幂等证据、"(root)" 上报结论。
3. **发射门事件(解锁 Lane B 的唯一开关,id 一个字符都不能错)**:

   ```
   emit_event(
     type="ArtifactCreated",
     event_id="ev-d5-015-knowledge-schema-landed",
     schema_version=1,
     payload={
       "title": "015 knowledge-layer schema landed",
       "migration": "015_knowledge_layer.sql",
       "dispatch_id": "disp-d5-lane-a-onyx",
       "artifact_id": "art-d5-015-knowledge-schema",
       "branch": "packet/onyx/d5-knowledge-schema"
     },
     refs=[
       {ref_type:"artifact", ref_id:"art-d5-015-knowledge-schema", role:"subject"},
       {ref_type:"seat",     ref_id:"seat-onyx-001",                role:"actor"},
       {ref_type:"dispatch", ref_id:"disp-d5-lane-a-onyx",          role:"context"}
     ]
   )
   ```

   该事件插入 `canonical_events` 时,013 的 `trg_dispatch_ready_on_event` 会把 Lane B
   (`disp-d5-lane-b-nimbus`,依赖正是这个 event id)翻 ready 并 pg_notify 唤醒 Nimbus。
   **不要**提前发、不要用别的 id、不要代替 Nimbus 做 Lane B 的事。
4. **"(root)" 上报**:emit `SupervisorMessage`(actor=seat-onyx-001,subject=archived_sessions
   归一化缺口或等价的 ref 组合,payload={unmapped_count:13, raw_slug:"(root)",
   decision_needed_from:"Aegis"})—— 策略裁决归 Aegis,你不猜。
5. `complete_dispatch(dispatch_id="disp-d5-lane-a-onyx", outcome="done")`。
6. **自请 Flux 技术验证**:enqueue_dispatch 到 `seat-flux-001`,payload 指向你的交付文档 +
   commit-pinned 三元组(branch / SHA / base=track/infra-foundation),按 COORDINATION_RULES §14。
7. Flux 验绿(ReviewVerdictIssued)后:`request_acceptance(dispatch_id="disp-d5-lane-a-onyx",
   branch="packet/onyx/d5-knowledge-schema", flux_verdict_ref=<Flux 判定事件 id>,
   how_to_verify=§7 原文)。**席位永不自己合并。**

## 7. 一步验法(验收卡 how_to_verify,Lyra 出,原样携带)

> 在 scratch 库跑两遍 `apply-db.sh`(含 015)→ 第二遍零 catalog 写入(幂等);
> 对 271 行 archived_sessions 跑归一化视图 → 10 个已知 slug 全部落进项目树、
> 13 行 "(root)" 报 unmapped 且已上报 Aegis(不猜);`\d memories` 可见三层 L0/L1/L2 与
> cold→hot 生命周期字段,四个值域 CHECK 均在。

## 8. 红线(违反即上报,不要自行其是)

- 不得产生第二个权威源;提炼产物一律落 SeatLoom,旁路文件不成事实源。
- 不引入外部记忆系统代码(OpenViking/Mem0/Letta 只吸收模式)。
- 不整分支合并;合并走 D2 accept 机制(`--no-ff` 进 track,冲突即拒)。
- 事件驱动,零轮询零查库;跨 seat 依赖只用 dependencies 边。
- 洗数据(Phase 0/2)、live 运维、张先生接口 = Aegis 车道,不越界。
