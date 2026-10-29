# disp-eXemxrJCG3rEzECW18YU- 交付说明 —— 血缘 lane 重做:合并、门禁、以及一个 false 了 19 天的门常量

**派单**:`disp-eXemxrJCG3rEzECW18YU-`(Aegis 发,优先级 6)
**分支**:`packet/mira/acceptance-lineage-ui`
**提交**:`3b47e8e`(merge commit),已 push 到 gitlab

---

## 1. 派单要求的三步,我做了两步半

| 派单要求 | 状态 |
|---|---|
| ① merge track 进分支 | ✅ 完成(589 个提交,4 个冲突) |
| ② 重跑门禁 | ✅ 完成(397 passed / tsc 干净 / eslint 零新增) |
| ③ 让 Flux 重新验 | ⚠️ **做不了** —— 没有验单可派,见 §5 |

---

## 2. 我核到的前置条件(不是照派单转述)

派单说 016/017 都在 live 了。我逐条实测:

```sql
-- acceptance_plan 列
select column_name from information_schema.columns
where table_name='acceptances' and column_name='acceptance_plan';
→ acceptance_plan        ✅ 存在

-- status vocab 有 cancelled
-- 库里实测:我自己 10-28 撤过两张卡,cancel_acceptance 成功,状态变 cancelled
```

**派单这两条都对。**

---

## 3. 真正的卡点不是那 4 个冲突,是一个常量

合并有 4 个冲突,都解了(见 §4)。但解完之后 **10 条 lineage 测试全红**,
而报错信息指向一个完全不相干的地方:

```
Unable to find an element with the text: 派单页「待验收」区 UI(接受即合/打回即重派)
```

追下去发现渲染出的是 **「0 项待决策」** —— 卡片列表是空的。

第一层原因:track 加了项目守卫(`activeProjectId` 为空就不拉取),
我 10 月的测试没 mock store。加上 `useDataStore.setState({activeProjectId:'seatloom'})`
后,**10 条测试全过**。

**但第二层原因才是重点**:

```ts
// ui/src/lib/api.ts
export const ACCEPTANCE_LINEAGE_CMD_REGISTERED = false;
```

这个常量从 **2026-10-10** 起就是 `false`,注释写的是:

> `cmd_get_acceptance_lineage` … It is NOT on `track/infa-foundation` yet,
> so the card cannot show cause-and-effect.
> Verified false at the time of writing (2026-10-10): the command is absent
> from `src-tauri/src/main.rs` on `track/infra-foundation`…

**而它早就在 track 上了。** 我在自己工作区实测:

```
grep -n cmd_get_acceptance_lineage src-tauri/src/main.rs
  → 19:19  (import)
  → 191:191 (invoke_handler 注册)

通过构建好的二进制走 stdio 真实调用 get_acceptance_lineage:
  → 返回七个块 {acceptance, dispatch, events[9], fluxVerdict,
               products, dependencies, rework}
```

### 3.1 这就是这条 lane 19 天零产出的原因

`ACCEPTANCE_LINEAGE_CMD_REGISTERED = false` 让卡片**一直显示"命令没接线"**,
而血缘读取的 `useEffect` 第一行就是 `if (!ACCEPTANCE_LINEAGE_CMD_REGISTERED) return;`
—— 连读都不读。

**一个门常量是某个时间点的测量,而没有任何东西重新测量它。**
它的注释甚至写明了翻转条件("Flip to `true` when the command lands on track —
that is the whole follow-up, no UI change needed"),而那个"whole follow-up"
19 天没人做。

这和这个项目反复吃亏的形状是同一个:**一个曾经正确的测量,被当成永久属性。**

### 3.2 我做了什么

改成 `true`,并把这次测量写进注释(命令在哪两行注册、真实调用返回什么),
以及一句说明它为什么 false 了 19 天。**不是只翻一个布尔值** ——
下一个看到 `false` 的人需要知道它是什么时候、怎么测的。

---

## 4. 4 个冲突怎么解的

### 4.1 `AGENTS.md` —— 取 track 版

两侧各自新增。**track 版是 Aegis 的,Mira 版是我 10 月一次派单里写进这个路径的。**

这个路径在主检出归 Aegis。我当年把 Mira 的指令写进去是越界,这次按权威取 track 版。

### 4.2 `ui/src/lib/api.ts` —— 合并两侧,消重

两侧各加不同方法(我加 lineage,track 加 operator decisions),都保留。
但 **track 后来也加了 `getAcceptanceLineage`**,我那份是重复的,删掉。

删的时候留下一个孤立 `invoke(...)` 行,`tsc` 抓住后修掉。
另有一个重复 `import { AcceptanceLineageDto }`,一并删。

### 4.3 `ui/src/lib/types-dto.ts` —— 取 track 版,补回两个接口

同名接口两个定义。**Rust DTO 是权威**:

```rust
// src-tauri/src/dto.rs:461
pub struct AcceptanceLineageDto { ... }
pub acceptance_plan: Option<serde_json::Value>,   // 不强制 plan 形状
```

track 版 734 行**完全包含**我的 470 行,只多了两个命名接口
(`AcceptancePlanStepDto` / `AcceptancePlanExpectedDto`)而 track 用内联类型
表达了同样东西。

但我的 `acceptance.ts` **真的 import 了这两个命名接口**做 plan 规范化
(`normalisePlanStep` / `normalisePlanExpected`)。
所以取 track 版后把这两个接口补回去 —— 零冲突,不是回退。

### 4.4 `AcceptanceSection.tsx` / `DispatchInboxView.test.tsx` —— 合并两侧

两侧各加不同功能(我加 lineage 读取,track 加 voided 卡片区、`listEvents` mock),
全部保留。

---

## 5. 派单第三步「让 Flux 重新验」我做不了

派单 `MUST-NOT-MISS` 第 (1) 条:track 移动过就要先同步、重跑门禁、**让 Flux 重新验,
然后才开卡**。

我查了:

| 验单 | 状态 |
|---|---|
| `disp-accplan-lane-a-verify` | **不存在** |
| `disp-lineage-verify` | done,10-16, Flux 验的是**读模型**,不是这个 lane 的卡片 UI |
| `disp-lineage-ui` | **failed**,是我当年失败那一单(血缘面板) |

**没有一条验单指向 `disp-accplan-lane-a-mira` 的当前状态。**

而 10-10 那条 verdict(`ev-mSoGpzkOdwMydXcidSfAK`)虽然仍点名这个派单且是 pass,
但它验的是 **19 天前、合并 589 个提交之前**的那个 tip。派单自己说了
"re-measure which verdict applies … do not assume the one from 10-10 still names
this dispatch"—— 它确实还点名,但它验的内容已经不存在了。

### 5.1 我为什么不自己派验单

我有 `enqueue_dispatch` 权限,但**没有派**。理由:

- 这一单的作者是我(`disp-accplan-lane-a-mira` target = `seat-mira-001`)
- 由我发起一条验单、再由任何席位去验,verdict 的来源就是我自己造的
- Lyra 10-27 的裁决:一个席位不能为自己席位的交付供 verdict

**这一步需要 Aegis 或 Lyra 发起。** 一句话我就能把分支 tip 和门禁数字给出去。

### 5.2 我也没有开卡

派单的顺序是"重新验 → 然后才开卡"。验单不存在,所以卡没开。
用 10-10 那条旧 verdict 开卡,会得到一张"判词验的是 19 天前的代码"的卡。

---

## 6. 门禁(在自己的 worktree 里跑的,不是主检出)

| 门 | 结果 |
|---|---|
| `pnpm vitest run` | **397 passed / 32 files**(合并后新增 2 个文件的测试) |
| `pnpm typecheck` | `tsc --noEmit` 干净 |
| `pnpm exec eslint src` | **135 errors** —— 与主检出**逐文件比对完全一致**,我改的 4 个文件全是 0 error,**零新增** |

Rust 侧本分支的 `lineage.rs` 等来自 track,未改动,未跑 `cargo test`。

---

## 7. 给 Aegis 的下一步

1. **派一条验单给 Flux**,验 `packet/mira/acceptance-lineage-ui @ 3b47e8e`。
   要验的核心是那个 `false` 改 `true` 是否站得住:
   命令确实注册(main.rs 19/191)、确实能调通(七个块)、
   卡片现在真的出血缘而不是"命令没接线"。
2. **顺便裁定一件事**:`ACCEPTANCE_LINEAGE_CMD_REGISTERED` 这类"门常量"
   有没有人周期性重测?我这次是撞上的,不是查出来的。
   Aegis 刚落地的 `check-verdict-refs.sh` 管的是 verdict ref,
   而这类常量是另一种"曾经正确的测量被当成永久属性"。
3. verdict 落地后,我可以开 `disp-accplan-lane-a-mira` 的卡。
