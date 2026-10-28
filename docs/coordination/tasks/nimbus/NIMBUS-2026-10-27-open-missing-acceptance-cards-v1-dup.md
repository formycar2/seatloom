# NIMBUS-2026-10-27-open-missing-acceptance-cards-v1 (dup) — 交付说明

派单 `disp-SNGDYQnlpe2SA4QHwaYqb`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-27-open-missing-acceptance-cards-v1.md`。

## 这一条是我刚上报的那条重复派单

同一个 packet、同一批三个 dispatch、同一段 summary,只有 id 和时间戳不同
(所以 payload 的 md5 不同,内容不是)。前一条是
`disp-Tg64EtzktcysA_ZAVl_tG`,交付说明见
`docs/coordination/tasks/nimbus/NIMBUS-2026-10-27-open-missing-acceptance-cards-v1.md`。

**我上报过这条重复**(`ev-tnKNlow-4CDScH2FOk7RN`),张先生还没回。

## 重新核了一遍现状:和前一条做完时完全一样,没有新东西

| 检查项 | 现在的值 |
|---|---|
| 我这三件的新 verdict | **无**(13:00 之后没有任何 `ReviewVerdictIssued`) |
| 我派给 Flux 的验证单 `disp-06a95xPk7h6E2pj6FDo4O` | 仍是 `ready` —— Flux 还没拉 |
| `acceptances` 里我这三件的行 | **零行**(仍不满足开卡条件) |

所以**答案和上一条一样:一张卡都不能开。**

- `disp-XnCxvGI88oAbdGcyeefhl` 的 verdict 仍是 fail,翻它只能由 Flux 重出;
- `disp-98QeI7oCe_posblRFa_nf` 和 `disp-lu24NPShI-80fqWuvNZeq` 仍没有 verdict。

## 没有重复派 Flux

这是这一条唯一需要判断的地方。

再发一次 `enqueue_dispatch` 给 Flux 是**最省事也最错**的做法:它会造出
第二条一模一样的验证单,Flux 会验两遍,出两条 verdict,而
`request_acceptance` 的守卫本来就会因为"该 dispatch 已有 pending acceptance"
拦下第二张卡 —— 于是那第二条 verdict 是纯垃圾,还占着一个
`verdict_event_id`。

原来的那条(`disp-06a95xPk7h6E2pj6FDo4O`)还在 `ready` 等着 Flux 拉,
它覆盖的正是这三件。重复发不会让它更快被处理。

## 等什么

等 Flux 拉走 `disp-06a95xPk7h6E2pj6FDo4O` 并落了 verdict。
**三条 pass verdict 各自落库之后**,按任务包第 3 节第 1 条,
用 `base=track/infra-foundation` 开三张卡,判据是
`select dispatch_id,status from acceptances` 能看到三行 `pending`。

在 verdict 落库之前,这一条重复派单再出现几次,答案都是同一个。

## 提交

仅本说明文件。无代码改动。
