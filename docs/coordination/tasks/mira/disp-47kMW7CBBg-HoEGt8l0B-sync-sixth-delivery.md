# disp-47kMW7CBBg-HoEGt8l0B- 交付说明 —— 同步(第六次)

**派单**:`disp-47kMW7CBBg-HoEGt8l0B-`(Aegis 发,BLOCKING)
**提交**:`5020bf4`、`5874b18`(两个 merge commit)

---

## 1. 派单要求的两项

| 要求 | 实测 |
|---|---|
| `git merge track/infra-foundation` 进 packet 分支并 push | `5020bf4` + `5874b18`;push → `3c851b7..5874b18` |
| `git rev-list --count <branch>..track/infa-foundation` = 0 | **0**(合并后复查两次,仍为 0) |

`do_not` 遵守:是 merge 不是 rebase、没有把分支合进 track。

## 2. 这一轮我合了**两次**,因为派单说"我要提一笔"而它提了两笔

派单 `measured_by_aegis_on_10_28_1505` 说 `behind_by: 0`,并说明它**正要**提交。
我 15:05:53 量到落后 1 个(`c01277d`),合并、推送。

推送后终验时发现**又落后 1 个**(`ab44408`)—— Aegis 在间隙又推了第二笔
(它派单里提到的那笔的两个组成部分:脚本 + wiring,以及随后的知识库索引)。

**我没有在第一次推送后就宣布完成。** 第二次合并后复查 `behind = 0`,且
track 已静止,才终验。

这正是前几次同步我采用的做法:合完必须重跑检查脚本,不能只合一次就报告。

## 3. 卡的可点性

```
$ bash scripts/check-cards-clickable.sh
FAIL: 1 of 28 pending cards are UNCLICKABLE
  属于 packet/vela/
我的 11 张:0 张在失败列表里
```

**从 28 张里只剩 1 张点不了,而且不是我的。** 之前几轮是 12 张、8 张、
4 张挂,那些席位各自同步后数字一路降下来。

## 4. 合进来的东西正好是我报告过的问题的机械化检查

派单 `what_i_am_committing` 说:「a new heartbeat check
(scripts/check-verdict-refs.sh)… The check found a verdict whose dispatch_id
pointed at 0 rows - one character wrong, 0 vs o in a nanoid alphabet.」

**这正是我几个回合前报告的形状。** 我跑了那个新脚本:

```
== pending acceptance cards: is each verdict ref a verdict about THIS dispatch? ==
  good=28 bad=0

== ReviewVerdictIssued events that name a dispatch which does not exist ==
  ev-030                       EMPTY  names nothing (seat:seat-lyra-001)
  ev-031                       EMPTY  names nothing (seat:seat-lyra-001)
  ev-XBrImD2cxJfuEGtgf7go2     EMPTY  names nothing (seat:seat-flux-001)
  ev-rKtx-DQARUF142u6lN3x5     EMPTY  names nothing (seat:seat-flux-001)
  ev-usP7dX7geulRhaDy1eXwD     WILD   names disp-B4Y5oAhMQwtXm5Vva1fw which has 0 rows (seat:seat-flux-001)

  5 verdict(s) point at nothing. They are not evidence for anything.
```

两个事实值得记:

1. **`good=28 bad=0`** —— 所有 pending 卡的 verdict ref 都正确点名了各自的派单。
   这说明 10-27 那条裁决(verdict 必须点名被接受的派单)在**已开的卡**上是干净的。
2. **5 条 verdict 指向空** —— 3 条什么都没点名,1 条点名了不存在的派单
   (一字符之差:`0` vs `o`,nanoid 字母表里两个都合法)。它们**不是任何东西的证据**,
   但目前没有被任何 pending 卡引用。

**这和我报告的十件是同一族但不同层**:我那十件是"verdict 点名了另一个**存在**的派单",
会被误用;这 5 条是"点名了**不存在**的派单",更容易被发现但也更危险 ——
脚本自己说了:「a future dispatch with one of those ids would inherit them」。

## 5. 门禁(合并后复跑,与合并前一致)

| 门 | 结果 |
|---|---|
| `pnpm test` | 355 passed / 30 files |
| `pnpm typecheck` | 干净 |
| `pnpm exec eslint src` | 144 problems(134 errors + 10 warnings)—— 既有 baseline,**未新增** |
| 工作区 | 干净 |
| merge 形态 | `5874b18` 和 `5020bf4` 都是 merge commit |

本地与远端一致:`5874b18a` / `5874b18a`,远端分支含 track(SYNCED)。

## 6. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error
   (`SessionTurnEvent` import 未使用)。track 上别人的文件。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点(`useGlobalShortcuts.ts:70`、
   `App.tsx:366` 缺 `isTypingContext` 守卫)。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、会话详情那一节的真实观感。
5. Vela 那 1 张卡仍不可点,需它自己同步。
6. 那 5 条指向空的 verdict —— 已由 Aegis 的新脚本持续监控,不是我的改动范围。
