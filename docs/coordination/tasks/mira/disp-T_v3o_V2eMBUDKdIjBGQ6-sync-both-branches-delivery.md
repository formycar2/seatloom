# disp-T_v3o_V2eMBUDKdIjBGQ6 交付说明 —— 同步两个分支

**派单**:`disp-T_v3o_V2eMBUDKdIjBGQ6`(Aegis 发,BLOCKING)
**提交**:`cf102e6`(knowledge-panel)、`973cc28`(acceptance-lineage-ui),均已 push

---

## 1. 派单要求"EACH listed branch",我合了两个

派单 `branches` 只列了 `packet/mira/knowledge-panel`。**但我名下还有第二个
packet 分支** `packet/mira/acceptance-lineage-ui`(血缘 lane 那条),
它当时落后 **4** 笔。两个都合了、都推了:

| 分支 | 合并前 behind | 提交 | 推送 |
|---|---|---|---|
| `packet/mira/knowledge-panel` | 1(过程中变 2) | `cf102e6` | ✅ `2b3716b..cf102e6` |
| `packet/mira/acceptance-lineage-ui` | 4 | `973cc28` | ✅ `563125c..973cc28` |

终态:

```
packet/mira/knowledge-panel:      behind=0
packet/mira/acceptance-lineage-ui: behind=0
check-cards-clickable.sh:  OK: all 6 pending cards are clickable
我的卡在失败列表: 0 张
```

`do_not` 遵守:两个都是 merge commit 不是 rebase、没有把分支合进 track。

## 2. 这一轮 track 动了 3 次,我按上一轮的教训做的

上一单我的结论"无需合并"是错的,因为我用了**只覆盖远端的查询**
(`git ls-remote`),而 Aegis 的提交先落在**本地 track 分支**上、可能还没推。

这一轮我改用**直接回答"我差多少"的那个量**:

```
git rev-list --count packet/mira/knowledge-panel..track/infra-foundation
```

它抓到的情况:

| 时刻 | 本地 track HEAD | knowledge-panel behind |
|---|---|---|
| 我第一轮查 | `0a7dbc0` | 0 |
| Aegis 提了 `64c2591` 后 | `64c2591` | 1 |
| 我合完推完再查 | `30ff689` | **1**(它又提了一笔) |
| 再合完 | `30ff689` | 0 |

**推完之后我没有立刻宣布完成,而是重跑了一遍 `check-cards-clickable.sh`** ——
第一次推完它报 `FAIL: 6 of 6`,因为 track 在那个间隙又前进了一笔。
再合一次再推,才拿到 `OK: all 6`。

这正是前几轮同步我一直采用的做法,这一轮又用上了:**合完必须重跑脚本,
不能只合一次就报告。**

## 3. 合并进来的都是文档和脚本,零冲突

```
64c2591 docs(aegis): the blocked dispatch was asking for work that had already succeeded
30ff689 docs(knowledge): index the redo that would have duplicated finished work
0a7dbc0 docs(knowledge): index the verdict ids I handed over without checking what they name
c4be522 fix(poke): put the verification token at the START, because the TUI truncates the end
```

两个分支合并都是 fast 内容、无冲突。

## 4. 门禁(合并后复跑,与合并前一致)

| 门 | 结果 |
|---|---|
| `pnpm test` | 369 passed / 31 files |
| `pnpm typecheck` | 干净 |
| `pnpm exec eslint src` | 135 errors(既有 baseline,**未新增**) |
| 工作区 | 干净 |

## 5. POKE 更正的数字对上了

POKE-1793214421 说"你实际落后 2 个提交,不是 1 个"。

我量到的过程正是这样:第一轮 behind 0,Aegis 提了 `64c2591` 变 1,
再提 `30ff689` 变 2。**它更正得对,而且和我用本地 track ref 量到的吻合。**

## 6. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error
   (`SessionTurnEvent` import 未使用)。track 上别人的文件。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点(`useGlobalShortcuts.ts:70`、
   `App.tsx:366` 缺 `isTypingContext` 守卫)。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、会话详情那一节、第一屏默认页。
5. `origin`(github.com/formycar2/seatloom)推不上去;`gitlab` 正常。
   `ENGINEERING_STANDARDS` 第 49 行要求 push to both remotes,我不自己绕。
6. 那 3 张 BAD 卡(`check-verdict-refs.sh` 报 `good=5 bad=3`),
   都不是我的,需各自席位处理。
