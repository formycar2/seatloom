| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-onyx-017-cancel-widening-acceptance-plan-v1 |
| title | Acceptance plan — migration 017 作废路径加宽(acceptances status +cancelled / event_type +AcceptanceCancelled) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-08-governed-dispatch-v1 (D2 addendum), NIMBUS-2026-10-10-d2-acceptance-base-and-cancel-v1 |
| tags | acceptance-plan, onyx, migration-017, cancel-path, surfaced-gap |

# Acceptance plan — migration 017 作废路径加宽

**对象**:新 lane(dispatch `disp-acclose-onyx` 内嵌,由 Onyx 交付)· 分支
`packet/onyx/acceptance-cancel-schema`(基于 track/infra-foundation)· Flux 判定以落地后
自请验绿为准。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(为什么单独立卡)

Nimbus 已交付 cancel_acceptance 正规路径(pending → cancelled 终态 + AcceptanceCancelled,
同一事务,理由必填),二进制里已有该工具。但**支撑它的 migration 从未落地**:任何分支/
迁移里都没有 `acceptances_status_vocab` 的 'cancelled' 与 `canonical_events_event_type_vocab`
的 'AcceptanceCancelled'(只在 Nimbus 的 .scratch 里起草过)。Aegis 结清旧空卡
`acc-disp-me6Nm8e9D6ZRe4fKyVokY` 的动作会被 DB CHECK 拒。017 补上这个 schema 前置:
additive + 幂等,全量清单(Nimbus Rust 钉 26 = 22 + 4 memory),从 track 的 21 值态或 live 的 25 值态任一状态干净应用(guarded widening 写全量,013/015 同模式)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认作废路径的 schema 前置就位:acceptances 可有 'cancelled' 终态、账本可记 AcceptanceCancelled —— Aegis 结清旧空卡、以及此后任何席位撤回自己的待验收请求,都不再被数据库值域卡住。",
  "preconditions": [
    "scratch 库可用(DATABASE_URL 指向 scratch,绝不是 live seatloom)",
    "psql 在 PATH;checkout 在含 017 的分支",
    "Nimbus 的 cancel_acceptance 已在二进制(既有事实);本单只补 schema,不动 Rust"
  ],
  "steps": [
    { "action": "scratch 库跑两遍 apply-db.sh(含 017)", "input": "DATABASE_URL=postgresql://…/seatloom_scratch scripts/apply-db.sh(连跑两次)" },
    { "action": "查 acceptances 状态值域", "input": "psql -c \"SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='acceptances_status_vocab';\"" },
    { "action": "查事件类型值域", "input": "psql -c \"SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='canonical_events_event_type_vocab';\"" },
    { "action": "正例:插入一行 status='cancelled' 的 acceptance(或对既有 pending 行走 cancel_acceptance,若二进制已就位)", "input": "psql -c \"INSERT INTO acceptances (id,dispatch_id,project_id,branch,base,how_to_verify,flux_verdict_ref,status,decided_by) VALUES ('acc-cancel-demo','<某个 done dispatch id>','seatloom','packet/x/y','track/infra-foundation','demo','<存在的事件 id>','cancelled','seat:seat-lyra-001');\"" },
    { "action": "正例:追加一条 AcceptanceCancelled 事件", "input": "经 emit_event(type='AcceptanceCancelled', refs=[subject=acceptance acc-cancel-demo, actor=seat:seat-lyra-001])" },
    { "action": "清理演示行", "input": "psql -c \"DELETE FROM acceptances WHERE id='acc-cancel-demo';\"" }
  ],
  "expected": [
    { "output": "两遍 apply:第二遍零 catalog 写入", "expect": "guarded widening 模式,幂等可证" },
    { "output": "acceptances_status_vocab = pending|accepted|rejected|cancelled", "expect": "cancelled 终态不可变(decide_acceptance 侧既有终态纪律覆盖)" },
    { "output": "canonical_events_event_type_vocab = 26 值且含 AcceptanceCancelled", "expect": "与 Nimbus Rust EventType::ALL 钉的 26 逐字一致(22 + 4 memory);apply-db.sh POST_SEED 注册行就位" },
    { "output": "两个正例均成功(此前会被 CHECK 拒)", "expect": "cancel_acceptance 对旧空卡的结清路径打通;Aegis 的 live 动作不再被 schema 阻塞" },
    { "output": "清理后 scratch 复原", "expect": "演示行与事件留在 scratch 账本无害(scratch 非 live)" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-onyx-001)

```
dispatch_id=<017 落地后 Onyx 为自己本轮 closeout 完成的 dispatch id(disp-acclose-onyx 或 Onyx 另立的 017 单>
branch=packet/onyx/acceptance-cancel-schema
base=track/infra-foundation
how_to_verify=scratch 两遍 apply 第二遍零 catalog 写入;status 值域含 cancelled、event_type 26 值含 AcceptanceCancelled;cancel 正例可落库
flux_verdict_ref=<017 落地后 Flux 验绿的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```

## 4. 路由备注(Aegis)

017 在 live apply 后,旧空卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 的 cancel_acceptance 结清即可
执行 —— 那是 Aegis 的动作(理由必填,本单不代做)。
