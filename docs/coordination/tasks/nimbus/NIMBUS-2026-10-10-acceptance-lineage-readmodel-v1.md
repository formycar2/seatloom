# NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1 — Lane B:血缘组装查询/命令 + request_acceptance 计划强制

**Owner:** Nimbus (seat-nimbus-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-accplan-lane-b-nimbus`(**blocked** —— 依赖 event `ev-016-acceptance-plan-landed`,
即 Onyx 的 migration 016 落地门事件;013 触发器会在该事件落账本时把你翻 ready 并唤醒你,
**不要轮询**)· **Milestone:** 验收卡血缘+结构化验收方案(张先生阻塞反馈,最高优)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/nimbus/acceptance-lineage-readmodel`
**Done-definition:** §4 全部落地 + §5 证明全绿 + §6 收尾流程走完。

---

## 0. 定位

张先生看了待验收卡:「前因后果都没了,任务的血缘关系看不出来」。Aegis 已实证:**血缘是纯
展示缺口** —— 仅凭卡上已有的 dispatch_id + flux_verdict_ref,用 psql 就挖出了上游派单
(标题/issuer/时间/1253 字符原始任务说明)、10 条事件链、Flux 判定详情、依赖边机制。
数据全在账本里,缺的是**组装**。本单把这条链做成一个查询/命令,并顺手把「缺验收方案
不许建卡」的强制做进 request_acceptance。

## 1. 前置与现状(点破一个分裂,别踩)

- **track 的 mcp-emit 没有 request_acceptance 工具**:D2 自举合并(6aa705a)是路径限定的,
  落了 src-tauri 的 acceptance_cmds/dto/main + seatloom-core db 层,但
  `crates/seatloom-mcp-emit` 的 request_acceptance 在 `packet/nimbus/waker-hardening`
  分支上,**live 跑的是含它的构建**。所以:计划强制(tool 层)要落在 request_acceptance
  真实所在处;若你交付时它仍未合入 track,在你的交付分支里基于 waker-hardening 的内容
  续作(或先把该工具路径限定地并入你的分支),并在交付文档里写清分支血缘 —— 不许静默
  分叉出第二个 request_acceptance 实现。
- **你在飞的缺陷单 `disp-E8BP2vKlOwMHLomm0eX8J`**(acceptance base 记成 'track' 而真实
  分支是 'track/infra-foundation' + 无被认可的作废卡片路径)和本单**都碰
  `dispatch.rs` / acceptance 写入路径**。串行消化:先缺陷单后本单,或同一交付分支续作;
  两个分支不许同时改同一文件后各自合并(会打架)。在交付文档里声明顺序。
- **D5 Lane B(recall)仍 blocked**(等 015 门事件)。本单的血缘组装与 recall 是**同一种
  形状**(对象 + 其上下文链,渐进披露)—— 按契约 §3 合并设计:一个 lineage 模块,acceptance
  血缘是第一个实例;015 落地后 recall 的主题祖先链组装插进同一模块,不另起一套。
- 你被唤醒时 `acceptances.acceptance_plan` 列(016)已就绪;读模型要 select 它。

## 2. 值域契约(逐字携带,Lane A/C/D 同文;不得改字)

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

## 3. 交付规格

### 3.1 血缘组装(seatloom-core,新模块 `db/lineage.rs` 或同义命名)
- `get_acceptance_lineage(db, acceptance_id) -> AcceptanceLineage`:按契约 §3 组装
  acceptance / dispatch / events / fluxVerdict / products / dependencies / rework 七块。
- events 按 occurred_at 排序;products 从事件链的 ref_type ∈ {artifact, document} 的
  refs 取(id/title/path,能 join artifacts/documents 就 join,join 不上就带 ref 原值 ——
  不猜、不补写)。
- 纯读:零写入、零事件。未知 acceptance_id = typed 拒绝(不是空对象)。

### 3.2 暴露面(两个,都做)
- **Tauri**:`cmd_get_acceptance_lineage(acceptance_id)` + DTO(`ui/src/lib/types-dto.ts`
  由 Mira 消费;DTO 字段与契约 §3 七块一一对应,camelCase wire 形状在边界归一)。
- **MCP 读工具** `get_acceptance_lineage`(tools/list 6→7;纯读):让 Aegis/任何席位
  组装血缘不必再 psql(与 D5 的 get_dispatch/query_ledger 同一方向)。
- 共用一个 repo 函数,不各写一遍。

### 3.3 request_acceptance 的计划强制(tool 层,契约 §2 第一层)
- 新增入参 `acceptance_plan`(JSON 对象)。**缺省/缺失/`{}`/形状不符 = typed 拒绝,
  在任何 DB 接触之前**(house discipline;负例测试覆盖每个拒绝分支)。
- 形状校验(严格,契约 §1):`plan_version` 必须是整数 1;`scenario` 非空 string;
  `steps` 非空数组、每项 `action` 非空、`input` 可选 string;`expected` 非空数组、
  每项 `output`/`expect` 非空;`preconditions` 可选 string 数组;**未知键拒绝**
  (与 dispatch 工具 additionalProperties:false 同纪律)。
- 通过时:plan 进 AcceptanceRequested 事件 payload + acceptances 行(同事务,
  `append_event_in_tx`,不第二实现);request 后不可变(无更新路径,与 how_to_verify 一致)。
- 值域 Rust 侧:`AcceptancePlanV1` 结构体 + 手写校验;**跨 Rust⇄PG 的 TEXT 枚联手写
  FromStr/Display 的既有纪律适用于任何新枚举**(本单不新增 TEXT 枚举;plan 是 JSONB,
  形状校验即上述严格规则)。

### 3.4 与 D5 recall 的合并设计(契约 §3 末段)
- lineage 模块的组装器按「对象 + 上下文链 + 渐进披露」组织;015 落地后 recall 的
  topic 祖先链组装进同一模块(同形不同实例)。本单交付文档里写清这个接缝,别让
  D5 Lane B 到时另起炉灶。

## 4. Done-definition

- [ ] `db/lineage.rs`(或同义)存在;`get_acceptance_lineage` 对在候卡
      `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 返回契约 §3 全七块(§5.1 实证)。
- [ ] Tauri `cmd_get_acceptance_lineage` + DTO 就位;MCP `get_acceptance_lineage` 就位
      (tools/list 7;纯读)。
- [ ] request_acceptance 缺/坏 plan 全部分支 typed 拒绝且零写入;合法 plan 建卡成功、
      plan 落在事件 payload 与行里;请求后无更新路径。
- [ ] 门禁全绿:`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
      `cargo test --workspace`;UI 侧如动 types-dto/api 则 `tsc --noEmit` + vitest 绿
      (UI 主改是 Mira 的单,你只加 DTO 与调用封装)。
- [ ] 证明脚本 `crates/seatloom-mcp-emit/proof/acceptance_lineage_proof.sh`(既有
      proof 模式):血缘七块实证 + plan 强制负例矩阵 + MCP 读工具实证。
- [ ] 交付分支 `packet/nimbus/acceptance-lineage-readmodel`,commit-pinned;分支血缘
      (与缺陷单/waker-hardening 的关系)写进交付文档;未动 `infra/**`、未整分支合并。

## 5. 验收查询/证明(必测点)

1. 对 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 调血缘命令:返回含上游派单
   `disp-me6Nm8e9D6ZRe4fKyVokY`(title/issued_by=seat:seat-lyra-001/issued_at/payload 原文)、
   事件链(SeatDispatchIssued → SeatDispatchDelivered → ArtifactCreated →
   SeatDispatchCompleted → AcceptanceRequested → ReviewVerdictIssued)、
   fluxVerdict(ev-UDrRZZqr8LUwfcgEeFdN5 的 verdict=pass/evidence/evidence_detail/target_branch)、
   dependencies(该卡 deps=0,机制在)、rework(null)。与 Aegis 手工 psql 挖掘的结果对比一致。
2. plan 强制负例矩阵:缺 plan / plan={} / plan_version=2 / 未知键 / steps 空 /
   expected 空 / action 空 —— 每个 typed 拒绝且 canonical_events 计数不变。
3. 合法 plan 建卡: AcceptanceRequested payload 含 plan;acceptances 行 acceptance_plan 一致。
4. MCP get_acceptance_lineage 与 Tauri 命令返回同一数据(共用一个 repo 函数的证据)。

## 6. 收尾流程

1. 提交到 `packet/nimbus/acceptance-lineage-readmodel`(commit 带 dispatch id)。
2. 交付说明 `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-lineage-readmodel-delivery-v1.md`
   (改了哪些文件、门禁结果、§5 证明输出、分支血缘声明、与缺陷单的先后)。
3. **发射门事件(解锁 Mira Lane A 的唯一开关,id 逐字符一致)**:
   ```
   emit_event(type="ArtifactCreated", event_id="ev-acceptance-lineage-readmodel-landed",
     schema_version=1,
     payload={"title":"acceptance lineage read model landed","dispatch_id":"disp-accplan-lane-b-nimbus",
              "artifact_id":"art-acceptance-lineage-readmodel","branch":"packet/nimbus/acceptance-lineage-readmodel"},
     refs=[{ref_type:"artifact",ref_id:"art-acceptance-lineage-readmodel",role:"subject"},
           {ref_type:"seat",ref_id:"seat-nimbus-001",role:"actor"},
           {ref_type:"dispatch",ref_id:"disp-accplan-lane-b-nimbus",role:"context"}])
   ```
   该事件插入即触发 013 触发器把 `disp-accplan-lane-a-mira` 翻 ready 并唤醒 Mira。
4. `complete_dispatch(dispatch_id="disp-accplan-lane-b-nimbus", outcome="done")`。
5. **自请 Flux**:enqueue 到 `seat-flux-001`,commit-pinned(branch/SHA/base=track/infra-foundation)。
6. Flux 验绿后 `request_acceptance(dispatch_id="disp-accplan-lane-b-nimbus",
   branch="packet/nimbus/acceptance-lineage-readmodel", flux_verdict_ref=<判定事件 id>,
   how_to_verify=§7 原文)。**席位永不自己合并。**

## 7. 一步验法(验收卡 how_to_verify,Lyra 出,原样携带)

> 对在候卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 调 `cmd_get_acceptance_lineage` → 一次返回
> 上游派单(标题/issuer/时间/原始 payload)、完整事件链、Flux 判定与证据、产物/交付文档
> 入口、依赖边;MCP `request_acceptance` 缺验收方案时 typed 拒绝且零写入,带合法 plan 才建卡。

## 8. 红线

- 血缘零新存储零新事件类型;不得产生第二个权威源;不引入外部系统代码。
- 记忆管理式的「后台自动」禁止蔓延:plan 强制是显式 tool 校验,不是后台补全。
- 不整分支合并;schema 问题上报 Onyx/Lyra,不自行改 `infra/**`。
- 事件驱动零轮询;跨 seat 依赖只用 dependencies 边。
- 洗数据/live 运维/张先生接口 = Aegis 车道;在候卡的 plan 回填 + VALIDATE 是 Aegis 的动作。
