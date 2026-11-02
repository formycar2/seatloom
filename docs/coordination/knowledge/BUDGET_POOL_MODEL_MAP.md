# 预算池 → 模型映射(2026-11-02 张先生口述 + 实测)

**这是计费事实,不是推测。** 席位选模型时按这张表走,不要猜。

> 🔴 **金额格式是 `已花费 / 总预算`,不是 `余额 / 预算`。**
> 本文档第一版把 a 当成了余额,于是得出"三个池已见底"的错误结论,全部反了。

> ♾️ **`coding-deepseek-v4.1f` 是 unlimited 池,不计费**(张先生 2026-11-02 明确)。
> 它上面的 ¥7145.49 只是累计显示计数器,**不是钱**,不会再增加。
> **真实计费只发生在下面六个池上。**

## 池的真实状态(2026-11-02 最新读数)

### 计费池

| 池 | 已花 | 预算 | **剩余** | 用掉 |
|---|---|---|---|---|
| 大模型系统coding-gpt-6.1-sol | 6.14 | 2000 | **1993.86** | 0.3% |
| stepcode-0903 | 0.00 | 1000 | **1000.00** | 0.0% |
| 大模型系统coding-opus-5.5 | 5.27 | 500 | **494.73** | 1.1% |
| 大模型系统coding-fable-5.1 | 13.17 | 100 | **86.83** | 13.2% |
| 大模型系统-ToB-Infra | 115.81 | 2500 | **2384.19** | 4.6% |
| 大模型系统coding-gpt-6-astra | 3.08 | 500 | **496.92** | 0.6% |
| **合计(计费)** | **143.47** | **6600** | **6456.53** | **2.2%** |

### 不计费池

| 池 | 显示计数 | 真实成本 |
|---|---|---|
| 大模型系统coding-deepseek-v4.1f | ¥7145.49 | **¥0(unlimited)** |

**所以整体实际支出是 ¥143.47,不是 ¥7288.96。** 差出来的 7145 全是免费额度。

### ⚠️ 两次读数之间新增 ¥50.04,其中 ¥49.93 是真金白银

| 池 | 增量 | 计费? |
|---|---|---|
| coding-deepseek-v4.1f | +0.11 | ♾️ 否(本文档作者的探针调用) |
| coding-gpt-6.1-sol | +6.05 | 💰 是 |
| coding-opus-5.5 | +5.27 | 💰 是 |
| coding-fable-5.1 | +13.17 | 💰 是 |
| ToB-Infra | +25.44 | 💰 是 |
| coding-gpt-6-astra | +0.00 | — |

**不是本文档作者的探针造成的** —— 我只调过 `gpt-6-astra`/`gpt-6-astra-sys`/
`step-5-preview-sys`/`deepseek-v4.1-flash-sys`/`stepcode-gpt-6.1-sol-eastus2`,
没碰过 opus-5.5、fable-5.1、ToB-Infra。十个席位跑在 `sp-team`/`sp-personal`,不经过这些池。

**这些池标着「仅可查看(非你管理的预算组)」—— 很可能是共享池,别人也在用。**
要归因只能靠 tenant-desk 的 access-logs 页面按 key 筛,但那需要浏览器登录态
(API key 读不到:`HTTP 401 This endpoint does not accept API key authentication.`)。

## 模型 → 池 → 剩余

| 模型 ID | 池 | **剩余** |
|---|---|---|
| `step-5-preview-sys` | coding-deepseek-v4.1f | ♾️ **不计费** |
| `deepseek-v4.1-flash-sys` | coding-deepseek-v4.1f | ♾️ **不计费** |
| `gpt-6.1-sol-sys-ptu` | coding-gpt-6.1-sol | ¥1999.91 |
| `stepcode-gpt-6.1-sol-eastus2` | stepcode-0903 | ¥1000.00 |
| `claude-opus-5-5-sys` | coding-opus-5.5 | ¥494.73 |
| `claude-fable-5-1-sys` | coding-fable-5.1 | ¥86.83 |
| `gpt-6-astra-sys` | coding-gpt-6-astra | ¥496.92 |

**未确认:`大模型系统-ToB-Infra`(¥2384.19 剩余)对应哪个模型。** 张先生说回头给映射。

## 三个 provider 的真实区别

**不是「公司 vs 个人」,而是「走不走预算池」。**

| provider | 端点 | key | 计费方式 |
|---|---|---|---|
| `sp-personal` | `api.stepfun.com/step_plan` | `1B1X0SND…` | StepFun 账户额度,**不走预算池** |
| `sp-team` | `api.stepfun.com/step_plan` | `2qUrZaAZ…` | 同端点另一 key,**不走预算池** |
| `modelproxy` | `mp.z01.normandyfunc.com/v1` | `ak-dghs6…` | **按上表扣池** |
| `modelproxy-anthropic` | 同上 | 同 key | 同上 |
| `dataworker` | `dataworker.bestony.com/v1` | `sk-752e2…` | 独立,`gpt-5.5` |

`sp-team` / `sp-personal` 的账户归属由张先生掌握,配置里看不出来。
**不要凭端点相同推断它俩等价** —— key 不同、额度分开。

## 修正后的结论

### 1. 真实计费只占 2.2%,而且集中在 ToB-Infra

去掉 unlimited 的 `deepseek-v4.1f`(¥7145 全是免费额度),真实支出:**¥143.47 / ¥6600**。

其中 `ToB-Infra` 一池就占 ¥115.81 —— **它是唯一在真正烧钱的池**,
但对应哪个模型还没确认。其余五池都在 1% 出头。

### 2. `step-5-preview-sys` 是免费的,这是 chain 第二跳的真正理由

```json
"chain": [
  "sp-team/step-5-preview",                    ← 主,不走池
  "modelproxy-anthropic/step-5-preview-sys",   ← ♾️ unlimited,一分钱不花
  "sp-personal/step-5-preview"                 ← 不走池
]

第二跳选它不是因为"池最宽裕"(那是按错误数字算的),而是因为它**根本不计费**。
这比"余量 ¥22854"强得多 —— 没有上限约束。

### 3. 计费模型里余量都还够,但别浪费

| 模型 | 剩余 | 评价 |
|---|---|---|
| `gpt-6.1-sol-sys-ptu` | ¥1993.86 | 1M 上下文,余量最足 |
| `claude-opus-5-5-sys` | ¥494.73 | 够用 |
| `gpt-6-astra-sys` | ¥496.92 | 够用 |
| `claude-fable-5-1-sys` | ¥86.83 | **最紧,13.2% 已用** |
| `stepcode-gpt-6.1-sol-eastus2` | ¥1000.00 | 独立池 |

**`claude-fable-5.1` 是唯一值得盯的计费池** —— 预算只有 ¥100 却已用 13.2%。

## 关于 stepcode-0903

张先生要求:**交互 harness 只有 saih,不直接启用 stepgo/stepcode。**

技术结论(实测):**不需要装 StepGo。**

- StepGo/StepCode 用同一网关 `https://mp.z01.normandyfunc.com/v1`
- 用同一个 API key
- 区别只在模型 ID:`stepcode-gpt-6.1-sol-eastus2` 只在 `/v1/models?purpose=code` 下可见

已在 `~/.saih/agent/models.json` 加该模型,按 model 覆盖 `api: openai-responses`
(不加报 400:`Function tools with reasoning_effort are not supported ... use /v1/responses`)。
实测 bash 工具、真实 seatloom MCP 均通过。

### 追加规则

**每月 15 日若该池已用 80%,追加 ¥1000。** 目前 ¥0,没到触发线。

### 但"用这个模型名就一定扣 0903"未证实

预算路由是规则表 `/desk/v2/tenant/budget-pool-route-rules`,`condition_type` 四种:

| 类型 | 示例目标 | 示例值 |
|---|---|---|
| `query` | `purpose` | `stepcode-` |
| `header` | `X-Source` | — |
| `json_path` | `$.metadata.user_id` | `stepcode-` |
| `model` | `model` | `stepcode-` |

最可能是 `model` 前缀匹配,但规则表要浏览器登录态,API key 读不到:
`HTTP 401 This endpoint does not accept API key authentication.`

## 来源

- 池金额:张先生 2026-11-02 贴的预算组页面(**a=已花,b=预算**)
- `coding-deepseek-v4.1f` 为 unlimited 不计费:张先生 2026-11-02 明确
- 模型→池映射:张先生口述
- 技术验证:实测(网关、模型可见性、API 覆盖、工具调用)
- 规则表字段:从 tenant-desk 前端 JS chunk 还原
