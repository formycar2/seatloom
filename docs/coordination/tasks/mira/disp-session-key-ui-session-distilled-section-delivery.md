# disp-session-key-ui 交付说明 —— 会话详情加「这个会话提炼出了什么」

**派单**:`disp-session-key-ui`(Aegis 发,优先级 8)
**契约**:`LYRA-2026-10-16-session-work-bridge-contract-v1.md` §11.2 slice 4
**分支**:`packet/mira/knowledge-panel`
**提交**:`252c5aa`(另:`726b691` 是进行中的 seam,已含在内)

---

## 1. 做了什么

在**已有的**会话详情屏(`HistorySessionDetailView`)里加了一节,不改导航、
不加新屏、不新增加载态。派单原文是「加一节」,我没有借机重组这个屏。

挂载位置在对话列表**之后、之外**:一个「什么都没提炼出来」的会话是
关于这个会话的真实回答,它必须在对话本身读不出来的时候也够得着 ——
所以没有嵌套进 turns 分支里。

## 2. 三种状态,以及它们为什么不能看起来一样

读模型存在的全部意义就是区分「这里没有」和「这里有但对不上」。
一个把两者都渲染成空列表的界面,把读模型唯一承载的信息毁掉了。

| 状态 | 判据 | 视觉 | 内容 |
|---|---|---|---|
| (a) | `numbers.total === 0` | 中性色 Explanation | `notes[0]` 原文 —— 读模型明说这个空计数有两个不同原因且拒绝合并,这里两个都显示,不挑一个 |
| (b) | `numbers.unresolved > 0` | **红框红字** | `N / total 条记录对不上原文` + 每条记录的 `missing[]` 原文 |
| (c) | 其余 | 正常列表 | kind(译)/ confidence(原样)/ 行号区间 / joinState / resolvedBy / 可点的 sourcePath |

外加一条**恒显**的健康行:`total · resolved · unresolved · 原文消息数 ·
drift 未消解数`。在 (a) 也显示 —— 唯一一个「什么都没提炼」是真实答案的
状态反而最需要证据,把它藏起来意味着那个状态什么都证明不了。

还有第四个分支:`total > 0` 但 `records` 数组是空的。这是读模型在和自己
的两个数字打架,不是空会话 —— 单独一个红/中性标题说清「读模型自己说一共
30 条,但一条也没返回」,不挑一个顺眼的说法。

## 3. 读模型怎么接的

```ts
sessionWork: (harness, id) => invoke('cmd_session_work', { harness, id })
```

**走 `invoke` 直接调,没有经过 `api.ts`。** 派单明令 `api.ts` 和
`types-dto.ts` 是 Nimbus 的 fence 不许改,而这个席位对这个读模型不加任何
形状,所以没有东西需要加到 fence 里。

命令返回读模型**自己的 JSON 文本**,在这里 parse,不重新造型。第二套摘要形状
是两个模型漂移的地方,而这个屏的存在恰恰是为了显示它们在哪儿**没有**对齐。

`parseSessionWork` 校验 `numbers.total` 是数字,否则抛错。读模型不返回这三个
数字的时候,这个屏必须说出来,而不是从数组长度算一个看起来一样的值。

## 4. fixture 是实测的,不是编的

| 状态 | 怎么量到的 |
|---|---|
| (a) `total:0` | `codex` / `019cb3ff-9525-73f0-8f2c-0712ba95103b`,对 live 真实调用 |
| (b) `unresolved:1` | scratch 库 `probe_mira_p0`,把记录 `mem-VLaeSW9S50aK7v7wCvNF6` 的 `l2_file_path` 指向 `raw/claude-code/does-not-exist-9999.jsonl`(archived_sessions 里没有) |
| (c) `30/30/0` | `claude-code` / `b0100cff-2426-4786-b165-45827c9e2713`,对 live 真实调用 |

字段名和 note 文本逐字复制,读模型漂移会以测试失败的形式出现,而不是一节
静默的空面板。

## 5. 变异测试:9 个,全红

| # | 变异 | 结果 |
|---|---|---|
| M1 | 把 (a) 和 (b) 合并成同一个空态 | **RED**(2 条) |
| M2 | 藏掉 (b) 的红色计数 | **RED**(2 条) |
| M3 | 读失败时静默渲染成空列表 | **RED**(1 条) |
| M4 | 抹掉 `numbers.total` 校验 | **RED**(1 条) |
| M5 | 不渲染每条记录的 `missing` | **RED**(1 条) |
| M6 | 健康行不报 drift | **RED**(1 条) |
| M7 | sourcePath 不可点 | **RED**(1 条) |
| M8 | 抹掉「读模型自相矛盾」那个区分 | **RED**(1 条) |
| M9 | (b) 改用中性样式(和 (a) 一样) | **RED**(1 条) |

变异一律改源码、跑测试、从**备份文件**还原(不用 `git checkout`,这个文件在
本提交前有未提交内容)。

## 6. 过程中自己抓到的两个测试缺陷

### 6.1 每条测试外层多 render 一次

7 条新测试全红,报 `Found multiple elements with the title: .../seatloom`。
原因是我在每个测试里写了 `render(...)` **然后**调 `openSeatloomSession(loader)`,
而 helper 内部自己也会 render —— 同一文档里叠了两棵面板树,每个 title 都匹配两次。

**是我的 bug,不是产品的。** 删掉外层那 6 行 render 后 26 条立刻转绿。

### 6.2 一条测试看错了重点,变异在它眼皮下保持绿色

M8 第一次测出来是**绿的**。查下来:那条测试只断言了正文
(`/读模型自己说一共 30 条/`),而我那个变异换掉的是**标题**。正文还写着 30,
所以断言照样满足。

补上对标题的断言后才红。这一条值得记:**变异测试的价值一半在变异,
一半在测试有没有盯着对的地方。** 一个只断言次要文本的测试,会在主要
区分被抹掉的时候鼓掌。

补断言时又踩了一个:标题原本写成 JSX 属性带插值
(`title="记录数是 {numbers.total},..."`),渲染成多个文本节点,`getByText`
匹配不到整串。改成单个模板字符串后用 `getByText('记录数是 30,但一条也没返回')`
精确断言。

## 7. 门禁

| 门 | 结果 |
|---|---|
| `pnpm test` | **355 passed / 30 files**(基线 348 / 30,新增 7 条) |
| `pnpm typecheck` | 干净 |
| `pnpm exec eslint src` | **144 problems(134 errors + 10 warnings)** —— 与基线一致,**未新增** |
| `prettier --check` | 已格式化,复查干净 |
| `pnpm build` | 通过(1.90s,仅 chunk 大小提示,非错误) |

Rust 侧本改动零触及,未跑。

## 8. 没测的 / 需要真人确认的

**这条必须写明:整个交互没有在真实 GUI 里点过。**

- 没有真的用鼠标点过那个 `↗` 按钮去看文件是否打开(只验了 loader 被以正确
  路径调用)。
- 没有在真实窗口里看过三种颜色的实际观感(jsdom 不解析 CSS 变量,
  视觉区分那条测试断言的是「(a) 的 inline style 不含 `sl-red`、(b) 含」,
  这是**接近**而非**等于**「看起来不一样」)。
- 没有验证过真实数据库上 30 条记录全部渲染时的滚动/高度表现(测试 fixture 只 2 条)。

## 9. 派单外发现的两件事(未擅自修)

### 9.1 那两个 `-live` 门事件按契约不该由我发

派单链上有人说「其中两个是你的,只差你发那两个 ArtifactCreated」。我核了
契约 §11.1:两个 `-live` 门的 emitter 都是 **Nimbus**。我已发报告说明,并
交代我自己 10-14 交付说明里把它们当成已合规模门的错话。
详见 `docs/coordination/tasks/mira/MIRA-2026-10-28-who-emits-the-two-live-gates.md`。

### 9.2 「四件已完成的活开验收卡」——那四件一件都不能开

派单说验绿依据都在 `ReviewVerdictIssued`。我逐件查了,找到 6 件(不是 4 件),
每件都挂着一条 `verdict=pass`。**但每条 verdict 的 `payload.dispatch_id`
指向的都是另一件派单**,不是它自己:

- 按 `ref_type=dispatch + ref_id=<我的派单>` 查 → 每件都「有 verdict」
- 按同一条事件的 `payload.dispatch_id` 对 → 全部对不上

那些被指向的派单**全都已经有卡了**。我没有开任何一张。
详见 `docs/coordination/tasks/mira/MIRA-2026-10-28-the-four-cards-have-no-verdict-naming-them.md`。

## 10. 遗留(跨派单,仍未修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error
   (`SessionTurnEvent` import 未使用)。Aegis 补回的测试文件第一次参与全仓 lint
   时暴露的,不是我的改动。删那一行即可,零风险,但那是 track 上别人的文件。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点(`useGlobalShortcuts.ts:70`、
   `App.tsx:366` 都缺 `isTypingContext` 守卫)—— 需要产品裁定 + 实现。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、以及本节的真实观感。
