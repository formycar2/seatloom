# 修 Flux 验绿指出的两处 — 交付说明

| Field | Value |
|---|---|
| dispatch | `disp-gjdWWBy949yMqAxi-tswd` |
| packet | `AEGIS-2026-10-22-mira-fix-flux-findings-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 实现提交 | `0e71598` — 标题「mira: 修 Flux 验绿指出的两处(重启位置被抢跑 / 落后数差 1)」,7 个文件。**绝对哈希,不用 `HEAD~n`**:写本说明的每个提交都会让相对引用后移一位,我为此连错三次,所以只写绝对值和标题。 |
| 同步判据 | 命令 + 当时真实输出 + 区间里的提交落在哪些目录。**「落后数 == 0」这个数字判据已作废** —— 见第二节
| 日期 | 2026-10-22 |

## 第一件:重启回到上次位置 — 修好了

### 改前是什么

三条缺陷叠在一起,Flux 的行号我逐条核过,全部属实:

1. `App.tsx:268` 的写入效果(`setActiveProjectId(activeProjectId)`)先跑,
   把 `useDataStore` 的启动默认值 `'p-1'` 落盘 —— 恢复效果(101–116)要读的值
   **先被覆盖掉了**。顺序上写入赢。
2. `useDataStore.ts:1614` 以测试专用 id `'p-1'`(SEED_PROJECTS_FOR_TESTS 第一行)
   开场。真实启动落在一个不在任何数据库里的项目上。
3. 于是位置**永久丢失**:盘上留下 `'p-1'`,下一次启动也从那里开始。

顺带查出同源缺陷第三处(派单没点名,但同一个坑):`InitDialog` 的
「初始化演示仓」按钮 `onInitialize={() => setActiveProject('p-1')}` —— 点一下
就把假项目塞进 activeProjectId,再被写盘。一并修掉了。

### 改后是什么

顺序从「靠巧合」变成「有保证」的三段契约(`ui/src/App.tsx`):

1. **启动记录只快照一次**:`const [persistedProjectId] = useState(() => useAppStore.getState().activeProjectId)`。
   在第一个效果之前取,组件自己再怎么写也动不了它。
2. **恢复等后端读完**:恢复效果门在 `backendHydrated` 上,读完才决定目的地。
   记录的项目不在后端返回里 → 落第一个真实项目;后端一个项目都没有 → `null`。
3. **写入等两件事**:`restoredPosition`(恢复已决定)和
   `loadedUIProjectId === activeProjectId`(标签已按新项目落定)。后者防的是
   上一个项目的已存标签在恢复的同一趟渲染里把目的地的标签写掉。

`useDataStore.ts:1614` 的启动默认值 `'p-1'` → `null`。真启动不再从测试种子开场。

### 怎么验的

新测试 `ui/src/App.restartRestore.test.tsx`,7 条,挂载真实 App,断言全部
**实测**:

| # | 断言 | 对应缺陷 |
|---|---|---|
| 1 | 后端读落地前,activeProjectId 仍是启动默认;读完才是记录的项目 | 竞态 |
| 2 | **任何一次**写入都不带 `'p-1'`(逐条看落盘值,不看最终值) | 缺陷 1 |
| 3 | 落在记录项目的**已存标签**上,不是另一个项目的 | 缺陷 3 |
| 4 | 记录的项目后端没返回 → 落第一个真实项目,记录被修好 | 已删项目 |
| 5 | 后端一个项目都没有 → `null`,不编造项目 | 空库 |
| 6 | 启动默认值不是任何 SEED_PROJECTS_FOR_TESTS 的 id | 缺陷 2 |
| 7 | 关掉重开(重挂载 + 从盘上重读),回到同一个项目**和同一个标签**,且中间没写过别的标签 | 完整闭环 |

**变异验证**(Flux 教的方法:改坏源码,看测试是否红)。六个变异,
五个被抓住,一个是等价变异:

| 变异 | 结果 |
|---|---|
| 恢复效果去掉 `backendHydrated` 门 | RED(3 条) |
| 写入效果去掉 `restoredPosition` 门 | RED(3 条) |
| 写入效果去掉 `loadedUIProjectId` 门 | RED(1 条) |
| 启动默认值改回 `'p-1'` | RED(1 条) |
| 整个写入效果删掉 | RED(全部) |
| 恢复效果的依赖数组改回去 | 全绿 —— 等价变异(zustand action 引用恒定),非守卫缺失 |

第 3 个变异一开始**没被抓到**:我的断言只看了最终落盘值。原始 bug 恰恰是
「最终值看起来对,但第一次写入就冲掉了记录」。改成逐条记录写入后才抓住。

全量:`pnpm typecheck` 干净、`pnpm test` **246 passed (20 files)**、
`pnpm build` 通过。lint 总数与改动前**持平**(145 处既有问题,我新增的文件
0 处;`App.tsx` 是 prettier 干净的,我的编辑一度破坏格式,已改回)。

### 附带修的两份既有测试(必须说明)

`SupervisorIntent.test.tsx`(6 条)和 `AcceptanceSection.test.tsx`(16 条)
在干净树上全过,改完默认值后挂了。原因:**它们隐式依赖那个缺陷默认值** ——
两个测试都不声明 activeProjectId,靠 `'p-1'` 非空才继续往下读(两个组件
都是 `if (!activeProjectId) 直接 return`)。

修法是让它们显式声明前置条件(各加一行
`useDataStore.setState({ activeProjectId: 'seatloom', ... })`),
不是把默认值改回去。改动前这两份文件已存在 prettier 不合规的行,我没扩大它。

## 第二件:分支同步 — 判据从「数字」改成「命令 + 范围」

> Aegis 在 `AEGIS-2026-10-22-mira-resync-and-fix-the-criterion-v1` 里把原来的
> 「落后数 == 0」作废了,换成:给命令 + 当时真实输出 + 要求那区间里不得有改动落在
> `ui/src` 或 `src-tauri` 下的提交。本节按新判据重写。旧的那版(我写「输出 0」)
> 保留在下面的「改前」里,因为它记录了我是怎么追着自己的尾巴跑的。

### 命令与当时真实输出

```
$ git merge --no-edit track/infra-foundation
Updating 6a83348..5a7943e
Fast-forward
 ...6-10-22-mira-resync-and-fix-the-criterion-v1.md | 81 ++++++++++++++++++++++
 1 file changed, 81 insertions(+)
 create mode 100644 docs/coordination/tasks/aegis/AEGIS-2026-10-22-mira-resync-and-fix-the-criterion-v1.md

$ git rev-list --count packet/mira/knowledge-panel..track/infra-foundation
0
```

### 区间里的提交,以及它们落在哪些目录

合并前,区间里有 2 个提交,都是 Aegis 侧的纯文档:

| 绝对哈希 | 标题 | 落在哪些目录 |
|---|---|---|
| `b8123d9` | aegis: Nimbus 豁免清单验绿 pass;派 V3 待决对象(更 2 处自己的错) | `docs/coordination/tasks/aegis/` 1 个 md(96 行新增) |
| `5a7943e` | aegis: 派 Mira 重同步(改判据)+ 接 Nimbus V3 待决对象并派验绿 | `docs/coordination/tasks/aegis/` 1 个 md(81 行新增) |

**`ui/src` 和 `src-tauri` 下:零个提交。** 合并后区间为空,所以 count 输出 0。

本轮同步共合并两次(第一次只并入 `b8123d9`,Aegis 随后又提了 `5a7943e`,
第二次才把它并进来)。两次并进来的都是纯文档。

### 关于 Aegis 说的「我为追数字连写 5 个提交」

属实,而且原因就在判据本身。Aegis 18 点往 track 提 `b8123d9`,晚于我的同步,
数字又从 0 变 1;我为了在说明里写对一个**会自己变的数字**,连着提交了 5 次
交付说明,把 `0e71598` 先后误写成 `4d4786d`、`fad70b0`。
**追一个会自己变的目标没有终点。** 换成看内容(那区间里有没有人改界面或 Rust)
之后,这一节不再有「追」的动作。

### 改前(记录我原来的写法,保留备查)

> | 我这次跑的命令 | `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` |
> | 输出 | `0` |
> | 并入的 track 提交 | `2aa35ad`、`6c90709`(147 行,全在 `scripts/`) |
>
> 两点确认:一、差的那 1 是 Aegis 16:00 提的 `6c90709`,晚于我 14:05 的同步
> 合并 `67fd267`,我的交付说明当时不可能测到它;二、我自报 `1f8c01b` 而实际头是
> `dbb51e7`,写的是比我交付的树更旧的树 —— 这正是新判据要防的事。

## 明确不做的,我没碰

- 三件已 pass 的实现(V5 编造数字、沙箱标识条、净化)未动。
- Nimbus 的豁免清单未动。
- `ui/src/app-v2/AppV2.tsx` 未删。
- 没合并、没自己开验收卡。

## 需上报

1. **两份既有测试隐式依赖缺陷默认值**。`SupervisorIntent.test.tsx` /
   `AcceptanceSection.test.tsx` 不声明 activeProjectId,靠 `'p-1'` 非空才读。
   这类「测试靠一个错值才绿」的模式,别的测试文件里可能还有。已按
   显式声明前置条件修,没有把默认值改回去。
2. **`InitDialog` 有同源第三处**(派单未点名):按钮点击即把 `'p-1'` 塞进
   activeProjectId 再被写盘。已改为重读后端项目列表。
3. 第 3 个变异(写入效果的 `loadedUIProjectId` 门)一开始没被我的测试抓到 ——
   教训记在这里:最终值正确的断言,抓不到「先写错再写对」的缺陷。

## 历史里多余的提交(如实记下,不重写)

`0e71598` 之后有一串只改本说明文字的提交(Aegis 数出来是 5 个)。它们全是我在追
自己的哈希留下的:每提交一次交付说明,`HEAD~1` 就后移一位,我连着把 `0e71598` 写成
`4d4786d`、又写成 `fad70b0`,三次都错。**写这一节的本身又让数量 +1** —— 所以这里
不写数字,要看有几个就跑:

```
git log --oneline 0e71598..HEAD
```

不 rebase 掉,是因为重写历史比多几个提交更危险,而且这些错误本身就是证据:
**交付说明不该写自己的哈希,写绝对哈希加标题。** 这次不再为本说明提交。

## 复现命令

```bash
cd ui
pnpm typecheck                                  # 干净
pnpm test                                       # 246 passed (20 files)
pnpm vitest run src/App.restartRestore.test.tsx # 7 passed
pnpm build                                      # 通过
cd ..

# 同步判据(新):合并 + 看区间里有没有落在 ui/src 或 src-tauri 的提交。
# 注意:这条命令的输出**会自己变** —— Aegis 在 track 上提交后它就不是 0。
# 判据不是这个数字,是区间里有没有界面/Rust 改动。
git merge --no-edit track/infra-foundation
git rev-list --count packet/mira/knowledge-panel..track/infra-foundation
git log --oneline packet/mira/knowledge-panel..track/infra-foundation
git log --name-only  packet/mira/knowledge-panel..track/infra-foundation | grep -E '^(ui/src|src-tauri)/' || echo '区间内无 ui/src 或 src-tauri 改动'
```

写本节时实测的最后一行输出:`区间内无 ui/src 或 src-tauri 改动`。
