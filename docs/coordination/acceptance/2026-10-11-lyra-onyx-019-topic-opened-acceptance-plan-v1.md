| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1 |
| title | Acceptance plan — migration 019:TopicOpened 进事件词汇表(27→28 全量清单) |
| status | active |
| author | lyra |
| doc_date | 2026-10-11 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-11-topic-opened-019-v1.md (Lyra 签署), NIMBUS-2026-10-11-open-topic-delivery-v1.md (Rust 侧,ALL 钉 28) |
| tags | acceptance-plan, onyx, migration-019, topic-opened |

# Acceptance plan — migration 019(TopicOpened)

**对象**:dispatch `disp-019-onyx`(done)· 分支 `packet/onyx/topic-opened-019` @ `724fa7e`
(含 deb946e)· Flux PASS,预声明判定 `ev-019-topic-opened-flux-verdict`。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

Nimbus 的 open_topic 选择了**新增** `TopicOpened` 事件类型(裁决要求的二选一,两路代价已
书面论证,Lyra 签署批准);Rust 侧 `EventType::ALL` 钉 28。本交付是剩下的那一半:PG 侧
`canonical_events_event_type_vocab` 27→28 的**全量清单** guarded 加宽(013 模式)。
落地前 open_topic 在 live 上会被 CHECK 拒 —— 这是整条夜间链的 schema 前置。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认 TopicOpened 成为账本里的一等事件类型:PG 值域与 Rust 钉的 28 值逐字一致、全量清单加宽(不会 revert 015/018 的 memory 类型)、第二遍 apply 零 catalog 写入 —— 「一个主题被有意开启」这件事在账本里可查、可回放,且 track 能重建正在线上跑的值域。",
  "preconditions": [
    "019 lane 已交付并 Flux 验绿(ev-019-topic-opened-flux-verdict,PASS,commit-pinned 724fa7e)",
    "scratch 库可用(DATABASE_URL 指向 scratch,绝不是 live seatloom);checkout 在含 019 的分支",
    "Nimbus 的 Rust 侧已钉 28(packet/nimbus/open-topic)—— 本单验的是 PG 侧与它逐字一致"
  ],
  "steps": [
    { "action": "scratch 库跑两遍 apply-db.sh(含 019)", "input": "DATABASE_URL=postgresql://…/seatloom_scratch scripts/apply-db.sh(连跑两次)" },
    { "action": "查事件类型值域终态", "input": "psql -c \"SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='canonical_events_event_type_vocab';\"" },
    { "action": "与 Rust 钉的 28 值逐字比对", "input": "packet/nimbus/open-topic 的 EventType::ALL 清单(交付文档 §1 有逐字列表)" },
    { "action": "确认 memory 四类型没被 revert", "input": "值域定义里找 MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten 与 AcceptanceCancelled" },
    { "action": "查 apply-db.sh 注册", "input": "POST_SEED_SCHEMA 里 019 行 + 理由注释" }
  ],
  "expected": [
    { "output": "两遍 apply:第二遍零 catalog 写入", "expect": "guarded widening 模式,幂等可证(前后 catalog 计数一致)" },
    { "output": "值域 = 28 值且含 TopicOpened", "expect": "与 Rust EventType::ALL 逐字一致(顺序无所谓,集合必须相同)" },
    { "output": "memory 四类型 + AcceptanceCancelled 均在", "expect": "全量清单写法没有 revert 任何既有值(019 起草时踩过并修掉了这个坑)" },
    { "output": "apply-db.sh 注册就位", "expect": "POST-SEED 理由注释说明为何在此组" },
    { "output": "负例印象:越界事件类型仍被拒", "expect": "CHECK 仍只放行 28 值之内的类型" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-onyx-001)

```
dispatch_id=disp-019-onyx
branch=packet/onyx/topic-opened-019
base=track/infra-foundation
how_to_verify=scratch 两遍 apply 第二遍零 catalog 写入;event_type 值域 28 值含 TopicOpened,与 Rust EventType::ALL 逐字一致,memory 四类型未被 revert;apply-db.sh 注册就位
flux_verdict_ref=ev-019-topic-opened-flux-verdict
acceptance_plan=<§2 JSON 逐字>
```

## 4. 备注

live apply 019 是 Aegis 的动作(不在本卡范围);本卡验的是「文件落地 + 值域正确 + 幂等」。
Onyx 的 MCP 进程若仍是旧镜像(无 acceptance_plan 入参),按派单指示上报并等重启,
卡就绪后是一个 staged 调用。
