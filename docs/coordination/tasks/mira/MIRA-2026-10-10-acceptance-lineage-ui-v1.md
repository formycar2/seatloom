# MIRA-2026-10-10-acceptance-lineage-ui-v1 — Lane A:验收卡改版(血缘 + 结构化验收方案,渐进披露)

**Owner:** Mira (seat-mira-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-accplan-lane-a-mira`(**blocked** —— 依赖 event
`ev-acceptance-lineage-readmodel-landed`,即 Nimbus 的血缘读模型落地门事件;013 触发器会在
该事件落账本时把你翻 ready 并唤醒你,**不要轮询**)· **Milestone:** 验收卡血缘+结构化验收方案
(张先生阻塞反馈,最高优)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/mira/acceptance-lineage-ui`
**Done-definition:** §4 全部落地 + §5 验收全绿 + §6 收尾流程走完。

---

## 0. 定位

张先生看了 track 上的待验收区,原话:「待验收给的信息太少了,前因后果都没了,任务的血缘关系
看不出来,验收方案(按照场景在产品上操作功能的步骤、入参、出参、预期结果)」。卡片现在只有
分支名 + 一行 how_to_verify —— 他无法凭此判断价值,只能问人或翻代码,那就退回成瓶颈了。

**诊断(已实证,分两类不混)**:血缘 = 纯展示缺口,数据全在账本,Nimbus 的本单(Lane B)做成
`cmd_get_acceptance_lineage` 命令;验收方案 = 真数据缺口,Onyx 的 016 落
`acceptances.acceptance_plan` JSONB 列(Nimbus 的 request_acceptance 强制非空)。
**你负责把这两样在卡片上展示出来**,并且默认只给「够判断」的量,细节可展开。

## 1. 前置(你被唤醒时已就绪)

- Nimbus 的 `cmd_get_acceptance_lineage(acceptance_id)` + DTO 已落地并在 track 构建里
  (门事件 `ev-acceptance-lineage-readmodel-landed` 即信号);016 的 `acceptance_plan` 列
  已在血缘返回体里。
- 现有实现(你的 D2 交付,已在 track):`ui/src/components/AcceptanceSection.tsx`(5s 轮询、
  `ACCEPTANCE_CMDS_REGISTERED` 门、无 mock 顶替真实后端值的 house rule)、
  `ui/src/views/DispatchInboxView.tsx`(宿主)、`ui/src/lib/acceptance.ts`(边界归一)、
  `ui/src/lib/types-dto.ts`(`AcceptanceCardDto`)、`ui/src/lib/api.ts`。
- **house rule 延续**:血缘命令若因构建未注册而不可用,照 `ACCEPTANCE_CMDS_REGISTERED`
  的既有模式显式渲染该事实,**不许用 mock 数据顶替**。

## 2. 值域契约(逐字携带,Lane B/C/D 同文;不得改字)

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

## 3. 卡片改版规格

### 3.1 信息架构(固定顺序,他的原话顺序,契约 §4 末段)
卡片自上而下:① 哪个里程碑/哪条 lane 下的什么任务 → ② 谁下的单、何时 → ③ 任务原始要求
(派单 payload 原文) → ④ 谁做的 → ⑤ 产物/交付说明在哪 → ⑥ Flux 判定与证据 →
⑦ 结构化验收方案(场景/步骤/入参/出参/预期) → ⑧ 接受/打回。
里程碑/lane 从 dispatch 标题与 payload 既有字段解析(标题形如「D2/Mira: …」「D5 Lane A —
…」);**不新增后端字段**。

### 3.2 渐进披露(契约 §4,与 D5 L0/L1/L2 同一思路,别造第四套)
- **L0(默认)**:①的任务标题 + ②的 issuer/时间 + ⑥的验绿徽章 + ⑦的场景、步骤数、
  首条 expected + ⑧ 按钮。目标:不点开就能判断「这是什么、谁验过、照着怎么验」。
- **L1(展开)**:③的 payload 原文(等宽/pre-wrap,可复制)、④执行席位、⑤产物与交付文档
  入口(路径文本 + 如可能的目标分支链接)、完整事件链(actor + 时间 + 事件类型)、
  完整 ⑦(steps 表格:action/input;expected 表格:output/expect;preconditions 有则显示)。
- **L2(详情行)**:branch、commit SHA、evidence 文档路径、dispatch/acceptance id(可选中复制)。
- 展开态按卡片独立记住;默认收起。空态/缺一项时显式说明「无」(如 deps=0 显示
  「无依赖边」),不许静默省略整块。

### 3.3 数据与边界
- `ui/src/lib/types-dto.ts`:新增 `AcceptanceLineageDto`(与 Nimbus 的 DTO 对应;
  wire 形状以他交付为准,边界归放在 `ui/src/lib/acceptance.ts` 的既有 normalise 模式)。
- `ui/src/lib/api.ts`:新增 `getAcceptanceLineage(acceptanceId)` 调用封装 +
  可用性标志(沿用 `ACCEPTANCE_CMDS_REGISTERED` 的 registered-flag 模式,命名随命令)。
- 轮询节奏不变(5s,与收件箱同一律); lineage 按卡懒加载(展开时取一次 + 手动刷新),
  不随 5s 轮询重复拉全链。

### 3.4 既有行为不许回归
接受(合并 sha 展示)/打回(必填反馈 + 返工单链接)/合并冲突原文展示/终态卡(已合入/已打回)
/错误态 —— D2 既有测试全保留并继续通过;本单只做增量。

## 4. Done-definition

- [ ] 卡片按 §3.1 顺序呈现血缘 + 结构化验收方案;L0/L1/L2 三档按 §3.2 实现。
- [ ] `AcceptanceLineageDto` + api 封装 + normalise 边界就位;无 mock 顶替(命令未注册
      时显式渲染事实)。
- [ ] `tsc --noEmit` 干净;vitest 全绿(含 D2 既有 46 个断言所在的 4 个文件不回归);
      `vite build` 干净。
- [ ] 测试新增:血缘渲染(七块各自出现/空态显式「无」)、plan 渲染(steps/expected 表格)、
      L0→L1 展开、命令未注册的门禁态。
- [ ] 交付分支 `packet/mira/acceptance-lineage-ui`,commit-pinned;未动 `crates/**`、
      `infra/**`;未整分支合并。

## 5. 验收(必测点)

1. 用 Nimbus 命令的真实返回(或录制的真实 fixture —— fixture 必须来自一次真实调用输出,
   交付文档里注明来源)渲染卡片:①-⑧ 全序出现;L0 不含 payload 全文,展开后含。
2. plan 渲染:scenario/steps(action+input)/expected(output+expect)/preconditions;
   步骤数与 L0 徽标一致。
3. 事件链按时间序;actor 显示为席位名(沿用 `seatName`)。
4. 未注册态:显式提示,不轮询报错,不显示假数据。

## 6. 收尾流程

1. 提交到 `packet/mira/acceptance-lineage-ui`(commit 带 dispatch id)。
2. 交付说明 `docs/coordination/tasks/mira/MIRA-2026-10-10-acceptance-lineage-ui-delivery-v1.md`
   (改了哪些文件、门禁结果、fixture 来源、不回归证据)。
3. `complete_dispatch(dispatch_id="disp-accplan-lane-a-mira", outcome="done")`。
4. **自请 Flux**:enqueue 到 `seat-flux-001`,commit-pinned(branch/SHA/base=track/infra-foundation),
   COORDINATION_RULES §14。
5. Flux 验绿后 `request_acceptance(dispatch_id="disp-accplan-lane-a-mira",
   branch="packet/mira/acceptance-lineage-ui", flux_verdict_ref=<判定事件 id>,
   how_to_verify=§7 原文)。**席位永不自己合并。**

## 7. 一步验法(验收卡 how_to_verify,Lyra 出,原样携带)

> 打开 SeatLoom 待验收区 → 卡片默认展示里程碑/lane+任务标题、下单人+时间、Flux 验绿
> 徽章、验收方案场景+步骤数+首条预期,以及接受/打回;展开后可见原始任务 payload、完整
> 事件链、产物/交付说明入口、完整验收方案 —— 不翻代码即可判断价值。

## 8. 红线

- 不得产生第二个权威源;不引入外部组件库/设计系统代码;不整分支合并(路径限定,走 D2 accept)。
- 无 mock 顶替真实后端值(house rule);不许为展示便利改后端契约(要改变上报 Lyra)。
- 事件驱动零轮询;跨 seat 依赖只用 dependencies 边。
- 洗数据/live 运维/张先生接口 = Aegis 车道。
