| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-11-lyra-nimbus-knowledge-readmodel-acceptance-plan-v1 |
| title | Acceptance plan — 知识读模型:主题树 + 按主题取记忆卡(三层按需披露,界面可调) |
| status | active |
| author | lyra |
| doc_date | 2026-10-11 |
| version | v1 |
| depends_on | LYRA-2026-10-11-overnight-headline.md (DTO 契约 §3), AEGIS-2026-10-11-overnight-plan.md (step 4) |
| tags | acceptance-plan, knowledge-readmodel, nimbus, overnight |

# Acceptance plan — 知识读模型(step 4)

**对象**:seat-nimbus-001 的 read-model lane(dispatch `disp-onight-readmodel`)· 分支
`packet/nimbus/knowledge-readmodel`。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么

把召回能力接到**界面能调的地方**:按主题树取记忆,默认只给摘要,按需才给详述和证据
(渐进披露不是优化,是 token 可控的前提)。DTO 契约见拆解文档 §3(与 Mira 的派单同文)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「读出来」这一步真的通:从库里按项目拿到主题树(带记忆数),点一个主题拿到它的记忆卡列表(默认只有一句话摘要),要详述才给 L1、要证据才给 L2 行号 —— 界面面板要展示的一切,后端都已经能一次取出。",
  "preconditions": [
    "read-model lane 已交付并 Flux 验绿(commit-pinned;判定事件 id 按契约预声明 ev-verdict-knowledge-readmodel)",
    "live 库已有至少一个主题和一条真实记忆(冒烟 lane 的证据,或 Vela 写库后)",
    "SeatLoom 应用构建含新命令(knowledge 命令已注册)"
  ],
  "steps": [
    { "action": "取主题树", "input": "Tauri cmd_get_knowledge_tree(project_id='seatloom')(或经 MCP get_knowledge_tree)" },
    { "action": "点一个主题,取记忆卡(默认深度)", "input": "Tauri cmd_get_topic_memories(topic_id='<树里任一个>')" },
    { "action": "同一主题要详述", "input": "cmd_get_topic_memories(topic_id='<同一个>', depth='l1')" },
    { "action": "同一主题要证据", "input": "cmd_get_topic_memories(topic_id='<同一个>', depth='l2')" },
    { "action": "验作用域:换一个 project 取树", "input": "cmd_get_knowledge_tree(project_id='<另一个 project>')" },
    { "action": "验纯读:连续调两次,账本不变", "input": "query_ledger 计数前后对比(除 recall 类审计事件外零新增 —— 本命令是纯读,零事件)" }
  ],
  "expected": [
    { "output": "主题树:topics 平表 + parentTopicId 可组树", "expect": "memoryCount 只数 active 且未被 supersede 的;空项目返回空树(不报错)" },
    { "output": "记忆卡默认层:id/title/kind/confidence/tier/status/l0Summary", "expect": "默认 depth=l0:不含 l1Overview、不含 l2 —— 渐进披露,默认量收敛" },
    { "output": "depth=l1 才出现 l1Overview", "expect": "按需披露;token 可控" },
    { "output": "depth=l2 才出现 l2 指针", "expect": "l2 含 refKind/filePath 或 documentId + bodyDigest + lineStart/lineEnd(archived_session 来源必有行号)" },
    { "output": "返回体带 topicChain(主题祖先链)", "expect": "与 recall 工具的链纪律一致:记忆不脱离上下文返回" },
    { "output": "两个 project 的树互不串味", "expect": "作用域 project+topic 是硬边界;跨项目取不到对方主题" },
    { "output": "纯读证明:账本零新增事件", "expect": "本命令不写任何行、不发任何事件(与 recall 的审计事件区分:recall 留痕,本读模型不留痕 —— 这是界面查询,不是记忆召回)" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-nimbus-001)

```
dispatch_id=disp-onight-readmodel
branch=packet/nimbus/knowledge-readmodel
base=track/infra-foundation
how_to_verify=cmd_get_knowledge_tree 返回带 memoryCount 的主题树;cmd_get_topic_memories 默认只回 L0、l1/l2 按需、带 topicChain、纯读零事件;两 project 作用域不互串
flux_verdict_ref=ev-verdict-knowledge-readmodel
acceptance_plan=<§2 JSON 逐字>
```

## 4. 围栏(与 Mira 派单同文)

文件集见拆解文档 §3:Nimbus 只碰 db/**、mcp-emit/**、knowledge_cmds.rs(新)、
dto.rs(只增)、main.rs(只增一行)、types-dto.ts(只增)、api.ts(只增);
**禁碰 ui/src/app-v2/**;DTO 与契约逐字一致(Mira 据此并行开发)。
