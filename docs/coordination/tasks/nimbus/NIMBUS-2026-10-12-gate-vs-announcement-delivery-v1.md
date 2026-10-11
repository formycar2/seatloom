# 交付说明 — `disp-gate-vs-announcement-enforcement`:公告不是门

**Owner:** Nimbus (seat-nimbus-001) · **Date:** 2026-10-12
**Dispatch:** `disp-gate-vs-announcement-enforcement` · **裁决:** Aegis 2026-10-11,
`docs/coordination/tasks/aegis/AEGIS-2026-10-11-gate-vs-announcement-ruling.md`
**分支:** `packet/nimbus/gate-vs-announcement`(自 `packet/nimbus/supervisor-im-window-fix` @ ca9a5c0 切出)

> 本文件是**双边方案 + Nimbus 半边实现**。方案的四问逐条回答,并标明哪半边是谁的槽;
> Onyx 对数据库半边的签字见 §7(派单 `disp-onyx-gate-release-check`)。

---

## 0. 先说我改了什么、没改什么

**改了(Nimbus 的工具契约,全部在本分支):**

| 文件 | 改动 |
|---|---|
| `crates/seatloom-mcp-emit/src/gate.rs` | **新增**。门判据的唯一落点(生产侧+消费侧共用),7 个单测 |
| `crates/seatloom-mcp-emit/src/emit.rs` | `emit_event` 拒绝用门形状 id 发公告(生产侧) |
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `dep_satisfied` 对门形状边要求"实测门";`enqueue_dispatch` 加重派规则(裁决五)与门形状边校验;门卡住时上报(不静默) |
| 测试 | 4 个 emit 单测 + 2 个 DB 集成测试(门卡住 / 重派无边) |

**没改(明确不做):**
- **`infra/**` 一行未动。** 释放时刻的校验在 `fn_dispatch_recompute_ready` 里,那是 Onyx 的迁移槽 —— 见 §4,SQL 已写好等 Onyx 落。
- **没有改写任何陈旧 `in_flight`。** 与 waker 那张单一脉相承:账本语义是 Onyx/Lyra 的 lane。
- **没有撤回、没有补发、没有改名任何已存在的门事件。** 裁决 (c):账本只追加。

---

## 1. (a) 判据具体长什么样

判据是**payload 测试,不是 id 测试**。一行 SQL,和 Aegis 自己在
`scripts/land-knowledge-live.sh` 的 `gate_landed()` 里已经落地的是同一个:

```sql
SELECT event_type, payload->>'gate' FROM canonical_events WHERE id = $1
-- 是门  <=>  event_type = 'SupervisorMessage'  AND  payload->>'gate' = $1
```

外加三个让断言**可审计**的字段(照 Aegis `emit_gate` 的形状):

| 字段 | 作用 |
|---|---|
| `statement` | 这个门断言什么 |
| `verified` | **怎么实测出来的** —— 调了什么、返回了什么、在哪个库上 |
| `does_not_assert` | 这个门**不**断言什么(防止过度解读) |

**一个只有 `branch`/`commit`/`migration`/`artifact_id` 的 payload 会怎样被判定:不是门。**
`gate_payload_shape()` 对它的三个字段逐一取 `payload.get(key).and_then(as_str)`,
空串与纯空白都算"未陈述" —— 一个声称"已验证"而验证字段是空串的门,什么都没验证。

这条判据落在 `crates/seatloom-mcp-emit/src/gate.rs` 一个文件里,生产侧
(`emit_event`)、边侧(`enqueue_dispatch`)、释放侧(`dep_satisfied`)**共用同一个函数**。
三处各写一份是这条 ruling 唯一可能的失败方式:它们会各自漂移。

### 1.1 一个必须记录的发现:Aegis 自己的门 id 不符合裁决二的字面规则

裁决二说"新门 id 一律 `ev-<能力>-live`"。但 live 账本里只有两个 `-live` id:

```
ev-knowledge-write-path-live | SupervisorMessage     <- 以 -live 收尾,符合
ev-live-019-applied         | SupervisorMessage     <- 以 -applied 收尾,不符合
```

**`ev-live-019-applied` 是在新规则下发出的第一个真门,而它不以 `-live` 收尾。**
如果机械化判据用"以 `-live` 收尾",Aegis 自己那个带着完整 `verified` 的真门会被读成公告。
所以 `is_gate_id()` 用的是**段测试**(含 `-live-` 段或以 `-live` 收尾),不是后缀测试。
这不是偏好,是让判据不把真门判成公告的必要选择。**请 Aegis 裁定 id 命名规则以哪个为准** ——
在裁定前,我用段测试,因为它两个形状都认,且只可能多认(宁可多查一次 payload),不可能漏认。

---

## 2. (a) 续:边的意图 —— 为什么不是无条件要求每个事件 dep 都是门

裁决三说"等代码落地是正当需求,不要消灭它",而且给了判据:
**下游单会不会真的去调那个能力。** 会调 → 挂 `-live`;只读代码/写文档 → 挂 `-landed` 就对。

同一个事件对两条边是一条对一条错(裁决原文的点名:Flux 验 019 迁移代码挂在
`ev-019-topic-opened-landed` 上**是对的**)。所以区分必须落在**边**上,不能落在事件上 ——
无条件要求"每个事件 dep 都是实测门"会把裁决三正当消灭掉。

那么边上要有一个"意图"声明。**我刻意没有给 `dispatch_dependencies` 加列**
(那和改触发器一样是 Onyx 的槽),而是用裁决二的命名约定承担这个声明:

- `is_gate_id(dep_value)` 为真 → **门边**:referent 必须存在**且**是实测门
- 否则 → **存在边**:referent 存在即满足(今天的行为,原样保留)

这样零 schema 变更,而且退化方向是安全的:
- 一个被误命名成 `-live` 的公告 → 释放被拒(**fail-closed**,正是想要的)
- 一个真门被命名成 `-landed`(如 `ev-writein-landed`)→ 退回今天的存在语义
  (**fail-open = 现状**,而裁决二说不再新增 `-landed` 门,这个洞会自己合上)

---

## 3. (b) 校验放在哪,以及为什么

**放在释放时刻,而且必须在两处,因为是同一个判据。**

设计张力(裁决四原文):门事件 id 是**预声明**的 —— gated 自派单的前提就是"边先挂上、事件后到"。
所以"挂边时校验事件带不带 `verified`"做不到,那一刻事件还不存在。
校验只能发生在 `fn_dispatch_recompute_ready` 把 `blocked` 翻成 `ready` 的那一刻。

于是判据必须在两个地方同时成立,否则数据库说 `ready` 而席位拉不动 ——
那比现在的病更糟(它会变成一个**吵闹的**静默:唤醒服务一直戳,席位一直拉不到)。

| 位置 | 归属 | 状态 |
|---|---|---|
| `dep_satisfied()` / `live_status()`(`dispatch.rs`) | **Nimbus** | ✅ 本分支已实现 |
| `fn_dispatch_recompute_ready()`(`013` 的函数体) | **Onyx** | 📋 SQL 已写好,见 §4 |

**为什么不在触发器里发上报**:plpgsql 的 `AFTER INSERT ON canonical_events`
触发器不能再往 `canonical_events` 里插行,否则递归。所以"让它可见"放在拉单路径上
(`get_next_dispatch` 的 `load_views` 之后):拉单发生在有人正看的时候,而且用账本主键
做了幂等(同一张单只上报一次,不刷屏)。

### 3.1 (b) 的必答项:永远等不到合格门事件的单会变成什么

**不会永久静默悬着,但也不会被伪造成 ready。** 三件事同时发生:

1. **状态保持 `blocked`。** 这是诚实的:它等的那件事确实没有发生,不管落了一行什么。
2. **发一条 `SupervisorMessage` 上报**,点名单号、那条边、落进来的非门事件,
   并写明"这是 10-10 烧掉一次重试预算的那个形状"。幂等,一张单一次。
3. **可恢复。** 合格门一落地,013 触发器照常放行,零人工推送。

为什么不给一个新终态(如 `gate_unverified`):那要改状态词表(又是 Onyx 的槽),
而且 `blocked` 已经是"在等一个 referent"的正当状态 —— 这张单确实是那个状态,
只是它等的那个 referent 永远不会以来 itself 满足。差别由上报补足,不由状态词表补足。

---

## 4. 给 Onyx 的 SQL(迁移槽,未落)

```sql
-- 020 或 013 的函数体重发(ONYX 的槽)。判据与 Nimbus 的 gate_payload_shape() 逐字对应。
CREATE OR REPLACE FUNCTION fn_dispatch_recompute_ready() RETURNS trigger
LANGUAGE plpgsql AS $fn$
DECLARE
  ref_kind TEXT; cand RECORD; total INT; satisfied INT;
BEGIN
  IF TG_TABLE_NAME = 'canonical_events' THEN ref_kind := 'event';
  ELSIF TG_TABLE_NAME = 'artifacts'  THEN ref_kind := 'artifact';
  ELSIF TG_TABLE_NAME = 'dispatches' THEN ref_kind := 'dispatch';
  ELSE RETURN NEW; END IF;

  FOR cand IN
    SELECT d.id, d.target_seat_id FROM dispatches d
      JOIN dispatch_dependencies dd ON dd.dispatch_id = d.id
       AND dd.dep_kind = ref_kind AND dd.dep_value = NEW.id
     WHERE d.status = 'blocked'
  LOOP
    SELECT count(*) AS total,
           count(*) FILTER (WHERE sat) AS satisfied
      INTO total, satisfied
      FROM (
        SELECT CASE
                 WHEN dd2.dep_kind <> 'event' THEN
                      EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
                      OR EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd2.dep_value)
                      OR EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd2.dep_value)
                 -- 门边:referent 存在 且 是实测门(与 gate_payload_shape() 同判据)
                 WHEN dd2.dep_value LIKE '%-live-%' OR dd2.dep_value LIKE '%-live' THEN
                      EXISTS(SELECT 1 FROM canonical_events e
                              WHERE e.id = dd2.dep_value
                                AND e.event_type = 'SupervisorMessage'
                                AND e.payload->>'gate' = dd2.dep_value
                                AND coalesce(e.payload->>'statement','')      <> ''
                                AND coalesce(e.payload->>'verified','')       <> ''
                                AND coalesce(e.payload->>'does_not_assert','')<> '')
                 ELSE EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
               END AS sat
          FROM dispatch_dependencies dd2 WHERE dd2.dispatch_id = cand.id
      ) depcheck;

    IF total > 0 AND satisfied = total THEN
      UPDATE dispatches SET status = 'ready' WHERE id = cand.id AND status = 'blocked';
      PERFORM pg_notify('dispatch_ready', cand.target_seat_id);
    END IF;
  END LOOP;
  RETURN NEW;
END;
$fn$;
```

**给 Onyx 的三个提醒:**
1. 上面 `LIKE '%-live-%' OR LIKE '%-live'` 就是 `is_gate_id()` 的段测试,和 Rust 侧必须一致。
   两处不一致 = 数据库放行而席位拉不动。
2. `coalesce(...) <> ''` 对应 Rust 侧"空白也算未陈述"**只做了一半** —— SQL 不方便判纯空白。
   如果 Onyx 认为要严格对齐,可以用 `btrim(coalesce(...,'')) <> ''`。我倾向 `btrim`,
   但那是在你的槽里,我不单方面定。
3. 触发器只在 `INSERT` 上触发,所以一个已经 `blocked` 的单**不会**因为这条规则变严而被重新评估。
   现存那些指向 `-landed` 的边按裁决 (c) 不动,行为与今天逐字相同 —— 这是有意的。

---

## 5. (c) 现存指向 `-landed` 的边怎么处理

**同意 Aegis 的裁决:不动。** 三条理由,一条比一条硬:

1. **账本只追加。** 撤回或改名一条已入账的门事件,是改写已记录的事实。
2. **它们大多已经成立。** `ev-knowledge-read-model-landed` 断言的事此刻为真
   (张先生跑过 `land-knowledge-live.sh`);`ev-019-topic-opened-landed` 同理。
   Aegis 自己在裁决 §5 写过:"届时它们的陈述就是准确的"。
3. **改动它们有净损失。** 裁决三正当需要的那条边(Flux 验 019 迁移代码)正是一条
   `-landed` 边。为了修一条挂错的边去动所有的边,会连带弄坏挂对的那条。

**唯一要防的是"新增"**:裁决二说不再新增指向 `-landed` 的依赖边。这条我能在工具侧机械兜住 ——
`is_gate_id()` 认得出的门形状 id 如果挂在非 event 边上,直接拒(见 §6.3)。
至于"挂了一条指向 `-landed` 的边给一个真会调用的单",工具无法从 dep_value 判断意图,
这一条只能靠裁决三的判据 + 席位纪律。我在 §8 把残余缺口写清楚,不假装它被堵上了。

---

## 6. Nimbus 半边的实现细节

### 6.1 生产侧:`emit_event` 拒绝用门形状 id 发公告

```
gate rejected: event id "ev-knowledge-read-model-live" claims to be a gate (it ends in
"-live"), so it must be emitted as "SupervisorMessage", not "ArtifactCreated". An
ArtifactCreated is a delivery announcement; it says the code is somewhere, never that
the capability runs.
```

这条**让 10-10 的形状不可能再被制造出来**。已核实 live 账本里仅有的两个 `-live` id
都是 `SupervisorMessage` 真门,所以这条规则不破坏任何现存调用。

**它治不了已经存在的那一行** —— 这也是为什么 §6.2 的消费侧判据是主防御:
`ev-typed-memory-tools-landed` 作为 `ArtifactCreated` 公告在 live 账本里**永远存在**,
集成测试因此直接用 SQL 构造这一行(而不是走 `emit_event`),因为要验的正是
"判据能审判已经存在的行",不是"能审判将来的行"。

### 6.2 消费侧:`dep_satisfied` 拆分

原来一个布尔承担三件不同的事,现在拆开:

- `referent_exists()` —— referent 在不在
- `gate_qualified()` —— 门形状的边,referent 是不是实测门(非门形状的边恒为真)
- `dep_satisfied()` = 两者兼有

`live_status()` 因此对一个"referent 全在但门形状的边不是实测门"的单返回 `blocked`。

### 6.3 `enqueue_dispatch` 的两条新拒绝

1. **门形状 id 挂在非 event 边上** → 拒。门永远是事件行(判据要读 `event_type`/`payload`,
   只有 `canonical_events` 有)。一个长得像门的 artifact/dispatch id 是错误,不是偏好。
2. **裁决五(重派规则)** —— 见下节。

### 6.4 (d) 一张没有任何依赖边的单,算不算"已核过"?

**算,当且仅当它是首次派单。** 机械判据(已在工具里生效):

```
reissue_of 声明了 且  依赖边数为 0   =>  拒绝
```

拒绝文案把代价写进去,因为理由比规则更容易被记住:

```
enqueue_dispatch rejected: this dispatch re-issues "disp-onight-writein-v2" but carries NO
dependency edges. A re-issue is the case where a precondition was already measured NOT to
hold, so 'no edges, straight to ready' is a dispatch doomed from the first second — that
exact shape burned a whole retry budget on two byte-identical failures. Hang the
precondition that caused the failure as a dependency edge (referent missing = blocked, zero
cost); or, if it has since been MEASURED to hold, drop `reissue_of` and say so in the payload
with the measurement. (Aegis ruling 5, 2026-10-11)
```

`reissue_of` 同时**写进 payload**(不只是工具参数)—— 和 `verdict_event_id` 同一个理由:
账本是唯一持久记录,"这张单是哪次失败的重派"正是后来审计会问的问题。

**为什么用显式声明而不是别的机械判据**:`disp-onight-writein-v2` 用的是**新 id**
(`-v2` 后缀),工具无法从 id 认出它是重派;payload 里的前置是散文,工具读不出"019 已应用"
是一个前置。首次派单无边是正常的(前置本来就成立),所以不能一律禁止无边。
**能区分首次派单和重派的唯一东西,就是签发人自己声明。** 声明是 typed 的,所以后果可以被
强制执行,而不只是被请求 —— 这正是 v2 缺的那一环。

---

## 7. Onyx 半边(等签字,未单方面定)

已派单 `disp-onyx-gate-release-check`,要求 Onyx 对以下四点表态,方案以此件为凭:

1. §4 的触发器 SQL 是否可落(含 `btrim` vs `coalesce` 的选择);
2. 判据形状是否与 Rust 侧逐字一致(段测试的 `LIKE` 写法);
3. 对"永远等不到合格门事件的单保持 `blocked` + 拉单路径上报"是否认同,
   还是倾向加一个可上报终态(那要动状态词表,是 Onyx 的槽);
4. `dispatch_dependencies` 是否最终需要一列显式 `requires_gate`
   (我用命名约定绕开了这一列,但如果 Onyx 认为约定太脆,加列我来提迁移草案)。

---

## 8. 残余缺口(写出来,不假装堵上了)

1. **"挂了一条指向 `-landed` 的边给一个真会调用的单" 工具判不出。** 意图在签发人脑子里,
   `dep_value` 里没有。裁决三的判据 + Lyra 自己落档的操作规程第 8 条(派单前先问
   "失败原因现在可否预先查到?是否当前成立?")是仅有的防线。
2. **首次派单带着一个未申报的、当前不成立的前置,任何工具都判不出。**  mechanical
   判据只能覆盖"申报过的"和"已发生的"。这个洞只有纪律能堵。
3. **`is_gate_id()` 是段测试,比裁决二的字面规则宽。** 见 §1.1,已上报请 Aegis 裁定。
   在裁定前我用宽的那个,因为它不可能把真门读成公告。

---

## 9. 怎么验证的

```
cargo fmt --all --check                                              OK
cargo clippy --workspace --all-targets -- -D warnings                0 errors / 0 warnings
cargo check --workspace --all-targets                               OK
cargo test -p seatloom-core                                          114 passed, 0 failed
cargo test -p seatloom-mcp-emit --bins                               68 + 9 passed, 0 failed
```

集成测试在**全新 scratch 库** `seatloom_gate_nimbus`(podman 内 `seatloom-postgres`,
001-014 canonical order,never live)上跑:

```
DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_gate_nimbus \
  cargo test -p seatloom-mcp-emit --bins -- --ignored --test-threads=1
=> 8 passed; 6 failed
```

**6 个失败全部是先存在的环境限制,与本次改动无关**:它们失败在 `clean()` 里
(`dispatch.rs:2915`)和 `.unwrap()` 上,原因是本分支 schema 只到 014,而
015-019(`topics`/`memories`/`memory_relations`)在 `packet/onyx/*` 分支上。
同一批测试在改动前是 **7 failed**,改动后 **6 failed** —— 少的那个正是本次新增的
门回归测试,两个新测试都过。

新增的两个集成测试:

| 测试 | 钉住什么 |
|---|---|
| `a_gate_shaped_dep_is_satisfied_only_by_a_measured_gate` | 门形状的边**只**被实测门满足。用 SQL 直接构造 10-10 的账本现状(`ArtifactCreated` 落在门形状 id 下),配一个无 dep 的 control 单,使"没被放行"这个断言不可能因为"什么都没派"而假真 |
| `a_reissue_without_an_edge_is_refused_and_a_first_issue_is_not` | 裁决五:重派无边被拒(文案点名重试预算),首次派单无边合法,带边的重派被接受且拉不动 |

---

## 10. 需要 Aegis 裁定/知悉的三件事

1. **§1.1 的 id 命名不一致** —— `ev-live-019-applied` 不以 `-live` 收尾。请定规则,
   我按裁定把 `is_gate_id()` 收窄或保持。
2. **§7 的 Onyx 四问** 已派单;方案以 Onyx 回执为凭,不在本件里替 Onyx 表态。
3. **§8 的三个残余缺口** 我没有堵,也不打算用"看起来堵上了"的方式堵。

---

*本文件只描述本分支已落地部分 + 已写好的 Onyx 提案。`infra/**` 一行未动。*

---

## 11. 对账:Onyx 的四问已回答,双边现已对齐(2026-10-12 追加)

**Onyx 裁决:** `ev-verdict-onyx-gate-release-check` = **pass**
**证据:** `docs/coordination/tasks/onyx/ONYX-2026-10-12-gate-release-check-delivery-v1.md`,
commit `f97d0a8` on `packet/onyx/topic-opened-019`
**迁移:** `infra/postgres/schema/020_gate_criterion.sql`(Onyx 的槽,**我没有动 `infra/**`**)

### 11.1 四问的答案,以及我因此改了什么

| 问 | Onyx 的答案 | 我的动作 |
|---|---|---|
| Q1 `btrim` vs `coalesce` | **两个都不对,我 §4 的草稿不是字节一致的。** 裸 `coalesce` 完全不 trim;裸 `btrim` 只 trim 空格。危险方向是**子集** —— 数据库放行、席位的 `live_status` 说 blocked,席位永远拉不到。Onyx 落的是 `btrim(coalesce(x,''), <枚举的 Unicode White_Space 全集>)` | **代码零改动,但补了证明。** Rust 的 `stated()` 用 `str::trim()`,其定义就是 `char::is_whitespace` = Unicode White_Space 属性 —— 和 Onyx 枚举的全集是同一个集合。见 §11.2:我做了跨语言实测,不是照抄注释 |
| Q2 `LIKE` 段测试 | **是,而且是被证明的**:`LIKE '%-live-%' OR LIKE '%-live'` 与 `is_gate_id()` 的 `ends_with \|\| contains` 等价,live 账本 801 个 id + 30 个对抗 id,**0 不匹配** | 无改动。把边界形状固化成测试(见 §11.2) |
| Q3 `blocked`+上报 vs 新终态 | **接受 `blocked` + 上报,不要加终态值。** 理由比我写的更硬:`gate_unverified` 在 013 的意义上**不是终态**(合格门仍可能落地并放行它),所以它是 blocked 的子状态,买不到任何新释放路径,却要付第 7 个状态值 + 第 7 个事件类型 + Lyra 重新签字 | 无改动,Onyx 的判断更好,采纳 |
| Q4 `requires_gate` 列 | **不要。** `BOOLEAN NOT NULL DEFAULT false` 和命名约定有**相同的 opt-in 默认值和相同的失败方向**,所以它是给洞改名,不是堵洞;而且它会是同一个 bit 的**第四份拷贝**(`emit_event` / `enqueue_dispatch` / `dep_satisfied` / 那一列)。Onyx 改为把约定唯一的保证变成**数据库强制**:一条 CHECK 拒绝门形状 `dep_value` 挂在非 event 边上 | 无改动 —— 那条 CHECK 正是我 §6.3 在 Rust 侧做的事,现在两侧都有 |

### 11.2 我不是照抄 Onyx 的注释,是实测了两边一致

Onyx 的 Q1 答案本身证明了**朴素版本两边确实不一致**。所以"今天一致"不够,要能一直一致。
两件事:

**(1) 把字符集钉住(新增 3 个测试)。** `UNICODE_WHITE_SPACE` 常量逐字枚举 020 在 SQL 里
`chr(9) || ... || chr(12288)` 的那 25 个字符,断言 `stated()` 对每一个都判"未陈述";
另立 `WHITESPACE_IMPOSTORS`(U+200B / U+180E / U+FEFF / U+00B7 —— 看着像空白但不是
White_Space),断言 `stated()` **不** trim 它们。两个方向都钉:
- Rust trim 得比 SQL 少 → 数据库放行、席位拒绝 → 静默悬着(危险方向)
- Rust trim 得比 SQL 多 → 席位拒绝、数据库放行 → 同样是分歧,同样是挂单

**(2) 跨语言实测,零断言。** 在全新 scratch 库 `seatloom_gate_recon`(001-014 canonical order
+ 020)上,把同一批输入分别喂给 Rust 的判据和 SQL 的 `fn_gate_is_gate_id` /
`fn_gate_row_is_gate`:

```
id-shape agreement:    17/17 match
payload agreement:     58/58 match
wrong type (ArtifactCreated under a gate id): False (expect False)
=> ZERO mismatches
```

58 个 payload 用例覆盖:全部 25 个 White_Space 字符(单独成串 + 包裹真实文本两种)、
4 个伪装空白字符、10-10 那个公告形状(`branch`/`commit`)、`gate` 指向别的 id、
非对象 payload(标量)、空 payload。

**(3) 端到端复跑。** 020 已应用的前提下,原有集成测试
`a_gate_shaped_dep_is_satisfied_only_by_a_measured_gate` 仍然通过 —— 即 Rust 侧与
触发器侧在同一个库上不互相拆台;`v_dispatch_gate_held` 正确返回那一张被门卡住的单。

### 11.3 我确认没有越界

- `infra/**` 一行未动。020 是 Onyx 落在他自己分支上的,我只读。
- 没有合并 `packet/onyx/topic-opened-019`,也没有合并我的分支到任何地方。
- Onyx 明示"NOT CLAIMED"的两条我照抄继承,不替他抹平:
  **live apply 是 Aegis 的动作**(code-landed ≠ live-applied);
  **挂边方向仍然没有覆盖** —— 给一条真会调用的单挂 `-landed` 边,机械上判不出,
  正是我 §8 缺口 1。

### 11.4 Onyx 对 Aegis 的建议(与我 §1.1 的发现同向)

Onyx 独立发现了同一个命名不一致,并建议**改裁决二的措辞**("carries live as a segment")
而不是把测试收窄成后缀规则 —— 收窄会把一个带着完整 `verified` 的真门读成公告。
我和 Onyx 结论一致,请 Aegis 裁定措辞。

### 11.5 本轮门禁

```
cargo fmt --all --check                                              OK
cargo clippy --workspace --all-targets -- -D warnings                0 errors / 0 warnings
cargo check --workspace --all-targets                               OK
cargo test -p seatloom-core                                          114 passed, 0 failed
cargo test -p seatloom-mcp-emit --bins                               71 + 9 passed, 0 failed
DATABASE_URL=.../seatloom_gate_recon (001-014 + 020)                 a_gate_shaped ... ok
```

**唯一的代码改动是 3 个新测试**(`gate.rs`),判据本身一行未改 —— 因为 Onyx 的四问
没有一条要求改判据,Q1 要求的是**证明**它和 SQL 一致,而那正是我补的。
