# 项目选择器交互体验(`disp-gvVtOC1Z_uvVz8mq-lc45`)— 五条全改,另发现一个数据缺陷

| Field | Value |
|---|---|
| dispatch | `disp-gvVtOC1Z_uvVz8mq-lc45` |
| packet | `AEGIS-2026-10-27-project-switcher-ux-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | 派单五条**全改**。另发现一个派单没列的第 6 个问题(`isPinned` 硬编码 `true`),**没改**,已上报。14 条新测试,7 个变异全杀死 |

## 一、五条逐条:改了没有,为什么

### 问题 1:placeholder 承诺了不存在的快捷键 —— **改了,删掉那句话**

核实时我确认 Aegis 说的是事实:

- `App.tsx` 的 Ctrl+K → 开关 Supervisor IM,`preventDefault()`
- `useGlobalShortcuts.ts` 的 Ctrl+K → Supervisor Command Bar,`preventDefault()`
- **没有任何一处打开 ProjectSwitcher**

所以那句话在邀请用户按一个做不到的组合键,按了出现别的东西。
**我选择删掉,不选择实现。**

理由:派单边界第 2 条说「要么做到,要么删掉」,而"做到"要改
App.tsx 那个已经既定下来的 L1 交互(Supervisor IM)。为一个搜索框的便利
去动 L1 的主入口,这个权衡不该由我在一个体验派单里做。
**placeholder 现在是「搜索项目」,不含任何快捷键承诺。**

### 问题 2:「最近访问」列显示创建时间 —— **改了,整列删掉**

先核数据源,确认派单的判断:

- `src-tauri/src/dto.rs:25` 的 `ProjectDto` 只有 id/name/created_at/两个预算/home_path
- 数据库 `projects` 表**没有** last_accessed 列(我 grep 过 schema 和 core)
- `dto-to-v1.ts:160` 是 `last_accessed: d.createdAt` —— 拿创建时间冒充访问时间
- `useDataStore.ts:1624` 确实在内存里更新 `last_accessed`,但 mapper 每次
  hydrate 都用 createdAt 覆盖,所以连内存那次更新也会被抹掉

**所以没有真数据可接。** 派单边界第 1 条说:要么接真数据(要后端加字段 + 落库,
那是另派单),要么不显示那一列。我选**不显示**。

派单原话:「不许换个说法继续显示 createdAt —— 那正是你今天修掉的那个
`~/` 假路径的形状。」我同意,而且这正是我不接受"改个标签"方案的原因:
把 `last_accessed` 改名成 `created_at` 显示,用户看到的还是一个他不知道
该怎么理解的时间戳。**一个界面上的数字,如果不能回答用户心里的问题,
就不该占那个位置。**

顺带保留的是**真的**那两个数(`条待处理`、`个活跃会话`)—— 它们来自
`projectData`,是每项目真实读出来的。

### 问题 3:分组标题和排序不符 —— **改了,标题改成只说做到的那件**

标题从「最近访问与置顶项目」改成「**置顶项目**」。

理由:排序确实只按 `isPinned`。访问排序不只需要 last_accessed 真数据,
还需要一个"最近"的定义 —— 两者现在都没有。标题承诺两件只做一件,
和问题 1 是同一个病(界面说了做不到的事)。

### 问题 4:没有键盘导航 —— **改了**

新增:`ArrowDown`/`ArrowUp` 移动高亮、`Enter` 选中、`Escape` 关闭、
`Home`/`End` 跳首尾。

几个我定的细节,以及为什么:

| 决定 | 理由 |
|---|---|
| **打开时不高亮任何行** | 用户刚打完搜索词,预选第 0 行会让 Enter 选中一个他没看过的项目 |
| **高亮在两端夹住,不循环** | 过滤后的列表里,从最后一行跳到第一行会丢位置,而且不容易察觉 |
| **Enter 只在有高亮时才选** | 见下面「我自己的两个假绿测试」 |
| **监听挂在 input 上(冒泡),不给行设 tabIndex** | 一个入口、没有 focus trap 可错;行不是 focusable,所以没有 tab 顺序要推理 |
| **高亮行 `scrollIntoView({block:'nearest'})`** | 420px 的列表,方向键移出可视区就等于选了个看不见的东西 |

派单边界第 3 条说「不要新增交互范式」—— 过滤列表框的方向键/回车/Esc
是既有组件的常规做法,不是新设计。

### 问题 5:搜索只匹配 name —— **改了**

`filter` 现在匹配 name **或** path。这样 `sf-code` 能找出它下面所有项目,
`project_delivery` 同理。空查询(和纯空格)显示全部。

## 二、第 6 个问题:派单没列,我核问题时发现的(没改)

`src/lib/dto-to-v1.ts:159`:

```ts
isPinned: true,        // ← 硬编码,每个项目都是 true
```

后果,比我原来想的严重:

1. **39 个项目全部显示成置顶**(Pin 图标全是实心)
2. **问题 3 的排序因此完全没作用** —— 全部相等,`sort` 是恒等变换
3. **Pin 按钮是个假控件** —— 点了会改内存态(`useDataStore.ts:1646`),
   但下次 hydrate 被 mapper 覆盖回 `true`

也就是说:**派单的问题 3 我说「改了」,其实只改对了标题;排序那一半
在当前数据下不可能对。** 这个必须说清,不能让我看起来修好了。

**我没改它。** 理由:真正的修法需要一个持久化的 pin 字段(后端 + schema + 迁移),
那是 Nimbus/Aegis 的 lane,不是我在一个交互派单里能顺手做的。
已在交付说明和事件里上报,请派单。

## 三、怎么验的

### 测试 `src/components/ProjectSwitcher.test.tsx`,14 条(新文件)

| 覆盖 | 断言 |
|---|---|
| placeholder | 不含 `Ctrl+K`(派单问题 1) |
| 访问时间列 | 行上**没有任何** `MM-DD HH:mm` 格式的时间戳;且不含「最近访问」字样(问题 2) |
| 真实计数保留 | 4 个项目 → 4 个「条待处理」+ 4 个「个活跃会话」 |
| 分组标题 | 是「置顶项目」,不是「最近访问与置顶项目」(问题 3) |
| 键盘导航 | 下键移动 + 回车选中**正确的行**;无高亮时回车不选也不抛;Escape 关闭;下端夹住不循环;Home/End 跳首尾(问题 4) |
| 搜索 | 匹配 name;匹配 path 里的父目录;空/空格查询显示全部;无结果时有提示且方向键/回车不抛(问题 5) |

**「断言格式而不是中文字面量」** 是问题 2 那个测试的关键:
`formatShortDateTimeZh` 输出 `MM-DD HH:mm`,**没有任何文字**。
我第一版断言 `not.toContain('最后访问')` —— 那是个永远绿的断言,
因为代码里从来没有这几个字。改成断言**格式**后才咬得住(见 M2)。

### 变异验证(7 个,全杀死)

| # | 变异 | 结果 |
|---|---|---|
| M1 | placeholder 把 `Ctrl+K` 加回去 | RED 11 |
| M2 | 把访问时间列加回来(用 `formatShortDateTimeZh`) | RED 1 |
| M3 | 标题改回「最近访问与置顶项目」 | RED 2 |
| M4 | 搜索去掉 path 匹配 | RED 1 |
| M5 | 删掉方向键导航 | RED 2 |
| M6 | 去掉 Enter 的 `highlighted < 0` 守卫 | 第一版**全绿(假绿!)** → 改测试后 RED 1 |
| M7 | 空结果时 Enter 不防护(越界) | 第一版**全绿(假绿!)** → 改测试后 RED 1 |

### M6/M7 是我的测试假绿,不是变异无效

这两个变异第一版**全绿**,而我明明知道守卫在工作(单独探针验证过 0 次调用)。
查下去发现:

> `filteredProjects[-1]` 是 `undefined`,`chosen.id` 抛 TypeError,
> **React 把 handler 里的异常吞掉了**,所以 `onSelect` 同样没被调用 ——
> 守卫生效和守卫崩溃,在我的断言下看起来一模一样。

**一个分不清「修好了」和「崩了」的测试,比没有测试更坏。**
修法:两条测试都加 `window.addEventListener('error', ...)` 断言**没有异常**,
再加一条「选择器还开着」(Enter 没做事不该顺带关掉它)。

改完之后 M6/M7 都被杀死。

这是我这段时间第二次踩同一个形状(前一次是 shift 快捷键那单的 IME 测试
target 选错)。共同点:**断言只观测了一个副作用,而那个副作用在
「正确路径」和「静默失败路径」下取值相同。**

## 四、全量门(与基线对比)

```
改动后: Test Files 25 passed (25)   Tests 314 passed (314)   Errors 0
基线  : Test Files 24 passed (24)   Tests 300 passed (300)   Errors 0
```

`pnpm typecheck` 干净、eslint 全仓 **143 problems 与基线逐字相同**、
我的两个文件 **0 error**、prettier 干净(第一版不合规,`--write` 后
逐行核对 diff 只动了我新增的代码)。

## 五、明确不做的,我没碰

- **没有改 Supervisor IM 或 Command Bar 的行为**(派单明令禁止)——
  问题 1 的正解因此是删掉那句话,不是实现快捷键
- **没有改账本、没有改后端 schema**
- **没有改 `dto-to-v1.ts` 的 `isPinned: true`** —— 那是第 6 个问题,
  需要持久化字段,超出本单且是别人的 lane
- **没有改视觉风格**(颜色/圆角/字号全未动)
- 没有合并、没有自己开验收卡

## 六、需要上报

1. **第 6 个问题(`isPinned: true` 硬编码)请派单。** 它让问题 3 的排序那一半
   在当前数据下不可能对,而且 Pin 按钮是假控件。修法需要一个持久化 pin 字段。
2. **问题 2 的真数据也没有。** 后端 `ProjectDto` 和 `projects` 表都没有
   last_accessed。如果产品要"最近访问"这一列,那是后端加字段 + 落库 + 迁移,
   也请派单。在那之前我按派单边界选了不显示。
3. 我给两条 Enter 测试加了 error 监听断言 —— 这不是派单要求的,
   但不加就是假绿测试。如果 Aegis 认为这是"改测试让变异红",请看 M6/M7
   两节:我加的是**崩溃检测**,不是把断言改松。

## 复现命令

```bash
cd ui
pnpm vitest run src/components/ProjectSwitcher.test.tsx   # 14 passed
pnpm test                                                 # 314 passed(基线 300)
pnpm typecheck                                            # 干净
pnpm exec eslint src 2>&1 | grep '✖'                     # 143 problems,与基线一致
pnpm build                                                # 通过

# 五个问题的变异(全部 RED):
#   placeholder 加回 Ctrl+K              → RED 11
#   访问时间列加回来                      → RED 1
#   标题改回「最近访问与置顶项目」          → RED 2
#   搜索去掉 path 匹配                    → RED 1
#   删掉方向键导航                        → RED 2
#   去掉 Enter 的 highlighted<0 守卫      → RED 1(加 error 断言后)
#   空结果 Enter 不防护                   → RED 1(加 error 断言后)

# 与基线对比:
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash pop
```

## 操作者怎么自己验

1. 点顶部项目名打开选择器 → 搜索框占位符只有「搜索项目」,没有 Ctrl+K
2. 每一行只有:项目名、路径、`N 条待处理 · N 个活跃会话` —— **没有时间戳**
3. 分组标题是「置顶项目」
4. 手放在键盘上:↓↑ 移动高亮、回车选中、Esc 关闭、Home/End 跳首尾
5. 搜索框输入 `sf-code` 或 `project_delivery` → 命中该目录下的项目
6. 输入不存在的名字 → 显示「没有匹配…」,按方向键/回车不出错、选择器不关
