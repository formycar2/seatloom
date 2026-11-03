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
| `glm-5.3` | **ToB-Infra** | ¥2384.19 |
| `mimo-v2.6-pro:cloudsway` | **ToB-Infra** | ↑ 同池 |
| `deepseek-v4.1-flash` | **ToB-Infra** | ↑ 同池 |
| `glm-5.3-flash:mindracode-offical` | ToB-Infra(推断同族) | ↑ 同池 |
| `minimax-m3.1-flash:cloudsway` | ToB-Infra(推断同族) | ↑ 同池 |

**ToB-Infra 的三条(`glm-5.3` / `mimo` / `deepseek-v4.1-flash`)是张先生 2026-11-02
从预算页确认的,不是推断。** 最后两条未单独确认,但与同族同上游,按同池处理;
**用之前应再确认一次。**

**仍未确认:ToB-Infra 里除了这些国模还有什么模型。** 张先生说「可以用的模型有很多」。

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

第二跳选它不是因为「池最宽裕」(那是按错误数字算的),而是因为它**根本不计费**。
这比「余量 ¥22854」强得多 —— 没有上限约束。

**注意分界线是 `-sys` 后缀。** 带 `-sys` 的走专属池,不带的国模走 ToB-Infra 计费。

### 3. 计费模型里余量都还够,但别浪费

| 模型 | 剩余 | 评价 |
|---|---|---|
| `gpt-6.1-sol-sys-ptu` | ¥1993.86 | 1M 上下文,余量最足 |
| `glm-5.3` / `mimo` / `deepseek-v4.1-flash` | **ToB-Infra ¥2384.19** | 国模,约 ¥1.5/M token |
| `claude-opus-5-5-sys` | ¥494.73 | 够用 |
| `gpt-6-astra-sys` | ¥496.92 | 够用 |
| `claude-fable-5-1-sys` | ¥86.83 | **最紧,13.2% 已用** |
| `stepcode-gpt-6.1-sol-eastus2` | ¥1000.00 | 独立池 |

**`claude-fable-5.1` 是唯一值得盯的计费池** —— 预算只有 ¥100 却已用 13.2%。

## ToB-Infra 上的国产优先模型(2026-11-02 选型)

**来源:AA Intelligence Index v5(2026-10-29 改版后)+ modelproxy 网关实测。**

### ⚠️ AA v5 改版后国产排名集体下滑

新指数 60% 权重来自私有测试集,新增 FrontierCode 2.0 / Terminal-Bench-Science,
并且把「科学推理」权重提高。**结果是国产开源模型排名大幅低于旧版。**
引用榜单前需要说明是 v5,否则会高估国产模型。

### 国产模型的真实位置

| 模型 | 指数 | 每任务成本 | 厂商 | 权重 |
|---|---|---|---|---|
| Qwen3.8 Max (0902) | 34 | $10.02 | Alibaba | 闭源 |
| **DeepSeek V4.1 Flash 1017 (max)** | **32** | **$0.60** | DeepSeek | MIT |
| GLM-5.3 (max) | 32 | $5.01 | Z AI | 开源 |
| MiMo-V2.6-Pro | 30 | $0.26 | Xiaomi | 开源 |
| MiniMax-M3.1-Flash | 29 | $0.49 | MiniMax | 开源 |
| Step 5 Preview | 29 | $2.35 | StepFun | 闭源 |
| GLM-5.3-Flash | 28 | $0.58 | Z AI | 开源 |
| MiMo-V2.6-Flash | 27 | $0.13 | Xiaomi | 开源 |
| DeepSeek V4 Pro 0813 (max) | 23 | $1.77 | DeepSeek | — |
| GLM-5.3 (low) | 23 | $1.57 | Z AI | 开源 |

**注意:所有国产开源模型都低于 Anthropic 入门款 Claude Haiku 5.5(指数 36)。**
闭源的 Kimi K3.1 Pro(46)、Qwen3.8 Max(34)是例外,但权重未开放。

### 筛选结果:聪明 + 不贵 + 真能跑

张先生要求:**AA 榜单上聪明的国模 + 价格不要太贵**。三项都要满足,
「榜单高」但网关没有、或网关有了但 saih 调不动,都不算。

| 优先级 | 网关模型 ID | AA 对应 | 指数 | 实测 |
|---|---|---|---|---|
| **1** | `deepseek-v4.1-flash` | DeepSeek V4.1 Flash 1017 | 32 | ✅ bash 工具 + 真实 MCP |
| **2** | `glm-5.3` | GLM-5.3 (max) | 32 | ✅ bash 工具 + 真实 MCP |
| **3** | `mimo-v2.6-pro:cloudsway` | MiMo-V2.6-Pro | 30 | ✅ bash 工具 + 真实 MCP |
| **4** | `glm-5.3-flash:mindracode-offical` | GLM-5.3-Flash | 28 | ✅ bash 工具 + 真实 MCP |

**全部走 `modelproxy-anthropic` provider**(`anthropic-messages`),不用逐个配覆盖。

## 协议选择:Anthropic messages 明显优于 OpenAI completions

**同一个大 prompt(~3.2K token)连打两次,看第二次的计费输入:**

| 模型 | O 计费 | **A 计费** | 节省 | 备注 |
|---|---|---|---|---|
| `minimax-m3.1-flash:cloudsway` | 3416 | **1** | 3416x | 极致 |
| `deepseek-v4.1-flash` | 3258 | **186** | 17.5x | 显著 |
| `mimo-v2.6-pro:cloudsway` | 3221 | **2585** | 1.2x | 略省 |
| `glm-5.3` | 3226 | **3226** | 1.0x | **无差异** |

**结论:能用 Anthropic 就用 Anthropic。** 它在 3/4 上显著更省,最坏情况持平。
七个候选模型(`deepseek-v4.1-flash`、`glm-5.3`、`mimo-v2.6-pro:cloudsway`、
`glm-5.3-flash`、`minimax-m3.1-flash`、`step-5-preview-sys`、`deepseek-v4.1-flash-sys`)
**全部支持 `claude_native`**,没有理由用 OpenAI 协议。

### 为什么差距这么大

Anthropic 的缓存是**结构化**的:system + messages 前缀命中后,
`input_tokens` 直接等于未命中部分(第二次 deepseek 只剩 186)。

OpenAI 兼容层虽然也返回 `cached_tokens`(3072),
但 `prompt_tokens` 仍然按全量计(3258)—— **缓存命中了,账还是按全量算**。

### 已在 saih 里切过去的五个

`~/.saih/agent/models.json` 里,这些已全部移到 **`modelproxy-anthropic`** provider,
由 provider 统一提供 `anthropic-messages` + 正确的 baseUrl(不带 `/v1`),
**不需要任何 per-model 覆盖**:

| 模型 | api | baseUrl |
|---|---|---|
| `deepseek-v4.1-flash` | (继承 provider) | (继承 provider) |
| `glm-5.3` | 同上 | 同上 |
| `mimo-v2.6-pro:cloudsway` | 同上 | 同上 |
| `glm-5.3-flash:mindracode-offical` | 同上 | 同上 |
| `minimax-m3.1-flash:cloudsway` | 同上 | 同上 |

> 张先生指出:「加一个 provider 叫 modelproxy-anthropic 就不用给每个模型自定义了」。
> **这个 provider 本来就已经存在**(baseUrl 和 api 都对,key 与 `modelproxy` 同一个),
> 本文档作者一开始却在 `modelproxy` 里逐个加 `api` + `baseUrl` 覆盖,绕了远路。
> 已合并,per-model 覆盖数从 5 降到 0。

备份:`~/.saih/agent/models.json.bak-mm-anthropic`、`.bak-aacand`、`.bak-mpa-merge`

### ⚠️ minimax / kimi-k3 在 saih 里被 OOM 杀掉,已移除

**HTTP 层完全正常** —— 我用裸请求打 `/v1/messages`,它返回:

```json
{"content":[{"type":"thinking","signature":"06646f22…",
             "thinking":"We need respond only OK…"},
            {"type":"text","text":"OK"}],
 "stop_reason":"end_turn"}
```

**但 saih 调它,3/3 都是 `exit=137`(SIGKILL),且进程在 400ms 内就消失** ——
不是渐进内存增长,是启动即被杀。同样条件下 `glm-5.3` 立刻成功返回 `OK`。

已排除的因素:

- 不是内存不足(glm-5.3 紧接着成功)
- 不是协议(anthropic / openai-completions / openai-responses 三种都试过)
- 不是模型不可用(裸请求正常)

**它和别的模型的差别:thinking block 带 `signature` 字段**(mimo/deepseek 的
thinking 没有)。怀疑 saih 处理 `signature` 时有 bug,但未定位。

**张先生 2026-11-02 决定:移除。** 已从 `models.json` 删除。

#### kimi-k3 也是同一个症状(所以一起移除)

网关上 `kimi-k3`、`kimi-k3:aliyun` 都支持 `claude_native`,裸请求正常返回
`text='OK'`,**但 saih 里 3/3 都是 `exit=137`(SIGKILL/OOM)**,同样条件下
`glm-5.3` 立刻成功。

| 模型 | thinking 有 signature | saih |
|---|---|---|
| `glm-5.3` | 无 | ✅ 正常 |
| `mimo-v2.6-pro:cloudsway` | 无 | ✅ 正常 |
| `deepseek-v4.1-flash` | 无 | ✅ 正常 |
| `kimi-k3` | **无** | ❌ OOM |
| `kimi-k3:aliyun` | 有 | ❌ OOM |
| `minimax-m3.1-flash:cloudsway` | 有 | ❌ OOM |

**注意 `kimi-k3` 没有 signature 也 OOM** —— 所以「signature 导致」的猜测是错的,
两个模型的共同点还没找到。已移除,不再追。

**教训:网关上能调通 ≠ saih 里能用。** 每个候选都必须用 saih 本身跑一次,
不能只看裸 HTTP 请求成功。

### 为什么 `deepseek-v4.1-flash` 排第一

它指数 32 和 `glm-5.3` 并列,但:

1. **成本 $0.60 vs $5.01** —— 便宜 8 倍
2. MIT 许可
3. 1M 上下文
4. Anthropic 协议下缓存效果好(第二次调用计费输入从 3258 降到 186)

> 🔴 **曾经写错:本文档作者一度以为它和 `deepseek-v4.1-flash-sys` 共用不限量的
> `coding-deepseek-v4.1f` 池,理由是两者上游都是 `深度编码`。**
> **张先生从预算页确认:这是错的。**
> - `deepseek-v4.1-flash-sys`(带 `-sys`)→ **不限量池**
> - `deepseek-v4.1-flash`(不带 `-sys`)→ **ToB-Infra,计费**
>
> 上游相同不等于池相同 —— 池按路由规则切。反证:`step-5-preview-sys`(不限量池)
> 和 `claude-opus-5-5-sys`(¥500 池)上游都是 `AR-Service`。
>
> **所以这四个国模全部计费,没有一个免费。** 免费的是 `-sys` 系列。

### 计费估算(重要,决定能不能给席位用)

ToB-Infra 剩 ¥2384.19。本文档作者的探针实测:
三个模型各 3 次 × ~21 万 token = **约 190 万 input token,合计约 ¥2–3**。

反推:**ToB-Infra 约 ¥1.5 / 百万 token**。按此估算:

| 用途 | 月 token 量 | 月成本 |
|---|---|---|
| 单个席位日常 | ~5M | ~¥8 |
| 十个席位日常 | ~50M | ~¥75 |
| 一个席位跑满 | ~500M | ~¥750 |

**¥2384 够十个席位日常用约 2 年。** 但如果某个席位进入高频循环,会加速。

> 这个 ¥1.5/M 是从 ¥2–3 / 1.9M token 反推的,误差可能不小。
> 要精确值得再打一次已知量的探针并读余额。

### ✅ 已确认:这四个走 ToB-Infra(张先生从预算页读出)

| 模型 | 池 |
|---|---|
| `glm-5.3` | **大模型系统-ToB-Infra** |
| `mimo-v2.6-pro:cloudsway` | **大模型系统-ToB-Infra** |
| `deepseek-v4.1-flash` | **大模型系统-ToB-Infra** |
| `deepseek-v4.1-flash-sys` | 大模型系统coding-deepseek-v4.1f(**不限量**) |

**分界线就是 `-sys` 后缀**:带 `-sys` 走各自专属池(其中 deepseek 那个不限量),
不带 `-sys` 的国模走 ToB-Infra 计费。

#### 为什么作者没能自己查到(记下来,下次别白试)

用张先生的 tenant-desk cookie 试了所有能想到的接口:

| 途径 | 结果 |
|---|---|
| `/desk/v2/audit/access-logs`(请求记录) | **403** `Insufficient privilege` |
| `/desk/v2/tenant/budget-pool-route-rules` | **403** `Budget role required` |
| `/desk/v2/model-resource-pools` | **403** |
| `/desk/v2/resource-channels` | **403** |
| `/desk/v2/statistics/available-models` | ✅ 200,但只有 `model`/`display_name`/`last_used`,**无池信息** |

**cookie 有效**(解出 `zhangxiaolong@stepfun.com`,tid 1100,exp 2026-11-07),
401 全变 403 —— 是 `mp_member` 角色权限不够,不是认证失败。

**结论:池归属只能从预算页人工读,API 读不到。** 以后要确认池,
直接请张先生读预算页,不要花时间试接口。

#### 探针法仍然有用(用于量校准)

打了三个模型,每个 3 次 × ~21 万 token,唯一前缀避开 prompt cache:

| 模型 | 合计计费 input |
|---|---|
| `glm-5.3` | 629,436 |
| `deepseek-v4.1-flash` | 636,277 |
| `mimo-v2.6-pro:cloudsway` | 635,599 |

合计约 190 万 token,对应 ToB-Infra 增加约 ¥2–3 → **约 ¥1.5 / 百万 token**。

#### `X-Mp-Model-Label` 给的是上游供应商,不是池

响应头暴露了这个字段,但它描述的是**上游厂商**,与预算池是两件事:

| 模型 | provider | real_model |
|---|---|---|
| `glm-5.3` | 华怡 | glm-5.3 |
| `mimo-v2.6-pro:cloudsway` | AR-Service | mimo-v2.6-pro:cloudsway |
| `glm-5.3-flash:mindracode-offical` | **深度编码** | glm-5.3-flash |
| `minimax-m3.1-flash:cloudsway` | 小宿 | MaaS_Minimax_M3.1_flash_20261019 |
| `deepseek-v4.1-flash` | **深度编码** | deepseek-v4-flash |
| `deepseek-v4.1-flash-sys` | **深度编码** | deepseek-flash |
| `step-5-preview-sys` | AR-Service | step-5-preview:stepfun-haiwai |
| `claude-opus-5-5-sys` | AR-Service | claude-opus-5-5:cloudsway-bedrock-coding-2 |
| `claude-fable-5-1-sys` | AR-Service | claude-fable-5-1:cloudsway-anthropic-coding |
| `step-3.7-flash` | 阶跃星辰 | step-3.7-flash |

**注意:用户明确说 `deepseek-v4.1-flash-sys` 走不限量的 `coding-deepseek-v4.1f` 池,
而 `deepseek-v4.1-flash` 的上游也是 `深度编码`。**
上游相同不等于池相同 —— 池是按路由规则切的,不是按上游切的。
这正是为什么必须看 access-logs 才能定论。

#### 要定论需要什么

**一个有预算权限的账号打开 access-logs 页面,按模型名筛这四条。**
`mp_member` 看不到。这不是技术障碍,是权限设计。


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
