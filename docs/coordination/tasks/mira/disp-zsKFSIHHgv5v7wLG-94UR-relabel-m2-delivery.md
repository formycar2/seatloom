# 改一处用词:M2 不是「等价变异」(`disp-zsKFSIHHgv5v7wLG-94UR`)

| Field | Value |
|---|---|
| dispatch | `disp-zsKFSIHHgv5v7wLG-94UR` |
| packet | `AEGIS-2026-10-27-mira-relabel-m2-and-note-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | 已改。交付说明和代码注释里 M2 都不再叫「等价变异」,改成「今天测试没覆盖」,并写上 Flux 的对照和结论。**guard 没删,测试没改。** 另外发现一件比用词更值得报的事,见第四节 |

## 一、改了什么

### `ui/src/App.tsx`(只改注释,代码行原样不动)

`prev?.kind` 那段注释从「defensive rather than load-bearing」改写为:

- **NOT COVERED**:查询排在订阅之前,所以它的 rejection 先落地,
  `failure.kind` 单独就给得出同样的答案 —— M2 因此通过本套件所有测试。
- **NOT EQUIVALENT**:Flux 的对照变异(M2 + 把查询移到订阅之后)**RED 2**。
  那个顺序下查询的 rejection 落在最后,只有 `prev?.kind` 能保住形状。
  **所以 guard 在重排下是 load-bearing 的,今天的调用顺序只是把它藏住了。**
- 保留它的理由不变:规则是关于「什么失败了」,不是关于调用顺序。

代码行 `prev?.kind === 'never-answered' || failure.kind === 'never-answered'`
**一字未改**(`git diff` 核对过,只有注释块在变)。

### 交付说明

`disp-J9ExyK_2-3ajGuXjBvlKj-both-fail-f2-delivery.md` 的变异验证那节:

- 标题「3 死 1 等价」→「3 死 1 未被覆盖」
- M2 那行「等价变异」→「今天测试没覆盖」
- 正文重写:写清「当前不可达」和「等价」的区别、Flux 的对照、以及
  **为什么这两个词会把读者送去不同的地方**
- 结论行也同步改了(它在表格上面,是第一眼看到的地方)

## 二、为什么这两个词要紧(我原来没想透的)

我说「等价变异」时,想表达的是"这个变异杀不死,所以我不能声称它被杀死"。
但这个词还有一个我没打算表达的含义:

> 「等价」= 这段代码是多余的,删掉它行为不变。

而这个含义在这里是**错的**。一个读到注释的人如果信了它,合理动作就是删掉
`prev?.kind` —— 而那会在有人重排 effect 时静默重新引入"后写的赢"。

「今天测试没覆盖」表达的是另一件事,而且指向一个动作:
这里有个没被钉住的行为,该补测试或者写清它为什么在。

**一个词的差别,是"该删"和"该补"的差别。**

## 三、Flux 对我那个按钮判断的承认(记在这里,但不用我改)

Flux 的原话(派单转述 + 我在账本原文 `ev-hfLMM6Gyn3z7V6qIatMlJ` q3 核过):

> RIGHT, and my P3 reasoning was the weaker half. Her third reason is the
> strongest: hiding a usable action and hiding a fact are the same error in
> opposite directions. **I conflated "this action probably won't work" with
> "this action shouldn't be offered"** — the first is a prediction about outcome,
> the second is a design decision about agency and **not mine to make on the
> operator's behalf**.

他还加了一条我们都没提的测量:**连续失败三次会列出三条理由**,
并说这是可接受的诚实历史,记下来免得将​​来成为意外。

这条不用我做什么,记在交付说明里备查。

## 四、一件比用词更值得报的事(我复现 Flux 对照时发现的)

派单要我改词,并要求「写上 Flux 那个对照」。我去复现那个对照,结果是:

```
$ M2 单独(kind = failure.kind,查询仍在订阅之前)
     Tests  20 passed (20)          ← 我的测试看不见它

$ M2 + 把 askSupervisorWindowStatus() 移到两个订阅之后
     Tests  20 passed (20)          ← 我的测试还是看不见它
```

**第二次输出是 20 passed,不是 Flux 的 RED 2。**

这不是我和他谁对的问题,而是说明了**两件不同的事**:

1. **他的 RED 2 是用他自己的探针测到的,不是用我的测试套。**
   所以他的结论(guard 在重排下 load-bearing)仍然成立 —— 他换了个能看到
   那个顺序的测量工具。
2. **我的测试套缺的不是一个用例,是那一整类排序场景。**
   我给"顺序独立性"写的那条测试,在 M2 下**也**仍然通过 —— 原因是 Flux
   诊断出来的:查询自己的 rejection 就带 `kind: 'never-answered'`,
   靠它自己就把形状升上去了,`prev?.kind` 根本没参与。
   所以那条测试证明的是"升级路径能工作",**不是**"guard 是让它工作的原因"。
   **它的断言比它的名字弱。**

**我没改测试**(派单明令禁止),但把这个事实写进了交付说明,
没有让它埋在"我和 Flux 结果不一致"这种含糊的话底下。

如果 Aegis 认为该补,那该是一个新派单:把查询的 reject 延迟到订阅之后,
让 `prev?.kind` 成为唯一能保住形状的东西 —— 那才是能杀死 M2 的测试。

## 五、明确不做的,我没碰

- **没有删那个 guard**(派单明令禁止;Flux 也说保留是对的)
- **没有改任何测试**(`git diff` 对 `App.supervisorSync.test.tsx` 和
  `SupervisorSyncLost.tsx` 都是空)
- **没有顺手改别的注释** —— 只改了那一段
- 没有合并、没有自己开验收卡

## 六、顺带:Aegis 说不用我做什么的那条,我确认了

派单里说 Flux 复现了 Aegis 三次没做出有效 M1,并明说
「the fault was in the mutations, not the description」。
我在账本原文 q5 核过,确实如此:

> RED 5 — 3 in her suite, 2 in mine — by collapsing reasons, kind and
> subscriptionsDown together.

所以**我的 M1 描述没问题**。这条不用改任何东西,记在这里备查。

## 复现命令

```bash
# 确认 guard 的代码行没变(只有注释变了):
git diff ui/src/App.tsx | grep -E '^[+-]' | grep -v '^[+-][+-]'
#   → 应只看到注释块的增删,看不到 prev?.kind === 'never-answered' 这一行

# 确认测试和组件零改动:
git diff --stat ui/src/App.supervisorSync.test.tsx ui/src/components/SupervisorSyncLost.tsx
#   → 空

# 确认注释里的两个词:
rg -n 'NOT COVERED|NOT EQUIVALENT' ui/src/App.tsx

# 全部门(与基线一致):
cd ui && pnpm test                                    # 300 passed
pnpm typecheck                                        # 干净
pnpm exec eslint src 2>&1 | grep '✖'                 # 143 problems,与基线一致
pnpm exec prettier --check src/App.tsx                # 干净
```

## 操作者怎么自己验

这一单**没有界面变化** —— 改的是注释和文档用词,代码行为一字未改。
`pnpm test` 300 passed 与改动前相同就是全部证据。
