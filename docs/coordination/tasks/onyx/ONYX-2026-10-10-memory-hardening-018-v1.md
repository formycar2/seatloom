# ONYX-2026-10-10-memory-hardening-018-v1 — Lane A1:migration 018 记忆底座加固(五缺口,硬前置)

**Owner:** Onyx (seat-onyx-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-mem018-onyx`(就绪;priority 50,高于你在飞的 017 closeout —— 本单是
D5 Phase 2 的硬前置:018 未落地不得开始 S3 LLM 抽取)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/onyx/memory-hardening-018`
**Done-definition:** §3 落地 + §4 验收查询全绿 + §5 收尾走完。

---

## 0. 定位

D5 Phase 2(知识层清洗 + 前向记录)设计已定稿(AEGIS-2026-10-10-washing-pipeline-design-v2.md,
张先生批准)。作用域收紧到 seatloom 一个 project(16 sessions / 168MB / 58,015 行)。
**硬顺序红线:018 完成前不得开始 S3 LLM 抽取** —— 实测 015 的 memories 表有五个真缺口,
往装不下价值的容器里灌 token,补完还得重洗 = 重花一次。本单补容器。

## 1. 实测缺口(015 的 memories,逐项 0 处)

- **G1 无 kind**:只有 l2_ref_kind/source_kind(都不是记录类型)→ 六类记录只能塞进无约束
  tags[],问不出「这个项目所有的 failure」。
- **G2 无 supersede/关系**:表达不了「这条决策后来被推翻」。把过期决策当现行召回,比没有
  记忆更糟(今日实例:base='track' 曾是事实后来错;「seats 跑 Claude Code」曾真后来假)。
- **G3 unresolved 无完结态**:status 仅 active|forgotten → 最可执行的一类永远无法关闭。
- **G4 无 confidence**:推断出的决策与原文明说的长得一样;L2 只到文件级时更要命。
- **G5 L2 无行号区间**:「证据在这个 134MB 文件里某处」≈ 没有证据。**全部丢失风险的收敛点,
  承重墙。**

## 2. 值域契约(逐字携带,Lane A2 同文;不得改字)

来源:`docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md`。

```
KNOWLEDGE-LAYER MEMORY HARDENING (D5 Phase 2, migration 018) — value-domain contract, Lyra sign-off 2026-10-10

G1 KIND (record type) — memories.kind TEXT NOT NULL + CHECK, vocab (6):
   decision | failure | fact | preference | artifact | unresolved
   Rationale: without it the six record types have nowhere to live but an unconstrained
   tags[] — you cannot ask "every failure in this project". Additive ALTER uses
   DEFAULT 'fact' for backfill ONLY; the remember tool REQUIRES kind explicitly
   (typed rejection when absent) — the default is never a write path.

G2 SUPERSESSION + RELATIONS:
   memories.supersedes TEXT NULL REFERENCES memories(id) (documents.supersedes style;
   create-time only, immutable after).
   memory_relations (
     from_id TEXT NOT NULL REFERENCES memories(id),
     to_id   TEXT NOT NULL REFERENCES memories(id),
     rel     TEXT NOT NULL CHECK IN ('supersedes','caused_by','fixed_by','refines'),
     PRIMARY KEY (from_id, to_id, rel))
   supersedes and memory_relations.rel='supersedes' are two spellings of the SAME edge;
   the write path keeps them consistent in ONE transaction. Recall DEFAULT excludes
   superseded rows (a stale decision served as current is worse than no memory).

G3 RESOLVED CLOSING STATE — memories.status vocab: active | forgotten | resolved
   (forgotten soft-tombstone semantics UNCHANGED). New columns:
   resolved_by TEXT NULL, resolved_at TIMESTAMPTZ NULL.
   CHECK: (status = 'resolved') = (resolved_by IS NOT NULL AND resolved_at IS NOT NULL).
   'unresolved' is the most executable record class; it must be closable. Closure is an
   explicit tool act (resolve), NEVER a background sweep.

G4 CONFIDENCE — memories.confidence TEXT NOT NULL + CHECK IN ('quoted','inferred')
   BINARY, never a score (LLM self-scored confidence is not trustworthy).
   'quoted' = the raw text says it; 'inferred' = derived. Existing rows backfill to
   'inferred' (they predate the distinction) via the additive DEFAULT; no row rewritten.

G5 L2 LINE RANGE (load-bearing wall — the convergence point of all loss risk):
   memories.l2_line_start INTEGER NULL, l2_line_end INTEGER NULL.
   CHECK: NOT (l2_ref_kind='file' AND source_kind='archived_session')
          OR (l2_line_start IS NOT NULL AND l2_line_end IS NOT NULL
              AND l2_line_end >= l2_line_start).
   "The evidence is somewhere in this 134MB file" is NOT evidence. Pruning (pipeline
   S2.5) may shrink what the LLM sees; it must NEVER shrink the L2 pointer.

EVENT VOCABULARY +1 (018): MemoryResolved — active -> resolved (the G3 closure
transition; exactly one transition, replayable). Full PG list after 018 = 27 values
(the 26 Nimbus's Rust pins today + MemoryResolved). 018 writes the COMPLETE list in its
guarded widening (013's pattern), so it applies cleanly from the track state (21), the
post-017 state (26), or any intermediate.

CROSS-RUST<->PG TEXT ENUM RULE (binding): every TEXT enum above is hand-written
FromStr/Display on the Rust side (objects/event_domain pattern); NO serde derive across
the boundary; unknown value = typed ParseValueError, never a silent default.

LIFECYCLE (unchanged, restated): tier cold|warm|hot, transitions ONLY via promote
(explicit target, up or down); heat +1 per recall only; forget = soft tombstone;
rows never hard-deleted. 018 adds no daemon and no automatic anything.
```

## 3. 迁移 018 规格

**文件:** `infra/postgres/schema/018_memory_hardening.sql`。additive + 幂等
(guarded DO-block + ADD COLUMN IF NOT EXISTS;第二遍 apply 零 catalog 写入)。

1. `ALTER TABLE memories ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'fact';`
   + `memories_kind_vocab` CHECK(6 值,guard 检查新值)。
2. `ALTER TABLE memories ADD COLUMN IF NOT EXISTS supersedes TEXT NULL REFERENCES memories(id);`
3. `CREATE TABLE IF NOT EXISTS memory_relations (...)`(契约 §G2 逐字,含 PK 与 rel CHECK)。
4. `ALTER TABLE memories ADD COLUMN IF NOT EXISTS resolved_by TEXT NULL, ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ NULL;`
   + status 值域加宽 active|forgotten|resolved(guard 检查 'resolved')+
   `memories_resolved_pairing` CHECK(契约 §G3 的布尔等式式)。
5. `ALTER TABLE memories ADD COLUMN IF NOT EXISTS confidence TEXT NOT NULL DEFAULT 'inferred';`
   + `memories_confidence_vocab` CHECK(quoted|inferred)。
6. `ALTER TABLE memories ADD COLUMN IF NOT EXISTS l2_line_start INTEGER NULL, ADD COLUMN IF NOT EXISTS l2_line_end INTEGER NULL;`
   + `memories_l2_lines_required` CHECK(契约 §G5 逐字)。
7. **事件词汇 +1**:`canonical_events_event_type_vocab` guarded DROP+ADD **全量 27 值**
   (Nimbus Rust 现钉 26 + MemoryResolved;26 值清单见 `VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md`
   §2 与本契约;017 若已落地,其 AcceptanceCancelled 在全量里保住 —— 全量写法使你与 017
   的先后无关)。
8. 索引按召回路径需要补(你定):如 `(project_id, topic_id, kind, status)`、
   `memory_relations(to_id, rel)`;`COMMENT ON TABLE/COLUMN` 按 015 密度写清每个缺口
   「为什么」。
9. `scripts/apply-db.sh` POST_SEED 注册(带理由注释)。
10. **禁止**:不建第二个表存记忆;不动 documents 模型;不插/改任何数据行;不新增后台机制
    (D2 已定:无 daemon、无自动衰减,cold→hot 由 recall 路径驱动);不重编号历史迁移。

## 4. Done-definition / 验收查询(scratch,必测点)

- [ ] 第二遍 apply 零 catalog 写入(照 013/015 度量方式)。
- [ ] `\d memories` 含 kind/supersedes/confidence/resolved_by/resolved_at/l2_line_start/l2_line_end;
      三个新 CHECK(kind 6 值 / resolved 配对 / l2 行号必填)在 `pg_constraint` 可见。
- [ ] `memory_relations` 表存在,rel 4 值 CHECK + PK。
- [ ] status 值域 = active|forgotten|resolved;event_type = 27 值含 MemoryResolved。
- [ ] 负例:kind 越界被拒;status='resolved' 但 resolved_by 空被拒(反之亦然);
      l2_ref_kind='file' 且 source_kind='archived_session' 但行号空被拒;
      l2_line_end < l2_line_start 被拒。
- [ ] 从 track 当前状态与「017 已落地」状态分别干净应用(全量写法,顺序无关)。
- [ ] 交付分支 `packet/onyx/memory-hardening-018`,commit-pinned;未动 crates/**。

## 5. 收尾流程

1. 提交到 `packet/onyx/memory-hardening-018`(commit 带 dispatch id)。
2. 交付说明 `docs/coordination/tasks/onyx/ONYX-2026-10-10-memory-hardening-018-delivery-v1.md`
   (改了哪些文件、§4 查询实测、幂等证据、与 017 的顺序无关性证明)。
3. **发射门事件(解锁 Nimbus Lane A2 的唯一开关,id 逐字符一致)**:
   ```
   emit_event(type="ArtifactCreated", event_id="ev-018-memory-hardening-landed", schema_version=1,
     payload={"title":"018 memory hardening landed","migration":"018_memory_hardening.sql",
              "dispatch_id":"disp-mem018-onyx","artifact_id":"art-018-memory-hardening",
              "branch":"packet/onyx/memory-hardening-018"},
     refs=[{ref_type:"artifact",ref_id:"art-018-memory-hardening",role:"subject"},
           {ref_type:"seat",ref_id:"seat-onyx-001",role:"actor"},
           {ref_type:"dispatch",ref_id:"disp-mem018-onyx",role:"context"}])
   ```
4. `complete_dispatch(dispatch_id="disp-mem018-onyx", outcome="done")`。
5. **自请 Flux**(commit-pinned)→ 验绿后 `request_acceptance`(how_to_verify 用 §6 原文)。
   验收方案由 Lyra 供稿(随卡下发,你调用时逐字入参)。

## 6. 一步验法(how_to_verify,Lyra 出)

> scratch 库两遍 apply 第二遍零 catalog 写入;`\d memories` 出现 kind(6 值)/supersedes/
> memory_relations(rel 4 值)/resolved 配对/confidence(quoted|inferred)/l2 行号区间
> (file+archived_session 必填);event_type 27 值含 MemoryResolved;四类负例插入均被拒。

## 7. 红线

- additive + 幂等;零数据行写入(live apply 是 Aegis 动作)。
- 值域逐字与 Lane A2 一致;不第二套;不整分支合并;事件驱动零轮询。
- 无后台 daemon/自动衰减(D2 已定);schema 之外(工具面)上报 Nimbus,不越界改 crates。
