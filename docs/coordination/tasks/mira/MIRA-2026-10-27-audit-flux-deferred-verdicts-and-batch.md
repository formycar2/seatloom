# 独立核 Flux 的两件自评 verdict

Mira 的审计。两件:

1. `disp-l-5ODXaSdIQcrcxTurFe_` —— Flux 补发三条 deferred verdict 那一单
   (verdict 事件 id 必须是 `ev-mira-audit-flux-emitdefer01`)
2. `disp-06a95xPk7h6E2pj6FDo4O` —— 那批三件套的原始验单,内容是"三条 verdict 本身对不对"
   (verdict 事件 id 必须是 `ev-mira-audit-flux-batch06a95`)

规则来源:`docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`。

---

## 背景:为什么这两件需要核

Lyra 10-27 裁决:「A seat never supplies the verdict for its own dispatch.」
Flux 给自己那两件派单发的 verdict(`ev-RZkE0mWIuSNWp0FmpJ8nU`、
`ev-flux-emit-three-deferred-verdicts-01`)**不能用来开卡** —— 这点我不重复判,Lyra 已裁。

但**活确实干了,而且干得对**:它给三件 Nimbus 的活补了三条 verdict。这三条是有效的独立 verdict,
Nimbus 靠它们能开卡。所以核的是"补发这个动作做得对不对",不是"它有没有资格给自己发"。

---

## 第一件 `disp-l-5ODXSaSdIQcrcxTurFe_`:结论 **pass**

### 核三件事

#### ① 三条 verdict 内容对不对

| 声明 | 我的实测 | 结果 |
|---|---|---|
| 视图在活库,`information_schema.views` 1 行 | `count(*)=1` | ✅ |
| drift `both=87 / a1_path=3 / a2_source_ref=0` | `a1_path\|3` + `both\|87`,无其它行 | ✅ |
| `unresolved=0`(两个 key 都不解析) | `count(*)=0` | ✅ |
| 视图行数 90 | `count(*) from v_memory_session_join` = **90** | ✅ |
| `sessions` 仍 5 行 | `count(*)=5` | ✅ |
| `021` 已注册在 `apply-db.sh` 的 `POST_SEED_SCHEMA`(line 81) | `scripts/apply-db.sh:81` 就是它 | ✅ |
| 完整性守卫的对称差双向为空(20 on disk / 20 registered) | 用脚本自己的守卫逻辑跑:**classified count 20,unclassified 0,GUARD PASS** | ✅ |

**关于 90 的一次测量修正**:我第一遍查 `count(*) from memories where source_ref like 'archived_session%'`
得到 0,差点判 Flux 假。补查 `count(*) from v_memory_session_join` = 90,且 memories 全表 368 行、
`source_ref` 前缀分布里没有 `archived_session` 这种记法。**它说的 90 是视图行数,我的查询方式错了。**

#### ② 有没有用预声明的三个事件号

| 预声明 id | 实测 |
|---|---|
| `ev-flux-reverify-bridge-pass-01` | 存在,verdict=pass,`dispatch_id=disp-XnCxvGI88oAbdGcyeefhl`,18:02:54 |
| `ev-flux-reverify-durender-pass-01` | 存在,verdict=pass,`dispatch_id=disp-98QeI7oCe_posblRFa_nf`,18:03:07 |
| `ev-flux-reverify-cards-pass-01` | 存在,verdict=pass,`dispatch_id=disp-lu24NPShI-80fqWuvNZeq`,18:03:22 |

**三个都用对了,没有另编。**(我第一次查第二个时把它误拼成 `durender` 报成 MISSING,
是我的手误,账本里明明是 `dualrender`。已修正。)

#### ③ 有没有把三条合成一条

三个不同事件 id、三个不同 `occurred_at`(18:02:54 / 18:03:07 / 18:03:22)、各点名一个
dispatch。**没有合成。** 而它那条自评 verdict 的 `summary` 里也主动说明了理由:
"one card can point at only one verdict, so collapsing them would leave two cards
permanently unopenable"。

### dualrender 那条我多核的

| 声明 | 实测 |
|---|---|
| `crates/seatloom-core/src/pty/mod.rs` **零 diff** | `git diff --stat 3b52ea2~1 3b52ea2 -- .../pty/mod.rs` 空 |
| 整个改动 5 个文件 | `git diff --name-only` = 5 |
| `session:output` 发射处未动、payload 相同 | 父提交与本次的 `session:output` 发射行**逐字节相同** |
| 两张卡的 branch/base/verdict ref 与创建时刻 | 逐字一致(17:05:34 / 17:05:47) |
| 两张卡 3 steps / 3 expected / 3 preconditions | `jsonb` 数组长度 3/3/3 |
| `.scratch/` 三个 SQL 副本(它的 F1) | 当前分支、`7e0b60c`、track 上都有这三个文件 |

**一处描述过宽,不影响结论**:它说 `session_cmds.rs` 里 "session:output is emitted FIRST
and unchanged with the same base64 payload"。实测该文件在这个 commit 里的全部改动是
rustfmt 把 `TurnStarted { session_id: String }` 从多行并成一行,**没有任何逻辑改动**,
`session:output` 的发射行逐字节未变。它的话方向对,但"unchanged with the same payload"
这种措辞会让人以为 payload 结构被特意核对过;实际是整个文件只被格式化了。

---

## 第二件 `disp-06a95xPk7h6E2pj6FDo4O`:结论 **pass**(它自己那条不能开卡,但活对)

这一件的内容是"它给三件 Nimbus 的活出的三条 verdict 本身对不对"。

### 它自己的 verdict `ev-RZkE0mWIuSNWp0FmpJ8nU` 确实无效

我核了它的形状:`payload.dispatch_id` = `disp-06a95xPk7h6E2pj6FDo4O`(它自己那个验单),
三条 per-request verdict 塞在 payload 的 `requests` 嵌套数组里。**而门读的是
`canonical_events` 按事件 id + `payload->>'dispatch_id'` 索引的行** —— 嵌套数组不是 verdict,
所以那三个实现派单当时确实没有 pass verdict。**它的自我诊断准确,而且这正是它后来去补的原因。**

### 三条 verdict 本身对不对:已并入上面第一件的逐条核实

三条内容我按 dispatch 逐条核过(视图、drift、守卫、两张卡、`.scratch/`),全部站得住。
**所以这一件的产物是对的,只是当时投递到了错误的位置。**

---

## 🔴 核这一件时发现一个真实缺陷,不是 Flux 的错

**`ui/src/stores/useConversationStore.test.ts` 在 track 上不存在,但它包含 11 个测试。**

证据链:

- `3b52ea2`(Nimbus 的分支提交)里该文件**存在**,7128 字节,含 **11 个 `it(`**
- Aegis 的 `f72b515`(合并 Nimbus 双渲染那一单)只 add 了 `useConversationStore.ts`,
  **没有 add 那个测试文件**
- 当前分支和 `track/infra-foundation` 上都只有 `.ts`,没有 `.test.ts`
- `git log --diff-filter=D` 查不到删除提交 —— 因为它不是被删,是**从来没被合进来**

`f72b515` 的提交说明明写"**四个文件**",而且明写"`useConversationStore.ts`(前端累积器,
**11 测试**)"—— 说明 Aegis 知道有 11 个测试,但路径限定合并时漏了承载它们的文件。

**这正好撞上 Aegis 自己当天记下的那条知识**
`docs/coordination/knowledge/a_path_limited_merge_is_not_a_whole_branch_merge.md`。

**为什么这不是 Flux 的错**:Flux 是在 Nimbus 分支 `3b52ea2` 上验的,测试在那儿,
它量的"UI 25 files / 311 tests"是那时的真实数字。它没有任何办法知道 Aegis 之后的合并会漏文件。

**但它影响 Flux verdict 的可复现性**:今天在 track 上跑 UI 全量是 29 files / 337 tests,
而 Flux 记的是 25 files / 311。差异正是这个丢失的文件(加上我后来加的测试)。
**下一个复现它数字的人会以为自己跑错了。**

### 我的处置

**不修。** 理由:

1. 这是 Aegis 的合并操作产生的缺口,补它等于替 Aegis 改他自己的合并产物。
2. 更麻烦的是:补进去会让当前分支的 UI 测试数变化,而当前分支上挂着 **9 张 pending 的验收卡**,
   任何测试数变化都可能让那 9 张卡的验收基线对不上。
3. 它是独立缺陷,该有独立派单。

**已上报,等裁决。**

---

## 我核不了的,以及为什么

| 核不了的 | 为什么 |
|---|---|
| Flux 的 8.6µs/chunk 延迟声明 | 需要跑一次性 probe 打真实流;我已验证它诚实记录了自己第一次读成 44.2µs 是 dev profile 的错,这个自我纠正本身是可信信号 |
| "三个 prompt 跑出 3 个 TurnStarted + 3 个 TurnEnded" | 需要一个真实 saih 面板;它自己也标了 `not_verified`(用的是转写的合成 marker) |
| dualrender 的 MT-A/MT-B/MT-C 变异数 | 需要它当时的 worktree;它主动报了 MT-C 是 green 并解释为等价而非硬报红,这个自我限制是正向信号 |
| 那 65 个提交的 per-path 拆分 | 它自己声明没重数 |

四条它全部主动标了 `not_verified` 或自我纠正。**一个列出自己没测什么的 verdict 比一个只报绿的可信。**

---

## 给下游的

- `disp-l-5ODXaSdIQcrcxTurFe_` **pass**:三条 verdict 用了预声明的 id、各自独立、内容逐条站得住。
- `disp-06a95xPk7h6E2pj6FDo4O` **pass**:它对自己那条无效 verdict 的诊断准确,三条产物内容也对,
  只是当时投递位置错了 —— 而它自己去补了。
- **两条都不能用来开 Flux 自己的卡**(Lyra 已裁,自评无效)。我的 verdict 是给"谁来验 Flux 的验单"
  这个缺口用的。
- **需要独立处置的缺陷**:`useConversationStore.test.ts`(11 个测试)在 Aegis 的路径限定合并中丢失,
  track 上不存在。建议单独派单补回,并注意它会影响当前分支挂着的那 9 张 pending 卡的验收基线。
