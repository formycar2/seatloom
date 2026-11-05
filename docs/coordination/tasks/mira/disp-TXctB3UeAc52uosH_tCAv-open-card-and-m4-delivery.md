# disp-TXctB3UeAc52uosH_tCAv 交付说明 —— 开卡,并补掉 Flux 的 M-4

**派单**:`disp-TXctB3UeAc52uosH_tCAv`(Aegis 发)
**提交**:`d2fea84`(两个远端都已推)

---

## 1. 卡开出来了

```
requested acceptance acc-disp-faEWBt8kSGgv-6pJs_Rqw
  for disp-faEWBt8kSGgv-6pJs_Rqw
  branch packet/mira/knowledge-panel, base track/infra-foundation
  flux verdict ev-Dq2vTW8WjIU_hQufDDp2F
  → pending operator decision
```

### 开卡前的核验(没有只信派单)

| 项 | 实测 |
|---|---|
| verdict 存在 | ✅ `ev-Dq2vTW8WjIU_hQufDDp2F` |
| verdict = pass | ✅ |
| 谁发的 | `seat:seat-flux-001` —— **第三方**,不是我自己的 |
| `payload.dispatch_id` | `disp-faEWBt8kSGgv-6pJs_Rqw` —— **点名本派单** ✅ |
| 分支包含 base | 开卡前实测 behind 42,已合并推送 → behind 0 / SYNCED |

**第一次 `request_acceptance` 被拒**过一次,理由正是分支未包含 base —— 所以那次
合并不是可选的。`branch` 和 `base` 都显式写出,没有走默认值(派单点名了这一点:
改动前的代码硬编码了一个叫 `track` 的分支,而那个分支不存在)。

## 2. `how_to_verify` 写的是真实范围,不是派单标题

派单说:"如果卡片只写『restore 不许持久化』,操作员会以为这是个小改动就接受了,
不会知道分支上带着整份裁定。" 我按它说的写:

**第一步,行为(真机三步)**:
1. 完全退出重开 → 左边栏必须高亮「派单」
2. 会话内切到另一项目 → 落在那个项目自己记住的页
3. **再切回第一个项目 → 它记住的页必须还在**(标为关键)

**第二步,这一笔实际改了什么**:
- 写盘 effect **已整块删除**,不是收窄
- 只有一个 `chooseTab(tab)` wrapper
- `updateProjectUIState` 只在那一处调用
- 恢复(`:489`)仍是裸 `setActiveTab`,**不写盘**
- 点名两条曾在 track 上红、现在绿的测试

**拒绝条件**:冷启动不落「派单」,或切项目不再记住各自的页 —— 后者不许坏。

**precondition 里写明了 M-4**(见 §3)及其性质,不隐瞒。

## 3. M-4 我补了 —— 派单说它是下一件活,但它是同一条链

派单原文:"That is your next task, not a reason to withhold the card"。
**我先开了卡,然后补了它** —— 因为上一课是"存活的变异等于活着的缺陷",
而这条变异正是 Flux 亲手跑出来存活的。

### 先复现,没有只信报告

```
chooseTab 把 selectedObjectId / selectedObjectType 写成 null
→ 12 passed | 0 failed          ← 存活,与 Flux 报的一致
```

### 难点:这个变异在冷启动帧**不可观测**

冷启动时什么都没选中,写 null 与正确行为在磁盘上**完全一样**。
而这一套测试里所有后端列表都 mock 成空,能造出真选中项的入口几乎都走不通。

### 解法:用保存下来的记录本身当载具

```
给 FIRST_REAL_PROJECT 存一个 selectedObjectId: 'wi-saved-42'
→ 切到那个项目,restore 把选中项应用到屏幕上
→ 操作员再换一次页
→ 断言那次写盘把选中项带过去了
```

变异下写的是 null:

```
× chooseTab persists the SELECTED OBJECT alongside the tab — M-4
  → expected null to be 'wi-saved-42'
Tests  1 failed | 12 passed (13)          ← 红
```

恢复后 `13 passed (13)`。

### 一个实测出来的装配顺序

spy 必须装在 `mountApp()` **之前**。

App 渲染时从 store 捕获 `updateProjectUIState`,**之后装的 spy 对它不可见**。
我第一版装在 mount 之后,得到 `writes.length === 0` —— 测试绿了,但
**它不可能因为正确的理由失败**。改到 mount 之前才真的盯住。

这个事实写进了注释。

## 4. 门禁与推送

```
pnpm test        → 375 tests / 31 files
pnpm typecheck   → clean
pnpm exec eslint → 135 errors = baseline(无新增)
prettier         → clean
pnpm build       → 1.86s
```

**两个远端都推成功了** —— 包括一直失败的 origin(github):

```
87509f7..d2fea84  HEAD -> packet/mira/knowledge-panel   (gitlab)
87509f7..d2fea84  HEAD -> packet/mira/knowledge-panel   (origin)
```

这是多轮以来第一次两个都通。

## 5. 仍未闭环

- **真机验证** —— 卡上三步操作只有张先生在真机上做才算数。
- M-3(等价变异)Flux 已正确分类,不改。
- `disp-E06O6IAaWefF-BOW-CNJd` 那个更早的派单仍无 verdict 点名它,没有为它开卡。
