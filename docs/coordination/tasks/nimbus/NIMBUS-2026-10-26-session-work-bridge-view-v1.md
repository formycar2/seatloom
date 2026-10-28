# NIMBUS-2026-10-26-session-work-bridge-view-v1 — 交付说明

派单 `disp-XnCxvGI88oAbdGcyeefhl`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-26-two-gates-blocked-v1.md` 的 **A 部分**。
契约 `docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`
§11.2 slice 1(在 `packet/lyra/milestone-summary` 上,工作树里没有)。
分支 `packet/nimbus/project-scoped-reads`。

## 交付物

一个只读视图:`v_memory_session_join`。文件 `infra/postgres/schema/022_session_work_bridge_view.sql`,
已登记进 `scripts/apply-db.sh` 的 POST-SEED 组(带完整理由)。

零行数据写入 —— 视图没有行。它不改 `memories` 或 `archived_sessions` 任何既有列的行为。

## 契约要求它回答的那个数,现在能答了

契约 slice 1 的原文:「这个视图必须回答一个数:**有多少 memory 带着一个解析不到任何行的
session 引用**。这个数是承重墙。」

在真实数据上(90 条 `source_kind='archived_session'` 的 memory):

```
 resolved_by | 行数
-------------+------
 both        |   87     <- 两个键都同意,健康状态
 a1_path     |    3     <- 只有路径键解得到,source_ref 解不到
 a2_source_ref|   0
 unresolved  |    0     <- 真正谁都解不到的:0
```

**以及两个键各自的判定(没有合并成一个数):**

```
 total | a1 | a2 | missing
-------+----+----+---------
    90 | 90 | 87 |       0
```

### 这 3 行是本次最重要的发现

`codex:019dda18-07f6-79e0-8008-e4eb16a5448d` 这个 source_ref,按契约的次级键 A2
(source_ref 冒号分割对 `(harness, id)`)解不到 —— 但按主键 A1(路径对路径)**解得到**。
这不是四舍五入的误差,是真实的键间分歧,正是契约存在要surface的那类漂移:
021 之前 3 个 codex 引用解不到任何行而产品什么都没报,一个 session 看起来像什么都没产出。

所以视图**并列报两个键**,再加一个 `resolved_by` 列说明是谁解到的,
再加一个 `missing` 列指名缺什么(session 引用 / 文件指针)。
一个只走 A1 的视图会把那 3 行藏起来;一个只走 A2 的视图会把它们误报成孤儿。

**承重断言不是「全都解得到」,而是「这个数可测」。**

## 判据逐条(都可重跑)

| # | 判据 | 实测 |
|---|---|---|
| 1 | `v_memory_session_join` 真存在,`select count(*)` 能跑 | `90`(真实数据下) |
| 2 | 视图能把 session 和工作关联起来,给行数+样例 | 见上;样例:3 条 decision/fact memory 关联到 `claude-code`/`b0100cff-…`,该 session `message_count=751` |
| 3 | 迁移幂等可重复跑 | 第二次 apply 成功、无 error、catalog 视图数不变(仍 1) |
| 4 | 应用后 `sessions` 表仍是 5 行 | 种子前 5、022 后 5、二次 apply 后 5 |

## 验证怎么做的(不省略关键的一步)

任务是可重跑的,所以我全部重跑了,但**先说一个我差点漏掉的问题**:

第一次我在一个只跑 schema、不跑 seed 的 scratch 库上验,`sessions` 表 0 行 ——
我按判据 4 记下「0 行」就错了。0 是因为**我没跑种子**,不是迁移清了表。
修正做法:按 `apply-db.sh` 的**完整规范顺序**重建(PRE-SEED 6 个 → 4 个种子 → POST-SEED 14 个,
含我的 022),在**有种子**的库上重测判据 4,得到 5 → 5 → 5。

同类问题还有一处:那个 schema-only 的 scratch 库里 `archived_sessions` 是 183 行(021 的加载),
live 是 272 行,于是 3 行 codex 显示 `unresolved`。**我没有把它当成视图的 bug**,
而是查出原因:那 3 行的 `l2_file_path` 指向的 session 只存在于 live(021 之后又有 out-of-band 加载)。
把那一行补进 scratch 后,视图输出 `both 87 / a1_path 3 / unresolved 0`,与 live 完全一致。
这证明**逻辑对,差的只是数据**。

种子 001 第一次也失败过(缺 009_runtime_triple,`default_harness` 列不存在)——
因为我自己简化了链路。按规范顺序跑就通过了。这个教训是:**验证 schema 迁移必须跑完整规范链路**,
简化的链路会制造假失败(或假通过)。

## 明确不做的,确认没做

- **没有造数据**。视图零行插入;为验证往 scratch 里灌的 90 条 memory + 1 条 session
  是从 live COPY 出来的真实行,scratch 库已删。
- **没有改那三张 blocked 单的状态**(那是账本,不是席位的权限)。
- **没有合并代码**。
- 没有改 `memories`/`archived_sessions` 的任何既有列。

## 需要上报

1. **`scripts/apply-db.sh` 仍然跑不起来**,因为 `021_history_sessions_corpus.sql` 至今
   不在 PRE_SEED 也不在 POST_SEED 列表里 —— 我在 10-22 同步单里报过,现在核过还在。
   那个完整性守卫会 exit 2 挡住所有 apply。我的 022 已经正确分类,但**只要 021 还悬空,
   这条自动路径对谁都是断的**。这是 Aegis/操作者的决定,不是我的。
2. **A2 那 3 行要不要修,是操作者的决定。** 它们是 codex 的 `source_ref` 用了文件名的
   形态,而 021 把 `id` 重键成了原生 id。修法有两种(改 3 条 memory 的 source_ref,
   或给视图再加一条回退匹配),都涉及「要不要动已写入的 memory 行」——
   015/018 的规则是 forget + remember,旧行留墓碑。我没有擅自选。
   现在这个状态是诚实的:数字可见、被点名、与 87 行分开统计。
3. **契约文件不在工作树里**。`LYRA-2026-10-16-session-work-bridge-contract-v1.md`
   只在 `packet/lyra/milestone-summary` 分支上。我从那儿读的 §11.2 slice 1 和 §9。
   如果它该进主干,那是另一件事。

## 提交

`feat(022): session↔work bridge view — v_memory_session_join`(绝对哈希见 `git log --oneline -1`)。
含迁移 + `apply-db.sh` 登记 + 本说明。
