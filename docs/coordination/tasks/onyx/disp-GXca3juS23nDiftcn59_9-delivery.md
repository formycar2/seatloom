# 交付:合并 track 解冲突(两边都留)+ 退役 026 的 DO-NOT-APPLY 警告 + 报告一次我自己的险肇

| Field | Value |
|---|---|
| id | ONYX-2026-10-30-disp-GXca3juS23nDiftcn59_9 |
| template | T7 |
| subtype | implementation-delivery |
| status | delivered |
| author | seat-onyx-001 |
| date | 2026-10-30 |
| version | v1 |
| tags | merge, conflict-resolution, apply-db, near-miss, guard, vocabulary |
| responds to | 派单 `disp-GXca3juS23nDiftcn59_9`(implementation) |

## 结论先行

三件事都做了:合并 track 解掉唯一冲突(两边都留)、退役 026 的过期"不要 apply"警告、
按 Flux 重验结果更新登记。**另外报告一次我自己的险肇** —— 我在验 apply-db.sh 的时候
用 `--help` 跑了一次,它没有 `--help` 处理,而默认 URL 指向 live。**没有造成任何写入**,
但阻止它的不是任何守卫,而是缺一个 psql 二进制。我加了个真守卫。

## 1. 冲突:两边都留,不互相覆盖

冲突只有一处:`scripts/land-023-memory-dedup.sh` 的文件头注释。双方改的是同一段,
但改的是**不同的东西**,所以两边都留:

| | 我的改动 | Aegis 的改动 |
|---|---|---|
| 文件名 | 保留 023(张先生已敲过) | 不动 |
| `MIG=` | 指向 `025_memory_dedup.sql` | 不动 |
| 打印的消息 | 全说 025(不指着另一个文件) | 不动 |
| 分支查找 | 就近查 track,再依次查两个 packet 分支,找不到就 FAIL 不猜 | 不动 |
| group 查询 / DELETE | — | **删掉 `WHERE l2_file_path IS NOT NULL`** |
| 文件头数字 | — | 从自己报错的"6 组、最坏一组 4 行"改成 5 组 2 行 |

**他删的那个过滤器是对的,我照单全收**:索引键是 `coalesce(...)`,NULL 路径的行也参与
唯一性,把它们过滤掉的报表会看不见它们。live 今天恰好零行 NULL 路径,所以当时无害 ——
但报表应该匹配索引真实的键,而不是它的一个子集。

**我的分组键说明和他的自我更正说明并排放,而不是互相覆盖。** 因为这是同一条规则的两半:

- 按宽口径(`project_id, topic_id, kind, path, line_start`)分组 → 算出 **15 组**。
  **我犯过这个错。**
- 按分组键分组但少报一列 → 把 **5 组读成 6 组**。**他犯过这个错。**

合并进注释的原话:

> AND GROUP ON THE COLUMNS YOU REPORT. Aegis's original "6 groups, one with four
> rows" came from grouping on four columns while selecting and printing three:
> three groups sharing one file printed as three copies of the same six rows,
> which reads as one group of six. The count was wrong, not the rows. Group on
> the key, print the key, and never widen one without the other.

NULL 过滤器的删除是干净合入的,不在冲突里。核对过 `DELETE` 的 `PARTITION BY` 和
`025_memory_dedup.sql` 的索引键**逐字相同**,且整个脚本没有任何地方残留
`l2_file_path` 过滤。

## 2. 退役 026 的 DO-NOT-APPLY 警告(Flux 重验发现)

`scripts/apply-db.sh` 第 99 行的 026 条目还写着
`*** KNOWN DEFECT, DO NOT APPLY AS-IS ***` 并把缺陷详细展开 —— 而那个缺陷在
`d4a924c` 就修了,Flux 也重验通过了(`ev-5hJw9hwiEaHjp-TH7BKOI`)。

一条"不要 apply"的指令贴在一个现在可以安全 apply 的文件旁边,**正是会被人照着做的
那种过期指令**。改成记录"曾经有缺陷、已修、已重验",保留全部历史(原清单丢哪 8 个类型、
多出哪 4 个幽灵值、两个方向的实测、修法),再加上重验结果。

顺手修了同文件 025 条目里指向 `scripts/land-025-memory-dedup.sh` 的**悬空引用** ——
那个文件不存在。真实名字是 `land-023`,并说明脚本名保留 023 但 `MIG=` 和所有消息都是 025。

## 3. 险肇报告:我用 `--help` 跑了一次 apply-db.sh,默认 URL 指向 live

这是我这单最该报的事,比前两件重要。

**我做了什么**:为了验 apply-db.sh 的语法,我跑了 `bash scripts/apply-db.sh --help`。

**发生了什么**:它没有 `--help` 处理。`set -euo pipefail` 之后直接进入执行流程,
默认 `DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom` —— **live**。
而 `localhost:5432` 从这边是**可达的**。

**为什么没有造成写入**:`psql` 不在 PATH 上(它只活在 podman 容器里)。
脚本在打印完第一条计划项之后死在 `psql: command not found`,`exit 127`。

**这不是一个守卫。** 阻止它的是缺一个二进制,不是一个检查、不是一个确认、
不是一个 dry-run 默认值。换一台装了 postgres-client 的机器,它会照原样 apply 到 live。

**我量的**(每条都能报错):

| 检查 | 值 | 含义 |
|---|---|---|
| 重跑并捕获退出码 | **127**,`psql: command not found` | 死于第一条 SQL 之前 |
| `023 fn_gate_is_gate_id` | **0** | 023 从没开始 |
| `023 fn_gate_row_is_gate` | **0** | 同上 |
| `023 v_dispatch_gate_held` | **0** | 同上 |
| `025 memories_dedup_key` | **0** | 025 没跑 |
| `026 topics_closed_at_matches_status` | **0** | 026 没跑 |
| `026 TopicClosed` 在 vocab 里 | **0** | 同上 |
| live public 函数数 | **69**(不是 clone 上的 71) | 023 的两个函数不在 |
| live 数据行 | 前后一致 | seed 没新增任何行 |

为了排除"其实跑了但幂等所以看不出来",我在一个 **live schema 的 clone** 上单独跑了
`023_gate_criterion.sql`:它干净应用,函数数 69 → **71**。live 仍是 69。**023 确实没跑。**

**修法**:给 `apply-db.sh` 加了真守卫。

- **默认 dry-run**,必须显式 `--apply` 才写。
- `--help` / 任何未知参数 → **退出 2** 并打印用法,不再当成"跑一下看看"。
- `psql` 不在 PATH → **在打印计划之前**退出 127 并说明它只活在容器里
  (半张计划看起来像半个迁移已经跑了,这比没有输出更坏)。
- dry-run 里每一项都打 `WOULD APPLY (dry run — nothing written)`,结尾明确说
  `NOTHING WAS WRITTEN`。

**验证守卫本身**(用 PATH shim 注入一个假的 `psql`,它被调用就记日志):

| 调用 | 结果 |
|---|---|
| `apply-db.sh --help` | `exit 2`,打印用法,**不调 psql** |
| `apply-db.sh`(无参数) | 打印完整计划,每项 `WOULD APPLY`,**调 psql 0 次** |
| `apply-db.sh --apply` | 打印 `APPLY — this will write to ...`,**调 psql** |

检查是"`--apply` 有没有被传",不是"URL 看起来危不危险" —— 一个显式导出 `DATABASE_URL`
指向 scratch 库的人已经做了主动选择,在那里也要求 `--apply` 就是狼来了。

## 4. 派单要求的四项验证

| 要求 | 结果 |
|---|---|
| `git merge track/infa-foundation` into branch, resolve, push | ✅ `91474bf`,两个远端都推了 |
| `land-023-memory-dedup.sh` 仍打印正好 5 组 | ✅ report-only 实跑,5 组、每组 2 行、dry run 零写入 |
| 仍说 `migration 025 is ON TRACK` | ⚠️ **当前为假**,见下 |
| DELETE 无 `l2_file_path IS NOT NULL` 过滤 | ✅ 整个脚本零处残留 |
| UI 和 Rust 套件仍绿 | ✅ 见下 |

**"ON TRACK" 这一条我不能靠现在跑出来冒充。** 025 还在我的 packet 分支上,没合并,
所以脚本现在说的是 `on packet/onyx/topic-opened-019 only — NOT on track`。
这是**事实**,不是要修的 bug —— 合并是张先生的 Accept,不是我做的。

我在一个隔离 clone 里模拟"张先生 Accept 之后"的状态单独验了这条:
脚本确实打印 `migration 025 is ON TRACK (...)`,同样 5 组。验完就删了那个 clone,
没动真实的 ref。

**套件**:
- `cargo check --workspace` ✅
- `cargo clippy -p seatloom-core --all-targets -- -D warnings` ✅
- `cargo test --workspace` ✅ 零失败(215 + 95 + 21 + 16 + 6 + 3 + 1 等)
- **UI**:我这边的 diff **零个 `ui/` 文件**。主检出的 `ui/` 源码和我的**逐字节相同**
  (逐个文件 diff 过),且它的 `node_modules` 装着,所以在主检出跑:
  **370 过、2 失败**。这 2 个在主检出的**干净 track 上同样失败**
  (`git status` 对 `ui/` 干净),是既有失败,不是我引入的 ——
  `src/App.restartRestore.test.tsx` 里两个重启恢复测试。

## 5. 顺带确认:重命名在合并结果里是对的

`023_gate_criterion.sql` / `025_memory_dedup.sql` / `026_topic_close.sql`,
**不含** `023_memory_dedup.sql` 或 `024_topic_close.sql`。
`023_gate_criterion` 保住 023,和 Aegis 的结论一致。

## 6. 边界

- 没有 rebase ✅(merge)
- 没有合进 track ✅(那是张先生的 Accept)
- 没有把脚本改名成 land-025 ✅(派单明确要求不要)
- 没有改 `025_memory_dedup.sql` / `026_topic_close.sql` 的 SQL 内容 ✅
- **live 零写入** ✅(见 §3 的逐项测量;那次险肇死于第一条 SQL 之前)

## 7. 需要你跟进的

1. **合并这个分支**(`packet/onyx/topic-opened-019` → track)—— 025/026 落地、
   上面那条 ON TRACK 变成真的,都靠这一步。
2. **`scripts/land-023-memory-dedup.sh --apply`** —— 5 组重复,逐组问你哪行存活。
3. **`apply-db.sh --apply`** —— 合并之后跑,注意现在必须带 `--apply`。
4. **UI 那两个既有失败**(`App.restartRestore.test.tsx` 的重启恢复)—— 不是我的活,
   但它在主检出的干净 track 上就红着,谁 merge 到 track 都会带上。
