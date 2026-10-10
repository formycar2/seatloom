# D5 Phase 2 拆解 — 知识层清洗(限 seatloom)+ 前向记录

**Owner:** Lyra (PO) · **Date:** 2026-10-10 · **Governing dispatch:** `disp-jwWHSX8Q2OeN66pnV3teY`
**状态:** 拆解完成;Lane A1/A2 已派(依赖边串联)、Lane B 协议已定稿并派 5 席位、Lane C 分两 Part
(等 vela 注册与两门事件,由 Lyra 的自派单在门落地时驱动)。

---

## 0. 框架与作用域(张先生定的)

知识层从「管 work」扩展到「管 knowledge」的 Phase 2:清洗历史(回填)+ 前向记录(从下一个
派单开始积累)。**作用域收紧:只做 seatloom 一个 project**(16 sessions / 168MB / 58,015 行 /
2026-04-24→10-08,占全量 1.2%);扩到其余 17 个 project 需张先生另行决定,不是自动下一步。

**硬顺序红线:Lane A(018 schema 加固)完成前,不得开始 S3 LLM 抽取。** 实测 015 的 memories
22 列有五个真缺口,往装不下价值的容器里灌 token,补完还得重洗 = 重花一次。

## 1. Lane 划分

| Lane | 席位 | 内容 | 状态 |
|---|---|---|---|
| A1 | Onyx | migration 018:五缺口(kind/supersedes+relations/resolved/confidence/l2 行号)+ MemoryResolved | **ready**(p50,高于其 017 closeout) |
| A2 | Nimbus | 工具面:remember typed 入参、recall 排除 superseded、新 resolve 工具、Rust 27 值 | blocked → `ev-018-memory-hardening-landed`(p45) |
| B | 5 席位(aegis/flux/mira/nimbus/onyx) | 前向记录协议写进各自 AGENTS.md 完成协议;宁缺勿滥判据 | **ready**(p15,协议即时生效) |
| C1 | seat-vela-001(待注册) | 流水线 Part 1:S0 归一化 / S1 脱敏 / S2 切 topic / S2.5 剪枝(零 LLM) | 等 `ev-seat-vela-001-registered` |
| C2 | seat-vela-001 | 流水线 Part 2:S3 抽取 / S4 分层 / S5 入库 | 等 **两门**:018 + typed tools |

## 2. 接线(全部数据边,零轮询)

```
Aegis 注册 seat-vela-001 + emit ev-seat-vela-001-registered
   → Lyra 自派单 disp-lyra-wash-p1 解封 → Lyra 派 C1 给 vela

Onyx 018 落地 + emit ev-018-memory-hardening-landed
   → Nimbus A2 解封(p45)→ 交付 + emit ev-typed-memory-tools-landed
   → Lyra 自派单 disp-lyra-wash-p2 解封(两门都要)→ Lyra 派 C2 给 vela
```

自派单(dispatch target=seat-lyra-001)是「未来动作的日程」:门事件经 013 触发器把自派单翻
ready 并唤醒 Lyra,Lyra 醒来即派下一给 vela。C1 不等 018(S0–S2.5 零 LLM,与 A 并行,
payload 明示);C2 等两门(S3/S5 写 typed record,硬顺序)。

## 3. 契约(两侧逐字一致)

`VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md` —— 018 的五缺口规格 + MemoryResolved
+ 跨 Rust⇄PG 手写枚举纪律,逐字携带进 A1/A2 两包;Lane C 的 S3 产出受同一词表约束。
前向记录协议:`docs/coordination/FORWARD_RECORDING_PROTOCOL.md`(含 AGENTS.md 增补原文与
「宁缺勿滥」三判据:会改变未来动作 / 别处没有 / 说得出证据)。

## 4. 验收(门禁,非主观)

整体 O1/O2/O3;逐项:S2 topic 清单交张先生肉眼判断「是不是独立工作主题」不过关则停;
S1 已知含密钥样本必须命中;S2.5 剪枝统计必交;抽样人读 L0/L1(抓要点非复述 + confidence
诚实);**L2 回溯抽验随机 5 条按路径+行号开原文**。全过 → Flux → Lyra 出验收方案 → Aegis 呈卡。

## 5. 边界与已知并行项

- Onyx 在飞:017 closeout(三卡);Nimbus 在飞:两分支同步 + 防线 + 两卡;Mira 在飞:一卡。
  本单各 lane 排在其后(one-in-flight),优先级只决定就绪集中的先后。
- 旧空卡结清、live apply、seat-vela-001 的注册与运行时(tmux/harness/.pi/mcp.json/
  dispatch-seats.tsv)= Aegis 车道;本单只派单与供稿。
- 不整分支合并;事件驱动零轮询;阻塞或需张先生判断 → emit_event + 上报 Aegis。

## 6. 证据索引

- 契约:`VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md`
- 协议:`docs/coordination/FORWARD_RECORDING_PROTOCOL.md`
- Packets:`ONYX-2026-10-10-memory-hardening-018-v1.md` /
  `NIMBUS-2026-10-10-memory-tools-018-v1.md` /
  `VELA-2026-10-10-washing-pipeline-v1.md`(C1/C2 共用,Part 分派)
- 设计权威:`AEGIS-2026-10-10-washing-pipeline-design-v2.md`(Aegis 分支)
