# disp-OQyilLqRjR1fDSWsWE-5n 交付说明 —— 四件产品发现

**派单**:`disp-OQyilLqRjR1fDSWsWE-5n`(Lyra 发)
**提交**:`da20424`(Finding 1)、`a9a00f7`(Finding 4)、`e700d34`(Finding 2)
**门禁**:375 tests / 31 files,tsc 净,eslint 135 errors = baseline,build 1.74s

---

## Finding 1 ✅ 完成 —— 我的测试在强制一个谎言

### 我先核了前提,没有只信报告

`hydrateProjectData` 填充 store 用的是:

```
api.listSeats / listWorkitems / listHandoffs / listArtifacts
    / listSessions / listEvents / listDocuments
```

**每一条都是真实 Postgres 读**,`inboxItems` 也由同一批数据构建。
**所谓"store 支撑的层"不存在。** 测试建基其上的区别是假的。

### 三处编辑

1. **`provenance.ts`** —— `LIVE_TAB_BACKING` 加入
   `dashboard / inbox / workitems / seats / artifacts / playbook` 为 `database`。
2. **`provenance-honesty.test.ts`** —— 枚举测试扩展到全部八个。
3. **`provenance-honesty.test.ts` + `provenance.test.ts`** ——
   重写那条 store-backed 测试(它测的区别不存在),并把 `seed` 分支文案改对。

### 为什么这是四件里最要紧的

一个错徽标只花操作员一瞥。**这个花他一场争吵** —— 修好它会看到三条红测试,
每条都读作"你弄坏了 provenance 逻辑"而不是"你让徽标诚实了",
而阻力最小的路就是回退、保持绿。**10-17 原则落地了,枚举从来没有,正是这个机制。**

### seed 文案改了什么

原文案有两个错:

> "These rows are read from PostgreSQL for real, but they are seed fixtures...
> The live mirror is under 实时席位."

1. **"确实从 PostgreSQL 读"** —— 对不上它现在只服务未接线的界面。
2. **把操作员指向别处找实时数据** —— 而那些数据就在这一页。

改成:这个视图还没接实时读,没有声明数据库支撑,你看到的是占位符;
**不是数据库不可达(那会显示 MOCK),是这里没接。**

### 变异

| 变异 | 结果 |
|---|---|
| 从 `LIVE_TAB_BACKING` 删掉 `sessions`/`dispatches` | **2 红**(与 Lyra 报的一致) |
| `chooseTab` 丢 `selectedObjectId`(done_definition 点名) | **1 红**,M-4 测试抓住 |

---

## Finding 4 ✅ 完成 —— 静默改数据 → 告知操作员

### 我先确认了 fallback 已经存在

`restore effect` 的 `!ids.has(persistedProjectId)` 分支**已经有** fallback,
而且有测试。**缺的只有告知。**

### 为什么告知不是锦上添花

`hydrateProjectData` 用真实数据新建 `inboxItems` 并赋值,替换掉 seed fixtures,
**所以空列表忠实地反映了这次读**。产品从来不问"这个空列表来自的项目存在吗"。

操作员的读法是"我的活都干完了",真实状态是"你在看一个数据库里没有的项目",
**而活在 seatloom**(7 工作项 / 3 handoffs / 全部 3 张待验收卡)。

**这是同一个故障往下一层:修好数据,藏起事件。**

### 实现

fallback 时记录 `{missing, switchedTo}`,渲染一条通知,**同时点名两个 id** ——
只说"你上次所在的项目已不存在"而不说名字,操作员无法判断那是不是他关心的那个。

文案直接写出因果:

> 如果这里显示的工作项比你预期的少,原因是这个,不是工作已经做完。

### 变异(两个都红)

| 变异 | 结果 |
|---|---|
| 撤掉 `setDroppedProject`(静默 fallback) | **1 红** —— 原有 fallback 断言仍然过,所以没这条断言纠正就不可见 |
| 告知不点名缺失的项目 | **1 红** —— `expected ... to contain 'deleted-project'` |

---

## Finding 2 ⚠️ 实现完成,但**没有测试**,按纪律不报完成

### 实现

`NavRail` 读所有项目、对卡取并集,计数挂到「派单」项。每 15 秒轮询,
读失败不计假数。

**刻意不按 `activeProjectId` 过滤**:卡在 seatloom,他的活动项目是别的,
按项目过滤的徽标会显示 0 而三张在等。**遍历所有项目取并集是徽标唯一不撒谎的算法。**

### 为什么没测试 —— 如实说

我写了测试,它在这个 suite 里跑不过去:

```
PROBE-BADGE pending=0 ids=[] per=[]
```

`restartRestore` 的 `beforeEach` 把 `useDataStore.setState({projects: []})`
覆盖掉,而 `hydrateFromBackend` 的 `set` 依赖 Tauri 检测与挂载顺序 ——
徽标 effect 跑到时 `projects` 还是空。我等到 `projects.length > 0` 仍然是 0。

**这是这个 suite 的初始化顺序,不是产品缺陷。** 但我不能证明徽标真的会出现,
所以:**我把测试撤了,没有留一条红的,也没有改断言让它绿。**

**没有测试覆盖的实现,按我自己的纪律不该报完成。** 这条算"已实现、未验证"。

面板自身那个按项目读的 bug 我**没有修** —— `api.listAcceptances(projectId)`
是 Nimbus 的围栏(我查了 Rust 侧:`project_id: Option<String>` 但
`require_project_id` 强制非空,所以 api.ts 的签名比后端更严)。已报告,未越界。

---

## Finding 3 —— 不是代码改动,是开卡流程

派单说这件"归你在开卡时做":跑 `prepare-verification-build.sh` 再让卡 pending。

**我已开的两张卡(`acc-disp-faEWBt8kSGgv-6pJs_Rqw`、
`acc-disp-E06O6IAaWefF-BOW-CNJd`)没有跑这个脚本。**

原因是实测的:安装的应用比修复旧约 43 小时,所以卡上第一步
"退出重开,tab 必须是派单"**在安装的应用上会失败** —— 写盘 effect 还在。

**这是我开卡时的疏漏,不是不知道规则。** 两张卡现在都 pending,我应该先准备
构建再开。已报告给 Lyra/Aegis,不掩饰。

---

## 推送状态

| 远端 | 状态 |
|---|---|
| gitlab | `da20424`、`a9a00f7` 已推;`e700d34` 推时 SSH 22 端口又开始超时 |
| origin | 一直无权限 |

提交都在本地完整,工作树干净。
