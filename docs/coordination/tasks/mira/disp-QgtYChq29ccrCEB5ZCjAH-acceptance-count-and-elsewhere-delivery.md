# disp-QgtYChq29ccrCEB5ZCjAH 交付说明 —— 待验收卡计数 + 空状态说清卡在别处

**派单**:`disp-QgtYChq29ccrCEB5ZCjAH`(Lyra 发)
**提交**:`7e76568`
**门禁**:378 tests / 31 files,tsc 净,eslint 135 errors = baseline,build 1.77s

---

## 1. Lyra 量的那第三格,我先自己核了

她说我保存的活动项目是 `p-1`,一个测试夹具。我核:

```
ui/src/stores/useDataStore.ts:98
export const SEED_PROJECTS_FOR_TESTS: Project[] = [
  { id: 'p-1', name: 'SeatLoom 协调核心', ... }   ← 第一条
```

```
ui/src/stores/useDataStore.ts:1848-1854
const nextActive = projects.find((p) => p.id === storedActive)?.id ?? projects[0]?.id ?? null;
```

**`p-1` 是 seed 常量第一条;hydrate 会丢掉后端不返回的 id 并退回 `projects[0]`。**
她的诊断成立。

**这意味着我上一单的表述不准确**:我说的是"面板按 activeProjectId 过滤、而他的活跃项目零张卡"。
真实情况是**那个项目根本不存在** —— 读不是过滤到了错的项目,是指向了一个不存在的项目。
她量的那一格确实比我的更糟。

## 2. 做了两件

### 2.1 NavRail 待验收卡计数(上一单实现但没测试,这单补上)

读**所有项目**并对卡取并集,计数挂到「派单」项。

**刻意不按 `activeProjectId` 过滤** —— 这正是 Lyra 量的第三格:卡在 seatloom,
他的活动项目是一个后端不返回的 id。按项目过滤的徽标会显示 0 而三张在等。

**依赖 `backendHydrated` 而不是空 dep 数组**。实测空数组时 effect 在水合前跑,
读到空项目列表,徽标停在 0 —— 和"没有卡在等"长得一模一样。轮询 15 秒后能纠,
**但一个在头一刻钟说谎的徽标,正是在操作员打开应用的那个窗口里说谎。**

### 2.2 SupervisorIntent 空状态改为查别处

原文案:

> 没有待运营者判定的验收卡。

**在作用域读返回空时说这句,而实际有三张在别的项目等 —— 比不显示更糟,
因为它主动告诉他没事可做。**

改成:先问其他项目,有卡就说

> 当前项目没有待验收卡,但别处有 **N** 张在等你(project-a, project-b)。
> 左边栏「派单」上的数字就是它们;切换到那个项目可以看到详情。

没有卡时才说原来那句。

## 3. 变异测试:六个,全红

| # | 变异 | 结果 |
|---|---|---|
| 1 | 撤掉 `ElsewherePendingAcceptances`(回到只说没有) | **1 红** |
| 2 | elsewhere 只报数不报项目 | **1 红** |
| 3 | `count` 恒 0 | **1 红** |
| 4 | 计数按 `activeProjectId` 过滤 | **2 红** |
| 5 | 删掉整个计数 | **1 红** |
| 6 | dep 数组改回空 | **1 红** |

## 4. 变异 4 一开始存活,原因值得记

第一版 mock 是:

```ts
listAcceptances: vi.fn(async () => state.acceptances),   // 忽略 projectId
```

**它不管问哪个项目都返回同样的卡。** 于是"按 activeProjectId 过滤"这个变异
不可观测 —— 过滤到错的项目也照样拿到卡。

改成按 `project_id` 过滤,和 Rust 命令一致后,变异 4 立刻被两条测试抓住。

**教训**:一个不按真实语义回答的 mock,会让"问错了对象"这类 bug 完全不可见。
我在这个链上已经第二次遇到同一件事(上一轮 M-4 也是"磁盘快照看不见把原值写回去")。

## 5. 上一单我留下的一条,这单补完了

上一单 Finding 2 的实现**没有测试**,我按纪律没报完成。这单补了两条:

- `a pending acceptance count is visible with the panel closed and regardless of project`
- `the count survives an active project that does not exist in the database`

第二条专门盯 Lyra 量的第三格 —— **活动项目是一个数据库里没有的 id**。

## 6. 未做 / 受阻

- **面板自身的按项目读 bug 我没有改**。`api.listAcceptances(projectId)` 是
  Nimbus 的围栏(我查了 Rust 侧:`project_id: Option<String>` 但
  `require_project_id` 强制非空,所以 api.ts 的签名比后端更严)。
  已报告,未越界。**我做的是在它说谎之前拦住,不是改它。**
- **`projects[0]` 的 fallback 我没有改**。Lyra 说它"也不是 seatloom"。
  改成"承载真实工作的那个"需要产品裁定"哪个项目是默认",我没有这个授权。
  **Finding 4 的告知已经覆盖了这个场景**(会说清从哪切到哪)。
- **推送失败** —— gitlab SSH 22 端口持续超时(重试 3 次),
  origin 一直无权限。提交在本地完整,工作树干净。
- **Finding 3**(跑 `prepare-verification-build.sh` 再开卡)是流程不是代码,
  已在上一单交付说明里报告了我的疏漏。
