| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-onyx-016-plan-carrier-acceptance-plan-v1 |
| title | Acceptance plan — migration 016 验收方案结构化承载(acceptance_plan JSONB + 非空 CHECK) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1 |
| tags | acceptance-plan, onyx, migration-016, acceptance-plan-carrier |

# Acceptance plan — migration 016 验收方案承载

**对象**:dispatch `disp-accplan-lane-c-onyx`(done)· 分支 `packet/onyx/acceptance-plan-schema`
@ `a6f2ac8`(含 track b6751a1)· Flux PASS(commit-pinned a6f2ac8)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

张先生阻塞反馈的「真数据缺口」半边:acceptances 增加 `acceptance_plan` JSONB 列
(shape v1:plan_version/scenario/steps(action+input)/expected(output+expect)/preconditions)
+ 非空 CHECK(NOT VALID,在候卡由 Aegis 回填 Lyra 供稿的 plan 后 VALIDATE)。
tool 层强制(request_acceptance 缺方案不许建卡)是 Nimbus 车道,本单是 DB 层承载与兜底。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认验收方案有了结构化载体且「缺方案不许建卡」有 DB 层兜底:此后每张待验收卡都必带「照着做就能验」的产品操作脚本,一句话验法不再能建卡。",
  "preconditions": [
    "scratch 库可用(DATABASE_URL 指向 scratch,绝不是 live seatloom)",
    "psql 在 PATH;checkout 在含 016 的分支或已合入 016 的 track",
    "已知在候卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 的 plan 回填 + VALIDATE 是 Aegis 的 live 动作,不在本单"
  ],
  "steps": [
    { "action": "scratch 库跑两遍 apply-db.sh(含 016)", "input": "DATABASE_URL=postgresql://…/seatloom_scratch scripts/apply-db.sh(连跑两次)" },
    { "action": "查新列形态", "input": "psql -c \"SELECT column_name,data_type,is_nullable,column_default FROM information_schema.columns WHERE table_name='acceptances' AND column_name='acceptance_plan';\"" },
    { "action": "查 CHECK 形态", "input": "psql -c \"SELECT conname,convalidated,pg_get_constraintdef(oid) FROM pg_constraint WHERE conname='acceptances_plan_required';\"" },
    { "action": "负例:尝试建一张 acceptance_plan 为 '{}' 的 acceptance 行", "input": "psql -c \"INSERT INTO acceptances (id,dispatch_id,project_id,branch,base,how_to_verify,flux_verdict_ref,status) VALUES ('acc-neg-001','<某个 done dispatch id>','seatloom','packet/x/y','track/infra-foundation','do A see B','<存在的事件 id>','pending');\"" },
    { "action": "正例:同一 INSERT 但带合法 plan", "input": "acceptance_plan 列给 '{\"plan_version\":1,\"scenario\":\"s\",\"steps\":[{\"action\":\"a\"}],\"expected\":[{\"output\":\"o\",\"expect\":\"e\"}]}'" }
  ],
  "expected": [
    { "output": "两遍 apply 后:第二遍零 catalog 写入", "expect": "前后 catalog 计数一致;016 头部度量方式" },
    { "output": "列:acceptance_plan / jsonb / NOT NULL / DEFAULT '{}'", "expect": "注释含 shape v1 与「'{}' 不是 plan」" },
    { "output": "CHECK acceptances_plan_required 存在且 convalidated = f", "expect": "NOT VALID 是故意的:唯一 legacy pending 行豁免,待 Aegis 回填后 VALIDATE;定义含 <> '{}'" },
    { "output": "负例 INSERT 被 DB 拒绝", "expect": "新行强制生效:缺方案建不出卡(错误信息指向 acceptances_plan_required)" },
    { "output": "正例 INSERT 成功", "expect": "随后 DELETE 该测试行,scratch 复原" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-onyx-001)

```
dispatch_id=disp-accplan-lane-c-onyx
branch=packet/onyx/acceptance-plan-schema
base=track/infra-foundation
how_to_verify=scratch 两遍 apply 第二遍零 catalog 写入;acceptance_plan JSONB 列 + 非空 CHECK(NOT VALID,convalidated=f)就位;'{}' 建卡被 DB 拒
flux_verdict_ref=<Flux 对 a6f2ac8 的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```
