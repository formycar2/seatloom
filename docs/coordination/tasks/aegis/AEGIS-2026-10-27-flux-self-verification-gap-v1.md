# Flux 的单怎么开验收卡 —— 需要 Lyra 定规则

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-10-27-flux-self-verification-gap-v1 |
| status | issued |
| author | aegis |
| date | 2026-10-27 |
| version | v1 |
| to | lyra |
| tags | acceptance,verdict,flux,rule-gap,escalation |

> 日期:2026-10-27 18:55。这是 Aegis 判不了的跨席位规则问题,按升级流程派给 Lyra(PO)。
> Aegis 不代决。

## 1. 问题

`request_acceptance` 的守门条件之一是
`flux_verdict_ref` 必须指向一个 `ReviewVerdictIssued` 事件,且该事件的
`payload.dispatch_id` **点名本 dispatch**(`VerdictNotAboutThisDispatch`)。

这条规则对"实现类"派单是对的:Mira 写 UI,Nimbus 写 Rust,Flux 独立验,张先生看卡拍板。

但 **Flux 自己的派单**没有 verdict。`canonical_events` 里 30 条最近的
`ReviewVerdictIssued`,`dispatch_id` 从来没有一次指向一个 Flux 的单。

结果是:Flux 的 8 件已完成派单(`disp-viPlKa_e9NoeRHeGe_LCC`、
`disp-yqYB1iZlPYe7HceKux71D`、`disp-xyvPfhtGxQo5x_2u4VGPU`、
`disp-IgwyJqKM_ef9T3j63sD-i`、`disp-i_ZLKc4E2S0Bm78K-Zmdy`、
`disp-VH6l1HixlUVYuy4m0NEp-`、`disp-4zePyIT3GboLBELuobHAS`、
`disp-E57bNvMK5uTMgrH7tg2cK`)**永远开不出验收卡**。

这不是 Flux 偷懒,是规则里有一个没有定义的格子。

## 2. 为什么不能让 Flux 自己发 verdict

如果允许 `seat-flux-001` 给自己的 dispatch 发一个 pass,那么:

- "独立验证"退化成"自己说自己过了";
- 张先生点下的 Accept 不再有第三方证据支撑;
- 验收卡和"没有任何检查"之间只剩一个工具调用。

**所以这一条我不建议放开。** 但需要 Lyra 给出正面规则,而不是留一个洞。

## 3. 可选的三种规则(请 Lyra 选一种,或给出第四种)

| 方案 | 做法 | 代价 |
|---|---|---|
| A. 交叉验证 | Flux 的验单由另一个席位(Nimbus 或 Mira)验 | 需要两个席位都懂验证方法;验证者自己也要被验,递归 |
| B. 复核链 | Flux 的 verdict 由 Aegis 抽检复核后另发一条 verdict | Aegis 变成瓶颈,而且 Aegis 也在被验的名单里 |
| C. 免验类 | 明确列出"哪些种类的派单不需要 verdict"(例如:纯调查/只读报告、纯文档归类) | 需要一份能长期维护的白名单;边界会被推 |

**我的倾向**:C 为主 + A 兜底 —— 纯只读、零代码改动的派单(比如
`disp-2o_BDY1tb3b92ZQwaYXEC` 那种"查清楚 89 个 NULL 是怎么回事"的调查报告)
免 verdict;只要动了代码或改了配置,就必须有第三方 verdict。

但这是我的判断,不是规则。**请 Lyra 写成正式规则落进 `ENGINEERING_STANDARDS.md`**,
然后 Aegis 照它去放这 8 件。

## 4. 在规则定下来之前

Aegis **不会**:

- 让 Flux 自己给自己发 verdict;
- 用别的单的 verdict 凑 ref;
- 改账本行。

这 8 件会继续停着。这是已知的、说清楚了的堵点,不是一个安静的洞。
