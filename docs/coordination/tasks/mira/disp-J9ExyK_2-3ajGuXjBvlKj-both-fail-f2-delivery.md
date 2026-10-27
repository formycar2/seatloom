# 修 Flux 的 F2:两条同时失败时,更糟的那个被藏了(`disp-J9ExyK_2-3ajGuXjBvlKj`)

| Field | Value |
|---|---|
| dispatch | `disp-J9ExyK_2-3ajGuXjBvlKj` |
| packet | `AEGIS-2026-10-27-mira-fix-flux-finding-f2-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | 已修。两条失败路径汇到一个**累计记录**:形状取最糟的,理由全部保留。第四种组合(查询+订阅都失败)现在是红 + 重试 + **两份错误原文都在**。8 条新测试,4 个变异(3 死,1 个今天测试没覆盖 —— 这个词后来改过,见变异验证那节) |

## 一、先核 Flux 的 F2(不凭转述)

读了 `ev-SNtpi-dbYOEjmkdz4fK7e`。F2 原文与派单一致,而且他是**实测四种组合**的。
我在自己 worktree 里复现了那四种,结果和他一致(见第二节)。

顺带确认了他在 verdict 里纠正 Aegis 的那件事:他说试了五种合并方式
**每次都是 RED 2,从来不是 3**,能给 RED 3 的是恢复 `.catch(() => {})`。
**我的交付说明把 M1 写成「贴回 `.catch(() => {})`」是对的**,
Aegis 在派单里把它描述成「合并两条失败路径」是标签错。我照着自己的正确描述
做的,动作对。

## 二、四种组合:修前实测 vs 修后实测

我用一个探针驱动真实 App、注入 api seam,四种组合各跑一遍。
**修前的输出和 Flux 的表逐行一致**,包括最后一行的问题:

| 组合 | 修前 | 修后 |
|---|---|---|
| 查询 OK + 订阅 OK | 无条 | 无条 |
| 查询 OK + 订阅失败 | `stale` 黄,无重试,订阅原文在 | 同 |
| 查询失败 + 订阅 OK | `never-answered` 红,有重试,查询原文在 | 同 |
| **查询失败 + 订阅也失败** | **`never-answered`,但订阅原文丢了、没说两条都断** | **`never-answered` 红,有重试,两份原文都在,明说「实时推送也接不上」** |

**我复现出的问题和 Flux 的略有差别,而且更糟一点**:他测到的是
「只有 stale,查询原文不见」;我这边查询原文还在(因为我的查询 reject 落在
更早的 microtask),但**订阅的原文被查询的 reporter 覆盖丢了**,
而且界面**没说两条都断**。方向完全一致 —— 单槽位 + 后写的赢,
只是哪份原文被吞取决于时序。这不妨碍结论,反而多一条证据:
**这个 bug 的表现时序依赖,谁被吞是碰运气。**

修后第四行:

```
kind=never-answered  retry=Y  queryErr=Y  subErr=Y  bothSaid=Y
```

## 三、派单四个问题的答案

### Q1:两个都失败时显示什么形状?

**红(`never-answered`)。** 理由:形状必须表达"有没有答案可以依靠",
两个都坏的时候**没有** —— 从来没答过。不能因为订阅也坏了就降成黄的,
那等于用次要故障掩盖主要故障。

### Q2:重试按钮该出现吗?

**该。** 查询那次读是可以重试的,订阅坏不影响"再问一次"这个动作本身
(它问的是同一件事)。Flux 定 P3 时说"重试大概也会失败" —— 我同意这个预测,
但**按钮该在**,因为:

1. 它是唯一指向"可能修好"的动作,藏起来等于连试都不让试;
2. 它失败了也不会更糟(条还在,原文还在,只是没变);
3. **藏一个能用的动作,和藏一个事实,是同一个错误的不同方向。**

### Q3:查询的错误原文还看得见吗?

**看得见,而且两份都在。** 这是 F2 里我最在意的一条 —— 派单原话
「它是操作者唯一能拿去找人的东西」。现在 `reasons` 是数组,
按上报顺序全列出来,一条不丢。

### Q4:文案要说清"从来没有答案"吗?

**说了,而且加了新的一句。** 两个都坏时多一段:

> **而且实时推送也接不上** —— 也就是说就算重启后立刻再拖一次面板,
> 这个界面也还是不知道。两件事一起坏,下面两条错误原文都列出来,一条都不要丢。

这段是 Q1/Q2 的落地:它告诉操作者"这不是一件事坏,是两件",
所以重启后如果还这样,他知道该预期什么,而不是以为重启没生效。

## 四、改了什么

### `ui/src/App.tsx`

`supervisorSyncLost` 从单槽位改成累计记录:

```ts
const [supervisorSyncLost, setSupervisorSyncLost] = useState<{
  kind: 'never-answered' | 'stale';
  reasons: string[];          // 每条理由,按上报顺序,不合并
  subscriptionsDown: boolean; // 订阅是否也断了
} | null>(null);
```

两个 reporter 都汇到一个 `reportSupervisorFailure`:

```ts
const reportSupervisorFailure = useCallback((failure) => {
  setSupervisorSyncLost((prev) => {
    const reasons = prev ? [...prev.reasons, failure.reason] : [failure.reason];
    // 查询失败是更糟的那个,所以它决定形状,不管谁先上报
    const kind = prev?.kind === 'never-answered' || failure.kind === 'never-answered'
      ? 'never-answered' : 'stale';
    const subscriptionsDown = (prev?.subscriptionsDown ?? false) || failure.kind === 'stale';
    return { kind, reasons, subscriptionsDown };
  });
}, []);
```

**重试成功时只清查询那一半**:

```ts
setSupervisorSyncLost((prev) => {
  if (!prev || prev.kind !== 'never-answered') return prev;
  if (!prev.subscriptionsDown) return null;
  return {                                  // 查询修好了,订阅还没
    kind: 'stale',
    reasons: prev.reasons.filter((r) => !r.startsWith('查询监督面板位置失败')),
    subscriptionsDown: true,
  };
});
```

这一条是 **F2 的镜像**:F2 是"故障因为到得晚被藏",
这里是"故障因为一次无关的成功被藏"。同一个病,另一个方向。
我把这条也测了(见 M4)。

### `ui/src/components/SupervisorSyncLost.tsx`

`reason: string` → `reasons: string[]` + `subscriptionsDown?: boolean`;
错误原文从单个 `<div>` 改成 `<ul><li>`,两条各自可读;
`bothDown` 时多渲染那段「实时推送也接不上」。

## 五、怎么验的

### 测试 `src/App.supervisorSync.test.tsx`,20 条(原 12 + 新 8)

新增的覆盖:

| 断言 | 覆盖 |
|---|---|
| 四种组合逐一(4 条) | Flux 那张表的每一行,包括第 4 行的全部六个断言 |
| 第 4 行:`kind=never-answered`、有重试、**两份原文都在**、说「实时推送也接不上」、**不**说「启动时查到的那一次」 | F2 本体 |
| 订阅先失败、查询后失败 → 形状从 stale **升级**成 never-answered | 顺序独立性 |
| 查询 OK + 订阅失败 → **不**出现「查询监督面板位置失败」 | 不能反过来撒谎 |
| 重试成功但订阅仍断 → 降级成 stale,订阅原文留着,无重试按钮 | F2 的镜像 |
| 重试成功且一切正常 → 条整个消失 | 重试是诚实的 |

「订阅先失败"那条是这一单最值得说的测试:我用 mock 把查询的 promise **挂住**,
让订阅先 reject,再让查询 reject —— 这样才能把"严重度由什么决定"和
"严重度由谁先到决定"分开。不然两个规则在正常时序下给出同样的答案。

### 变异验证(4 个,3 死 1 未被覆盖)

| # | 变异 | 结果 |
|---|---|---|
| M1 | 整个 reporter 换成单槽位(F2 本身) | **RED 3** |
| M2 | `kind = failure.kind`(严重度按到达顺序) | **全绿 —— 今天测试没覆盖,见下** |
| M3 | 只保留第一个 reason(丢后来者的原文) | RED 3 |
| M4 | 重试成功时清掉整条(隐藏没修好的部分) | RED 1 |

**M2 全绿。我最初把它叫作「等价变异」,那个词是错的。**

事实层面我说对了:`kind = failure.kind` 和现在的写法,在当前代码结构下行为相同 ——
`askSupervisorWindowStatus()` 在 effect 里排在两个订阅**之前**(第 296 行 vs
订阅在其后),所以查询的 rejection 总是先落地,`prev?.kind` 那个保护项走不到。

**但「当前不可达」和「等价」是两件事,我把它们混成了一件事。**
Flux 指出了区别(`ev-hfLMM6Gyn3z7V6qIatMlJ`,q4),并给了决定性对照:

> **M2 加上「把查询移到两个订阅之后」→ RED 2。**

也就是说:把调用顺序反过来,查询的 rejection 就落在**最后**,那时只有
`prev?.kind` 能保住形状是 `never-answered`,否则订阅会把它降级成 `stale`。
**所以那个 guard 在重排下是 load-bearing 的,不是死代码。**

我在自己 worktree 里复现了这个对照,但**结果和他不一致**,见下面那节。

**为什么这两个词要紧**:读的人看到「等价变异」,合理推论是
「这段代码是多余的,删掉它行为不变」—— 而那是**错的**。
看到「今天测试没覆盖」,推论是「这里有个没被钉住的行为」—— 那是对的,
而且指向一个动作:要么补测试,要么至少在注释里写清它为什么在那儿。

**我那个顺序测试也没测到我以为它测的东西**(Flux 的诊断,我已复现确认):
它让订阅先 reject、查询后 reject,在 M2 下**仍然通过** —— 因为查询自己的
rejection 就带 `kind: 'never-answered'`,靠它自己就把形状升上去了,
`prev?.kind` 根本没参与。所以那条测试证明的是"升级路径能工作",
**不是**"guard 是让它工作的原因"。

**我没改测试**(派单明令禁止),但把这条如实记在这里:
顺序测试的断言比它的名字弱,这是它和 M2 共同的盲区。

### M2 对照的复现(我在自己 worktree 里跑的)

```
$ M2 单独(kind = failure.kind,查询仍在订阅之前)
     Tests  20 passed (20)          ← 我的测试看不见它

$ M2 + 把 askSupervisorWindowStatus() 移到两个订阅之后
     Tests  20 passed (20)          ← 我的测试还是看不见它
```

第二次输出和 Flux 的 RED 2 不同。**这说明我的测试缺的不是一个用例,
而是那一整类排序场景** —— Flux 用的是他自己的探针,不是我的测试套。
所以对照本身成立(他的 RED 2 是他的探针测到的),但我不能声称
「我的测试套在这个对照下会红」。这两件事都要说清。

### 全量门(与基线对比)

```
改动后: Test Files 24 passed (24)   Tests 300 passed (300)   Errors 0
基线  : Test Files 24 passed (24)   Tests 292 passed (292)   Errors 0
```

`pnpm typecheck` 干净、`pnpm build` 通过。
eslint 全仓 **143 problems,与基线逐字相同** —— 我中途引入过 1 个 warning
(`reportSupervisorFailure` 没进依赖数组),用加进依赖数组的方式修掉了,
不是 `eslint-disable`。prettier 三个文件干净,且 `git diff` 核对过
prettier 只动了我自己改的区域。

## 六、明确不做的,我没碰

- **没有让"后写的赢"继续存在**(派单明令禁止)
- **没有丢任何错误原文**
- **没有改 ⌘K / 内嵌面板渲染逻辑**
- 没有顺手改别的 `.catch`
- 没有合并、没有自己开验收卡

## 七、需要上报

1. **Flux 说的部署问题仍在**:他两次 verdict 都记了跑着的二进制是 00:03 的,
   这些修复的文本不在它的 blob 里(他称之为 `eb47ec8` 那一课的第四/五次复发)。
   本单不涉及(我不重建二进制),但操作者至今看不到任何黄条/红条。
2. **M2 暴露的一个结构事实**:查询调用排在两个订阅之前,所以它的 rejection
   总是先到。这不是缺陷(结果正确),但意味着 `reportSupervisorFailure` 里
   那个 `prev?.kind` 保护项当前不可达。我已注释说明,不建议删。

## 复现命令

```bash
cd ui
pnpm vitest run src/App.supervisorSync.test.tsx   # 20 passed
pnpm test                                         # 300 passed(基线 292)
pnpm typecheck                                    # 干净
pnpm exec eslint src 2>&1 | grep '✖'              # 143 problems,与基线一致
pnpm build                                        # 通过

# 四种组合的实测输出(修后):
#   1 query OK  + sub OK   | kind=(none)          retry=N queryErr=N subErr=N bothSaid=N
#   2 query OK  + sub fail | kind=stale           retry=N queryErr=N subErr=Y bothSaid=N
#   3 query fail + sub OK  | kind=never-answered  retry=Y queryErr=Y subErr=N bothSaid=N
#   4 query fail + sub fail| kind=never-answered  retry=Y queryErr=Y subErr=Y bothSaid=Y

# 证明 F2 修法会红:把 reportSupervisorFailure 整体换成单槽位版本
#   setSupervisorSyncLost({ kind: failure.kind, reasons: [failure.reason],
#                           subscriptionsDown: failure.kind === 'stale' });
#   → 3 failed
# 证明原文不被丢:把 reasons 改成 `prev ? prev.reasons : [failure.reason]`
#   → 3 failed
# 证明镜像那条也钉住:把重试成功时的 setSupervisorSyncLost((prev)=>...) 换成 null
#   → 1 failed

# 与基线对比:
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash -u && pnpm exec eslint src 2>&1 | grep '✖'
git stash pop
```

## 操作者怎么自己验

1. 正常启动 → 不该看到任何条
2. 两种/三种失败日常点不出来(需要后端拒绝);可信的办法是看代码路径,
   或等重建后由 Aegis 决定怎么造这个场景
3. 若真看到红条:它**必须**同时有「再问一次」按钮和至少一条错误原文;
   如果只有一条原文而你知道两件事都坏了,那是 F2 复发,请报给 Aegis
