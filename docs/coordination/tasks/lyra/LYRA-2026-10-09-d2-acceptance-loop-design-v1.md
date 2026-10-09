# D2 产品设计 — 在界面里验收一个里程碑并闭环 (acceptance gate)

**Owner:** Lyra (PO). **Status:** ACTIVE — rulings issued, lanes dispatched.
**Date:** 2026-10-09. **Milestone:** D2 (优先级 D2 > D3 > D4 > D1).
**Governing dispatch:** `disp-lyra-d2-milestone`.

---

## 0. 定位与北极星

用户的主交互是一个主动的 Assistant(飞书 robot,后端 Aegis):主动盯进展、把需决策的事备好推给用户。
**SeatLoom PC UI 是下钻后备,不是主路** —— 但 D2 的验收闭环先在 PC UI 落地(D1 再把同一条
acceptance 读模型推送到 IM,对象设计为此预留)。

合并准则(操作性定义):一组交付**只有在 Flux 技术验证通过 + 用户验收"用户可感知结果"后才合入
track**。每个"完成" = 一个用户日常使用的闭环场景。D2 就是把这条准则变成系统能力。

## 1. 用户闭环场景(D2 的全部交互)

```
席位做完一组交付(分支 packet/<seat>/<id>)
  → (1) 用户被告知:"有交付待验收:X,Flux 已验绿、零回归,
       一步可验:做 A 看到 B,建议接受"        [UI 待验收区;IM 推送是 D1]
  → (2) 用户一步验证(按 how_to_verify 做)
  → (3) 点"接受" → 自动合并进 track → 显示合入 commit
  → (4) 或"打回 + 一句反馈" → 自动生成带反馈的新派单,回到该席位(waker 唤醒)
  → (5) 用户看到闭环结果(时间线事件 + 收件箱新派单 / 合入记录)
```

## 2. 数据模型 — 新对象 `acceptances`(迁移 014,Onyx)

验收是**独立一等对象**,不是 dispatch 的一个状态:它的 actor 是用户/发起人(不是席位),
生命周期不同,且自带 payload(分支、怎么验、合并 commit)。dispatch 行保持"工作单元"语义
(012/013 不动),验收行通过 dispatch_id 建立血缘。

```sql
acceptances (
  id                 TEXT PRIMARY KEY,            -- acc-<dispatch-id>
  dispatch_id        TEXT NOT NULL REFERENCES dispatches(id),
  project_id         TEXT NOT NULL REFERENCES projects(id),
  branch             TEXT NOT NULL,               -- 被验收的交付分支 packet/<seat>/<id>
  base               TEXT NOT NULL DEFAULT 'track',
  how_to_verify      TEXT NOT NULL,               -- 一步可验:"做 A 看到 B"
  flux_verdict_ref   TEXT NOT NULL,               -- Flux 验绿判定(ReviewVerdictIssued 事件/artifact id)
  status             TEXT NOT NULL,               -- pending | accepted | rejected
  feedback           TEXT,                        -- 打回反馈(issuer 原话)
  rework_dispatch_id TEXT,                        -- 打回自动重派的新派单(血缘)
  merge_commit       TEXT,                        -- 接受时的合并 commit
  decided_by         TEXT,                        -- 决策者 actor_ref(操作者)
  decided_at         TIMESTAMPTZ,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
-- status 域 CHECK IN ('pending','accepted','rejected');accepted/rejected 终态不可变
-- (同 013 理由:dispatches 行是账本的投影,终态行回退=静默篡改已记录事实)
-- 索引:(dispatch_id)、(status) — 待验收列表 = WHERE status='pending'
-- 约束:同一 dispatch 同时最多一个 pending(部分唯一索引,防重复提请)
```

**Flux 验绿复用现有事件类型 `ReviewVerdictIssued`**(12 值词汇表里已有),不新增类型:
refs subject=<dispatch 或 workitem>,payload={verdict:"pass",evidence:"..."}。`flux_verdict_ref`
引用该事件/artifact 的 id,接受前强制校验其存在(合并准则的第一半)。

## 3. 事件词汇表 — 本次 sign-off(契约扩展,我所有)

每个变迁恰好一个事件(投影可回放,契约 rule 2)。词汇表演进:

**013(Onyx 已设计,本文正式签署)** — `SeatDispatchFailed`(in_flight→failed,终态)、
`SeatDispatchRetried`(in_flight→ready,retry 重开)、`SeatDispatchCancelled`(
ready|blocked|in_flight→cancelled,终态,actor=issuer)。**全部批准。**
对 Onyx 的开放问题的裁决:**retry 重开为普通 `ready`(任何合格席位可拉),不设粘性**——
不为"偏好席位"发明契约表面;反复失败的作业应能被别的席位接手,这是舰队弹性。

**014(本次新增,本文签署)**:

| Event type | Transition | Actor | Refs |
|---|---|---|---|
| `AcceptanceRequested` | — → pending | seat | subject=acceptance, actor=seat, context=dispatch |
| `AcceptanceGranted` | pending → accepted | operator | subject=acceptance, actor=operator, context=dispatch; payload={branch,base,merge_commit} |
| `AcceptanceRejected` | pending → rejected | operator | subject=acceptance, actor=operator, context=dispatch; payload={feedback,rework_dispatch_id} |

配套词汇加宽:`ref_type` +`acceptance`(验收事件的主体类型;refs 是多态 TEXT 无 FK,
与 `dispatch` 同理)。operator 的 actor ref 用 `ref_type='seat'` + 配置的操作者 ref
(polymorphic,无 FK 强制);操作者将来有一等身份时再加 ref_type,不预先发明。

`acceptances` 行同样是投影:三个事件齐全才可回放。接受/打回后终态不可变。

## 4. 动作与合并机制(Nimbus)

**席位侧(MCP,`seatloom-mcp-emit`)——新工具 `request_acceptance`:**
```
request_acceptance(dispatch_id, branch, how_to_verify, flux_verdict_ref) -> acceptance_id
```
- actor 从 SEATLOOM_ACTOR_REF 解析(席位本人);dispatch 须属于该席位且已 complete
  (SeatDispatchCompleted 已落);flux_verdict_ref 必须存在(事件或 artifact);
  该 dispatch 无 pending 验收(唯一性)。
- 插入 acceptance 行 + 追加 AcceptanceRequested,**同一事务**(append_event_in_tx,rule 2)。
- 席位流程:complete_dispatch(工作完成)→ Flux 验绿 → request_acceptance(提请验收)。
  **席位永不自己合并。**

**用户侧(Tauri 命令,`src-tauri/src/commands/acceptance_cmds.rs`,新文件):**
- `cmd_list_acceptances` — 待验收读模型(status=pending,含 dispatch 标题/席位/分支/验绿/时间)。
- `cmd_accept_delivery(acceptance_id)`:
  1. 复查 flux_verdict_ref 存在(无则拒绝:"未验绿不可合");
  2. 在主 checkout 执行 `git merge --no-ff <branch> -> <base=track>`(经 std::process::Command);
     **冲突即拒**(清晰错误 + 事件),**永不 force**;
  3. 落 merge_commit、status=accepted、decided_by=操作者 ref、decided_at,追加 AcceptanceGranted;
  4. 返回合并结果给 UI(commit sha)。
- `cmd_reject_delivery(acceptance_id, feedback)`:
  1. status=rejected + decided_* + 追加 AcceptanceRejected;
  2. **自动重派**:经同一 write 路径 enqueue 新 dispatch——target=原席位,
     title="打回重做:<原标题>",payload={rework_of:dispatch_id, feedback, packet, branch},
     priority=原 priority+10(重做插队);enqueue 自身触发 pg_notify → waker 唤醒空闲席位;
  3. rework_dispatch_id 回写 acceptance 行(血缘)。
- 操作者身份:SEATLOOM_OPERATOR_REF(显式配置,绝不临场发明;缺省即拒绝执行并提示配置)。
- 仓库根:显式配置(env SEATLOOM_REPO_ROOT),拒绝在非 git 目录或脏得无法合并时操作。

## 5. UI 形态(Mira,`ui/src/**`)

派单页新增**「待验收」区**(与收件箱同页或同 tab 分段,5s 轮询同 inbox):
- 卡片:dispatch 标题、席位、分支、**how_to_verify(那句"做 A 看到 B")**、Flux 验绿徽章、
  issued/completed 时间;
- 动作:[接受](直接合,显示 commit 结果)/ [打回](弹出反馈输入,必填一句);
- 结果态:已合入(commit sha)/ 已打回(链接到新派单);
- 空态/错误态:合并冲突时展示 git 的冲突信息(不静默)。

## 6. Dogfooding — D2 自己是第一个走验收的交付

四席位的代码包 + Lyra 的设计文档(共 5 个分支)全部走新流程:
各席位在 Flux 对其包验绿后,自己 `request_acceptance`(lyra 文档包由我提请,引用同一
Flux D2 判定);用户在 UI 待验收区逐卡"一步验证 → 接受",分支逐一合入 track。
**这是"验收能力"本身的第一次实跑**——UI 里出现待验收卡片即证明接线成功。

## 7. Lane 划分(文件不相交,payload 自足)

| 席位 | Lane | 文件 |
|---|---|---|
| Onyx | 013 落地(用其 §4 原文)+ 014 设计落地 | `infra/postgres/schema/013_*.sql`、`014_acceptance_gate.sql`、`scripts/apply-db.sh` |
| Nimbus | MCP request_acceptance + Tauri 三命令 + 合并机制 | `crates/seatloom-mcp-emit/**`、`src-tauri/src/commands/acceptance_cmds.rs`、`src-tauri/src/dto.rs` |
| Mira | 待验收 UI | `ui/src/**` |
| Flux | 验收证明脚本 + 每包的"怎么验"那句 + D2 判定事件 | `crates/seatloom-mcp-emit/proof/acceptance_loop_proof.sh` |

依赖顺序(软):Onyx schema → Nimbus Rust → Mira UI → Flux proof 全链。各自 payload 内含
完整契约,可并行开工;集成以各自 delivery 为准。

## 8. 边界(D2 不包含)

- **IM 推送待验收给用户 = D1**(Aegis 居中 IM→派单);D2 的 acceptance 读模型按可推送设计
  (status=pending 可查),届时复用。
- GitLab 侧合并/对账 = P6(需 PAT),不做。
- dispatch 状态机的 failed/cancelled/retry(013)是 D3 自愈的地基,本次只落地 schema+Rust,
  不建自愈策略(D3)。
- 作战室/历史回放 = D4。

## 9. 给用户的一句话

D2 交付后,你的动作是:打开 SeatLoom 派单页 → 待验收区出现 D2 的五张卡片(每张带"做 A
看到 B")→ 逐张一步验证 → 点接受,看它们合入 track。第一个被这个新能力验收的交付,就是
这个能力本身。
