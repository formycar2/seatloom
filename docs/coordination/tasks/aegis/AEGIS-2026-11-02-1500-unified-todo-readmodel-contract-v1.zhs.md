# 统一待办读模型：接口、验收用例与四席位派单契约

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-1500-unified-todo-readmodel-contract-delivery |
| status | delivered |
| author | aegis |
| date | 2026-11-03 |
| version | v1 |
| depends_on | DISPATCH-2026-11-02-1500-unified-todo-readmodel-contract-v1.zhs.md |
| tags | architecture, contract, todo, snapshot, verification |

## 1. 目标与范围

落实架构评审的第一项：现有总控面板中的“需要我处理”用共享后端读模型，计数、列表和通知不再各自判断。
本契约不是实现交付，不是产品验收通过，也没有创建账本派单。文件名保留派单指定的 11-02，实际交付日期是 11-03。

输入：
- `DISPATCH-2026-11-02-1500-unified-todo-readmodel-contract-v1.zhs.md` 全文。
- `AEGIS-2026-11-02-dsh-architecture-review-delivery-v1.zhs.md` 全文。
- 当前静态代码基线 `4529979285a3b8537913dd6852cb6557f8a8b921`；已有模型 `db/models.rs::OperatorDecisionRow/AcceptanceRow/PromptInstanceRow` 与 schema 005/020。

**不加 stage 列、不改 status、不建表、不跑迁移、不重建已丢失的记录。** 本轮只写文件。
后续实现测试只能使用明确隔离的临时数据库/内存及测试 doubles，不能向 live 库塞探针或样本。
保留现有 React/Tauri 组件、人工决定与人工 Accept；查询成功不代表获得执行权限。

### 本轮第一项纳入什么

| 来源 | 纳入条件 | 分类 / 责任人 |
|---|---|---|
| operator_decisions | project 匹配、status=pending、decider 精确匹配可信操作者 | decision / 记录里的 decider |
| acceptances | project 匹配、status=pending，操作者为当前配置的人类操作者 | acceptance / 当前配置操作者；不是派单执行席位 |
| prompt_instances | project 匹配、status=active、policy 是 needs_approval 或 human_required | prompt / 当前项目的配置操作者；这是本版治理路由，不是表中已有 assignee |

prompt 表没有独立 assignee。本版不得猜负责的席位或从正文解析“谁批准”。若当前配置不能确定操作者，整个查询拒绝，不显示可执行空名单。若现有 prompt 动作入口不支持某操作，显示“需在原会话处理”及会话链接，不制造新批准能力。

**排除** auto_allowed prompt、终态对象、发给席位的 decision、其他项目/操作者的 decision。
依赖缺失、in_flight、失败事件和自动重试本身不是人工 obligation；仍由既有工作/系统状态区域展示，不扩大本列表。
不把全局不存在的历史记录算作“已处理”；空库成功读到 0 只写“尚无待办记录”。

## 2. 查询接口（拟新增，不是已注册工具）

后端共享服务建议名 `query_project_obligations`，Tauri 适配名 `cmd_list_project_obligations`。
MCP/通知若接入只能用同一服务和分类规则，不另造计数 SQL；名称是后续实现契约。

### 2.1 输入

```text
Request {
  project_id: nonblank string,                 // 必填，无默认项目
  kinds?: set<decision|acceptance|prompt>,      // 缺省全部；空集合拒绝，去重规范化
  page_size?: integer,                        // 缺省20，范围1..100
  cursor?: opaque string                      // 首次必须缺省；后续使用返回值
}
Context { authenticated_actor_ref: string }   // 宿主可信上下文，不是模型/前端传参
```

两者都参与查询：项目来自明确选择，操作者来自受信任入口。第一版只服务当前配置人类操作者。
missing/blank project → InvalidProject；未知项目 → ProjectNotFound；操作者缺失/非当前操作者 → OperatorContextUnavailable/Forbidden。
不能添加默认 human:supervisor、回退别的项目，不能因为名字以 human: 开头就认为是本人。
本版复用现有可信身份解析与保护入口，不声称解决共享 OS 凭据的安全隔离问题。

### 2.2 响应与 obligation 的强制字段

```text
Response {
  schema_version: 1,
  scope: { project_id, actor_ref, kinds },
  snapshot: { id, captured_at, expires_at },
  total: integer >= 0,
  counts: { decision, acceptance, prompt },
  items: Obligation[],
  next_cursor: opaque string | null,
  diagnostics: Diagnostic[]
}
Obligation {
  key: { kind, id },                           // 复合身份，不能只用id/标题
  project_id,
  actor_ref,                                  // 欠谁操作，不是创建者
  actor_basis: recorded_decider|configured_operator,
  source: { table, id, status, event_ref?: string },
  title,
  pending_since: RFC3339 timestamp,
  required_action: decision|review|human_response,
  subject_refs: [{ kind, id, basis }],
  details: discriminated union,
  action: { target_kind, target_id, mode: navigate|invoke_existing,
            enabled: boolean, disabled_reason: string|null }
}
```

- decision details 必须含原 target、choices、consequence、evidence；不得把摘要改成新选择。
- acceptance details 必须含 dispatch_id、branch、base、verdict_ref、acceptance_plan（原样），保持现有验收详情入口。没有计划/证据时仍计入待验收，给明确缺失说明；Accept 能否执行仍由原门即时判定。
- prompt details 必须含 session_id、prompt_kind、policy、available_actions、evidence_ref/preview 的可空语义；不把未知填成空字符串或虚构操作。
- pending_since：decision.created_at / acceptance.created_at / prompt.detected_at；时间取 DB 值。
- 排序固定为 pending_since 升序、kind 按 decision/acceptance/prompt 顺序、id 按字节序；不得按标题归并。
- 相同 kind/id 经多行 join 仍只有一个 obligation；不同 kind 的相同 id 必须保留两个。
- source event_ref 可不存在，不能为了填字段补造事件。诊断缺失证据与真正不存在的来源对象要区分。
- unknown kind/schema 必须明确报版本不支持，不能静默漏掉新类别。

### 2.3 关联规则

项目级联合列表不要求每个对象都有派单关联。

1. acceptance.dispatch_id 是结构化关联，且关联派单必须存在、同项目；不成立则该 acceptance 仍显示，subject_refs 不输出假关联，diagnostics 给 BrokenSubjectReference，动作跳到验收自身的错误详情。
2. prompt.session_id 同样只在会话存在且同项目时关联；不得从会话最近的 dispatch 猜当前工作。
3. decision.target/evidence 是文本/字符串集合，**本版不从中提取 dispatch**，subject_refs=[]。即使 target 恰好等于派单标题或看似包含 id，也不因此认作结构化关联。
4. 不采用“主题相似”“同席位”“最新一条”“只有一件活”兜底。
5. 同一派单有待决定与待验收可以有两个 obligation；不做互斥 stage 去掉其中一个。

这是刻意小的第一版，不新增 JSON 关联协议。以后支持 decision 的结构化关联须单独定来源契约。

## 3. 快照与分页（强制，不允许先数后查两个实时结果）

### 3.1 第一页构造

在**同一个 PostgreSQL REPEATABLE READ、READ ONLY 事务**中读取三类对象及所需关联事实，形成完整匹配结果 R。
允许一个 UNION 查询或同一事务多查询；禁止已有各列表接口的 limit=200/500 截断后再拼接。
先按复合 key 去重、分类、关联校验、排序，形成不可变 R；此后：

```text
total = |R|
counts[k] = R 中 kind=k 的数量
sum(counts) = total
items = R[0:page_size]
```

同一次响应必须携带 counts 和 items，不允许前端另请求 cmd_count_* 来填同一个 badge。
counts 受输入 kinds 过滤；未选类别计数为0。diagnostics 不计入 total。
transaction snapshot 是数据库一致性，snapshot.id 是应用服务生成的不可猜标识；captured_at/expires_at 是时间元数据，**时间戳本身不是快照身份**。

### 3.2 后续页面

本版采用服务进程内**有限寿命不可变结果快照**，不建表、不持久化派单状态、不长期保持数据库事务：

- 首次事务完成后缓存 R 和 scope/page_size，TTL 固定60秒，最多32个快照、总缓存最多32 MiB（按序列化结果字节计，不含程序开销）。
- 任一完整 R 超过预算或容量已满时，拒绝 SnapshotCapacityExceeded；不截断、不赶走仍有效快照、不返看似完整的部分数据。
- 每次访问先清理过期快照；进程重启丢失缓存，返回 SnapshotExpired，不把旧 cursor 当第一页。
- cursor 绑定 snapshot.id、offset、原 scope、page_size 与可信 actor。可服务端随机 token 映射，不允许客户端篡改 offset/actor 后访问。
- 后续页只切已缓存 R，**不得重新实时查询**，counts/total/captured_at 不变；末页 next_cursor=null。
- 一页一页取完应无重复无遗漏，并集复合 key 数量等于 total。同 cursor 重放在 TTL 内返回同一页。
- cursor 与新 project/kinds/page_size 或当前 actor 不符 → CursorScopeMismatch/Forbidden；过期 → SnapshotExpired。不可悄悄换新快照。
- UI 过期时明确重新读取第一页，丢弃旧页集合。显示的是“截至 captured_at 的待办”；不能把缓存宣传为实时状态。

缓存是临时只读结果，不是新事实源。源对象改变后新首请求产生新快照；执行动作时**必须即时重验来源及权限**，旧 snapshot 不能授权。

### 3.3 UI 与通知一致性

UI 同一次响应的 items/total/counts 同步发布到当前 project/actor；切项目时拒绝旧异步响应。
只展示当前页时显示“共N项，当前页M项”，不写“待办M项”。若已有卡计数展示另一范围，必须标范围，不能共用这个 badge 名称。

通知消费者也调用共享服务，分页只能沿同一快照枚举。完成枚举才形成全量摘要；某页失败/过期时不发布“没有待办”。通知只按复合 key 去重，不按标题；读模型不发送消息。
通知与 UI **不同时间刷新可以不同**，以 snapshot.id/captured_at 标明；不要求两个独立请求永远相等，也禁止把新通知数字配旧 UI 列表。通知点击触发新首请求并定位 kind/id；对象已处理时显示“已处理”，不重建它。

TTL 60秒与通知重提间隔是不同规则，不修改现有6小时重提策略。查询/缓存不新增后台自动执行或外发副作用。

### 3.4 错误与状态

任一来源查询失败：整次请求失败 SourceReadFailed，带来源分类与可诊断错误（不泄露凭据），不返回total=0。
不作“先显示成功两类、漏掉失败一类”的部分成功。异常对象关联可以 diagnostics 展示，但来源表查询失败不是关联诊断。
纯读调用不得插事件、改状态、更新 read_count、生成派单或记忆。缓存/快照 ID 属内存读过程，不是账本写入。

## 4. 可执行验收用例目录

**所有用例均为未来测试要求，本轮未运行。** 每条需调用被测服务/真实组件并断言结果；禁止用 include_str/grep 自己的断言字符串当行为测试。

统一夹具说明：P=`test-todo-p`、Q=`test-todo-q`；H=`human:todo-owner`（测试宿主配置操作者），J=`human:other`；S=`seat:todo-worker`。
时间 t0=`2026-11-01T00:00:00Z`，逐条增加1秒。测试 DB 使用真实 schema 与枚举，所有必需父行、合法 CHECK 字段由夹具 builder 补齐并显式设置；下表不是可直接粘进 live 的 SQL。
除各例增补外，主夹具 F 是：

- D1=P,pending,decider=H,target=“重复标题”,choices=[“保留”,“取消”],consequence非空,created_by=H,created_at=t0。
- D2=P,pending,decider=J；D3=Q,pending,decider=H；D4=P,resolved,decider=H（chosen/resolved_by/resolved_at完整）；D5=P,pending,decider=S。
- W1=P,done 的派单；A1=P,pending,dispatch_id=W1,created_at=t0+1，合法plan/branch/base/verdict fixture。
- A2=Q,pending；A3=P,accepted（合法终态字段）。
- Session1=P；T1=P,active,needs_approval,session_id=Session1,detected_at=t0+2；T2=P,active,auto_allowed；T3=Q,active,human_required；T4=P,resolved；T5=P,active,human_required,detected_at=t0+3。
- F 对 P/H 的正确 R=[decision:D1,acceptance:A1,prompt:T1,prompt:T5]，total=4，counts={1,1,2}。

### 4.1 原评审 §5 第1条：计数、隔离与分页拆成独立断言

| 用例 | 输入/构造 | 独立断言 | 必须失败的实现 / 变异 |
|---|---|---|---|
| TODO-01 完整集合 | F，P/H，page_size=100 | key 集合精确等于4个预声明key，total=4 | 漏prompt、混入D2/D3/D4/D5、仅断言非空 |
| TODO-02 计数 | F 同请求 | counts精确{1,1,2}，sum=4=total | 独立宽范围COUNT、按当前页算总数 |
| TODO-03 项目隔离 | 将D3/A2/T3标题改为与P相同，查询P再Q | P无任何Q key；Q结果也不含P，不能只检查一个方向 | 去掉project谓词或按同名合并 |
| TODO-04 操作者隔离 | F；可信H查询P；可信J与配置H不符再查询 | H只含D1不含D2/D5；J请求Forbidden，参数传H不能绕过 | 只看human前缀/客户端指定actor |
| TODO-05 分页总数 | F,page_size=2 | 首页items=2,total=4,counts不缩水,next非空 | total=items.length |
| TODO-06 超过旧上限 | P下创建505个独立pending decision，其他类空，page_size=100 | 首total=505；6页长度100/100/100/100/100/5，并集505、无重复 | 先调旧limit=500或200列表再count |
| TODO-07 排序平局 | 3类同timestamp，分别多个id，乱序插入；page_size=1 | 顺序按约定kind/id完整相等，cursor重放相同 | 不稳定ORDER BY、仅按时间排序 |
| TODO-08 kinds过滤 | F,kinds=[prompt] | total=2,counts={0,0,2},items仅T1/T5 | counts忽略过滤；空kinds被当all |

### 4.2 原第2条：终态与多个义务

| 用例 | 输入/构造 | 期望 | 必须失败的实现 |
|---|---|---|---|
| TODO-09 done还欠验收 | F的W1=done，A1=pending | A1仍出现 | JOIN要求dispatch=in_flight |
| TODO-10 裁定已解决 | 快照R后合法resolve D1，再新首请求 | 新R不含D1,total=3；旧R按快照保持4 | 缓存当永久事实、终态仍显示pending |
| TODO-11 两件不能合并 | D1.target=W1标题，同时A1.dispatch_id=W1 | 两个不同key均可打开自身详情；D1无dispatch关联 | 按标题/工作聚合后只剩一张 |
| TODO-12 跨类同id | 合法构造decision与prompt相同id | 两key保留，计数各+1 | 单独id当React key或去重键 |

### 4.3 原第3/4条：未知、关联与非人工状态

| 用例 | 输入/构造 | 期望 | 必须失败的实现 |
|---|---|---|---|
| TODO-13 自动恢复非人待办 | 另造blocked W2缺依赖、in_flight W3无进展观测、自动重试状态fixture；T2=auto_allowed | obligation集合仍精确F的4个；工作区W3不能渲染“正在运行” | 根据in_flight/关键词自动造人工obligation |
| TODO-14 未关联decision | D1.target含W1标题和id，evidence含看似dispatch串 | D1.subject_refs=[]，action.target_id=D1 | 文本提取/同席位/最近一单兜底 |
| TODO-15 关联范围 | A1有合法W1/P，另构造A_bad/P指向W_Q/Q（schema允许时用真实行） | A1 refs含W1；A_bad仍显示但无W_Q ref，诊断BrokenSubjectReference | 不核project就显示跨项目关联 |
| TODO-16 孤立关联 | 用纯组装器fixture让prompt会话/acceptance派单缺失（不破坏测试DB FK） | 不补造父行；保留原待办+诊断，无假subject | 遇孤立对象悄悄丢卡或猜一个父对象 |

### 4.4 快照、竞态与纯读

| 用例 | 输入/构造 | 期望 | 必须失败的实现 / 故障注入 |
|---|---|---|---|
| TODO-17 事务内竞态 | 两独立连接：读事务快照建立后暂停，在另一事务resolve D1并insert D6，再恢复第一事务；page_size=100 | 第一响应整个R仍为F；新快照移除D1加入D6；所有counts/items同版本 | READ COMMITTED多查询/不同连接count+list。同步屏障在快照建立后，不能靠sleep赌时序 |
| TODO-18 跨页竞态 | F首page_size=2；随后resolve T1、添加更早D0；再取旧cursor末页 | 旧全部4项无重复；总数/快照id/时间不变；新首请求出现新集合 | 后页重新query live或offset实时分页 |
| TODO-19 错cursor | F首cursor；Q/kinds变更/size变更/可信actor变更/随机token逐一提交 | 明确ScopeMismatch/Forbidden/InvalidCursor，不返别的项目页 | cursor只有可改offset或未绑定身份 |
| TODO-20 过期/重启 | fake clock推进61秒或清缓存模拟重启 | SnapshotExpired；UI旧页丢弃后重取首页 | 无声重建新快照并接旧分页 |
| TODO-21 容量 | fake缓存填满32个仍有效快照；第33个请求；或单结果序列化>32MiB | CapacityExceeded，既有cursor仍有效，不部分返回 | 驱逐有效快照、静默截断 |
| TODO-22 来源失败 | 分别让三类读取在一次请求中抛错；至少一类已成功 | SourceReadFailed，无成功counts/items；UI错误区别于“尚无记录” | Promise.catch(()=>[])、先显示两个成功来源 |
| TODO-23 真空结果 | 合法空测试项目，三类查询成功 | total=0,counts全0,items=[],next=null；无故障诊断 | 空与读取失败同文案 |
| TODO-24 纯读 | 临时DB中调用首页/后页，前后比canonical_events及所有源行的数量和字段摘要；mock命令/外发端口记录调用 | 数据完全不变，外发/写/领单/resolve调用次数0 | 读时记MemoryRecalled/补事件/更新状态或自动恢复工作 |

### 4.5 真实组件消费与旧动作

| 用例 | 输入/构造 | 期望 | 必须失败的实现 |
|---|---|---|---|
| TODO-25 数字配列表 | 真实既有待办面板注入F首页2项/total4 | badge=4，当前页=2；单一响应更新；不能调用独立count接口 | badge取items.length或旧全局计数 |
| TODO-26 切项目竞态 | 延迟P响应，先切Q且Q先返回，再释放P | 屏幕只显示Q scope与Q列表/数字 | 无project/actor/request-generation guard |
| TODO-27 原动作与缺数据 | 点击D1/A1/T1，缺计划A_legacy也加入 | 精确打开自身详情；decision原choices不改，Accept不自动执行；A_legacy计入总数并显示计划缺失 | 模型生成新选择、默认批准、缺plan就过滤 |
| TODO-28 通知来源 | 同一完整快照4项，通知fake sink；旧独立q函数设为抛错 | 通知枚举4个复合key，摘要total4；无独立来源查询；缺页时报错而非“无待办” | 通知仍另写三套SQL，只枚举第一页 |
| TODO-29 陈旧动作 | 获取A1 pending快照后原动作已拒绝/接受，再点击旧页面按钮 | 原入口重验拒绝再次执行，UI刷新；查询不得直接执行merge | snapshot内容被当授权或重复Accept无重验 |

## 5. 原八条验收要求的范围归属

不是把八条全部塞给Nimbus；本轮实现只承担第一项，不偷带权限/恢复改造。

| 原条目 | 本版归属 |
|---|---|
| 1 count/完整列表/项目/分页 | TODO-01..08、17..21、25..26 |
| 2 done仍待验收/resolve移除/同时欠两事 | TODO-09..12、27 |
| 3 自动重试/未知执行/依赖不是欠人 | TODO-13；工作状态区既有行为的回归，不新增运行判定器 |
| 4 禁止猜关联 | TODO-14..16，Flux去掉关联校验做变异应杀死 |
| 5 授权伪造/扩大范围 | 本版入口actor隔离TODO-04/19，其他授权scope/policy/parent测试属于第二项，**延后且不算本版通过** |
| 6 停单/重试不洗拒绝/观察只读 | 本版纯读TODO-24；暂停/重试/观察状态机属于第二/四项，延后 |
| 7 外发回执/Git-DB恢复/重复点击 | 本版TODO-28使用fake sink、29保留原动作门；外发持久恢复与Git/DB故障注入属于第三项，延后；不借本单绕过现有已知风险 |
| 8 变异与证据 | 下列独立验证协议；全单适用 |

延后项未来另有用例契约与实现派单，本单不得以29个用例过了宣称八条跨系统能力全部完成。

## 6. 四张可直接转交的派单规格（此处代号不是账本id）

共同前提：后续派发时采用真实新派单，不补造昨天生命周期；实现不得自行合并。交付提交、独立验证、用户可感知验收沿用现有流程。若账本仍不可写，就保持文件任务；不能自动从本表伪造事件。

### 派单一：Lyra 确认人工待办的用户契约

- 输入：本文§1–5、现有总控面板/人工决定/验收交互契约。
- 输出：明确确认纳入三类、prompt路由到配置操作者、项目级列表及空/失败/过期文案；给现有组件中的展示位置与可点击验收步骤。只定契约，不画新交互。
- 依赖：无；**最先开工**。
- 验收：Aegis核是否越过本架构边界；Flux审用例能否观察失效。Lyra不能独自把自己的契约批准当实现验绿。
- 完成条件：逐项确认或列具体冲突，不写“总体同意”。冲突若改变接口分类/范围，回Aegis裁决后版本化，不让实现猜。

### 派单二：Nimbus 共享查询与快照分页

- 输入：本文§2–4、派单一确认的语义版本、真实schema。
- 输出：共享Rust服务、DTO和Tauri薄适配，测试夹具builder；无schema变更，所有消费者只复用此查询。
- 依赖：派单一语义确认；确认前可并行读现有接口、设计测试夹具，不写新的产品语义。
- 验收人：Flux独立技术验证；Lyra核数据是否表达用户契约。
- 必测：TODO-01..24、29服务端半部。提交真实测试入口命令（含integration tests，不能只cargo test --lib）。
- 完成条件：给固定提交、作用域、资源上限与错误路径证据；读失败不可报零、跨页数据来自同一快照、读操作零写。

### 派单三：Mira 现有总控面板与通知消费接线

- 输入：本文DTO/排序/分页/错误、派单一文案、派单二接口。
- 输出：复用现有面板/卡/分页或现有加载更多控件；计数与列表共用响应，旧动作入口保留；通知扫描适配共享服务（若通知脚本由Nimbus维护，Mira只定消费契约、Nimbus负责薄适配，不出现双份规则）。
- 依赖：派单一确认；**可与派单二并行**用本文固定DTO mock开发。接真实IPC/通知查询须等派单二固定提交。
- 验收人：Flux独立组件/端到端验证，Lyra用户可感知验收。
- 必测：TODO-25..29及20/22/23界面半部。mock不算真实接线验收。
- 完成条件：项目/操作者切换无旧响应覆盖，badge不是page size，失败不空白，按对象key定位且不自动批准；最终在隔离沙箱构建演示。

### 派单四：Flux 独立验绿与变异验证

- 输入：本用例清单、派单一确认、Nimbus/Mira各自固定提交、隔离数据库与沙箱构建。
- 输出：按用例逐条pass/fail/not-run、独立oracle、变异diff/位置/命令/退出码、用户可见结果证据；不是抄作者绿灯。
- 依赖：准备oracle/fixture审查可立即并行；最终验绿须派单二、三完成真实接线，并核正确分支与构建。
- 验收人：Lyra判用户验收条件，Aegis接跨席位阻塞；最终合并仍由操作者Accept。
- 完成条件：29个本版用例都有结果；延后项明确not-in-scope。至少独立移除project过滤、改total为页长、后页重新查实时、加入文本猜关联、源错返回[]、去掉UI切项目guard，六种变异各杀死相应测试。
- 多个实现单元各自需要实际实现派单id与verdict引用，不把一条无主体的“全绿”贴到不同卡。Flux不能给自己写的验证工作发实现验收结论。

```text
Lyra语义确认 ──┬── Nimbus服务 ──┐
               └── Mira mock ──┼── Mira真实接线 ── Flux最终验绿 ── Lyra用户验收 ── 人Accept
Flux oracle准备（独立、可立即做）┘
```

估算：仍以3–5个工作日历日为目标，前提是四方并行且仅第一项；约6–9人工工程日（Lyra0.5–1、Nimbus2–3、Mira1.5–2、Flux2–3），不含排队、授权等待、返工与已有合并恢复风险。若测试环境/快照缓存接线超出此范围，先报拆分，不削弱用例换交期。

## 7. 测试实施与交付纪律

- 每个测试用例都必须有精确输入key和独立期望，不能从实现输出再算一遍“正确结果”。
- fixture身份/项目/枚举合法；跨项目同名、超旧limit、空/失败双向、两连接竞态、fake clock是强制构造。
- 先在未变异的固定实现跑全套，然后逐个变异；先保存git diff证明变异在目标函数，再跑具体测试；恢复后全套重跑和diff干净。没有实际退出码就不记 killed。
- 技术报告逐例列测试文件/函数、命令、提交、结果。命令失败时保留stderr，不使用grep成功替代测试退出码。
- 沙箱视觉验收不是两条“环境检查都绿”；让操作者看到跨项目不同待办、总数大于页数、错误不是空、已处理项移除。构建来自卡上固定提交并有来源标识，不能拿主应用猜运行行为。
- 现有项目有0条待办/缺历史记录时不得灌假数据伪装上线。演示夹具留在隔离沙箱，正式界面明确无记录。

## 8. 本轮完成记录

已交付接口、同快照强制分页、29个独立用例、原八条范围映射、四张派单与并行依赖。
本轮没有实现或运行这些行为测试，没有改代码/库/迁移，没有调用enqueue/remember/emit_event/complete_dispatch。
遵从本派单“只写文件、不进账本”的明确要求，不补造昨天的派单行。
验证仅为文档检查与内容核对，不冒称实现验绿。提交只包含本交付文件；已存在未跟踪的 `scripts/bind-seatloom-seats.sh` 保持不动。

最容易做错的一点：**第一页返回正确total并不等于分页一致；若后页重新读取live，pending变更会制造漏项/重复，badge仍看起来很准确。** TODO-17/18必须以两连接屏障和跨页变更验证，不接受只对静态夹具分页的绿灯。
