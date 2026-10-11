# SeatLoom 运行知识库

Aegis 的记忆落在这里,而不是 Aegis 自己那个 agent memory 目录。

**为什么搬:** 那个目录在张先生的 Mac 上,每次写入都要他 approve。
于是他出门或睡觉时 Aegis 就记不了东西 —— 而「记不了」恰好发生在最该记的时候(出事的时候)。
放在仓库里,Aegis 自己就能维护。

**为什么是这个形状:** frontmatter 的字段**对齐 `memories` 表的列**。
等 `topics` 能创建、写入路径打通之后,洗数据流水线可以直接把这些文件灌进库,
不需要再转换一次。张先生定过:能一步到位就别做一次性的过渡产物。

## 字段

| 字段 | 含义 |
|---|---|
| `memory_id` | 唯一标识,也是文件名 |
| `project_id` / `topic_id` | 作用域。project + topic 是唯一的作用域框架 |
| `kind` | `decision` / `failure` / `fact` / `preference` / `artifact` / `unresolved` |
| `confidence` | **二元,不是分数**。`quoted` = 原文明说;`inferred` = 推断得出 |
| `status` | `active` / `forgotten` / `resolved` |
| `tier` | `cold` / `warm` / `hot` —— 生命周期 |
| `l2_file_path` + `l2_line_start/end` | **行级证据锚点**。指不到具体行号的记忆不算有证据 |
| `l0_summary` | 约 100 token,召回时默认只给这一层 |
| `curation` | `authored` = 人/Agent 真判断过;`mechanical` = 按规则映射的,**未精炼** |

`curation` 这个字段不在 `memories` 表里,是这里额外加的。
理由:机械映射的 `kind` 和 `confidence` 不能和真判断混在一起 ——
把映射结果当成判断,以后没人分得清哪条可信。灌库前该有一次精炼,这个字段就是精炼的待办清单。

## 现状

**已迁入:2 条**(Aegis 新写的)。

| 记忆 | 类型 | 置信度 | 一句话 |
|---|---|---|---|
| [aegis_never_launder_permissions_through_seats](aegis_never_launder_permissions_through_seats.md) | failure | quoted | 让席位去做 Aegis 自己被拦住的事,是绕过权限而不是解决问题 |
| [waker_idle_agent_looks_identical_to_a_working_one](waker_idle_agent_looks_identical_to_a_working_one.md) | failure | inferred | 「pane 里有 pi 活着」不是进度信号,全队因此停摆 17 小时 |

**待迁入:35 条** —— 仍在 Aegis 那个 agent memory 目录里。
批量迁移需要张先生跑一条命令(读他 Mac 上的目录 + 批量写仓库,Aegis 被权限拦住):

```
python3 scripts/migrate-agent-memory.py
```

脚本已写好并演练通过:35 条可迁(18 条 preference + 17 条 fact;3 条 quoted + 32 条 inferred),
**无任何文件因疑似密钥被跳过**。加 `--dry-run` 可先看不写。

> 置信度里只有 3 条 `quoted` 看着偏低,这是**刻意的**。判据只认「」、"原话"、"verbatim"
> 这类真正指向他原话的标记。ASCII 双引号出现在 31 个文件里,但那是代码、路径、各种引用都在用 ——
> 拿它当判据会把 Aegis 的推断标成张先生的原话,正好是 `confidence` 这个字段存在要防的事。
