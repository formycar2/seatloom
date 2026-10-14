# Delivery — disp-onight-panel(夜间 step 5+6:知识面板 + 围栏 + 预声明判定 id)

**Seat:** Mira (seat-mira-001) · **Dispatch:** `disp-onight-panel` · **Date:** 2026-10-11(夜间)
**Packet:** `docs/coordination/tasks/lyra/LYRA-2026-10-11-overnight-headline.md` §3/§4/§5
**Branch:** `packet/mira/knowledge-panel` · **Base:** `track/infra-foundation`(`85bcc06`)

---

## 0. 一句话

界面里多了一块从没见过的东西:**项目 → 主题树;点主题 → 记忆卡;每张卡默认一句话摘要,展开给详述,
再展开给指回原文的行号**。三层披露与 D5 的 L0/L1/L2 完全同一套,没有第四种。

---

## 1. 做了什么

| 文件 | 改动 |
|---|---|
| `ui/src/app-v2/dashboard/KnowledgeSection.tsx` | **新增**。主题树 + 记忆卡 + 三层披露 + 行级证据 + 有意义空态 + 已注册门禁态 |
| `ui/src/app-v2/dashboard/ProjectDashboard.tsx` | **只增**:1 行 import + 1 行挂载(见 §3 的实测) |
| `ui/src/app-v2/dashboard/KnowledgeSection.test.tsx` | **新增**。13 个断言组 |

**没动** `crates/**`、`src-tauri/**`、`lib/types-dto.ts`、`lib/api.ts`、`tokens.css`
(面板复用既有 `--sl-*` token,无需新增)。

### 面板形态

- **主题树**:根由 `parentTopicId` 定(null 或指向不在树里的节点),子序由服务端的
  `childTopicIds` 声明定(服务端次序优先,数组次序只作兜底);每个节点带 `status` 与
  `memoryCount`(active 且未被 supersede 的数)。
- **点主题** → 取该主题的记忆,并显示 `topicChain`(ancestors-first),记忆不脱离上下文读。
- **三层披露,严格三档**:
  - **L0(默认)**:title + kind 徽章 + confidence + tier + l0 一句话摘要;
  - **L1(展开一次)**:l1 详述;卡上没有 l1 时**显式说明**,不用假详述顶替;
  - **L2(再展开一次)**:refKind + documentId/filePath + bodyDigest + lineStart/lineEnd,
    外加来源(sourceKind/sourceRef)与被取代关系;第三击收回 L0,**不造第四档**。
- **l2 按需取**:L0 只拉摘要列表,l2 在展开到第二层时才取(`depth='l2'`)。
- **零轮询**:挂载后取一次主题树,其余按点击取,没有 setInterval。

### 已注册门禁(house rule)

`KNOWLEDGE_CMDS_REGISTERED = false`,并已实证:`track/infra-foundation` @ `85bcc06` 的
`src-tauri/src/main.rs` **没有**注册 `cmd_get_knowledge_tree` / `cmd_get_topic_memories`,
`src-tauri/src/commands/knowledge_cmds.rs` **尚不存在** —— Nimbus 的 read-model lane(step 4)
并行进行中,未落地。因此面板把「命令未上线」作为主状态渲染,并点名缺的两条命令,
**不发一次调用、不显示一个假值**。他的只增补落地后翻一个常量即生效,不需要再改 UI。

### 本地 invoke 缝(为什么在组件文件里)

规范入口是 `lib/api.ts`,但该文件在 Nimbus 的文件围栏内,本席位不许编辑。所以缝放在
`KnowledgeSection.tsx` 内并明确标注:它调的是**同一条** Tauri 命令、**同一个** repo 函数、
**同一个** MCP 读工具 —— 是一个缝,不是第二个实现。Nimbus 的只增补
(`getKnowledgeTree` / `getTopicMemories`)落地后整块换成 `api.*` 即可。

DTO 契约按 Lyra 拆解 §3 **逐字镜像**在本文件顶部(`KnowledgeTopicNode` /
`KnowledgeMemoryCard` / `KnowledgeTopicTree` / `KnowledgeTopicMemories`),与 Nimbus 的
read-model lane 同文,因此两边不会漂。

---

## 2. 门禁(全绿)

| 门禁 | 命令 | 结果 |
|---|---|---|
| 类型 | `pnpm typecheck`(`tsc --noEmit`) | ✅ 干净 |
| 单测 | `pnpm test` | ✅ **59 passed / 5 files**(基线 46,新增 13,零失败零跳过) |
| 构建 | `pnpm build`(`vite build`) | ✅ 干净(仅既有 >500kB chunk 提示) |
| Lint | `eslint`,仅本单新增/触碰文件 | ✅ 两个新文件 **0 error 0 warning** |
| 格式 | `prettier --check`,同三个文件 | ✅ 全部符合 |

### 围栏合规:实测零新增问题

派单与 Aegis 都要求「碰一个本来就脏的文件会让它变红 —— 守在围栏内,交付**零新增问题**的
实测结果,绝不顺手重排版」。`ProjectDashboard.tsx` 在基线上就是脏的,所以逐条量过:

```
baseline(track/infra-foundation)ProjectDashboard.tsx: 33 issues
mine(加了两行之后)             ProjectDashboard.tsx: 33 issues
rule+message 多重集比对: 完全相同
NEW issues introduced: NONE
issues removed:       none
```

**33 → 33,多重集逐条相同,新增 0。** `lint-changed.sh` 仍以非零退出(因为该文件基线即脏),
这是预期的;本单没有为了「让它变绿」而去重排版别人的代码 —— 那正是派单禁止的。

---

## 3. ProjectDashboard 的改动确实只是「只增」

```
$ git diff --stat ui/src/app-v2/dashboard/ProjectDashboard.tsx
 ui/src/app-v2/dashboard/ProjectDashboard.tsx | 5 +++++
 1 file changed, 5 insertions(+)

$ git diff ui/src/app-v2/dashboard/ProjectDashboard.tsx
@@ import 区 @@
+import { KnowledgeSection } from './KnowledgeSection';
@@ 渲染区 @@
       <GoalsSection goals={goals} currentGoal={currentGoal} stages={stages} />
+
+      {/* 7. ── Knowledge (D5: memories distilled from historical sessions) ──
+          Additive mount only: one line, no re-layout of the sections above. */}
+      <KnowledgeSection projectId={projectId ?? channelId} />
```

**5 行新增,0 删除 0 修改**:1 行 import + 4 行注释与挂载。既有 6 个 section 的顺序、样式、
props 一概未动;挂载点在最后一个既有 section 之后、全局 hover 弹窗之前。主界面骨架原样继承
(`feedback_v1_main_interface_reference` 未被触碰)。

---

## 4. 测试覆盖(13 组,按既有 section 测试模式)

| # | 断言组 | 覆盖的派单要求 |
|---|---|---|
| 1 | 命令未接线:显式说明、**零调用**、**零假数据** | (5) registered-flag 模式 |
| 2 | 主题树渲染 memoryCount,点击取该主题记忆 + 祖先链 | (1) 主题树 |
| 3 | 子序按 `childTopicIds` 声明,不按数组序 | (1) 主题树 |
| 4 | kind/confidence/tier/status 映射 + l0 是默认层 | (2) 默认层 |
| 5 | 未知 kind/confidence/tier/status **逐字显示**,不改名不丢;已知 kind 仍翻译 | (2) 不发明 |
| 6 | L0 → L1 → L2 → 收回 L0,**严格三档**,l2 按需取 | (2) 三层披露 |
| 7 | l2 缺行号/缺一端时的四种诚实说法 | (2) 行级证据 |
| 8 | 卡上没有 l1、没有 l2 时显式说明,不留白 | (4) 不留空 |
| 9 | l2 声明 document 却没记 documentId → 明说 | (2) 行级证据 |
| 10 | superseded 记忆标注取代者 | (1) 记忆卡 |
| 11 | 无主题项目的有意义空态(说清为什么空 + 接下来发生什么) | (4) 有意义空态 |
| 12 | 有主题无记忆的空态,与「无主题」措辞不同 | (4) 有意义空态 |
| 13 | 读取拒绝时原样展示,不是空面板 | (5) 不用 mock |

测试在模块边界 mock api 层(与 `AcceptanceSection.test.tsx` 同一手法),断言落在**实际发出的
命令与参数**上(`cmd_get_knowledge_tree` / `cmd_get_topic_memories` 的 `depth`/`includeSuperseded`),
不落在内部实现上。

---

## 5. 需上报(按派单第 6/9/10 条)

### 5.1 坏卡仍未作废(step 6 前半被 live 库挡住)

按派单第一动作,我用 `cancel_acceptance` 撤 `acc-disp-me6Nm8e9D6ZRe4fKyVokY`(理由用派单建议
原文)。**vocabulary 已经修好了**(017 落地,`acceptances_status_vocab` 现含 `cancelled`),
但被 **016 的 plan CHECK** 挡住,原样:

```
dispatch rejected: db error | ERROR: new row for relation "acceptances" violates check
constraint "acceptances_plan_required"
DETAIL: Failing row contains (..., cancelled, base recorded as nonexistent track; ..., {})
| sqlstate=23514 constraint=Some("acceptances_plan_required")
```

只读实证:`acceptances_plan_required` 是 `CHECK (acceptance_plan <> '{}') NOT VALID`;
遗留行的 `acceptance_plan` 就是 `{}`。NOT VALID 只豁免**既有行的首次校验**,任何 UPDATE 都会
重新检查 —— 所以撤卡先要有人给这行填上真 plan。

**这不是本席位能做的**:回填是 Aegis 车道的 live-DB 动作,Lyra 的亲笔方案早在
`docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md` §2 等着。
按 AGENTS.md「守住本职边界」,我只记录 + 上报,没有自行 `ALTER`。

**当前风险可控**:该卡 base 记为 `track`(不是分支),`cmd_accept_delivery` 在 checkout 处直接
拒绝,走不到合并,所以「整分支合并删掉 track 上 8 个文件」的场景已经不可能。剩下只是一张挂在
运营者待验收列表里的过期卡。

**顺带一个正向观察**:`memories` 表现在**存在**了(015 已应用),`topics` 存在但 **0 行** ——
即 `open_topic` 尚未被真实调用过。这与 Lyra 拆解 §1 的实测一致。

### 5.2 二进制与 track 源码不同源(第三次观察到)

部署中的 `seatloom-mcp-emit`(02:13 重建)广告 15 个工具,含 `remember/recall/promote/forget/
resolve`;而 `track/infra-foundation` @ `85bcc06` 的 `crates/` 里没有这些实现。工具面超前于
track 源码与 live schema,建议 Aegis 明确该二进制的来源分支。

---

## 6. 边界确认

只做派单要求的事:建面板、只增挂载、按契约写测试、跑门禁、量零新增问题。围栏内四个文件只碰了
三个(`tokens.css` 无需改动 —— 全部复用既有 token)。未合并任何分支、未写 live 库、未重启任何
服务、未改任何 schema。发现的问题只记录 + 上报,未自行其是。

---

## 7. 判定 id 预声明(派单第 9 条,契约)

自请 Flux 验单时,验单**必须**要求产出预声明判定事件 id:

```
ev-verdict-knowledge-panel
```

Lyra 持 gated 自派单 `disp-lyra-onight-card-headline` 挂在该 id 上(与
`ev-typed-memory-tools-landed` 相与);判定落地即唤醒她派「头条卡」建卡步 ——
「验完→建卡」无人推。

---

## 8. 产物索引

- 实现 commit:见下。
- Dispatch:`disp-onight-panel`。
- 账本事件:`ev-knowledge-panel-landed`(门事件,ArtifactCreated)。
- 自请 Flux 验单:commit-pinned,要求预声明判定 id `ev-verdict-knowledge-panel`。
