# Delivery — disp-jwWHSX8Q2OeN66pnV3teY(D5 Phase 2:知识层清洗拆解 + 前向记录协议)

**Seat:** Lyra (PO) · **Dispatch:** `disp-jwWHSX8Q2OeN66pnV3teY` · **Date:** 2026-10-10
**Branch:** `packet/lyra/milestone-summary` @ `bc0d575`

## 做了什么

D5 Phase 2 拆解完毕并全部派发。产物 5 文档(契约 + 拆解 + 协议 + 3 packet),零代码:

| 产物 | 路径 |
|---|---|
| 018 值域契约(三路逐字一致,md5 `ea81bebe…`) | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md` |
| Phase 2 拆解 | `docs/coordination/tasks/lyra/LYRA-2026-10-10-d5-phase2-washing.md` |
| **前向记录协议**(Lane B,本人产出) | `docs/coordination/FORWARD_RECORDING_PROTOCOL.md` |
| Lane A1 packet(Onyx 018) | `docs/coordination/tasks/onyx/ONYX-2026-10-10-memory-hardening-018-v1.md` |
| Lane A2 packet(Nimbus 工具面) | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-memory-tools-018-v1.md` |
| Lane C packet(vela 流水线,Part 1/2 共用) | `docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-v1.md` |

**派发结果(enqueue 返回值即证据):**

| Dispatch | 目标 | 状态 | 依赖 |
|---|---|---|---|
| `disp-mem018-onyx` | seat-onyx-001 | **ready**(p50,高于其 017 closeout) | 无 |
| `disp-mem018-nimbus` | seat-nimbus-001 | **blocked** | `ev-018-memory-hardening-landed` |
| `disp-fwdrec-aegis/flux/mira/nimbus/onyx` | 5 席位 | **ready**(p15) | 无 |
| `disp-vela-register` | seat-aegis-001 | **ready**(p20) | 无 |
| `disp-lyra-wash-p1` | seat-lyra-001(自派单) | **blocked** | `ev-seat-vela-001-registered` |
| `disp-lyra-wash-p2` | seat-lyra-001(自派单) | **blocked** | 018 + typed tools **两门** |

## 关键设计处置

1. **五缺口 → 018(Lane A1,Onyx 侧)+ 工具面升级(Lane A2,Nimbus 侧)**:契约逐字携带
   (md5 校验)。kind 6 值 / supersedes + memory_relations 双拼写同事务 / resolved 配对 CHECK /
   confidence 二值(quoted|inferred,不要分数)/ l2 行号区间(file+archived_session 必填,承重墙)。
   另加一个 payload 未逐字列出的推导:**G3 的闭合需要事件** → 018 加宽 event_type +1
   `MemoryResolved`(全量 27 值写法,与 017 顺序无关);A2 相应新增 `resolve` 工具(否则
   「可关闭」在产品里不存在)。已在拆解文档说明推导理由。
2. **Lane B 是协议不是流水线**:协议文档含 AGENTS.md 增补原文 + 「宁缺勿滥」三判据
   (会改变未来动作 / 别处没有 / 说得出证据);5 席位各派一个 docs-only 小单把 §5 原文
   补进各自 worktree 的 AGENTS.md(我不越界写他席工作区)。typed 字段随 A2 生效,此前按
   当前 remember 形状过渡(kind 暂记 tags),旧行不重写。
3. **Lane C 用自派单做「未来动作日程」**:vela 不存在 → FK 挡着不能直接派;Aegis 注册 +
   发射 `ev-seat-vela-001-registered` → 我的自派单 p1 解封 → 我派 C Part 1(S0–S2.5,零 LLM,
   与 A 并行);C Part 2(S3/S5)等 018 + typed tools 两门(硬顺序红线)。全程零轮询。
4. **作用域红线写进每个包**:只 seatloom(16 sessions/168MB/58,015 行),扩域须张先生决定。

## 怎么验证

- Lane 状态:上表(enqueue 返回值;两个 blocked 自派单即证明门依赖已落库)。
- 契约一致性:三份文档首个代码围栏 md5 均 `ea81bebe5990861029281c511388d5a0`。
- 门事件 id 三处一致:018 包 §5 / A2 包 §3 / 自派单 p2 依赖,逐字符相同。
- 前向协议:AGENTS.md 增补段落在协议文档 §5 可逐字复制。

## 需 Aegis 执行(已路由)

- `disp-vela-register`:注册 seat-vela-001(席位行 + 角色绑定 + tmux/harness + .pi/mcp.json +
  dispatch-seats.tsv,参照 3db670b 的舰队上线模式)+ 发射门事件。
- live apply(018/017)与旧空卡结清仍是他车道;本单未碰。

## 边界确认

只做派单要求的事:拆解 + 契约 + 协议 + 3 packet + 9 个 dispatch。未合并分支、未写 live、
未注册席位(派 Aegis 做)、未替任何席位写实现。验收目标 O1/O2/O3 与门禁①–⑤ 已写进 Lane C
包;各 lane 交付后自请 Flux,验绿后验收方案由我供稿、Aegis 呈张先生。

## 产物索引

- Commit `bc0d575`(5 文档)。
- Dispatch:`disp-mem018-onyx` / `disp-mem018-nimbus` / `disp-fwdrec-*`×5 /
  `disp-vela-register` / `disp-lyra-wash-p1` / `disp-lyra-wash-p2`。
- 账本事件:ArtifactCreated `ev-Dw27iaj6HsbOX4zXRwGxv`(本单产物捕获)。
- **自派单被 stall 升级是预期行为**:`disp-lyra-wash-p1`/`disp-lyra-wash-p2` 是有意 blocked
  (等 `ev-seat-vela-001-registered` / 等 018+typed tools 两门)—— 我的收件箱全 blocked 时,
  调度器的 stall 检测会升级一条 SupervisorMessage,这是契约规则 5 的设计行为,不是故障;
  门事件落地时 013 触发器翻 ready + pg_notify,waker 会唤醒我继续驱动 Lane C。

## Follow-up(2026-10-10 晚):vela 注册门事件已落地

`ev-seat-vela-001-registered` 落账本 → 自派单 `disp-lyra-wash-p1` 解封 → 已派
`disp-wash-vela-p1`(Lane C Part 1:S0–S2.5,零 LLM,seatloom 16 sessions)给
seat-vela-001(status=ready)。`disp-lyra-wash-p2` 仍 blocked,等 018 + typed tools 两门。
