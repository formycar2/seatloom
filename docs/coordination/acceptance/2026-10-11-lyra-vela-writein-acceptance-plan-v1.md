| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-11-lyra-vela-writein-acceptance-plan-v1 |
| title | Acceptance plan — 真的写进库:S5 真实入库(88 条提炼记录)+ 三项统计 + 他读得懂的交付说明 |
| status | active |
| author | lyra |
| doc_date | 2026-10-11 |
| version | v1 |
| depends_on | AEGIS-2026-10-11-overnight-plan.md (step 3 + 必须有), VELA-2026-10-10-washing-pipeline-v1.md |
| tags | acceptance-plan, writein, vela, overnight, must-have |

# Acceptance plan — 真的写进库(step 3)

**对象**:seat-vela-001 的 write-in lane(dispatch `disp-onight-writein`,blocked on
`ev-typed-memory-tools-landed`)· 分支 `packet/vela/washing-pipeline`。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么

主题工具上线 + 一次真实 remember 成功(门事件)之后,把 Part 2 已 staged 的 88 条 typed
record 按 project+topic **真正落库**:每个 S2 段主题经 open_topic 开启,每条 record 经
remember 写入(行+事件同事务),幂等可续跑。交付必须附三项统计 + 一份张先生读得懂的说明。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「数据真的进去了」:库里那些记忆是从他自己 16 个历史会话提炼出来的(不是测试数据),每一条能按主题找到、能按行号回原文;并且有一份说明告诉他——提炼了多少、丢了多少、为什么丢。",
  "preconditions": [
    "门事件 ev-typed-memory-tools-landed 已落账本(一次真实 remember 已成功)",
    "Vela 的 MCP 进程是重启后的新二进制(含 open_topic/remember);live 库已 apply 015-018",
    "Part 2 的 staged 产物(88 条 record + S2 主题树 + checkpoint 状态)在手"
  ],
  "steps": [
    { "action": "看写入结果总览", "input": "交付说明 §写入统计:写入主题数 / 写入记忆数 / 按 kind 分布" },
    { "action": "按主题抽查:任选 3 个主题,数它们的记忆卡", "input": "MCP recall(project_id='seatloom', topic_id='<任选>') 或界面面板(若已上线)" },
    { "action": "抽 1 条 failure 类记忆,按 L2 行号回原文", "input": "recall(depth='l2') 取 filePath + lineStart/lineEnd,打开原文对一眼" },
    { "action": "看三项统计", "input": "交付说明:剪枝统计(原始行数→送模型行数)/ 脱敏命中数 / 因凑不出行号丢弃的记录数" },
    { "action": "看丢弃说明", "input": "交付说明:为什么丢(哪类内容凑不出证据)、丢弃占总量比例" },
    { "action": "验幂等:同一 checkpoint 重跑一次", "input": "按交付说明的续跑命令再跑一遍" }
  ],
  "expected": [
    { "output": "库里记忆数 > 0 且来源是真实语料", "expect": "source_kind=archived_session、L2 指向真实历史文件;非测试数据(唯一的例外是冒烟 lane 那条明确标注的验证记忆)" },
    { "output": "主题与记忆的归属符合 S2 分段", "expect": "每条记忆的 topic 是 S2 切出的主题;主题经 open_topic 受治理创建(无手工 SQL 播种)" },
    { "output": "抽查的 L2 行号全部打得开原文", "expect": "凑不出行号的记录数为入库前的丢弃数(入库后为 0)—— 承重墙未伤" },
    { "output": "三项统计齐备", "expect": "剪枝:原始→送模型行数(预期砍 80–90%,偏离有解释);脱敏:命中数与替换样例(⟪REDACTED:类型:指纹⟫);丢弃:因无行号丢弃的记录数与原因" },
    { "output": "重跑零重复", "expect": "按内容摘要去重 + session sha256 checkpoint;重跑不产生第二条同内容记忆" },
    { "output": "交付说明张先生读得懂", "expect": "提炼多少/丢多少/为什么丢,三段说清,不堆术语" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-vela-001)

```
dispatch_id=disp-onight-writein
branch=packet/vela/washing-pipeline
base=track/infra-foundation
how_to_verify=库里有从 16 个真实会话提炼的记忆(非测试数据);抽查 L2 行号全部回得了原文;剪枝/脱敏/丢弃三项统计齐备且有一读就懂的交付说明;重跑零重复
flux_verdict_ref=<该 lane 的预声明判定事件 id>
acceptance_plan=<§2 JSON 逐字>
```

## 4. 红线(从计划原文承接)

- 绝不为赶时间跳脱敏:脱敏在送模型之前、写库之前;漏报按缺陷,不是瑕疵。
- 绝不放行凑不出行号的记录;丢弃数必须出现在统计里,不许省略。
- 本轮范围只有 seatloom(16 sessions);扩范围须张先生另行决定。
- 写库只走 seatloom-core 记忆写路径(remember 同一条:行+事件同事务);不 raw SQL、不旁路文件。
