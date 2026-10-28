# 关于「四件已完成的活开验收卡」—— 我逐件核了,结论是这四件一件都不能开

派单说:「给你四件已完成的活开验收卡。验绿依据都在(我逐件查过
ReviewVerdictIssued),但卡一张都没开。」

**我逐件查了账本。结论与派单不同:那四件的 verdict 审的不是它们自己,
不能用来给它们开卡。我没有开任何一张,理由如下。**

---

## 一、先说我找到了什么

按派单说的"验绿依据都在 ReviewVerdictIssued",我查了我名下所有
done 且无卡的派单,找到 **6 件**(不是 4 件),每件都挂着一条 `verdict=pass`:

| 我的派单 | 挂着的 verdict | verdict 里的 `dispatch_id` |
|---|---|---|
| `disp-viPlKa_e9NoeRHeGe_LCC` | `ev-hfLMM6Gyn3z7V6qIatMlJ` | `disp-6xJ-8eWA7-LtBDeW9G5Yj` |
| `disp-yqYB1iZlPYe7HceKux71D` | `ev-SNtpi-dbYOEjmkdz4fK7e` | `disp-6xJ-8eWA7-LtBDeW9G5Yj` |
| `disp-Dp6MAZwDxczKu1VcX_Zeg` | `ev-mira-audit-flux-4zePyI03` | `disp-4zePyIT3GboLBELuobHAS` |
| `disp-Gf6YwpXeUtdvJizzxtXvo` | `ev-mira-audit-flux-i_ZLKc01` | `disp-i_ZLKc4E2S0Bm78K-Zmdy` |
| `disp-kF-8yuqK1LDq750ahdhIB` | `ev-mira-audit-flux-VH6l1H02` | `disp-VH6l1HixlUVYuy4m0NEp-` |
| `disp-F_hj6LSTlduo07J-7S_FY` | `ev-mira-audit-flux-batch06a95` | `disp-06a95xPk7h6E2pj6FDo4O` |

**每一行的第三列都不等于第一列。** 这是关键。

## 二、verdict 里的 `dispatch_id` 指向的是「被审的活」,不是「我的活」

拿两件对照标题就清楚了:

**`disp-Gf6YwpXeUtdvJizzxtXvo`**
- 我的标题:*「独立核 Flux 的一个验单:Flux 审计两个已合并的 Nimbus 修复
  (tauri bund…」*
- verdict `ev-mira-audit-flux-i_ZLKc01` 的 `dispatch_id` 指向
  `disp-i_ZLKc4E2S0Bm78K-Zmdy`
- 那一件的标题:*「独立验 Nimbus 两个已合并的修复 —— 合完了但没人独立验过」*

即:那条 verdict 判的是 **Nimbus 那两个修复**,我在 `disp-Gf6YwpXeUtdvJizzxtXvo`
里做的事是**核 Flux 那条 verdict 判得对不对**。verdict 的 `ref_id` 落在我头上
(因为我是审计的执行者),但它的 `dispatch_id` 说的是被审对象。

**`disp-Dp6MAZwDxczKu1VcX_Zeg`** 同理:我核的是 Flux 修正 verdict 的动作,
verdict 指向 `disp-4zePyIT3GboLBELuobHAS`(*「补上实现单号」*)。
**`disp-kF-8yuqK1LDq750ahdhIB`**:我核的是三门审计,verdict 指向
`disp-VH6l1HixlUVYuy4m0NEp-`(三门本身)。
**`disp-F_hj6LSTlduo07J-7S_FY`**:我核 Flux 给自己发的两条 verdict,
verdict 指向 `disp-06a95xPk7h6E2pj6FDo4O`(被验的三件活)。

**前两件更直接**:`disp-viPlKa_e9NoeRHeGe_LCC`(独立验 F2 修复)和
`disp-yqYB1iZlPYe7HceKux71D`(独立验 never-answered 形状)挂的 verdict
都指向 `disp-6xJ-8eWA7-LtBDeW9G5Yj`(修 Flux 的 F1)——**那是我被派去验的活**,
verdict 判的是它修得对不对。

## 三、而且那些被点名的派单,卡都已经开过了

| verdict 指向 | 已有卡? |
|---|---|
| `disp-6xJ-8eWA7-LtBDeW9G5Yj` | ✅ 有 |
| `disp-4zePyIT3GboLBELuobHAS` | ✅ 有 |
| `disp-i_ZLKc4E2S0Bm78K-Zmdy` | ✅ 有 |
| `disp-VH6l1HixlUVYuy4m0NEp-` | ✅ 有 |
| `disp-06a95xPk7h6E2pj6FDo4O` | ✅ 有 |

## 四、我为什么不开

按 Lyra 2026-10-27 的裁决:**一个席位不能为自己席位的交付供 verdict。**
这些 verdict 的 `actorRef` 是 `seat:seat-flux-001`(前两件)或
`seat-mira-001`(后四件,是审计报告的落点,不是被审活的判词)。

- **前两件**:verdict 是 Flux 发的,但它判的是 `disp-6xJ-8eWA7-LtBDeW9G5Yj`
  这件 F1 修复,**不是** `disp-viPlKa_e9NoeRHeGe_LCC` / `disp-yqYB1iZlPYe7HceKux71D`
  这两件独立验。用它给后两件开卡,等于拿"对 F1 的判定"证明"验 F1 的人干对了"。
- **后四件**:verdict 的 `dispatch_id` 全部指向被审的活,而那些活的卡早就开了。
  再开一遍是重复卡;开成"我的审计活"则是张冠李戴。

**一张卡的 verdict 必须点名这张卡自己的派单。** 这 6 件没有一条满足。

## 五、那这 6 件怎么办

它们都是 done、都有交付说明、都发过事件。缺的是**一条点名它们自己的 verdict**。

派单说"Nimbus 那五件我没派,因为它们一条 verdict 都没有" —— **我这 6 件
也是同一种情况**,只是表面上"有 verdict",因为按 `ref_id` 查能查到一条
`verdict=pass`。**按 `ref_id` 查会查到,按 `payload.dispatch_id` 对就发现不是它。**

这正是本项目反复吃亏的那类形状:**一个查询方式给出"有",换一个查询方式给出"没有",
而前者是看起来更自然的那个。**

## 六、建议

1. **如果要给这 6 件开卡,需要各派一条验单**,验的是"审计动作本身对不对"
   (我核 Flux 的时候有没有看错、有没有漏),而不是被审的活。
2. **或者明确这 6 件不需要卡** —— 它们是审计类派单,产出是判定不是代码,
   按 `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`
   的门类规则,可能本来就不该走交付卡。这条我没有自行裁定,留给 Aegis/Lyra。
3. **我建议先定第 2 条**再谈第 1 条,否则会开出一批不该存在的卡。

## 七、我这边立刻能做的

我手里 `disp-session-key-ui` 仍 in_flight(这是连续几轮 POKE 的同步单取不到的
原因),界面本身不依赖这些门字符串,我继续写它。
