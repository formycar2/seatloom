| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-vela-wash-p1-acceptance-plan-v1 |
| title | Acceptance plan — D5 Phase 2 Lane C Part 1:清洗流水线 S0–S2.5(归一化/脱敏/切 topic/剪枝) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | AEGIS-2026-10-10-washing-pipeline-design-v2.md, VELA-2026-10-10-washing-pipeline-v1.md |
| tags | acceptance-plan, d5-phase2, vela, washing-pipeline, p1 |

# Acceptance plan — Lane C Part 1(S0–S2.5,零 LLM)

**对象**:dispatch `disp-wash-vela-p1`(seat-vela-001)· 分支 `packet/vela/washing-pipeline`
· scope:seatloom 16 sessions / 168MB / 58,015 行。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么

历史清洗流水线的前半(零 LLM):S0 把 16 个 session 归一成 canonical turn 流
(codex+claude-code 两个必做 adapter,cursor 读已有 chat_dump.json,gemini 只登记);
S1 脱敏(ak-*/sk-*/ghp_*/AKIA*/JWT/PEM/Bearer/65 字符 stepplan key/cookie/内网域名+高熵
启发 → ⟪REDACTED:类型:指纹⟫);S2 切 topic(静默>30min / cwd 变化 / 任务边界 / tool-call
聚簇);S2.5 剪枝(丢 tool_result 正文,留 tool 名+成败+字节数+行号区间;预期砍 80–90%)。
**门禁①就在本卡**:topic 清单要张先生肉眼判断是不是独立工作主题。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认清洗流水线前半段产出可信:16 个 seatloom session 被完整归一(未知行容错不丢文件)、敏感信息被指纹化脱敏、topic 切分看起来是一个个独立工作主题、剪枝大幅减量但不动 L2 指针 —— 值得继续花 token 跑 S3 抽取。",
  "preconditions": [
    "Part 1 已交付并 Flux 验绿(commit-pinned)",
    "交付物在手:p1 交付文档(S0 统计 / S1 命中证明 / S2 topic 树 / S2.5 剪枝统计)+ 中间产物目录",
    "已知含密钥样本清单(S1 门禁用)可从交付文档或 vela 处获得"
  ],
  "steps": [
    { "action": "看 S0 统计:16 个 session 的总行数 / 已解析 / meta 行数,按 harness 分解", "input": "p1 交付文档 §S0 统计表" },
    { "action": "抽查 S0 输出:任选 1 个 codex + 1 个 claude-code session,看 canonical turn 流是否合乎 {seq,ts,role,kind,text,tool,refs} 规范", "input": "session id 任选;中间产物路径见交付文档" },
    { "action": "验 S1:在已知含密钥样本上确认命中(替换为 ⟪REDACTED:类型:指纹⟫,指纹=sha256 前 6 位)", "input": "已知含密钥样本(交付文档列出的那条)" },
    { "action": "看 S2 topic 树:逐个 topic 问自己「这是不是一个独立工作主题」", "input": "p1 交付文档 §S2 topic 树(或中间产物)" },
    { "action": "看 S2.5 剪枝统计:原始行数 → 入 LLM 行数,按 session", "input": "p1 交付文档 §S2.5 统计表" },
    { "action": "抽 1 条被剪枝的 tool_result,确认只丢正文、保留了 tool 名+成败+字节数+行号区间", "input": "任选一条含 tool_result 的 turn" }
  ],
  "expected": [
    { "output": "S0 统计:16 session 全覆盖;meta 行有计数且不为 0 时能说出是什么未知行", "expect": "没有整文件失败;unknown 行按设计记 meta 并计数" },
    { "output": "S0 抽查:turn 流字段齐全、顺序合理", "expect": "seq 连续、role/kind 可信、refs 可溯源" },
    { "output": "S1:已知密钥样本全部命中并指纹化", "expect": "漏报 = 缺陷(门禁②不过,停线修 pattern);同值同指纹可辨不可逆" },
    { "output": "S2 topic 树:每个 topic 是一个独立工作主题", "expect": "门禁①:张先生肉眼判断通过;不过关则停(切错则所有召回建在沙上)" },
    { "output": "S2.5 统计:体量砍掉 80–90%(偏离需解释)", "expect": "按 session 给出前后行数;偏离要能说清原因" },
    { "output": "剪枝抽检:tool_result 正文已丢,指针四要素在", "expect": "剪枝只影响送 LLM 的内容,L2 指针(行号区间)未动 —— 承重墙未伤" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-vela-001)

```
dispatch_id=disp-wash-vela-p1
branch=packet/vela/washing-pipeline
base=track/infra-foundation
how_to_verify=S0 统计 16 session 全覆盖且未知行容错;S1 已知密钥样本全命中指纹化;S2 topic 树经张先生肉眼判断为独立工作主题;S2.5 剪枝 80–90% 且 L2 指针未动
flux_verdict_ref=<Flux 对 Part 1 的 ReviewVerdictIssued 事件 id(经 query_ledger 按 branch/dispatch 检索>
acceptance_plan=<§2 JSON 逐字>
```
