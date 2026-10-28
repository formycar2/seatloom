# NIMBUS-2026-10-22-operator-decision-object-v1 — 交付说明

派单 `disp-dwSmYkULRU4HdSDJCDWH9`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-22-operator-decision-object-v1.md`。
分支 `packet/nimbus/project-scoped-reads`,提交 `NIMBUS-2026-10-22-operator-decision-020`。

## 一句话

「只有张先生能定的事」从「只以消息存在」变成了一张有结构的表 `operator_decisions`(020),
外加三个 MCP 工具、三个桌面命令、一个界面分区,和 `fleet-status.sh` 里单独数出来的一类。

## 变了什么(文件级)

**迁移(020)** — `infra/postgres/schema/020_operator_decisions.sql`,已注册进 `scripts/apply-db.sh` 的
POST-SEED 列表,理由写在该脚本的条目里。

- 新表 `operator_decisions`:任务包 §5 的六个字段(target / choices / decider / status /
  consequence / evidence)+ 其余 schema 都有的 lineage(project_id / created_by / created_at)。
- 加宽两个词汇表 CHECK,都是 013 的守卫式 DROP+ADD **完整列表**:`event_type` 28→30
  (+`OperatorDecisionRequested`/`+OperatorDecisionResolved`),`ref_type` 12→13(+`decision`)。
- 四条形状 CHECK:状态域、decider 形状(只许 `human:*` / `seat:*`)、创建者形状(只许 `human:*` /
  `seat:seat-aegis-001`)、resolved 配对(chosen/resolved_by/resolved_at 三者 IFF resolved,双向)。
- **没有种子数据**。表按构造就是空的:只有 request 路径能插行,而它要求创建者是操作者或 Aegis。

**Rust 核心** — `crates/seatloom-core`

- `event_domain.rs`:两个 `EventType`、一个 `RefType::Decision`,以及钉住词汇表的那个测试
  (28/11 → 30/13,测试名一并改成 `..._020_checks`)。
- `models.rs`:`OperatorDecisionRow`。
- `repositories.rs`:`get_operator_decision` / `list_pending_operator_decisions` /
  `list_resolved_operator_decisions` / `request_operator_decision` / `resolve_operator_decision`,
  每次写入都是「行 + 事件」一个事务。写入参数收进 `NewOperatorDecision` 结构体 ——
  九个同为 `&str` 的位置参数,排错了 `consequence` 和 `target` 是编译不出来的。
  三个新的 `DbError` 变体,各自的名字就是它的拒绝理由。

**MCP 工具** — `crates/seatloom-mcp-emit/src/decisions.rs`(新模块)+ `protocol.rs` / `main.rs`

- `request_operator_decision` / `resolve_operator_decision` / `list_operator_decisions`。
- 工具总数 22→25,`emit_event` 广告的枚举 28→30 / 11→13(顺带补上了 015 起就该在的 `memory`)。
- 拒绝一律是带类型的、写不了任何东西,并且说出破了哪条规则。

**桌面命令** — `src-tauri/src/commands/operator_decision_cmds.rs`(新模块)+ `dto.rs` / `main.rs`

- `cmd_list_operator_decisions` / `cmd_count_pending_operator_decisions` /
  `cmd_resolve_operator_decision`。
- decider 一律从 `~/.seatloom/operator`(或同名环境变量)解析,**不是参数** —— 否则调用方
  就能读到(并且在 resolve 那条上假装裁定)别人的义务。这跟 014 的 `cmd_accept_delivery`
  用的是同一处 `read_operator_ref`,没有第二份实现。

**界面** — `ui/src/components/OperatorDecisionSection.tsx`(新分区)+ `DispatchInboxView.tsx`
挂载 + `api.ts` / `types-dto.ts`

- 新 tab?没有。挂在派单页「待验收」下面,**首屏结构没动**(任务包明确说那要等张先生拍板)。
- 卡片围绕两句话:在定什么(target),定了之后会怎样(consequence)。选项本身就是按钮 ——
  没有自由文本裁定入口,因为超出所陈述选项的裁定不是裁定。
- 分区标题「等你裁定」,并写明「和上面的『待验收』不是一回事」。

**`scripts/fleet-status.sh`**

- 「等你拍板」现在两类分开数、分开列,每条都标明「类型:验收(交付做完了,要不要收)」或
  「类型:裁定(只有你能定的事)」。
- 表还没建到库里时会明说「这一项没被检查,不是『没有』」,并给出修复命令 —— 一个沉默的 0
  和一个真实的 0 不是一回事。这条路径已经在 live 库上实测过(表确实还不存在)。

## 怎么验的(全部实跑)

| 检查 | 结果 |
| --- | --- |
| `cargo build --workspace` | 零 warning |
| `cargo test --workspace` | 全绿(core 168、emit 89+16) |
| `cargo clippy --workspace --all-targets` | 零 warning |
| `cargo fmt --all -- --check` | 干净 |
| `cd ui && pnpm vitest run` | 199 passed(含新增 5 个) |
| `cd ui && pnpm tsc --noEmit` | 干净 |
| 020 迁移应用到 scratch 库(001→020 全链) | 成功 |
| 020 二次应用 | 零 catalog 写入(NOTICE only),行数不变,触发器不重复 |
| 词汇表核对 | event_type 30 值、ref_type 13 值;8 个 020 之前的 event 值逐一确认未被回退 |
| 六条负面路径(scratch 库,直接 INSERT) | 全部按名拒绝:空白选项(触发器)、空数组(CHECK)、席位创建(CHECK)、假 decider(CHECK)、resolved 无裁定(CHECK)、pending 带裁定(CHECK) |
| 正面路径 | 行落库、`OperatorDecisionRequested` 恰好一条、subject ref 指向该 decision |
| 集成测试(scratch 库) | 6 passed |

## 验收标准逐条

1. **有一条命令回答「有几件在等操作者」,且与界面显示一致** ——
   `cmd_count_pending_operator_decisions` 与 `cmd_list_operator_decisions` 走同一个
   `list_pending_operator_decisions`(decider 一律是解析出来的操作者),`fleet-status.sh`
   用的是同一个 SQL 形状。测过:UI 空态、有卡、已裁定三种状态都一致。
2. **缺 `choices` 或 `consequence` → 拒绝并说出缺哪个** ——
   `validate_request` 按名拒绝(`Empty { field }`),单元测试 `missing_choices_or_consequence_is_refused_by_name`
   逐个跑过空数组、空白选项、空白 consequence、空白 evidence、重复选项、假 decider、席位创建。
3. **席位身份创建 → 拒绝(不是「允许但不鼓励」)** ——
   `may_create_obligation` 只放行 `seat:seat-aegis-001` 和非空 `human:*`;全舰队每个席位都在
   拒绝名单里,近邻串(`seat:seat-aegis`、`aegis`、`human:`、空串、`system`)也都拒绝。
   schema 侧还有 `operator_decisions_creator_shape` 兜底(scratch 库实测)。
4. **已裁定项不可变** —— Rust 路径拒绝二次裁定(`AlreadyResolved`);集成测试
   `a_resolved_decision_is_immutable_at_the_database` 用**裸 SQL** 绕开 Rust 路径,证明
   配对 CHECK 在两个方向都拦得住。也就是说一个命令、一个脚本、一个 psql 会话撞的是同一堵墙。
5. **测试:缺字段拒绝、席位创建拒绝、已裁定不可变** —— 三条都有,见上。都不是同义反复:
   每条都断言了具体的拒绝类型和消息内容,而不是只断言「会失败」。
6. **迁移幂等、无种子数据** —— 实测二次应用零 catalog 写入;无 INSERT。
7. **cargo build/test/clippy/fmt 全过** —— 见上表。

## 过程中发现并修掉的两个真问题

**1. PostgreSQL 不允许在 CHECK 里用子查询。** 第一版把「选项非空白」写成
`NOT EXISTS (SELECT 1 FROM jsonb_array_elements_text(choices) ...)`,scratch 库直接报
`cannot use subquery in check constraint`。这不是格式问题:CHECK 必须能只凭本行求值,规则才在
每一条写入路径(包括裸 INSERT)上都成立。于是拆成两个机制 —— CHECK 保留单行表达式能表达的
(是数组、非空),触发器(`trg_operator_decisions_choices_nonblank`)承载扫元素那一半,并且
**用同一条消息文本**,调用方看到的是一条规则的一次拒绝,不是同一抱怨的两种形状。拆开而不是
丢掉。

**2. `may_create_obligation` 第一版用 `starts_with("human:")`,`"human:"` 本身会通过。**
是测试抓出来的(`"human:"` 后面没名字,那是个前缀而不是身份)。改成 `strip_prefix` 后要求非空。
同一次测试里我还删掉了一个自己写错的断言:`"human:seat-aegis-001"` 我原本断言「不该放行」,
但它**确实该放行** —— 操作者身份就是 `~/.seatloom/operator` 里写的那个,重新解释一个人自己
配的名字等于自创规则。测试改对了,函数是对的。注释里写明了为什么这一条不在拒绝名单里。

## 需要上报 / 留给上位决定的

1. **020 尚未 live 应用**(`operator_decisions` 表在 live 库里不存在,已实测)。按 ONYX 规矩
   这是张先生的动作,不是席位的。应用前:三个 MCP 工具会在 `get_project` 之后撞 FK 而拒绝,
   桌面命令会报 relation 不存在,`fleet-status.sh` 会明说「这一项没被检查」。**没有一处会
   静默显示 0。**
2. **没有为 `emit_event` 之外的新事件类型建投影表。** 020 只往 `canonical_events` 写两行,
   不读投影(读的是 `operator_decisions` 本身),所以没有遗漏。但若将来要按事件重放重建义务
   清单,需要两条 `OperatorDecision*` 的重放逻辑 —— 现在没有,也不假装有。
3. **Aegis 给自己建的义务(decider = `seat:seat-aegis-001`)要从 MCP 路径裁定**,
   桌面命令按 decider 校验拒绝。任务包说「只有操作者和 Aegis 能创建」,裁定侧我按 decider
   字段走:谁的义务谁裁。这与 014 的拆分一致(MCP 侧是席位,点击侧是操作者)。
