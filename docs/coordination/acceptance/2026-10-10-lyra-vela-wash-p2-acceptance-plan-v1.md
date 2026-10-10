| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-vela-wash-p2-acceptance-plan-v1 |
| title | Acceptance plan — D5 Phase 2 Lane C Part 2:S3 抽取(六类 typed record)/ S4 分层 / S5 入库 |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | AEGIS-2026-10-10-washing-pipeline-design-v2.md, VELA-2026-10-10-washing-pipeline-v1.md, VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md |
| tags | acceptance-plan, d5-phase2, vela, washing-pipeline, p2 |

# Acceptance plan — Lane C Part 2(S3/S4/S5)

**对象**:dispatch `disp-wash-vela-p2`(seat-vela-001)· 分支 `packet/vela/washing-pipeline`
· 前置:Part 1 已交付、018 + typed memory tools 两门已落地(硬顺序)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么

流水线后半(唯一 LLM 段 S3):从剪枝后的 turn 流抽**六类 typed record**
(decision|failure|fact|preference|artifact|unresolved,每条带 confidence 与 L2 行号区间,
凑不出行号不得入库);S4 分层(L0 一句话 / L1 够决策 / L2 路径+digest+行号,绝不拷贝原文);
S5 幂等入库(斜杠路径→projects 树,深环检测是流水线责任;段→topics;记录→memories,
只走 seatloom-core 记忆写路径,行+事件同事务;按内容摘要去重、按 session sha256 checkpoint)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认历史真正变成了「管得住的知识」:抽出来的记录是提炼不是复述、confidence 标注诚实、每条都能按路径+行号回溯到原文实证、失败类记录带根因和修法 —— 知识层从空表变成可召回的资产,且 O1/O2/O3 有了回填基础。",
  "preconditions": [
    "Part 2 已交付并 Flux 验绿(commit-pinned);Part 1 已验收或同批验收",
    "交付物在手:p2 交付文档(record 集统计与 confidence 分布 / S4 分层样例 / S5 幂等证明 / 门禁①–⑤证据)",
    "live 或演示库可查 memories(经 recall 工具,不经 psql)"
  ],
  "steps": [
    { "action": "看 S3 产出统计:六类 record 各多少条、confidence 分布(quoted/inferred 占比)", "input": "p2 交付文档 §S3 统计" },
    { "action": "抽 3 条人读 L0/L1:判断是真抓住要点还是复述原文", "input": "任选 3 条(尽量覆盖 decision/failure/unresolved 各一)" },
    { "action": "对同 3 条检查 confidence 标注是否诚实(inferred 的是否真的是推断)", "input": "同上 3 条的原文上下文(按 L2 指针打开)" },
    { "action": "L2 回溯抽验:随机 5 条,按路径+行号真的打开原文确认有据", "input": "随机 5 条的 l2_file_path + l2_line_start/end" },
    { "action": "用 recall 工具抽一条 failure:确认带 topic 祖先链、可按需取 L1/L2", "input": "MCP recall(project_id='seatloom', topic_id=<某个已入库 topic>)" },
    { "action": "看 S5 幂等证明:重跑不重复入库;深环检测证据", "input": "p2 交付文档 §S5" }
  ],
  "expected": [
    { "output": "S3 统计:failure 类不为空(实证:最可执行的一类);六类分布合理", "expect": "凑不出行号的记录数为 0(门禁:不得入库);confidence 二值分布可见" },
    { "output": "L0/L1 抽读:是提炼不是复述", "expect": "门禁④:L0 一句话说清要点;L1 够决策;不是原文搬运" },
    { "output": "confidence 抽检:标注诚实", "expect": "quoted 的确实原文明说;inferred 的确实有推断成分" },
    { "output": "L2 回溯:随机 5 条全部按路径+行号开得出实证", "expect": "门禁⑤:有一条对不上即缺陷(承重墙裂缝)" },
    { "output": "recall 抽验:返回带 topic 链,l1/l2 按需披露", "expect": "每次 recall 恰一条 MemoryRecalled 留痕;零 psql" },
    { "output": "S5:重跑零重复;深环检测有证据(或明确报告无环)", "expect": "按内容摘要去重生效;session sha256 checkpoint 可续跑;发现环会停并上报而非静默跳过" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-vela-001)

```
dispatch_id=disp-wash-vela-p2
branch=packet/vela/washing-pipeline
base=track/infra-foundation
how_to_verify=六类 typed record 入库(凑不出行号的为 0);抽读 L0/L1 是提炼非复述且 confidence 诚实;L2 回溯随机 5 条全部开得出实证;recall 带 topic 链可召且全程零 psql;S5 重跑零重复
flux_verdict_ref=ev-flux-verdict-wash-p2(预先申判的判定事件 id;若 Flux 实际用了别的 id,以实际为准并在交付文档记录)
acceptance_plan=<§2 JSON 逐字>
```
