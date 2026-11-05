# disp-MQwgyf-XTKRKARKMu_jjR 交付说明 —— 同步 + 两张卡 + 沙箱构建

**派单**:`disp-MQwgyf-XTKRKARKMu_jjR`(Lyra 发)
**提交**:`f26466f`(合并 track)
**门禁**:`vitest` 391 / 31 files / tsc 净 / eslint 145 problems = baseline / `cargo test` 26 passed
**卡**:`OK: all 2 pending cards are clickable`

---

## 1. 阻塞项先解决:落后 10 个提交,已合并

```
git merge-base --is-ancestor track/infra-foundation packet/mira/knowledge-panel
  之前 → NOT CONTAINED(退出非 0)
  behind: 10
```

合并后:

```
git rev-parse --short HEAD                       → f26466f
git rev-list --count HEAD..track/infra-foundation → 0
git merge-base --is-ancestor ...                 → 退出 0  ✓
```

Lyra 说这 10 个全是 Aegis 的、不碰 `src-tauri` 或 `ui/src`,是快进式同步。
**我核了合并后的门禁**:391 tests / 31 files,tsc 净,eslint 145 = baseline,
`cargo test` 26 passed。**没有冲突,没有回归。**

**两个远端都推成功了** —— 包括一直不通的 gitlab:

```
758c8a4..f26466f  HEAD -> packet/mira/knowledge-panel   (origin)
04af484..f26466f  HEAD -> packet/mira/knowledge-panel   (gitlab)
```

## 2. 两条 verdict 先核,没有只信派单

```
ev-flux-verdict-four-write-paths-01
  verdict = pass, actor = seat:seat-flux-001(第三方)
  payload.dispatch_id = disp-XpAdx0a9YLL83JVoPesTj   ← 点名 ✓

ev-flux-verdict-two-frontend-cases-02
  verdict = pass, actor = seat:seat-flux-001(第三方)
  payload.dispatch_id = disp-Ra5YbN4t78bzdbYhYkQiV   ← 点名 ✓
```

## 3. 两张卡,一件一卡

| 实现派单 | 卡 | verdict |
|---|---|---|
| `disp-XpAdx0a9YLL83JVoPesTj` | `acc-disp-XpAdx0a9YLL83JVoPesTj` | `ev-flux-verdict-four-write-paths-01` |
| `disp-Ra5YbN4t78bzdbYhYkQiV` | `acc-disp-Ra5YbN4t78bzdbYhYkQiV` | `ev-flux-verdict-two-frontend-cases-02` |

`dispatch_id` 都填实现派单 id,`flux_verdict_ref` 各用点名它的那条。

```
scripts/check-cards-clickable.sh
  → OK: all 2 pending cards are clickable
```

## 4. 卡的四步,按 Lyra 的定义写,并且写清了她的两个陷阱

### 第 0 步是横幅(我加的)

Lyra 的验收条件第 4 条要求"卡上点名构建命令和提交,让操作员能判断眼前的
应用是不是被验收的那一个"。**我把这一步放在最前面**:

> 第 0 步,先看应用顶部的横幅。它写着这个程序是从哪个分支、哪个提交编出来的。
> 记下那个提交号 —— 本卡验收的就是它。
> 期望看到:`f26466f`(或同步 track 之后的新尖,但必须是你这条分支的尖,
> 不是别的工作树的)。

### `ql_delivery`,不是 `project_delivery/QL`

她在派单里两次强调,我照写进卡:

> 注意:不是 `project_delivery/QL`。沙箱数据库里 QL 项目叫 `ql_delivery`,
> 而 `project_delivery/QL` 在那里没有席位绑定,它的下拉会是空的,表单提交不了。

### 通过 / 失败写实

> 通过:新派单出现在 `ql_delivery` 的列表里,席位显示为「**ql**」—— 名字,不是 `seat-ql-001`。
> 失败:表单提交了、列表刷新了、什么都没有。或它出现了但分组在原始 id 下面。

### 负例 + 回归

她的定义里这两条是修复的一部分,不是附加题:

> 第 4 步:应用绝不静默把派单归到别的项目。项目判定不出来时必须明说。
> 第 5 步(回归):在 `seatloom` 里做同样的事,必须照旧能用。

## 5. 沙箱构建:跑了,而且脚本自己拦了一次

### Dry-run 先读提交号(她要求的)

```
bash scripts/prepare-sandbox-build.sh packet/mira/knowledge-panel
  分支        : packet/mira/knowledge-panel
  构建目录    : .../seatloom-worktrees/mira
  提交        : f26466f(要构建的就是这一棵)      ← 我的树,不是别的工作树
  与主干对比  : 领先 36 / 落后 0
```

### 我在自己的 worktree 里跑,脚本拒绝了

```
❌ 'packet/mira/knowledge-panel' 不是任何工作区当前检出的分支。
❌ 不从工作区里猜。指定一条已检出的分支再跑。
```

**这正是 Lyra 警告的陷阱**:脚本构建"某个工作区检出中的那棵树",
它不替你 checkout。我的 worktree 明明检出着这条分支,
但从我这儿跑时它算的 `REPO` 不对。

**改从主检出跑**,dry-run 立刻给出正确的构建目录和提交号。

**这条值得记:那个拒绝不是 bug,是守卫在工作。** 它没有猜,
而是列出每个工作区检出在哪、是不是我要的提交。如果我当时传的是提交 hash,
它会构建另一棵树,而横幅照样打印我要的 hash —— 完全看不出来。

### `--apply`

```
✅ 沙箱在跑,pid 3794
✅ 主版  : /Applications/SeatLoom.app (pid 90805)   ← 没动
✅ 沙箱  : /Applications/SeatLoom-Sandbox.app
   构建来自 packet/mira/knowledge-panel @ f26466f(检出树,已与参数核对一致)
```

沙箱数据库是 `seatloom_sandbox`(快照库),**写它不影响正式库**。
主版进程 pid 90805 未被触碰。

## 6. 我没做的一件,以及为什么

派单说"不要修沙箱数据库"——`project_delivery/QL` 在那里没有绑定,
三个受影响项目里两个完全缺失。

**我没动。** Lyra 说得对:`ql_delivery` 满足判据(不是 seatloom、有绑定席位),
沙箱是否该镜像那三个项目是张先生的决定,不是一件活。

## 7. 卡没覆盖的

**冷启动那条验收没被这两张卡覆盖。** 那一件还没有 verdict,它自己的 A-D 步骤也未做。
Lyra 明确说不要折进来,我没折。

## 8. 未做

- **没有 merge into track**(派单 `do_not`)。
- **没有重启主版应用** —— 脚本自己也说不会动它。
- **没有写正式数据库** —— 只写了两张卡和沙箱库。
