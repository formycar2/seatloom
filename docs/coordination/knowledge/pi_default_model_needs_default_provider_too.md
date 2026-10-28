---
memory_id: pi_default_model_needs_default_provider_too
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: fact
confidence: inferred
status: active
tier: hot
source_kind: file
source_ref: scripts/prepare-verification-build.sh
l2_ref_kind: file
l2_file_path: docs/coordination/knowledge/pi_default_model_needs_default_provider_too.md
l2_line_start: 1
l2_line_end: 40
curation: authored
l0_summary: 只设 defaultModel 不够 —— 同一个模型名可以出现在多个 provider 下,pi 会自己挑一个。必须同时设 defaultProvider 才消歧。2026-10-16 实测:只设 defaultModel=modelproxy-anthropic/step-5-preview-sys,pi 起在 modelproxy(openai 协议)上;补上 defaultProvider 才落到 modelproxy-anthropic(anthropic 协议)。
---

## 发生了什么

张先生 2026-10-16 要求 pi 启动默认用 `modelproxy-anthropic/step-5-preview-sys`。

第一步我把 `~/.pi/agent/settings.json` 的 `defaultModel` 设成
`modelproxy-anthropic/step-5-preview-sys`(带 provider 前缀,以为是标准写法)。
实测:pi 起来了,但状态栏显示 `(modelproxy) step-5-preview-sys`,
会话记录里 `api = openai-completions` —— **是另一个 provider**。

查 `models.json` 才发现:`step-5-preview-sys` 这个模型名同时存在于两个 provider:

| provider | api | 有这个模型 |
|---|---|---|
| `modelproxy` | openai-completions | ✅ |
| `modelproxy-anthropic` | anthropic-messages | ✅ |

而 `defaultModel` 的文档说明是「Startup model **ID**」—— 它要的是模型 id,
不是 `provider/model`。provider 前缀被丢掉,于是 pi 按目录顺序/凭据自己挑了一个。

## 为什么会改变未来动作

- **要指定默认模型,必须两个一起设**:
  ```json
  "defaultModel": "step-5-preview-sys",
  "defaultProvider": "modelproxy-anthropic"
  ```
  源码里 `setDefaultModelAndProvider(provider, modelId)` 就是成对写的,
  `settings-manager.js` 里 `defaultProvider` 和 `defaultModel` 是两个独立字段。
- **这很可能就是 2026-10-16 上午那次「全队静默换模型」的根因。**
  当时 6 个席位从 `stepplan-anthropic/step-5-preview` 无声无息变成
  `modelproxy-anthropic/step-5-preview-sys`,没有报错、没有配额提示、
  事件里连"为什么换"的字段都没有,我查了一圈没找到原因。
  现在看:同名模型跨 provider 存在时,任何一次重新解析都可能挑错一个,
  而挑错的表现就是"静默换了个模型"。**当时没查到这里,是因为我去查了
  cron、models.json 时间戳、设置覆盖,没去查"这个模型名是不是不唯一"。**
- **显式 `--model provider/model` 会覆盖这两个设置。** 实测确认:
  带 `--model stepplan-anthropic/step-5-preview` 起,用的就是 stepplan。
  所以 `scripts/seat-launch.sh` 用 `--model` 显式传模型的席位,**不吃这个默认值**。
  要改席位的模型得改数据库里的 `seats.default_model`,或者改启动脚本。

## 证据

- `models.json`:`modelproxy` 和 `modelproxy-anthropic` 都有 `step-5-preview-sys`
- 只设 `defaultModel: "modelproxy-anthropic/step-5-preview-sys"` →
  会话记录 `provider=modelproxy, model=step-5-preview-sys, api=openai-completions`
- 补设 `defaultProvider: "modelproxy-anthropic"` →
  会话记录 `provider=modelproxy-anthropic, model=step-5-preview-sys, api=anthropic-messages`
- 显式 `--model stepplan-anthropic/step-5-preview` → 会话记录用的是 stepplan(设置被覆盖)
- pi 1.3.0 更新日志自己写着这条:`Fixed --model choosing a provider by catalog order
  when several providers offer the match; Pi now prefers the provider with credentials
  and otherwise reports the ambiguity`
