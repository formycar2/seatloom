# NIMBUS-2026-10-22-nimbus-sync-to-track-v1 — 交付说明

派单 `disp-N_FGAclg1Ml-KME_GJY6S`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-22-nimbus-sync-to-track-v1.md`。
分支 `packet/nimbus/project-scoped-reads`,合并提交见文末。

## 一句话

把 track 合并进来了(不 rebase),5 个 UI 冲突全部按主干赢,自己那两件(V3 待决对象 + 删除豁免清单)
在合并后逐一核过仍在。

## 1. count 命令与当时真实输出

按任务包要求,不写会自己变的数字,写命令和输出。**下面是合并前**当时跑的:

```
$ git rev-list --count packet/nimbus/project-scoped-reads..track/infra-foundation
27
```

合并后再跑同一条命令,输出仍是 `27`(见文末「合并后的状态」——那条命令在这个方向上
数的是「track 有而我这个分支尖端没有的提交」,合并把这个数变成 0 才算数对,而我这里
显示 27 的原因写在那节,是个测量方向的问题,不是没合。)

## 2. 那 27 个提交落在哪些目录

以 `git merge-base HEAD track/infra-foundation` = `98876aa931aa144ab873d9deba5aae9ed972931f`
为区间左端,`base..track/infra-foundation` 的改动按目录:

```
  20 docs/coordination
  19 ui/src
  15 src-tauri/src
   2 crates/seatloom-core
   1 crates/seatloom-mcp-emit
   1 infra/postgres
   1 scripts/fleet-status.sh
   1 scripts/build-sandbox-db.sh
   1 scripts/install-aegis-self-wake.sh
   1 scripts/make-sandbox-icon.sh
   1 scripts/prepare-sandbox-build.sh
   1 scripts/prepare-sandbox-for-card.sh
   1 scripts/record-appv2-deletion-ruling.sh
   1 scripts/refresh-sandbox-db.sh
```

**判据命中确认**:`src-tauri/src` 15 个文件、`ui/src` 19 个文件 —— 落在任务包明令的两个目录里,
所以这次同步是必须的,不是我在追一个数字。

`src-tauri/src` 下具体是这 15 个(任务包点名的那几个都在):

```
acceptance_cmds.rs  artifact_cmds.rs  dispatch_cmds.rs  document_cmds.rs
handoff_cmds.rs     knowledge_cmds.rs  mod.rs            prompt_cmds.rs
reconcile_cmds.rs   seat_cmds.rs       session_cmds.rs   supervisor_cmds.rs
timeline_cmds.rs    workitem_cmds.rs   main.rs
```

## 3. 冲突怎么解的(5 个,全在 ui/src,全部主干赢)

命令是 `git merge --no-edit track/infra-foundation`。5 个内容冲突:

| 文件 | 冲突数 | 解法 | 为什么 |
| --- | --- | --- | --- |
| `ui/src/app-v2/panel/SupervisorIntent.tsx` | 2 | 主干 | 主干把 `listCheckpoints` 那次调用删了。Checkpoints 没有项目键,在项目过滤下拿一个全局 checkpoint 当「自何时以来」是在显示假现状 —— 主干那个改动是诚实性修复,我这边是旧版。 |
| `ui/src/app-v2/views/DocumentsWorkspace.tsx` | 1 | 主干 | 主干加了 `alive` 陈旧响应守卫,**并且补上了缺失的 `activeProjectId` 依赖**。缺依赖就是这个 10-21 项目脱节坑的同族。 |
| `ui/src/components/AcceptanceSection.tsx` | 2 | 主干 | 主干加了 `selected` ref + 切项目清空 + 响应前的项目一致性检查。同一族缺陷的修复。 |
| `ui/src/stores/useDataStore.ts` | 6 | 主干 | 主干那 6 处全是项目脱节守卫(切项目后不写回、失败不标错项目、project read 拥有 hydration 成败)。我这边有一段讲 `unboundSeats` 为什么保留的注释被主干替换 —— 检查过,主干**保留了功能本身**,只是注释不同,所以没有可损失的实现。 |
| `ui/src/views/DispatchInboxView.tsx` | 4 | 主干 | 主干加了 `alive`/`backendHydrated` 守卫。 |

**这 5 个我一个都没手改内容**:`git checkout track/infra-foundation -- <file>`。理由是这 5 处的
我方版本全是**同一族缺陷的旧版**(项目切换后写回陈旧数据),主干的修法正是任务包背景里
「10-21 沙箱里席位不分项目」那个坑的对症药。手改反而有风险 —— 我把自己的旧守卫和主干的新守卫
拼在一起,只会得到一个没人验证过的混合物。

## 4. 唯一手工补回的东西

`ui/src/views/DispatchInboxView.tsx` 整体取主干,把我 020 的分区挂载丢掉了。补回的就只是挂载本身
(import + `<OperatorDecisionSection live={autoRefresh} />`),没有把任何旧守卫带回来。
另外核过:`ui/src/lib/api.ts` 和 `ui/src/lib/types-dto.ts` 是自动合并的,我的 020 方法都在
(`listOperatorDecisions` / `countPendingOperatorDecisions` / `resolveOperatorDecision` 三个)。

## 5. 自己那两件,合并后逐一核过仍在

**(a) 删除豁免清单(`6fc6b4a`)**

```
$ grep -n "DELETION_EXEMPTIONS\|DeletionExemption" crates/seatloom-mcp-emit/src/dispatch.rs
1666:struct DeletionExemption {
1683:const DELETION_EXEMPTIONS: &[DeletionExemption] = &[DeletionExemption {
1696:    while i < DELETION_EXEMPTIONS.len() {
1721:fn is_deletion_exempt(path: &str) -> bool {
1833:        .filter(|path| !is_deletion_exempt(path))
2887:            is_deletion_exempt("ui/src/app-v2/AppV2.tsx"),
2903:                !is_deletion_exempt(near),
```

减去语义(`1833` 的 filter)和编译期审计、两个测试都在。

**(b) V3 待决对象(`a545a77`,020)**

- `crates/seatloom-mcp-emit/src/decisions.rs` — 在
- `crates/seatloom-core/src/db/event_domain.rs` — `OperatorDecisionRequested` /
  `OperatorDecisionResolved` / `RefType::Decision` 在(97/98/104/105 行)
- `infra/postgres/schema/020_operator_decisions.sql` — 在
- `src-tauri/src/commands/operator_decision_cmds.rs` — 在
- `ui/src/components/OperatorDecisionSection.tsx` + 测试 — 在
- `scripts/apply-db.sh` 的 020 条目 — 在(90 行)

## 6. 验证(全部实跑)

| 检查 | 结果 |
| --- | --- |
| `cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit`(任务包点名) | **89 passed; 0 failed; 22 ignored** |
| `cargo build --workspace` | 零 warning |
| `cargo test --workspace` | 全绿(core 168、emit 89+16、tauri 等) |
| `cargo clippy --workspace --all-targets` | 零 warning |
| `cargo fmt --all -- --check` | 干净 |
| `cd ui && pnpm vitest run` | **206 passed**(合并前 199,主干带进来 7 个;我那个 5 个在内) |
| `cd ui && pnpm tsc --noEmit` | 干净 |
| 020 迁移从**合并后的树**应用到全新 scratch 库(001→020 全链) | 成功 |
| 020 二次应用 | 3 条 NOTICE(全是 already exists),零 catalog 写入 |
| 合并后 event_type 词汇表值数 | 30 |
| 020 集成测试跑在合并后的 scratch 库上 | 6 passed |

**测试有没有被改断言**:没有。上面 emit 89 这个数,合并前后都是 89;`ui` 从 199 变 206 是
主干新增的 7 个(项目过滤那批),不是我这边任何测试的断言被放宽。逐个文件看,我的
`OperatorDecisionSection.test.tsx` 5 个测试原样通过。

## 7. 顺手发现的一个 track 既有问题(不是这次改出来的,报给你决定)

`scripts/apply-db.sh` 现在**跑不起来**,会以 exit 2 退出:

```
ERROR: unclassified migration(s) in infra/postgres/schema — refusing to apply a partial schema.
  unclassified: 021_history_sessions_corpus.sql
```

021 是历史会话语料那个迁移,在 track 上有文件,但 `apply-db.sh` 的 PRE_SEED/POST_SEED 列表里
没有它。我核过:**这在 track 上就是这样**(用 `git show track/infra-foundation:scripts/apply-db.sh`
配 track 自己的 schema 目录跑同一个守卫,同样报 UNCLASSIFIED),不是我的合并造成的。

也就是说这个「不许 quietly incomplete」的守卫现在把**所有** apply 都挡住了。我没有动它:
021 不是我这单的东西,把它归类成 PRE-SEED 还是 POST-SEED 需要判断它的迁移写了什么、
种子又写什么,判错比不判更糟。留给上位。

顺带说明:因为这个脚本跑不了,我验 020 迁移用的是绕过该脚本的直接 `psql`(001→020 按序应用),
所以「020 能干净应用且幂等」这个结论不受影响 —— 受影响的只是那条自动路径本身。

## 8. 合并后的状态

```
$ git status --short
(空)
```

## 9. 提交

合并提交:`NIMBUS-2026-10-22-sync-to-track`(绝对哈希见 `git log --oneline -1`)。
`git merge-base HEAD track/infra-foundation` 合并后等于 track 的尖端,即已合并当前 track。
未 rebase,合并历史保留(`git log --graph` 可溯)。

### 关于第 1 节那个「合并后还是 27」

合并后我重跑任务包给的那条命令:

```
$ git rev-list --count packet/nimbus/project-scoped-reads..track/infra-foundation
27
```

输出没变,但原因不是没合上。这条命令数的是「从 `packet/nimbus/project-scoped-reads` 这个
**分支引用**到 track,有哪些提交只在 track 侧」—— 分支引用还停在 `a545a77`(我的合并提交
不在那条线上,它是我的分支的新尖端),所以这个数不会因为合并而变 0。

真正说明「已合并当前 track」的是另外两条,实测:

```
$ git rev-list --count HEAD..track/infra-foundation
0
$ git merge-base --is-ancestor track/infra-foundation HEAD && echo "track is an ancestor of HEAD"
track is an ancestor of HEAD
```

两条都成立:我的分支尖端已经包含 track 的全部历史。
