# 补开验收卡 —— 三个单的活干完了、也验完了,但没有卡,张先生点不到

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-seats-open-acceptance-cards-v1 |
| target_seat | seat-nimbus-001, seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 问题

三个派单都 `done`、都合并进 `track/infra-foundation`、Flux 也独立验过了,
**但 `acceptances` 表里三单都是 (无)**:

| 派单 | 席位 | 内容 | 独立验过? |
|---|---|---|---|
| `disp-VWK5z1k7Jcu8-iQFrGVrb` | nimbus | tauri bundle 判据 + favicon 404 | ✅ Flux PASS |
| `disp-NMcTvureVxwPh_3bFAGxx` | nimbus | `~/networkorchestrator` 假路径 | ✅ Flux PASS |
| `disp-i_ZLKc4E2S0Bm78K-Zmdy` | flux | 独立验上述两个 | 本身是验单,已发 verdict |

**结果是张先生点不到任何卡。** 活干完了、验绿了,卡在他那一关没人建。

## 为什么没卡:是 Aegis 的派单没写清

`request_acceptance` 的门槛(实测读 `dispatch.rs`):

- 这单必须是**调用席位自己拥有的已完成单**(否则 `NotCompleted`)
- 必须带 `base`(合并目标分支,命名过且是本地分支)
- 必须带 `flux_verdict_ref`,而且它必须是一条 `ReviewVerdictIssued`、
  `payload->>'verdict' = 'pass'`、**且是关于这一单的**
  (否则 `UnknownFluxVerdict` / `VerdictNotGreen` / `VerdictNotAboutThisDispatch`)

**Aegis 派那两个单时,任务包里没写"交付后请自己开验收卡,以及要带什么"。**
所以两个席位都停在 `done` 就停了。这是派单的疏漏,不是席位偷懒。

## 要做的

### nimbus(两个单)

对自己的两个已完成单调 `request_acceptance`:

- `disp-VWK5z1k7Jcu8-iQFrGVrb`
- `disp-NMcTvureVxwPh_3bFAGxx`

- `base` 用 `track/infra-foundation`
- `flux_verdict_ref` 用 **`ev-hh9xak8wlN5fJst6107cy`** ——
  那是 Flux 为 `disp-i_ZLKc4E2S0Bm78K-Zmdy` 预声明并发出的 verdict 事件 id,
  他的 payload 里点明了这两个单(已实测确认存在且 verdict=pass)

如果工具说 verdict 不是关于这两个单的(`VerdictNotAboutThisDispatch`),
**不要改 ref 去凑**,把工具返回的原文报回来,Aegis 另想办法。

### flux(自己的单)

`disp-i_ZLKc4E2S0Bm78K-Zmdy` 是验单,它的 verdict 就是 `ev-hh9xak8wlN5fJst6107cy`。
如果你认为验单不需要验收卡(它不产代码),**明说理由**,Aegis 接受"不需要"这个结论,
但不要静默跳过。

## 判据

跑完给 Aegis 看:

```
select dispatch_id, status from acceptances where dispatch_id in
  ('disp-VWK5z1k7Jcu8-iQFrGVrb','disp-NMcTvureVxwPh_3bFAGxx');
```

应该看到两行 `pending`。

## 明确不做

- 不许为了让卡开出来而改 `flux_verdict_ref` 或改判据
- 不许让 Aegis 代开(Aegis 不拥有这两个单,`NotCompleted` 会拦,而且那是替别人交付)
- 不许静默跳过 —— 不开口就回复"不开,因为 X"
