# D2 验收包(acceptance packet)— 交回 Aegis,呈张先生价值验收

**Owner:** Lyra (PO) · **Date:** 2026-10-10 · **Dispatch:** `disp-Xs30DFtXqqeUQFL62He85`(收尾编排)
**收件人:** Aegis(seat-aegis-001)——第 5 步"呈卡给张先生"是你的车道,本包是那次呈卡的全部输入。

---

## 1. 可验收卡(已存在于 live,待验收区可见)

| 字段 | 值 |
|---|---|
| acceptance_id | `acc-disp-me6Nm8e9D6ZRe4fKyVokY` |
| dispatch | `disp-me6Nm8e9D6ZRe4fKyVokY`(Mira 的 D2 UI 交付,已 done) |
| branch | `packet/mira/dispatch-ui` @ `bbfdfee` |
| base | track |
| status | **pending**( AcceptanceRequested 事件已入账本,actor=seat-mira-001) |
| how_to_verify | **在 SeatLoom UI 待验收区点接受 → 看到合并 sha;点打回 → 看到打回徽标+反馈+跳转返工单** |
| flux_verdict_ref | `ev-UDrRZZqr8LUwfcgEeFdN5`(Flux 复验 PASS,HOLD 已清) |

这就是用户可见的「待验收卡」。**第一个走新模型验收的交付就是验收能力本身**——卡能在
待验收区出现,即证明接线成功;按 how_to_verify 点一遍,即证明闭环成立。

## 2. 这个卡验收的是什么(D2 交付全景)

D2 = 在界面里验收一个里程碑并闭环。完整交付横跨**三个分支**(各自 Flux 验绿):

| 分支 | 内容 | Flux 判定 |
|---|---|---|
| `packet/onyx/dispatch-acceptance-gate` | `013_dispatch_states.sql` + `014_acceptance_gate.sql` + apply-db.sh 注册 | `ev-nalA9gijQuoSci8AeUazJ` PASS |
| `packet/nimbus/waker-hardening` | Rust 后端:request_acceptance/cancel_dispatch MCP + AcceptanceResult(status+reworkDispatchId)+ decide_acceptance 回读 + Tauri 三命令 + 合并机制 | `ev-OrgcgAsHA0XkpFcpsppQO` PASS + `ev-UDrRZZqr8LUwfcgEeFdN5` PASS(复验) |
| `packet/mira/dispatch-ui` | 派单页「待验收」区 UI(接受即合/打回即重派) | 同上(复验覆盖其消费的线形) |

**注意:013/014 的 schema 文件目前只在 Onyx 分支上,不在 track**——live 库的 013/014 是你
此前 out-of-band 应用的;文件上 track 后,scratch 构建才能复现本能力。

## 3. 合并安全性证据(全部只读实测,2026-10-10)

```
git merge-tree --write-tree track/infra-foundation <branch>   # 三方合并模拟,非两点 diff
  packet/onyx/dispatch-acceptance-gate  → exit 0, 零冲突 (tree 660e6f9)
  packet/nimbus/waker-hardening         → exit 0, 零冲突 (tree e12a5ae)
  packet/mira/dispatch-ui               → exit 0, 零冲突 (tree 4840538)
```

**对"stale 分支整支合并会回退 track"警告的更正**(我派单里转述了该警告,现予修正):
该警告的证据是两点 diff(`git diff track..branch`),它把 track 新增文件显示为分支缺失,
**不预测合并行为**。Flux 在 scratch clone 实测 + 我上面的 merge-tree 一致证明:三方
`--no-ff` 合并干净、不删任何文件、合并树门禁全绿(fmt/clippy -D warnings/test --workspace
104+40+4+3+3)。**整支 `--no-ff` 合并是安全的**,无需(也不建议)路径限定手术——手工
挑文件不可审计、易做半截。结论以三方合并模拟为准。

## 4. 给 Aegis 的执行建议(第 5 步怎么跑)

1. **验证用构建**:主 checkout 当前在 `track/infra-foundation @ b09ec6e`(纯 track 构建,
   没有待验收 UI 和后端命令)。张先生点卡之前,app 必须包含 UI+后端——由你决定构建方式
   (临时 integration 构建/或先集成后重建),**但不因此在验收前写 track**。
2. **呈卡**:张先生按 how_to_verify 一步验证。
3. **点接受** → `cmd_accept_delivery` 自动 `--no-ff` 合并卡片分支(mira),sha 落
   acceptance 行,卡面显示"已合入"。**同一窗口内**你完成另外两个分支的集成(顺序:
   onyx schema → nimbus 后端,依赖序),闭合"UI 有了、后端也有了"的缺口。此后 track 上的
   待验收能力完整、scratch 可复现。
4. **点打回**(若发生)→ 全自动:`cmd_reject_delivery` 落 AcceptanceRejected + 自动生成重做
   派单回 Mira(priority+10,waker 唤醒),无需你介入。
5. 稳态:此后单个自足交付走"卡→接受→自动合并"原路径;跨多分支的交付参照本包第 4.3 条。

## 5. 边界确认

席位间协调/任务打包/验收标准 = Lyra(本包即终点);基础设施/live 运维/服务重启/跨席位裁决/
张先生接口 = Aegis。我未合并任何分支、未写 live、未重启任何服务。

## 6. 证据索引

- Flux 复验交付:`FLUX-2026-10-10-nimbus-acceptanceresult-reverify-delivery-v1.md`(§4 合并
  模拟 + §5 PASS 判定)
- Nimbus 修复交付:`disp-58ZddqlToyI5dgkC1jiIP-delivery.md`(范围核验:零 ui/** 改动 ✅)
- Mira 此前的三阻塞交付:`disp-7Odk3ndocUENElEQYfQO6-delivery.md`(阻塞 A/B 已被你解决,
  C 为误报——分支本地可见,merge-tree 干净)
- 设计契约:`LYRA-2026-10-09-d2-acceptance-loop-design-v1.md` + VALUE-DOMAIN ADDENDUM
