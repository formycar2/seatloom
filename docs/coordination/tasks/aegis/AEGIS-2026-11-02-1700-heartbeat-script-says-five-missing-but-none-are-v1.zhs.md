# Aegis 17:00 心跳交付：座位登记脚本长期说反话；席位模型已改名，脚本和知识文档未跟上

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-1700-heartbeat-script-says-five-missing-but-none-are |
| status | delivered |
| author | aegis |
| date | 2026-11-02 |
| version | v1 |
| tags | heartbeat, defect, seats, script |

## 变了什么

本轮初始 `202288404fdce59271fe68d067ceb789d73a156b`，分支 `track/infra-foundation`；
已全文读取 Aegis 心跳协议并逐项执行。自醒在 17:00 自然唤醒（日志新增 POKED），
`check-ledger-exists.sh` 35 表 1 事件；队列、48 小时内 done 无卡、pass verdict 无卡、
pending 裁定均为空；没有陈旧 `in_flight`；抢救补丁、卡检查、verdict 检查均通过但样本为空。
**空样本不等于工作已完成。**

## 抓到一处长期失效的脚本（它一直在说反话）

`bash scripts/register-fleet-seats.sh`（干跑）输出：

```
库里现有席位: seat-flux / seat-lyra / seat-mira / seat-networkorchestrator /
             seat-nimbus / seat-onyx / seat-ql / seat-stepaicloud / seat-vela
缺的是这五个(有 FK,不补就没法给它们派单):
(空)
确认无误就跑: bash scripts/register-fleet-seats.sh --apply
```

**九个席位全在，却写着「缺五个」并打印空清单，然后给出执行命令。**
代码事实：`scripts/register-fleet-seats.sh:45` 的标题是写死的字符串，
循环逐个数五个席位、只打印计数为 0 的；当五个都在时清单为空，标题仍是「缺的是这五个」。
一个清单为空的「缺五个」+ 一条执行命令，会让操作者以为还没注册而再跑一次 `--apply`。
`ON CONFLICT DO NOTHING` 让重跑无害，所以危害是误导而不是写坏数据。

**这正是我自己记录过的错误家族**：一个清单为空的断言读起来像结论。
这次它指向的方向是「缺」（我此前判过「不在」比「在」危险），而它其实不存在。

## 还有两处和今天的改动不一致，未修

1. `ae1bb07` 把 provider 改名（`stepplan*` → `sp-personal`/`sp-team`），
   live `seats.default_model` 已随之改为新名，运行中席位也显示新 provider。
   但 `scripts/register-fleet-seats.sh:164-169` 的 INSERT 仍写旧名
   `stepplan/step-5-preview`、`stepplan-wash-anthropic/step-5-preview`。
   五个核心席位已存在且 `ON CONFLICT DO NOTHING` 不会改它们，
   所以现在跑 `--apply` 不会把它们改坏；但这脚本若用于重建/新库，会写入已废弃的 provider 名。
   另有一个待执行的 `scripts/rename-stepplan-providers.sh`（预演默认），属 live 写操作，等你。
2. 新出现的未跟踪文件 `docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md`（16:59）
   是你 11-02 口述加实测的预算池映射，内容我读完了，没有改。
   它不在版本库，也未登记到知识索引；我没有替你提交或改写。

## 你会看到什么 / 需要你做什么

- 自醒、账本、席位在线、唤醒服务都正常；应用验收门可点但当前 0 张卡。
- **本会话无法改 live 数据库**，所以上面两处「脚本/文档与现状不一致」只报告，
  由你决定是否要我改脚本默认值或补登记。

## 自报

上轮我说「自醒修复后待自然唤醒验证」，16:00、17:00 两轮都拿到自然 POKED，已确认。
但**没有把「席位模型改名后全线一致」当成已验证**：我只核了 live 表和运行窗格，
没有逐个重启席位验证新 provider 生效，也没有改那份写死旧名的脚本。

未调用 remember / ArtifactCreated / complete_dispatch：本会话无 SeatLoom MCP 工具，
也无本轮生命周期事件。不伪造事件来源，不用文件冒充记忆。
你的 `.gitignore` 改动与 12:00 派单原文未动，未代为提交。
