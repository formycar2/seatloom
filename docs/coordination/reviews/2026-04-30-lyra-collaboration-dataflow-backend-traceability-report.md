# Review: Collaboration Dataflow Backend Traceability

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-30-collaboration-dataflow-backend-traceability-report-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/architecture-design.md`, `docs/architecture-decisions.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/003_write_ingest_reconcile.sql`, `infra/postgres/schema/004_operational_review_and_continuity.sql`, `crates/seatloom-core/src/db/reconcile.rs`, `crates/seatloom-core/src/db/document_parser.rs`, `crates/seatloom-core/src/db/repositories.rs`, `scripts/ingest-documents.sh`, `.seatloom/bootstrap/source-map.yaml`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`, `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-real-collaboration-postgres-baseline-acceptance.md`, `docs/coordination/acceptance/2026-04-30-lyra-nimbus-postgres-baseline-verifier-hardening-acceptance.md` |
| tags | lyra, review, postgres, backend, traceability, collaboration, dataflow, authority |
| owner | Lyra |

## Verdict

**不能诚实地说“我们当前协作开发 SeatLoom 产生的所有数据流都已经被 backend 完整覆盖并可端到端追溯”。**

更准确的结论是：

1. **已经覆盖很大一部分历史协作真相**：尤其是文档、任务、评审、验收、记忆、Seat/Role/Delegation、Session/WorkItem/Handoff 基线、Artifact 元数据、Checkpoint、Pipeline、Review/Comment、以及相当一部分事件审计。
2. **已经具备一套可信的 PostgreSQL 权威底座**：并且经过 commit-pinned 远端验证，基础 bootstrap / schema / seed / reconcile / body ingest / DB integration tests 都走通了。
3. **但还没有达到“产品上线后所有新增协作数据都会被 SeatLoom 规范写入、审计、回放、并发保护”的完整状态**。
4. **最关键的未闭环处** 仍然是：
   - prompt lifecycle 的产品级权威化仍未被接受为 canonical truth；
   - mobile / desktop / supervisor 的状态变更 action receipt 还未进入已接受闭环；
   - SeatLoom 自身上线后的 live write ingress（直接从产品表面写入 PG 的边界、幂等、冲突控制、actor/channel 归因）还没有完成已接受设计。

结论用一句话概括：

> **现在的 backend 已经足够支撑“真实历史协作数据的结构化/半结构化沉淀与大部分上下追溯”，但还不足以声称“所有协作数据流已经完整产品化闭环”。**

## 审阅范围与判断边界

### 已接受 / 已验证范围（本报告作为“当前可信底座”）

- **核心 PostgreSQL 基线已接受**：见 `docs/coordination/acceptance/2026-04-29-lyra-nimbus-real-collaboration-postgres-baseline-acceptance.md`
  - exact verified commit: `a658086b54323259fda2ad2a958d097701f1fbbd`
- **baseline verifier hardening 已接受**：见 `docs/coordination/acceptance/2026-04-30-lyra-nimbus-postgres-baseline-verifier-hardening-acceptance.md`
  - final verified commit: `b9bb7340416e3729f30dc751a4bb1f41ee726520`
- **review / continuity / document authority 已通过远端 PostgreSQL 验证链路**：见 `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v5-delivery.md`

### 当前工作树中存在、但本报告不当作“已接受产品真相”的范围

以下能力现在已经出现在 repo 中，但**没有在当前协调记录中形成明确 acceptance closure**，因此本报告只把它们记为 **in-flight / HOLD**：

- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`
- `crates/seatloom-core/src/db/models.rs` 中对应 prompt / channel-action 行模型
- `crates/seatloom-core/src/db/repositories.rs` 中对应 write/read 方法

也就是说：**repo 里已经有 draft / delivery 痕迹，不等于这部分已经成为我们可以对外宣称的“产品级已接受事实”。**

## 状态标记

- **PASS**：已结构化、已验证、已能支撑稳定提取与追溯。
- **PARTIAL**：已有结构和查询面，但历史数据是 curated reconstruction、关联维护不完整、或 live write 仍不完整。
- **HOLD**：还不能作为已接受 canonical truth；要么只在工作树中，要么 live mutation / runtime authority 还未闭环。

## 总体覆盖矩阵

| 数据家族 | 当前协作中的真实输入 / 输出 | SeatLoom 当前 canonical 存储 | 获取 / 写入路径 | 提取 / 查询路径 | 向上 / 向下追溯 | 状态 | 关键缺口 |
|---|---|---|---|---|---|---|---|
| Seat / Role / Delegation | seats、职责、项目角色、临时代理 | `seats`, `project_role_bindings`, `seat_delegations` | baseline seed（真实协作映射） | `SeatloomDb` list/get + FK 查询 | Seat → Role → Delegation → WorkItem / Session | PASS | 主要是历史重建写入，不是 live product write |
| Session / WorkItem / Handoff 基线 | 会话、任务、交接 | `sessions`, `workitems`, `handoffs` | baseline seed（真实协作映射） | `SeatloomDb` list/get | WorkItem ↔ Handoff ↔ Session ↔ Seat | PARTIAL | 历史被压缩；不是完整逐条 runtime replay |
| Handoff receipt | 交接确认 / ACK | `handoff_receipts` | operational seed（真实 acceptance 时刻重建） | `SeatloomDb` read path + SQL | Receipt → Handoff → WorkItem → Actor | PASS | 尚未接入 live UI/mobile action ingress |
| Artifact metadata | 真实产出物的 family/subtype、来源路径 | `artifacts` | baseline seed | `SeatloomDb` list/get + filter | Artifact → Session / WorkItem / File path | PARTIAL | body/正文不是对所有 artifact 自动入库；部分 artifact 仍是元数据级 |
| Typed documents | PRD/spec/task/review/acceptance/memory/governance 等 markdown | `documents`, `document_sections` | reconcile + curated seed + direct body ingest | `SeatloomDb` list/get + PG FTS | Document → Section → Version → Assoc → Artifact / WorkItem / Session | PASS | `document_associations` 仍主要靠 seed，不是 reconcile 自动维护 |
| Document versions / reconcile bookkeeping | 文档版本、扫描、变更记录 | `document_versions`, `reconcile_runs`, `reconcile_items` | `run_reconcile()` | `SeatloomDb` / SQL | File path → Run → Version → Document | PASS | 主要覆盖 repo 文档面，不覆盖 runtime-originated object mutation |
| Document associations | 文档与 workitem/session/handoff/project 的显式关联 | `document_associations` | 目前主要来自 seed | `SeatloomDb` / SQL | Document ↔ WorkItem / Session / Handoff / Project | PARTIAL | 不是自动维护；后续 runtime 新文档关联还未闭环 |
| Canonical events | 决策 / 状态 / milestone 审计 | `canonical_events`, `event_object_refs` | baseline seed（真实事件重建） | `SeatloomDb` list/get + type filter | Event → Object refs → downstream objects | PARTIAL | 不是完整原始事件流；部分状态仍靠 payload 推断 |
| Checkpoints / continuity | continuity pack、上下文分层、恢复点 | `checkpoints` | operational seed（真实 continuity 状态重建） | `SeatloomDb` / SQL | Checkpoint → Session → Artifacts / Decisions / Budget | PASS | 还未形成 live checkpoint write path |
| Pipeline verification | cargo check/test/fmt/clippy 等验证运行 | `pipeline_runs` | operational seed（真实验证行为重建） | `SeatloomDb` / SQL | Pipeline → WorkItem → Evidence refs / Artifacts | PASS | 实时产品触发写入还未闭环 |
| Review / comments | 评审意见、修订、移动端短反馈的统一对象族 | `review_threads`, `review_comments` | operational seed（SG-01 等真实评审链重建） | `SeatloomDb` / SQL | Comment → Thread → Target object → Evidence | PASS | live mobile/desktop 提交同一对象族的入口还未接上 |
| Prompt lifecycle | prompt blocked、分类、策略、动作历史 | draft `prompt_instances`, `prompt_actions` | 工作树 draft / 未接受 packet | draft repository methods | 设计上可追，但未形成 accepted truth | HOLD | 还不能算已接受 canonical runtime authority |
| Channel action receipts | mobile/desktop/supervisor 的状态变化回执 | draft `channel_action_receipts` | 工作树 draft / 未接受 packet | draft repository methods | 设计上可追，但未形成 accepted truth | HOLD | 尚未形成统一 live write ingress 闭环 |
| SeatLoom 上线后的新数据直接入库 | UI 直接产生的新任务、新评论、新审批、新 prompt 决策 | 未完成 accepted live write boundary | 未闭环 | 未闭环 | 未闭环 | HOLD | 幂等、revision guard、冲突处理、actor/channel 写边界未完成 |
| 原始终端 transcript / bounded prompt evidence authority | 原始 terminal 输出、stdin required、prompt preview | 目前无已接受 canonical family | 暂无 accepted path | 暂无 accepted path | 只能部分依赖 artifact / event / checkpoint 引用 | HOLD | 原始 runtime 证据与 bounded preview 还未成为 accepted 权威对象 |

## 现有协作数据（输入 / 输出）分门别类梳理

### A. 治理与决策文档输入

包括：

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/architecture-design.md`
- `docs/architecture-decisions.md`
- `docs/coordination/COORDINATION_RULES.md`

#### 1) 获取

- 当前由 repo 中的 markdown 文件提供。
- `crates/seatloom-core/src/db/reconcile.rs` 的 `run_reconcile()` 会扫描有界路径：
  - `docs/*.md`
  - `docs/coordination/**/*.md`

#### 2) 解析

- `crates/seatloom-core/src/db/document_parser.rs`
  - `parse_header()` 解析 universal header
  - `extract_sections()` 提取 deterministic heading/anchor sections
  - `validate_subtype()` 做 subtype 合法性校验

#### 3) 处理

- `run_reconcile()` 对每个文件执行：扫描 → digest → parse → upsert documents → 重建 sections → 写 reconcile bookkeeping → 必要时写 version snapshot。

#### 4) 存储

- `documents`
- `document_sections`
- `document_versions`
- `reconcile_runs`
- `reconcile_items`

#### 5) 归档

- 文档正文与 header 变化进入 `document_versions`
- 每次 reconcile 有独立 `reconcile_run`
- 每个文件在每次 run 中有独立 `reconcile_item`

#### 6) 提取

- `SeatloomDb::list_documents()` / `get_document()` / `list_document_sections()` / `list_document_versions()`
- PostgreSQL `tsvector` / GIN 支持 L2 全文检索

#### 7) 上下追溯

- **向上追溯**：document → file path → reconcile run → version history
- **向下追溯**：document → `document_associations` → workitem / session / handoff / project

#### 8) 当前判断

- **PASS**，这是当前 backend 最完整、最可信的一条 canonical dataflow。

### B. 协调过程产出的任务 / 评审 / 验收 / memory 文档输出

包括：

- `docs/coordination/tasks/**`
- `docs/coordination/reviews/**`
- `docs/coordination/acceptance/**`
- `docs/coordination/memory/**`
- `docs/coordination/roles/**`

这些既是“协作输出”，也是下一轮协作的“输入”。

#### SeatLoom 当前如何处理

- 获取、解析、处理、存储链路与 A 类相同。
- 差别在于这些文档还会进一步映射到：
  - `artifacts`
  - `document_associations`
  - `review_threads` / `review_comments`
  - `canonical_events`

#### 当前能表达什么

- 任务包本身可以成为 typed document。
- review / acceptance / memory 可以被结构化、分段、检索、版本化。
- 某些文档已经通过 seed 被关联到具体 workitem / artifact / session。

#### 关键不足

- 这些文档与对象之间的关联，**目前并不是自动从文档内容推导并持续维护**；很多关联仍来自 curated seed。
- 因此：
  - 文档本身是 **PASS**；
  - 文档到业务对象的关联维护能力是 **PARTIAL**。

### C. 人员 / Seat / 权限结构输入

包括：

- Seat 身份
- project-local role binding
- authority doc refs
- constraints
- active delegation

#### 获取

- 当前主要来自 baseline seed 对真实协作的重建。
- 证据来源记录在 `.seatloom/bootstrap/source-map.yaml`。

#### 解析与处理

- 属于直接结构化对象，不经过 markdown parser。
- 由 seed SQL 显式写入 relational tables。

#### 存储

- `seats`
- `project_role_bindings`
- `seat_delegations`

#### 归档

- 当前以 state snapshot 为主。
- delegation 有 issued / expires / status，能表达阶段性 authority overlay。

#### 提取

- `SeatloomDb::list_seats()` / `get_seat()`
- `SeatloomDb::list_role_bindings_for_project()`
- `SeatloomDb::list_delegations()` / `get_delegation()`

#### 上下追溯

- **向上**：delegation → issuer/from/to seat → role binding → authority doc refs
- **向下**：seat / delegation → workitem / session / handoff / review target

#### 当前判断

- **PASS**（结构层面）
- 但仍然是“真实协作的 curated reconstruction”，不是 live UI 驱动的直接写入。

### D. 执行过程对象：Session / WorkItem / Handoff

#### 当前协作中的真实来源

- 我们通过多 seat 协作产生的：
  - 开始工作 / 恢复工作 / 阻塞 / 完成
  - 任务下发 / 修改 / 验收
  - 交接、回执、复查

#### SeatLoom 当前获取方式

- 主要来自 `infra/postgres/seed/001_real_collaboration_baseline.sql` 的真实重建。
- 来源说明由 `.seatloom/bootstrap/source-map.yaml` 记录。

#### 存储

- `sessions`
- `workitems`
- `handoffs`

#### 提取

- `SeatloomDb::list_sessions()` / `list_sessions_for_seat()`
- `SeatloomDb::list_workitems()` / `get_workitem()`
- `SeatloomDb::list_handoffs()`

#### 上下追溯

- **向上**：handoff → workitem → owner seat → session / role / delegation
- **向下**：workitem → handoff → artifact / review / pipeline / checkpoint

#### 当前判断

- **PARTIAL**

原因不是“没有表”，而是：

1. 当前这批对象是**真实历史的精选重建**，不是完整 runtime replay。
2. 一个真实 session 期间发生的细粒度状态变更，不一定逐条持久化为 session state transition rows。
3. 部分状态仍然需要依赖 `canonical_events.payload` 或外部文档解释。

也就是说：

- **能表达主干事实**；
- **不能声称保留了每一次细粒度输入输出的逐条原样运行史**。

### E. 产出物：Artifacts 与 typed documents

#### 当前协作中的真实输出

- PRD/spec/review/acceptance/memory/governance docs
- 交付包 / 修复包 / 验证包
- 未来还会包括 test report、diff summary、screen capture、pipeline outputs 等

#### 当前 SeatLoom 处理方式

1. `artifacts` 存 metadata：
   - `template`
   - `subtype`
   - `title`
   - `summary`
   - `source_session_id`
   - `source_workitem_id`
   - `storage_path`
2. `documents` 存可解析 markdown 文档正文与结构。

#### 获取

- metadata：seed 导入
- body：
  - `run_reconcile()` 负责 bounded markdown ingest
  - `scripts/ingest-documents.sh` 负责将一组 curated 关键文档的 full body 直接灌入 `documents.body_text`

#### 当前判断

- **Artifact metadata：PARTIAL**
- **Typed markdown documents：PASS**

原因：

- 文档类产出已经能被 PostgreSQL 作为正文权威存储。
- 但“所有 artifact family”还没有统一进入“metadata + full body / payload + version + open/read flow”的完整模型。

### F. 文档内容点评、审阅、回流

这是你特别关心的一类，因为它关系到“文档内容点评、反馈、协作闭环”。

#### 当前已具备

- `review_threads`
- `review_comments`
- `anchor_kind`
- `anchor_ref`
- `anchor_label`
- `review_tier`
- `change_tier_record`（JSONB）
- `evidence_refs`

#### 这意味着什么

SeatLoom 已经可以把以下行为归入同一 canonical family：

- 桌面端 review comment
- supervisor-assisted comment
- 移动端短反馈回流（结构上已对齐到同一对象族）

#### 当前存储与提取

- thread 存在 `review_threads`
- comment 存在 `review_comments`
- 可按 target object、document、artifact、workitem、handoff、session 查询

#### 上下追溯

- **向上**：comment → thread → target object → evidence refs
- **向下**：review thread → follow-up work / reissue / timeline event（理论上可做；当前部分仍依赖上层产品实现）

#### 当前判断

- **PASS（作为 canonical object family）**
- 但“从 UI 真正提交这些 comment 到 PG”的 live write path 仍未完全 accepted，因此产品化闭环仍然不是满分。

### G. 交接确认、连续性、验证运行

#### 已覆盖对象

- `handoff_receipts`
- `checkpoints`
- `pipeline_runs`

#### 为什么这很重要

这三类对象决定 SeatLoom 是否能把“只是做了事情”变成“可恢复、可验收、可回放、可度量的协作真相”。

#### 当前能力

- handoff receipt 已有独立表，可审计 ACK 行为。
- checkpoint 已有 tier0/1/2、artifact refs、branch、last commit、budget、delta context。
- pipeline run 已能记录 deterministic verification / execution history。

#### 上下追溯

- `handoff_receipts` → `handoffs` → `workitems`
- `checkpoints` → `sessions` → `artifacts` / decisions / continuity tiers
- `pipeline_runs` → `workitems` / evidence refs / artifacts

#### 当前判断

- **PASS**

但也要明确：

- 这批数据当前依然主要来自**真实协作的结构化重建与已验证 seed**；
- 不是说 SeatLoom 产品本身已经在运行时自动把这些新对象持续写入 PG。

### H. Canonical event ledger 与对象引用

#### 当前已具备

- `canonical_events`
- `event_object_refs`

#### 当前用途

- 保存跨对象的状态变化与里程碑事件
- 让 timeline / audit / replay 能从统一 event family 读取
- 用 `event_object_refs` 把 event 挂到具体 object 上

#### 当前限制

- 这批事件现在更多是“审计骨架”，不是完整逐条 runtime 原始事件流。
- 某些 richer state 仍然压在 payload 中，或者只能借助外部文档解释。

#### 当前判断

- **PARTIAL**

换句话说：

- **足以支撑审计和追溯的主干**；
- **不足以承担完整 raw runtime replay 的唯一载体**。

### I. Prompt blocked / mobile action / channel receipt / live mutation

这是当前**最关键的未闭环区**。

#### PRD / INT / Acceptance 已经要求了什么

- `US-P0-11`：prompt blocked 必须有分类、策略、动作、审计
- `US-P0-13` ~ `US-P0-15`：mobile approve/reject/escalate、feedback 回流、interrupt triage 都要与桌面共享同一 canonical history
- `E-10`：prompt-state auditability
- `E-11`：跨渠道 action 必须可审计、可回放

#### 当前 backend 真实状态

- 已接受范围内：**还没有完整闭环**。
- 当前工作树：**已经出现 draft schema 005 与 repository methods**，意图补齐：
  - `prompt_instances`
  - `prompt_actions`
  - `channel_action_receipts`

#### 为什么我仍然不给 PASS

因为“repo 中有草稿 / delivery”不等于“我们可以对外说这一层已经被接受为产品真相”。

当前仍缺：

1. 明确 acceptance closure
2. commit-pinned verifier 对该 packet 的独立闭环说明
3. 从实际 SeatLoom 产品动作进入这些表的 live ingress 边界
4. 与 optimistic concurrency / idempotency / resulting canonical event 的完整 accepted contract

#### 当前判断

- **HOLD**

### J. SeatLoom 上线后的“新增数据如何被抓取写入 PG”

这是你之前反复问到的关键点，也是本报告必须明确回答的地方。

**当前设计没有被已接受地覆盖完整。**

现在已接受的强项是：

- repo markdown → PG 的 reconcile 路线
- curated real-history seed → PG 的 baseline bootstrap 路线

但真正还没完全落定的是：

- **当 SeatLoom 自己成为工作台后，用户在 UI 上新建 / 修改 / 批准 / 拒绝 / 评论 / takeover / prompt approve 时，这些新增数据如何直接、规范地进入 PG**。

也就是说，当前已接受设计还没有完整回答以下问题：

1. **写边界是谁**：Tauri command / service / repository 的 canonical mutation edge 是什么？
2. **幂等怎么做**：重复点击、重复提交、网络重放如何避免重复写入？
3. **并发冲突怎么做**：expected revision / resulting revision / conflict result 如何统一？
4. **actor/channel 怎么归因**：desktop / mobile / supervisor / system 的责任如何结构化记录？
5. **状态变化与审计如何一一对应**：每个 mutation 是否都落到 canonical event + receipt？

所以，对这个问题的直接回答是：

> **现在的 backend 还没有完成“SeatLoom 上线后新增协作数据直接由产品抓取并规范写入 PG”的完整 accepted 设计。**

## 目前已经能做的“上追下追”能力

### 1. 从文档往下追

可以做到：

- `document` → `document_sections`
- `document` → `document_versions`
- `document` → `document_associations`
- `document.assoc` → `workitem` / `session` / `handoff` / `project`
- `document.artifact_id` → `artifacts`

这意味着：

- 从一份 PRD / review / acceptance 出发，能往下追到相关任务、会话、交接、产出物。

### 2. 从任务往上追

可以做到：

- `workitems` → `owner_seat_id`
- `workitems` → `handoffs`
- `workitems` → `artifacts.source_workitem_id`
- `workitems` → `review_threads.workitem_id`
- `workitems` → `pipeline_runs.workitem_id`

这意味着：

- 从一个 WorkItem 可以追到谁负责、有哪些交接、有哪些产出、有哪些评审、跑过哪些验证。

### 3. 从 session 往上往下追

可以做到：

- `sessions` → `seats`
- `sessions` → `checkpoints`
- `artifacts.source_session_id` → `sessions`
- `review_threads.session_id` → `sessions`
- `checkpoints.artifact_ids_at_checkpoint` → `artifacts`

这意味着：

- 从一个 Session 可以追到 continuity snapshot、关键产出、相关评论与恢复线索。

### 4. 从 review / comment 往上追

可以做到：

- `review_comments` → `review_threads`
- `review_threads` → `target_kind + target_id`
- `review_threads` → `document_id` / `artifact_id` / `workitem_id` / `handoff_id` / `session_id`
- `review_threads.evidence_refs` → evidence objects

这意味着：

- 评论不是漂浮文本，而是已经可以挂到对象、文档段落或证据链上。

### 5. 还做不到完全闭环的追溯

目前仍然做不到完整闭环的典型链条：

- live prompt detected → policy classified → approve from mobile → input injected → resulting session resume → resulting event emitted → receipt stored → UI replay

原因不是“没有任何设计”，而是**这一条链还没有以 accepted canonical runtime authority 的形式完成。**

## 对“当前数据是否准确”的判断

### 可以认为准确的部分

- Seat / role / delegation 主干事实
- 关键 workitem / handoff / review / acceptance / checkpoint / pipeline 骨架
- 文档内容本身、文档 section、文档 version、文档检索面
- 评审线程与评论对象族

### 必须承认是“有损重建”的部分

- sessions 的真实细粒度运行过程
- workitem / handoff / event 的全量逐条时序
- 一部分 artifact family 的正文 / payload
- 某些对象间关联依赖 seed，而不是自动解析 / 自动维护

### 因此更准确的表述

> **当前 PostgreSQL 中的数据是“真实协作的可信结构化重建 + 文档正文权威化 + 审计骨架”，不是“全量原样 runtime 逐条镜像”。**

## SeatLoom backend 还需要追加的要求（基于 PRD / INT / Acceptance）

### Requirement 1: 建立已接受的 live write ingress contract

必须补齐一条明确的产品写入边界：

- 谁能写：desktop / mobile / supervisor / system
- 写什么：workitem state, handoff state, review comment, prompt action, gate decision, approval action
- 如何写：typed repository contract
- 幂等键：必须有
- 并发保护：expected revision / resulting revision 或等效 guard
- 成功 / rejected / conflict / noop 的结果枚举：必须有
- resulting canonical event linkage：必须有

### Requirement 2: Prompt lifecycle 进入 accepted canonical truth

需要正式收口为已接受对象族：

- prompt instance
- prompt action history
- bounded preview / evidence reference
- policy classification
- allowed actions
- supervisor assist budget accounting
- result event linkage

### Requirement 3: Channel action receipt 统一 desktop / mobile / supervisor

所有状态变化动作都应该进入同一 canonical receipt 层，至少覆盖：

- approve
- reject
- escalate
- reserve desktop takeover
- return
- comment submit
- stop

并记录：

- actor
- channel
- target object
- note / evidence
- expected revision
- applied revision
- receipt status
- resulting event

### Requirement 4: document associations 自动维护

当前 `document_associations` 不应长期依赖 seed。

需要后续补齐：

- reconcile 后的 deterministic association extraction，或
- 明确的人机确认绑定流程，写入 PG 成为 canonical relation

否则文档到对象的追溯会长期停留在“部分人工整理”。

### Requirement 5: runtime transcript / bounded evidence strategy

需要明确 accepted policy：

- 原始 transcript 是否入库
- 入库到什么粒度
- prompt bounded preview 与 raw transcript 的关系
- transcript_tail_ref 指向什么 canonical 存储
- 是否按 session chunk 归档

现在 checkpoint 里已有 `transcript_tail_ref` 字段，但还没有已接受的 transcript authority family。

### Requirement 6: artifact 正文 / payload 统一策略

现在 markdown 文档的正文权威化已经较强，但非 markdown artifact 仍未统一。

后续至少要分清：

- 哪些 artifact 只存 metadata + file path
- 哪些 artifact 要存 structured payload / extracted text
- 哪些 artifact 要有 version / diff
- 哪些 artifact 只作为 external blob reference

### Requirement 7: baseline reconstruction 与 live product truth 的双轨规则

必须明确区分两条路径：

1. **historical bootstrap**
   - source map 驱动
   - 允许 curated reconstruction
2. **live product truth**
   - runtime direct write
   - 不允许靠 memory/seed 再补回主状态

否则未来会混淆：

- “历史导入对象”
- “产品实时生成对象”

## 最终结论

如果问题是：

> “我们目前协作开发 SeatLoom 产生的所有数据流，在 backend 中都有能力把信息结构化/半结构化，并梳理清楚数据流，能向上向下追溯了吗？”

我的结论是：

- **大体上，已经做到了很大一部分，而且底座是靠谱的。**
- **但还不能说已经全部做到了。**

更精确地说：

1. **历史协作真相层**：已经相当强，足够支撑真实数据驱动的 UI、检索、审阅、连续性和大部分追溯。
2. **产品运行时真相层**：还没有完整闭环，尤其是 prompt / channel action / live write ingress / concurrency guard。
3. **所以现在适合做的事**：继续让 Nimbus 只做基础设施，把 runtime authority 与 direct-write contract 补齐，而不是回退到 file-only 或直接跳到业务代码。

在补齐上述 HOLD 项之前，最诚实的结论应当是：

> **SeatLoom backend 现在已经能承载“真实协作历史的结构化/半结构化真相层”，但尚未完成“SeatLoom 作为未来主工作台时，对所有新增协作数据的产品级权威写入与全链路追溯”。**
