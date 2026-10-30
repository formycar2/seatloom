# 交付:025/026 迁移落号 + 登记,并修掉 026 一份会毁掉 live 的值域清单

| Field | Value |
|---|---|
| id | ONYX-2026-10-30-migrations-025-026-registration |
| template | T7 |
| subtype | implementation-delivery |
| status | delivered(025 可直接 apply;026 需重新验绿;apply-db.sh 已登记两者) |
| author | seat-onyx-001 |
| date | 2026-10-30 |
| version | v1 |
| tags | migration, numbering, apply-db, vocabulary, dedup, topic-close, defect |
| responds to | 派单 `disp-p-zIkNv1Z1jbUQk4voEfr`(implementation) |

## 结论先行

派单让我做三件事:解编号撞车、把两个迁移登记进 `apply-db.sh`、给出 apply 到 live 的命令。
**三件都做了,但第二件的过程中发现 026 有一个比编号撞车严重得多的问题** ——
它的事件类型值域清单是**旧的**,照原样 apply 会**替换掉 live 正确的清单**。
我把它修了,并且在 scratch 库上把两个方向都量了。

**因此 026 不能直接 apply:它的值域清单被我改过,需要重新验绿。025 可以。**

| 迁移 | 落号 | apply-db.sh | 内容状态 |
|---|---|---|---|
| `023_memory_dedup.sql` | → **`025_memory_dedup.sql`** | ✅ 已登记 | 未改内容,可直接 apply(**但会被 live 现有重复挡住**) |
| `024_topic_close.sql` | → **`026_topic_close.sql`** | ✅ 已登记 | **值域清单已修**(30+TopicClosed),需重新验绿 |
| `023_gate_criterion.sql` | **保持 023** | 原样 | 没动(见 §1) |

## 1. 编号撞车怎么解的(以及为什么不是我自己的文件让号)

撞车是真的:`infra/postgres/schema/` 里同时有 `023_gate_criterion.sql` 和
`023_memory_dedup.sql`。而且 `023_gate_criterion.sql` 的文件头写着"023 两边都空着"——
**那句话现在是假的**,023 早被 dedup 占了。

我让 **dedup 和 topic_close 让号**,`gate_criterion` 保持 023。理由:

1. `023_gate_criterion.sql` **已经在 track 上、已经登记、已经验收合并**。
   为它再改一次号,会让一个已完成并通过的产物第三次改名,并且让它现有的登记行变成孤儿。
2. 它 10-28 那次改号(020→023)时,023 确实是空的 —— 它是**先来的**。
   后来者让号,符合先占原则,也符合我自己 10-28 立的先例(那次是我让号,因为 track 的 020 先占)。
3. 让号只改文件名和自引用,不动任何 SQL 对象名和依赖。

新号:dedup → **025**,topic_close → **026**。025/026/027 都本来是空的,我查过。

同步改了 3 个引用旧文件名的脚本(这是派单没提但必须做的,否则它们会指向不存在的文件):
`crates/seatloom-mcp-emit/proof/memory_dedup_proof.sh`、
`crates/seatloom-mcp-emit/proof/topic_close_proof.sh`、
`scripts/land-023-memory-dedup.sh`(脚本名里的 023 我没改 —— 改名会破坏张先生已有的调用习惯,
但文件内的 `MIG=` 路径改了,并在交付里说明)。

## 2. 严重问题:026 的值域清单是旧的,apply 会毁掉 live

这是我这单最重要的发现,比编号撞车严重。

026 要把 `canonical_events_event_type_vocab` 加宽(加 `TopicClosed`)。
加宽的手法是 **DROP + ADD 完整清单** —— 和 019/020 一样。问题是**它写的清单是错的**:

| | 026 原稿写的 | live 实际有的 |
|---|---|---|
| `SeatDispatchIssued` / `Delivered` / `Completed` / `Failed` / `Retried` / `Cancelled` | **没有** | 有(**1371 行**) |
| `AcceptanceGranted` / `AcceptanceRejected` | **没有** | 有 |
| `DispatchEnqueued` / `DispatchStatusChanged` / `AcceptanceDecided` | **有** | 从来不存在 |

也就是说,它写的是一个早期设计版的 26 值清单,**丢掉了 8 个数据库真在用的类型,
加了 4 个哪都不存在的类型**。

我把 001-020 每个改值域的迁移都解了一遍,**只有 026 是错的**:
010=12、012=15、013=18、014=21、015=25、017=26、018=27、019=28、020=30,全是真清单;
只有 026 写 26 值且是幽灵版。live 的 30 值和 Rust 钉的 `EventType::ALL`
(`crates/seatloom-core/src/db/event_domain.rs`)**逐值相同**。

### 两个方向都在 scratch 库上量了

**方向一:有数据的库 —— PostgreSQL 直接拒绝,迁移是"无操作失败"**

```
ERROR: check constraint "canonical_events_event_type_vocab" of relation
       "canonical_events" is violated by some row
```

DROP 成功、ADD 失败,事务回滚。**看起来 apply 失败了,其实是它想做的事本身做不到** ——
在 live 上就是这 1371 行挡着。好消息:它不会毁掉任何东西。

**方向二:空库 —— 它"成功",然后静默毁掉**

把挡着的那行删掉再 apply,`exit 0`。这时候:
- **丢掉**:`AcceptanceGranted` `AcceptanceRejected` `SeatDispatchCancelled`
  `SeatDispatchCompleted` `SeatDispatchDelivered` `SeatDispatchFailed`
  `SeatDispatchIssued` `SeatDispatchRetried`
- **多出**:`DispatchEnqueued` `DispatchStatusChanged` `AcceptanceDecided`

**一个空库从零建出来,dispatch 事件一条都插不进去,而且不报错。**
这比方向一危险得多 —— 它会伪装成成功。

### 修法

把清单换成 live 实际的 30 值 + `TopicClosed`,共 31 个。
逐个值和 Rust 的 `EventType::ALL` 对过,集合和顺序都一致。
IF/ELSE 两个分支都改了(原稿两处都是错清单)。

修完重新量的结果:

| 检查 | 结果 |
|---|---|
| 有 4 行 dispatch/acceptance 事件的库上 apply | **exit 0** |
| apply 后值域 | **31 个类型**,含 `TopicClosed` |
| 6 个 dispatch 类型 | **全部还在** |
| 原有 4 行 | **一行没丢** |
| 第二遍 apply | **exit 0**,catalog 计数 `635/255/4235` 完全不变(幂等) |
| topics 关闭路径 | `open → closed` 成功,`closed_at` 自动写 |
| 配对约束 | `open` 带 `closed_at` 被拒;`closed` 不带 `closed_at` 被拒 |

## 3. apply-db.sh 登记(派单要求的规格)

两条都按现有条目的标准写:说清排序理由、PRE/POST-SEED 及为什么、以及失败时会怎样。

**025**:
- POST-SEED(和 015/018 同规则:索引的是 memories 的列,015 才建它,种子不写 memories)
- 规范顺序 `015 -> 025`,与 020-023 无关
- **明确写了它会拒**:"live 上实测有 5 组重复,唯一索引建不起来 ——
  重复→索引被拒→去重不生效→更多重复。这个死结由 `scripts/land-025-memory-dedup.sh`
  按操作者逐组决定哪行存活来解,**不是这个文件能解的**"

**026**:
- POST-SEED(topics 由 010 建,种子不写 topics)
- 规范顺序 `019 -> 026`
- **明确写了"不要照现状 apply"**,并把两个方向的实测结果、丢哪些值、多哪些值、
  修法是什么、在哪个交付说明里追踪,全写进登记行的理由里

完备性守卫跑过:`GUARD PASSES — every migration on disk is registered`。

## 4. 给张先生的命令(我不跑,因为 live 写操作在我权限之上)

```bash
# 025 —— 会失败,而且失败就是答案:它列出的 5 组重复需要你逐组决定哪行存活
bash scripts/land-025-memory-dedup.sh

# 026 —— 等 Flux 重新验绿之后再 apply(它的值域清单被我改过)
podman exec -i -e PGPASSWORD=seatloom seatloom-postgres \
  psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 \
  < infra/postgres/schema/026_topic_close.sql
```

**025 现在跑会发生什么**(我量的):它建唯一索引,而 live 有 5 组重复,
所以 PostgreSQL 会拒绝,脚本会把 5 组两行全文都列出来问你哪行留。
**这是设计如此** —— 哪一行是"真的"是关于已记录事实的判断,我不该机械地做。

**026 为什么不能现在 apply**:我改了它的值域清单(从错的 26 值改成对的 31 值)。
一个被我改过内容的迁移,拿旧的验绿去 apply 是不对的。需要 Flux 重新验一次。

## 5. 我修正了派单里的两个数字(都量过)

1. **"5 组重复"是对的,我一开始算成 15 是错的。** 我第一遍按
   `(project_id, topic_id, kind, l2_file_path, l2_line_start)` 分组得到 15 组。
   但索引的键是 `(l2_file_path, l2_line_start, l2_line_end, l0_summary)` coalesce 之后 ——
   **不含 topic_id/kind/project_id**。按索引真正的键算,**5 组、5 行冗余**。
   派单的 5 是对的,我用错了分组口径。
2. **"memories_dedup_key 索引不存在"** ✅ 确认,`pg_indexes` 0 行。

## 6. 边界

- 没有 apply 任何迁移到 live ✅(全程只读 live;所有 apply 都在一次性 scratch 库,用完 drop)
- 没有写 live 任何一行 ✅(2834 events / 420 memories,前后一致)
- 没有改 021/022 ✅
- 没有改 `023_gate_criterion.sql` 的内容 ✅(只让另外两个让号)
- 改了 `026_topic_close.sql` 的值域清单 ✅(这是必须的,它在 live 上会失败/毁库;已在 §2 说清)

## 7. 需要你跟进的

1. **026 需要 Flux 重新验绿** —— 我改了它的内容,旧验绿不覆盖。
2. **025 的 5 组重复需要你决定** —— 跑上面那条命令,它会逐组问你。
3. **派单说"023 编号撞了",但撞的方式和它描述的不完全一样**:
   它说 gate_criterion 的注释声称 023 空着但实际被占 —— 这是真的。
   但时间线上是 **dedup 先占了 023**,gate_criterion 10-28 改号时没查到底。
   两个文件现在都在 track 上了,所以这是已落地的事实,只能靠改号解。
