# AEGIS-2026-10-10-washing-pipeline-design-v1

把 11 个月、271 个 session、14 GB 的历史 agent transcript 洗成 project+topic 作用域下可召回的 golden
memory。本文是 Phase 2 的实现契约,供洗数据专职席位(Vela)照此实现。

| 字段 | 值 |
|---|---|
| template | T2 |
| subtype | design |
| id | AEGIS-2026-10-10-washing-pipeline-design-v1 |
| status | draft — 待张先生定两处决策(§7) |
| author | aegis |
| date | 2026-10-10 |
| version | v1 |
| tags | aegis, design, d5, knowledge-layer, washing, corpus-ingest |

---

## 1. 为什么做 / 现状实测

张先生的框架:**project + topic 是唯一作用域**;harness + LLM 只是服务于它的执行实体;data 经提炼成
context → memory,cold→hot 生命周期也在该框架下。SeatLoom 的目的是**数据+AI 驱动的 AI Assistant 基座**,
历史 harness session 都归它管。

**已实测的原料(非推断):**
- `archived_sessions` = 271 行;**14 GB / 481 万行 / 2025-11-20 → 2026-10-08 / 5 种 harness 目录**
  (codex 210 / stepcode 20 / claude-code 18 / cursor 13 / gemini 9 个文件)
- 原始文件在 `~/Documents/GitHub/AgentHistory/raw/<harness>/`,DB 里只有索引(`raw_path`+`sha256`+`lines`)
- **project 名自带层级**:`sf-code/storage`、`sf-code/brainpp`、`ai-assistant/android-agent`、
  `fde/field_evaluation` —— 斜杠就是子项目

**已有的前置工作(张先生 2026-09-28 建的,不要重做):**
`AgentHistory/` 下已有 `BUILD-PROMPTS.md`(6 阶段手册)、`_manifest.json`(271 条,字段与
`archived_sessions` 一一对应)、`summaries/`(**271 份 per-session 摘要**)、
`by-project/<project>/{SUMMARY.md,sessions.md}`(**18 个项目**)。

**但这层是「索引+存根」,不是「知识」**(实测):`by-project/Assistant_Aegis/SUMMARY.md` 仅 1023 B,
「做过什么」栏写 `(见 summary)` 或直接粘 AGENTS.md 原文;「待接手」是**空勾选框**
(`[ ] 汇总各 session 的未竟事项`)—— 手册自己承认没做完。OfficeBench 有 66 session,SUMMARY 仅 3003 B。
且全是 markdown 散文:**不可查询、无结构、不在 SeatLoom 里**;召回方式是"先知道是哪个项目,再去读文件"。

**所以 Phase 2 要洗的不是 `raw → summary`(已做,薄),而是 §3 的五段。**
现有 `summaries/` 与 `by-project/` 作为**提示与交叉校验**使用(它们已标出每 session 的 project/时间/
参与 agent),不作为唯一输入 —— 原始 `raw/` 才是真相。

## 2. 落点(migration 015,Onyx 交付,Flux 已验绿)

`memories` 表的结构与本设计吻合,直接用,不要另造:
`project_id` + `topic_id` 作用域;`l0_summary`(NOT NULL,唯一默认层)、`l1_overview`(按需);
**L2 只存指针** `l2_ref_kind(document|file)` + `l2_file_path` + `l2_body_digest`;
`source_kind` 已含 **`archived_session`**,`source_ref` 格式 `'<harness>:<id>'`;
`heat` / `recall_count` / `last_recalled_at` / `tier(cold|warm|hot)` / `status(active|forgotten)`。
`projects.parent_project_id` 已加(自引用 FK,`projects_parent_not_self` 只挡 1-环)。

## 3. raw → golden:五段(只有 S3 用 LLM)

### S0 归一化(确定性,零 LLM)
每种 harness 一个 adapter,统一成 canonical turn 流:
`{seq, ts, role, kind, text, tool, refs}`,`kind ∈ {user_msg, assistant_msg, tool_call, tool_result, meta}`。
已实测的格式差异:
- **codex** — `{timestamp, type, payload}` 统一信封
- **claude-code** — `{type, sessionId, ...}`(首几行是 `leafUuid`/`mode`/`permissionMode` 等控制行)
- **stepcode** — `{ts, type, sessionId, agent, hookEvent, toolName, model, cwd, ccSessionId}`
- **cursor** — **不是 jsonl**,是 sqlite `state.vscdb`(`ItemTable` 里 `composer*`/`aiService*`/
  `workbench.panel.aichat*` 键);见 BUILD-PROMPTS.md 源清单
- **gemini** — history/memory 文件,非 jsonl
这是唯一需要按 harness 分支的代码。**adapter 必须对未知行容错**(记为 `meta` 并计数,不得整文件失败)。

### S1 脱敏(强制,在 LLM 之前、在任何东西落库之前)
张先生自己的红线(BUILD-PROMPTS.md):「原始会话含 API Key/cookie/内网信息……摘要里一律脱敏」。
对归一化流扫描:`ak-*` / `sk-*` / `ghp_*` / `AKIA*` / JWT / PEM 块 / `Bearer ` / 65 字符 stepplan key /
cookie / 内网域名 + 高熵串启发。替换为 `⟪REDACTED:<类型>:<指纹>⟫`,**指纹 = sha256 前 6 位** ——
同一密钥在不同位置可被识别为同一个,内容不泄露。
**验收要求**:在已知含密钥的样本上必须命中(仓库里现成的真实样本:`gitlab_pat.txt`、
`key/stepplan_key_unlimit.txt` 的值曾出现在会话里)。漏报按缺陷处理。

### S2 切 topic ⚠️ 本方案最弱一环
4.8M 行不是「一 session 一摘要」,而是**多个主题**。按信号切段:静默 > 30 min、`cwd` 变化、
显式任务边界、tool-call 聚簇。每段 = 一个候选 topic(写入 `topics`,可用 `parent_topic_id` 表达父子)。
**这是 `topics` 表真正产生层级的唯一来源。**
**此步无外部设计可参照,是 Aegis 自行构造,最可能错。** 切错 → topic 树是垃圾 → 后面所有召回建在沙上。
因此 §6 的试点**专门验这一步**,不过关不得继续。

### S3 抽取(LLM;用 `stepplan-wash*` provider)
每段抽**带类型的记录,不是散文**。六类:

| kind | 抽什么 | 为什么值得召回 |
|---|---|---|
| `decision` | 定了什么 + 为什么 + 否掉了哪些备选 | 回答「当初为什么选 X」 |
| `failure` | 什么坏了 + 根因 + 怎么修 | **最高价值**(见下) |
| `fact` | 系统的持久真相 | 如「pi 读 .pi/mcp.json,Claude Code 读 .mcp.json」 |
| `preference` | 张先生希望怎么做事 | 喂给 Aegis 的人格 |
| `artifact` | 产出的文件/commit/文档 | 血缘 |
| `unresolved` | session 结束时的未竟线头 | 正是那个一直空着的勾选框 |

`failure` 排第一有实证依据:2026-10-10 这一天 Aegis 写进 memory 的条目绝大多数是 failure
(pi/MCP 的三条件、接受按钮会删已合并文件、重启把 seats 打回 Claude Code),且当天**全部复发过**。

### S4 分层
`l0_summary` 一句话断言(~100 tok,唯一默认层)/ `l1_overview` 断言+约束+怎么用(~2K,够决策)/
L2 = `l2_file_path` + `l2_body_digest` 指向 `raw/`。**绝不拷贝原文入库**(14 GB),原始文件是唯一真相;
L0/L1 是**可重炼的派生物**。

### S5 入库(幂等)
斜杠 project 路径 → `projects` 树(**深层环检测是本流水线的责任**,015 只挡 1-环,Onyx 已注明甩给
Phase 2 / Aegis lane);S2 的段 → `topics`;记录 → `memories`。
按内容摘要去重;按 session `sha256` 记 checkpoint,中断可续跑、重跑零重复。
`(root)` 那 13 个无归属 session **需要明确归属策略,不得猜** —— 上报 Aegis 定。

## 4. golden data 格式(示例)

```
{ project_id: "sf-code/storage",  topic_id: "<段 id>",
  kind: "decision",
  title: "存储后端选型",
  l0_summary: "存储后端选了 X 而非 Y,因为 Y 的跨区一致性需要额外协调器",
  l1_overview: "<断言 + 当时约束 + 否掉 Y 的具体理由 + 现在如何套用>",
  l2_ref_kind: "file",
  l2_file_path: "raw/codex/rollout-2026-03-03T21-59-57-....jsonl",
  l2_body_digest: "82a97b03b081aef5...",
  source_kind: "archived_session",  source_ref: "codex:rollout-2026-03-03T...",
  tags: ["storage","选型"],  tier: "cold",  heat: 0,  status: "active" }
```

## 5. 召回方法(两条路,**不做向量化** —— 张先生 2026-10-10 明确排除)

1. **结构召回(默认)** — 沿 `projects` 树 + `topics.parent_topic_id` 走,返回**带父上下文的 L0 子树**,
   而非孤立行。零新基础设施。
2. **词法召回** — 对 `l0_summary` + `l1_overview` 建 `pg_trgm` / `tsvector` 索引。
   **`pg_trgm` 1.6 已安装在 live 库**,今天就能用。

**明确不做**:pgvector / embedding。实测:live 是 `postgres:16-alpine`,`vector` 扩展不可用
(`pg_available_extensions` 无,`pkglibdir` 只有 `pg_trgm.so`);换官方 pgvector 镜像是 Debian(glibc),
与现 Alpine(musl)**collation 不同,跨 libc 迁移需 REINDEX 文本索引**;且**当前 6 个 provider 全是
chat/completion,无确认的 embedding 端点**。schema 保持可增量加列(将来真要加,走 018,不为此改现表)。

**召回留痕**:每次 recall 记一条 `canonical_event` —— 账本从「发生了什么」扩展到「知道了什么」。

## 6. 试点与验收(先试点,绝不一上来烧全量)

1. 先洗 **`seatloom` 那 16 个 session**(规模小、内容是我们自己最熟的,错了看得出来)。
2. **S2 专项验收**:切出的 topic 列表交张先生**肉眼判断「是不是一个个独立的工作主题」**。不过关则停。
3. **S1 专项验收**:在已知含密钥样本上必须命中;漏报=缺陷。
4. 抽样人读 L0/L1,判断「是否真抓住了要点」而非复述。
5. 以上全过 → Flux 技术验证 → 张先生看样本 → **才放开 271 个 / 14 GB**。

## 7. 待张先生决策(两处,实测发现)

- **D1 L2 溯源粒度**:015 只有 `l2_file_path` + `l2_body_digest`,**无行号区间** → 溯源是文件级。
  在 14 GB 里定位一条断言会很糙。选项:(a) 接受文件级;(b) 让 Onyx 增量加 span 列(migration 018)。
  **Aegis 倾向 (b)** —— 没有行级锚点,「可验证不是幻觉」这条就落不实。
- **D2 cold→hot 是否自动**:Onyx 定了 `heat` **永不自动衰减**,tier 转换**只能靠显式 promote 工具,
  无后台 daemon**。若张先生说的「cold→hot 生命周期管理」指系统自动升降温,则与此冲突,需改设计。

## 8. 归因(诚实记账 —— 半年后别把「自己推的」当「业界标准」)

**仅对 OpenViking 做了联网核实**(官方文档 + 仓库页 + 第三方评测);Letta/Graphiti/Cognee/ReMe/memsearch
**未读源码**,用的是公开描述 + 训练印象。star 数不引用(搜到 OpenViking 有 16.5k/21.7k/30k 三个版本,
均来自不同时间的软文)。

| 设计点 | 来源 | 可信度 |
|---|---|---|
| L0/L1/L2 渐进披露 | OpenViking(`.abstract.md`/`.overview.md`) | 高,读过文档 |
| 返回带父上下文的子树 | OpenViking「目录递进检索」 | 高 |
| session → 抽取记忆 | OpenViking「commit 会话 → 抽成可审阅 Markdown」 | 高 |
| 召回留痕 | OpenViking「可视化检索轨迹」+ 映射到我们已有事件账本 | 混合 |
| 记忆操作即 Agent 工具 | Letta / MemGPT 核心主张 | 中,未读码 |
| ECL 流水线形状 | Cognee | 低,只取名字所述形状 |
| 文件是真相、索引可重建 | ReMe / memsearch | 低,**趋同多于借鉴**(我们已是 sha256+raw_path) |
| 双时态纪律 | Graphiti (Zep) | 低,且 `canonical_events` 已有 occurred_at+created_at |

**我们自己的(非借鉴):**
- **project + topic 作为唯一作用域轴** — 张先生的框架。此处**故意背离 OpenViking**
  (它是 `viking://resources|user|agent`,agent/user 为轴);两套寻址并存 = 碎片化,必须选一个。
- **强制脱敏在 LLM 之前** — 张先生自己的 BUILD-PROMPTS.md 红线。
- **六类 typed record** — **无外部出处**。来自 ① 手册里那个没填的「待接手」勾选框 → `unresolved`;
  ② 2026-10-10 当天 Aegis 写入 memory 的内容绝大多数是 `failure` 且全部复发过。
- **用 `pg_trgm` 而非向量库** — **故意不跟**那几个项目(它们都上向量库)。
- **S2 按静默/cwd/tool 聚簇切 topic** — Aegis 自行构造,**无外部验证,最弱一环**。

## 9. 红线

- 🔴 **脱敏先于 LLM、先于落库**(§S1)。漏报按缺陷处理。
- 🔴 **不得产生第二个权威源**:提炼产物一律落 SeatLoom;`raw/` 只读,不改不删。
- 🔴 **不引入外部记忆系统代码**(OpenViking/Mem0/Letta 等只吸收模式)——
  张先生要 SeatLoom 做基座,基座里套基座自相矛盾。
- 🟡 **先试点后全量**(§6)。即便预算近无上限,14 GB 的墙上时间仍然真实。
- 🟡 **`(root)` 13 个无归属 session**:不得猜,上报 Aegis。

## 10. 前置条件(Vela 开工前必须满足)

1. **015 必须在 live 库**(当前 track 最高 014,live 无 `memories` 表)—— 走验收流程落地,Aegis 执行。
2. **`seat-vela-001` 必须先注册进 `seats` 表** —— `dispatches.target_seat_id` 有 FK 到 `seats(id)`,
   未注册则任何派单插不进去。Aegis 动作(席位注册完整性是 AEGIS.md 的 Owns 条目)。
3. **provider 就绪** ✅ 已完成并冒烟验证:`stepplan-wash`(openai-completions)/
   `stepplan-wash-anthropic`(anthropic-messages),模型 `step-5-preview` + `step-3.7-flash`。
4. 建议 S2/初筛用 `step-3.7-flash`(吞吐),`decision`/`failure`/`unresolved` 三类高价值用
   `step-5-preview`。预算近无上限,全用 `step-5-preview` 亦可,代价只是墙上时间。
