| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-onyx-015-knowledge-layer-acceptance-plan-v1 |
| title | Acceptance plan — migration 015 知识层 schema(项目树 + memories L0/L1/L2 + cold→hot + 归一化规则) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-knowledge-layer-v1 |
| tags | acceptance-plan, d5, onyx, migration-015, knowledge-layer |

# Acceptance plan — migration 015 知识层 schema

**对象**:dispatch `disp-d5-lane-a-onyx`(done)· 分支 `packet/onyx/d5-knowledge-schema` @ `4520758`
(含 track b6751a1)· Flux PASS(commit-pinned 4520758)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

SeatLoom 从「管 work」扩展到「管 knowledge」的 schema 底座:projects 加父子字段使
sf-code/storage 这类历史层级可表达;新表 memories(作用域 project+topic,三层 L0≈100 token
摘要 / L1≈2K 概览 / L2 原文指针,cold→hot 生命周期);271 行 archived_sessions 自由文本的
归一化规则落成可执行视图;事件词汇 +4(Memory*)、ref_type +memory。零行数据插入
(洗数据是 Aegis Phase 2 车道)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认知识层 schema 落地且幂等:项目树能表达历史层级、记忆底座三层与 cold→hot 生命周期就位、271 行历史 slug 有确定映射且 '(root)' 不猜已上报 —— 「管 knowledge」在结构上成立。",
  "preconditions": [
    "scratch 库可用:DATABASE_URL 指向 scratch(如 seatloom_scratch),绝不是 live seatloom",
    "psql 在 PATH;仓库 checkout 在含 015 的分支或已合入 015 的 track",
    "scratch 库已按 apply-db.sh 规范顺序应用过 001-014(+seed)"
  ],
  "steps": [
    { "action": "在 scratch 库跑一遍 apply-db.sh(含 015)", "input": "DATABASE_URL=postgresql://…/seatloom_scratch scripts/apply-db.sh" },
    { "action": "同一 scratch 库再跑一遍 apply-db.sh(第二遍,验幂等)", "input": "同一 DATABASE_URL,命令同上" },
    { "action": "查项目树与记忆底座结构", "input": "psql -c '\\d projects' -c '\\d memories'" },
    { "action": "查归一化视图对 271 行的映射分布", "input": "psql -c \"SELECT map_status, count(*) FROM v_archived_sessions_normalized GROUP BY 1;\" -c \"SELECT raw_slug, normalized_parent, normalized_leaf FROM v_archived_sessions_normalized WHERE map_status='mapped';\"" },
    { "action": "查两个词汇表 CHECK 终态", "input": "psql -c \"SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname IN ('canonical_events_event_type_vocab','event_object_refs_ref_type_vocab');\"" },
    { "action": "抽一条历史 slug 看记忆作用域字段", "input": "psql -c \"SELECT id,project_id,topic_id,l0_summary,l1_overview,l2_ref_kind,l2_document_id,l2_file_path,heat,recall_count,last_recalled_at,tier,status FROM memories LIMIT 1;\"(空表亦预期 —— 本单零行插入)" }
  ],
  "expected": [
    { "output": "第一遍 apply:015 应用成功,memories 表出现", "expect": "无 ERROR;apply-db.sh exit 0" },
    { "output": "第二遍 apply:零 catalog 写入", "expect": "前后 pg_class/pg_constraint/pg_attribute 计数与 \\d memories 完全一致(015 头部的度量方式)" },
    { "output": "projects 多出 parent_project_id;memories 含契约 §4 全列", "expect": "parent_project_id 可空自 FK、禁自引用;l0/l1/l2 三层 + l2 document|file 配对 CHECK + heat/recall_count/last_recalled_at + tier(cold|warm|hot)+ status(active|forgotten)均在" },
    { "output": "归一化分布:mapped 覆盖 10 个已知 slug,unmapped 恰为 '(root)' 13 行", "expect": "sf-code/storage 分解为 parent=sf-code / leaf=storage;'(root)' 不猜、已 SupervisorMessage 上报 Aegis" },
    { "output": "词汇表:event_type 25 值、ref_type 12 值", "expect": "含 MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten 与 memory" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-onyx-001)

```
dispatch_id=disp-d5-lane-a-onyx
branch=packet/onyx/d5-knowledge-schema
base=track/infra-foundation
how_to_verify=scratch 两遍 apply 第二遍零 catalog 写入;271 行归一化 10 mapped + '(root)' 13 unmapped 已上报;memories 三层与 cold→hot 字段与值域 CHECK 就位
flux_verdict_ref=<Flux 对 4520758 的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```
