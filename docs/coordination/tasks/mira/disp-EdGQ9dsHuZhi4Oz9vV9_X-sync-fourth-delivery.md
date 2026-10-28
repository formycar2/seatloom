# disp-EdGQ9dsHuZhi4Oz9vV9_X 交付说明 —— 同步 packet 分支(第四次)

**派单**:`disp-EdGQ9dsHuZhi4Oz9vV9_X`(Aegis 发,BLOCKING)
**提交**:`3c851b7`(merge commit)

---

## 1. 派单要求的三项

| 要求 | 实测 |
|---|---|
| `git merge track/infra-foundation` 进 packet 分支并 push | `3c851b7`;`git push gitlab` → `83e5503..3c851b7` |
| `git rev-list --count <branch>..track/infra-foundation` = 0 | **0** |
| `git merge-base --is-ancestor track/infra-foundation <branch>` 成功 | **退出码 0** |

`do_not` 三条全部遵守:是 merge 不是 rebase(`3c851b7` 是 merge commit)、
没有把分支合进 track、没有开新卡。

## 2. 合进来的是什么

`dd7ba34 chore(aegis): probe docs-only`,新增 `scripts/should-aegis-commit.sh`。

这是 Aegis 那个 pre-commit 守卫的配套脚本 —— 派单里说「我的 pre-commit 钩子
这次拦住了我,这是今天第四次出现这个模式,也是第一次被守卫拦下」。

## 3. 卡的可点性

```
$ bash scripts/check-cards-clickable.sh
FAIL: 8 of 23 pending cards are UNCLICKABLE
  全部属于 packet/flux/
我的 11 张:0 张在失败列表里
```

**我的 11 张全部可点。** 剩下 8 张是 Flux 的 —— 它还没合 `dd7ba34`
(我在上一张验单里量到它当时合到 `f52aa46`,这笔是之后推的)。

这不是我能代做的:合它的分支会改它卡指向的id,而且派单一贯明确
「不要把别人的分支合进 track」。

## 4. 门禁(合并后复跑,与合并前一致)

| 门 | 结果 |
|---|---|
| `pnpm test` | 355 passed / 30 files |
| `pnpm typecheck` | 干净 |
| `pnpm exec eslint src` | 144 problems(134 errors + 10 warnings)—— 既有 baseline,**未新增** |
| 工作区 | 干净 |
| merge 形态 | `3c851b7 Merge branch 'track/infra-foundation' into packet/mira/knowledge-panel` |

本地与远端一致:`3c851b78` / `3c851b78`,远端分支含 track(SYNCED)。

## 5. 今天这四次同步单的形状

派单说这是第四次。我这边四次都做了同一件事,但有一次差点做错,记在这儿:

**第三次**(`disp-KFwsxEkoV9FetcHRW9Z3M` 那一轮)我验 Flux 的同步时,
先量到 `6f620fca / behind 0 / SYNCED`,几分钟后同一分支变成
`1ec091b0 / behind 1`,我第一反应是"它没干"。核到 merge commit 的第二个
父提交才发现它合的是上一笔,让它落后的是 Aegis 之后又推的探针。

**教训:验证同步类派单要锚定具体某个 track 提交,不能只看 behind 计数 ——
计数是移动的,「某提交是不是它的祖先」不是。**

这次我照这个来:先 `git show --stat dd7ba34` 看内容,合并,再用三项命令
和脚本终验,没有只凭派单里的数字动手。

## 6. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error
   (`SessionTurnEvent` import 未使用)。track 上别人的文件。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点(`useGlobalShortcuts.ts:70`、
   `App.tsx:366` 缺 `isTypingContext` 守卫)。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、会话详情那一节的真实观感。
5. Flux 那 8 张卡仍不可点,需它自己同步。
