# 夜间头条拆解 — 在界面里看到从历史会话提炼的知识(Aegis 夜间计划执行)

**Owner:** Lyra (PO) · **Date:** 2026-10-11(夜间)· **来源:** Aegis 夜间交代(人类通道转达)+
`AEGIS-2026-10-11-overnight-plan.md` + `AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md`
(两者在主的 checkout,未提交 track —— Aegis 的 commit 被权限拦着;本拆解引用其 id)。
**状态:** 拆解完成,7 个 dispatch 已派(含 2 个 gated 自派单),4 份验收方案已供稿。

---

## 0. 他要醒来看到的那一件事

**SeatLoom 界面里,从他自己的历史会话提炼出的知识**:项目 → 主题树;点主题 → 记忆卡;
默认一句话摘要,展开给详述,再展开给**指回原文的行号**。满足三样:全新的功能(界面上
多一块从没见过的东西)/ 更丰富的数据结构(主题树 + 六类记忆 + 三层披露 + 行级证据,
全在 project+topic 作用域下)/ 更扎实(顺带修好验收卡,他能点一次"验收")。

## 1. 开局实测(拆解依据)

- track tip `85bcc06`:015–018 已在 track(memory tools 集成 + waker 修复 + containment);
  live 库已 apply 015–018(张先生跑过 apply-schema-to-live.sh)。
- **step 1 卡点仍在**:`packet/nimbus/open-topic` 分支已建(tip 仍是 a04aedf,无 commit)——
  Aegis 已派单、Nimbus 已接单,**不重复派**;本拆解只补其验收方案与外围 lane。
- `open_topic` 未上 track;topics 表现 0 行;**每一次 remember 此刻仍会外键失败**。
- Vela:Part 2 已交付(e72a250,88 条 typed record  staged)+ Flux PASS with findings
  (3669476);S5 是 staged dry-runnable 驱动 + checkpoint,等 live 路径。
- 坏卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY`(base='track' 不存在)仍 pending,请求方是 Mira。
- **我的席位进程仍是旧二进制**(MCP 只暴露 6 工具,无 query_ledger/get_dispatch/recall)——
  Aegis 裁决里「二进制已新,席位进程仍旧」对我同样成立;本轮我不依赖新工具,全部经
  派单 + 文档驱动。

## 2. 拆解与派单(顺序即 Aegis 定的;前一步不落地后一步不开工)

| # | Lane | 席位 | Dispatch | 状态/门 |
|---|---|---|---|---|
| 1 | open_topic + 判定 id 契约 | Nimbus | (Aegis 已派,进行中) | 不碰;只补方案 |
| 1b | open_topic 验收方案供稿 | Nimbus | `disp-onight-ot-plan`(p15) | ready(排在其 lane 后) |
| 2 | 全队换二进制 | Aegis | (其自醒巡检,step 1 验绿后;Vela 除外) | 非我车道 |
| 2b | **真实往返冒烟 + 发门** | Nimbus | `disp-onight-smoke`(p45) | ready(排在 open_topic 后) |
| 3 | 真的写进库(S5 真跑) | Vela | `disp-onight-writein`(p35) | **blocked → `ev-typed-memory-tools-landed`** |
| 4 | 召回接到界面能调的地方 | Nimbus | `disp-onight-readmodel`(p40) | ready |
| 5 | 界面知识面板 + 作废坏卡 | Mira | `disp-onight-panel`(p40) | ready |
| 6a | read-model 建卡 | 我(自派单) | `disp-lyra-onight-card-readmodel` | blocked → `ev-verdict-knowledge-readmodel` |
| 6b | **头条卡**建卡 | 我(自派单) | `disp-lyra-onight-card-headline` | blocked → `ev-verdict-knowledge-panel` **+** `ev-typed-memory-tools-landed` |

**关键机制决策(承接两份 Aegis 文档):**

1. **判定 id 预声明 = 契约**:step 4/5 的派单强制要求验单产出预声明判定事件 id
   (`ev-verdict-knowledge-readmodel` / `ev-verdict-knowledge-panel`),我持 gated 自派单
   在该 id 上 —— 判定落地即唤醒我派建卡步,"验完→建卡"无人推。Aegis 裁决二升格了
   我为 Part 2 做的变通;open_topic lane 的判定 id 由 Nimbus 按契约自持(其建卡用其
   自派单,方案路径经 1b 送达)。
2. **门的条件是 live 可用,不是代码合并**:`ev-typed-memory-tools-landed` 只在
   **一次真实 remember 成功之后**由冒烟 lane(2b)发出,带证据(topic id / memory id /
   recall 返回)。绝不在代码落地时发。Vela 的写库(3)blocked 在该门上 —— 门未发,
   一条记录都不许入库。
3. **并行活文件集不相交,双边派单写明围栏**(Aegis 两条硬要求之二):step 4(Nimbus)与
   step 5(Mira)会同时动数据传输结构。围栏见 §3,在两边派单里逐字相同。
4. **坏卡作废 = Mira 的动作**(cancel_acceptance 只能由请求席位撤自己的 pending),
   折进 step 5 派单的第一动作,理由必填。

## 3. 文件围栏(step 4 与 step 5 同文,逐字)

```
NIMBUS(step 4)可碰:
  crates/seatloom-core/src/db/**            (knowledge 读函数 + lineage 模块,增量)
  crates/seatloom-mcp-emit/src/**           (两个 MCP 读工具包装,tools/list 15->17)
  src-tauri/src/commands/knowledge_cmds.rs  (新文件)
  src-tauri/src/dto.rs                      (只增:新 Knowledge* DTO,禁改既有 struct)
  src-tauri/src/main.rs                     (只增:一行 command 注册)
  ui/src/lib/types-dto.ts                   (只增:新 Knowledge* wire types)
  ui/src/lib/api.ts                         (只增:新 knowledge* invoke 包装)
MIRA(step 5)可碰:
  ui/src/app-v2/dashboard/KnowledgeSection.tsx     (新文件)
  ui/src/app-v2/dashboard/ProjectDashboard.tsx     (只增:挂载一行,禁重排版既有 section)
  ui/src/app-v2/dashboard/KnowledgeSection.test.tsx (新文件,按既有 section 测试模式)
  ui/src/app-v2/styles/tokens.css                  (只增,优先复用既有 token)
共享文件规则(硬):
  - types-dto.ts / api.ts / dto.rs 归 Nimbus 围栏;MIRA 永不编辑 —— 需要变更经
    emit_event 上报 Lyra 等派单,不自行改。
  - MIRA 永不碰 crates/**、src-tauri/**;NIMBUS 永不碰 ui/src/app-v2/**。
  - 检查按文件粒度:碰一个本来就脏的文件会让它变红 —— 守在围栏内,交付"零新增问题"
    的实测结果,绝不顺手重排版(Aegis 硬要求)。
```

**DTO 契约(两边派单同文,Mira 据此先行,集成在 read-model 落地后):**

```ts
export interface KnowledgeTopicNode {
  topicId: string; title: string; status: string;      // open|closed|merged (010 vocab)
  parentTopicId: string | null; childTopicIds: string[];
  memoryCount: number;                                  // active 且未被 supersede 的记忆数
}
export interface KnowledgeMemoryCard {
  memoryId: string; topicId: string; title: string;
  kind: string;            // decision|failure|fact|preference|artifact|unresolved
  confidence: string;      // quoted|inferred
  tier: string; status: string;   // cold|warm|hot ; active|forgotten|resolved
  l0Summary: string;       // ~100 token,唯一默认层
  l1Overview?: string | null;
  l2?: { refKind: string; documentId?: string|null; filePath?: string|null;
         bodyDigest?: string|null; lineStart?: number|null; lineEnd?: number|null } | null;
  supersededBy?: string | null;   // includeSuperseded 时标注取代者
  sourceKind: string; sourceRef: string;
}
export interface KnowledgeTopicTree { projectId: string; topics: KnowledgeTopicNode[]; }
export interface KnowledgeTopicMemories {
  topicId: string; topicChain: {topicId:string; title:string}[]; depth: string;
  memories: KnowledgeMemoryCard[];
}
// Tauri: cmd_get_knowledge_tree(project_id) / cmd_get_topic_memories(topic_id, depth='l0', includeSuperseded=false)
// MCP(同 repo 函数薄包装): get_knowledge_tree / get_topic_memories
```

## 4. 验收标准(门禁,非主观;四条验收方案已供稿)

| Lane | 方案文档 | 一句话门槛 |
|---|---|---|
| open_topic | `…-nimbus-open-topic-acceptance-plan-v1.md` | 契约全条可演:幂等/服务端 opened_by/四类 typed 拒绝/环路拒绝/行+事件同事务 |
| 冒烟(无卡) | (证据=门事件本身) | 一次真实 open_topic→remember→recall 往返成功,门事件带证据落账本 |
| read-model | `…-nimbus-knowledge-readmodel-acceptance-plan-v1.md` | 树+按主题取卡+三层按需+纯读;DTO 与契约逐字一致 |
| 写库 | `…-vela-writein-acceptance-plan-v1.md` | 真实提炼记忆在库 + 剪枝/脱敏/丢弃三项统计 + 他读得懂的交付说明 |
| 面板/头条 | `…-mira-knowledge-panel-headline-acceptance-plan-v1.md` | 主题树+记忆卡+三层披露+行级证据在界面可见;坏卡已作废 |

**夜间必须有(低于即失败)**:① 主题工具上线且有**一次真实成功记忆写入**;② 库里有**真实
提炼**的记忆(非测试数据);③ 一份他读得懂的交付说明(提炼多少/丢多少/为什么丢)。
**可以没有(但须说清差什么)**:面板完整形态(主题树+摘要列表即算);头条卡重建(坏卡作废
即算合格)。
**绝对不许**:无真实成功就发门;跳脱敏(漏报=缺陷);凑不出行号的记录入库;扩洗数据范围。

## 5. 边界

- step 2(重启)、live apply、张先生接口 = Aegis;我不重启任何席位、不写 live。
- open_topic 的实现与判定 id 契约 = Nimbus(Aegis 已派);若其选择**新增** TopicOpened
  事件类型(裁决要求其二选一并书面说明代价),则 PG 值域加宽需 Onyx 迁移 + 我的契约签署
  —— 已在 1b 派单里写明该分支路径,Nimbus 必须先上报再动 schema 侧。
- 事件驱动零轮询;阻塞或需张先生判断 → emit_event + 上报 Aegis。

## 6. 证据索引

- Aegis 计划/裁决(主 checkout 未提交 track):`AEGIS-2026-10-11-overnight-plan.md`、
  `AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md`。
- 四方案:§4 表。派单 payload 自含,packet 路径 = 各 lane 文档 + 本拆解。

## Follow-up(2026-10-11 凌晨):门溯源未定 → 头条卡改绑正向证据;019 方案缺口已补

**事实(按时间)**:Nimbus open_topic 交付(选新增 TopicOpened,Rust 钉 28)→ Lyra 签署契约
+ 派 Onyx 019 → Nimbus 冒烟**如实上报 blocked**(未假写、未发门)→ Onyx 019 落地 + Flux
PASS(其卡因「Onyx 进程旧 + Lyra 019 方案未派」而 blocked)—— **019 方案是 Lyra 的缺口,
已补**(\`2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1.md\` + 建卡步
\`disp-019-onyx-card\`)→ Mira 面板 Flux PASS(诚实的不接线态)。

**门溯源结论(截至本记录)**:没有任何席位在 live 上真实 remember 过(smoke blocked、
Vela 未写),故 \`ev-typed-memory-tools-landed\` 的发出者与证据**仍未认定**。按 v1 自派单
自带的守卫(绝不在假门上建头条卡):**未派头条卡**;v1 自派单按「等待+改绑」分支完成,
新派 \`disp-lyra-onight-card-headline-v2\`,门改为两个**正向证据**:
\`ev-knowledge-read-model-landed\`(读模型交付)+ \`ev-writein-landed\`(Vela 真实记录入库)。
头条卡只在「已交付、有证据」的现实上触发。已 SupervisorMessage(\`ev-Zf318ktgqy5z8vTv8gT8R\`)
请 Aegis:(a) 认定门事件发出者与证据(其 commit 被权限拦着、本席二进制读不了账本,
请用派单等席位可见的方式确认);(b) 019 的 live apply 与全队重启仍是 Vela 写库的关键路径。
