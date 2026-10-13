# sf-code 历史 session 清洗方案(待张先生确认)

**状态**:草案,**未派单**。按你 2026-10-13 的要求"先设计、你确认再派"。
**派单方**:Aegis · **执行方(待定)**:洗数据席位 seat-vela-001

---

## 1. 范围与优先级(按你的原话定的)

你说:"目前优先洗 sf-code 下面的历史 session,我要接着做 cn-gpu-infer-fabric 以及 stepmind/brainpp 下面的 oss quota 管理相关的开发"。

我把 270 个 session 按 **cwd**(不是内容关键词)归类,精确范围是:

| 批次 | 项目 | session 数 | harness |
|---|---|---|---|
| **P0(你点名要接着做的)** | `cn-gpu-infer-fabric` | 4 | codex 2 / claude-code 1 / stepcode 1 |
| **P0** | `sf-code/stepmind/brainpp`(oss quota) | 2 | codex |
| P1 | `sf-code/storage` | **58** | 主要是 codex |
| P1 | `sf-code/brainpp` | 25 | 主要是 codex |
| P2 | `sf-code/stepatlas` | 4 | codex |
| P2 | `sf-code/jobutils` / `stepcloud` 等 | 5 | codex |
| | **合计** | **98** | codex 92 / claude 2 / stepcode 2 / cursor 0 / gemini 0 |

**为什么按 cwd 定而不是按内容搜**:内容搜会命中"提到了这个项目但其实在干别的"的会话,把范围吹大。按 cwd 定,一个会话只属于它真正干活的那个仓库。

**建议:先做 P0(6 个)验证整条链,再做 P1(83 个)。** 理由是 P0 是你立刻要用的,而且 6 个会话足够暴露方案的问题;等 P1 跑完再发现问题,返工代价大一个量级。

---

## 2. 数据格式(复用现有语料库,不另起一套)

现有 SeatLoom 语料库已经有 123 条记忆、25 个主题、六类 typed record。**新洗的记录必须落进同一套结构**,否则界面读不出来、`recall` 召不起来。

### 2.1 一条记录长什么样

```
title          标题(人读的第一眼)
kind           六类之一:decision / failure / fact / unresolved / preference / artifact
confidence     二值:quoted(原话这么说)/ inferred(推断得出)
l0_summary     第一层:约 100 token,默认只显示这个
l1_overview    第二层:约 2K token,按需才取
l2_file_path   第三层:证据文件路径(仓库相对路径或 ~ 展开)
l2_line_start / l2_line_end   证据行号区间
source_kind    archived_session / canonical_event / document / manual
source_ref     来源 session 标识
topic_id       所属主题
```

### 2.2 主题树怎么长

沿用现有的 `project → topic` 两层。建议:

```
seatloom/operating-knowledge               ← 现有(本项目运行知识)
├─ sf-code/storage                          ← 新:P1
├─ sf-code/brainpp                          ← 新:P1
├─ sf-code/stepmind/brainpp                 ← 新:P0(oss quota)
└─ cn-gpu-infer-fabric                      ← 新:P0
```

**每个主题下再按工作主题分段**(不是按会话文件分段)—— 沿用上次洗数据席位切 25 个主题的做法,让"这是一个独立工作主题"成立。

---

## 3. 清洗方案(五种 harness → 一种规范格式)

这是技术上最难的一段,因为 **5 种 harness 的记录格式互不相同**:

| harness | 格式特点(实测) |
|---|---|
| codex | `rollout-*.jsonl`,主体是 response_item |
| claude-code | `*.jsonl`,类型有 system / user / attachment / assistant / custom-title / agent-name / file-history-snapshot / last-prompt |
| cursor | **没有 cwd 字段**(13 个全部),要从内容里推项目归属 |
| gemini | **没有 cwd 字段**(9 个全部) |
| stepcode | `*.jsonl`,格式接近 claude-code |

**方案:先归一化成统一的 turn 流,再提炼。** 统一流的字段:

```
{ seq, ts, role(user|assistant|system|tool), kind, text, refs }
```

- **cursor / gemini 没有 cwd** → 不猜。要么从内容里找到明确的仓库路径证据才归类,要么**单独放一个"归属不明"主题**,不硬塞。这条和整个项目的纪律一致:猜出来的归类比没有归类更糟。
- **`file-history-snapshot` / `attachment` 这类元数据行** → 不当正文。上次审计已经抓到过:有 3 条记忆的"证据"指向 `{"type":"mode",...}` 这种元数据行。**归一化阶段就要把它们过滤掉,不能留给提炼阶段。**

---

## 4. 剪枝与脱敏(沿用上次的判据)

| 项 | 规则 |
|---|---|
| 剪枝目标 | 送进模型的内容砍掉 80-90%,**L2 行号指针一律不动** |
| 脱敏 | 必做。已知密钥样本必须全命中并指纹化;入库文本脱敏命中必须为 0 |
| 丢弃 | **凑不出诚实行号区间的记录,丢弃并记录原因和条数**。不许"证据在这个大文件里某处" |
| 不扩大 | 本轮只洗 §1 定的 98 个,不扩到其余 172 个 |

---

## 5. 质量门(达不到就不入库,不是警告)

上次全量审计查出的问题,这次在**入口**就拦掉,而不是等洗完了再审计:

1. **每条记录的 L2 必须开得出原文**(路径存在 + 行号不越界)
2. **区间里的文本要和这句话有实质词汇重叠** —— 上次有 3 条标着 `quoted` 却指向元数据行,那就是入口没拦
3. **`quoted` 必须真的是原话**。综合结论一律标 `inferred`。上次有 3 条把横跨多个会话的综合结论标成 `quoted`
4. **不静默丢弃前置/依赖**。抓不到就明说,不悄悄放过
5. 交付必须附统计:原始行数 → 剪枝后行数、脱敏命中数、**因凑不出行号而丢弃的条数及原因**

---

## 6. 我要你确认的(确认完我就派)

1. **范围**:P0 6 个 → P1 83 个 → P2 9 个,这个分批顺序可以吗?还是你要一次全洗?
2. **cursor / gemini 那 22 个没有 cwd 的**:按"归属不明单独放"处理,还是本轮跳过?
3. **主题树的粒度**:按仓库分(如上),还是按"工作主题"再细分?
4. **记录语言**:标题/摘要用中文(和现有一致),还是英文?(影响 `recall` 的检索效果)
5. **cn-gpu-infer-fabric 的那 4 个会话**:它的工作区现在有 3 个脏文件 + 35 个未跟踪。洗之前要不要我先把它的状态存住?

---

## 7. 和另外三件事的关系(不冲突,并行)

- **推送**:未提交项目推 github(个人)/ gitlab `zhangxiaolong/`(公司)。**和清洗无关,可以同时做。**
- **意图入口**:界面席位正在做最后一张单,和清洗无关。
- **`⌘K` 重复绑定**:我裁过还没人执行,和清洗无关。

**清洗不等任何一件,那三件也不等清洗。**
