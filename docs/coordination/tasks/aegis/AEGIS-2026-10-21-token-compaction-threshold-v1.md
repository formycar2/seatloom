# Token 消耗优化:第一刀已完成,阈值是根因

| Field | Value |
|---|---|
| id | AEGIS-2026-10-21-token-compaction-threshold-v1 |
| target_seat | seat-aegis-001 |
| issued_by | 张先生(经 saih 转达) |
| date | 2026-10-21 |
| 状态 | 第一刀已完成,待你重启席位生效 |

## 先说结论:根因不是"没开压缩"

你之前大概率和我一样,会以为四个席位 `compaction` 计数为 0 是没配置。**不是。**

`compaction.enabled` 默认就是 `true`。真实原因是阈值设在了几乎不可能到达的位置:

```ts
// packages/coding-agent/src/core/compaction/compaction.ts:267
return contextTokens > contextWindow - settings.reserveTokens;
//                                              ^^^^^^^^^^^^^^^^ 默认 16,384
```

你的模型全部声明 `contextWindow = 1,000,000`,所以:

```
触发阈值 = 1,000,000 − 16,384 = 983,616 token
```

各席位 session 实际累积在 100–500 万 token。理论上早该触发,但计数为 0。

**这和之前 `Request timed out` 是同一个病根**:不是不压缩,是压缩来得太晚——上下文先触及网关极限,把回合拖垮,压缩根本轮不到执行。

## 实测数据(全量清点,含 .saih 与 .pi 两套 session)

**总量 ≈ 2592 万 token**

| 类别 | token | 占比 |
|---|---|---|
| `toolResult` | 15,219,566 | 58% |
| `toolCall` | 6,086,533 | 23% |
| `thinking` | 4,181,927 | 16% |
| `assistant` 正文 | 433,196 | 1% |

**按席位**:AEGIS(主)1053 万 · nimbus 476 万 · mira 315 万 · flux 290 万 · lyra 208 万 · vela 130 万 · onyx 121 万

**Aegis 一个席位占全队 41%。**

**compaction 计数**:AEGIS×3 · nimbus×3 · mira×1 · **flux/lyra/onyx/vela = 0**

未压缩的后果是复利式浪费——累积的 toolResult 在每一个后续回合被完整重发。

## 已做的改动

`~/.saih/agent/settings.json`(席位无项目级覆盖,全局生效):

```json
"compaction": {
  "enabled": true,
  "keepRecentTokens": 120000,
  "reserveTokens": 780000
}
```

| | 旧 | 新 |
|---|---|---|
| 触发阈值 | 983,616 token | **220,000 token** |
| 压缩后保留 | 20,000 token | **120,000 token** |

保留量从 2 万提到 12 万:压太狠会丢任务上下文,反而拖累席位协作。

**`:pi/agent/settings.json` 没动** —— saih 树才是活跃的,`.pi` 已被卸载仅作历史保留。

## 已验证 / 未验证(这条请你看住)

**已验证**:配置被 harness 正确读取(`reserveTokens` 覆盖默认 16384);阈值判定函数按新值正确分流;harness 能正常跑通(`stepplan-wash/step-3.7-flash` 返回 OK)。

**未验证**:**真实压缩动作没有端到端跑过一次。** 那需要累积 22 万 token,成本不划算。我依据的是触发函数源码 + 配置生效验证,不是实测的 compress 事件。

所以请你重启席位后,**核验方式**:席位跑一段时间后,查它的 session 是否出现 `compaction` 条目。如果计数仍为 0,说明我的判断有漏,告诉我。

## 需要你做的

1. **重启四个零压缩的席位**(flux/lyra/onyx/vela)——已跑的长 session 不会追溯压缩,新阈值只对后续回合生效
2. 重启后核验 `compaction` 条目是否开始出现
3. 若阈值不合适(22 万可能偏激进或偏保守),按实际任务长度调 `reserveTokens`

## 排除项(不是瓶颈,别浪费时间)

- 派单 payload:216 条,平均 1769 字节,总计 382KB。可忽略。
- 指令文件:`AEGIS-HEARTBEAT.md` 6.8KB、`knowledge/` 84KB、`memory/` 184KB(25 条)。量级正常。
- user/system prompt 占比极低,提示词工程本身精简,无浪费。

## 接下来

~~第二刀:`write` 工具回传全文,686 次共 139 万 token~~

**更正(我做完了才发现,这条撤回):**
我原先判断 `write` 回传全文、可省 139 万 token。读了源码和实测,这个归因是错的:

```ts
// packages/coding-agent/src/core/tools/write.ts:88
content: [{ type: "text", text: `Successfully wrote to ${path}` }]
```

回传侧**本来就只有一行路径**。实测:686 次调用
- `toolCall` arguments(**生成侧**)1,715,784 tok
- `toolResult`(回传侧)31,476 tok,占 **1%**

那 172 万 token 的成因是模型**生成** `{path, content}` 时必须输出全文,是 `write` 工具的固有
生成成本,不是回传浪费。要降只能改变工作方式(用 `edit` 做增量、或用 bash heredoc),
那不是 harness 能"优化"的,属于协作方式取舍。

第三刀:压缩时优先丢弃 `thinking`(419 万),需单席位试点验证质量不降,谨慎推进。
