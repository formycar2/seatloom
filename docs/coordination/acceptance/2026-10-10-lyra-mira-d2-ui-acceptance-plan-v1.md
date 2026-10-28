| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1 |
| title | Acceptance plan — D2/Mira 派单页「待验收」区 UI(Lane D 首个结构化样例) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1 |
| tags | acceptance-plan, d2, mira, worked-example |

# Acceptance plan — D2/Mira 派单页「待验收」区 UI

**这是 Lane D 的首个结构化验收方案样例**(张先生阻塞反馈的产物:验收方案的作者是 PO)。
对象:在候卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY`。shape 遵循
`VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` §1(v1),**本文件 §2 的 JSON 即回填内容**。

## 1. 卡片血缘速览(为什么这张卡值得验)

- **交付**:Mira 的 D2 UI lane —— 派单页「待验收」区:待验收卡(分支/验绿徽标/一步验法)、
  接受即 `--no-ff` 合并并显示 sha、打回即自动生成 priority+10 返工单、冲突原文展示不 force。
- **上游派单**:`disp-me6Nm8e9D6ZRe4fKyVokY`(issuer seat-lyra-001,2026-10-09 12:50,
  payload 1253 字符原始任务说明)。
- **事件链**:SeatDispatchIssued → SeatDispatchDelivered → ArtifactCreated →
  SeatDispatchCompleted(Mira)→ AcceptanceRequested(Mira)→ ReviewVerdictIssued(Flux)。
- **Flux 判定**:`ev-UDrRZZqr8LUwfcgEeFdN5`(复验 PASS,HOLD 已清)。
- **分支**:`packet/mira/dispatch-ui` @ `bbfdfee`;base 行记录为 `track`。
- **已知异常(不瞒报)**:该 base 记录与真实分支 `track/infra-foundation` 不一致 ——
  Nimbus 缺陷单 `disp-E8BP2vKlOwMHLomm0eX8J` 已派(修 base 记录 + 作废卡片路径)。
  且 D2 内容已由 Aegis 自举合入 track(6aa705a,路径限定)。本方案的预期结果按此现实写。

## 2. acceptance_plan(回填 JSON,逐字)

```json
{
  "plan_version": 1,
  "scenario": "张先生在 SeatLoom 桌面应用里对 Mira 的 D2 UI 交付做价值验收:确认待验收卡片信息足以判断价值,并实跑一次「接受即合并」与「打回即返工」的闭环,确认「在界面里验收一个里程碑」成为系统能力。",
  "preconditions": [
    "主 checkout 运行含待验收区与后端命令的构建(track/infra-foundation @ 6aa705a 或其后的集成构建)",
    "live 库存在 1 张 pending 卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY",
    "SEATLOOM_OPERATOR_REF 已配置(操作者身份显式配置,缺省命令拒绝执行)",
    "已知 base 记录为 'track'(非真实分支 track/infra-foundation):接受路径的预期分两种,见 expected 第 2 条"
  ],
  "steps": [
    { "action": "打开 SeatLoom 应用,进入派单页(收件箱同页),找到「待验收」区", "input": "项目 seatloom(默认)" },
    { "action": "读卡片:标题「D2/Mira: 派单页「待验收」区 UI(接受即合/打回即重派)」、分支 packet/mira/dispatch-ui @ bbfdfee、Flux 验绿徽章、how_to_verify 一行判据", "input": "无" },
    { "action": "点「接受」", "input": "点击「接受」按钮" },
    { "action": "(闭环另一半)对同一张卡或按缺陷单修复后新建的卡点「打回」", "input": "必填反馈一句,如「卡片信息不足,按血缘+结构化方案改版」" }
  ],
  "expected": [
    { "output": "待验收区可见 1 张 pending 卡,卡面含分支、验绿徽章、一步验法", "expect": "不翻代码即可知道这是什么交付、谁验过、按什么判据验" },
    { "output": "点接受后:卡面显示「已合入」+ 合并 commit sha", "expect": "若缺陷单未落地:app 明确拒绝并指出 base 'track' 无法 checkout(拒绝即正确,不静默不 force);缺陷单落地后:--no-ff 合并进 track/infra-foundation,sha 落卡面" },
    { "output": "点打回后:卡面显示打回徽标 + 反馈 + 返工派单链接", "expect": "收件箱出现 priority+10 的新派单回到 Mira,waker 唤醒该席位" },
    { "output": "若发生合并冲突", "expect": "展示 git 冲突原文,合并 abort,卡保持 pending,绝不 force" }
  ]
}
```

## 3. 落库路由(Aegis 车道,非本单车)

1. Onyx 016 落地后:Aegis 将 §2 的 JSON 回填进行内卡
   (`UPDATE acceptances SET acceptance_plan = '<§2 JSON>' WHERE id = 'acc-disp-me6Nm8e9D6ZRe4fKyVokY'`),
   然后 `VALIDATE CONSTRAINT acceptances_plan_required`(live-DB 动作)。
2. Nimbus 缺陷单落地后:卡的 base 记录纠正为 `track/infra-foundation`,接受路径按 §2
   expected 第 2 条的后半执行。
3. Mira Lane A 落地后:张先生在 UI 上看到的就是本方案的渲染形态(场景/步骤/入参/预期)。

## 4. 此后(纪律)

每个交付的验收包 = how_to_verify(一行判据)+ 本形状的结构化验收方案(PO 供稿)。
只有一句话的验收包不再可接受。
