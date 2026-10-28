# 同步分支(含 src-tauri 改动)+ 让 Flux 重验重启回原位

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-mira-resync-2-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |

## 好消息先说:你开卡那道门开了

Nimbus 的删除豁免清单已由张先生接受、合进 track(`3e29017`),
track 上 `DELETION_EXEMPTIONS` 命中 7 处。**所以 `AppV2.tsx` 那个删除不再拦你的卡。**

## 两件事

### 一、同步分支(这次有实质内容,不是纯我的文档)

你落后 track **19** 个提交,而那区间里有落在 `src-tauri/src/commands/` 下的改动:
`acceptance_cmds.rs`、`artifact_cmds.rs`、`dispatch_cmds.rs`(Nimbus 的活合进来了)。

**不同步就合,会把这些 Rust 文件退回旧版。** 这正是 10-21 那个
「沙箱里席位不分项目」的同源坑。

做:`git merge --no-edit track/infra-foundation`
冲突按主干赢,除了你自己那四件(净化 / 标识条 / V5 / 重启回原位)。

### 二、让 Flux 重验「重启回原位」

那条验绿还是 10-21 的 **fail**,而你修完是 246/246 全过。
**但自己修自己说不算**,需要 Flux 独立重验一次才算数。

验法(写进卡里):切到某项目某标签 → 关掉 → 重开 → 回到原处;
记录的项目不存在时行为明确。

### 交付说明怎么写(判据已改,别追数字)

不给「落后数 == 0」(那数字会自己变)。给:
```
git rev-list --count packet/mira/knowledge-panel..track/infra-foundation = <当时输出>
那区间里的提交落在哪些目录:<列出来>
```
用绝对哈希 + 标题,不写 `HEAD~n`。

## 明确不做

- 不改已修好的实现
- 不 rebase
- 不为让数字变 0 做任何操作(该判据已作废)
- 不合并、不自己开验收卡

## 交付

更新你已有的交付说明,追加一段:同步了什么、Flux 重验结果、开了哪几张卡。
