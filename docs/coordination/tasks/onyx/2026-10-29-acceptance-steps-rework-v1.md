# 返工说明:我的 4 张验收卡的验证步骤里,有四条让张先生敲 SQL —— 其中一条在往账本里写

| Field | Value |
|---|---|
| id | ONYX-2026-10-29-acceptance-steps-rework-v1 |
| template | T7 |
| subtype | implementation-delivery (rework note — 卡本身未能原地修改,原因见 §4) |
| status | delivered |
| author | seat-onyx-001 |
| date | 2026-10-29 |
| version | v1 |
| tags | acceptance, how-to-verify, rework, verification, honesty |
| responds to | 派单 `disp-TRxqo8RDIyg1ycBj36NXb`(Aegis 审计我的卡发现的问题) |

## 结论先行

**Aegis 说得对,而且最严重的那条比它说的还要糟。**

它指出 4 张卡的 `acceptance_plan.steps` 里有 shell 命令。我复核之后,最严重的是
`acc-disp-accplan-lane-c-onyx`:它的第 4、5 步是**两条 `INSERT INTO acceptances`** ——
让张先生往**正在验收的那张表**里写两行假卡。Aegis 说的是"写它正在验证的表",我复核后
补一句它没说的:**这两行会长在账本里,任何之后数 pending 卡的人都会把它们当真卡数进去**,
包括 Aegis 自己的 `check-cards-clickable.sh`(它读的就是 `acceptances where status='pending'`)。

这不是风格问题,这是污染账本。

## 1. 我怎么发现自己有这个问题(以及为什么我的自审没发现)

Aegis 在 10-29 早上跑它的 `check-card-verifications.sh`,报的是 `shell-based=0`。
我自己也跑过一次,同样是 0,而且我当时还逐张自审过,写了"我的卡里没有 shell 命令"。

**那句话我说错了。** 我说的是 `how_to_verify`(卡面那一行),确实没有 shell;
但 shell 在 `acceptance_plan.steps` 里,而我只看了 `how_to_verify`。
Aegis 的检查器也有两个 bug(它自己承认了):只读 `how_to_verify` 不读 plan,
以及 join 里的换行把行切碎了。所以**它的工具报 0,我的自审也报 0,两边都错了**。

它靠的是"自己去读卡",不是靠工具。这条我记下了:**工具报 0 不是证据,尤其当那个工具
刚刚被发现有两个 bug 的时候。** 我这次的错误是拿一个坏工具的输出给自己的结论背书。

## 2. 四张卡逐张的问题(我量的)

| 卡 | 问题 | 严重度 |
|---|---|---|
| `acc-disp-019-onyx` | 第 4 步是可选的只读 `select count(*) from topics` | 低,但该删 |
| `acc-disp-bc20UzOznJQ8gCgbHw876` | 第 4 步是可选的只读 `select count(*) from seats` | 低,但该删 |
| `acc-disp-d5-lane-a-onyx` | 第 1-5 步全是 `apply-db.sh` 两遍 + 4 条 psql 内省 | 高(整张卡是 shell) |
| `acc-disp-accplan-lane-c-onyx` | 第 1-5 步同上,**外加 2 条 `INSERT INTO acceptances`** | **最高(写账本)** |

派单给的两条理由我都认:

1. **张先生的验收是"用户可感知的验收"。** 一条 psql 内省结果不是他能感知的东西。
   他跑一条命令去确认一条命令跑过 —— 那是席位的活,不是他的活。
2. **验证步骤写它正在验证的表,不是读,是写。** 而且写进去的行看起来像真卡。

## 3. 改好的验证步骤(按派单要求:UI 步骤放前面,DB 证据留在交付说明里)

### 3.1 `acc-disp-accplan-lane-c-onyx`(016 验收方案载体)

**该看的界面**:派单页,点开任一张待验收卡。

| # | 动作 | 预期 |
|---|---|---|
| 1 | 打开 SeatLoom 派单页,点开任一张**待验收**卡 | 卡面上能看到**结构化的验收方案**:场景、步骤、预期,不是只有一句话 |
| 2 | 看这张卡自己的方案 | 场景写的是「张先生确认……」这种他能看懂的话,不是席位黑话 |
| 3 | 看步骤区 | 每一步有「做什么」和「看什么」,并且**第一步是界面里的动作**(打开某个页面/点某个按钮),不是跑命令 |
| 4 | 如果 016 没生效(方案不存在) | 卡面只剩一行 `how_to_verify`,没有场景/步骤/预期 —— 那就是失败,别点接受 |

**为什么这样改**:016 这件的意义就是"卡面能承载结构化方案"。所以验证它就该看卡面。
原先那 5 条 psql(含 2 条 INSERT)是**实现细节**,属于交付说明,不属于验收步骤。

### 3.2 `acc-disp-d5-lane-a-onyx`(015 知识层)

**该看的界面**:知识面板。

| # | 动作 | 预期 |
|---|---|---|
| 1 | 打开 SeatLoom,进知识面板 | 能看到**主题树**—— 左侧/顶部是主题列表,不是空白、不是报错 |
| 2 | 点开任一主题 | 能看到该主题下的**记忆卡片**,每张卡有标题和一行摘要 |
| 3 | 点开任一记忆卡片 | 能看到**三层**:先一行摘要(L0),展开有概述(L1),再展开有出处指针(L2,文件名+行号)。**不是一屏摊开全文** |
| 4 | 看 L2 那层 | 给的是文件路径和行号,指向真实文件 —— 而不是把正文抄一份在库里 |
| 5 | 如果 015 没生效 | 知识面板空的/报错,或卡片只有一层 —— 那就是失败,别点接受 |

**为什么这样改**:015 这件的意义是"渐进披露的知识层落在库里并且界面上读得到"。
第 3 步的"三层,不是一屏摊开"正是它区别于"一列塞全文"的地方,而这个区别**只有在界面上才看得出来** —— 在 psql 里看列名是看不出来的。

### 3.3 `acc-disp-019-onyx` 和 `acc-disp-bc20UzOznJQ8gCgbHw876`

删掉那条可选的命令行(它们本来就是 UI 步骤 + 一条"可选"的 psql)。UI 步骤已经够了:
019 是"督导面板点读取主题,看到 10-28 那三个主题,自己再开一个";
种子 005 是"席位列表数 10 个,点开 vela 看能力标签"。

## 4. 我改不了这 4 张卡(结构性,不是我不做)

派单让我"原地更新 acceptance_plan,如果工具允许"。**工具不允许,而且是设计如此:**

```
crates/seatloom-mcp-emit/src/dispatch.rs:178
    /// §2, tool layer): a missing, empty or malformed plan is a typed
    /// rejection, so "no plan -> no card" cannot be bypassed by a caller that
    /// skips the check. Authored by the PO; immutable after the request, same
    /// rule as `how_to_verify`.
```

我把 5 个 acceptance 相关的工具都列了一遍:`request_acceptance` / `cancel_acceptance` /
`get_acceptance_lineage`,**没有任何一个能改一张已存在卡的 plan**。

派单同时明确说"不要撤回你自己的卡来做这件事"。所以我按它给的第三条路走:
**开这份返工说明,把正确的步骤写在这里,让 Aegis 接手。**

**给 Aegis 的具体动作**(它说"如果两者都不行,告诉我,我来扛这个发现"):
这 4 张卡的 plan 需要重写。路径有两条,都要它定:

1. **cancel + 用同一个 id 重开**。工具文档明说这被允许("Supply it to re-open a card you
   cancelled under the SAME id")。但 `cancel_acceptance` 是终局、要理由,而且卡历史里会留下
   "requested → cancelled → requested" 的痕迹 —— 操作者会看到我撤回了自己的卡。
2. **改 `request_acceptance` 或加一个 `amend_acceptance_plan`**。让席位在卡还是 pending 时
   能改自己的 plan(最好限定"只能改 steps/expected,不能改 scenario/preconditions",
   且每次改动留一条事件)。这是根治,但那是 Nimbus 的活,不是我的。

**我倾向 2**,因为 1 会让"改错步骤"这件事在操作者眼里长得像"撤验收"。
但这是我的看法,决定是 Aegis 的。

## 5. 附带发现:两个席位的 capability_tags 是空的

Aegis 在同一次审计里发现的,我复核确认:

| 席位 | capability_tags | 来源 |
|---|---|---|
| `seat-ql-001` | `{ql,delivery,huawei-storage}` | adopt-ql.sh 声明 |
| `seat-vela-001` | `{corpus-ingest,extraction,redaction,data-washing}` | bind-vela 时声明 |
| `seat-networkorchestrator-001` | `{}` | adopt 脚本写的就是空数组 |
| `seat-stepaicloud-001` | `{}` | 同上 |

它问我要不要声明、还是要说明为什么空。**我的答复:保持空,并且说明。**

理由:

- 这两个项目是**外部项目**(`project_delivery/networkOrchestrator`、
  `project_delivery/stepaicloud`),接入脚本是 Aegis 为了把别人的会话纳管写的,
  当时**没有**给它们定义能力域。
- 我**没有证据**说它们会什么。凭项目名猜 `{network,orchestration}` 或
  `{cloud,deploy}` 就是编数据 —— 而种子这一整件事的意义就是"种子说得准"。
  给种子注水,比留空更糟。
- 空数组在界面上是可表达的: SeatLoom 的席位卡可以显示"未声明能力"。
  **要改的是界面怎么显示空,不是种子怎么填。**

所以这条我不改种子 005。它写进 §6 的报告里,作为给 Aegis 的待决项。

## 6. 边界

- 没有撤回任何卡 ✅(派单明确禁止;而且我也不能原地改)
- 没有动 live 库 ✅(全程只读)
- 没有改种子 005 的能力标签 ✅(§5 说明了为什么)
- 三个分支已同步 ✅(`disp-BjfvrSm4iEtMIFSPG6OPn` 那笔之后 track 又前进一笔
  `5206674`,已 merge,三个分支 behind=0、ancestor=YES、已推两个 remote)

## 7. 需要 Aegis 做的两件事

1. **重写这 4 张卡的 plan**(§3 已经写好了,可以直接用)。路径二选一,见 §4。
2. **决定 capability_tags 怎么办**(§5)。我的建议是界面显示"未声明能力",种子不动。
