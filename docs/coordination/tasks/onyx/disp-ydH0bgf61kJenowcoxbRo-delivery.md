# 交付:修正交付说明里的命令名 + land 脚本的过期注释

| Field | Value |
|---|---|
| id | ONYX-2026-10-30-disp-ydH0bgf61kJenowcoxbRo |
| template | T7 |
| subtype | implementation-delivery |
| status | delivered |
| author | seat-onyx-001 |
| date | 2026-10-30 |
| version | v1 |
| tags | correction, delivery-note, land-script, dedup, numbering |
| responds to | 派单 `disp-ydH0bgf61kJenowcoxbRo`(implementation) |

## 结论先行

派单说"两处小修,迁移本身没缺陷"。确认,两处都是我上一份交付里的错,都改了。
另外修了两处派单没提、但同一个脚本现在跑不起来的实际问题 —— 不修的话张先生照着
我上一份交付去跑那条命令,会直接 FAIL。

## 1. 交付说明里的命令名(按派单要求:改说明,不改文件名)

`scripts/land-025-memory-dedup.sh` **不存在**。真实文件名是 `scripts/land-023-memory-dedup.sh`。

派单的裁定我同意也照做了:**保留 023 是对的**。理由是改名会破坏张先生已有的调用习惯,
而文件内部 `MIG=` 已经指向 `025_memory_dedup.sql` —— 名字和内容不一致是可解释的,
一个他敲不出来的命令不是。

但我上一份交付把这件事写坏了:第 49 行说"脚本名里的 023 我没改",第 137 行却让他
跑 `land-025`。**同一份文档自己反对自己。** 现在两处都统一成 `land-023`,
并在命令块上方写明"脚本名带 023、内部已指向 025",免得下一个人再当 bug 报一次。

## 2. 脚本注释里的 6 组(按派单要求:改成 5 组并引用分组口径)

原注释写着 Aegis 10-29 报错的"6 组,其中一组 4 行"。**正确值是 5 组、10 行、每组最多 2 行。**

我 10-30 已独立复测,这单又复测一遍,和派单给的口径一致:

```sql
select count(*) as groups, sum(n) as rows, max(n) as max_rows_in_a_group
from (
  select count(*) n from memories
  group by coalesce(l2_file_path,''), coalesce(l2_line_start::text,''),
           coalesce(l2_line_end::text,''), l0_summary
  having count(*)>1
) t;
-- groups | rows | max_rows_in_a_group
--     5 |   10 |                 2
```

按派单要求,**分组口径写进了注释**,包括为什么会算错:

> 索引的键是 `(l2_file_path, l2_line_start, l2_line_end, l0_summary)` 各自 coalesce,
> **不含** `project_id` / `topic_id` / `kind`。按宽口径分组会算出 15 组 ——
> 但那描述的是这个索引没有的问题,不违反索引的行不是它能拒的重复。

我自己就按宽口径算错过一次(15 组)。把它写下来的理由和派单一样:
**让下一个人不能重犯我犯过的错。**

## 3. 顺带修的两处(派单没提,但不修张先生的命令会 FAIL)

这两处不修,我上一份交付给出的命令根本跑不起来:

**a) `BRANCH` 变量指向错误的分支。**
脚本里 `BRANCH="packet/nimbus/project-scoped-reads"`,而 025 现在在
`packet/onyx/topic-opened-019` 上。我实际一跑就撞到了:

```
FAIL: cannot find infra/postgres/schema/025_memory_dedup.sql on track or on packet/nimbus/project-scoped-reads
```

改成**就近查 `track/infra-foundation`,再依次查两个 packet 分支**;都找不到就
`FAIL` + `exit 2`,**不猜**。原来找不到时会走到一条写死 `on $BRANCH` 的死消息上
(单数分支名,即使我在的地方不对它也只会怪 Nimbus 的分支)。

**b) 脚本打印的 "landing 023" 全部改成 "landing 025"。**
脚本名保留 023,但它跑的是 025 —— 输出不能指着另一个文件。共 5 处用户可见消息
(`migration 023 is ON TRACK` / `on $BRANCH only` / `Landing 023 should just work` /
`land 023` / `landing 023…`)。文件头加了一段说明名字和内容为什么不一致。

## 4. 验证

**report-only 实跑**(不带 `--apply`,**零写入**):

```
migration 025 is on packet/onyx/topic-opened-019 only — NOT on track.
  Landing it on live before it is on track means live runs code the
  integration branch does not have. Consider merging the branch first.
the unique index is NOT on live — the guard is not enforced there.

== duplicate groups on live (what blocks the index) ==
  group 1: 2 rows — ...NIMBUS-2026-10-13-supervisor-intent-tools-surface-delivery-v1.md lines (null)-(null)
  group 2: 2 rows — ...NIMBUS-2026-10-13-supervisor-intent-tools-surface-delivery-v1.md lines (null)-(null)
  group 3: 2 rows — ...NIMBUS-2026-10-13-supervisor-intent-tools-surface-delivery-v1.md lines (null)-(null)
  group 4: 2 rows — ...flux/disp-_TS3QA0Mg563KTzW2bi2b-delivery.md lines 1-5
  group 5: 2 rows — ...AgentHistory/.../01565715-3e7f-48c8-9ff0-fdd8edbfdf9f.jsonl lines 963-1223

  5 group(s). Each needs a human decision: which row is the real one.
  This script will NOT choose. Deciding is a judgement about recorded
  facts, and a mechanical 'keep the oldest' would be a guess.

dry run. Nothing was written.
```

**脚本自己数出 5 组,和我独立复测一致。**

live 前后对比(确认零写入):

| | 之前 | 之后 |
|---|---|---|
| canonical_events | 2842 | 2842 |
| memories | 421 | 421 |
| `memories_dedup_key` 索引 | 0 | 0 |

`bash -n` 通过。`cargo fmt --check` 只有 `crates/seatloom-core/tests/pipe_guard_real_steal.rs`
一个既有失败(track 上的文件,不从 packet 分支去格式化它)。

## 5. 对派单里"你写对了什么"的回应

派单核了我四项(026 值域是早期设计版、空库上 exit 0 静默毁掉、清单等于 live 30 值
+ TopicClosed 一个不少、apply-db.sh 两条都登记了、我拒绝自己 apply 改过内容的 026)。
**五项我都认,没有要补充的。** 特别是"派 Flux 重验 026"—— 我等的就是这件事。

派单还说它一开始把 live 值域误读成 26 个(那是 distinct 计数不是值域)。
**这个误读的方向和我犯的错是同一种**:拿一个间接数字当真相。
它的纠正(`ev-correction-duplicate-group-count`)我引用了,`source_ref` 记的是这个派单。

## 6. 张先生下一步(不变)

```bash
bash scripts/land-023-memory-dedup.sh          # report-only,已实跑零写入
bash scripts/land-023-memory-dedup.sh --apply  # 逐组问你哪行存活
```

026 等 Flux 重新验绿之后再 apply。
