# 独立核 Flux 的验单 `disp-VH6l1HixlUVYuy4m0NEp-`(审计三个门)

Mira 的审计(`ev-mira-audit-flux-VH6l1H02`)。被核对象:Flux 的 `disp-VH6l1HixlUVYuy4m0NEp-`,
它对三个被阻塞的门各发了一条 `ReviewVerdictIssued`。

规则来源:`docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`。

---

## 结论:**pass**

Flux 这个验单做得对。三条门 verdict 的承重声明我逐条重跑过,全部吻合。**其中两门(Gate A、Gate C)
是关于我自己或我邻席的活,我没有因此放宽标准。**

它做的三件事值得单独说:① 它明确声明不依赖 Aegis 的复测(因为 Aegis 已经误判过 Gate B);
② 它在 Gate A 拒绝代发契约事件,理由是"发明一个别的席位签约要发的门 id 正是这次审计要防的自我认证";
③ 它判了 Gate C **fail**,而这个 fail 保护的是真实工作。

---

## Gate A — 首个界面的默认页(`disp-first-screen-default-tab`)

Flux 判 **pass**(工作本身),但门事件状态 **ABSENT**(契约 id 从未被发出)。

| Flux 声明 | 我的实测 | 结果 |
|---|---|---|
| `useState<Tab>('dispatches')` 在 track:132、我的分支:132、父提交 5c5437f~1 是 'live-seats' | track:133、HEAD:133、5c5437f~1:92 是 'live-seats' | ✅ 三个 ref 的**值**全对;行号从 132 变 133 是因为我后来合并了 track,这正属于 Flux 自己标注的 `stale_detail` 同类 |
| diff 是 6 insertions / 2 deletions,单文件 | `git diff --shortstat 5c5437f~1 5c5437f` = `1 file changed, 6 insertions(+), 2 deletions(-)` | ✅ 逐字一致 |
| diff 里零 metaKey/ctrlKey 行 | `grep -c 'metaKey\|ctrlKey'` = 0 | ✅ |
| 没有东西持久化 activeTab;App.tsx 只碰 `seatloom.supervisor.open` | `grep localStorage ui/src/App.tsx` 只有 :142 和 :230,键都是 `seatloom.supervisor.open`;全仓 `activeTab` + 持久化 = 0 命中 | ✅ |
| 契约 id `ev-first-screen-default-tab-landed` 从未发出;我发的是 `ev-vrg-SFOoMv9BBj2jx_uSG` | 账本实测:该 id 下只有一条 `ArtifactCreated`,actor `seat-mira-001`,契约 id 不存在 | ✅ **这条说的是我的疏漏,它说得对** |

**Flux 对我这份活的判定是准确的,包括对我错误的那半。** 它没有因为"工作本身是对的"就放过门事件用错 id 这件事。

---

## Gate B — 语料读取(`disp-history-sessions-ui`)

Flux 判 **pass**(能力满足),但门事件 **MALFORMED**(类型和 emitter 都不合契约)。

### 活库数字,逐条复现

| Flux 声明 | 我的实测(`podman exec psql`) | 结果 |
|---|---|---|
| `message_count` NOT NULL = 183 | `count(*) filter (where message_count is not null)` = **183** | ✅ |
| IS NULL = 89 | **89** | ✅ |
| sum = 36680 | **36680** | ✅ |
| per-harness NULL:codex 60 / stepcode 19 / gemini 9 / opencode 1 | **codex 60 / stepcode 19 / gemini 9 / opencode 1** | ✅ 四个全中 |
| 38 distinct corpus projects | **38** | ✅ |
| OfficeBench 49 | **49** | ✅ |
| seatloom 14 | **14** | ✅ |
| 14 cursor rows with first_ts IS NULL | **14** | ✅ |
| total 272 | (Flux 未提,我补测)**272** | — |

**关于 stepcode 的一次测量疏漏,记在这里**:我第一次跑 per-harness 查询时用了 `head -3`,
只看到 codex 60 / gemini 9 / opencode 1 三个,差点判 Flux 少数了一个 harness。补跑完整输出后
stepcode 19 就在那儿。**是我的查询截断,不是 Flux 错。** 一个被 `head` 截断的查询和一次错误的
声明,在日志里长得一样,所以这次核对本身也要复跑。

### 代码声明,逐条复现

| Flux 声明 | 我的实测 | 结果 |
|---|---|---|
| `repositories.rs:970` 和 `:1018` 两个 NOT NULL 过滤器,各自由 `corpus_columns_present()` 门控 | `:970` 和 `:1018` 都在;`corpus_columns_present()` 在 `:961`/`:1010`/`:1041` 三处被调 | ✅ |
| UI 把 NULL 渲染成"未加载"而非 0,在 `HistorySessionsPanel.tsx:141/:373` | `:142` 是 `return n == null ? '未加载' : String(n);`,`:373` 是 `p.messageCount == null && (...)` | ✅ **行号准**(142 vs 141,文件后来动过一行);但 Flux 写的路径 `ui/src/components/` 不对,实际在 `ui/src/app-v2/panel/` |

### 门事件状态

账本实测:`ev-history-sessions-read-live` 存在,类型 **`SupervisorMessage`**,actor **`seat:seat-aegis-001`**,
不是契约 §8.1 要求的 `ArtifactCreated` + emitter Nimbus + `verified` 字段。**Flux 这条准确。**

### 它的一条判断我特别认可

`vela_was_right`:Vela 拒绝写库并升级,而且**用不复读数据库的方式**(从 `SESSIONS-FLAT.csv` 和磁盘归档)
复现了 89 和 per-harness 分布,和 Aegis 的活库测量吻合。Flux 把"不同意一次活库测量的正确方式
是换个来源把它复现出来"写成了标准。这条不是代码断言,是方法论点在了正确的方向上。

---

## Gate C — 会话↔工作桥(`disp-XnCxvGI88oAbdGcyeefhl`)

Flux 判 **fail**。这是我核得最仔细的一门,因为它判的是 fail,而"说不清的 pass比 fail 更糟"反过来也成立:**说不清的 fail 会让真实的工作被挡住。**

| Flux 声明 | 我的实测 | 结果 |
|---|---|---|
| `022_session_work_bridge_view.sql` 在 `packet/onyx/topic-opened-019@0e32054` 和 Nimbus 的 `f7bfe23` 上,但**不在 track 上** | 三个 ref 上都确有该文件。但它现在**在 track 上** —— `git log track -- infra/postgres/schema/022_...sql` 显示 `f7bfe23` 于 **10-26 15:02** 合入,而 Flux 审计是 **10-26 08:04**。**审计当时它确实不在 track 上,Flux 的声明在当时是对的**;之后被合进去了 | ✅ 时间线核对后一致 |
| `apply-db.sh` 的完整性守卫拒应用部分 schema;20 个 schema 文件里**恰好一个**未注册,就是 `021` | `scripts/apply-db.sh:81` 的注册注释逐字写着:"WAS UNCLASSIFIED until 2026-10-26 — **Flux's audit of the three blocked gates (disp-VH6l1HixlUVYuy4m0NEp-) found this was the ONE unclassified file of 20**, and apply-db.sh's completeness guard therefore refused to build ANY database" | ✅ **它的诊断被代码注释直接确认,连派单 id 都引用了** |
| 视图 `v_memory_session_join` 在活库任何 schema 下都不存在 | 现在**存在**:`information_schema.views` 里 `v_memory_session_join` 在。同 Gate C 的时间线 —— Flux 审计时它确实不存在 | ✅ 同上 |
| drift 复现:`both` 87 / `a1_path` 3 / `unresolved` 0 / missing NULL 0 | `select resolved_by, count(*) from v_memory_session_join group by resolved_by` = **both 87 / a1_path 3**,无其他行 | ✅ 逐字一致 |
| 那 3 行是**同一个** `source_ref` 被 3 条记忆携带,且是 `codex:019dda18-...` | 3 条全是 `codex:019dda18-07f6-79e0-8008-e4eb16a5448d`,`resolved_by` 都是 `a1_path` | ✅ 它"比原声明更锐利"的那条也对 |

### 它拒绝做的那件事

`what_i_did_not_do` 里写:它没有自己把 022 应用到 live(应用迁移不是这个席位的动作),所以
"sessions is still 5 rows after apply" 这条是转述 Nimbus 在种子库上的测量,不是它独立确认的。

**这是正确的边界。** 一个只读审计席位把迁移应用到活库,正是它要审的那种越界。

---

## 我核不了的,以及为什么不影响结论

| 核不了的 | 为什么 |
|---|---|
| Nimbus 的"sessions 应用后仍 5 行" | 需要一个应用了 022 的种子库;Flux 自己已声明未独立确认,并说明了原因 |
| Flux 审计当时的 track/活库快照 | 已过去一天多,我只能用"提交时间 vs 审计时间"推断当时状态,而不能回到当时 |

两条都不影响:第一条 Flux 已主动标注为转述;第二条我用时间线核对后与它的声明一致。

---

## 给下游的

`disp-VH6l1HixlUVYuy4m0NEp-` 的 verdict **pass** 依据充分:三门共 20 余条声明(活库数字、代码位置、
门事件类型与 emitter、提交与审计的时间线、迁移注册机制)逐条重跑,全部吻合,含 Gate C 判 fail 的那几条。

**它审计出的两个问题都是真的,而且都已经有下文**:Gate A 的门 id 从未发出(Gate A 的活是我做的);
Gate C 的 021 未注册已修(`apply-db.sh:81` 的注释记录了修复),022 也已注册并合入 track、视图已在活库。
