# 9 件已验绿的活,开卡记录(`disp-AB5Zrd6fiUmWXxvPkvcA0`)

派单:`AEGIS-2026-10-27-open-missing-acceptance-cards-v1`。9 件 done 且已有其他席位 pass verdict
的派单,张先生点不到卡。

**先核账本,不信转述,再开卡。**

---

## 一、verdict 逐条核过

派单给了 9 个 `dispatch_id -> flux_verdict_ref`。我逐条查了 `canonical_events`
(`event_types=['ReviewVerdictIssued']` + `ref_type='dispatch'` + `ref_id=<dispatch_id>`),
确认四样:事件存在、类型是 `ReviewVerdictIssued`、`payload.verdict` 是 `pass`、且
`payload.dispatch_id` 点名了这个 dispatch。**9 个全部通过,与派单转述一致。**

| dispatch | verdict ref | actor | verdict |
|---|---|---|---|
| `disp-x7QjIhw2ZsvokB_yidBj1` | `ev-N8vSfa-3zKNWukPsfzcrZ` | seat-flux-001 | pass |
| `disp-OwOSLvL4EbqoB6YhlBQHi` | `ev-nbpmWap9wlZue7V_A70D1` | seat-flux-001 | pass |
| `disp-H66BOKrYda4xltEgIwvuG` | `ev-3qVHyZW597RSpMiK2zxby` | seat-flux-001 | pass |
| `disp-wa7F61uczSw8ebZzeXwml` | `ev-WirH7Qh1eNDP7dZSV5uVs` | seat-flux-001 | pass |
| `disp-gvVtOC1Z_uvVz8mq-lc45` | `ev-sMrl5_wXRSDsj_V4qe3gs` | seat-flux-001 | pass |
| `disp-zsKFSIHHgv5v7wLG-94UR` | `ev-1mD-lbquk7qa_0venCWXO` | seat-flux-001 | pass |
| `disp-J9ExyK_2-3ajGuXjBvlKj` | `ev-Q1LjVPXRhHH4_xTQEjlim` | seat-flux-001 | pass |
| `disp-yhtJXKP7_krz7NBwz6v12` | `ev-SZNBEZHSGuj1U2fV7TzNd` | seat-flux-001 | pass |
| `disp-Ksm1oC3fHNRnH2zumBN_p` | `ev-Oc4hMs_bBXlnb52Pr1JJK` | seat-flux-001 | pass |

---

## 二、开卡前又撞上同一条:分支必须包含 base

`git merge-base --is-ancestor track/infra-foundation HEAD` 返回"不是祖先",所以先合并。

**这次我照 AGENTS.md 的规则核了 interval**:track 只领先 1 个提交、3 个文件,全是 `docs/`,
不碰 `ui/src` 与 `src-tauri`。按 AGENTS.md"任何 `ui/src`/`src-tauri` 变更才需要合并"的判断,
本该不用合。**工具照样拒。**

所以实际规则比 AGENTS.md 写的严,这条我上次已经记过,这次再次印证:

> **开卡前必须让 base 成为分支的祖先,与 interval 内容无关。**
> `request_acceptance` 只认 `merge-base --is-ancestor`,不认"这次改动没碰代码"。

合并后复跑门禁:`pnpm test` 337 passed / 29 files、`tsc` 干净、
`eslint` 143 problems(133 errors + 10 warnings)与 baseline 一致。

---

## 三、9 张卡全部开成(`status=pending`)

一个 dispatch 一张卡,没有合并。每张都写了 `acceptance_plan`(scenario / steps / expected /
preconditions 四段齐全)和一句话的 `how_to_verify`。

| dispatch | acceptance id | verdict ref |
|---|---|---|
| `disp-x7QjIhw2ZsvokB_yidBj1` | `acc-disp-x7QjIhw2ZsvokB_yidBj1` | `ev-N8vSfa-3zKNWukPsfzcrZ` |
| `disp-OwOSLvL4EbqoB6YhlBQHi` | `acc-disp-OwOSLvL4EbqoB6YhlBQHi` | `ev-nbpmWap9wlZue7V_A70D1` |
| `disp-H66BOKrYda4xltEgIwvuG` | `acc-disp-H66BOKrYda4xltEgIwvuG` | `ev-3qVHyZW597RSpMiK2zxby` |
| `disp-wa7F61uczSw8ebZzeXwml` | `acc-disp-wa7F61uczSw8ebZzeXwml` | `ev-WirH7Qh1eNDP7dZSV5uVs` |
| `disp-gvVtOC1Z_uvVz8mq-lc45` | `acc-disp-gvVtOC1Z_uvVz8mq-lc45` | `ev-sMrl5_wXRSDsj_V4qe3gs` |
| `disp-zsKFSIHHgv5v7wLG-94UR` | `acc-disp-zsKFSIHHgv5v7wLG-94UR` | `ev-1mD-lbquk7qa_0venCWXO` |
| `disp-J9ExyK_2-3ajGuXjBvlKj` | `acc-disp-J9ExyK_2-3ajGuXjBvlKj` | `ev-Q1LjVPXRhHH4_xTQEjlim` |
| `disp-yhtJXKP7_krz7NBwz6v12` | `acc-disp-yhtJXKP7_krz7NBwz6v12` | `ev-SZNBEZHSGuj1U2fV7TzNd` |
| `disp-Ksm1oC3fHNRnH2zumBN_p` | `acc-disp-Ksm1oC3fHNRnH2zumBN_p` | `ev-Oc4hMs_bBXlnb52Pr1JJK` |

落库证据:9 条 `AcceptanceRequested` 事件,`refs` 各含 `acceptance` subject +
`dispatch` context + `seat` actor,`payload.acceptance_plan.plan_version` 均为 1。

---

## 四、判据对得上

派单判据是"开完卡后 `select dispatch_id,status from acceptances` 应看到对应行 `status=pending`"。

工具面没有直接的 SQL 查询入口,我用 `query_ledger` 查每个 dispatch 名下的
`AcceptanceRequested`,9 条全部在,且工具返回 `pending operator decision`。
**9/9 有卡。**

---

## 五、写 plan 时的两条自我约束

1. **不把做不到的验点写成可验的。** 有两件的真机验证我做不了
   (`disp-OwOSLvL4EbqoB6YhlBQHi` 的真机滚动、`disp-wa7F61uczSw8ebZzeXwml` 的滚轮),
   这两张卡的 `preconditions` 写明"装好的 SeatLoom 应用",`how_to_verify` 写成"打开应用滚一个
   历史长的 seat"—— 留给能真机操作的人,没有假装我已经验过。
2. **纯注释/措辞类的两件,判据写成可复现的实验而不是"看起来对"。**
   `disp-x7QjIhw2ZsvokB_yidBj1` 的 plan 写的是"跑测试得 21 passed,再把 kind 那一行改成只读
   `failure.kind`,同一条测试应当变红"—— 一个有明确红绿判据的检查,不是一句主观评价。

---

## 六、没有静默跳过任何一件

9 件全部开卡,0 拒,0 漏。
