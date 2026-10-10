# NIMBUS-2026-10-10-memory-tools-018-v1 — Lane A2:记忆工具面升级(kind/confidence/supersedes/relations/resolved/l2 行号)

**Owner:** Nimbus (seat-nimbus-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-mem018-nimbus`(**blocked** —— 依赖 event `ev-018-memory-hardening-landed`,
即 Onyx 的 migration 018 落地门事件;013 触发器在该事件落账本时把你翻 ready 并唤醒你,
**不要轮询**)· priority 45(解封后高于你队列里的前向协议单)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/nimbus/memory-tools-018`
**Done-definition:** §3 落地 + §4 门禁与证明全绿 + §5 收尾走完。

---

## 0. 定位

D5 Phase 2 硬前置的后半:018 给 memories 补上五个缺口(kind/supersedes+relations/resolved/
confidence/l2 行号区间),**你负责让工具面能写、能按新语义读**。没有这半,前向记录协议
(Lane B)写不出 typed record,清洗流水线(Lane C)的 S3/S5 也无路可走 —— 两者都等在门事件上。

## 1. 前置(你被唤醒时已就绪)

- `ev-018-memory-hardening-landed` 已落账本:kind/supersedes/memory_relations/
  resolved_by+resolved_at/confidence/l2_line_start+l2_line_end 全在,三个新 CHECK 生效,
  event_type 27 值含 MemoryResolved。
- 既有工具面(14 个)与四类记忆事件、手写 FromStr/Display 纪律不变;本单是增量。

## 2. 值域契约(逐字携带,Lane A1 同文;不得改字)

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

## 3. 工具面规格(增量)

### 3.1 remember(扩展入参;全部 typed 校验,拒绝先于任何 DB 接触)
- `kind`(必填,6 值词表,手写 FromStr);`confidence`(必填,quoted|inferred)。
- `supersedes`(可选,memory id):**仅建档时设定,不可变**;设定时同一事务写
  `memory_relations(from=new, to=supersedes, rel='supersedes')` —— 两种拼写保持一致。
- `relations`(可选数组 `[{to_id, rel}]`,rel 4 值):同事务落 memory_relations;to_id 不存在
  = typed 拒绝(FK 之前先校验,给清晰错误);自引用/重复 PK = typed 拒绝。
- `l2_line_start`/`l2_line_end`(可选整数):当 `l2_ref_kind='file'` 且
  `source_kind='archived_session'` 时**必填**(镜像 018 的 CHECK,拒绝信息指出所缺);
  否则给值也接受但不得违反 end >= start。
- `resolved_by/resolved_at/status` 仍**不可由调用方设定**(status 恒 active 起步)。

### 3.2 recall(语义升级)
- **默认排除被 supersede 的行**(陈旧决策当现行召回比没有记忆更糟);新入参
  `include_superseded`(默认 false)显式 opt-in,opt-in 时返回体标注 superseded_by
  (哪条取代了它),让调用方自己判断。
- 其余不变:depth l0 默认、topic 祖先链、每命中行恰一条 MemoryRecalled、heat+1。

### 3.3 resolve(新工具,G3 的闭合通道)
- 入参 `memory_id`;`resolved_by` 取 SEATLOOM_ACTOR_REF(不接受参数伪造),`resolved_at`=now。
- 同事务:status active→resolved + `MemoryResolved` 事件(payload 带 memory_id/resolved_by)。
- typed 拒绝:未知 id;已 resolved(不可改);已 forgotten(墓碑不可改);**没有后台 sweep,
  这是唯一闭合路径**。
- 事件域:Rust `EventType` +`MemoryResolved`,`ALL` 钉 **27**(与 018 的 PG 全量逐字一致)。

### 3.4 Rust 侧落点(既有模式增量)
- `objects/memory.rs`:+`MemoryKind`(6)、+`MemoryConfidence`(2)域,手写 FromStr/Display,
  无 serde derive跨边界;未知值 typed ParseValueError。
- `db/event_domain.rs`:+MemoryResolved,ALL=27,穷举测试更新。
- `db/models.rs` + `db/repositories.rs`:relations 读写、supersedes 一致性(同事务)、
  resolve 写路径、recall 的 superseded 过滤 + superseded_by 标注。
- `mcp-emit/src/memory.rs`:三个工具的入参形状与拒绝类型;`protocol.rs` tools/list
  **14 → 15**;`main.rs` 接线。
- 证明脚本 `crates/seatloom-mcp-emit/proof/memory_018_proof.sh`(既有 proof 模式)。

## 4. Done-definition

- [ ] remember 的 kind/confidence 必填、supersedes/relations/l2 行号按契约校验;负例矩阵
      (kind 越界、confidence 越界、缺行号、end<start、relations 指向不存在、自引用)全部
      typed 拒绝且 canonical_events 计数不变。
- [ ] recall 默认不见 superseded;opt-in 时标注 superseded_by。
- [ ] resolve:active→resolved + MemoryResolved 同事务;已 resolved/forgotten 被拒;
      resolved_by 来自 env,不可伪造。
- [ ] Rust EventType::ALL 钉 27,与 018 PG 全量逐字一致;objects 两新域穷举测试通过。
- [ ] 门禁:`cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings`、
      `cargo test -p seatloom-core -p seatloom-mcp-emit -- --include-ignored --test-threads=1`
      (DB 测试对含 018 的 scratch 库跑;隔离库名,绝不 live)。
- [ ] 证明脚本 8+ 点行为证明(含 supersede 双拼写一致性:列与 relation 行同事务同生)。
- [ ] 交付分支 `packet/nimbus/memory-tools-018`,commit-pinned;未动 infra/**。

## 5. 收尾流程

1. 提交到 `packet/nimbus/memory-tools-018`(commit 带 dispatch id)。
2. 交付说明 `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-memory-tools-018-delivery-v1.md`。
3. **发射门事件(解锁 Lane C part 2 的第二个开关,id 逐字符一致)**:
   ```
   emit_event(type="ArtifactCreated", event_id="ev-typed-memory-tools-landed", schema_version=1,
     payload={"title":"typed memory tools landed","dispatch_id":"disp-mem018-nimbus",
              "artifact_id":"art-typed-memory-tools","branch":"packet/nimbus/memory-tools-018"},
     refs=[{ref_type:"artifact",ref_id:"art-typed-memory-tools",role:"subject"},
           {ref_type:"seat",ref_id:"seat-nimbus-001",role:"actor"},
           {ref_type:"dispatch",ref_id:"disp-mem018-nimbus",role:"context"}])
   ```
4. `complete_dispatch(dispatch_id="disp-mem018-nimbus", outcome="done")`。
5. **自请 Flux**(commit-pinned)→ 验绿后 `request_acceptance`(how_to_verify 用 §6 原文,
   base=track/infra-foundation;acceptance_plan 由 Lyra 供稿,逐字入参)。

## 6. 一步验法(how_to_verify,Lyra 出)

> remember 一条 kind=failure/confidence=quoted 且带 l2 行号的记忆 → 成功且列与 relation
> 双拼写一致;recall 默认不返回被 supersede 的旧决策(opt-in 才见并标注取代者);
> resolve 后 status=resolved 且账本多 MemoryResolved;六类负例全部 typed 拒绝零写入。

## 7. 红线

- 值域逐字与 018/Lane C 一致;不第二套写路径(记忆只经 remember/resolve 工具 +
  同事务事件);无后台 daemon/自动衰减。
- 事件驱动零轮询;不整分支合并;不碰 infra/**(schema 问题上报 Onyx/Lyra)。
- 你在飞的 closeout(两分支同步+防线+两卡)优先消化 —— 本单解封自然排在其后;若closeout
  已完而本单未解封,转 idle 等门事件,不要轮询。
