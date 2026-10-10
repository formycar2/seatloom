# ONYX-2026-10-10-acceptance-plan-schema-v1 — Lane C:migration 016 验收方案结构化承载

**Owner:** Onyx (seat-onyx-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-accplan-lane-c-onyx`(就绪,无依赖;priority 高于你队列里的 D5 015,
张先生阻塞反馈的最高优单)· **Milestone:** 验收卡血缘+结构化验收方案(张先生阻塞反馈)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/onyx/acceptance-plan-schema`
**Done-definition:** §4 全部落地 + §5 验收查询全绿 + §6 收尾流程走完。

---

## 0. 定位

张先生看了 track 上的待验收区,给了阻塞性反馈:卡片只有分支名 + 一行 how_to_verify,
「前因后果都没了,任务的血缘关系看不出来」,要的是结构化验收方案(场景/产品操作步骤/
入参/出参/预期结果)。诊断分两类:**血缘是纯展示缺口**(数据全在账本,UI 没取 —— Nimbus/Mira
的车道),**验收方案是真数据缺口**(本单):让方案可结构化承载,且缺方案就不许建卡。

## 1. 现状(直接可用)

- `acceptances`(014,已在 track 6aa705a 上):id/dispatch_id/project_id/branch/base/
  how_to_verify/flux_verdict_ref/status(pending|accepted|rejected)/feedback/
  rework_dispatch_id/merge_commit/decided_by/decided_at/created_at;
  一行 pending:`acc-disp-me6Nm8e9D6ZRe4fKyVokY`(Mira D2 UI,its plan 由 Lyra 供稿、Aegis 回填)。
- 词汇表现状(track = live = 21 event types / 11 ref types;D5 的 015 会加到 25/12,
  但那是我并行派给同一席位的另一单 —— **两单串行,文件都在你手里,不会撞**)。
- 本单**不加事件类型、不加 ref_type**(血缘是展示缺口;方案骑在既有 AcceptanceRequested
  事件的 payload 里,投影行落 JSONB 列 —— 与 dispatches.payload JSONB 同思路)。
- 迁移号:**016**(015 是 D5 知识层,你的另一单,先它后本单或反之由优先级决定;两单都
  只在 schema/ 与 apply-db.sh 末尾追加,无重叠编辑点)。

## 2. 值域契约(逐字携带,Lane A/B/D 同文;不得改字)

来源:`docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md`。

```
ACCEPTANCE CARD LINEAGE + STRUCTURED ACCEPTANCE PLAN — value-domain contract, Lyra sign-off 2026-10-10

BACKGROUND (blocking feedback from the operator, 2026-10-10): the pending-acceptance
card shows only branch + one-line how_to_verify. The operator cannot judge value from
it — no cause-and-effect, no lineage, no acceptance plan. Two DISTINCT gaps:
  A. lineage = pure display gap — all data is already in the ledger; nothing new to store.
  B. acceptance plan = real data gap — needs a structured carrier, required at card creation.

1. ACCEPTANCE PLAN (NEW structured payload on acceptances, migration 016) — shape v1:
   acceptance_plan JSONB, an object with EXACTLY these keys (unknown key = typed rejection):
     plan_version  INTEGER = 1
     scenario      STRING  non-empty   -- what the user is trying to achieve (product language)
     steps         ARRAY   non-empty   -- the product-operation script; each item:
       { action: STRING non-empty,     -- where to click / what to fill / what to say
         input:  STRING optional }     -- the input for that step
     expected      ARRAY   non-empty   -- each item:
       { output: STRING non-empty,     -- the observable output
         expect: STRING non-empty }    -- the pass criterion (seeing X = pass)
   preconditions  ARRAY of STRING, optional (empty allowed) -- required prior state
   Empty object '{}' is NOT a plan. One plan per acceptance, set at request time,
   immutable after (same immutability rule as how_to_verify).

2. ENFORCEMENT (two layers, both required):
   - TOOL (request_acceptance, MCP): a missing/malformed plan is a TYPED rejection
     before any DB contact (house discipline: typed errors, never a DB-error crutch).
     No plan -> no card.
   - DB (016): CHECK (acceptance_plan <> '{}'::jsonb) added NOT VALID (additive: the one
     legacy pending row is exempt until Aegis backfills its Lyra-authored plan and
     VALIDATEs — that backfill is a live-DB act, Aegis lane).

3. LINEAGE (pure read assembly — NO new storage; the ledger already has everything):
   From an acceptance id, assemble:
     acceptance  (id/status/branch/base/how_to_verify/acceptance_plan/flux_verdict_ref/
                  created_at/decided_*/merge_commit/feedback/rework_dispatch_id)
     dispatch    (id/title/issued_by/issued_at/payload — the ORIGINAL task statement)
     events      (the dispatch's canonical_events in occurred_at order: SeatDispatchIssued
                  -> SeatDispatchDelivered -> ArtifactCreated* -> SeatDispatchCompleted
                  -> AcceptanceRequested -> ReviewVerdictIssued; actor + occurred_at + payload)
     fluxVerdict (the ReviewVerdictIssued event named by flux_verdict_ref:
                  verdict/evidence/evidence_detail/target_branch/lane)
     products    (artifact + document refs found in the event chain: ids/titles/paths)
     dependencies(dispatch_dependencies of the dispatch: dep_kind/dep_value + exists?)
     rework      (rework_dispatch_id -> that dispatch's row, when rejected)
   Assembly lives in ONE module (seatloom-core db lineage), first instance = acceptance
   lineage. D5's recall (memory + topic ancestor chain) is the SAME SHAPE — object plus
   its context chain, progressively disclosed. One design, two assemblers; do not build
   a second ad-hoc query per feature.

4. CARD DISCLOSURE LEVELS (UI, same three-level idea as D5's L0/L1/L2 — do not invent
   a fourth pattern):
     L0 default (enough to judge): milestone/lane + task title; issuer + when;
          Flux verdict badge; acceptance-plan scenario + step count + first expected;
          accept/reject actions.
     L1 expanded: the dispatch's original payload (full task statement); the doing seat;
          products/delivery-doc links; the full event chain; the full plan.
     L2 ids/links: branch, commit SHAs, evidence doc paths, dispatch/acceptance ids.
   Card section order (the operator's own words): which milestone/lane/task -> who
   issued it and when -> the original task requirements (dispatch payload) -> who did
   the work -> where the products/delivery notes are -> Flux verdict and evidence ->
   the structured acceptance plan -> and only then accept/reject.

5. AUTHORSHIP: the acceptance plan is authored by the PO (Lyra), never by Mira/Nimbus/
   Flux. Every acceptance packet from now on carries the follow-it-to-verify product
   script; a one-line how_to_verify alone is no longer an acceptable acceptance packet.
   how_to_verify REMAINS the one-line criterion; the plan is the script that executes it.

6. RED LINES (binding): lineage adds NO new table and NO new event type (display gap);
   the plan adds ONE JSONB column + ONE NOT VALID CHECK (016, additive + idempotent,
   second apply = zero catalog writes); no whole-branch merges (branches are stale vs
   track; path-scoped only, via the D2 accept mechanism); event-driven, zero polling —
   cross-seat sequencing uses enqueue_dispatch dependency edges only.
```

## 3. 迁移 016 规格

**文件:** `infra/postgres/schema/016_acceptance_plan.sql`(016 = 015 之后的下一个号;
006/007 历史空号不重编号)。

1. **additive + 幂等**(010/013 house style): guarded DO-block + `ADD COLUMN IF NOT EXISTS`;
   **第二遍 apply 零 catalog 写入**(§5 实测)。
2. **列**:`ALTER TABLE acceptances ADD COLUMN IF NOT EXISTS acceptance_plan JSONB NOT NULL
   DEFAULT '{}'::jsonb;` —— NOT NULL DEFAULT 使既有行(那张在候卡)无害回填为 '{}';
   `COMMENT ON COLUMN` 逐字引用契约 §1 的 shape v1(与 010/013 的注释密度一致,写清
   「'{}' 不是 plan」「request 时设定后不可变」「shape 校验在 tool 层,DB 只守非空」)。
3. **CHECK**(契约 §2 DB 层):
   ```sql
   DO $$ BEGIN
     IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'acceptances_plan_required') THEN
       ALTER TABLE acceptances
         ADD CONSTRAINT acceptances_plan_required
         CHECK (acceptance_plan <> '{}'::jsonb) NOT VALID;
     END IF;
   END $$;
   ```
   **NOT VALID 是故意的**:既有的唯一 pending 行豁免,待 Aegis 回填 Lyra 供稿的 plan 后
   `VALIDATE CONSTRAINT`(live 动作,Aegis 车道)。注释里写明这个交接,别让下个读 migration
   的人以为忘了 validate。
4. **apply-db.sh 注册**:POST_SEED_SCHEMA 追加 `"016_acceptance_plan.sql|<理由>"`,
   理由风格同既有条目(新列 + 新 CHECK,seed 不写 acceptances;014 之后)。
5. **禁止**:不加事件类型/ref_type(契约 §6);不建新表;不改 acceptances 既有列;
   不插/改任何数据行(在候卡的 plan 回填是 Aegis 的 live 动作);不重编号历史迁移。

## 4. Done-definition

- [ ] `016_acceptance_plan.sql` 存在,§5 查询全绿(scratch 库)。
- [ ] 幂等:第二遍 apply 零 catalog 写入(照 013 的度量方式给前后对比)。
- [ ] `acceptances.acceptance_plan` 列:JSONB / NOT NULL / DEFAULT '{}';注释含 shape v1。
- [ ] CHECK `acceptances_plan_required` 存在且 `convalidated = false`(NOT VALID,见 §3.3)。
- [ ] 负例:插入 `acceptance_plan='{}'` 被 DB 拒(新行强制生效)。
- [ ] apply-db.sh 注册行就位;从 track 当前状态干净应用(015 若已合/未合都不影响本单)。
- [ ] 交付分支 `packet/onyx/acceptance-plan-schema`,commit-pinned;未动 `crates/**`、`ui/**`。

## 5. 验收查询(scratch,必测点)

1. 幂等:第二遍 apply 前后 `pg_class`/`pg_constraint`/`pg_attribute` 计数对比,零变化。
2. 列形态:`SELECT column_name,data_type,is_nullable,column_default FROM information_schema.columns
   WHERE table_name='acceptances' AND column_name='acceptance_plan';`
3. CHECK 形态:`SELECT conname,convalidated,pg_get_constraintdef(oid) FROM pg_constraint
   WHERE conname='acceptances_plan_required';` —— 存在、`convalidated=f`、定义含 `<> '{}'`。
4. 负例:`INSERT` 一行 acceptance_plan='{}'(其余必填列给合法值)→ 被拒;
   `acceptance_plan='{"plan_version":1,...}'` → 通过。

## 6. 收尾流程

1. 提交到 `packet/onyx/acceptance-plan-schema`(commit 带 dispatch id)。
2. 交付说明 `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-plan-schema-delivery-v1.md`
   (改了哪些文件、§5 实测输出、幂等证据、NOT VALID 的交接说明)。
3. **发射门事件(解锁 Nimbus Lane B 的唯一开关,id 逐字符一致)**:
   ```
   emit_event(type="ArtifactCreated", event_id="ev-016-acceptance-plan-landed", schema_version=1,
     payload={"title":"016 acceptance-plan schema landed","migration":"016_acceptance_plan.sql",
              "dispatch_id":"disp-accplan-lane-c-onyx","artifact_id":"art-016-acceptance-plan-schema",
              "branch":"packet/onyx/acceptance-plan-schema"},
     refs=[{ref_type:"artifact",ref_id:"art-016-acceptance-plan-schema",role:"subject"},
           {ref_type:"seat",ref_id:"seat-onyx-001",role:"actor"},
           {ref_type:"dispatch",ref_id:"disp-accplan-lane-c-onyx",role:"context"}])
   ```
   该事件插入即触发 013 触发器把 `disp-accplan-lane-b-nimbus` 翻 ready 并唤醒 Nimbus。
4. `complete_dispatch(dispatch_id="disp-accplan-lane-c-onyx", outcome="done")`。
5. **自请 Flux**:enqueue 到 `seat-flux-001`,commit-pinned(branch/SHA/base=track/infra-foundation),
   COORDINATION_RULES §14。
6. Flux 验绿后 `request_acceptance(dispatch_id="disp-accplan-lane-c-onyx",
   branch="packet/onyx/acceptance-plan-schema", flux_verdict_ref=<判定事件 id>,
   how_to_verify=§7 原文)。**席位永不自己合并。**

## 7. 一步验法(验收卡 how_to_verify,Lyra 出,原样携带)

> scratch 库跑两遍 apply-db.sh(含 016)→ 第二遍零 catalog 写入;`acceptances` 多出
> `acceptance_plan` JSONB 列 + 非空 CHECK(NOT VALID,在 `pg_constraint` 可见
> convalidated=false);插入一张 acceptance_plan='{}' 的行被 DB 拒绝。

## 8. 红线

- 不得产生第二个权威源;不引入外部记忆/文档系统代码;不整分支合并(路径限定,走 D2 accept)。
- 事件驱动零轮询;跨 seat 依赖只用 dependencies 边。
- 洗数据/live 运维/张先生接口 = Aegis 车道:在候卡的 plan 回填 + VALIDATE CONSTRAINT
  是 Aegis 的动作,你不代做;D5 的 015 是你的另一单,串行消化,别合并编辑。
