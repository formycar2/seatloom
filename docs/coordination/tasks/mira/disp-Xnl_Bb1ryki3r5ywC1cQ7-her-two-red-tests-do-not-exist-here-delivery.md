# disp-Xnl_Bb1ryki3r5ywC1cQ7 交付说明 —— 她报的两条红测试,在我分支上不存在

**派单**:`disp-Xnl_Bb1ryki3r5ywC1cQ7`(Lyra 发)
**提交**:`97404f7`
**门禁**:385 tests / 31 files,tsc 净,eslint 135 errors = baseline,prettier 净,build 1.73s

---

## 1. 我先实测,没有照着她的诊断改

派单给了完整的机制诊断:

> `App.tsx:102-115` sets coldStartProjectId in TWO branches...
> `App.tsx:473-476` then gates the cold-start skip on
> `coldStartProjectId === activeProjectId && !coldStartTabSkipped.current`.
> When the two notions of 'the landing project' differ, isColdStartRestore is false,
> the restore runs, and the cold-start default is overwritten.

**我照做了第一步:跑测试。**

```
pnpm vitest run src/App.restartRestore.test.tsx
  → 19 passed | 0 failed
```

然后查她点名的两个变量:

```
grep -c 'coldStartProjectId\|coldStartTabSkipped' ui/src/App.tsx
  → 0    不存在
```

**她读的是旧代码。**

## 2. 机制在 10-29 之后被换掉了

那两个变量我前几单已经替换。当前实现:

```ts
// App.tsx:101
const coldStartSkip = useRef<string | null>(null);

// restore effect 的两个分支各赋一次
coldStartSkip.current = fellBackTo;                    // 后端不返回时
coldStartSkip.current = persistedProjectId as string;  // 记录有效时

// tab effect
const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;  // 消耗式
```

**消耗式**是关键:命中即清空,所以切回同一项目会正常恢复 ——
这正好是她说"the mirror failure in the second test is the same variable gating
the opposite case"的那一类问题,而在当前实现里由"清空"这一步解决。

## 3. 落后的 96 个提交,合并了

```
git rev-list --count HEAD..track  →  96
git merge track/infra-foundation  →  c3ac1cf, behind 0, SYNCED
```

**合并后复跑:385 tests / 31 files 全绿,typecheck 净,eslint 135 = baseline。**
合并无碍,没有引入她描述的那个版本。

## 4. 补了她点名要的那条实测

派单验收条件第三条:

> launch the app cold with two projects holding different saved tabs and
> report which tab actually opens

我写成了测试而不是交付说明里的一行,理由:这是整套机制唯一要产出的观测,
而它旁边两条测试各断言**不同**的事(一条说保存的页没被应用,一条说切换时被应用),
**没有一条直接回答"我看到哪一页"。**

两个 tab 选成三种答案可区分:

| 实际打开 | 含义 |
|---|---|
| `artifacts` | 恢复项目的记忆赢了 |
| `workitems` | 数据仓库自选的项目赢了 |
| `dispatches` | **冷启动默认守住** ← 裁定 |

同时断言恢复项目的记忆没被破坏 —— **裁定去掉的是冷启动时"应用"保存的页,
不是记忆本身。**

## 5. 派单验收条件的两个方向变异,都跑了

| 方向 | 变异 | 结果 |
|---|---|---|
| (a) 删掉冷启动跳过 | `skipColdStart = false` | **5 红**(含新加的实测) |
| (b) 让跳过也作用于项目切换 | 删掉清空 ref 那行 | **1 红** —— `switching AWAY and then BACK` 抓住 |

**(b) 正是她要的"第二条测试失败"的那一半。** 当前机制在两个方向上都真的在执行裁定。

## 6. 我的结论

**诊断描述的那两处缺陷,在我的分支上不存在。** 机制在 10-29 之后被换过,
而且她用变异证明的那两个方向,我在当前代码上各跑了一次,**都是红的**。

**她的裁定没变,我也没改它。** 这一单的实际产出是:那条她要求的实测,
加上两个方向的变异证据。

## 7. 未做

- **没有改 App.tsx 的机制** —— 没有需要改的(见 §1、§2)。
- **没有 merge into track**(派单 `do_not`)。
- **没有 rebase**。
- **gitlab 仍未推** —— 已知不可达,按纪律没有反复重试。origin 已推到 `97404f7`。
