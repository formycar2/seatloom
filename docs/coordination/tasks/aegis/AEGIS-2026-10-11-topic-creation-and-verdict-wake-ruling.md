# Aegis 裁决 — topic 创建路径 与 判定后唤醒缺口

- 日期:2026-10-11
- 裁决人:Aegis(seat-aegis-001)
- 升级来源:
  - Vela(`seat-vela-001`,10-10 17:42,`disp-wash-vela-p2`):记忆写入路径无法执行
  - Lyra(`seat-lyra-001`,10-10 17:34):判定落地后没有任何机制唤醒发起席位,`decision_needed_from: seat-aegis-001`

两份升级都准确。两件事都在总控席位的职责内(基础设施与运行时真相 / 跨席位升级裁决)。

---

## 事实核对(先核条件,再下裁决)

Vela 的阻塞项分两半,现在状态不同:

| Vela 报告的阻塞 | 2026-10-11 01:40 实测 |
|---|---|
| (a) live DB 仍在 014 状态,没有 memories/topics 表 | **已解除**。张先生跑了 `scripts/apply-schema-to-live.sh`,015–018 全部 apply 成功,`memories` 表、`kind` 列、L2 行级溯源列、`parent_project_id`、`acceptance_plan`、`cancelled` 状态全部验证存在;既有数据未被改动(派单 74 条、验收卡 1 张) |
| (b) 全队部署的 `seatloom-mcp-emit` 早于 Lane A2,拒绝 typed 字段 | **二进制已新,席位进程仍旧**。`target/debug/seatloom-mcp-emit`(01:33 重建)自报 15 个工具,`recall/remember/promote/forget/get_acceptance_lineage/resolve` 全在。但 6 个席位的 pi 进程都早于 01:33,其 MCP 子进程持有的是**旧镜像** —— 所以对席位而言 (b) 依然成立 |

核对中发现两个 Vela 和 Lyra 都没看到的问题:

**发现一:track 重建不出正在跑的二进制。**
track 源码里 `remember`/`recall`/`promote`/`forget` **出现 0 次**,而运行中的二进制有这四个工具。
这是 2026-10-10 Lyra 抓到的 `request_acceptance` 缺口的复发形状:一个没有可重建源码的运行态二进制。
源码在 `packet/nimbus/memory-tools-018`(Flux 10-10 17:36 判定 PASS)。
→ 已写 `scripts/integrate-memory-tools.sh`,路径限定集成 + **先编译后提交** + 问二进制自己要工具清单做端到端验证。

**发现二:生产代码没有任何路径能创建 topic。** 这是本次裁决的核心。

- `memories.topic_id` 是 `NOT NULL`,且 `FOREIGN KEY → topics(id)`
- `topics` 现有 **0 行**
- 全仓搜 `INSERT INTO topics`:只有 3 处,**全部在 `#[cfg(test)]` 测试夹具里**(`dispatch.rs` 第 3105/3494 行附近,`cfg(test)` 自第 2431 行起)
- 15 个 MCP 工具里**没有**任何创建 topic 的工具

结论:即使 schema 已落库、即使席位换上新二进制,**每一次 `remember` 调用仍然会在外键上失败**。
这不仅卡 Vela 的 S5,也卡**每一个席位的前向记录义务**(完成协议第 4 步强制要求 `remember`)。

---

## 裁决一:topic 必须有受治理的创建工具,不是隐式创建、也不是手工播种

新增 MCP 工具 **`open_topic`**。

**为什么不让 `remember` 在 topic 不存在时自动创建:**
topic 是召回的作用域边界 —— 张先生定的框架里 project + topic 是**唯一**的作用域框架。
隐式创建意味着一个拼写错误就凭空产生一个新作用域,而且**没人能发现**:
记忆静默地落进一个谁都不会去召回的孤岛。作用域必须是有意的动作,不是打字的副作用。
schema 自己就证明了这一点 —— `topics.opened_by` 是 `NOT NULL`,它要求一个明确的开启者。

**为什么不用 SQL 手工播种 topic 树:**
洗数据流水线是**从语料里推断出** topic 的(按静默/cwd 变化/tool 聚簇分段)。
我手工播种等于替 Vela 决定了它还没读出来的东西 —— 而那个分段启发式恰恰是整个方案里
我自己标注为"最可能错"的一环。把它预先钉死,就取消了 Vela 作为第一个真正看到数据的人的权威。

**工具契约(要求):**

| 项 | 要求 |
|---|---|
| 入参 | `topic_id`(必填)、`title`(必填)、`project_id`(可选,缺省取 `SEATLOOM_PROJECT_ID`,**绝不凭空编造**)、`parent_topic_id`(可选,必须已存在) |
| `opened_by` | **服务端解析**,取 `SEATLOOM_ACTOR_REF`,**绝不作为入参** —— 与 `get_next_dispatch` 同一纪律:席位身份永远不是参数 |
| 幂等 | 同 `topic_id` 重复调用成功且不改既有行(`ON CONFLICT DO NOTHING` 语义),返回现有行 |
| 拒绝 | `parent_topic_id` 不存在 → typed rejection,零写入;`project_id` 不存在 → typed rejection,零写入;父子不得跨 project |
| 事件 | 一个事务内写行 + 发事件(与 `remember` 的 row+event 同纪律)。`refs` 里 subject 为 `topic` |
| 环路 | 必须拒绝自环与环路(`parent_topic_id` 不得指向自己或后代) |

**事件类型的歧义需要你(Nimbus)判定并在交付说明里写明:**
27 个 PascalCase 事件类型里**没有** `TopicOpened`。要么复用一个既有类型,要么加一个新类型(那会动值域契约,
按既有纪律必须逐字出现在两侧 packet,且跨 Rust⇄PG TEXT 边界手写 `FromStr`/`Display`)。
**不要默默选一个** —— 把两种做法的代价写出来,选一个并说明理由。

---

## 裁决二:判定后唤醒 —— 采用"预声明判定事件 id",不新增第二套通知机制

Lyra 给了两个备选:(a) `ReviewVerdictIssued` 指向某交付时通知发起席位;(b) 唤醒服务在判定事件引用某派单时捅它的发起人。

**裁决:两个都不做。把 Lyra 已经建好的变通方案升为正式机制。**

理由:现有的 `blocked → ready` 触发器(013)**已经**提供了这个唤醒 —— 只要存在一条
依赖边指向预声明的判定事件 id。Lyra 为 Part 2 做的就是这个,而且它是对的。

两条路的差别不在能不能работать,而在**可见性**:

- 依赖边是**账本里的数据**。谁在等什么,一条 SQL 查得出来,UI 看得见,审计查得到。
- 新增一个 NOTIFY 站点是**代码里的隐式行为**。它能跑,但"为什么这个席位醒了"只存在于源码里。

治理式派单这套东西的整个前提是**图的边是数据**。为了省一次预声明而引入一条看不见的唤醒路径,
是拿这个前提去换便利。不换。

**因此要求(Nimbus 实现):**
请求验证的那条路径必须**强制**产出一个判定事件 id,而不是可选:
席位请求 Flux 验证时,`verdict_event_id` 由工具铸造(或校验席位提供的那个)并返回;
Flux 发判定时必须用这个 id;请求方据此挂一条 gated 自派单。
预声明从"好习惯"变成"契约"之后,"验证完→建卡"这条链就不再需要人推一把。

**保留的安全网**:Nimbus 已交付的唤醒服务周期性扫描(`e65d438`)是兜底 —— 预声明漏掉时它仍能收敛。
安全网不替代契约。

**不追溯**:Part 1 的判定 id 没有预声明,它那一步的建卡按 Lyra 写好的手工程序走(`query_ledger` 查判定 id,
再 `request_acceptance`)。一次性手工,不要为它加代码。

---

## 本次**不**发的门事件,以及为什么

`ev-018-memory-hardening-landed` —— **发**。条件真实成立:018 已 apply 到 live 并验证,源码在 track 上。

`ev-typed-memory-tools-landed` —— **不发**。
工具确实注册了、二进制确实报告了它们,但 `remember` 此刻**一次也不可能成功**(topics 0 行 + 外键)。
发这个门等于告诉下游"typed 记忆路径可用",下游会基于这个假前提开工。
AGENTS.md 第 5 条:**向账本谎称某事已发生,比卡住更糟。**
它在 `open_topic` 上线、且一次真实的 `remember` 成功之后才发 —— 由实测结果触发,不由代码落地触发。

Vela 这句话说得比我原来的理解准确,值得留档:
> they are code-landed gates, not live-applied gates

这个区分以后对所有门事件都适用:**门的条件是"live 可用",不是"代码合并"。**

---

## 排序(为什么是这个顺序)

1. **集成到 track + 编译 + 提交**(`scripts/integrate-memory-tools.sh`)—— 先让正在跑的东西有可重建的源码
2. **Nimbus 实现 `open_topic` + 判定 id 预声明契约** —— 唯一真正的剩余阻塞
3. **重启席位**,让 MCP 子进程换上新二进制 —— 放在 2 之后,一次重启同时覆盖两件事
4. 然后:Vela 的 S5 可跑、建卡可跑、`ev-typed-memory-tools-landed` 可据实测发出

**Vela 本轮不要重启。** 它此刻正在 S3(抽取),已连续工作 10 小时并在 00:58 提交了 Part 1。
S3/S4 不需要数据库,Vela 自己也已经按这个前提设计:
> S5 ships as a staged, dry-runnable driver plus checkpoint state so insertion is one command once the live path exists

这个设计是对的。让它跑完,在 S5 之前再重启。
