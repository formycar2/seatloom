# NIMBUS-2026-10-10-acceptance-closeout-v1 — 两分支同步 + request_acceptance containment 防线 + 两卡收尾

**Owner:** Nimbus (seat-nimbus-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-acclose-nimbus` · **Milestone:** 验收卡 closeout(Aegis 阻塞裁决驱动,最高优)
**Base branch:** `track/infra-foundation`(@ b6751a1)· **Delivery branch:** 见 §2 分支策略
**Done-definition:** §1(同步)两分支完成 + §2(防线)落地验绿 + §3(两卡)建出 + §4 收尾走完。

---

## 0. 定位

Aegis 阻塞裁决两件事你都相关:① 你的两条已验绿 lane(血缘读模型 / D5 recall)分支未同步,
**不许提请验收**;② 「点接受会删掉已合并工作」的结构缺陷要做进产品,不靠纪律 ——
这正是 request_acceptance 的 containment 防线,与你刚做的 base 校验同一处。验收方案已由
Lyra 供稿(六份,shape v1),你只执行调用。

## 1. 步骤 1 — 两分支同步到 track b6751a1(先做)

- `packet/nimbus/acceptance-lineage-readmodel`(tip 32a3369,merge-base b84684a2)、
  `packet/nimbus/d5-mcp-recall`(tip db7e1f3,merge-base b84684a2):各自把
  `track/infra-foundation` 合进来。
- **冲突解律(硬约束):绝不删除 track 上已追踪的文件**。历史包袱 known:旧 base 删过 track
  文档、重写过 DispatchInboxView —— 合并时 track 侧内容对非本 lane 文件为准,本 lane 文件
  用你的版本;拿不准的文件 = 上报,不猜。
- 同步后**重跑门禁**:`cargo fmt --all --check`、`cargo clippy --workspace --all-targets --
  -D warnings`、`cargo test -p seatloom-core -p seatloom-mcp-emit -- --include-ignored
  --test-threads=1`、UI 侧 `pnpm typecheck` + `pnpm test`(你动到的部分)。
- 未同步的分支不许提请验收(Aegis 裁决);同步本身是新交付态 → Flux 重验(见 §2 末)。

## 2. 步骤 2 — containment 防线(request_acceptance 建卡时校验;同一处合并实现,不第二套)

在你已做的 base 校验(resolve_base + MissingBase/UnknownBase)**同一处**追加:

- **(b) branch 已包含 base**:`merge-base(branch, base) == base`(用你 `crates/seatloom-core/
  src/git/ops.rs` 里刚加的共享 git helper 家族,不引第三方);不满足 → typed 拒绝
  (`BranchNotSynced` 或同义),信息说明「请先把 track 合进分支再提请」。
- **(c) 合并不删除 base 上已追踪的文件**:预演该合并会删除的 base 已追踪文件集合
  (如 `git merge --no-commit --no-ff` 预演或等价的 tree diff:`git diff --name-status
  <merge-base-result>..` 取 D 项,或 merge-tree);**若会删 → typed 拒绝并逐条列出将被删的
  路径**(`MergeWouldDelete` 或同义)。这是「接受按钮不毁已合并工作」的产品级保证:
  onyx 分支曾会删 6 个文件(含 acceptance_cmds.rs 本体)、mira 8 个、你旧 base 12 个 ——
  此类删除今后在建卡时就被挡住,而不是等操作者点下去。
- 拒绝全部发生在**任何 DB 接触之前**(house discipline),负例测试覆盖:base 不存在(既有)、
  branch 不存在(既有)、branch 不含 base、合并会删文件、正常通过。
- **分支策略(推荐,你可按实际调整但必须声明)**:防线落在
  `packet/nimbus/acceptance-lineage-readmodel`(你第一条要建卡的分支);两分支都同步到 track
  后,把同步后的 lineage 分支合进同步后的 recall 分支(**一个方向**,避免 dispatch.rs 分叉),
  使两分支都含防线且无分歧;最终分支血缘写进交付文档(既有 chain:waker-hardening →
  lineage → recall,保持单向无分叉)。
- 同步 + 防线落地后:**自请 Flux 对两分支新 tip 重验**(commit-pinned),作为建卡的
  flux_verdict_ref。你自己的卡因此成为防线的第一个正面实证(分支含 base、合并不删文件)。

## 3. 步骤 3 — 两卡收尾(方案由 Lyra 供稿)

卡 1 — 血缘读模型:
```
request_acceptance(dispatch_id="disp-accplan-lane-b-nimbus",
  branch="packet/nimbus/acceptance-lineage-readmodel"(同步后 tip), base="track/infra-foundation",
  how_to_verify="对 acc-disp-me6Nm8e9D6ZRe4fKyVokY 调血缘命令一次返回上游派单/事件链/Flux 判定/产物/依赖边;缺 plan 的 request_acceptance typed 拒绝且零写入",
  flux_verdict_ref=<同步后 Flux 重验判定>,
  acceptance_plan=<docs/coordination/acceptance/2026-10-10-lyra-nimbus-lineage-readmodel-acceptance-plan-v1.md §2 逐字>)
```
卡 2 — D5 recall:同式,`dispatch_id="disp-d5-lane-b-nimbus"`,
`branch="packet/nimbus/d5-mcp-recall"`(同步后 tip),
`acceptance_plan=<…-nimbus-d5-recall-acceptance-plan-v1.md §2>`。

**注意**:若 Onyx 的 017/Mira 的卡先建,你的防线会对它们生效(它们分支已含 base,应通过)——
不要因他人卡片异常而改弱防线,异常即上报。

## 4. 收尾流程

1. 按 §1/§2 提交(commit 带 dispatch id;分支血缘声明)。
2. 自请 Flux 重验两分支新 tip(commit-pinned)。
3. 按 §3 建两卡(在 Flux 验绿之后)。
4. 写交付说明 `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-closeout-delivery-v1.md`
   (同步证据含「同步后分支包含 track tip」的 rev-parse 输出、防线设计与负例矩阵、两卡
   acceptance_id、分支血缘)。
5. `complete_dispatch(dispatch_id="disp-acclose-nimbus", outcome="done")`。
6. **席位永不自己合并。**

## 5. 红线

- 冲突解律硬约束:绝不删除 track 已追踪文件;防线是产品行为,不靠「记得别点」。
- 值域/形状与 Lyra 契约逐字一致;不第二套 request_acceptance 实现。
- 事件驱动零轮询零查库(纯读工具已上线);不整分支合并;不碰 `infra/**`(017 是 Onyx 的)。
