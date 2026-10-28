# `disp-x7QjIhw2ZsvokB_yidBj1` — P0:注释里那句没跑过的变异数字

## 一句话

**注释里那句假话删了,但我不只删 —— 我把那个次序补成了真测试,所以注释现在能回答「这个 guard 凭什么留着」,而且答案里的每个数字都是我跑出来的。**

---

## 先说 Aegis 的三个事实,我都独立复现了

不信转述,自己跑。

### 事实 1:Flux 的 "RED (2 failures)" 是假的 —— 复现了

按注释原文做变异(M2 = `kind = failure.kind`)+ 把 `askSupervisorWindowStatus()` 移到两个订阅之后:

```
Test Files  1 passed (1)
     Tests  20 passed (20)
```

**0 red。** 与 Aegis、Nimbus 一致。

### 事实 2:guard 确实承重,证据是"订阅拒绝晚于查询拒绝" —— 复现了

我把这个次序写成了正式测试(见下)。在 M2 下它 **RED 1**。

### 事实 3:Nimbus 的机制是错的 —— 我也数了代码

```
查询链:  .then((s)  => {...})  .catch((err) => {...})
订阅链:  .then((fn) => {...})  .catch(reportSubscriptionFailure(...))
```

**两边都只有一个 `.then`。** 他说"查询链多一个 `.then`"不存在。

真实原因(我补充一句 Aegis 没细说的):查询的 `.catch` 挂在 `.then` **之后**,而那个 `.then`
回调里又**同步**调了一次 `setSupervisorSyncLost`(成功后清条那条路径)。所以查询的同步路径
先落地,它的 `.catch` 要等 promise 真的 reject 才跑 —— 于是查询的拒绝总是最后到。

**结论对、理由错,比结论错更难发现**:下一个人照他的理由推,会推出错的结论。

---

## 我做了什么

### 1. 新测试:`a subscription failing AFTER the query does not demote the shape`

这是派单 C 要的那个测试,**但不是 Aegis 探针的照抄** —— 他的开关加在他的探针里,派单明说
"我那个是探针不是交付物,你验收时自己重写"。我重写了:

- mock 里加 `holdReembedListen` + `pendingReembedReject`:扣住 `supervisor:reembedded`
  这条订阅的 promise,让它可以**晚于**查询拒绝
- 测试流程:`rejectStatus = true` + `holdReembedListen = true` → 挂载 → 查询先失败,
  断言拿到红色 never-answered → 释放订阅拒绝 → 断言**形状没被降级**、订阅的理由仍在、
  重试按钮仍在

**为什么必须扣住订阅而不是查询**:已有的那条 `a subscription failing BEFORE the query does
not steal the shape` 走的是反方向(扣住查询,让订阅先失败)。那条测的是
`failure.kind === 'never-answered'` 那一半 —— 查询晚到,它自己的 kind 就能升级形状。
**`prev?.kind` 那一半只有查询先失败时才被碰到**,而这个次序在生产里不会发生(查询在订阅之前调用),
所以必须造出来。

### 2. 注释改写

删掉那句 "NOT EQUIVALENT: Flux's control mutation ... turns RED (2 failures)",
连带删掉它上面那段现在已过时的 "The `prev?.kind` half is NOT covered by the tests today"
(有了新测试,这句就是假的)。

换成我实测的四个数字:

| 状态 | 实测 |
|---|---|
| guard 原样 | **21 passed** |
| M2(`kind = failure.kind`) | **1 red** —— 新测试 |
| 只删 `prev?.kind` 半边 | **1 red** —— 同一条 |
| Flux 的次序变异(M2 + 查询移到订阅后) | **1 red** —— 同一条 |

**最后一行是关键发现**:Flux 那个变异现在**能**杀掉测试了。它当时报 RED 2 是假的
(它自己的 suite 下 0 red),但方向没错 —— 那个次序确实暴露 guard,只是当时没有测试够得着它。
**现在够得着了。**

逻辑一行未动,`git diff` 确认只有注释变化。

---

## 变异验证(4 个,全红)

| 变异 | 结果 |
|---|---|
| M2 完整删 guard | **RED 1**(新测试) |
| M2b 只删 `prev?.kind` 半边 | **RED 1**(同一条) |
| M3 Flux 的次序变异(M2 + 查询后移) | **RED 1**(同一条) |
| M4 对照:新测试在未加 guard 的树上 | 见下 |

**关于"新测试是否真的咬住了产品代码"**:这个坑我在这轮已经犯过一次(派单
`disp-OwOSLvL4EbqoB6YhlBQHi` 里,测试手写了产品序列,变异全绿)。所以这次我核了:
把产品代码改回 M2,新测试确实 RED。它测的是产品代码,不是它自己的 mock。

---

## 派单判据逐条对

| 判据 | 状态 |
|---|---|
| A. 删掉/改写那句 "turns RED (2 failures)" | ✅ 删了,换成实测 |
| B. 不许删 guard | ✅ 逻辑一行未动,`git diff` 只有注释 |
| C. 把那个变异补成正式测试,并自己重写 | ✅ 重写了(不是照抄探针),M2 下 RED 1,与派单说的 "1 failed / 20 passed" 一致 |
| 注释里不许再出现没有被跑过的变异数字 | ✅ 四个数字全是我本轮跑的 |
| 注释必须能回答「这个 guard 凭什么留着」 | ✅ 见下 |

**注释现在的答案**:因为严重性必须由"什么坏了"决定,而不是"谁先报"。查询失败比订阅失败更严重,
所以订阅后到不能把 never-answered 降级成 stale。`failure.kind` 单独做不到这件事 ——
它读的是**当前**这次失败,那时已经是 stale 了。而那个次序(查询先失败、订阅后失败)
在生产里不会自然发生,所以有了 `holdReembedListen` 这个开关把它造出来;
没有那个开关,这个 guard 今天没有任何测试够得着。

---

## 门禁

- `pnpm vitest run src/App.supervisorSync.test.tsx` → **21 passed**(派单判据要求这条)
- `pnpm tsc --noEmit` → 干净
- `pnpm test` → **337 passed / 29 files**(改前 336,+1 新测试)
- `pnpm exec eslint src` → **143 problems(133 errors + 10 warnings),与 baseline 完全一致**;
  我这两个文件 16 个 error 全是既有 `no-explicit-any`,stash 对比确认改前也是 16
- `prettier` → 我改的两个文件干净(改前也干净)
- `pnpm build` → 通过

---

## 需要上报的

1. **派单说"改完自己开验收卡,派 Flux 验(或者派别人验,你定的)"。** 我的选择:派 Flux 验。
   理由:这个改动虽然只碰注释+测试,但它修正的是一条 Flux 自己发错的结论,而且新测试让 Flux
   原来的变异第一次能被杀。让 Flux 看到"你的方向对、数字错、现在有测试了"比让第三方验更有信息量。
   如果 Aegis 认为该派别人,我按裁决改。

   **已派**:`disp-PFvlPjNO8cbKeke9gtT2K`(priority 100),预声明的 verdict event id 是
   `ev-N8vSfa-3zKNWukPsfzcrZ`,payload 里附了交付说明路径、我的四个实测数字、以及"验出 fail
   就照实发 fail"的明确规则。

   **卡还没开,原因是工具拦的,不是漏工**:`request_acceptance` 要求 `flux_verdict_ref` 指向一个
   **已存在**的 pass verdict。我试了一次(用预声明的那个 id),被拒;隔了几分钟再查账本,
   `disp-x7QjIhw2ZsvokB_yidBj1` 名下仍然 0 条 verdict。**在 verdict 落库之前,这张卡开不出来,
   而且不该由我自己发一个去填** —— 那正是 Lyra 裁决要防的自我认证。
   **等 Flux 的 verdict 一到,我就开卡。** 这一条在这份交付里是未闭合项,不是已完成项。

2. **合并 track 又被拦了一次,和上次同一条**。`request_acceptance` 要求交付分支包含 base。
   我合并前核过 interval:30 个文件全是 `docs/`,不碰 `ui/src`/`src-tauri`,按 AGENTS.md
   "任何 ui/src/src-tauri 变更才需要合并"的判断本该不用合。**但工具不认这个判断,它只认
   `merge-base --is-ancestor`。** 所以实际规则比 AGENTS.md 写的严:开卡前必须让 base 成为
   祖先,与 interval 内容无关。合并后复跑门禁(337 passed / tsc 干净 / eslint 143 baseline)再开。

3. **Nimbus 那条错误归因值得单独记**:他说查询链多一个 `.then`。这个理由如果被下一个人采用,
   会推出错的结论(比如有人去删那个 `.then`)。已写进交付说明,并已入 memory。
4. **遗留**:`dto-to-v1.ts:159` 的 `isPinned: true` 硬编码仍未修(前几条派单已报)。
