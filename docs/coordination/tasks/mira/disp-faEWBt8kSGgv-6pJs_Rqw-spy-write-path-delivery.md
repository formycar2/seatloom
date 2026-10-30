# disp-faEWBt8kSGgv-6pJs_Rqw 交付说明 —— spy 写盘路径,抓住 restore 持久化变异

**派单**:`disp-faEWBt8kSGgv-6pJs_Rqw`(Lyra 发)
**提交**:`3bf314b`
**门禁**:372 tests / 31 files,tsc 净,eslint 135 errors = baseline

---

## 1. Lyra 的变异,复现后抓住

先复现她的结果(没有只信报告):

```
restore 后加 updateProjectUIState(...)  →  12 passed | 0 failed   ← 存活
```

然后装 spy,同一个变异:

```
× switching projects inside a session still restores THAT project's tab
  → expected [ { projectId: 'seatloom', …(1) } ] to deeply equal []
Tests  1 failed | 11 passed (12)                       ← 红
```

**抓它的正是新加的那条断言。**

## 2. 为什么原来抓不到:两个不同的主张

原有测试断言的是 **"tab 被恢复了"**。
 ruling 要的是 **"恢复它的时候没有写盘"**。

**前者可以在后者被违反时通过** —— 这正是 Lyra 的变异演示的事。

## 3. 实现:spy store action,不是磁盘快照

```ts
const realUpdate = useAppStore.getState().updateProjectUIState;
vi.spyOn(useAppStore.getState(), 'updateProjectUIState').mockImplementation(
  (projectId, updates) => {
    restoreFrameWrites.push({ projectId, updates });
    realUpdate(projectId, updates);
  },
);
...
expect(restoreFrameWrites).toEqual([]);   // restore 帧内零写盘
```

**为什么 spy store action 而不是看磁盘**:磁盘级检查看不见"把原值写回去"的
写入 —— 磁盘前后一致。**而这个变异恰恰就是把原值写回去。** "有没有发生写入"
只有在 store action 这一层可判定。

## 4. 测试放在哪一帧:切项目,不是冷启动

**这是实测出来的,不是选的。**

我在上一单就量过:restore 在 `if (!skipColdStart)` 里,而冷启动帧
`skipColdStart` 恒为 true —— **那一行在冷启动时根本不执行**。

所以:

- **冷启动帧**无法承载这条测试( restore 不在那里跑)
- **切项目帧**才是 restore 真正运行的地方,写盘路径必须在它上面盯

测试里两处断言:

```ts
mountApp();
await waitFor(... activeProjectId === RECORDED_PROJECT);
expect(restoreFrameWrites).toEqual([]);        // 冷启动也没写
// ...切到 FIRST_REAL_PROJECT...
expect(restoreFrameWrites).toEqual([]);        // restore 没写
```

## 5. 变异测试:6 个,5 红 1 存活(存活的是测试桩,不是产品)

派单要求"跑你能想到的所有变异,报告存活的"。

| # | 变异 | 结果 |
|---|---|---|
| 1 | **Lyra 原句**:restore 后加 `updateProjectUIState` | **1 红** —— 新测试抓住 |
| 2 | restore 删掉 `setActiveTab`(恢复整个没了) | **2 红** |
| 3 | `setLoadedUIProjectId` 之后无条件写盘 | **4 红** |
| 4 | `chooseTab` 不调 `setActiveTab`(只存盘不换页) | **1 红** |
| 5 | `skipColdStart` 恒 false | **4 红** |
| 6 | spy 自身失效(`mockImplementation` 只记录不调真函数) | **12 passed,存活** |

### 关于 6:我判定它是**测试桩的变异**,不是产品缺陷

变异 6 把 `realUpdate(projectId, updates)` 从 mock 里删掉,只留记录。
测试仍然全绿。

**原因,而且这是有意的**:这条测试断言的是"**有没有调用写盘路径**",
不是"写盘路径做得对不对"。删掉真函数的调用不影响前者。

后者(写盘路径做得对)由其余 372 条测试覆盖 —— 包括上一单那条
`an operator-driven tab change IS persisted`,它断言磁盘上真的出现了
`workitems` 且 `relaunch()` 读得回。

**如果我把真函数调用也变成断言的必需条件,这条测试就同时在测两件事,
而其中一件别处已经测了。** 我按"一条测试一个主张"处理,并在派单报告里
说明这一条存活的变异是什么性质。

## 6. 我自己犯的一个错,必须报告

跑变异 6 时,我用 `git checkout src/App.restartRestore.test.tsx` 恢复文件,
**把当时未提交的 spy 测试整个冲掉了。**

这正是我自己纪律里写过的:**未提交的新文件不许用 `git checkout` 恢复,
要用备份文件。** 我写了那条纪律,这次违反了它。

已重做(spy 测试重建,`grep -c restoreFrameWrites` = 7),并改用
`cp /tmp/tr.spy.bak` 恢复。**重建后立即提交,不再让它处在未提交状态。**

## 7. done_definition 逐条对

| # | 要求 | 实测 |
|---|---|---|
| 1 | restore 持久化时至少一条测试红 | ✅ **变异 1:1 红**,新测试 |
| 2 | 代码如提交时整个契约文件绿 | ✅ `12 passed (12)` |
| 3 | 新测试断言写盘路径**未被调用**,不只是 tab 被恢复 | ✅ spy `updateProjectUIState`,`toEqual([])` |
| 4 | 全套绿 / typecheck 净 / eslint 不劣于 baseline | ✅ 372/31,tsc 净,135 errors |
| 5 | 跑所有能想到的变异,报告存活的 | ✅ 6 个,5 红 1 存活(见 §5,存活的是测试桩) |

## 8. 仍未闭环

- **没开验收卡** —— `disp-E06O6IAaWefF-BOW-CNJd` 仍无 verdict 点名它。
- **真机验证未做**。
- **两个远端都不通** —— gitlab SSH 22 超时,origin 无权限。提交在本地 `3bf314b`。
