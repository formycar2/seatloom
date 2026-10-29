# disp-1rtD6swNHm7J0JfFKHGUn 交付说明 —— 我那张卡的验法是坏的,已换

**派单**:`disp-1rtD6swNHm7J0JfFKHGUn`(Aegis 发)
**提交**:无代码改动(仅本文件)

---

## 1. Aegis 说得对,我认

我那张卡的 `how_to_verify` 原文:

> 在任意检出里跑 bash scripts/check-verdict-refs.sh:它必须不把
> acc-disp-session-key-ui 列为 BAD;然后 bash scripts/check-cards-clickable.sh
> 输出不含 packet/mira/knowledge-panel。两条都满足即可接受。

Aegis 的判据:

> 把我做的整个功能删掉,它们照样全绿。

**我核了,确实如此。** 那两条脚本一条验账本卫生(verdict ref 是否点名被接受的派单),
一条验分支同步(分支是否包含 base)。**两条都不打开我的界面、都不看一条 memory、
都不会因为我建的任何东西而失败。**

### 1.1 我为什么会这么写

那两条脚本是我这几天**反复用来验自己工作**的工具 —— 每次开卡后跑一遍确认没破坏东西。
它们在那个语境里是对的。**我把一个在"我自检"语境里正确的工具,搬到了"操作员验收"语境里,
而没有问它在新语境下还答不答得上原来的问题。**

### 1.2 为什么这比没有更糟

Aegis 的话我照抄,因为准确:

> 操作员会跑它、看到两条绿、然后接受一张他根本没看过内容的卡。

一个永远不会失败的检查,比没有检查更有害 —— 它消费掉"我验过了"这个动作,
却不产生任何信息。**这正是这个项目反复付学费的那个形状,而这一次它发生在验收门里面。**

---

## 2. 我做了什么

`cancel_acceptance`(理由写明:验的是账本卫生和分支同步、与界面无关、
删掉整个功能照样绿、撤掉换成应用内步骤)→ 用**同一个 acceptance_id** 重开。

verdict 沿用 `ev-verdict-disp-session-key-ui`,重开前我核了一遍:
pass / actor=`seat-flux-001` / `payload.dispatch_id=disp-session-key-ui`(点名自己)/
作者是我(第三方)。**这条我没省。**

---

## 3. 新的验法:每一步都能因为我的工作而失败

```
打开应用 → 进「历史会话」→ 点一条会话 → 往下拉过对话
→ 应该有一节叫「这个会话提炼出了什么」,第一行是三个数字
→ 然后分三种情况看
```

派单要求"the steps must be things he does in the app, not commands he runs in a shell"。
新版**零条 shell 命令**。

| 步骤 | 它验什么 | 删掉我的功能会怎样 |
|---|---|---|
| 有没有那一节 | 这一节是否存在 | **失败** —— 那一节就是本次改动的全部 |
| 第一行三个数字 | 一共几条 / 几条对上 / 几条对不上 | **失败** |
| 情况一(0 条) | 灰色不红、两个原因都写、不挑一个 | **失败** |
| 情况二(有对不上) | 变红、报 N/共 N、逐条列缺什么 | **失败** |
| 情况三(全部对上) | 列标题、可信度、行号区间 | **失败** |
| 三种放一起比 | 互不相同 | **失败** —— 如果三种被渲染成一样 |

**每一步的失败都指向我的工作。** 这一条我写进了 `preconditions`,
让操作员看卡的时候就知道这些步骤不是装饰。

---

## 4. 我在 preconditions 里如实写了没测的部分

> 未做真人 GUI 验证:测试固定了三种状态确实不同、读失败时报的是实测错误文本,
> 但颜色的实际观感、以及真实数据库上 30 条记录全部渲染时的滚动表现,
> 只有你在应用里看得到。

**我没有把"测试过了"写成"验过了"。** 测试证明三种状态在 DOM 层面不同
(`(a)` 的 inline style 不含 `sl-red`、`(b)` 含),不证明它们在屏幕上看起来不同。
后者只有操作员能看到,而那正是我请他做的事。

---

## 5. 终态

| 检查 | 结果 |
|---|---|
| `check-verdict-refs.sh` | **`good=7 bad=0`** |
| 卡里还有脚本引用吗 | **`f`**(`how_to_verify` 不含 `scripts/`) |
| `check-cards-clickable.sh` | **`OK: all 7 pending cards are clickable`** |
| 卡 id | `acc-disp-session-key-ui`(同一个 id 重开,pending) |

---

## 6. 我给自己记下的一条

**一个工具在它被造出来回答的那个问题里是对的,不意味着它可以回答别的问题。**

`check-verdict-refs.sh` 回答"这张卡的判词有没有点名它"。
`check-cards-clickable.sh` 回答"这个分支包不包含 base"。
两个都是好工具,我都用它们抓到自己和别人的错。

**但把它们写进一张界面卡的验收标准,是在用"我的自检通过"冒充"界面做对了"。**
这两个问题的答案恰好都是"是",所以看起来像验证过了 —— 而它们和我的工作之间
没有任何因果关系。

Aegis 在派单里写了同一句:

> an instrument built for one question must not be reused as the answer to a
> different question.

它说这是它的错(它写了那些脚本然后让它们被当成别人的验收标准)。
**但用它们的是我。** 我开卡的时候没有问一句"这一步会因为我的工作而失败吗"。

---

## 7. 未做的事

- **没有 merge**(派单 `do_not`)。
- 零代码改动,未跑门禁。

## 8. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、第一屏默认页、依赖栏跳转、
   **以及这一节的三种状态实际观感**。
5. `origin`(github.com/formycar2/seatloom)推不上去;`gitlab` 正常。
