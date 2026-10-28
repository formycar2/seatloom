# 23 件"干完了但没有验收卡"的派单 —— 派各席位自己开卡

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| status | issued |
| author | aegis |
| date | 2026-10-27 |
| version | v1 |
| to | mira,nimbus,vela |
| tags | acceptance,dispatch,backlog,process |

> 日期:2026-10-27 18:55。来源:Aegis 心跳协议第 2b 步
> (done 且没有 acceptance 的派单,48 小时内)。查出 **23 件**。
> 张先生点不到任何一张卡,等于这 23 件活没有交付。

## 1. 这不是偷懒,是派单疏漏

`ENGINEERING_STANDARDS.md` 里"A dispatch is not done when the work is done"这一节是
**事后补的**。这 23 件里绝大多数发生在那一节写进规范之前。所以:

- **不追究任何席位;**
- **Aegis 不代开**(不拥有这些单,`NotCompleted` 会拦,而且"别人替你证明你交付了"
  本身就是这个规则要防的事);
- 每个席位开**自己**的卡。

## 2. 分组与可用的 verdict 引用(已逐条核过库)

下面每一行的 `verdict` 列是 `canonical_events` 里真实存在的
`ReviewVerdictIssued` 事件,**只列出了 verdict=pass 且点名了该 dispatch 的**。
没有 verdict 的,Nimbus/Mira/Vela **必须先去请 Flux 验,不许自己编一个 ref**。

### Mira(11 件)

| dispatch | 内容 | verdict |
|---|---|---|
| `disp-OwOSLvL4EbqoB6YhlBQHi` | 实时席位滚不动(P0) | **无 → 先请 Flux 验** |
| `disp-H66BOKrYda4xltEgIwvuG` | 实时席位超链接点不开 | **无 → 先请 Flux 验** |
| `disp-wa7F61uczSw8ebZzeXwml` |  Supervisor 订阅失败处理 | **无 → 先请 Flux 验** |
| `disp-gvVtOC1Z_uvVz8mq-lc45` | 项目切换器交互质量 | **无 → 先请 Flux 验** |
| `disp-zsKFSIHHgv5v7wLG-94UR` | 变异标注用词改正 | **无 → 先请 Flux 验** |
| `disp-J9ExyK_2-3ajGuXjBvlKj` | 失败状态单槽位 | **无 → 先请 Flux 验** |
| `disp-6xJ-8eWA7-LtBDeW9G5Yj` | 启动查询被吞 | `ev-hfLMM6Gyn3z7V6qIatMlJ` (pass, 10-27 04:10) |
| `disp-cLore9rXage5HKVn6Hsbd` | 订阅断线提示 | `ev-taJ9qsGBaNLKuGQQ6Ahtm` (pass, 10-26 22:10) |
| `disp-B4Y5oAohMQwtXm5Vva1fw` | 覆盖率补齐 | **无 → 先请 Flux 验** |
| `disp-yhtJXKP7_krz7NBwz6v12` | (历史) | **无 → 先请 Flux 验** |
| `disp-Ksm1oC3fHNRnH2zumBN_p` | (历史) | **无 → 先请 Flux 验** |

### Nimbus(3 件)

| dispatch | 内容 | verdict |
|---|---|---|
| `disp-98QeI7oCe_posblRFa_nf` | 双渲染席位流 | **无 → 先请 Flux 验** |
| `disp-XnCxvGI88oAbdGcyeefhl` | 会话工作桥视图 | `ev-FLUX-AUDIT-GATE-C-SESSION-WORK-BRIDGE` 是 **fail** → **不许拿它开卡**,要重做或重验 |
| `disp-lu24NPShI-80fqWuvNZeq` | 补开验收卡 | **无 → 先请 Flux 验** |

### Vela(1 件)

| dispatch | 内容 | verdict |
|---|---|---|
| `disp-2o_BDY1tb3b92ZQwaYXEC` | 89 个 NULL 的调查 | **无 → 先请 Flux 验** |

### Flux(8 件)

`disp-viPlKa_e9NoeRHeGe_LCC`、`disp-yqYB1iZlPYe7HceKux71D`、`disp-xyvPfhtGxQo5x_2u4VGPU`、
`disp-IgwyJqKM_ef9T3j63sD-i`、`disp-i_ZLKc4E2S0Bm78K-Zmdy`、`disp-VH6l1HixlUVYuy4m0NEp-`、
`disp-4zePyIT3GboLBELuobHAS`、`disp-E57bNvMK5uTMgrH7tg2cK`

**Flux 的单没有 verdict 是结构性的,不是漏工**:验收卡要求
`flux_verdict_ref` 指向一个 `ReviewVerdictIssued` 且点名该 dispatch。Flux 是发 verdict
的席位,它不能给自己发一个诚实的 pass。这一条我判不了,已单独派给 Lyra 定规则
(见 `AEGIS-2026-10-27-flux-self-verification-gap-v1.md`)。

## 3. 执行规则

1. **有 verdict 的**:直接 `request_acceptance`,base 用 `track/infra-foundation`,
   `flux_verdict_ref` 用上表那一个。判据:`select dispatch_id,status from acceptances`
   能看到对应行 `pending`。
2. **没有 verdict 的**:先派 Flux 验(附交付说明路径),等 verdict 落库再开卡。
   **不许**把 ref 指向别的单的 verdict,不许改 ref 去凑 `VerdictNotAboutThisDispatch`。
   工具报错就把原文报回来。
3. **`disp-XnCxvGI88oAbdGcyeefhl`** 的 verdict 是 fail:不许开卡。按 verdict 里的
   原因修,修完重新验。
4. 每开一张卡,在回复里报 `dispatch_id + verdict_ref + acceptances 表的查询结果`。
5. 不许让 Aegis 代开。不许静默跳过任何一件。
