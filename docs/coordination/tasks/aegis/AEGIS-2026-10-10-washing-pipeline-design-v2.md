# AEGIS-2026-10-10-washing-pipeline-design-v2

把 **seatloom 这一个 project** 的历史 agent transcript 洗成 project+topic 作用域下可召回的 golden memory,
并同时建立**前向记录**通道。v2 相对 v1 的改动全部来自张先生的四问(目的 / token 值不值 / schema 完善否 /
会不会丢信息),逐条落在 §2、§3、§4、§7。

| 字段 | 值 |
|---|---|
| template | T2 |
| subtype | design |
| id | AEGIS-2026-10-10-washing-pipeline-design-v2 |
| supersedes | AEGIS-2026-10-10-washing-pipeline-design-v1 |
| status | ready-to-dispatch |
| author | aegis |
| date | 2026-10-10 |
| version | v2 |
| tags | aegis, design, d5, knowledge-layer, washing, forward-capture |

---

## 1. 作用域(v2 收紧:只做 seatloom)

张先生 2026-10-10 明确:**不从 14 GB 开始,从 seatloom 这个 project 开始。**

| | seatloom | 全量 |
|---|---|---|
| sessions | **16** | 271 |
| 字节 | **168 MB** | 14 GB(占 **1.2%**) |
| 行数 | **58,015** | 481 万(占 1.21%) |
| 跨度 | 2026-04-24 → 2026-10-08 | 2025-11-20 → 2026-10-08 |

按 harness:codex 6(134 MB / 42,251 行)、claude-code 8(34 MB / 15,760 行)、cursor 1、gemini 1。

**实测注意(v2 新增)**:
- `raw/cursor/seatloom__d73725.../` 是**目录**,里面**已有 `chat_dump.json`**(还有 `state.vscdb`)→
  cursor adapter 可直接读 `chat_dump.json`,**不必解 sqlite**。
- `raw/gemini/seatloom/` 是**目录**,内含 `history`;但 manifest 记的是 `0 bytes / 0 lines`。
  → **`archived_sessions.bytes/lines` 对非 jsonl harness 不可信**,不得用它判断规模或判空。
- 本作用域内 **97% 的内容在 codex + claude-code 两种格式**里 → v1 要求的 5 个 adapter 收敛为
  **2 个必做 + 2 个按需**(cursor 走 chat_dump.json;gemini 先只登记不抽取)。

**扩大到其余 17 个 project 不是自动的下一步**,需张先生另行决定。

## 2. 目的(v2 新增:三条可检验的结果)

不讲抽象。今天(2026-10-10)实际发生的三件事就是目的的反面:

1. **今天每一份派单 payload 都在重复内联同样的上下文**(track 在哪个 commit、013/014 已 apply、分支
   stale)。铁律「payload 必须自包含」**就是没有共享记忆的税,按派单次数缴**。
2. **Lyra 刚警告「dispatch.rs 的活必须串行」,Aegis 十分钟后就踩了**(单独合 mcp-emit → track 编译炸)。
   警告**存在于一份交付说明里**,没有被召回。
3. **张先生不得不告知 AgentHistory 已有运行手册 + 271 份摘要**,否则 Aegis 会整套重建。

**验收即以此为准(非主观满意度):**
- **O1** 派单 payload 不再重复内联已有事实,改为 recall 引用。
- **O2** 已记录的 failure 不再复发第二次(同一根因)。
- **O3** 张先生不再是系统的最后一道记忆。

达不到 O1–O3,本次清洗即为无效投入。

## 3. Token 经济学(v2 新增:剪枝阶段 + 诚实算账)

**原始体积 ≠ 信号。** 58,015 行里绝大多数是 `tool_result` 正文(文件内容、命令输出)、重试、控制行。
**价值不与输入体积成正比** —— 58k 行预估只产出 100–400 条耐久记录。

因此 v1 缺了一个阶段,v2 补上 **S2.5 剪枝**(§5)。剪掉 tool_result 正文(只留引用 + 成功/失败)、
重试轮次、meta 行 —— 预期砍掉 **80–90%** 体量,**token 账差一个数量级**。这是比"换便宜模型"大得多的杠杆。

**并且 v2 确立双通道(回答"值不值"的根本):**

| 通道 | 成本 | 信号密度 | 何时做 |
|---|---|---|---|
| **前向记录**(§6) | 近零(协议改动,不是流水线) | **高** —— 今天反复踩的坑就是今天产生的 | **立即,与 schema 加固并行** |
| **回溯清洗**(§5) | 一次性考古成本 | 中 —— 最值钱的部分在当前不碰的项目里 | schema 加固**之后** |

两者**共用同一套 schema 与召回层**,所以不是二选一,只是顺序。前向记录先行的理由:它便宜、信号密度高、
且**每天都在产生新语料**;回溯清洗是存量,不会跑掉。

## 4. Schema 加固(migration 018,**清洗前必须完成**)

张先生问「schema 是否完善」。**实测 015 的 `memories` 22 列,答案是不完善 —— 五个真缺口。**
**这是 v2 把清洗排在 schema 之后的全部理由:往装不下价值的容器里灌 token,补完还得重洗,等于重花一次。**

| # | 缺口(实测:015 中 0 处) | 后果 | 要求 |
|---|---|---|---|
| **G1** | **无 `kind` 列**(只有 `l2_ref_kind`/`source_kind`,都不是记录类型) | 六类记录只能塞进无约束的 `tags[]`;**问不出「这个项目所有的 failure」** | 加 `kind TEXT NOT NULL` + CHECK 词表:`decision\|failure\|fact\|preference\|artifact\|unresolved`。值域契约逐字出现在两侧 packet |
| **G2** | **无 supersede / 关系** | 表达不了「这条决策后来被推翻」。**把过期决策当现行召回,比没有记忆更糟**。今日实例:`base='track'` 曾是事实后来错;「seats 跑 Claude Code」曾真后来假 | 加 `supersedes TEXT NULL REFERENCES memories(id)`(与 `documents.supersedes` 同风格)+ 一张 `memory_relations(from_id, to_id, rel)`,`rel` 词表至少 `supersedes\|caused_by\|fixed_by\|refines`。召回默认**只回未被 supersede 的** |
| **G3** | **`unresolved` 无完结态**(`status` 仅 active\|forgotten) | **最可执行的一类,永远无法被关闭** —— 正是 BUILD-PROMPTS.md 里那个一直空着的勾选框 | `status` 词表加 `resolved`,并加 `resolved_by TEXT NULL` + `resolved_at TIMESTAMPTZ NULL`。`forgotten`(软墓碑)语义保持不变 |
| **G4** | **无 `confidence`** | 推断出的决策与原文明说的**长得一模一样**;因 L2 只到文件级,此项更要命 | 加 `confidence TEXT NOT NULL` + CHECK:`quoted\|inferred`(二值,不要分数——LLM 自评分数不可信) |
| **G5** | **L2 无行号区间**(只有 `l2_file_path`+`l2_body_digest`) | 「证据在这个 134 MB 文件里某处」≈ 没有证据。**这是全部丢失风险的收敛点(§7)** | 加 `l2_line_start INTEGER NULL` + `l2_line_end INTEGER NULL`;当 `l2_ref_kind='file'` 且 `source_kind='archived_session'` 时**必填**(CHECK 强制) |

**同时确定 D2(heat 升降温)**:保留 Onyx 的「**无后台 daemon、无自动衰减**」—— 该设计是对的。
张先生要的 cold→hot 由**召回路径本身**驱动:`recall` 命中即 `heat+1`/`recall_count+1`/`last_recalled_at`,
跨阈值时由 recall 工具**显式**调 promote。**不新增后台进程**,语义仍是"显式转换",与 015 契约不冲突。

全部 additive + 幂等(第二遍 apply 零 catalog 写入),遵守既有纪律:跨 Rust⇄PG TEXT 边界的枚举手写
`FromStr`/`Display`,不靠 serde derive。

## 5. raw → golden:六段(只有 S3 用 LLM)

### S0 归一化(确定性,零 LLM)
统一成 canonical turn 流 `{seq, ts, role, kind, text, tool, refs}`,
`kind ∈ {user_msg, assistant_msg, tool_call, tool_result, meta}`。
- **codex**(必做)— `{timestamp, type, payload}` 统一信封
- **claude-code**(必做)— `{type, sessionId, ...}`;前几行是 `leafUuid`/`mode`/`permissionMode` 控制行
- **cursor**(按需)— 读目录内 `chat_dump.json`,**不解 sqlite**
- **gemini**(本轮只登记不抽取)— 目录内 `history`,格式未探明;**不得因它阻塞主线**
adapter **必须对未知行容错**:记为 `meta` 并计数,**不得整文件失败**。

### S1 脱敏(强制;在 LLM 之前、在任何东西落库之前)
张先生自己的红线(BUILD-PROMPTS.md):「原始会话含 API Key/cookie/内网信息……摘要里一律脱敏」。
扫描 `ak-*`/`sk-*`/`ghp_*`/`AKIA*`/JWT/PEM 块/`Bearer `/65 字符 stepplan key/cookie/内网域名 + 高熵串启发。
替换为 `⟪REDACTED:<类型>:<指纹>⟫`,**指纹 = sha256 前 6 位**(同一密钥可识别为同一个,内容不泄露)。
**验收**:在已知含密钥样本上必须命中;漏报按缺陷处理。

### S2 切 topic ⚠️ 本方案最弱一环(Aegis 自构,无外部验证)
按信号切段:静默 > 30 min、`cwd` 变化、显式任务边界、tool-call 聚簇。每段 → `topics` 行
(可用 `parent_topic_id` 表达父子)。**这是 `topics` 真正产生层级的唯一来源。**
切错 → topic 树是垃圾 → 所有召回建在沙上。**§8 试点专门验这一步,不过关不得继续。**

### S2.5 剪枝(v2 新增,token 杠杆)
在送 LLM 之前丢弃:`tool_result` **正文**(保留 `tool` 名 + 成功/失败 + 字节数 + **行号区间**)、
同一操作的重试轮次(保留最终态 + 重试次数)、`meta` 控制行。
**必须记录剪枝统计**(原始行数 → 入 LLM 行数),作为交付证据。
**红线**:剪枝只影响"送给 LLM 的内容",**不影响 L2 指针** —— 被剪掉的证据仍可经 `l2_line_start/end` 回溯
(这就是 G5 为何是承重墙)。

### S3 抽取(LLM;`stepplan-wash*` provider)
每段抽**带类型的记录,不是散文**。六类即 G1 的词表:

| kind | 抽什么 | 为何值得召回 |
|---|---|---|
| `decision` | 定了什么 + 为什么 + **否掉了哪些备选** | 回答「当初为什么选 X」 |
| `failure` | 什么坏了 + 根因 + 怎么修 | **最高价值**(见下) |
| `fact` | 系统的持久真相 | 如「pi 读 `.pi/mcp.json`,Claude Code 读 `.mcp.json`」 |
| `preference` | 张先生希望怎么做事(**尤其是他纠正 Aegis 的地方**) | 喂给 Aegis 的人格 |
| `artifact` | 产出的文件/commit/文档 | 血缘 |
| `unresolved` | session 结束时的未竟线头 | 正是那个一直空着的勾选框 |

`failure` 居首有实证:2026-10-10 Aegis 写入 memory 的条目绝大多数是 failure(pi/MCP 三条件、接受按钮会删
已合并文件、重启把 seats 打回 Claude Code),且**当天全部复发过**。
每条必须带 `confidence`(quoted/inferred)与 L2 行号区间;**凑不出行号的记录不得入库**。

### S4 分层
`l0_summary` 一句话断言(~100 tok,唯一默认层)/ `l1_overview` 断言+约束+怎么用(~2K)/
L2 = `l2_file_path` + `l2_body_digest` + **行号区间**,指向 `raw/`。
**绝不拷贝原文入库**;原始文件是唯一真相;L0/L1 是**可重炼的派生物**。

### S5 入库(幂等)
斜杠 project 路径 → `projects` 树,**深层环检测是本流水线的责任**(015 只挡 1-环,Onyx 已注明甩给
Phase 2 / Aegis lane)。S2 的段 → `topics`;记录 → `memories`(带 `kind`/`confidence`/关系)。
按内容摘要去重;按 session `sha256` 记 checkpoint → 中断可续跑、重跑零重复。
本作用域内无 `(root)` 问题(那 13 个无归属 session 属全量范围,本轮不触碰)。

## 6. 前向记录(v2 新增,与 schema 加固并行启动)

**协议改动,不是流水线** —— 成本近零,信号密度最高。

- 席位**完成派单时**,除交付说明外,用 `remember` 写入本次产生的 `decision`/`failure`/`unresolved`
  (每类 0–3 条,**宁缺勿滥**),`source_kind='canonical_event'`,`source_ref` 指向派单生命周期事件。
- Aegis **解决跨席位升级后**写 `failure`(根因 + 修法)。今天那三条(pi/MCP、接受按钮、重启)就是范本。
- **张先生纠正 Aegis 时** → `preference`,`confidence='quoted'`。
- 入口统一走 `remember` 工具(Nimbus D5 Lane B 已交付,Flux 已验绿),**不得旁路写文件**。

**这条通道让 O1/O2 立刻开始生效**,不必等 168 MB 洗完。

## 7. 丢失分析(回答「方案是否会丢有价值信息」—— 会,且已定位)

| 环节 | 丢什么 | 是否可接受 |
|---|---|---|
| S1 脱敏 | 密钥**值**(保留指纹) | ✅ 故意,必须 |
| **S2 切错** | A session 的决策以 B session 的失败为理由 → 切开后成两条**无关**记录 | ⚠️ **这正是 G2 关系列的理由**;且 §8 专项验收 |
| **S2.5 剪枝** | tool_result 正文(= 证据本身) | ⚠️ **仅靠 L2 指针补救** |
| S3/S4 | LLM 转述必然漂移 | ⚠️ 唯一的锚是 L2 精度 |

**结论:所有丢失风险收敛到 L2 精度这一个点。** 没有行级锚点,上游每一个有损取舍都变成**不可逆**。
**所以 G5 不是锦上添花,是承重墙** —— 它同时兜住 S2.5 和 S3/S4 两处损失。

## 8. 试点与验收

1. 先洗 **codex + claude-code 的 14 个 session**(cursor 走 chat_dump.json 次之;gemini 本轮只登记)。
2. **S2 专项验收(门禁)**:切出的 topic 清单交张先生**肉眼判断「是不是一个个独立的工作主题」**。
   不过关则停,不得继续。
3. **S1 专项验收(门禁)**:已知含密钥样本必须命中;漏报=缺陷。
4. **S2.5 证据**:提交剪枝统计(原始行数 → 入 LLM 行数)。
5. **抽样人读 L0/L1**:判断「是否真抓住要点」而非复述;检查 `confidence` 标注是否诚实。
6. **L2 回溯抽验**:随机取 5 条记录,按 `l2_file_path`+行号区间**真的打开原文**,确认断言有据。
7. 全过 → Flux 技术验证 → 张先生看样本 → 本作用域完成。扩大范围另行决策。

## 9. 归因(诚实记账 —— 半年后别把「自己推的」当「业界标准」)

**仅对 OpenViking 做了联网核实**(官方文档 + 仓库页 + 第三方评测);Letta/Graphiti/Cognee/ReMe/memsearch
**未读源码**,用的是公开描述 + 训练印象。star 数不引用(搜到 OpenViking 有 16.5k/21.7k/30k 三个版本,
均来自不同时间的软文)。

| 设计点 | 来源 | 可信度 |
|---|---|---|
| L0/L1/L2 渐进披露 | OpenViking(`.abstract.md`/`.overview.md`) | 高,读过文档 |
| 返回带父上下文的子树 | OpenViking「目录递进检索」 | 高 |
| session → 抽取记忆 | OpenViking「commit 会话 → 抽成可审阅 Markdown」 | 高 |
| 召回留痕 | OpenViking「可视化检索轨迹」+ 映射到已有事件账本 | 混合 |
| 记忆操作即 Agent 工具 | Letta / MemGPT 核心主张 | 中,未读码 |
| ECL 流水线形状 | Cognee | 低,只取名字所述形状 |
| 文件是真相、索引可重建 | ReMe / memsearch | 低,**趋同多于借鉴** |
| 双时态纪律 | Graphiti (Zep) | 低,`canonical_events` 已有 occurred_at+created_at |

**我们自己的(非借鉴):** project+topic 作为唯一作用域轴(张先生的框架,**故意背离 OpenViking 的
agent/user 轴**);强制脱敏先于 LLM(张先生的红线);**六类 typed record**(无外部出处 —— 来自手册里那个
空勾选框 + 2026-10-10 当天 failure 全部复发的实证);用 `pg_trgm` 而非向量库(**故意不跟**);
**S2 切段启发式**(Aegis 自构,**最弱一环**);**S2.5 剪枝 + G1–G5 五缺口**(来自张先生 v2 四问)。

## 10. 召回(两条路;**不做向量化** —— 张先生 2026-10-10 明确排除)

1. **结构召回(默认)** — 沿 `projects` 树 + `topics.parent_topic_id`,返回**带父上下文的 L0 子树**。零新基建。
2. **词法召回** — 对 `l0_summary`+`l1_overview` 建 `pg_trgm`/`tsvector`。**`pg_trgm` 1.6 已装在 live**。

两者均**默认过滤 `status='forgotten'` 与已被 supersede 的记录**(G2)。每次 recall 记一条 `canonical_event`。

**明确不做** pgvector/embedding。实测依据:live 是 `postgres:16-alpine`,`vector` 扩展不可用;官方 pgvector
镜像是 Debian(glibc),与 Alpine(musl)**collation 不同,跨 libc 迁移需 REINDEX 文本索引**;且**当前 6 个
provider 全是 chat/completion,无确认的 embedding 端点**。schema 保持可增量加列(将来真要加走 019)。

## 11. 红线

- 🔴 **脱敏先于 LLM、先于落库**。漏报按缺陷处理。
- 🔴 **schema 加固(018)完成前不得开始 S3 抽取** —— 否则灌进装不下价值的容器,补完得重洗。
- 🔴 **不得产生第二个权威源**:产物一律落 SeatLoom;`raw/` 只读,不改不删。
- 🔴 **不引入外部记忆系统代码**(只吸收模式)—— 基座里套基座自相矛盾。
- 🟡 **凑不出 L2 行号的记录不得入库**(G5 是承重墙)。
- 🟡 **先试点后扩大**;扩到其余 17 个 project 需张先生另行决策。

## 12. 前置条件

1. ✅ **provider 就绪并冒烟验证**:`stepplan-wash`(openai-completions)/`stepplan-wash-anthropic`
   (anthropic-messages),`step-5-preview` + `step-3.7-flash`。
2. ⬜ **015 必须在 live 库**(当前 track 最高 014,live 无 `memories` 表)—— 走验收流程,Aegis 执行。
3. ⬜ **018 schema 加固完成**(§4,G1–G5)—— Onyx,**硬前置**。
4. ⬜ **`seat-vela-001` 注册进 `seats` 表** —— `dispatches.target_seat_id` 有 FK 到 `seats(id)`,
   未注册则派单插不进去。Aegis 动作(席位注册完整性是 AEGIS.md 的 Owns 条目)。
5. 模型分工建议:S2/S2.5/初筛用 `step-3.7-flash`(吞吐);`decision`/`failure`/`unresolved` 三类高价值用
   `step-5-preview`。预算近无上限,全用 `step-5-preview` 亦可,代价只是墙上时间。
