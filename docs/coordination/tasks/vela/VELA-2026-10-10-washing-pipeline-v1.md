# VELA-2026-10-10-washing-pipeline-v1 — Lane C:D5 Phase 2 清洗流水线(seatloom 16 sessions)

**Owner:** seat-vela-001(待 Aegis 注册)· **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** Part 1 = `disp-wash-vela-p1`(S0–S2.5,零 LLM,注册后即派);Part 2 =
`disp-wash-vela-p2`(S3–S5,**两个门事件都落地后才派**:`ev-018-memory-hardening-landed` +
`ev-typed-memory-tools-landed`)
**Base branch:** `track/infra-foundation` · **Delivery branch:** `packet/vela/washing-pipeline`
**设计权威:** `docs/coordination/tasks/aegis/AEGIS-2026-10-10-washing-pipeline-design-v2.md`
(Aegis 分支;v2 supersedes v1,全部改动来自张先生四问:目的是什么 / token 值不值 /
schema 完善否 / 会不会丢信息)。**值域契约:** `VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md`
(六类记录词表、confidence、l2 行号 —— S3 产出的每条记录都受它约束)。

---

## 0. 作用域(张先生明确收紧,红线)

**只做 seatloom 一个 project**:16 sessions / 168 MB / 58,015 行 / 2026-04-24→10-08
(占全量 1.2%)。按 harness:codex 6(134MB)、claude-code 8(34MB)、cursor 1、gemini 1。
扩到其余 17 个 project **不是自动的下一步**,需张先生另行决定 —— 不要自作主张扩域。

**硬顺序红线:Lane A(018 schema 加固)完成前,不得开始 S3 LLM 抽取。** Part 2 的两个门
事件即此保证;门未到而 S3 已跑 = 违规(往装不下价值的容器里灌 token,补完重洗 = 重花一次)。

## 1. Part 1 — S0/S1/S2/S2.5(零 LLM,注册后即可开始)

### S0 归一化(零 LLM)→ canonical turn 流
- 输出规范:`{seq, ts, role, kind, text, tool, refs}` 一行一 turn,落中间产物(scratch 目录,
  非 git)。
- **本作用域 97% 内容在 codex + claude-code** → 只需 2 个必做 adapter;cursor 读目录内
  **已存在的 chat_dump.json**(不必解 sqlite);gemini 本轮**只登记不抽取**(记一行 meta:
  harness=gemini, session id, 路径),不得因它阻塞主线。
- adapter 必须对未知行**容错**:记为 meta 并计数,**不得整文件失败**;每个 adapter 输出
  「总行数 / 已解析 / meta 行数」统计。

### S1 脱敏(强制,在 LLM 之前、落库之前)
- 命中模式:`ak-*` / `sk-*` / `ghp_*` / `AKIA*` / JWT / PEM / Bearer / 65 字符 stepplan key /
  cookie / 内网域名 + 高熵启发。
- 替换为 `⟪REDACTED:类型:指纹⟫`,指纹 = sha256 前 6 位(同值同指纹,保去重可辨不可逆)。
- 门禁②:在**已知含密钥样本**上必须命中;漏报 = 缺陷,停线修 pattern,不带病前进。

### S2 切 topic ⚠️ 本方案最弱一环(Aegis 自构,无外部验证)
- 信号:静默 >30min / cwd 变化 / 任务边界 / tool-call 聚簇。
- 切错则 topic 树是垃圾、所有召回建在沙上 —— 所以门禁①:topic 清单交张先生**肉眼判断
  「是不是一个个独立工作主题」,不过关则停**(不停 = 全链路返工)。
- 产出:topic 树( parent→child,落 topics 前是中间产物;入库存 S5)。

### S2.5 剪枝(token 杠杆)
- 送 LLM 前丢弃:tool_result **正文**(留 tool 名 + 成败 + 字节数 + **行号区间**)、
  重试轮次、meta 行。预期砍掉 80–90% 体量。
- 必须提交剪枝统计(原始行数 → 入 LLM 行数,按 session)。
- **红线:剪枝只影响送给 LLM 的内容,不影响 L2 指针**(靠 G5 行号区间回溯原文)。

## 2. Part 2 — S3/S4/S5(两门事件落地后才可开始)

### S3 抽取(唯一 LLM 段)
- 产出**六类 typed record**(= G1 词表 decision|failure|fact|preference|artifact|
  unresolved),每条必须带 `confidence`(quoted|inferred)与 **L2 行号区间**;
  **凑不出行号的记录不得入库**(丢证据的记录 = 丢信息,张先生四问之四)。
- `failure` 优先级最高(实证:今天写入 memory 的多数是 failure 且全部复发过)。
- 输出:typed record 集(中间产物),待人读抽样(门禁④)。

### S4 分层
- L0 一句话(唯一默认层)/ L1 够决策 / L2 = 路径 + digest + **行号区间**。
- **绝不拷贝原文入库**(脱敏后也不拷 —— 记忆是指针+提炼,不是第二个原文库)。

### S5 入库(幂等)
- 斜杠路径 → projects 树;**深层环检测是本流水线责任**(015 只挡 1-环,深环靠你在入库前
  跑环检测,发现即停并上报,不得静默跳过)。
- 段 → topics;记录 → memories。
- **写入只走 seatloom-core 的记忆写路径(remember 工具同一条:行 + 事件同事务),
  不得 raw SQL、不得旁路文件。**
- 幂等:按内容摘要去重;按 session sha256 记 checkpoint(中断续跑不重复入库)。

## 3. 验收门禁(门禁,不是主观满意度)

整体目标三条可检验结果:**O1** 派单 payload 不再重复内联已有事实,改为 recall 引用;
**O2** 已记录的 failure 不再同根因复发;**O3** 张先生不再是系统最后一道记忆。

逐项门禁(全过才谈 Flux):
1. **S2 topic 清单交张先生肉眼判断「是不是一个个独立工作主题」,不过关则停。**
2. S1 在已知含密钥样本上必须命中;漏报 = 缺陷。
3. 提交 S2.5 剪枝统计(原始行数 → 入 LLM 行数)。
4. 抽样人读 L0/L1:判断是否真抓住要点而非复述;检查 confidence 标注是否诚实。
5. **L2 回溯抽验:随机 5 条,按路径+行号真的打开原文确认有据。**

全过 → Flux 技术验证 → Lyra 出验收方案(场景/步骤/入参/出参/预期)→ Aegis 呈张先生。

## 4. 收尾流程(两 Part 各自)

1. 提交到 `packet/vela/washing-pipeline`(Part 1 / Part 2 分别 commit,带各自 dispatch id)。
2. Part 1 交付说明:`docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-p1-delivery-v1.md`
   (S0 统计、S1 命中证明、S2 topic 树、S2.5 剪枝统计)。
3. Part 2 交付说明:`…-p2-delivery-v1.md`(S3 record 集统计与 confidence 分布、S4 分层
   样例、S5 入库幂等证明、门禁①–⑤ 证据)。
4. 各自 `complete_dispatch`;自请 Flux(commit-pinned);验绿后 `request_acceptance`
   (base=track/infra-foundation;acceptance_plan 由 Lyra 供稿,逐字入参)。
5. **席位永不自己合并。**

## 5. 红线

- 作用域只 seatloom;扩域须张先生决定。硬顺序:S3 前必有 018 + typed tools 两门事件。
- 不引入外部记忆系统代码;不拷贝原文入库;剪枝不伤 L2 指针;幂等 + checkpoint。
- 事件驱动零轮询零查库(get_dispatch/query_ledger/recall 已够用);不整分支合并。
- 阻塞或需张先生判断(如门禁①不过)→ emit_event + 上报 Aegis,不自行放行。
