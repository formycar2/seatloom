# 同步分支:落后 26 个,其中含别人改 src-tauri 的提交

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-nimbus-sync-to-track-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 前提 | 你的 V3 待决对象已由 Flux 独立验绿 pass,活本身没问题 |

## 先说清楚:你的活没问题,是分支状态挡住验收

`a545a77`(operator decisions)经 Flux 独立验绿 **pass**(`ev-qGwjjmysUqELKo2haQP5c`),
Aegis 也逐条核过派单的四条硬要求(只有操作者与 Aegis 能建 / 缺 choices 或
consequence 即拒 / resolved 不可变 / 迁移不灌种子),89/89 测试过。

**但 `packet/nimbus/project-scoped-reads` 落后 track 26 个提交,而且那 26 个里
有落在 `src-tauri/src/commands/` 下的改动**(acceptance_cmds.rs、artifact_cmds.rs、
dispatch_cmds.rs、document_cmds.rs、handoff_cmds.rs、knowledge_cmds.rs、
mod.rs、prompt_cmds.rs 等)。

也就是说:**你分支上的 Rust 侧是旧的。** 现在合进去,会把这些文件退回旧版 ——
那正是 10-21 「沙箱里席位不分项目」那个坑的成因(分支缺了别人改的代码,
沙箱显示一份假现状)。

## 为什么这次要同步(判据是范围,不是数字)

Aegis 刚作废了「落后数 == 0」这个判据 —— 它是个会自己变的数字,追它没终点
(Mira 为此连写 5 个提交去追,那个浪费是我造成的)。

**新判据:那区间里不得有落在 `ui/src` 或 `src-tauri` 下的提交。**

你这次正好命中:26 个提交里有 `src-tauri` 的改动。所以必须同步。

## 要做

1. `git merge --no-edit track/infra-foundation`
2. 冲突按主干赢,除了你自己那两件(V3 待决对象 + 删除豁免清单)——
   那两个是你的活,别被覆盖掉
3. 合并后跑 `cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit`,要全过
   (V3 那批测试 + 既有防护测试都要在)
4. 交付说明里写(不写会自己变的数字,写命令和当时真实输出):
   ```
   git rev-list --count packet/nimbus/project-scoped-reads..track/infra-foundation
     = <当时真实输出>
   git log --oneline <同区间> 落在哪些目录:<列出来>
   ```
5. 说明里用**绝对哈希 + 标题**引用提交,不写 `HEAD~n`

## 验收

- 已合并当前 track
- 交付说明给了 count 命令与当时真实输出,并列出区间内提交落在哪些目录
- `cargo test` 全过(89 + 原有,不许有测试被改断言)
- `git status` 干净
- 代码实现**不改**(V3 和豁免清单都已验绿,别顺手重构)

## 明确不做

- 不改 V3 待决对象、不改豁免清单的实现
- 不 rebase(保留可追溯的合并历史)
- 不为让数字变 0 做任何操作(该判据已作废)
- 不合并、不自己开验收卡
